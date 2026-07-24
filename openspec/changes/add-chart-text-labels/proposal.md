## Why

Charts cannot currently identify themselves. `ChartData::labels` and
`Dataset::label` are carried all the way through the data model and then never
drawn, so a bar chart renders as anonymous coloured boxes: no tick values, no
category names, no legend. Anything using the library today has to build its own
UI alongside the chart to say what it is looking at.

This is the largest functional gap in the library and the one thing that most
separates it from being usable by another game.

## What Changes

Text rendered as **world-space geometry**, not as a screen overlay. A label is a
mesh of textured quads built from Bevy's glyph atlas, so it depth-tests and
occludes like everything else in the scene: a label behind a bar is hidden by
that bar. This follows from what the library is for — charts that are objects in
a game world, not an analytics panel bolted over the viewport.

New capability:

- Value-axis tick labels, positioned at the same round numbers the grid lines use.
- Category labels along the category axis, from `ChartData::labels`.
- A series legend, from `Dataset::label`.
- An optional chart title.
- Billboarding, so labels turn to face the camera, with an in-plane alternative
  for charts meant to read as physical objects.
- Control over which of the above are drawn, their size in world units, and
  their colour.

Deliberately **not** in this change:

- Automatic decluttering when labels overlap. Charts with many categories will
  collide; callers control this by drawing fewer labels or making the chart
  bigger. Doing it properly needs screen-space measurement and is its own change.
- Rich text, multiple fonts per chart, or right-to-left scripts.
- Screen-space overlay labels. A reasonable future `LabelMode`, but a different
  thing with different trade-offs; see `design.md`.

## Capabilities

### New Capabilities

- `chart-labels`: what text a chart draws, where it is placed, how it is
  oriented, and how it is styled.

### Modified Capabilities

- `chart-styling`: gains label appearance alongside the existing palette and
  axis styling, and the requirement that a chart with no font configured still
  renders its geometry rather than failing.

## Impact

- New module `src/label.rs`, plus label placement in each chart's build system.
- New dependency surface: the `bevy_text` and `bevy_image` features of `bevy`,
  which the crate has so far avoided. `default_font` is **not** required — a
  font can be supplied by the caller — but the examples will want it.
- `AxisStyle` and/or a new `LabelStyle` component gains fields; existing charts
  keep working because labels default to on with sensible values.
- Examples and the showcase render gain readable labels; `assets/showcase.png`
  will need regenerating.
