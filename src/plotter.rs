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
pub struct SeriesStyle {
    /// How to color the points
    pub color: ColorMode,
    /// Shape of markers
    pub marker_shape: MarkerShape,
    /// Marker size in pixels
    pub marker_size: f32,
    /// Line pattern
    pub line_pattern: LinePattern,
    /// Line width in pixels
    pub line_width: f32,
}

impl SeriesStyle {
    /// Create a new series style with defaults
    pub fn new(color: ColorMode) -> Self {
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

impl Default for SeriesStyle {
    fn default() -> Self {
        Self {
            color: ColorMode::Solid(iced::Color::WHITE),
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
pub enum ColorMode {
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
        values: Option<Vec<f32>>,
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
        values: Option<Vec<f32>>,
    },
}

impl ColorMode {
    /// Convert a solid Color to ColorMode for convenience
    pub fn solid(color: iced::Color) -> Self {
        ColorMode::Solid(color)
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

impl Default for PlotPoints<'_> {
    fn default() -> Self {
        PlotPoints::Owned(Vec::new())
    }
}

pub struct PlotSeries<'a> {
    pub label: String,
    pub style: SeriesStyle,
    pub points: PlotPoints<'a>,
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

impl Plotter<'_> {
    /// Main function for drawing plotter in view using GPU shaders.
    pub fn draw<'a, Message>(&'a self) -> Element<'a, Message>
    where
        Message: 'a,
    {
        shader(self).width(Length::Fill).height(Length::Fill).into()
    }
}
