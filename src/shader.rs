//! Shader-based rendering for the plotter using iced's wgpu backend.

use crate::gpu_types::{RawPoint, Uniforms};
use crate::pipeline::Pipeline;
use crate::plotter::{ColorMode, PlotPoints, PlotSeries, Plotter, PlotterOptions, ViewState};
use crate::ticks::compute_ticks;

use iced::keyboard;
use iced::mouse::Cursor;
use iced::wgpu;
use iced::widget::shader::{self, Viewport};
use iced::{mouse, Event, Point, Rectangle};

// ================================================================================
// Interaction State
// ================================================================================

/// Tracks which interaction mode the user is currently in.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InteractionMode {
    #[default]
    Idle,
    Panning,
    /// Ctrl+drag rectangle zoom selection.
    ZoomSelecting,
}

/// State for elastic spring-back animation.
#[derive(Debug, Clone)]
pub struct ElasticState {
    /// The over-scrolled view at the start of the animation.
    pub from_x: Option<(f32, f32)>,
    pub from_y: Option<(f32, f32)>,
    /// The clamped target view to animate towards.
    pub to_x: Option<(f32, f32)>,
    pub to_y: Option<(f32, f32)>,
    /// When the animation started.
    pub start_time: std::time::Instant,
    /// Duration of the animation in milliseconds.
    pub duration_ms: u64,
}

/// State for the shader program (persists across frames via iced's widget tree).
#[derive(Default)]
pub struct PlotterState {
    /// Current interaction mode.
    pub interaction_mode: InteractionMode,
    /// Screen position where the drag started (relative to widget bounds).
    pub drag_start: Option<Point>,
    /// View state snapshot at the start of a drag (for computing deltas).
    pub drag_start_view: Option<ViewState>,
    /// Last known cursor position (absolute screen coords).
    pub last_cursor: Option<Point>,
    /// Timestamp of last click for double-click detection.
    pub last_click_time: Option<std::time::Instant>,
    /// Current keyboard modifiers (for Ctrl detection).
    pub modifiers: keyboard::Modifiers,
    /// Current position during zoom selection (relative to widget bounds).
    pub zoom_select_current: Option<Point>,
    /// Active elastic animation (spring-back after over-scroll).
    pub elastic_animation: Option<ElasticState>,
}

// ================================================================================
// Render Config & Primitive
// ================================================================================

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
    /// Selection rectangle overlay vertices (if zoom-selecting)
    selection_vertices: Vec<RawPoint>,
    /// Series boundaries to prevent line connections between series
    #[allow(dead_code)]
    series_boundaries: Vec<usize>,
    pub tick_info: TickInfo,
}

