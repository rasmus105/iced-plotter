//! GPU-compatible data types for shader rendering.

use bytemuck::{Pod, Zeroable};
use iced::Color;

/// A point with position and color, ready for GPU upload.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct RawPoint {
    /// Position in data coordinates (x, y)
    pub position: [f32; 2],
    /// RGBA color
    pub color: [f32; 4],
}

impl RawPoint {
    pub fn new(x: f64, y: f64, color: Color) -> Self {
        Self {
            position: [x as f32, y as f32],
            color: [color.r, color.g, color.b, color.a],
        }
    }
}

/// A vertex for line rendering (position + color + side info for thickness).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LineVertex {
    /// Screen position (x, y)
    pub position: [f32; 2],
    /// RGBA color
    pub color: [f32; 4],
}

impl LineVertex {
    pub fn new(x: f32, y: f32, color: Color) -> Self {
        Self {
            position: [x, y],
            color: [color.r, color.g, color.b, color.a],
        }
    }
}

/// Uniform data passed to shaders for coordinate transformation.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Uniforms {
    /// Viewport size in pixels (width, height)
    pub viewport_size: [f32; 2],
    /// Data X range (min, max)
    pub x_range: [f32; 2],
    /// Data Y range (min, max)
    pub y_range: [f32; 2],
    /// Padding in pixels (horizontal, vertical)
    pub padding: [f32; 2],
    /// Marker radius in pixels
    pub marker_radius: f32,
    /// Line width in pixels
    pub line_width: f32,
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            viewport_size: [800.0, 600.0],
            x_range: [0.0, 1.0],
            y_range: [0.0, 1.0],
            padding: [50.0, 50.0],
            marker_radius: 4.0,
            line_width: 2.0,
        }
    }
}

/// Configuration for what to render.
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderConfig {
    pub show_markers: bool,
    pub show_lines: bool,
}
