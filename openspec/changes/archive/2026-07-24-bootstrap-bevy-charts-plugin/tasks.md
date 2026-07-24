## 1. Crate setup

- [x] 1.1 `Cargo.toml` — library crate `bevy_charts`, edition 2024, MSRV 1.95
- [x] 1.2 Depend on `bevy` 0.19 with `default-features = false` and only
      `bevy_asset`, `bevy_color`, `bevy_pbr`, so games control the rest
- [x] 1.3 Depend on `plotters` with `default-features = false` for tick placement
- [x] 1.4 Dual-license MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) to
      match Bevy and the wider Rust ecosystem
- [x] 1.5 Verify the Bevy 0.19 API against vendored sources rather than recall

## 2. Data model

- [x] 2.1 `src/data.rs` — `ChartData`, `Dataset`, `PointDataset`
- [x] 2.2 Builder methods and `value_range` / `category_count` / `point_bounds`
- [x] 2.3 Treat non-finite values as absent throughout

## 3. Scales and axes

- [x] 3.1 `src/axis.rs` — `Scale` mapping a data range onto an axis length
- [x] 3.2 Delegate tick placement to `plotters`' `Ranged::key_points`
- [x] 3.3 Widen degenerate ranges instead of dividing by zero
- [x] 3.4 `AxisStyle` plus axis/grid line-list geometry
- [x] 3.5 Unit tests for mapping, degenerate ranges, and tick rounding

## 4. Styling

- [x] 4.1 `src/palette.rs` — eight-slot categorical palette, dark and light steps
- [x] 4.2 Clamp past the last slot rather than cycling
- [x] 4.3 Sequential ramp for continuous magnitude (surface height)
- [x] 4.4 Document the three-series ceiling on `ScatterChart3d`

## 5. Chart infrastructure

- [x] 5.1 `src/charts/mod.rs` — `ChartSize`, `ChartPrimitives`, `MaterialCache`
- [x] 5.2 Shared unit cube and unit sphere for all marks
- [x] 5.3 Cache materials by color so rebuilds do not leak assets
- [x] 5.4 `ChartCorePlugin` with `is_unique() == false` so each chart plugin can
      register the shared resources independently

## 6. Chart types

- [x] 6.1 `src/charts/bar.rs` — `BarChart3d` + `BarChartPlugin`
- [x] 6.2 `src/charts/line.rs` — `LineChart3d` + `LineChartPlugin`
- [x] 6.3 `src/charts/scatter.rs` — `ScatterChart3d` + `ScatterChartPlugin`
- [x] 6.4 `src/charts/surface.rs` — `SurfaceChart3d` + `SurfaceChartPlugin`
- [x] 6.5 `src/charts/histogram.rs` — `HistogramChart3d`, binning onto bar geometry
- [x] 6.6 Unit tests for scale selection, triangulation, and binning edge cases

## 7. Plugin entry point

- [x] 7.1 `src/lib.rs` — `BevyChartsPlugin` and `prelude`
- [x] 7.2 Crate-level docs covering layout and styling

## 8. Examples and docs

- [x] 8.1 `examples/bar_chart.rs`
- [x] 8.2 `examples/line_chart.rs` — live-updating data
- [x] 8.3 `examples/scatter_chart.rs`
- [x] 8.4 `examples/surface_chart.rs` — animated height field
- [x] 8.5 `examples/showcase.rs` — all five types in one scene
- [x] 8.6 `README.md` — installation, compatibility table, usage, design notes
- [x] 8.7 `assets/showcase.png`

## 9. Verification

- [x] 9.1 `cargo check` clean with no warnings
- [x] 9.2 17 unit tests and 7 doctests passing
- [x] 9.3 All examples compile
- [x] 9.4 Run the app on a real GPU and confirm all five chart types render
- [x] 9.5 Fix: `Mesh3d` requires only `Transform` in Bevy 0.19, so chart children
      needed an explicit `Visibility` to be picked up by render extraction

<!-- Deferred work lives in ROADMAP.md at the repository root, not here: this
     file tracks only what is in scope for this change, so that a completed
     change reads as completed. -->
