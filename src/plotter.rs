use std::borrow::Cow;

use iced::widget::canvas;
use iced::widget::shader;
use iced::widget::stack;
use iced::{Element, Font, Length, Point, Renderer, Theme};

// ================================================================================
// Style Types
// ================================================================================

/// Shape of markers to render for a series
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum MarkerShape {
    Circle = 0,
    Square = 1,
    Diamond = 2,
    TriangleUp = 3,
    TriangleDown = 4,
    Cross = 5,
    Plus = 6,
    None = 7,
}

impl MarkerShape {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Pattern for rendering lines
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum LinePattern {
    Solid = 0,
    Dashed = 1,
    Dotted = 2,
    DashDot = 3,
    None = 4,
}

impl LinePattern {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// Styling options for a plot series
#[derive(Clone, Debug)]
pub struct SeriesStyle<'a> {
    /// How to color the points
    pub color: ColorMode<'a>,
    /// Shape of markers
    pub marker_shape: MarkerShape,
    /// Marker size in pixels
    pub marker_size: f32,
    /// Line pattern
    pub line_pattern: LinePattern,
    /// Line width in pixels
    pub line_width: f32,
}

impl<'a> SeriesStyle<'a> {
    /// Create a new series style with defaults
    pub fn new(color: ColorMode<'a>) -> Self {
        Self {
            color,
            marker_shape: MarkerShape::Circle,
            marker_size: 4.0,
            line_pattern: LinePattern::Solid,
            line_width: 2.0,
        }
    }

    /// Set marker shape
    pub fn with_marker_shape(mut self, shape: MarkerShape) -> Self {
        self.marker_shape = shape;
        self
    }

    /// Set marker size
    pub fn with_marker_size(mut self, size: f32) -> Self {
        self.marker_size = size;
        self
    }

    /// Set line pattern
    pub fn with_line_pattern(mut self, pattern: LinePattern) -> Self {
        self.line_pattern = pattern;
        self
    }

    /// Set line width
    pub fn with_line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }
}

impl Default for SeriesStyle<'_> {
    fn default() -> Self {
        Self {
            color: ColorMode::solid(iced::Color::WHITE),
            marker_shape: MarkerShape::Circle,
            marker_size: 4.0,
            line_pattern: LinePattern::Solid,
            line_width: 2.0,
        }
    }
}

// ================================================================================
// Color Mode
// ================================================================================

/// How points in a series should be colored
#[derive(Clone, Debug)]
pub enum ColorMode<'a> {
    /// Single solid color for all points
    Solid(iced::Color),

    /// Gradient based on a value (Y coordinate or separate values array)
    ValueGradient {
        /// Color at minimum value
        low: iced::Color,
        /// Color at maximum value
        high: iced::Color,
        /// Optional: use separate value array instead of Y coordinate
        /// If None, Y coordinate is used
        values: Option<Cow<'a, [f32]>>,
    },

    /// Gradient based on point index (0 = start, 1 = end)
    IndexGradient {
        /// Color at first point
        start: iced::Color,
        /// Color at last point
        end: iced::Color,
    },

    /// Use a named colormap
    Colormap {
        /// Name of the colormap to use
        name: crate::colormap::ColormapName,
        /// Optional: use separate value array instead of Y coordinate
        /// If None, Y coordinate is used
        values: Option<Cow<'a, [f32]>>,
    },
}

impl<'a> ColorMode<'a> {
    /// Convert a solid Color to ColorMode for convenience
    pub fn solid(color: iced::Color) -> Self {
        ColorMode::Solid(color)
    }

    pub fn value_gradient(low: iced::Color, high: iced::Color) -> Self {
        ColorMode::ValueGradient {
            low,
            high,
            values: None,
        }
    }

    pub fn value_gradient_values<V>(low: iced::Color, high: iced::Color, values: V) -> Self
    where
        V: Into<Cow<'a, [f32]>>,
    {
        ColorMode::ValueGradient {
            low,
            high,
            values: Some(values.into()),
        }
    }

    pub fn index_gradient(start: iced::Color, end: iced::Color) -> Self {
        ColorMode::IndexGradient { start, end }
    }

    pub fn colormap(name: crate::colormap::ColormapName) -> Self {
        ColorMode::Colormap { name, values: None }
    }

    pub fn colormap_values<V>(name: crate::colormap::ColormapName, values: V) -> Self
    where
        V: Into<Cow<'a, [f32]>>,
    {
        ColorMode::Colormap {
            name,
            values: Some(values.into()),
        }
    }
}

// ================================================================================
// Utility Types
// ================================================================================

#[derive(Clone)]
pub struct PlotPoint {
    pub x: f32,
    pub y: f32,
}

impl From<(f32, f32)> for PlotPoint {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

/// Describes a function y = f(x) with an optional range for x and a number of
/// points.
pub struct ExplicitGenerator<'a> {
    pub function: Box<dyn Fn(f32) -> f32 + 'a>,
    pub x_range: (f32, f32), // start, end
    pub points: usize,
}

