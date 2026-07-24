## ADDED Requirements

### Requirement: Series colors are assigned in a fixed, safety-checked order

`ChartPalette` SHALL assign categorical colors by slot index in a fixed order
whose adjacency is chosen for color-vision-deficiency separation. It SHALL NOT
cycle.

#### Scenario: Series take slots in order

- **WHEN** a chart has three series and none override their color
- **THEN** they take palette slots one, two, and three in that order

#### Scenario: A ninth series clamps rather than repeating an earlier hue

- **WHEN** a color is requested for an index past the last slot
- **THEN** the last slot's color is returned, so the repetition is at the tail
  rather than colliding with an early series

#### Scenario: An explicit color overrides the slot

- **WHEN** a dataset sets its own color
- **THEN** that color is used instead of the palette slot

### Requirement: The palette carries steps for both dark and light scenes

`ChartPalette` SHALL provide the same hues stepped separately for dark and light
scene backgrounds, selected rather than derived by inverting.

#### Scenario: Choosing the light palette changes the steps

- **WHEN** a chart is given `ChartPalette::light()` instead of the default dark
- **THEN** series, axis, and grid colors all change to the light-background steps

### Requirement: Scatter charts document a three-series ceiling

`ScatterChart3d` SHALL document a three-series ceiling and the remedy for
exceeding it. Scatter marks are compared pairwise rather than in sequence, so
they need pairwise color separation, which only the first three slots guarantee.

#### Scenario: The limit is discoverable at the point of use

- **WHEN** a developer reads the `ScatterChart3d` documentation
- **THEN** the three-series ceiling and the suggested remedy (faceting) are stated

### Requirement: Axes and grid are configurable and recessive

`AxisStyle` SHALL control whether axis lines and grid lines are drawn and roughly
how many value ticks are placed. Grid lines SHALL be visually recessive relative
to axis lines, and both SHALL be unlit so their weight does not depend on scene
lighting.

#### Scenario: Grid can be turned off independently

- **WHEN** `show_grid` is disabled but `show_axes` is left on
- **THEN** the three axis lines are drawn and no grid lines are

#### Scenario: Grid lines align with value ticks

- **WHEN** the grid is drawn
- **THEN** a grid line appears at each value tick position, on the back and side
  walls of the chart box

#### Scenario: A zero line is drawn when zero is interior

- **WHEN** the value range spans zero with zero strictly inside it
- **THEN** an additional axis line is drawn at the zero position

### Requirement: Style components default so a bare chart is well-formed

`ChartSize`, `ChartPalette`, and `AxisStyle` SHALL be required components of every
chart type, so a chart spawned with data alone renders with sensible defaults.

#### Scenario: A chart spawned with only data renders

- **WHEN** an entity is spawned with just a chart component
- **THEN** it acquires default size, palette, and axis style, and draws
