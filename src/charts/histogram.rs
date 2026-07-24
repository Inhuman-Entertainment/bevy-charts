//! Histograms.
//!
//! A histogram is a bar chart whose categories are derived rather than given, so
//! this module does the binning and hands the result to
//! [`bar`](crate::charts::bar) — the geometry, spacing, and axes are the bar
//! chart's, unchanged.

use bevy::prelude::*;

use crate::axis::{AxisStyle, spawn_axes};
use crate::charts::bar::{BarLayout, bar_value_scale, spawn_bars};
use crate::charts::{ChartAssets, ChartChanged, ChartPrimitives, ChartSize, MaterialCache};
use crate::data::{ChartData, Dataset};
use crate::palette::ChartPalette;

/// A histogram of one or more sets of observations.
///
/// Each [`Dataset`] here holds raw samples, not per-category values; the chart
/// counts them into `bins` shared across every series so the series stay
/// comparable.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_charts::prelude::*;
/// # fn setup(mut commands: Commands) {
/// commands.spawn(HistogramChart3d::new(vec![Dataset::new(
///     "Samples",
///     vec![1.0, 1.2, 2.4, 2.5, 2.6, 3.9],
/// )]));
/// # }
/// ```
#[derive(Component, Debug, Clone, PartialEq)]
#[require(Transform, Visibility, ChartSize, ChartPalette, AxisStyle)]
pub struct HistogramChart3d {
    /// Series of raw observations.
    pub datasets: Vec<Dataset>,
    /// Number of bins to count into.
    pub bins: usize,
    /// Explicit bin range. Samples outside it are dropped. Defaults to the
    /// combined extent of the observations.
    pub range: Option<(f32, f32)>,
    /// Fraction of each bin slot left empty, in `0.0..1.0`.
    ///
    /// Defaults to zero: histogram bars are contiguous because the bins are, and
    /// a gap would imply gaps in the data.
    pub bin_gap: f32,
    /// Fraction of each series slot left empty, in `0.0..1.0`.
    pub series_gap: f32,
}

impl Default for HistogramChart3d {
    fn default() -> Self {
        Self {
            datasets: Vec::new(),
            bins: 10,
            range: None,
            bin_gap: 0.0,
            series_gap: 0.2,
        }
    }
}

impl HistogramChart3d {
    /// A histogram of `datasets` with the default bin count.
    pub fn new(datasets: impl Into<Vec<Dataset>>) -> Self {
        Self {
            datasets: datasets.into(),
            ..default()
        }
    }

    /// Set the bin count.
    pub fn with_bins(mut self, bins: usize) -> Self {
        self.bins = bins;
        self
    }

    /// Pin the bin range instead of deriving it from the observations.
    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.range = Some((min, max));
        self
    }

    /// The bin range actually used: the explicit one, or the combined extent of
    /// every finite observation.
    fn effective_range(&self) -> Option<(f32, f32)> {
        if let Some((min, max)) = self.range {
            return (min < max).then_some((min, max));
        }
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for v in self
            .datasets
            .iter()
            .flat_map(|d| d.data.iter().copied())
            .filter(|v| v.is_finite())
        {
            min = min.min(v);
            max = max.max(v);
        }
        (min <= max).then_some((min, max))
    }

    /// Count the observations into bins, producing data a bar chart can draw.
    ///
    /// Returns `None` when there is nothing to count.
    pub fn to_chart_data(&self) -> Option<ChartData> {
        if self.bins == 0 || self.datasets.is_empty() {
            return None;
        }
        let (min, max) = self.effective_range()?;
        // A single repeated value has no width to divide into bins; widen it so
        // the sample still lands somewhere. `effective_range` guarantees
        // `min <= max`, so the `abs` is belt-and-braces against that changing.
        let (min, max) = if (max - min).abs() < f32::EPSILON {
            (min - 0.5, max + 0.5)
        } else {
            (min, max)
        };
        let width = (max - min) / self.bins as f32;

        let labels = (0..self.bins)
            .map(|i| {
                let lo = min + width * i as f32;
                format!("{:.2}–{:.2}", lo, lo + width)
            })
            .collect::<Vec<_>>();

        let datasets = self
            .datasets
            .iter()
            .map(|dataset| {
                let mut counts = vec![0.0f32; self.bins];
                for value in dataset.data.iter().copied().filter(|v| v.is_finite()) {
                    if value < min || value > max {
                        continue;
                    }
                    // The top edge belongs to the last bin rather than to a bin
                    // past the end.
                    let index = (((value - min) / width) as usize).min(self.bins - 1);
                    counts[index] += 1.0;
                }
                Dataset {
                    label: dataset.label.clone(),
                    data: counts,
                    color: dataset.color,
                }
            })
            .collect::<Vec<_>>();

        Some(ChartData { labels, datasets })
    }
}

