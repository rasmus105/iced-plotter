//! Shader-based rendering for the plotter using iced's wgpu backend.

use crate::gpu_types::{RawPoint, Uniforms};
use crate::pipeline::Pipeline;
use crate::plotter::{ColorMode, PlotPoints, PlotSeries, Plotter, PlotterOptions};
use crate::ticks::compute_ticks;

use iced::Rectangle;
use iced::mouse::Cursor;
use iced::wgpu;
use iced::widget::shader::{self, Viewport};

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

#[derive(Debug, Clone)]
pub struct TickInfo {
    pub x_ticks: Vec<f32>,
    pub y_ticks: Vec<f32>,
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
    /// Pre-computed grid line vertices
    grid_vertices: Vec<RawPoint>,
    /// Series boundaries to prevent line connections between series
    #[allow(dead_code)]
    series_boundaries: Vec<usize>,
    pub tick_info: TickInfo,
}

impl PlotterPrimitive {
    /// Create a new primitive from plotter data.
    pub fn new<'a>(
        series: &'a [PlotSeries<'a>],
        bounds: Rectangle,
        options: &PlotterOptions,
    ) -> Self {
        // Default to showing both markers and lines
        let config = RenderConfig {
            show_markers: true,
            show_lines: true,
        };

        // First pass: collect all points and calculate data ranges, tracking series boundaries
        let mut all_points_with_colors: Vec<(f32, f32, ColorMode<'a>)> = Vec::new();
        let mut series_boundaries: Vec<usize> = Vec::new(); // Stores start index of each series
        let mut x_min = f32::INFINITY;
        let mut x_max = f32::NEG_INFINITY;
        let mut y_min = f32::INFINITY;
        let mut y_max = f32::NEG_INFINITY;

        for s in series {
            series_boundaries.push(all_points_with_colors.len()); // Record start of this series
            match &s.points {
                PlotPoints::Owned(points) => {
                    for p in points {
                        all_points_with_colors.push((p.x, p.y, s.style.color.clone()));
                        x_min = x_min.min(p.x);
                        x_max = x_max.max(p.x);
                        y_min = y_min.min(p.y);
                        y_max = y_max.max(p.y);
                    }
                }
                PlotPoints::Borrowed(points) => {
                    for p in *points {
                        all_points_with_colors.push((p.x, p.y, s.style.color.clone()));
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
                        all_points_with_colors.push((x, y, s.style.color.clone()));
                        x_min = x_min.min(x);
                        x_max = x_max.max(x);
                        y_min = y_min.min(y);
                        y_max = y_max.max(y);
                    }
                }
            }
        }

        // Handle empty data and constant y values
        if all_points_with_colors.is_empty() {
            x_min = 0.0;
            x_max = 1.0;
            y_min = 0.0;
            y_max = 1.0;
        } else if (y_max - y_min).abs() < f32::EPSILON {
            // Handle constant y values
            y_min -= 0.5;
            y_max += 0.5;
        }

        // Use padding from options, with a default
        let padding = options.padding;

        // Get marker radius and line width from the first series (if available)
        let marker_radius = series.first().map(|s| s.style.marker_size).unwrap_or(4.0);
        let line_width = series.first().map(|s| s.style.line_width).unwrap_or(2.0);

        let uniforms = Uniforms {
            viewport_size: [bounds.width, bounds.height],
            x_range: [x_min, x_max],
            y_range: [y_min, y_max],
            padding: [padding, padding],
            marker_radius,
            line_width,
        };

        // Second pass: convert points with color mode to final RawPoints
        let all_points =
            Self::apply_color_mode(&all_points_with_colors, x_min, x_max, y_min, y_max);

        // Generate line vertices (thick lines as quads), passing series boundaries
        let line_vertices = if config.show_lines {
            Self::generate_line_vertices(&all_points, &series_boundaries, &uniforms)
        } else {
            Vec::new()
        };

        let grid_vertices = Self::generate_grid_vertices(options, &uniforms);

        let x_ticks = compute_ticks(x_min, x_max, &options.x_axis.ticks);
        let y_ticks = compute_ticks(y_min, y_max, &options.y_axis.ticks);
        let tick_info = TickInfo { x_ticks, y_ticks };

        Self {
            points: all_points,
            line_vertices,
            uniforms,
            config,
            grid_vertices,
            series_boundaries,
            tick_info,
        }
    }

    /// Apply color modes to raw point data, computing final RGBA colors.
    fn apply_color_mode(
        points_with_colors: &[(f32, f32, ColorMode<'_>)],
        _x_min: f32,
        _x_max: f32,
        y_min: f32,
        y_max: f32,
    ) -> Vec<RawPoint> {
        let mut result = Vec::with_capacity(points_with_colors.len());

        for (idx, (x, y, color_mode)) in points_with_colors.iter().enumerate() {
            let color = match color_mode {
                ColorMode::Solid(c) => *c,
                ColorMode::ValueGradient { low, high, values } => {
                    let value = values.as_ref().map(|v| v[idx]).unwrap_or(*y);
                    let value_min = if let Some(v) = values {
                        v.iter().fold(f32::INFINITY, |a, &b| a.min(b))
                    } else {
                        y_min
                    };
                    let value_max = if let Some(v) = values {
                        v.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
                    } else {
                        y_max
                    };

                    let t = if (value_max - value_min).abs() < f32::EPSILON {
                        0.5
                    } else {
                        (value - value_min) / (value_max - value_min)
                    };

                    Self::lerp_color(*low, *high, t)
                }
                ColorMode::IndexGradient { start, end } => {
                    let total = points_with_colors.len() as f32;
                    let t = if total > 1.0 {
                        idx as f32 / (total - 1.0)
                    } else {
                        0.5
                    };
                    Self::lerp_color(*start, *end, t)
                }
                ColorMode::Colormap { name, values } => {
                    let value = values.as_ref().map(|v| v[idx]).unwrap_or(*y);
                    let value_min = if let Some(v) = values {
                        v.iter().fold(f32::INFINITY, |a, &b| a.min(b))
                    } else {
                        y_min
                    };
                    let value_max = if let Some(v) = values {
                        v.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
                    } else {
                        y_max
                    };

                    let t = if (value_max - value_min).abs() < f32::EPSILON {
                        0.5
                    } else {
                        (value - value_min) / (value_max - value_min)
                    };

                    name.sample(t)
                }
            };

            result.push(RawPoint::new(*x, *y, [color.r, color.g, color.b, color.a]));
        }

        result
    }

    /// Linearly interpolate between two colors.
    fn lerp_color(a: iced::Color, b: iced::Color, t: f32) -> iced::Color {
        let t = t.clamp(0.0, 1.0);
        iced::Color::from_rgb(
            a.r + (b.r - a.r) * t,
            a.g + (b.g - a.g) * t,
            a.b + (b.b - a.b) * t,
        )
    }

    /// Generate line vertices as quads for thick lines, respecting series boundaries.
    fn generate_line_vertices(
        points: &[RawPoint],
        series_boundaries: &[usize],
        uniforms: &Uniforms,
    ) -> Vec<RawPoint> {
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

        // Process each series independently to avoid lines between series
        for series_idx in 0..series_boundaries.len() {
            let start_idx = series_boundaries[series_idx];
            let end_idx = if series_idx + 1 < series_boundaries.len() {
                series_boundaries[series_idx + 1]
            } else {
                points.len()
            };

            // Generate lines within this series only
            for window_idx in start_idx..end_idx - 1 {
                let p0 = &points[window_idx];
                let p1 = &points[window_idx + 1];
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
        }

        vertices
    }

    fn generate_grid_vertices(options: &PlotterOptions, uniforms: &Uniforms) -> Vec<RawPoint> {
        let mut vertices = Vec::new();

        let padding_x = uniforms.padding[0];
        let padding_y = uniforms.padding[1];
        let plot_width = uniforms.viewport_size[0] - 2.0 * padding_x;
        let plot_height = uniforms.viewport_size[1] - 2.0 * padding_y;
        let x_range = uniforms.x_range;
        let y_range = uniforms.y_range;

        let push_line_quad =
            |vertices: &mut Vec<RawPoint>,
             x0: f32,
             y0: f32,
             x1: f32,
             y1: f32,
             half_width: f32,
             color: [f32; 4]| {
                let dx = x1 - x0;
                let dy = y1 - y0;
                let len = (dx * dx + dy * dy).sqrt();
                if len < 0.001 {
                    return;
                }
                let nx = -dy / len * half_width;
                let ny = dx / len * half_width;

                let v0 = RawPoint::new(x0 + nx, y0 + ny, color);
                let v1 = RawPoint::new(x0 - nx, y0 - ny, color);
                let v2 = RawPoint::new(x1 + nx, y1 + ny, color);
                let v3 = RawPoint::new(x1 - nx, y1 - ny, color);

                vertices.push(v0);
                vertices.push(v1);
                vertices.push(v2);

                vertices.push(v1);
                vertices.push(v3);
                vertices.push(v2);
            };

        if options.grid.show {
            let grid_color = [
                options.grid.color.r,
                options.grid.color.g,
                options.grid.color.b,
                options.grid.color.a,
            ];
            let grid_half = options.grid.line_width / 2.0;

            let x_ticks = compute_ticks(x_range[0], x_range[1], &options.x_axis.ticks);
            for &v in &x_ticks {
                if v < x_range[0] || v > x_range[1] {
                    continue;
                }
                let x_norm = (v - x_range[0]) / (x_range[1] - x_range[0]);
                let screen_x = padding_x + x_norm * plot_width;
                push_line_quad(
                    &mut vertices,
                    screen_x,
                    padding_y,
                    screen_x,
                    padding_y + plot_height,
                    grid_half,
                    grid_color,
                );
            }

            let y_ticks = compute_ticks(y_range[0], y_range[1], &options.y_axis.ticks);
            for &v in &y_ticks {
                if v < y_range[0] || v > y_range[1] {
                    continue;
                }
                let y_norm = (v - y_range[0]) / (y_range[1] - y_range[0]);
                let screen_y = padding_y + (1.0 - y_norm) * plot_height;
                push_line_quad(
                    &mut vertices,
                    padding_x,
                    screen_y,
                    padding_x + plot_width,
                    screen_y,
                    grid_half,
                    grid_color,
                );
            }
        }

        if options.x_axis.show {
            let color = [
                options.x_axis.color.r,
                options.x_axis.color.g,
                options.x_axis.color.b,
                options.x_axis.color.a,
            ];
            let half = options.x_axis.line_width / 2.0;
            let screen_y = padding_y + plot_height;
            push_line_quad(
                &mut vertices,
                padding_x,
                screen_y,
                padding_x + plot_width,
                screen_y,
                half,
                color,
            );
        }

        if options.y_axis.show {
            let color = [
                options.y_axis.color.r,
                options.y_axis.color.g,
                options.y_axis.color.b,
                options.y_axis.color.a,
            ];
            let half = options.y_axis.line_width / 2.0;
            let screen_x = padding_x;
            push_line_quad(
                &mut vertices,
                screen_x,
                padding_y,
                screen_x,
                padding_y + plot_height,
                half,
                color,
            );
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
            &self.grid_vertices,
        );
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        if !self.grid_vertices.is_empty() {
            pipeline.render_grid(render_pass, self.grid_vertices.len() as u32);
        }

        if self.config.show_lines {
            pipeline.render_lines(render_pass, self.line_vertices.len() as u32);
        }

        if self.config.show_markers {
            pipeline.render_markers(render_pass, self.points.len() as u32);
        }

        true
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
