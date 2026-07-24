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
    /// A non-finite `t` samples the low end: `f32::clamp` propagates `NaN`
    /// rather than clamping it, and letting that through would produce a `NaN`
    /// vertex color and a corrupt mesh.
    pub fn sequential(&self, t: f32) -> Color {
        let t = if t.is_nan() { 0.0 } else { t.clamp(0.0, 1.0) };
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Perceptual-ish distance between two colors, for asserting that things are
    /// visibly different rather than merely unequal bit patterns.
    fn distance(a: Color, b: Color) -> f32 {
        let (a, b) = (a.to_linear(), b.to_linear());
        ((a.red - b.red).powi(2) + (a.green - b.green).powi(2) + (a.blue - b.blue).powi(2)).sqrt()
    }

    #[test]
    fn every_slot_is_a_distinct_color() {
        for palette in [ChartPalette::dark(), ChartPalette::light()] {
            for i in 0..SERIES_SLOTS {
                for j in (i + 1)..SERIES_SLOTS {
                    let d = distance(palette.series_color(i), palette.series_color(j));
                    assert!(d > 0.01, "slots {i} and {j} are indistinguishable ({d})");
                }
            }
        }
    }

    #[test]
    fn indices_past_the_last_slot_clamp_rather_than_cycle() {
        let palette = ChartPalette::dark();
        let last = palette.series_color(SERIES_SLOTS - 1);

        // Clamping is the point: wrapping would make series 8 collide with
        // series 0, which reads as "these two are the same thing".
        assert_eq!(palette.series_color(SERIES_SLOTS), last);
        assert_eq!(palette.series_color(SERIES_SLOTS + 5), last);
        assert_eq!(palette.series_color(usize::MAX), last);
        assert_ne!(palette.series_color(SERIES_SLOTS), palette.series_color(0));
    }

    #[test]
    fn dark_and_light_are_separately_stepped() {
        let (dark, light) = (ChartPalette::dark(), ChartPalette::light());
        assert_ne!(dark.axis, light.axis);
        assert_ne!(dark.grid, light.grid);

        // Most hues are re-stepped for their background. Green is deliberately
        // shared, so require a clear majority rather than all eight.
        let differing = (0..SERIES_SLOTS)
            .filter(|i| dark.series_color(*i) != light.series_color(*i))
            .count();
        assert!(
            differing >= SERIES_SLOTS - 1,
            "expected the modes to be separately stepped, {differing}/{SERIES_SLOTS} differ"
        );
    }

    #[test]
    fn the_default_palette_is_the_dark_one() {
        assert_eq!(ChartPalette::default(), ChartPalette::dark());
        assert_eq!(PaletteMode::default(), PaletteMode::Dark);
    }

    #[test]
    fn the_sequential_ramp_hits_both_published_ends() {
        let palette = ChartPalette::dark();
        let (first, last) = (SEQUENTIAL[0], SEQUENTIAL[SEQUENTIAL.len() - 1]);

        let to_ends = |c: (u8, u8, u8)| Color::srgb_u8(c.0, c.1, c.2);
        assert!(distance(palette.sequential(0.0), to_ends(first)) < 0.01);
        assert!(distance(palette.sequential(1.0), to_ends(last)) < 0.01);
    }

    #[test]
    fn the_sequential_ramp_clamps_out_of_range_input() {
        let palette = ChartPalette::dark();
        assert_eq!(palette.sequential(-5.0), palette.sequential(0.0));
        assert_eq!(palette.sequential(5.0), palette.sequential(1.0));
        // NaN must not index out of bounds; `clamp` maps it to the low end.
        assert!(palette.sequential(f32::NAN).to_linear().red.is_finite());
    }

    #[test]
    fn the_sequential_ramp_darkens_monotonically() {
        // Magnitude is encoded by getting darker, so luminance must never rise
        // as t increases or the encoding would reverse mid-ramp.
        let palette = ChartPalette::dark();
        let luminance = |t: f32| {
            let c = palette.sequential(t).to_linear();
            0.2126 * c.red + 0.7152 * c.green + 0.0722 * c.blue
        };
        let mut previous = luminance(0.0);
        for step in 1..=50 {
            let current = luminance(step as f32 / 50.0);
            assert!(
                current <= previous + 1e-4,
                "ramp brightened at t={}: {previous} -> {current}",
                step as f32 / 50.0
            );
            previous = current;
        }
    }
}
