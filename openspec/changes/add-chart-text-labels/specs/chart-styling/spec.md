## ADDED Requirements

### Requirement: Label appearance is styled alongside the palette and axes

Label colour, size, orientation, font, and which kinds are drawn SHALL be
configured through a component, defaulted like the other style components so a
chart spawned with data alone is labelled sensibly.

#### Scenario: A chart spawned with only data gets default labelling

- **WHEN** an entity is spawned with just a chart component
- **THEN** it acquires a default label style and draws labels without any further
  configuration

#### Scenario: Label style can be overridden at spawn

- **WHEN** a chart is spawned with an explicit label style
- **THEN** that style is used instead of the default

### Requirement: Label colour comes from the palette by default

Label text SHALL take a readable colour from the chart palette by default, so
that switching between the dark and light palettes keeps text legible against
its background.

#### Scenario: Switching to the light palette keeps text readable

- **WHEN** a chart's palette is changed from dark to light
- **THEN** the label colour changes with it rather than staying tuned for the
  previous background

#### Scenario: Legend swatches match their series

- **WHEN** a legend is drawn for a multi-series chart
- **THEN** each entry's swatch is the colour that series is drawn in
