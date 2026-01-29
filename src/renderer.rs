use crate::canvas::PlotterState;
use crate::plotter::{PlotPoints, PlotSeries, Plotter};
use iced::widget::canvas;
use iced::{Color, Point};

// ================================================================================
// Free Functions
// ================================================================================

/// Draws points to the frame given pre-computed (x, y) values and known ranges.
fn draw_points_with_ranges(
    frame: &mut canvas::Frame,
    points: impl Iterator<Item = (f64, f64)>,
    x_range: (f64, f64),
    y_range: (f64, f64),
    plot_bounds: (f32, f32, f32, f32), // left, right, top, bottom
    point_color: Color,
) {
    let (plot_left, plot_right, plot_top, plot_bottom) = plot_bounds;
    let plot_width = plot_right - plot_left;
    let plot_height = plot_bottom - plot_top;

    let (x_min, x_max) = x_range;
    let x_span = x_max - x_min;

    let (y_min, y_max) = y_range;
    let y_span = if (y_max - y_min).abs() < f64::EPSILON {
        1.0 // Avoid division by zero for constant functions
    } else {
        y_max - y_min
    };

    let dot_radius = 3.0;
    for (x, y) in points {
        let screen_x = plot_left + ((x - x_min) / x_span) as f32 * plot_width;
        let screen_y = plot_bottom - ((y - y_min) / y_span) as f32 * plot_height;

        let dot = canvas::Path::circle(Point::new(screen_x, screen_y), dot_radius);
        frame.fill(&dot, point_color);
    }
}

// ================================================================================
// Private Methods
// ================================================================================

impl Plotter<'_> {
    /// Draws points from a PlotSeries.
    fn draw_series_points(
        &self,
        series: &PlotSeries,
        frame: &mut canvas::Frame,
        plot_bounds: (f32, f32, f32, f32),
    ) {
        match &series.points {
            PlotPoints::Owned(points) => {
                if points.is_empty() {
                    return;
                }
                let x_min = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
                let x_max = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
                let y_min = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
                let y_max = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);

                draw_points_with_ranges(
                    frame,
                    points.iter().map(|p| (p.x, p.y)),
                    (x_min, x_max),
                    (y_min, y_max),
                    plot_bounds,
                    series.color,
                );
            }
            PlotPoints::Borrowed(points) => {
                if points.is_empty() {
                    return;
                }
                let x_min = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
                let x_max = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
                let y_min = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
                let y_max = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);

                draw_points_with_ranges(
                    frame,
                    points.iter().map(|p| (p.x, p.y)),
                    (x_min, x_max),
                    (y_min, y_max),
                    plot_bounds,
                    series.color,
                );
            }
            PlotPoints::Generator(generator) => {
                let (x_min, x_max) = generator.x_range;
                let x_span = x_max - x_min;

                // Generate all (x, y) values
                let y_values: Vec<(f64, f64)> = (0..generator.points)
                    .map(|i| {
                        let t = i as f64 / (generator.points - 1).max(1) as f64;
                        let x = x_min + t * x_span;
                        let y = (generator.function)(x);
                        (x, y)
                    })
                    .collect();

                // Calculate y range (auto-scale)
                let y_min = y_values
                    .iter()
                    .map(|(_, y)| *y)
                    .fold(f64::INFINITY, f64::min);
                let y_max = y_values
                    .iter()
                    .map(|(_, y)| *y)
                    .fold(f64::NEG_INFINITY, f64::max);

                draw_points_with_ranges(
                    frame,
                    y_values.into_iter(),
                    (x_min, x_max),
                    (y_min, y_max),
                    plot_bounds,
                    series.color,
                );
            }
        }
    }

    /// Draws all series.
    pub(crate) fn draw_series(
        &self,
        frame: &mut canvas::Frame,
        _state: &PlotterState,
        bounds_width: f32,
        bounds_height: f32,
        padding: f32,
        _default_color: Color,
    ) {
        // Calculate plot area
        let plot_left = padding;
        let plot_right = bounds_width - padding;
        let plot_top = padding;
        let plot_bottom = bounds_height - padding;
        let plot_bounds = (plot_left, plot_right, plot_top, plot_bottom);

        for series in &self.series {
            self.draw_series_points(series, frame, plot_bounds);
        }
    }

    /// Draw legend with latest value for each series, and button for toggling
    /// each line series visibility
    pub(crate) fn draw_legend(&self) {}

    /// Draws the coordinate axes (X and Y) on the frame
    pub(crate) fn draw_axes(
        &self,
        frame: &mut canvas::Frame,
        bounds_width: f32,
        bounds_height: f32,
        padding: f32,
        axis_color: Color,
    ) {
        // Define the plot boundaries with padding
        let plot_left = padding;
        let plot_right = bounds_width - padding;
        let plot_top = padding;
        let plot_bottom = bounds_height - padding;

        // Draw X-axis (horizontal line at bottom)
        let x_axis = canvas::Path::line(
            Point {
                x: plot_left - padding * 0.2,
                y: plot_bottom,
            },
            Point {
                x: plot_right,
                y: plot_bottom,
            },
        );
        frame.stroke(
            &x_axis,
            canvas::Stroke::default()
                .with_color(axis_color)
                .with_width(2.0),
        );

        // Draw Y-axis (vertical line at left)
        let y_axis = canvas::Path::line(
            Point {
                x: plot_left,
                y: plot_top,
            },
            Point {
                x: plot_left,
                y: plot_bottom + padding * 0.2,
            },
        );
        frame.stroke(
            &y_axis,
            canvas::Stroke::default()
                .with_color(axis_color)
                .with_width(2.0),
        );
    }
}
