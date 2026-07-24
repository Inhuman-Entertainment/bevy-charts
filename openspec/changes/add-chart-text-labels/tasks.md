## 1. Spike: how do we get a glyph layout?

Nothing else can be designed around until this is settled. Timebox it, and record
the answer in `design.md` before moving on.

- [ ] 1.1 Add the `bevy_text` and `bevy_image` features and confirm the crate
      still builds with `default-features = false`
- [ ] 1.2 Spawn a hidden `Text2d` entity in a scene whose only camera is a
      `Camera3d`, and check whether `TextLayoutInfo` is populated
- [ ] 1.3 If it is not, try it with `Visibility::Visible` but off-screen, to
      separate "hidden entities are skipped" from "needs a 2D camera"
- [ ] 1.4 If neither works, prototype driving `TextPipeline` directly:
      `update_buffer` then `update_text_layout_info`, owning the
      `ComputedTextBlock` and `FontAtlasSet`
- [ ] 1.5 Record which route was taken and why, then update `design.md`

## 2. Glyph mesh construction

- [ ] 2.1 `src/label.rs` — turn a `TextLayoutInfo` into a `Mesh` of one quad per
      glyph, with positions from `PositionedGlyph::position` and UVs from
      `atlas_info.rect`
- [ ] 2.2 Scale from atlas pixel metrics to a world-unit line height
- [ ] 2.3 Horizontal and vertical anchoring, so a label can be centred on a
      category, right-aligned against the value axis, or sat above the box
- [ ] 2.4 Unlit, alpha-blended material carrying the atlas texture, cached per
      (atlas, colour) like the existing material cache
- [ ] 2.5 Unit tests: quad count matches glyph count, UVs stay inside `0..1`,
      an empty string produces no mesh

## 3. Styling and configuration

- [ ] 3.1 `LabelStyle` component: which kinds are on, world-unit size, colour
      override, orientation, optional font handle
- [ ] 3.2 Default colour derived from `ChartPalette` so dark and light stay legible
- [ ] 3.3 Add `LabelStyle` to every chart's `#[require(...)]` set
- [ ] 3.4 Fall back to the default font when the feature is on; draw no text and
      do not panic when there is no font at all

## 4. Placement

- [ ] 4.1 Value tick labels at `Scale::ticks` positions, formatted from the value
- [ ] 4.2 Category labels centred on each category slot, from `ChartData::labels`
- [ ] 4.3 Legend entries with colour swatches, from `Dataset::label`
- [ ] 4.4 Optional chart title above the box
- [ ] 4.5 Wire placement into each chart's build system, sharing one helper

## 5. Orientation

- [ ] 5.1 `LabelOrientation::{Billboard, InPlane}`, billboard by default
- [ ] 5.2 Billboard system running after transform propagation, touching only
      label entities
- [ ] 5.3 Confirm billboarding is correct when the chart entity is itself rotated

## 6. Verification

- [ ] 6.1 Integration tests in the existing headless harness: labels are created,
      rebuilt on data change, left alone when unchanged, and absent when disabled
- [ ] 6.2 Test that a chart with no font still renders its geometry
- [ ] 6.3 Test that disabling every label kind creates no label entities
- [ ] 6.4 Render the showcase and read it — the one thing no test can do. Check
      occlusion behind bars and legibility while orbiting.
- [ ] 6.5 Regenerate `assets/showcase.png`

## 7. Documentation

- [ ] 7.1 README: labelling section, and remove text labels from "Not yet
      implemented"
- [ ] 7.2 Document the overlap caveat honestly, with the knobs callers have
- [ ] 7.3 Note the added dependency surface and any `labels` cargo feature
- [ ] 7.4 Update `ROADMAP.md`
