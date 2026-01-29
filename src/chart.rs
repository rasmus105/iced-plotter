use iced::{mouse, widget::canvas, Color, Element, Length, Point, Rectangle, Renderer, Theme};

pub struct PlotPoint {
    x: f64,
    y: f64,
}

/// Describes a function y = f(x) with an optional range for x and a number of
/// points.
pub struct ExplicitGenerator<'a> {
    pub function: Box<dyn Fn(f64) -> f64 + 'a>,
    pub x_range: (f64, f64), // start, end
    pub points: usize,
}
pub enum PlotPoints<'a> {
    Owned(Vec<PlotPoint>),
    Borrowed(&'a [PlotPoint]),
    Generator(ExplicitGenerator<'a>),
}

pub struct ChartOptions {
    show_legend: bool,
}

impl Default for ChartOptions {
    fn default() -> Self {
        ChartOptions { show_legend: false }
    }
}

impl Default for PlotPoints<'_> {
    fn default() -> Self {
        PlotPoints::Owned(Vec::new())
    }
}

#[derive(Default)]
pub struct Chart<'a> {
    pub points: PlotPoints<'a>,
    pub options: ChartOptions,
}

#[derive(Default)]
pub struct CanvasState {
    is_dragging: bool,
    x_range: (f64, f64),
    y_range: (f64, f64),
}

impl<Message> canvas::Program<Message> for Chart<'_> {
    type State = CanvasState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let padding = 50.0;

        self.draw_points(
            &mut frame,
            state,
            bounds.width,
            bounds.height,
            padding,
            theme.palette().primary,
        );

        self.draw_legend();

        self.draw_axes(
            &mut frame,
            bounds.width,
            bounds.height,
            padding,
            theme.palette().text,
        );

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &iced::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        None
    }
}

///
/// Private methods
///
impl Chart<'_> {
    fn draw_points(
        &self,
        frame: &mut canvas::Frame,
        state: &CanvasState,
        bounds_width: f32,
        bounds_height: f32,
        padding: f32,
        point_color: Color,
    ) {
        use PlotPoints::*;

        // Calculate plot area
        let plot_left = padding;
        let plot_right = bounds_width - padding;
        let plot_top = padding;
        let plot_bottom = bounds_height - padding;
        let plot_width = plot_right - plot_left;
        let plot_height = plot_bottom - plot_top;

        match &self.points {
            Owned(_points) => todo!(),
            Borrowed(_points) => todo!(),
            Generator(generator) => {
                let (x_min, x_max) = generator.x_range;
                let x_span = x_max - x_min;

                // First pass: generate all y values to find y range
                let mut y_values: Vec<(f64, f64)> = Vec::with_capacity(generator.points);
                for i in 0..generator.points {
                    let t = i as f64 / (generator.points - 1).max(1) as f64;
                    let x = x_min + t * x_span;
                    let y = (generator.function)(x);
                    y_values.push((x, y));
                }

                // Calculate y range (auto-scale)
                let y_min = y_values
                    .iter()
                    .map(|(_, y)| *y)
                    .fold(f64::INFINITY, f64::min);
                let y_max = y_values
                    .iter()
                    .map(|(_, y)| *y)
                    .fold(f64::NEG_INFINITY, f64::max);
                let y_span = if (y_max - y_min).abs() < f64::EPSILON {
                    1.0 // Avoid division by zero for constant functions
                } else {
                    y_max - y_min
                };

                // Draw each point as a small filled circle
                let dot_radius = 3.0;
                for (x, y) in y_values {
                    // Transform to screen coordinates
                    let screen_x = plot_left + ((x - x_min) / x_span) as f32 * plot_width;
                    let screen_y = plot_bottom - ((y - y_min) / y_span) as f32 * plot_height;

                    let dot = canvas::Path::circle(Point::new(screen_x, screen_y), dot_radius);
                    frame.fill(&dot, point_color);
                }
            }
        }
    }

    /// Draw legend with latest value for each series, and button for toggling
    /// each line series visibility
    fn draw_legend(&self) {}

    /// Draws the coordinate axes (X and Y) on the frame
    fn draw_axes(
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

///
/// Public methods
///
impl Chart<'_> {
    pub fn new() -> Self {
        Chart {
            points: PlotPoints::default(),
            options: ChartOptions::default(),
        }
    }

    pub fn draw<'a, Message>(&'a self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        canvas(self).width(Length::Fill).height(Length::Fill).into()
    }
}
