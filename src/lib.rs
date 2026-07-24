//! 3D charts and graphs for the [Bevy](https://bevy.org) game engine.
//!
//! Charts are plain components. Spawn one with data, and a system builds the
//! geometry as its children; change the data and it rebuilds. Nothing has to be
//! torn down or refreshed by hand.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_charts::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins((DefaultPlugins, BevyChartsPlugin))
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn(Camera3d::default());
//!     commands.spawn(BarChart3d::new(
//!         ChartData::new()
//!             .with_labels(["Q1", "Q2", "Q3", "Q4"])
//!             .with_dataset(Dataset::new("Revenue", vec![12.0, 19.0, 7.0, 15.0])),
//!     ));
//! }
//! ```
//!
//! # Layout
//!
//! Every chart draws into the box `0..size` in its own local space, with the
//! origin at the bottom-left-front corner: x is the category axis, y the value
//! axis, z the series axis. Position a chart by giving it a [`Transform`]; size
//! it with [`ChartSize`].
//!
//! # Styling
//!
//! [`ChartSize`], [`ChartPalette`], and [`AxisStyle`] are required components, so
//! every chart has them at defaults and any of them can be overridden at spawn:
//!
//! ```no_run
//! # use bevy::prelude::*;
//! # use bevy_charts::prelude::*;
//! # fn setup(mut commands: Commands) {
//! commands.spawn((
//!     BarChart3d::new(ChartData::default()),
//!     ChartSize(Vec3::new(8.0, 4.0, 3.0)),
//!     ChartPalette::light(),
//!     AxisStyle { show_grid: false, ..default() },
//! ));
//! # }
//! ```

#![forbid(unsafe_code)]

pub mod axis;
pub mod charts;
pub mod data;
pub mod palette;

use bevy::prelude::*;

pub use crate::axis::{AxisStyle, Scale};
pub use crate::charts::bar::{BarChart3d, BarChartPlugin};
pub use crate::charts::histogram::{HistogramChart3d, HistogramChartPlugin};
pub use crate::charts::line::{LineChart3d, LineChartPlugin};
pub use crate::charts::scatter::{ScatterChart3d, ScatterChartPlugin};
pub use crate::charts::surface::{SurfaceChart3d, SurfaceChartPlugin};
pub use crate::charts::{ChartPrimitives, ChartSize, MaterialCache};
pub use crate::data::{ChartData, Dataset, PointDataset};
pub use crate::palette::{ChartPalette, PaletteMode};

/// Adds every chart type.
///
/// Add this once alongside `DefaultPlugins`. To pull in only some chart types,
/// add the individual plugins ([`BarChartPlugin`] and friends) instead — each
/// one registers the shared resources itself.
pub struct BevyChartsPlugin;

impl Plugin for BevyChartsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BarChartPlugin,
            LineChartPlugin,
            ScatterChartPlugin,
            SurfaceChartPlugin,
            HistogramChartPlugin,
        ));
    }
}

/// Shared resources every chart type needs.
///
/// Each chart plugin adds this, so adding several of them — or all of them via
/// [`BevyChartsPlugin`] — is fine.
pub struct ChartCorePlugin;

impl Plugin for ChartCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChartPrimitives>()
            .init_resource::<MaterialCache>();
    }

    /// Every chart plugin depends on this one, so it is added more than once by
    /// design; `init_resource` makes the repeats harmless.
    fn is_unique(&self) -> bool {
        false
    }
}

/// The types you need to build a chart.
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        AxisStyle, BarChart3d, BevyChartsPlugin, ChartData, ChartPalette, ChartSize, Dataset,
        HistogramChart3d, LineChart3d, PaletteMode, PointDataset, ScatterChart3d, SurfaceChart3d,
    };
}