pub enum PlotPoints<'a> {
    Owned(Vec<PlotPoint>),
    Borrowed(&'a [PlotPoint]),
    Generator(ExplicitGenerator<'a>),
}

impl<'a> PlotPoints<'a> {
    pub fn owned(points: Vec<PlotPoint>) -> Self {
        PlotPoints::Owned(points)
    }

    pub fn borrowed(points: &'a [PlotPoint]) -> Self {
        PlotPoints::Borrowed(points)
    }

    pub fn generator<F>(function: F, x_range: (f32, f32), points: usize) -> Self
    where
        F: Fn(f32) -> f32 + 'a,
    {
        PlotPoints::Generator(ExplicitGenerator {
            function: Box::new(function),
            x_range,
            points,
        })
    }
}

impl From<Vec<PlotPoint>> for PlotPoints<'_> {
    fn from(points: Vec<PlotPoint>) -> Self {
        PlotPoints::Owned(points)
    }
}

impl<'a> From<&'a [PlotPoint]> for PlotPoints<'a> {
    fn from(points: &'a [PlotPoint]) -> Self {
        PlotPoints::Borrowed(points)
    }
}

impl Default for PlotPoints<'_> {
    fn default() -> Self {
        PlotPoints::Owned(Vec::new())
    }
}

pub struct PlotSeries<'a> {
    pub label: String,
    pub style: SeriesStyle<'a>,
    pub points: PlotPoints<'a>,
}

impl<'a> PlotSeries<'a> {
    pub fn new(label: impl Into<String>, points: PlotPoints<'a>) -> Self {
        Self {
            label: label.into(),
            style: SeriesStyle::default(),
            points,
        }
    }

    pub fn with_style(mut self, style: SeriesStyle<'a>) -> Self {
        self.style = style;
        self
    }
}

// ================================================================================
// Plotter
// ================================================================================

#[derive(Clone, Debug)]
pub struct GridStyle {
    pub show: bool,
    pub color: iced::Color,
    pub line_width: f32,
}

impl Default for GridStyle {
    fn default() -> Self {
        Self {
            show: true,
            color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.1),
            line_width: 1.0,
        }
    }
}

pub struct AxisConfig {
    pub show: bool,
    pub color: iced::Color,
    pub line_width: f32,
    pub label_color: iced::Color,
    pub label_size: f32,
    pub ticks: crate::ticks::TickConfig,
    pub format: Box<dyn Fn(f32) -> String>,
}

impl Clone for AxisConfig {
    fn clone(&self) -> Self {
        Self {
            show: self.show,
            color: self.color,
            line_width: self.line_width,
            label_color: self.label_color,
            label_size: self.label_size,
            ticks: self.ticks.clone(),
            format: Box::new(|v| format!("{v:.2}")),
        }
    }
}

impl std::fmt::Debug for AxisConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AxisConfig")
            .field("show", &self.show)
            .field("color", &self.color)
            .field("line_width", &self.line_width)
            .field("label_color", &self.label_color)
            .field("label_size", &self.label_size)
            .field("ticks", &self.ticks)
            .finish()
    }
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            show: true,
            color: iced::Color::WHITE,
            line_width: 1.5,
            label_color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.7),
            label_size: 12.0,
            ticks: crate::ticks::TickConfig::default(),
            format: Box::new(|v| format!("{v:.2}")),
        }
    }
}

impl AxisConfig {
    pub fn with_format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format = Box::new(f);
        self
    }
}

#[derive(Clone, Debug)]
pub struct PlotterOptions {
    pub show_legend: bool,
    pub padding: f32,
    pub grid: GridStyle,
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
}

impl Default for PlotterOptions {
    fn default() -> Self {
        Self {
            show_legend: false,
            padding: 50.0,
            grid: GridStyle::default(),
            x_axis: AxisConfig::default(),
            y_axis: AxisConfig::default(),
        }
    }
}

#[derive(Default)]
pub struct Plotter<'a> {
    // data related
    pub series: Vec<PlotSeries<'a>>,

    // configuration related
    pub options: PlotterOptions,
}

// ================================================================================
// Public Methods
// ================================================================================

impl<'a> Plotter<'a> {
    pub fn new(series: Vec<PlotSeries<'a>>) -> Self {
        Self {
            series,
            options: PlotterOptions::default(),
        }
    }

    pub fn with_options(mut self, options: PlotterOptions) -> Self {
        self.options = options;
        self
    }

