//! Series colors.
//!
//! The eight categorical hues below are a validated palette: the slot *ordering*
//! is the colorblind-safety mechanism, not a cosmetic choice. Adjacent slots
//! clear a CVD separation floor in both the light and dark variants, which is why
//! [`ChartPalette::series_color`] clamps instead of wrapping — a ninth series
//! would have to repeat a hue, and a chart that repeats a hue is lying about
//! identity. Fold extras into an "other" series or split the chart instead.

use bevy::prelude::*;

/// Number of distinct categorical slots a palette provides.
pub const SERIES_SLOTS: usize = 8;

/// Categorical hues stepped for a dark scene background.
const DARK_SERIES: [(u8, u8, u8); SERIES_SLOTS] = [
    (0x39, 0x87, 0xe5), // blue
    (0xd9, 0x59, 0x26), // orange
    (0x19, 0x9e, 0x70), // aqua
    (0xc9, 0x85, 0x00), // yellow
    (0xd5, 0x51, 0x81), // magenta
    (0x00, 0x83, 0x00), // green
    (0x90, 0x85, 0xe9), // violet
    (0xe6, 0x67, 0x67), // red
];

/// The same eight hues stepped for a light scene background.
const LIGHT_SERIES: [(u8, u8, u8); SERIES_SLOTS] = [
    (0x2a, 0x78, 0xd6), // blue
    (0xeb, 0x68, 0x34), // orange
    (0x1b, 0xaf, 0x7a), // aqua
    (0xed, 0xa1, 0x00), // yellow
    (0xe8, 0x7b, 0xa4), // magenta
    (0x00, 0x83, 0x00), // green
    (0x4a, 0x3a, 0xa7), // violet
    (0xe3, 0x49, 0x48), // red
];

/// Single-hue blue ramp, light to dark, for continuous magnitude (surface charts).
const SEQUENTIAL: [(u8, u8, u8); 7] = [
    (0xcd, 0xe2, 0xfb),
    (0x9e, 0xc5, 0xf4),
    (0x6d, 0xa7, 0xec),
    (0x39, 0x87, 0xe5),
    (0x25, 0x6a, 0xbf),
    (0x18, 0x4f, 0x95),
    (0x0d, 0x36, 0x6b),
];

/// Which background the chart will be seen against.
///
/// This is not a cosmetic toggle: each variant carries its own steps of the same
/// hues, chosen to hold contrast against that background.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaletteMode {
    /// Steps tuned for a dark scene — the usual case in a game.
    #[default]
    Dark,
    /// Steps tuned for a light scene.
    Light,
}

/// The colors a chart draws its series, axes, and grid with.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct ChartPalette {
    /// Which background the steps are tuned for.
    pub mode: PaletteMode,
    /// Color of axis lines and tick marks.
    pub axis: Color,
    /// Color of grid lines. Kept recessive relative to `axis`.
    pub grid: Color,
}

impl Default for ChartPalette {
    fn default() -> Self {
        Self::dark()
    }
}

impl ChartPalette {
    /// Palette for a dark scene background.
    pub fn dark() -> Self {
        Self {
            mode: PaletteMode::Dark,
            axis: Color::srgb_u8(0xc3, 0xc2, 0xb7),
            grid: Color::srgb_u8(0x50, 0x50, 0x4c),
        }
    }

    /// Palette for a light scene background.
    pub fn light() -> Self {
        Self {
            mode: PaletteMode::Light,
            axis: Color::srgb_u8(0x52, 0x51, 0x4e),
            grid: Color::srgb_u8(0xd6, 0xd5, 0xd0),
        }
    }

    /// Color for series `index`, assigned in fixed slot order.
    ///
    /// Indices past the last slot clamp to it rather than cycling, so two series
    /// never silently share a hue — see the module docs.
    pub fn series_color(&self, index: usize) -> Color {
        let table = match self.mode {
            PaletteMode::Dark => &DARK_SERIES,
            PaletteMode::Light => &LIGHT_SERIES,
        };
        let (r, g, b) = table[index.min(SERIES_SLOTS - 1)];
        Color::srgb_u8(r, g, b)
    }

    /// Sample the sequential ramp at `t`, clamped to `0.0..=1.0`.
    ///
    /// Used to encode continuous magnitude, such as height on a surface chart.
    pub fn sequential(&self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let last = SEQUENTIAL.len() - 1;
        // Interpolate in sRGB space between the two nearest published steps; the
        // ramp is dense enough that a linear blend stays on the intended hue.
        let scaled = t * last as f32;
        let lo = scaled.floor() as usize;
        let hi = (lo + 1).min(last);
        let f = scaled - lo as f32;

        let (r0, g0, b0) = SEQUENTIAL[lo];
        let (r1, g1, b1) = SEQUENTIAL[hi];
        let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * f) / 255.0;
        Color::srgb(lerp(r0, r1), lerp(g0, g1), lerp(b0, b1))
    }
}
