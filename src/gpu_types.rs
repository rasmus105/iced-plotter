//! GPU-compatible data types for shader rendering.

use bytemuck::{Pod, Zeroable};

/// A point with position, color, and rendering options, ready for GPU upload.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct RawPoint {
    /// Position in data coordinates (x, y)
    pub position: [f32; 2],
    /// RGBA color
    pub color: [f32; 4],
    /// Marker shape as u32 (MarkerShape enum value)
    pub shape: u32,
    /// Padding for alignment (16-byte boundaries)
    pub _padding: u32,
}

impl RawPoint {
    pub fn new(x: f32, y: f32, color: [f32; 4]) -> Self {
        Self {
            position: [x, y],
            color,
            shape: 0, // Default to circle
            _padding: 0,
        }
    }

    /// Create with explicit marker shape
    pub fn with_shape(x: f32, y: f32, color: [f32; 4], shape: u32) -> Self {
        Self {
            position: [x, y],
            color,
            shape,
            _padding: 0,
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

/// A vertex for line rendering with distance tracking for patterns.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LineVertex {
    /// Position in screen coordinates
    pub position: [f32; 2],
    /// RGBA color
    pub color: [f32; 4],
    /// Distance along line segment (for pattern rendering)
    pub distance: f32,
    /// Line pattern as u32 (LinePattern enum value)
    pub pattern: u32,
}

impl LineVertex {
    pub fn new(x: f32, y: f32, color: [f32; 4], distance: f32, pattern: u32) -> Self {
        Self {
            position: [x, y],
            color,
            distance,
            pattern,
        }
    }
}

/// A vertex for fill rendering (area under curves).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FillVertex {
    /// Position in screen coordinates
    pub position: [f32; 2],
    /// RGBA color
    pub color: [f32; 4],
}

impl FillVertex {
    pub fn new(x: f32, y: f32, color: [f32; 4]) -> Self {
        Self {
            position: [x, y],
            color,
        }
    }
}
