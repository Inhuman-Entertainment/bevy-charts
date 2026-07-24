## ADDED Requirements

### Requirement: Charts occupy a predictable local box

Every chart SHALL draw into the box `0..size` in its own local space, with the
origin at the bottom-left-front corner: x the category axis, y the value axis,
z the series axis. `ChartSize` SHALL control that box.

#### Scenario: Positioning a chart uses only its Transform

- **WHEN** a chart entity is given a `Transform` translation
- **THEN** the whole chart, including its axes, moves with it, and no geometry is
  emitted outside its `ChartSize` box on the category and series axes

### Requirement: Bar charts are measured from zero

`BarChart3d` SHALL place categories along x and series along z, and its value
axis SHALL always include zero.

#### Scenario: An all-positive series still starts at zero

- **WHEN** a bar chart's only series holds `[10.0, 20.0]`
- **THEN** the value scale spans `0.0..20.0`

#### Scenario: Negative values extend the axis below zero

- **WHEN** a bar chart's series holds `[-5.0, 8.0]`
- **THEN** the value scale spans `-5.0..8.0` and bars below zero hang beneath the
  baseline

### Requirement: Line charts do not force a zero baseline

`LineChart3d` SHALL default its value axis to the data extent, and SHALL offer an
opt-in to include zero.

#### Scenario: A narrow band far from the origin is not flattened

- **WHEN** a line chart's series holds `[100.0, 104.0]` with the default settings
- **THEN** the value scale spans `100.0..104.0`

#### Scenario: Opting in extends the axis to zero

- **WHEN** the same series is charted with `begin_at_zero` enabled
- **THEN** the value scale starts at `0.0`

### Requirement: Scatter points carry their own coordinates

`ScatterChart3d` SHALL take point series whose members hold x, y, and z in data
space, and SHALL map them into the chart box using the combined extent of every
series so that series remain comparable.

#### Scenario: All series share one mapping

- **WHEN** two point series with different extents are charted together
- **THEN** both are mapped by the same per-axis scales, derived from their union

### Requirement: Surface charts triangulate a height field

`SurfaceChart3d` SHALL accept a row-major height field over a `cols` × `rows`
grid, and SHALL emit two triangles per cell with normals facing up and per-vertex
colors encoding height.

#### Scenario: A well-formed grid produces the expected mesh

- **WHEN** a 3 × 4 grid of twelve samples is charted
- **THEN** the mesh holds twelve vertices and `(3-1) * (4-1) * 6` indices

#### Scenario: A mismatched sample count is rejected

- **WHEN** the height count does not equal `cols * rows`
- **THEN** the chart is treated as invalid and draws nothing, rather than guessing
  at the intended shape

#### Scenario: A grid too small to hold a quad is rejected

- **WHEN** either dimension is below two
- **THEN** the chart is treated as invalid

### Requirement: Histograms bin raw observations and reuse bar geometry

`HistogramChart3d` SHALL take series of raw samples, count them into a shared set
of bins, and draw the counts with the same geometry as `BarChart3d`.

#### Scenario: Every in-range sample is counted exactly once

- **WHEN** eleven samples spanning `0.0..10.0` are binned into five bins
- **THEN** the counts sum to eleven and five bin labels are produced

#### Scenario: The top edge belongs to the last bin

- **WHEN** samples `[0.0, 10.0]` are binned into four bins over that range
- **THEN** the first and last bins each hold one sample, and no sample falls past
  the end

#### Scenario: Samples outside an explicit range are dropped

- **WHEN** a range of `0.0..10.0` is set and the samples are `[-5.0, 1.0, 50.0]`
- **THEN** exactly one sample is counted

#### Scenario: A single repeated value still bins

- **WHEN** every sample has the same value
- **THEN** the range is widened so the samples still land in a bin, and all are
  counted

### Requirement: Non-finite values are skipped, not drawn

Charts SHALL ignore `NaN` and infinite values rather than emitting degenerate
geometry.

#### Scenario: A NaN in a series does not produce a mark

- **WHEN** a series contains a `NaN`
- **THEN** no mark is emitted for that entry and the remaining entries draw
  normally
