## ADDED Requirements

### Requirement: Charts are components that build their own geometry

Every chart type SHALL be a Bevy `Component`. Spawning the component with data
SHALL be sufficient to produce rendered geometry; no builder, handle, or manual
registration step is required.

#### Scenario: Spawning a chart produces geometry

- **WHEN** an entity is spawned with a chart component holding at least one
  finite data point, and the corresponding plugin is registered
- **THEN** the chart entity gains child entities carrying `Mesh3d`,
  `MeshMaterial3d`, `Transform`, and `Visibility`, and those children are visible
  to the renderer

#### Scenario: A chart with nothing finite to plot draws nothing

- **WHEN** a chart's data is empty, or contains no finite values
- **THEN** the chart despawns any existing children and spawns none, rather than
  panicking or dividing by a zero-width range

### Requirement: Charts rebuild when their data or style changes

A chart SHALL regenerate its geometry when its own component changes, or when any
of its shared style components (`ChartSize`, `ChartPalette`, `AxisStyle`) change.

#### Scenario: Mutating chart data redraws the chart

- **WHEN** a system mutably dereferences a chart component and alters its data
- **THEN** on the next run of the build schedule the chart's previous children are
  despawned and replaced with geometry reflecting the new data

#### Scenario: Unchanged charts are not rebuilt

- **WHEN** a frame passes in which neither a chart component nor its style
  components were changed
- **THEN** that chart's children are left untouched

### Requirement: Value scales map data onto the chart box

A `Scale` SHALL map a data range onto an axis length in world units, and SHALL
place approximately a requested number of round-numbered ticks within that range.

#### Scenario: Endpoints map to the ends of the axis

- **WHEN** a `Scale` over `0.0..10.0` with extent `4.0` maps the values `0.0`,
  `5.0`, and `10.0`
- **THEN** the results are `0.0`, `2.0`, and `4.0`

#### Scenario: A degenerate range is widened rather than dividing by zero

- **WHEN** a `Scale` is constructed whose minimum equals its maximum
- **THEN** the range is widened around its midpoint, and mapping any value returns
  a finite result

#### Scenario: Ticks land on round numbers inside the range

- **WHEN** roughly five ticks are requested over the range `0.0..100.0`
- **THEN** every returned tick lies within the range and the set includes round
  values such as `50.0`

#### Scenario: Requesting zero ticks returns none

- **WHEN** zero ticks are requested
- **THEN** an empty set is returned

### Requirement: Shared resources are registered once regardless of plugin mix

Each chart plugin SHALL register the shared primitive meshes and material cache,
and adding several chart plugins together SHALL NOT panic or duplicate them.

#### Scenario: Adding all chart plugins together succeeds

- **WHEN** an app adds `BevyChartsPlugin`, which adds all five chart plugins
- **THEN** the app builds without a duplicate-plugin panic, and one set of shared
  resources exists

#### Scenario: Adding a single chart plugin is sufficient

- **WHEN** an app adds only `BarChartPlugin`
- **THEN** bar charts render, without requiring `BevyChartsPlugin`

### Requirement: Rebuilding does not leak assets

Repeated rebuilds SHALL NOT grow the material asset table without bound.

#### Scenario: Repeated rebuilds reuse materials

- **WHEN** a chart is rebuilt many times with the same series colors
- **THEN** the materials for those colors are reused rather than newly created on
  each rebuild