    fn compute_data_ranges(&self) -> ([f32; 2], [f32; 2]) {
        let mut x_min = f32::INFINITY;
        let mut x_max = f32::NEG_INFINITY;
        let mut y_min = f32::INFINITY;
        let mut y_max = f32::NEG_INFINITY;

        for s in &self.series {
            let iter: Box<dyn Iterator<Item = (f32, f32)> + '_> = match &s.points {
                PlotPoints::Owned(pts) => Box::new(pts.iter().map(|p| (p.x, p.y))),
                PlotPoints::Borrowed(pts) => Box::new(pts.iter().map(|p| (p.x, p.y))),
                PlotPoints::Generator(generator) => {
                    let (x0, x1) = generator.x_range;
                    let span = x1 - x0;
                    let n = generator.points;
                    Box::new((0..n).map(move |i| {
                        let t = i as f32 / (n - 1).max(1) as f32;
                        let x = x0 + t * span;
                        let y = (generator.function)(x);
                        (x, y)
                    }))
                }
            };
            for (x, y) in iter {
                x_min = x_min.min(x);
                x_max = x_max.max(x);
                y_min = y_min.min(y);
                y_max = y_max.max(y);
            }
        }

        if x_min > x_max {
            x_min = 0.0;
            x_max = 1.0;
            y_min = 0.0;
            y_max = 1.0;
        } else if (y_max - y_min).abs() < f32::EPSILON {
            y_min -= 0.5;
            y_max += 0.5;
        }

        ([x_min, x_max], [y_min, y_max])
    }

    pub fn draw<'view, Message>(&'view self) -> Element<'view, Message>
    where
        Message: 'view,
    {
        let (x_range, y_range) = self.compute_data_ranges();
        let x_ticks = crate::ticks::compute_ticks(x_range[0], x_range[1], &self.options.x_axis.ticks);
        let y_ticks = crate::ticks::compute_ticks(y_range[0], y_range[1], &self.options.y_axis.ticks);

        let x_labels: Vec<String> = x_ticks.iter().map(|v| (self.options.x_axis.format)(*v)).collect();
        let y_labels: Vec<String> = y_ticks.iter().map(|v| (self.options.y_axis.format)(*v)).collect();

        let overlay = AxisOverlay {
            x_ticks,
            y_ticks,
            x_labels,
            y_labels,
            x_range,
            y_range,
            padding: self.options.padding,
            x_label_color: self.options.x_axis.label_color,
            y_label_color: self.options.y_axis.label_color,
            x_label_size: self.options.x_axis.label_size,
            y_label_size: self.options.y_axis.label_size,
            show_x: self.options.x_axis.show,
            show_y: self.options.y_axis.show,
        };

        stack![
            shader(self).width(Length::Fill).height(Length::Fill),
            canvas(overlay).width(Length::Fill).height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

struct AxisOverlay {
    x_ticks: Vec<f32>,
    y_ticks: Vec<f32>,
    x_labels: Vec<String>,
    y_labels: Vec<String>,
    x_range: [f32; 2],
    y_range: [f32; 2],
    padding: f32,
    x_label_color: iced::Color,
    y_label_color: iced::Color,
    x_label_size: f32,
    y_label_size: f32,
    show_x: bool,
    show_y: bool,
}

impl<Message> canvas::Program<Message> for AxisOverlay {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let plot_width = bounds.width - 2.0 * self.padding;
        let plot_height = bounds.height - 2.0 * self.padding;
        let x_span = self.x_range[1] - self.x_range[0];
        let y_span = self.y_range[1] - self.y_range[0];

        if self.show_x && x_span.abs() > f32::EPSILON {
            for (tick, label) in self.x_ticks.iter().zip(&self.x_labels) {
                if *tick < self.x_range[0] || *tick > self.x_range[1] {
                    continue;
                }
                let x_norm = (tick - self.x_range[0]) / x_span;
                let screen_x = self.padding + x_norm * plot_width;
                let screen_y = self.padding + plot_height + 6.0;

                frame.fill_text(canvas::Text {
                    content: label.clone(),
                    size: iced::Pixels(self.x_label_size),
                    position: Point::new(screen_x, screen_y),
                    color: self.x_label_color,
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Top,
                    font: Font::MONOSPACE,
                    ..canvas::Text::default()
                });
            }
        }

        if self.show_y && y_span.abs() > f32::EPSILON {
            for (tick, label) in self.y_ticks.iter().zip(&self.y_labels) {
                if *tick < self.y_range[0] || *tick > self.y_range[1] {
                    continue;
                }
                let y_norm = (tick - self.y_range[0]) / y_span;
                let screen_y = self.padding + (1.0 - y_norm) * plot_height;
                let screen_x = self.padding - 6.0;

                frame.fill_text(canvas::Text {
                    content: label.clone(),
                    size: iced::Pixels(self.y_label_size),
                    position: Point::new(screen_x, screen_y),
                    color: self.y_label_color,
                    align_x: iced::alignment::Horizontal::Right.into(),
                    align_y: iced::alignment::Vertical::Center,
                    font: Font::MONOSPACE,
                    ..canvas::Text::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }
}
