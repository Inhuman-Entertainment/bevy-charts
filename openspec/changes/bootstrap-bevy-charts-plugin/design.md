## Context

The repository was empty apart from a README, LICENSE, and `.gitignore`. Bevy
0.19 is current, and its API has moved substantially since the widely-published
tutorials: required components replaced bundles, `AmbientLight` became a
per-camera component, `EventWriter` became `MessageWriter`, and
`DirectionalLight::shadows_enabled` became `shadow_maps_enabled`. Every API used
here was verified against the vendored 0.19 sources rather than recalled.

The brief asked to survey 2D Rust plotting libraries for anything worth reusing,
follow Bevy conventions, and prefer reuse over novel code.

## Goals / Non-Goals

**Goals:**

- Charts are ordinary components; spawning one is the whole API.
- Data changes rebuild geometry automatically, with no manual teardown.
- Reuse existing crates for solved problems (tick placement, primitive meshes).
- Keep the dependency surface small enough to embed in a game.
- Colors that survive colorblind viewers by construction, not by taste.

**Non-Goals:**

- Text rendering in 3D (labels, legends, tick numbers) — deferred to a follow-up.
- Being a general plotting library. This targets in-game and in-editor 3D charts.
- Matching the feature breadth of a 2D charting library.

## Decisions

### Charts are components; a system rebuilds children on change

Each chart type is a `Component` with `#[require(Transform, Visibility, ChartSize,
ChartPalette, AxisStyle)]`. A `PostUpdate` system queries
`Or<(Changed<TheChart>, Changed<ChartSize>, Changed<ChartPalette>, Changed<AxisStyle>)>`,
despawns the chart's children, and respawns them.

Rebuilding wholesale rather than diffing is the right trade here: chart geometry
is cheap to regenerate, and diffing would add a lot of bookkeeping for a case
that is usually "the data changed completely". The cost is real for per-frame
mutation, which the docs and examples steer away from.

### Reuse: `plotters` for tick placement only

Surveyed `plotters`, `charming`, `charton`, and `kuva`. `plotters` is the only
one whose useful part is separable: `Ranged::key_points` implements the standard
1/2/5-times-a-power-of-ten "nice numbers" search. With `default-features = false`
it costs three small crates (`plotters-backend`, `num-traits`, `autocfg`) and
brings no rendering backend.

Everything else in these libraries is 2D-vector-oriented — SVG paths, rasterized
marks — and cannot produce 3D meshes.

### Reuse: two shared primitive meshes for all marks

Bars, line segments, and scatter markers are a shared unit cube or unit sphere
placed by a `Transform`. A chart of a thousand points holds two mesh assets. Only
the surface chart generates geometry, because a height field genuinely needs it.

Line segments are drawn as thin boxes rather than GPU line primitives so they
take scene lighting and hold a constant world-space thickness. The axis and grid
*do* use `LineList`, because a recessive reference grid wants constant pixel
width.

### Materials are cached by color

Charts rebuild on every data change. Minting a `StandardMaterial` per bar per
rebuild would grow the asset table without bound, so `MaterialCache` keys handles
by linear-color bits plus a lit/unlit flag. Meshes need no such cache: they hang
off despawned child entities and are released by refcount.

### Color: fixed slot order, clamped, not cycled

The palette is a validated categorical set where the *slot ordering* is the
colorblind-safety mechanism — adjacent slots clear a CVD separation floor.
Consequences encoded in the API:

- `series_color` clamps past the last slot instead of wrapping. A ninth series
  would repeat a hue, and a chart that repeats a hue is lying about identity.
- Scatter plots compare non-adjacent marks and so need pairwise separation, which
  only the first three slots guarantee. Documented on `ScatterChart3d`.

### Bars measure from zero; lines do not

`bar_value_scale` always includes zero because a bar encodes length. Line charts
default to the data range, since a series varying in a narrow band far from the
origin would otherwise be flattened; `begin_at_zero` opts in.

### Local space is `0..size`

Every chart draws into a box with its origin at the bottom-left-front corner: x
categories, y values, z series. Placement is then just a `Transform`, and charts
compose with the rest of a scene without special cases.

### `charton`: an optional feature later, not a core dependency

`charton` 0.5.9 has a genuine grammar-of-graphics split, and its public
`ScaleTrait` (`normalize(f64) -> [0,1]`, `normalize_string`, `domain`) is a
better abstraction than the hand-rolled `Scale` here — it would bring log, band,
and time scales.

It is not adopted as a core dependency now because:

- It is entirely 2D. Its `mark` and `render` layers — the bulk of the crate —
  cannot produce 3D meshes.
- The statistics most worth borrowing (`stat_binning`, `stat_loess`) are
  `pub(crate)` and unreachable from outside.
- Even with `default-features = false` it always pulls `ahash`,
  `csscolorparser`, `html-escape`, `kernel-density-estimation`, `thiserror`, and
  `time` — dead weight inside a game.
- At 0.5.x with a single maintainer, making it the core would couple this crate's
  public API to one still in flux.

Planned instead: an optional `charton` feature that adapts `ScaleTrait` to this
crate's `Scale` (unlocking log/time/band axes) and powers extension chart types
such as boxplot and density. Cost today: zero.

## Risks / Trade-offs

- **No text labels.** This is the largest gap. `ChartData::labels` and
  `Dataset::label` are carried through the data model but not drawn, so charts
  currently rely on external UI for identification. Billboarded `Text2d` in a 3D
  scene needs its own design pass.
- **Rebuild cost.** A chart with many marks respawns every child on any change.
  Fine at the sampling rates the examples use; a per-frame 10k-point chart would
  need incremental updates.
- **Bevy API churn.** Bevy breaks its API every release, and this crate touches
  meshes, materials, hierarchy, and visibility. Expect a version bump per Bevy
  release; the compatibility table in the README is the contract.
- **Palette clamping can surprise.** A caller passing nine series silently gets
  two identical colors at the tail. Clamping is the lesser evil versus cycling,
  but it is a real sharp edge until legends exist to make it obvious.
