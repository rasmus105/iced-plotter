use std::borrow::Cow;

use iced::widget::shader;
use iced::{Element, Length};

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
pub struct PlotterOptions {
    pub show_legend: bool,
    /// Padding around the plot area in pixels
    pub padding: f32,
}

impl Default for PlotterOptions {
    fn default() -> Self {
        Self {
            show_legend: false,
            padding: 50.0,
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

    /// Main function for drawing plotter in view using GPU shaders.
    pub fn draw<'view, Message>(&'view self) -> Element<'view, Message>
    where
        Message: 'view,
    {
        shader(self).width(Length::Fill).height(Length::Fill).into()
    }
}