impl PlotterPrimitive {
    /// Create a new primitive from plotter data.
    ///
    /// `view_x_range` and `view_y_range` are the resolved visible ranges
    /// (already accounting for ViewState auto-fit).
    /// `selection_rect` is an optional screen-space rectangle for zoom selection overlay.
    pub fn new<'a>(
        series: &'a [PlotSeries<'a>],
        bounds: Rectangle,
        options: &PlotterOptions,
        view_x_range: [f32; 2],
        view_y_range: [f32; 2],
        selection_rect: Option<(Point, Point)>,
    ) -> Self {
        let config = RenderConfig {
            show_markers: true,
            show_lines: true,
        };

        // Collect all points with color info, tracking series boundaries
        let mut all_points_with_colors: Vec<(f32, f32, ColorMode<'a>)> = Vec::new();
        let mut series_boundaries: Vec<usize> = Vec::new();

        // We still need data-space min/max for color gradient normalization
        let mut data_y_min = f32::INFINITY;
        let mut data_y_max = f32::NEG_INFINITY;

        for s in series {
            series_boundaries.push(all_points_with_colors.len());
            match &s.points {
                PlotPoints::Owned(points) => {
                    for p in points {
                        all_points_with_colors.push((p.x, p.y, s.style.color.clone()));
                        data_y_min = data_y_min.min(p.y);
                        data_y_max = data_y_max.max(p.y);
                    }
                }
                PlotPoints::Borrowed(points) => {
                    for p in *points {
                        all_points_with_colors.push((p.x, p.y, s.style.color.clone()));
                        data_y_min = data_y_min.min(p.y);
                        data_y_max = data_y_max.max(p.y);
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
                        data_y_min = data_y_min.min(y);
                        data_y_max = data_y_max.max(y);
                    }
                }
            }
        }

        // Handle empty data
        if all_points_with_colors.is_empty() {
            data_y_min = 0.0;
            data_y_max = 1.0;
        } else if (data_y_max - data_y_min).abs() < f32::EPSILON {
            data_y_min -= 0.5;
            data_y_max += 0.5;
        }

        let padding = options.padding;
        let marker_radius = series.first().map(|s| s.style.marker_size).unwrap_or(4.0);
        let line_width = series.first().map(|s| s.style.line_width).unwrap_or(2.0);

        // Use the view ranges (not data ranges) for rendering
        let uniforms = Uniforms {
            viewport_size: [bounds.width, bounds.height],
            x_range: view_x_range,
            y_range: view_y_range,
            padding: [padding, padding],
            marker_radius,
            line_width,
        };

        // Apply color mode using *data* y range for gradient normalization
        let all_points = Self::apply_color_mode(
            &all_points_with_colors,
            view_x_range[0],
            view_x_range[1],
            data_y_min,
            data_y_max,
        );

        let line_vertices = if config.show_lines {
            Self::generate_line_vertices(&all_points, &series_boundaries, &uniforms)
        } else {
            Vec::new()
        };

        let grid_vertices = Self::generate_grid_vertices(options, &uniforms);

        // Generate selection rectangle overlay
        let selection_vertices = if let Some((start, end)) = selection_rect {
            Self::generate_selection_rect(start, end)
        } else {
            Vec::new()
        };

        let x_ticks = compute_ticks(view_x_range[0], view_x_range[1], &options.x_axis.ticks);
        let y_ticks = compute_ticks(view_y_range[0], view_y_range[1], &options.y_axis.ticks);
        let tick_info = TickInfo { x_ticks, y_ticks };

        Self {
            points: all_points,
            line_vertices,
            uniforms,
            config,
            grid_vertices,
            selection_vertices,
            series_boundaries,
            tick_info,
        }
    }

    /// Generate the selection rectangle as screen-space quads.
    /// Renders a semi-transparent fill with a solid border.
    fn generate_selection_rect(start: Point, end: Point) -> Vec<RawPoint> {
        let mut vertices = Vec::new();

        let x0 = start.x.min(end.x);
        let y0 = start.y.min(end.y);
        let x1 = start.x.max(end.x);
        let y1 = start.y.max(end.y);

        // Semi-transparent fill
        let fill_color = [0.3, 0.5, 0.8, 0.15];
        // Two triangles for the fill quad
        vertices.push(RawPoint::new(x0, y0, fill_color));
        vertices.push(RawPoint::new(x1, y0, fill_color));
        vertices.push(RawPoint::new(x0, y1, fill_color));
        vertices.push(RawPoint::new(x1, y0, fill_color));
        vertices.push(RawPoint::new(x1, y1, fill_color));
        vertices.push(RawPoint::new(x0, y1, fill_color));

        // Border lines (1.5px thick)
        let border_color = [0.4, 0.6, 0.9, 0.8];
        let half = 1.0;

        let push_border_line = |verts: &mut Vec<RawPoint>, ax: f32, ay: f32, bx: f32, by: f32| {
            let dx = bx - ax;
            let dy = by - ay;
            let len = (dx * dx + dy * dy).sqrt();
            if len < 0.001 {
                return;
            }
            let nx = -dy / len * half;
            let ny = dx / len * half;

            let v0 = RawPoint::new(ax + nx, ay + ny, border_color);
            let v1 = RawPoint::new(ax - nx, ay - ny, border_color);
            let v2 = RawPoint::new(bx + nx, by + ny, border_color);
            let v3 = RawPoint::new(bx - nx, by - ny, border_color);

            verts.push(v0);
            verts.push(v1);
            verts.push(v2);
            verts.push(v1);
            verts.push(v3);
            verts.push(v2);
        };

        // Top, bottom, left, right borders
        push_border_line(&mut vertices, x0, y0, x1, y0);
        push_border_line(&mut vertices, x0, y1, x1, y1);
        push_border_line(&mut vertices, x0, y0, x0, y1);
        push_border_line(&mut vertices, x1, y0, x1, y1);

        vertices
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

        let to_screen = |x: f32, y: f32| -> (f32, f32) {
            let x_norm = (x - x_range[0]) / (x_range[1] - x_range[0]);
            let y_norm = (y - y_range[0]) / (y_range[1] - y_range[0]);
            let screen_x = uniforms.padding[0] + x_norm * plot_width;
            let screen_y = uniforms.padding[1] + (1.0 - y_norm) * plot_height;
            (screen_x, screen_y)
        };

        for series_idx in 0..series_boundaries.len() {
            let start_idx = series_boundaries[series_idx];
            let end_idx = if series_idx + 1 < series_boundaries.len() {
                series_boundaries[series_idx + 1]
            } else {
                points.len()
            };

            if end_idx <= start_idx + 1 {
                continue;
            }

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

                let dx = sx1 - sx0;
                let dy = sy1 - sy0;
                let len = (dx * dx + dy * dy).sqrt();

                if len < 0.001 {
                    continue;
                }

                let nx = -dy / len * half_width;
                let ny = dx / len * half_width;

                let v0 = RawPoint::new(sx0 + nx, sy0 + ny, color);
                let v1 = RawPoint::new(sx0 - nx, sy0 - ny, color);
                let v2 = RawPoint::new(sx1 + nx, sy1 + ny, color);
                let v3 = RawPoint::new(sx1 - nx, sy1 - ny, color);

                vertices.push(v0);
                vertices.push(v1);
                vertices.push(v2);

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

        let push_line_quad = |vertices: &mut Vec<RawPoint>,
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

// ================================================================================
// Coordinate conversion helpers
// ================================================================================

/// Convert screen coordinates (relative to widget bounds) to data coordinates.
fn screen_to_data(
    screen: Point,
    bounds: Rectangle,
    view_x: [f32; 2],
    view_y: [f32; 2],
    padding: f32,
) -> (f32, f32) {
    let plot_width = bounds.width - 2.0 * padding;
    let plot_height = bounds.height - 2.0 * padding;
    let x_norm = (screen.x - bounds.x - padding) / plot_width;
    let y_norm = 1.0 - (screen.y - bounds.y - padding) / plot_height;
    let x = view_x[0] + x_norm * (view_x[1] - view_x[0]);
    let y = view_y[0] + y_norm * (view_y[1] - view_y[0]);
    (x, y)
}

/// Clamp a view range to bounds, keeping the range size the same (shift rather than squash).
/// If the view range exceeds bounds+padding, clamp it to the bounds size.
fn clamp_range_to_bounds(
    range: (f32, f32),
    bounds: Option<(f32, f32)>,
    padding_frac: f32,
) -> (f32, f32) {
    let (mut lo, mut hi) = range;
    if let Some((b_lo, b_hi)) = bounds {
        let pad = (b_hi - b_lo) * padding_frac;
        let min_bound = b_lo - pad;
        let max_bound = b_hi + pad;
        let bounds_size = max_bound - min_bound;
        let range_size = hi - lo;

        // If the view is wider than bounds+padding, clamp to bounds size and center
        if range_size > bounds_size {
            let center = (min_bound + max_bound) / 2.0;
            lo = center - bounds_size / 2.0;
            hi = center + bounds_size / 2.0;
        } else {
            // Shift to stay within bounds
            if lo < min_bound {
                lo = min_bound;
                hi = lo + range_size;
            }
            if hi > max_bound {
                hi = max_bound;
                lo = hi - range_size;
            }
        }
    }
    (lo, hi)
}

/// Apply elastic resistance when dragging past bounds.
/// Returns the elastically-damped range (allows slight over-scroll).
fn apply_elastic_resistance(
    range: (f32, f32),
    bounds: Option<(f32, f32)>,
    padding_frac: f32,
    elastic_limit: f32,
) -> (f32, f32) {
    let (lo, hi) = range;
    if let Some((b_lo, b_hi)) = bounds {
        let pad = (b_hi - b_lo) * padding_frac;
        let min_bound = b_lo - pad;
        let max_bound = b_hi + pad;
        let range_size = hi - lo;
        let max_overscroll = range_size * elastic_limit;

        let mut new_lo = lo;
        let mut new_hi = hi;

        // Apply damping when past bounds (exponential decay)
        if lo < min_bound {
            let over = min_bound - lo;
            let damped = max_overscroll * (1.0 - (-over / max_overscroll).exp());
            new_lo = min_bound - damped;
            new_hi = new_lo + range_size;
        } else if hi > max_bound {
            let over = hi - max_bound;
            let damped = max_overscroll * (1.0 - (-over / max_overscroll).exp());
            new_hi = max_bound + damped;
            new_lo = new_hi - range_size;
        }

        (new_lo, new_hi)
    } else {
        (lo, hi)
    }
}

/// Check if a range is outside its bounds (needs spring-back).
fn is_out_of_bounds(range: (f32, f32), bounds: Option<(f32, f32)>, padding_frac: f32) -> bool {
    if let Some((b_lo, b_hi)) = bounds {
        let pad = (b_hi - b_lo) * padding_frac;
        let min_bound = b_lo - pad;
        let max_bound = b_hi + pad;
        range.0 < min_bound - 0.001 || range.1 > max_bound + 0.001
    } else {
        false
    }
}

/// Ease-out cubic: decelerating to zero velocity.
fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Interpolate between two ranges using an easing function.
fn lerp_range(from: (f32, f32), to: (f32, f32), t: f32) -> (f32, f32) {
    let t = ease_out_cubic(t);
    (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t)
}

// ================================================================================
// shader::Primitive implementation
// ================================================================================

impl shader::Primitive for PlotterPrimitive {
    type Pipeline = Pipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        // Combine grid + selection vertices for the grid render pass
        if self.selection_vertices.is_empty() {
            pipeline.update(
                device,
                queue,
                &self.uniforms,
                &self.points,
                &self.line_vertices,
                &self.grid_vertices,
            );
        } else {
            let mut combined = self.grid_vertices.clone();
            combined.extend_from_slice(&self.selection_vertices);
            pipeline.update(
                device,
                queue,
                &self.uniforms,
                &self.points,
                &self.line_vertices,
                &combined,
            );
        }

        // Compute scissor rects in absolute physical pixel coordinates.
        // iced sets the viewport to the widget's bounds before calling draw,
        // but set_scissor_rect always operates in absolute framebuffer coords.
        let scale = viewport.scale_factor() as f32;
        let pad_x = self.uniforms.padding[0];
        let pad_y = self.uniforms.padding[1];

        // Widget bounds in physical pixels
        let wx = (bounds.x * scale) as u32;
        let wy = (bounds.y * scale) as u32;
        let ww = (bounds.width * scale) as u32;
        let wh = (bounds.height * scale) as u32;
        pipeline.widget_scissor = [wx, wy, ww.max(1), wh.max(1)];

        // Plot area (inside padding) in physical pixels
        let px = (bounds.x + pad_x) * scale;
        let py = (bounds.y + pad_y) * scale;
        let pw = (bounds.width - 2.0 * pad_x) * scale;
        let ph = (bounds.height - 2.0 * pad_y) * scale;
        pipeline.plot_scissor = [
            px as u32,
            py as u32,
            (pw as u32).max(1),
            (ph as u32).max(1),
        ];
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        let total_grid = self.grid_vertices.len() + self.selection_vertices.len();
        if total_grid > 0 {
            pipeline.render_grid(render_pass, total_grid as u32);
        }

        // Set scissor rect to clip markers and lines to the plot area (inside padding).
        // These are absolute physical-pixel coordinates computed during prepare().
        let [sx, sy, sw, sh] = pipeline.plot_scissor;
        render_pass.set_scissor_rect(sx, sy, sw, sh);

        if self.config.show_lines {
            pipeline.render_lines(render_pass, self.line_vertices.len() as u32);
        }

        if self.config.show_markers {
            pipeline.render_markers(render_pass, self.points.len() as u32);
        }

        // Restore scissor rect to full widget bounds so iced's subsequent rendering is correct.
        let [wx, wy, ww, wh] = pipeline.widget_scissor;
        render_pass.set_scissor_rect(wx, wy, ww, wh);

        true
    }
}

impl shader::Pipeline for Pipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        Pipeline::new(device, queue, format)
    }
}

