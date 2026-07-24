## Why

The repository was an empty shell. PR #1 (`copilot/setup-rust-bevy-plugin`) was
merged while still marked `[WIP]`: it contained a single empty "Initial plan"
commit and zero files, so the merge produced nothing. The plan itself — a Bevy
0.19 charting plugin reusable across several games — survived only in the PR body.

This change implements that plan, and establishes OpenSpec as the place where the
remaining work is tracked so the next gap is visible rather than silent.

## What Changes

- A `bevy_charts` library crate targeting Bevy 0.19, following Bevy plugin
  conventions (component + system + plugin, required components, `prelude`).
- A Chart.js-shaped data model shared by every chart type.
- Five chart types: bar, line, scatter, surface, histogram.
- A validated categorical color palette with dark and light scene variants.
- Axis and grid geometry with tick placement delegated to `plotters`.
- Five runnable examples and a README with installation and usage docs.

Explicitly deferred, and recorded here so they are not mistaken for oversights:
text labels, stacked bars, log/time axes, and picking.

## Capabilities

### New Capabilities

- `chart-core`: the data model, value scales, and the component/system lifecycle
  that turns a chart component into rendered geometry.
- `chart-types`: the five concrete chart types and the geometry each produces.
- `chart-styling`: series colors, palette safety rules, and axis/grid appearance.

### Modified Capabilities

None — this is the first change in the repository.

## Impact

- New crate: `bevy_charts` (`src/`, `examples/`, `Cargo.toml`).
- Dependencies: `bevy` 0.19 (`default-features = false`, only `bevy_asset`,
  `bevy_color`, `bevy_pbr`) and `plotters` 0.3 (`default-features = false`).
- `README.md` replaced; `assets/showcase.png` added.
- MSRV 1.95, edition 2024.
