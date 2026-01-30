//! Colormap implementations for value-based coloring.
//!
//! Colormaps are useful for visualizing continuous data values as colors,
//! commonly used in scientific visualization (temperature, pressure, etc.).

use iced::Color;

/// Named colormaps for value-based coloring.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum ColormapName {
    /// Perceptually uniform colormap (blue → green → yellow).
    /// Colorblind-friendly and good for scientific visualization.
    Viridis,

    /// Similar to Viridis but with more purple/yellow tones.
    /// Also perceptually uniform.
    Plasma,

    /// Modern improved rainbow (red → orange → yellow → green → cyan → blue).
    /// Better than traditional rainbow for perceptual uniformity.
    Turbo,

    /// Classic heat map (black → red → yellow → white).
    /// Intuitive for temperature visualization.
    Heat,

    /// Simple grayscale (black → white).
    Grayscale,
}

impl ColormapName {
    /// Sample the colormap at position t, where t is in [0, 1].
    /// t=0 returns the color at the start, t=1 returns the color at the end.
    pub fn sample(&self, mut t: f32) -> Color {
        // Clamp t to [0, 1]
        t = t.clamp(0.0, 1.0);

        match self {
            ColormapName::Viridis => sample_viridis(t),
            ColormapName::Plasma => sample_plasma(t),
            ColormapName::Turbo => sample_turbo(t),
            ColormapName::Heat => sample_heat(t),
            ColormapName::Grayscale => {
                let v = t;
                Color::from_rgb(v, v, v)
            }
        }
    }
}

/// Helper function to linearly interpolate between two colors.
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::from_rgb(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
    )
}

/// Helper function to interpolate in a color palette using lookup table.
fn sample_palette(palette: &[(f32, Color)], t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);

    if t <= palette[0].0 {
        return palette[0].1;
    }
    if t >= palette[palette.len() - 1].0 {
        return palette[palette.len() - 1].1;
    }

    for i in 0..palette.len() - 1 {
        let (t0, c0) = palette[i];
        let (t1, c1) = palette[i + 1];
        if t >= t0 && t <= t1 {
            let local_t = (t - t0) / (t1 - t0);
            return lerp_color(c0, c1, local_t);
        }
    }

    palette[palette.len() - 1].1
}

/// Viridis colormap: perceptually uniform, colorblind-friendly.
fn sample_viridis(t: f32) -> Color {
    // Simplified viridis with 6 color stops
    let palette = [
        (0.0, Color::from_rgb(0.267, 0.004, 0.329)), // Dark purple
        (0.25, Color::from_rgb(0.282, 0.140, 0.458)), // Purple
        (0.5, Color::from_rgb(0.204, 0.286, 0.469)), // Blue
        (0.6, Color::from_rgb(0.128, 0.400, 0.369)), // Green-blue
        (0.75, Color::from_rgb(0.527, 0.510, 0.149)), // Yellow-green
        (1.0, Color::from_rgb(0.993, 0.906, 0.144)), // Yellow
    ];
    sample_palette(&palette, t)
}

/// Plasma colormap: perceptually uniform, high contrast.
fn sample_plasma(t: f32) -> Color {
    // Simplified plasma with 6 color stops
    let palette = [
        (0.0, Color::from_rgb(0.050, 0.030, 0.530)),  // Dark blue
        (0.25, Color::from_rgb(0.275, 0.005, 0.610)), // Purple
        (0.5, Color::from_rgb(0.553, 0.027, 0.416)),  // Magenta-red
        (0.6, Color::from_rgb(0.764, 0.190, 0.217)),  // Red
        (0.75, Color::from_rgb(0.960, 0.380, 0.113)), // Orange
        (1.0, Color::from_rgb(0.940, 0.975, 0.131)),  // Yellow
    ];
    sample_palette(&palette, t)
}

/// Turbo colormap: improved rainbow with better perceptual uniformity.
fn sample_turbo(t: f32) -> Color {
    // Simplified turbo with 7 color stops
    let palette = [
        (0.0, Color::from_rgb(0.180, 0.070, 0.450)), // Deep blue
        (0.2, Color::from_rgb(0.000, 0.300, 0.740)), // Blue
        (0.4, Color::from_rgb(0.000, 0.780, 0.870)), // Cyan
        (0.5, Color::from_rgb(0.000, 0.980, 0.600)), // Green-cyan
        (0.6, Color::from_rgb(0.850, 0.970, 0.110)), // Yellow
        (0.8, Color::from_rgb(0.970, 0.430, 0.000)), // Orange
        (1.0, Color::from_rgb(0.880, 0.000, 0.000)), // Red
    ];
    sample_palette(&palette, t)
}

/// Heat colormap: intuitive temperature visualization (black → red → yellow → white).
fn sample_heat(t: f32) -> Color {
    // 5 color stops for smooth transitions
    let palette = [
        (0.0, Color::from_rgb(0.000, 0.000, 0.000)),  // Black
        (0.25, Color::from_rgb(0.500, 0.000, 0.000)), // Dark red
        (0.5, Color::from_rgb(1.000, 0.000, 0.000)),  // Red
        (0.75, Color::from_rgb(1.000, 0.500, 0.000)), // Orange
        (1.0, Color::from_rgb(1.000, 1.000, 0.000)),  // Yellow
    ];
    sample_palette(&palette, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colormap_bounds() {
        // Test that colormaps clamp to valid range
        let c = ColormapName::Viridis.sample(1.5);
        assert!(c.r >= 0.0 && c.r <= 1.0);
        assert!(c.g >= 0.0 && c.g <= 1.0);
        assert!(c.b >= 0.0 && c.b <= 1.0);

        let c = ColormapName::Viridis.sample(-0.5);
        assert!(c.r >= 0.0 && c.r <= 1.0);
        assert!(c.g >= 0.0 && c.g <= 1.0);
        assert!(c.b >= 0.0 && c.b <= 1.0);
    }

    #[test]
    fn test_colormap_endpoints() {
        // Test that endpoints match expected colors
        let start = ColormapName::Viridis.sample(0.0);
        let end = ColormapName::Viridis.sample(1.0);

        // Start should be dark purple-ish
        assert!(start.r < 0.5 && start.b > 0.2);

        // End should be yellow-ish
        assert!(end.r > 0.9 && end.g > 0.8 && end.b < 0.3);
    }
}