// ================================================================================
// shader::Program implementation (event handling + drawing)
// ================================================================================

impl<Message: Clone> shader::Program<Message> for Plotter<'_, Message> {
    type State = PlotterState;
    type Primitive = PlotterPrimitive;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<shader::Action<Message>> {
        let interaction = &self.interaction;

        // Check if any interaction is enabled at all
        let has_any_interaction = interaction.pan_x
            || interaction.pan_y
            || interaction.zoom_x
            || interaction.zoom_y
            || interaction.double_click_to_fit
            || interaction.zoom_select;

        if !has_any_interaction {
            return None;
        }

        let (view_x, view_y, _data_x, _data_y) = self.resolve_view_ranges();
        let padding = self.options.padding;

        // ---------- Elastic spring-back animation ----------
        // Tick the animation on every event while it's active.
        // Each tick publishes the interpolated view and requests the next redraw.
        if let Some(ref anim) = state.elastic_animation.clone() {
            let elapsed = anim.start_time.elapsed().as_millis() as u64;

            if elapsed >= anim.duration_ms {
                // Animation complete: snap to target
                let mut new_view = self.view_state.clone();
                if let (Some(_from), Some(to)) = (anim.from_x, anim.to_x) {
                    new_view.x_range = Some(to);
                }
                if let (Some(_from), Some(to)) = (anim.from_y, anim.to_y) {
                    new_view.y_range = Some(to);
                }
                state.elastic_animation = None;

                if let Some(ref on_change) = self.on_view_change {
                    return Some(shader::Action::publish((on_change)(new_view)));
                }
                return None;
            }

            // Still animating: interpolate and request next frame
            let t = elapsed as f32 / anim.duration_ms as f32;
            let mut new_view = self.view_state.clone();
            if let (Some(from), Some(to)) = (anim.from_x, anim.to_x) {
                new_view.x_range = Some(lerp_range(from, to, t));
            }
            if let (Some(from), Some(to)) = (anim.from_y, anim.to_y) {
                new_view.y_range = Some(lerp_range(from, to, t));
            }

            if let Some(ref on_change) = self.on_view_change {
                // Publish triggers a redraw, which triggers another update cycle
                return Some(shader::Action::publish((on_change)(new_view)));
            }
            return Some(shader::Action::request_redraw());
        }

        match event {
            // ---- Track keyboard modifiers ----
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.modifiers = *modifiers;
                None
            }

            // ---- Mouse button press ----
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    // Double-click detection
                    if interaction.double_click_to_fit {
                        let now = std::time::Instant::now();
                        if let Some(last) = state.last_click_time
                            && now.duration_since(last).as_millis() < 300 {
                                // Double-click: reset to auto-fit
                                state.last_click_time = None;
                                state.interaction_mode = InteractionMode::Idle;
                                state.elastic_animation = None;

                                if let Some(ref on_change) = self.on_view_change {
                                    let new_view = ViewState {
                                        x_range: if interaction.pan_x || interaction.zoom_x {
                                            None
                                        } else {
                                            self.view_state.x_range
                                        },
                                        y_range: if interaction.pan_y || interaction.zoom_y {
                                            None
                                        } else {
                                            self.view_state.y_range
                                        },
                                    };
                                    return Some(
                                        shader::Action::publish((on_change)(new_view))
                                            .and_capture(),
                                    );
                                }
                                return Some(shader::Action::capture());
                            }
                        state.last_click_time = Some(now);
                    }

                    // Ctrl+click = zoom select
                    if interaction.zoom_select && state.modifiers.control() {
                        state.interaction_mode = InteractionMode::ZoomSelecting;
                        state.drag_start = Some(pos);
                        state.zoom_select_current = Some(pos);
                        return Some(shader::Action::capture());
                    }

                    // Start panning
                    if interaction.pan_x || interaction.pan_y {
                        state.elastic_animation = None; // Cancel any ongoing animation
                        state.interaction_mode = InteractionMode::Panning;
                        state.drag_start = Some(pos);
                        state.drag_start_view = Some(ViewState {
                            x_range: Some((view_x[0], view_x[1])),
                            y_range: Some((view_y[0], view_y[1])),
                        });
                        return Some(shader::Action::capture());
                    }
                }
                None
            }

            // ---- Mouse button release ----
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                match state.interaction_mode {
                    InteractionMode::Panning => {
                        state.interaction_mode = InteractionMode::Idle;
                        state.drag_start = None;
                        state.drag_start_view = None;

                        // Check if we need to spring back from over-scroll
                        if interaction.elastic {
                            let current_x =
                                self.view_state.x_range.unwrap_or((view_x[0], view_x[1]));
                            let current_y =
                                self.view_state.y_range.unwrap_or((view_y[0], view_y[1]));

                            let x_out = interaction.pan_x
                                && is_out_of_bounds(
                                    current_x,
                                    interaction.x_bounds,
                                    interaction.boundary_padding,
                                );
                            let y_out = interaction.pan_y
                                && is_out_of_bounds(
                                    current_y,
                                    interaction.y_bounds,
                                    interaction.boundary_padding,
                                );

                            if x_out || y_out {
                                let target_x = if x_out {
                                    Some(clamp_range_to_bounds(
                                        current_x,
                                        interaction.x_bounds,
                                        interaction.boundary_padding,
                                    ))
                                } else {
                                    None
                                };
                                let target_y = if y_out {
                                    Some(clamp_range_to_bounds(
                                        current_y,
                                        interaction.y_bounds,
                                        interaction.boundary_padding,
                                    ))
                                } else {
                                    None
                                };

                                state.elastic_animation = Some(ElasticState {
                                    from_x: if x_out { Some(current_x) } else { None },
                                    from_y: if y_out { Some(current_y) } else { None },
                                    to_x: target_x,
                                    to_y: target_y,
                                    start_time: std::time::Instant::now(),
                                    duration_ms: interaction.elastic_duration_ms,
                                });

                                return Some(shader::Action::request_redraw().and_capture());
                            }
                        }

                        Some(shader::Action::capture())
                    }
                    InteractionMode::ZoomSelecting => {
                        // Complete the zoom selection
                        if let (Some(start), Some(current)) =
                            (state.drag_start, state.zoom_select_current)
                        {
                            // Convert screen coords to data coords
                            let (x0, y0) = screen_to_data(
                                Point::new(start.x + bounds.x, start.y + bounds.y),
                                bounds,
                                view_x,
                                view_y,
                                padding,
                            );
                            let (x1, y1) = screen_to_data(
                                Point::new(current.x + bounds.x, current.y + bounds.y),
                                bounds,
                                view_x,
                                view_y,
                                padding,
                            );

                            // Only zoom if the rectangle is big enough (>5px in both directions)
                            let dx = (current.x - start.x).abs();
                            let dy = (current.y - start.y).abs();

                            if dx > 5.0 || dy > 5.0 {
                                let mut new_view = self.view_state.clone();

                                if interaction.zoom_x && dx > 5.0 {
                                    let lo = x0.min(x1);
                                    let hi = x0.max(x1);
                                    new_view.x_range = Some((lo, hi));
                                }

                                if interaction.zoom_y && dy > 5.0 {
                                    let lo = y0.min(y1);
                                    let hi = y0.max(y1);
                                    new_view.y_range = Some((lo, hi));
                                }

                                state.interaction_mode = InteractionMode::Idle;
                                state.drag_start = None;
                                state.zoom_select_current = None;

                                if let Some(ref on_change) = self.on_view_change {
                                    return Some(
                                        shader::Action::publish((on_change)(new_view))
                                            .and_capture(),
                                    );
                                }
                            }
                        }

                        state.interaction_mode = InteractionMode::Idle;
                        state.drag_start = None;
                        state.zoom_select_current = None;
                        Some(shader::Action::capture())
                    }
                    InteractionMode::Idle => None,
                }
            }

            // ---- Mouse move (drag) ----
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                state.last_cursor = Some(*position);

                match state.interaction_mode {
                    InteractionMode::Panning => {
                        if let (Some(start), Some(start_view)) =
                            (state.drag_start, &state.drag_start_view)
                        {
                            let start_view_x = start_view.x_range.unwrap();
                            let start_view_y = start_view.y_range.unwrap();

                            let plot_width = bounds.width - 2.0 * padding;
                            let plot_height = bounds.height - 2.0 * padding;

                            // position is absolute screen coords; drag_start is relative to bounds
                            let current = Point::new(position.x - bounds.x, position.y - bounds.y);
                            let dx_screen = current.x - start.x;
                            let dy_screen = current.y - start.y;

                            // Convert screen delta to data delta
                            let dx_data =
                                -dx_screen / plot_width * (start_view_x.1 - start_view_x.0);
                            let dy_data =
                                dy_screen / plot_height * (start_view_y.1 - start_view_y.0);

                            let mut new_view = self.view_state.clone();

                            if interaction.pan_x {
                                let raw = (start_view_x.0 + dx_data, start_view_x.1 + dx_data);
                                let new_x = if interaction.elastic {
                                    apply_elastic_resistance(
                                        raw,
                                        interaction.x_bounds,
                                        interaction.boundary_padding,
                                        interaction.elastic_limit,
                                    )
                                } else {
                                    clamp_range_to_bounds(
                                        raw,
                                        interaction.x_bounds,
                                        interaction.boundary_padding,
                                    )
                                };
                                new_view.x_range = Some(new_x);
                            }

                            if interaction.pan_y {
                                let raw = (start_view_y.0 + dy_data, start_view_y.1 + dy_data);
                                let new_y = if interaction.elastic {
                                    apply_elastic_resistance(
                                        raw,
                                        interaction.y_bounds,
                                        interaction.boundary_padding,
                                        interaction.elastic_limit,
                                    )
                                } else {
                                    clamp_range_to_bounds(
                                        raw,
                                        interaction.y_bounds,
                                        interaction.boundary_padding,
                                    )
                                };
                                new_view.y_range = Some(new_y);
                            }

                            if let Some(ref on_change) = self.on_view_change {
                                return Some(
                                    shader::Action::publish((on_change)(new_view)).and_capture(),
                                );
                            }
                            return Some(shader::Action::capture());
                        }
                        None
                    }
                    InteractionMode::ZoomSelecting => {
                        // Update the current selection corner
                        let relative = Point::new(position.x - bounds.x, position.y - bounds.y);
                        state.zoom_select_current = Some(relative);
                        // Request redraw to update the selection rectangle
                        Some(shader::Action::request_redraw().and_capture())
                    }
                    InteractionMode::Idle => None,
                }
            }

            // ---- Scroll wheel (zoom) ----
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if !interaction.zoom_x && !interaction.zoom_y {
                    return None;
                }

                // Only zoom if cursor is within bounds
                let cursor_pos = cursor.position_in(bounds)?;

                let scroll_y = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 50.0,
                };

                if scroll_y.abs() < f32::EPSILON {
                    return None;
                }

                // Cancel any elastic animation
                state.elastic_animation = None;

                // Zoom factor: positive scroll = zoom in (shrink range)
                let factor = 1.0 - scroll_y * interaction.zoom_speed;
                let factor = factor.clamp(0.1, 10.0); // safety clamp

                // Get cursor position in data space (zoom center)
                let (cx, cy) = screen_to_data(cursor_pos, bounds, view_x, view_y, padding);

                let mut new_view = self.view_state.clone();

                if interaction.zoom_x {
                    let new_lo = cx - (cx - view_x[0]) * factor;
                    let new_hi = cx + (view_x[1] - cx) * factor;
                    let clamped = clamp_range_to_bounds(
                        (new_lo, new_hi),
                        interaction.x_bounds,
                        interaction.boundary_padding,
                    );
                    new_view.x_range = Some(clamped);
                }

                if interaction.zoom_y {
                    let new_lo = cy - (cy - view_y[0]) * factor;
                    let new_hi = cy + (view_y[1] - cy) * factor;
                    let clamped = clamp_range_to_bounds(
                        (new_lo, new_hi),
                        interaction.y_bounds,
                        interaction.boundary_padding,
                    );
                    new_view.y_range = Some(clamped);
                }

                // For axes with auto-fit that are not being zoomed,
                // keep them as None (auto-fit)
                if !interaction.zoom_x && self.view_state.x_range.is_none() {
                    new_view.x_range = None;
                }
                if !interaction.zoom_y && self.view_state.y_range.is_none() {
                    new_view.y_range = None;
                }

                if let Some(ref on_change) = self.on_view_change {
                    return Some(shader::Action::publish((on_change)(new_view)).and_capture());
                }
                Some(shader::Action::capture())
            }

            _ => None,
        }
    }

    fn draw(&self, state: &Self::State, _cursor: Cursor, bounds: Rectangle) -> Self::Primitive {
        let (view_x, view_y, _, _) = self.resolve_view_ranges();

        // Build selection rectangle from state if zoom-selecting
        let selection_rect = if state.interaction_mode == InteractionMode::ZoomSelecting {
            if let (Some(start), Some(current)) = (state.drag_start, state.zoom_select_current) {
                Some((start, current))
            } else {
                None
            }
        } else {
            None
        };

        PlotterPrimitive::new(
            &self.series,
            bounds,
            &self.options,
            view_x,
            view_y,
            selection_rect,
        )
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        let has_any = self.interaction.pan_x
            || self.interaction.pan_y
            || self.interaction.zoom_x
            || self.interaction.zoom_y
            || self.interaction.zoom_select;

        if !has_any {
            return mouse::Interaction::default();
        }

        match state.interaction_mode {
            InteractionMode::Panning => mouse::Interaction::Grabbing,
            InteractionMode::ZoomSelecting => mouse::Interaction::Crosshair,
            InteractionMode::Idle => {
                if cursor.is_over(bounds) {
                    // Show crosshair when Ctrl is held (indicating zoom select is available)
                    if self.interaction.zoom_select && state.modifiers.control() {
                        mouse::Interaction::Crosshair
                    } else {
                        mouse::Interaction::Grab
                    }
                } else {
                    mouse::Interaction::default()
                }
            }
        }
    }
}
