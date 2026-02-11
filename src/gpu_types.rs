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
    /// Perpendicular distance from line center, normalised to [0, 1] at the
    /// original (non-extended) half-width.  Used by the line fragment shader
    /// for edge anti-aliasing.  Ignored for markers / grid.
    pub edge_distance: f32,
}

impl RawPoint {
    pub fn new(x: f32, y: f32, color: [f32; 4]) -> Self {
        Self {
            position: [x, y],
            color,
            shape: 0, // Default to circle
            edge_distance: 0.0,
        }
    }

    /// Create with explicit marker shape
    pub fn with_shape(x: f32, y: f32, color: [f32; 4], shape: u32) -> Self {
        Self {
            position: [x, y],
            color,
            shape,
            edge_distance: 0.0,
        }
    }

    /// Create a line vertex with edge distance for anti-aliasing.
    /// `edge_dist` is the normalised perpendicular distance from the line
    /// centre: 0.0 at the centre, 1.0 at the original half-width edge.
    pub fn with_edge_distance(x: f32, y: f32, color: [f32; 4], edge_dist: f32) -> Self {
        Self {
            position: [x, y],
            color,
            shape: 0,
            edge_distance: edge_dist,
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
