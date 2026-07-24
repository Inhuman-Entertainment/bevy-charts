## Context

The library renders charts as lit 3D geometry inside a Bevy scene. Everything it
draws today is a mesh: bars and markers are shared primitives placed by a
`Transform`, axes and grid are line lists, the surface is generated. Nothing
draws text, so `ChartData::labels` and `Dataset::label` are dead weight in the
data model.

Bevy 0.19 offers two workable routes to text in a 3D scene, and they produce
materially different libraries. This was put to the maintainer as an explicit
choice; **world-space geometry was selected**.

## Goals / Non-Goals

**Goals:**

- A chart can be read without any surrounding UI.
- Labels behave like the rest of the scene: they depth-test, occlude, and move
  with the chart's `Transform`.
- No second camera, no render layers, no per-frame screen projection.
- Labels are optional and cheap to turn off.

**Non-Goals:**

- Decluttering overlapping labels. Out of scope, and noted in the proposal.
- Matching the typographic quality of a 2D charting library. Text here is a
  texture on a quad; it will soften when the camera is very close.
- Screen-space labels in this change.

## Decisions

### Labels are world-space glyph meshes

Each label becomes one `Mesh` of textured quads — one quad per glyph — with the
font atlas as `base_color_texture` on an unlit, alpha-blended `StandardMaterial`.

The data needed is already public. `TextLayoutInfo::glyphs` is a
`Vec<PositionedGlyph>`, and each glyph carries:

- `position: Vec2` — where the glyph sits within the laid-out text block,
- `atlas_info.texture: AssetId<Image>` — which atlas image it landed in,
- `atlas_info.rect: Rect` — its pixel bounds in that atlas, which become UVs.

So a label is a straightforward transformation from that list into positions and
UVs. One mesh and one draw call per label, and it renders through the existing
PBR pipeline with no new machinery.

The alternative — projecting anchors with `Camera::world_to_viewport` and
positioning Bevy UI nodes — gives crisper text for much less work, but the labels
would always draw on top, never occlude, and would not survive a chart that is a
physical object in the world. That conflicts with what the library is for. It
remains a sensible future `LabelMode` for debug and analytics views.

### Bevy does the layout; we only consume it

Text layout is genuinely hard — shaping, kerning, line breaking, atlas packing —
and `TextPipeline::update_text_layout_info` is public but low-level: it wants a
`ComputedTextBlock` that has already been through `update_buffer`, plus a
`FontAtlasSet`, a `ScaleCx`, and bounds.

Preferred approach: spawn a hidden `Text2d` entity per label and let Bevy's own
systems populate its `TextLayoutInfo`, then read that component and build the 3D
mesh from it. We never drive the pipeline, we consume the component Bevy already
computes, and we inherit shaping and atlas management for free.

**This is the main risk in the change** — see below.

### Two orientations, billboard by default

```
LabelOrientation::Billboard   // turns to face the camera (default)
LabelOrientation::InPlane     // fixed to the chart's axis planes
```

Billboard is the default because readability is the entire point of adding
labels. `InPlane` exists because a chart meant to read as a physical object — a
holographic readout, a prop on a desk — looks wrong with text that swivels.

Billboarding needs the camera's rotation, so it is a system running after
transform propagation, updating only label entities.

### Size is in world units

Everything else in the library is sized in world units inside the chart's box, so
labels are too: a `size` in world units per line of text, not a pixel size. The
glyph mesh is scaled from the atlas's pixel metrics to that world height. This
keeps a chart's appearance independent of window resolution, which a game needs.

The consequence is that labels do not stay a constant size on screen as the
camera moves. That is correct for world-space geometry and is the trade being
made.

### The font is the caller's, with a default

A `LabelStyle` carries an `Option<Handle<Font>>`. When it is `None` the crate
falls back to Bevy's built-in font if the `default_font` feature is enabled, and
otherwise draws no text at all — the chart still renders its geometry. A charting
library must not panic or render nothing because a game has not set a font.

### What gets labelled

- **Value ticks** — at the positions `Scale::ticks` already computes for the
  grid, so labels and grid lines agree by construction.
- **Categories** — from `ChartData::labels`, centred on each category slot.
- **Legend** — one entry per dataset, from `Dataset::label`, with a colour swatch
  reusing the palette slot. This is what makes the colour encoding legible; a
  multi-series chart without one is asking the viewer to guess.
- **Title** — optional, above the chart box.

## Risks / Trade-offs

### Hidden `Text2d` may not lay out without a 2D camera

`update_text2d_layout` queries `(&Camera, &VisibleEntities, Option<&RenderLayers>)`
to resolve scale factors. It is not established that it will process an entity
when the only camera in the scene is a `Camera3d`, or whether a hidden entity is
laid out at all.

**This must be settled by a spike before anything else is built.** If it does not
work, the fallback is to drive `TextPipeline` directly, which is more code — we
would own `ComputedTextBlock` and `FontAtlasSet` handling — but uses only public
API and removes the dependency on `bevy_sprite` behaviour entirely. The spike
decides between the two; the rest of the design is unaffected either way.

### Alpha-blended text sorts by distance

Alpha-blended geometry is drawn back to front by object, not per pixel. Labels
that intersect each other, or intersect transparent chart geometry, can sort
wrongly. Mitigation: labels are thin and rarely intersect; if it bites, alpha
masking (`AlphaMode::Mask`) trades soft edges for order independence.

### Text is a texture

Close to the camera, labels soften. This is inherent to atlas text and is the
price of it being real geometry. Callers who need crisp text at any zoom want the
screen-space mode, which is why that idea is recorded rather than discarded.

### Dependency growth

The crate deliberately depends on `bevy` with `default-features = false` and only
three features. Labels add `bevy_text` and `bevy_image`, and the examples will
want `default_font`. That is a real increase for a game to carry, and it is the
reason labels are optional at runtime — but the dependency is not optional at
compile time. Putting the whole label module behind a `labels` cargo feature,
default-on, is worth considering during implementation.

### Labels will overlap

With many categories, or a small chart, labels will collide and become
unreadable. There is no decluttering in this change. The honest mitigation is
documentation: say so, and give callers the knobs (draw every *n*th category,
turn labels off, make the chart bigger) rather than pretending it is solved.
