## ADDED Requirements

### Requirement: Charts are readable without surrounding UI

A chart SHALL be able to draw the text needed to identify what it shows: value
ticks, category names, and a series legend, without the caller building any
separate UI.

#### Scenario: A labelled bar chart names its own categories and values

- **WHEN** a bar chart with category labels and named series is spawned with
  labels enabled
- **THEN** text is drawn for the value ticks, for each category, and for each
  series in the legend

#### Scenario: Data that carries no names draws no text for them

- **WHEN** a chart's `ChartData::labels` is empty and its datasets have empty
  label strings
- **THEN** no category or legend text is drawn, and the chart still renders its
  geometry and value ticks

### Requirement: Labels are world-space geometry

Labels SHALL be rendered as meshes in the chart's own coordinate space, not as a
screen-space overlay. They SHALL depth-test against the rest of the scene.

#### Scenario: A label behind geometry is occluded by it

- **WHEN** a label sits behind opaque chart geometry from the camera's point of
  view
- **THEN** it is hidden by that geometry, as any other object in the scene would be

#### Scenario: Labels move with the chart

- **WHEN** a chart entity's `Transform` is translated or rotated
- **THEN** its labels move with it, without any per-frame reprojection

#### Scenario: No additional camera is required

- **WHEN** a scene contains a single `Camera3d`
- **THEN** labels render, without a second camera or render layers

### Requirement: Value tick labels agree with the grid

Value labels SHALL be placed at the tick positions the value scale produces, so
that a label and its grid line describe the same number.

#### Scenario: A label appears at each drawn grid line

- **WHEN** a chart draws grid lines at its value ticks and value labels are enabled
- **THEN** there is one value label per grid line, positioned at that line

#### Scenario: Tick text is formatted from the tick value

- **WHEN** a value tick falls at 50
- **THEN** the label reads as that number rather than as an internal position

### Requirement: Label orientation is selectable

Labels SHALL default to turning to face the camera, and SHALL offer a fixed
in-plane alternative for charts intended to read as physical objects.

#### Scenario: Billboarded labels stay legible as the camera orbits

- **WHEN** the orientation is `Billboard` and the camera moves around the chart
- **THEN** the labels turn to face the camera and remain readable

#### Scenario: In-plane labels stay fixed to the chart

- **WHEN** the orientation is `InPlane` and the camera moves
- **THEN** the labels keep their orientation relative to the chart and turn away
  from the viewer like any other surface

### Requirement: Label size is expressed in world units

Label size SHALL be given in world units, consistent with every other dimension
in the library, so that a chart's appearance does not depend on window
resolution.

#### Scenario: A chart looks the same at different window sizes

- **WHEN** the same chart is rendered to two differently sized viewports
- **THEN** the labels occupy the same proportion of the chart in both

### Requirement: A missing font degrades gracefully

A chart SHALL render its geometry when no font is available, rather than
panicking or rendering nothing.

#### Scenario: No font configured and no default compiled in

- **WHEN** no font is set on the label style and the default font is unavailable
- **THEN** the chart's bars, lines, axes, and grid still render, and no text is
  drawn

#### Scenario: A caller-supplied font is used

- **WHEN** a font handle is set on the label style
- **THEN** the labels are drawn with that font

### Requirement: Labels can be turned off individually

Each kind of label SHALL be independently controllable, so a caller can keep the
ones that fit and drop the ones that clutter.

#### Scenario: Turning off category labels keeps the value labels

- **WHEN** category labels are disabled and value labels are left enabled
- **THEN** value tick text is drawn and no category text is

#### Scenario: Disabling all labels leaves the chart as it was

- **WHEN** every label kind is disabled
- **THEN** the chart draws the same geometry as it would with no label support at
  all, and creates no label entities

### Requirement: Labels are rebuilt with the chart

Label geometry SHALL follow the same change-detection rules as the rest of a
chart: regenerated when the chart's data or style changes, and left alone
otherwise.

#### Scenario: Changing the data updates the labels

- **WHEN** a chart's categories are replaced
- **THEN** the category labels are rebuilt to match

#### Scenario: An unchanged chart does not rebuild its labels

- **WHEN** a frame passes with no change to a chart or its style
- **THEN** its existing label entities are left in place
