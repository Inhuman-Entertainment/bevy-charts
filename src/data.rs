//! Chart.js-shaped data containers, shared by every chart type.
//!
//! Categorical charts (bar, line, histogram) read [`ChartData`]: shared x-axis
//! labels plus parallel value series. Charts whose points carry their own
//! coordinates (scatter) read [`PointDataset`] instead.

use bevy::prelude::*;

/// Categorical chart data: shared x-axis labels plus one or more value series.
///
/// Datasets do not have to be the same length as `labels`; missing trailing
/// values are simply not drawn.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChartData {
    /// One label per category slot along the x axis.
    pub labels: Vec<String>,
    /// The series plotted against those categories.
    pub datasets: Vec<Dataset>,
}

impl ChartData {
    /// Empty data.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the category labels.
    pub fn with_labels<I, S>(mut self, labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Append a series.
    pub fn with_dataset(mut self, dataset: Dataset) -> Self {
        self.datasets.push(dataset);
        self
    }

    /// Number of category slots, which is the longest of the labels and any series.
    ///
    /// A chart with values but no labels still needs slots to draw into, so this
    /// does not simply return `labels.len()`.
    pub fn category_count(&self) -> usize {
        self.datasets
            .iter()
            .map(|d| d.data.len())
            .chain(std::iter::once(self.labels.len()))
            .max()
            .unwrap_or(0)
    }

    /// Smallest and largest finite value across every series.
    ///
    /// Returns `None` when there is nothing finite to plot, which callers use to
    /// skip building a chart rather than dividing by a zero-width range.
    pub fn value_range(&self) -> Option<(f32, f32)> {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for value in self
            .datasets
            .iter()
            .flat_map(|d| d.data.iter().copied())
            .filter(|v| v.is_finite())
        {
            min = min.min(value);
            max = max.max(value);
        }
        (min <= max).then_some((min, max))
    }
}

/// A single named series of values, one per category slot.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Dataset {
    /// Name shown in legends and used to identify the series.
    pub label: String,
    /// One value per category slot.
    pub data: Vec<f32>,
    /// Overrides the palette slot this series would otherwise get.
    pub color: Option<Color>,
}

impl Dataset {
    /// A series named `label` holding `data`, colored from the chart palette.
    pub fn new(label: impl Into<String>, data: impl Into<Vec<f32>>) -> Self {
        Self {
            label: label.into(),
            data: data.into(),
            color: None,
        }
    }

    /// Pin this series to an explicit color instead of its palette slot.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// A named series of points that carry their own 3D coordinates.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PointDataset {
    /// Name shown in legends and used to identify the series.
    pub label: String,
    /// Points in data space, before the chart maps them into its bounding box.
    pub points: Vec<Vec3>,
    /// Overrides the palette slot this series would otherwise get.
    pub color: Option<Color>,
}

impl PointDataset {
    /// A point series named `label`.
    pub fn new(label: impl Into<String>, points: impl Into<Vec<Vec3>>) -> Self {
        Self {
            label: label.into(),
            points: points.into(),
            color: None,
        }
    }

    /// Pin this series to an explicit color instead of its palette slot.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// Per-axis extent of a set of point series, in data space.
///
/// Returns `None` when no series holds a finite point.
pub fn point_bounds(datasets: &[PointDataset]) -> Option<(Vec3, Vec3)> {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    let mut any = false;
    for point in datasets
        .iter()
        .flat_map(|d| d.points.iter().copied())
        .filter(|p| p.is_finite())
    {
        min = min.min(point);
        max = max.max(point);
        any = true;
    }
    any.then_some((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_count_follows_the_longest_series_not_the_labels() {
        // Values without labels still need slots to draw into, so this is not
        // simply `labels.len()`.
        let data = ChartData::new()
            .with_labels(["a", "b"])
            .with_dataset(Dataset::new("s", vec![1.0, 2.0, 3.0, 4.0]));
        assert_eq!(data.category_count(), 4);

        let data = ChartData::new().with_labels(["a", "b", "c"]);
        assert_eq!(data.category_count(), 3);

        assert_eq!(ChartData::new().category_count(), 0);
    }

    #[test]
    fn value_range_spans_every_series() {
        let data = ChartData::new()
            .with_dataset(Dataset::new("low", vec![-3.0, 1.0]))
            .with_dataset(Dataset::new("high", vec![4.0, 9.0]));
        assert_eq!(data.value_range(), Some((-3.0, 9.0)));
    }

    #[test]
    fn value_range_ignores_non_finite_values() {
        let data = ChartData::new().with_dataset(Dataset::new(
            "s",
            vec![f32::NAN, 2.0, f32::INFINITY, 5.0, f32::NEG_INFINITY],
        ));
        assert_eq!(data.value_range(), Some((2.0, 5.0)));
    }

    #[test]
    fn value_range_is_none_when_nothing_is_plottable() {
        assert_eq!(ChartData::new().value_range(), None);
        let empty_series = ChartData::new().with_dataset(Dataset::new("s", Vec::new()));
        assert_eq!(empty_series.value_range(), None);
        let all_nan = ChartData::new().with_dataset(Dataset::new("s", vec![f32::NAN]));
        assert_eq!(all_nan.value_range(), None);
    }

    #[test]
    fn an_explicit_color_overrides_the_palette_slot() {
        let plain = Dataset::new("s", vec![1.0]);
        assert_eq!(plain.color, None);
        let pinned = Dataset::new("s", vec![1.0]).with_color(Color::WHITE);
        assert_eq!(pinned.color, Some(Color::WHITE));
    }

    #[test]
    fn point_bounds_covers_the_union_of_every_series() {
        // Series must share one mapping, so the bounds are the union rather
        // than each series being normalised to itself.
        let datasets = vec![
            PointDataset::new("a", vec![Vec3::new(-1.0, 0.0, 5.0)]),
            PointDataset::new("b", vec![Vec3::new(3.0, 8.0, -2.0)]),
        ];
        assert_eq!(
            point_bounds(&datasets),
            Some((Vec3::new(-1.0, 0.0, -2.0), Vec3::new(3.0, 8.0, 5.0)))
        );
    }

    #[test]
    fn point_bounds_skips_non_finite_points_and_reports_nothing_when_empty() {
        let with_nan = vec![PointDataset::new(
            "a",
            vec![Vec3::splat(f32::NAN), Vec3::new(1.0, 2.0, 3.0)],
        )];
        assert_eq!(
            point_bounds(&with_nan),
            Some((Vec3::new(1.0, 2.0, 3.0), Vec3::new(1.0, 2.0, 3.0)))
        );

        assert_eq!(point_bounds(&[]), None);
        assert_eq!(point_bounds(&[PointDataset::new("a", Vec::new())]), None);
    }
}
