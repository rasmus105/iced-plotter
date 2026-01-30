//! Shader-based rendering for the plotter using iced's wgpu backend.

use crate::gpu_types::{RawPoint, Uniforms};
use crate::pipeline::Pipeline;
use crate::plotter::{PlotPoints, PlotSeries, Plotter, PlotterOptions};

use iced::mouse::Cursor;
use iced::wgpu;
use iced::widget::shader::{self, Viewport};
use iced::Rectangle;

/// State for the shader program (zoom/pan state, etc.).
#[derive(Default)]
pub struct PlotterState {
    pub is_dragging: bool,
}

/// Configuration for what to render.
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderConfig {
    pub show_markers: bool,
    pub show_lines: bool,
}

/// The primitive that holds all data to be rendered on the GPU.
#[derive(Debug)]
pub struct PlotterPrimitive {
    /// Points to render as markers
    points: Vec<RawPoint>,
    /// Pre-computed line vertices (triangles for thick lines)
    line_vertices: Vec<RawPoint>,
    /// Uniform data for coordinate transformation
    uniforms: Uniforms,
    /// Config for what to render
    config: RenderConfig,
}

impl PlotterPrimitive {
    /// Create a new primitive from plotter data.
    pub fn new(series: &[PlotSeries], bounds: Rectangle, _options: &PlotterOptions) -> Self {
        // Default to showing both markers and lines
        let config = RenderConfig {
            show_markers: true,
            show_lines: true,
        };

        // Collect all points and calculate data ranges in a single pass
        let mut all_points: Vec<RawPoint> = Vec::new();
        let mut x_min = f32::INFINITY;
        let mut x_max = f32::NEG_INFINITY;
        let mut y_min = f32::INFINITY;
        let mut y_max = f32::NEG_INFINITY;

        for s in series {
            let color = s.color;
            let color_array = [color.r, color.g, color.b, color.a];
            match &s.points {
                PlotPoints::Owned(points) => {
                    for p in points {
                        all_points.push(RawPoint::new(p.x, p.y, color_array));
                        x_min = x_min.min(p.x);
                        x_max = x_max.max(p.x);
                        y_min = y_min.min(p.y);
                        y_max = y_max.max(p.y);
                    }
                }
                PlotPoints::Borrowed(points) => {
                    for p in *points {
                        all_points.push(RawPoint::new(p.x, p.y, color_array));
                        x_min = x_min.min(p.x);
                        x_max = x_max.max(p.x);
                        y_min = y_min.min(p.y);
                        y_max = y_max.max(p.y);
                    }
                }
                PlotPoints::Generator(generator) => {
                    let (x_min_range, x_max_range) = generator.x_range;
                    let x_span = x_max_range - x_min_range;
                    for i in 0..generator.points {
                        let t = i as f32 / (generator.points - 1).max(1) as f32;
                        let x = x_min_range + t * x_span;
                        let y = (generator.function)(x);
                        all_points.push(RawPoint::new(x, y, color_array));
                        x_min = x_min.min(x);
                        x_max = x_max.max(x);
                        y_min = y_min.min(y);
                        y_max = y_max.max(y);
                    }
                }
            }
        }

        // Handle empty data and constant y values
        if all_points.is_empty() {
            x_min = 0.0;
            x_max = 1.0;
            y_min = 0.0;
            y_max = 1.0;
        } else if (y_max - y_min).abs() < f32::EPSILON {
            // Handle constant y values
            y_min -= 0.5;
            y_max += 0.5;
        }

        let padding = 50.0;

        let uniforms = Uniforms {
            viewport_size: [bounds.width, bounds.height],
            x_range: [x_min, x_max],
            y_range: [y_min, y_max],
            padding: [padding, padding],
            marker_radius: 4.0,
            line_width: 2.0,
        };

        // Generate line vertices (thick lines as quads)
        let line_vertices = if config.show_lines {
            Self::generate_line_vertices(&all_points, &uniforms)
        } else {
            Vec::new()
        };

        Self {
            points: all_points,
            line_vertices,
            uniforms,
            config,
        }
    }

    /// Generate line vertices as quads for thick lines.
    fn generate_line_vertices(points: &[RawPoint], uniforms: &Uniforms) -> Vec<RawPoint> {
        if points.len() < 2 {
            return Vec::new();
        }

        let mut vertices = Vec::with_capacity((points.len() - 1) * 6);

        let plot_width = uniforms.viewport_size[0] - 2.0 * uniforms.padding[0];
        let plot_height = uniforms.viewport_size[1] - 2.0 * uniforms.padding[1];
        let x_range = uniforms.x_range;
        let y_range = uniforms.y_range;
        let half_width = uniforms.line_width / 2.0;

        // Convert data coords to screen coords
        let to_screen = |x: f32, y: f32| -> (f32, f32) {
            let x_norm = (x - x_range[0]) / (x_range[1] - x_range[0]);
            let y_norm = (y - y_range[0]) / (y_range[1] - y_range[0]);
            let screen_x = uniforms.padding[0] + x_norm * plot_width;
            let screen_y = uniforms.padding[1] + (1.0 - y_norm) * plot_height;
            (screen_x, screen_y)
        };

        for window in points.windows(2) {
            let p0 = &window[0];
            let p1 = &window[1];
            let x0 = p0.position[0];
            let y0 = p0.position[1];
            let x1 = p1.position[0];
            let y1 = p1.position[1];
            let color = p0.color;

            let (sx0, sy0) = to_screen(x0, y0);
            let (sx1, sy1) = to_screen(x1, y1);

            // Calculate perpendicular direction
            let dx = sx1 - sx0;
            let dy = sy1 - sy0;
            let len = (dx * dx + dy * dy).sqrt();

            if len < 0.001 {
                continue; // Skip zero-length segments
            }

            let nx = -dy / len * half_width;
            let ny = dx / len * half_width;

            // Create quad (2 triangles)
            let v0 = RawPoint::new(sx0 + nx, sy0 + ny, color);
            let v1 = RawPoint::new(sx0 - nx, sy0 - ny, color);
            let v2 = RawPoint::new(sx1 + nx, sy1 + ny, color);
            let v3 = RawPoint::new(sx1 - nx, sy1 - ny, color);

            // Triangle 1
            vertices.push(v0);
            vertices.push(v1);
            vertices.push(v2);

            // Triangle 2
            vertices.push(v1);
            vertices.push(v3);
            vertices.push(v2);
        }

        vertices
    }
}

impl shader::Primitive for PlotterPrimitive {
    type Pipeline = Pipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        pipeline.update(
            device,
            queue,
            &self.uniforms,
            &self.points,
            &self.line_vertices,
        );
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        // Draw lines first (behind markers)
        if self.config.show_lines {
            pipeline.render_lines(render_pass, self.line_vertices.len() as u32);
        }

        // Draw markers on top
        if self.config.show_markers {
            pipeline.render_markers(render_pass, self.points.len() as u32);
        }

        true // We handled the rendering
    }
}

impl shader::Pipeline for Pipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        Pipeline::new(device, queue, format)
    }
}

impl<Message> shader::Program<Message> for Plotter<'_> {
    type State = PlotterState;
    type Primitive = PlotterPrimitive;

    fn draw(&self, _state: &Self::State, _cursor: Cursor, bounds: Rectangle) -> Self::Primitive {
        PlotterPrimitive::new(&self.series, bounds, &self.options)
    }
}
