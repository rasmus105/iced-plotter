//! GPU-compatible data types for shader rendering.

use bytemuck::{Pod, Zeroable};

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
    pub fn new(x: f32, y: f32, color: [f32; 4]) -> Self {
        Self {
            position: [x, y],
            color,
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
