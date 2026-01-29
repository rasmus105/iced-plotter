use iced::{mouse, widget::canvas, Color, Element, Length, Point, Rectangle, Renderer, Theme};

pub struct PlotPoint {
    x: f64,
    y: f64,
}

/// Describes a function y = f(x) with an optional range for x and a number of
/// points.
pub struct ExplicitGenerator<'a> {
    function: Box<dyn Fn(f64) -> f64 + 'a>,
    x_range: (f64, f64), // start, end
    points: usize,
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

#[derive(Default)]
pub struct Chart {
    height: f64,
    width: f64,

    options: ChartOptions,
}

#[derive(Default)]
pub struct CanvasState {
    is_dragging: bool,
}

impl<Message> canvas::Program<Message> for Chart {
    type State = CanvasState;

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Draw the axes
        let padding = 50.0;
        self.draw_axes(&mut frame, bounds.width, bounds.height, padding);

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
impl Chart {
    /// Draws the coordinate axes (X and Y) on the frame
    fn draw_axes(
        &self,
        frame: &mut canvas::Frame,
        bounds_width: f32,
        bounds_height: f32,
        padding: f32,
    ) {
        // Define the plot boundaries with padding
        let plot_left = padding;
        let plot_right = bounds_width - padding;
        let plot_top = padding;
        let plot_bottom = bounds_height - padding;

        // Draw X-axis (horizontal line at bottom)
        let x_axis = canvas::Path::line(
            Point {
                x: plot_left,
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
                .with_color(Color::BLACK)
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
                y: plot_bottom,
            },
        );
        frame.stroke(
            &y_axis,
            canvas::Stroke::default()
                .with_color(Color::BLACK)
                .with_width(2.0),
        );
    }
}

///
/// Public methods
///
impl Chart {
    pub fn new(width: f64, height: f64) -> Self {
        Chart {
            width,
            height,
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