fn build_histogram_charts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cache: ResMut<MaterialCache>,
    primitives: Res<ChartPrimitives>,
    charts: Query<
        (
            Entity,
            &HistogramChart3d,
            &ChartSize,
            &ChartPalette,
            &AxisStyle,
        ),
        ChartChanged<HistogramChart3d>,
    >,
) {
    for (entity, chart, size, palette, axis_style) in &charts {
        let size = size.0;
        let mut assets = ChartAssets {
            meshes: &mut meshes,
            materials: &mut materials,
            cache: &mut cache,
            primitives: &primitives,
        };

        let binned = chart
            .to_chart_data()
            .and_then(|data| bar_value_scale(&data, size.y).map(|scale| (data, scale)));
        let Some((data, scale)) = binned else {
            commands.entity(entity).despawn_related::<Children>();
            continue;
        };

        commands
            .entity(entity)
            .despawn_related::<Children>()
            .with_children(|parent| {
                spawn_axes(parent, &mut assets, size, &scale, axis_style, palette);
                spawn_bars(
                    parent,
                    &mut assets,
                    &data,
                    palette,
                    &BarLayout {
                        size,
                        scale: &scale,
                        category_gap: chart.bin_gap,
                        series_gap: chart.series_gap,
                    },
                );
            });
    }
}

/// Adds the [`HistogramChart3d`] build system.
pub struct HistogramChartPlugin;

impl Plugin for HistogramChartPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::ChartCorePlugin)
            .add_systems(PostUpdate, build_histogram_charts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_every_sample_exactly_once() {
        let samples = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let chart = HistogramChart3d::new(vec![Dataset::new("s", samples.clone())]).with_bins(5);
        let data = chart.to_chart_data().unwrap();
        let total: f32 = data.datasets[0].data.iter().sum();
        assert_eq!(total, samples.len() as f32);
        assert_eq!(data.labels.len(), 5);
    }

    #[test]
    fn the_maximum_lands_in_the_last_bin_not_past_it() {
        let chart = HistogramChart3d::new(vec![Dataset::new("s", vec![0.0, 10.0])]).with_bins(4);
        let counts = &chart.to_chart_data().unwrap().datasets[0].data;
        assert_eq!(counts[0], 1.0);
        assert_eq!(counts[3], 1.0, "the top edge belongs to the last bin");
    }

    #[test]
    fn samples_outside_an_explicit_range_are_dropped() {
        let chart = HistogramChart3d::new(vec![Dataset::new("s", vec![-5.0, 1.0, 50.0])])
            .with_bins(2)
            .with_range(0.0, 10.0);
        let counts = &chart.to_chart_data().unwrap().datasets[0].data;
        assert_eq!(counts.iter().sum::<f32>(), 1.0);
    }

    #[test]
    fn a_single_repeated_value_still_bins() {
        let chart = HistogramChart3d::new(vec![Dataset::new("s", vec![7.0, 7.0, 7.0])]);
        let counts = &chart.to_chart_data().unwrap().datasets[0].data;
        assert_eq!(counts.iter().sum::<f32>(), 3.0);
    }

    #[test]
    fn no_datasets_means_no_data() {
        assert!(HistogramChart3d::new(vec![]).to_chart_data().is_none());
    }
}
