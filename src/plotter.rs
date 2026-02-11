use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use iced::widget::canvas;
use iced::widget::shader;
use iced::widget::stack;
use iced::{Element, Font, Length, Point, Renderer, Theme};

/// Shared state for the legend, including visibility toggles and layout info.
///
/// Store this in your application state and pass it to [`Plotter::with_legend_state`]
/// to persist legend toggle state and enable proper hit testing across frames.
///
/// Create with `LegendState::default()`.
#[derive(Clone, Debug, Default)]
pub struct LegendState {
    pub hidden_series: Rc<RefCell<HashSet<usize>>>,
    pub layout: Rc<RefCell<LegendLayout>>,
}

/// For backwards compatibility — alias for the hidden series set.
pub type HiddenSeries = Rc<RefCell<HashSet<usize>>>;

// ================================================================================
// Interaction Types
// ================================================================================

/// Represents the visible range of the plot.
///
/// Each axis range is `Option` — `None` means "auto-fit to data bounds".
/// This allows the common pattern of panning X while auto-fitting Y.
///
/// Owned by the user's application state and passed to [`Plotter`].
#[derive(Clone, Debug, Default)]
pub struct ViewState {
    /// Visible X range. `None` = auto-fit to data bounds.
    pub x_range: Option<(f32, f32)>,
    /// Visible Y range. `None` = auto-fit to data bounds.
    pub y_range: Option<(f32, f32)>,
}

impl ViewState {
    /// Create a new ViewState with both axes auto-fitting to data.
    pub fn auto_fit() -> Self {
        Self {
            x_range: None,
            y_range: None,
        }
    }

    /// Create a new ViewState with explicit ranges for both axes.
    pub fn with_ranges(x_range: (f32, f32), y_range: (f32, f32)) -> Self {
        Self {
            x_range: Some(x_range),
            y_range: Some(y_range),
        }
    }

    /// Set the X range (or None to auto-fit).
    pub fn with_x_range(mut self, range: Option<(f32, f32)>) -> Self {
        self.x_range = range;
        self
    }

    /// Set the Y range (or None to auto-fit).
    pub fn with_y_range(mut self, range: Option<(f32, f32)>) -> Self {
        self.y_range = range;
        self
    }
}

/// Configuration for what interactions are enabled on the plot.
#[derive(Clone, Debug)]
pub struct InteractionConfig {
    /// Allow panning along the X axis.
    pub pan_x: bool,
    /// Allow panning along the Y axis.
    pub pan_y: bool,
    /// Allow zooming along the X axis.
    pub zoom_x: bool,
    /// Allow zooming along the Y axis.
    pub zoom_y: bool,
    /// Hard limits for X scrolling. `None` = no limits.
    pub x_bounds: Option<(f32, f32)>,
    /// Hard limits for Y scrolling. `None` = no limits.
    pub y_bounds: Option<(f32, f32)>,
    /// Percentage of visible range to show as padding beyond data bounds (0.0 - 1.0).
    pub boundary_padding: f32,
    /// Zoom speed multiplier (default 0.1 = 10% per scroll tick).
    pub zoom_speed: f32,
    /// Enable double-click to reset view (fit all data).
    pub double_click_to_fit: bool,
    /// Enable Ctrl+drag rectangle zoom selection.
    pub zoom_select: bool,
    /// Enable elastic over-scroll with spring-back animation.
    pub elastic: bool,
    /// How far past bounds you can over-scroll (fraction of view range, 0.0 - 1.0).
    /// Higher = more stretchy. Default 0.3.
    pub elastic_limit: f32,
    /// Duration of the spring-back animation in milliseconds. Default 200.
    pub elastic_duration_ms: u64,
}

impl Default for InteractionConfig {
    fn default() -> Self {
        Self {
            pan_x: true,
            pan_y: false,
            zoom_x: true,
            zoom_y: false,
            x_bounds: None,
            y_bounds: None,
            boundary_padding: 0.05,
            zoom_speed: 0.1,
            double_click_to_fit: true,
            zoom_select: true,
            elastic: true,
            elastic_limit: 0.3,
            elastic_duration_ms: 200,
        }
    }
}

impl InteractionConfig {
    /// No interactions enabled.
    pub fn none() -> Self {
        Self {
            pan_x: false,
            pan_y: false,
            zoom_x: false,
            zoom_y: false,
            x_bounds: None,
            y_bounds: None,
            boundary_padding: 0.05,
            zoom_speed: 0.1,
            double_click_to_fit: false,
            zoom_select: false,
            elastic: false,
            elastic_limit: 0.3,
            elastic_duration_ms: 200,
        }
    }

    /// Pan and zoom on both axes.
    pub fn full() -> Self {
        Self {
            pan_x: true,
            pan_y: true,
            zoom_x: true,
            zoom_y: true,
            ..Self::default()
        }
    }

    /// Pan X, auto-fit Y (common for time-series).
    pub fn pan_x_autofit_y() -> Self {
        Self::default()
    }
}

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

    /// Get a single representative color for this color mode (used in legends).
    pub fn representative_color(&self) -> iced::Color {
        match self {
            ColorMode::Solid(c) => *c,
            ColorMode::ValueGradient { low, high, .. } => {
                // Midpoint blend
                iced::Color::from_rgb(
                    (low.r + high.r) / 2.0,
                    (low.g + high.g) / 2.0,
                    (low.b + high.b) / 2.0,
                )
            }
            ColorMode::IndexGradient { start, end } => {
                // Midpoint blend
                iced::Color::from_rgb(
                    (start.r + end.r) / 2.0,
                    (start.g + end.g) / 2.0,
                    (start.b + end.b) / 2.0,
                )
            }
            ColorMode::Colormap { name, .. } => {
                // Sample at midpoint
                name.sample(0.5)
            }
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

impl PlotPoints<'_> {
    /// Get the last Y value in the series (for legend display).
    pub fn last_y(&self) -> Option<f32> {
        match self {
            PlotPoints::Owned(pts) => pts.last().map(|p| p.y),
            PlotPoints::Borrowed(pts) => pts.last().map(|p| p.y),
            PlotPoints::Generator(_) => None, // generators don't have a "latest" point
        }
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
// Legend Types
// ================================================================================

/// Position of the legend within the plot area.
#[derive(Clone, Debug, Copy, Default, PartialEq, Eq)]
pub enum LegendPosition {
    #[default]
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

/// Configuration for the plot legend.
pub struct LegendConfig {
    /// Position of the legend within the plot area.
    pub position: LegendPosition,
    /// Color of the label text.
    pub text_color: iced::Color,
    /// Font size for legend labels.
    pub text_size: f32,
    /// Background color of the legend box.
    pub background_color: iced::Color,
    /// Internal padding within the legend box.
    pub padding: f32,
    /// Distance from the plot edge.
    pub margin: f32,
    /// Size of the color toggle square.
    pub toggle_size: f32,
    /// Whether to show the latest value next to the label.
    pub show_value: bool,
    /// Format function for the latest value.
    pub value_format: Box<dyn Fn(f32) -> String>,
}

impl Default for LegendConfig {
    fn default() -> Self {
        Self {
            position: LegendPosition::default(),
            text_color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.7),
            text_size: 12.0,
            background_color: iced::Color::from_rgba(0.1, 0.1, 0.1, 0.8),
            padding: 8.0,
            margin: 10.0,
            toggle_size: 12.0,
            show_value: true,
            value_format: Box::new(|v| format!("{v:.2}")),
        }
    }
}

impl Clone for LegendConfig {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            text_color: self.text_color,
            text_size: self.text_size,
            background_color: self.background_color,
            padding: self.padding,
            margin: self.margin,
            toggle_size: self.toggle_size,
            show_value: self.show_value,
            value_format: Box::new(|v| format!("{v:.2}")),
        }
    }
}

impl std::fmt::Debug for LegendConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LegendConfig")
            .field("position", &self.position)
            .field("text_color", &self.text_color)
            .field("text_size", &self.text_size)
            .field("background_color", &self.background_color)
            .field("padding", &self.padding)
            .field("margin", &self.margin)
            .field("toggle_size", &self.toggle_size)
            .field("show_value", &self.show_value)
            .finish()
    }
}

impl LegendConfig {
    /// Set the value format function.
    pub fn with_value_format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.value_format = Box::new(f);
        self
    }
}

// ================================================================================
// Tooltip Types
// ================================================================================

/// Configuration for hover tooltips on data points.
pub struct TooltipConfig {
    /// Maximum screen-space distance (in pixels) to snap to a point.
    pub max_distance: f32,
    /// Background color of the tooltip box.
    pub background_color: iced::Color,
    /// Text color inside the tooltip.
    pub text_color: iced::Color,
    /// Font size for tooltip text.
    pub text_size: f32,
    /// Internal padding within the tooltip box.
    pub padding: f32,
    /// Format function for the X value.
    pub format_x: Box<dyn Fn(f32) -> String>,
    /// Format function for the Y value.
    pub format_y: Box<dyn Fn(f32) -> String>,
    /// Color of the highlight ring drawn around the hovered point.
    pub highlight_color: iced::Color,
    /// Radius of the highlight ring (in pixels).
    pub highlight_radius: f32,
    /// Line width of the highlight ring (in pixels).
    pub highlight_width: f32,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            max_distance: 10.0,
            background_color: iced::Color::from_rgba(0.1, 0.1, 0.1, 0.9),
            text_color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.9),
            text_size: 12.0,
            padding: 6.0,
            format_x: Box::new(|v| format!("{v:.2}")),
            format_y: Box::new(|v| format!("{v:.2}")),
            highlight_color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.8),
            highlight_radius: 8.0,
            highlight_width: 2.0,
        }
    }
}

impl Clone for TooltipConfig {
    fn clone(&self) -> Self {
        Self {
            max_distance: self.max_distance,
            background_color: self.background_color,
            text_color: self.text_color,
            text_size: self.text_size,
            padding: self.padding,
            format_x: Box::new(|v| format!("{v:.2}")),
            format_y: Box::new(|v| format!("{v:.2}")),
            highlight_color: self.highlight_color,
            highlight_radius: self.highlight_radius,
            highlight_width: self.highlight_width,
        }
    }
}

impl std::fmt::Debug for TooltipConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TooltipConfig")
            .field("max_distance", &self.max_distance)
            .field("text_size", &self.text_size)
            .field("highlight_radius", &self.highlight_radius)
            .finish()
    }
}

impl TooltipConfig {
    /// Set the X value format function.
    pub fn with_format_x(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format_x = Box::new(f);
        self
    }

    /// Set the Y value format function.
    pub fn with_format_y(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format_y = Box::new(f);
        self
    }
}

/// Information about a data point that the cursor is hovering near.
#[derive(Clone, Debug)]
pub struct HoveredPoint {
    /// Index of the series this point belongs to.
    pub series_index: usize,
    /// Label of the series.
    pub series_label: String,
    /// Data-space X coordinate.
    pub x: f32,
    /// Data-space Y coordinate.
    pub y: f32,
    /// Screen-space position of the point (relative to widget bounds).
    pub screen_pos: Point,
}

/// Shared state for tooltip hover detection.
///
/// Store this in your application state and pass it to [`Plotter::with_tooltip_state`]
/// to enable tooltip rendering. The shader layer writes hovered point info,
/// and the canvas overlay reads it to draw the tooltip.
///
/// Create with `TooltipState::default()`.
#[derive(Clone, Debug, Default)]
pub struct TooltipState {
    pub hovered: Rc<RefCell<Option<HoveredPoint>>>,
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
    /// Optional axis title (e.g. "Time (s)", "Temperature (°C)").
    pub title: Option<String>,
    /// Color for the axis title text.
    pub title_color: iced::Color,
    /// Font size for the axis title.
    pub title_size: f32,
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
            title: self.title.clone(),
            title_color: self.title_color,
            title_size: self.title_size,
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
            title: None,
            title_color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.7),
            title_size: 14.0,
        }
    }
}

impl AxisConfig {
    pub fn with_format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format = Box::new(f);
        self
    }

    /// Set the axis title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the axis title color.
    pub fn with_title_color(mut self, color: iced::Color) -> Self {
        self.title_color = color;
        self
    }

    /// Set the axis title font size.
    pub fn with_title_size(mut self, size: f32) -> Self {
        self.title_size = size;
        self
    }
}

#[derive(Clone, Debug)]
pub struct PlotterOptions {
    /// Legend configuration. `None` = no legend, `Some(config)` = show legend.
    pub legend: Option<LegendConfig>,
    /// Tooltip configuration. `None` = no tooltip, `Some(config)` = show tooltip on hover.
    pub tooltip: Option<TooltipConfig>,
    pub padding: f32,
    pub grid: GridStyle,
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
    /// Fractional padding added around the data extent when auto-fitting.
    /// 0.05 means 5% of the data span is added on each side.
    /// Set to 0.0 to disable.
    pub autofit_padding: f32,
    /// Optional background color for the plot area (inside the padding).
    /// `Some(color)` draws a filled rectangle behind the grid and data.
    /// Defaults to a subtle darkening overlay for visual separation.
    pub background_color: Option<iced::Color>,
}

impl Default for PlotterOptions {
    fn default() -> Self {
        Self {
            legend: None,
            tooltip: None,
            padding: 50.0,
            grid: GridStyle::default(),
            x_axis: AxisConfig::default(),
            y_axis: AxisConfig::default(),
            autofit_padding: 0.05,
            background_color: Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.15)),
        }
    }
}

pub struct Plotter<'a, Message> {
    // data related
    pub series: Vec<PlotSeries<'a>>,

    // configuration related
    pub options: PlotterOptions,

    // interaction
    pub view_state: &'a ViewState,
    pub interaction: InteractionConfig,

    // callback: maps a new ViewState to the user's Message type
    pub(crate) on_view_change: Option<Box<dyn Fn(ViewState) -> Message + 'a>>,

    // shared legend state (visibility toggles + layout for hit testing)
    pub(crate) legend_state: LegendState,

    // shared tooltip state (hovered point info for tooltip rendering)
    pub(crate) tooltip_state: TooltipState,
}

// ================================================================================
// Public Methods
// ================================================================================

impl<'a, Message> Plotter<'a, Message> {
    pub fn new(series: Vec<PlotSeries<'a>>, view_state: &'a ViewState) -> Self {
        Self {
            series,
            options: PlotterOptions::default(),
            view_state,
            interaction: InteractionConfig::default(),
            on_view_change: None,
            legend_state: LegendState::default(),
            tooltip_state: TooltipState::default(),
        }
    }

    /// Set the shared legend state.
    ///
    /// This allows you to persist legend toggle state and hit-test layout across frames.
    /// Create with `LegendState::default()` and store in your app state.
    pub fn with_legend_state(mut self, state: LegendState) -> Self {
        self.legend_state = state;
        self
    }

    /// Set the shared tooltip state.
    ///
    /// This allows you to persist tooltip hover state across frames.
    /// Create with `TooltipState::default()` and store in your app state.
    pub fn with_tooltip_state(mut self, state: TooltipState) -> Self {
        self.tooltip_state = state;
        self
    }

    /// Set the shared hidden series state (convenience method).
    ///
    /// This allows you to persist legend toggle state across frames.
    /// Create with `Rc::new(RefCell::new(HashSet::new()))` and store in your app state.
    pub fn with_hidden_series(mut self, hidden: HiddenSeries) -> Self {
        self.legend_state.hidden_series = hidden;
        self
    }

    pub fn with_options(mut self, options: PlotterOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_interaction(mut self, interaction: InteractionConfig) -> Self {
        self.interaction = interaction;
        self
    }

    /// Set a callback that maps view state changes to your app's Message type.
    /// Without this, pan/zoom interactions will not be communicated back.
    pub fn on_view_change(mut self, f: impl Fn(ViewState) -> Message + 'a) -> Self {
        self.on_view_change = Some(Box::new(f));
        self
    }

    /// Compute the bounding box of all visible (non-hidden) data points.
    pub fn compute_data_ranges(&self) -> ([f32; 2], [f32; 2]) {
        let mut x_min = f32::INFINITY;
        let mut x_max = f32::NEG_INFINITY;
        let mut y_min = f32::INFINITY;
        let mut y_max = f32::NEG_INFINITY;

        let hidden = self.legend_state.hidden_series.borrow();
        for (idx, s) in self.series.iter().enumerate() {
            if hidden.contains(&idx) {
                continue;
            }
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

    /// Resolve the actual view ranges by combining ViewState with data bounds.
    ///
    /// When `enforce_bounds` is `true` and elastic bounds are active, explicit
    /// view ranges are clamped so the rendered view stays within the padded
    /// data bounds.  Pass `false` during active drag / elastic animation so
    /// that over-scroll is still visible.
    ///
    /// Returns (view_x_range, view_y_range, data_x_range, data_y_range).
    pub fn resolve_view_ranges(&self, enforce_bounds: bool) -> ([f32; 2], [f32; 2], [f32; 2], [f32; 2]) {
        let (data_x, data_y) = self.compute_data_ranges();
        let af = self.options.autofit_padding;
        let interaction = &self.interaction;

        let view_x = match self.view_state.x_range {
            Some((lo, hi)) => {
                if enforce_bounds && interaction.elastic && interaction.pan_x {
                    let bounds = interaction.x_bounds.or(Some((data_x[0], data_x[1])));
                    let (clo, chi) =
                        crate::shader::clamp_range_to_bounds((lo, hi), bounds, interaction.boundary_padding);
                    [clo, chi]
                } else {
                    [lo, hi]
                }
            }
            None => {
                let span = data_x[1] - data_x[0];
                let margin = span * af;
                [data_x[0] - margin, data_x[1] + margin]
            }
        };
        let view_y = match self.view_state.y_range {
            Some((lo, hi)) => {
                if enforce_bounds && interaction.elastic && interaction.pan_y {
                    let bounds = interaction.y_bounds.or(Some((data_y[0], data_y[1])));
                    let (clo, chi) =
                        crate::shader::clamp_range_to_bounds((lo, hi), bounds, interaction.boundary_padding);
                    [clo, chi]
                } else {
                    [lo, hi]
                }
            }
            None => {
                let span = data_y[1] - data_y[0];
                let margin = span * af;
                [data_y[0] - margin, data_y[1] + margin]
            }
        };

        (view_x, view_y, data_x, data_y)
    }

    /// Build the plotter widget. Consumes `self` (the Plotter is a builder).
    pub fn draw(self) -> Element<'a, Message>
    where
        Message: Clone + 'a,
    {
        let (view_x, view_y, _, _) = self.resolve_view_ranges(true);

        let x_ticks = crate::ticks::compute_ticks(view_x[0], view_x[1], &self.options.x_axis.ticks);
        let y_ticks = crate::ticks::compute_ticks(view_y[0], view_y[1], &self.options.y_axis.ticks);

        let x_labels: Vec<String> = x_ticks
            .iter()
            .map(|v| (self.options.x_axis.format)(*v))
            .collect();
        let y_labels: Vec<String> = y_ticks
            .iter()
            .map(|v| (self.options.y_axis.format)(*v))
            .collect();

        // Build legend entries if legend is enabled
        let legend_entries: Vec<LegendEntry> = if self.options.legend.is_some() {
            self.series
                .iter()
                .map(|s| LegendEntry {
                    label: s.label.clone(),
                    color: s.style.color.representative_color(),
                    latest_value: s.points.last_y(),
                })
                .collect()
        } else {
            Vec::new()
        };

        let overlay = AxisOverlay {
            x_ticks,
            y_ticks,
            x_labels,
            y_labels,
            x_range: view_x,
            y_range: view_y,
            padding: self.options.padding,
            x_label_color: self.options.x_axis.label_color,
            y_label_color: self.options.y_axis.label_color,
            x_label_size: self.options.x_axis.label_size,
            y_label_size: self.options.y_axis.label_size,
            show_x: self.options.x_axis.show,
            show_y: self.options.y_axis.show,
            // Axis titles
            x_title: self.options.x_axis.title.clone(),
            x_title_color: self.options.x_axis.title_color,
            x_title_size: self.options.x_axis.title_size,
            y_title: self.options.y_axis.title.clone(),
            y_title_color: self.options.y_axis.title_color,
            y_title_size: self.options.y_axis.title_size,
            // Legend
            legend_config: self.options.legend.clone(),
            legend_entries,
            hidden_series: self.legend_state.hidden_series.clone(),
            legend_layout: self.legend_state.layout.clone(),
            // Tooltip
            tooltip_config: self.options.tooltip.clone(),
            tooltip_state: self.tooltip_state.clone(),
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

/// Computed rectangle for a legend toggle button (for hit testing).
#[derive(Clone, Debug)]
pub struct LegendToggleRect {
    pub series_index: usize,
    /// Rectangle in widget-local coordinates.
    pub rect: iced::Rectangle,
}

/// Precomputed legend layout for hit testing from the shader.
#[derive(Clone, Debug, Default)]
pub struct LegendLayout {
    /// Bounding box of the entire legend (for blocking interactions).
    pub bounds: Option<iced::Rectangle>,
    /// Individual toggle button rects.
    pub toggles: Vec<LegendToggleRect>,
}

/// Shared legend layout info for hit testing from the shader.
pub type LegendLayoutInfo = Rc<RefCell<LegendLayout>>;

/// Data for a single legend entry.
#[derive(Clone, Debug)]
struct LegendEntry {
    label: String,
    color: iced::Color,
    latest_value: Option<f32>,
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
    // Axis titles
    x_title: Option<String>,
    x_title_color: iced::Color,
    x_title_size: f32,
    y_title: Option<String>,
    y_title_color: iced::Color,
    y_title_size: f32,
    // Legend
    legend_config: Option<LegendConfig>,
    legend_entries: Vec<LegendEntry>,
    hidden_series: HiddenSeries,
    legend_layout: LegendLayoutInfo,
    // Tooltip
    tooltip_config: Option<TooltipConfig>,
    tooltip_state: TooltipState,
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

        // ---- X tick labels ----
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

        // ---- Y tick labels ----
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

        // ---- X axis title ----
        if let Some(ref title) = self.x_title {
            let center_x = self.padding + plot_width / 2.0;
            // Place below tick labels: padding + plot_height + tick_label_space
            let y = self.padding + plot_height + 6.0 + self.x_label_size + 8.0;
            frame.fill_text(canvas::Text {
                content: title.clone(),
                size: iced::Pixels(self.x_title_size),
                position: Point::new(center_x, y),
                color: self.x_title_color,
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Top,
                font: Font::DEFAULT,
                ..canvas::Text::default()
            });
        }

        // ---- Y axis title (rotated 90° counter-clockwise) ----
        if let Some(ref title) = self.y_title {
            let center_y = self.padding + plot_height / 2.0;
            // Place to the left of tick labels
            let x = 4.0;
            frame.with_save(|frame| {
                // Move to the desired position, rotate, then draw centered at origin
                frame.translate(iced::Vector::new(x, center_y));
                frame.rotate(-std::f32::consts::FRAC_PI_2);
                frame.fill_text(canvas::Text {
                    content: title.clone(),
                    size: iced::Pixels(self.y_title_size),
                    position: Point::new(0.0, 0.0),
                    color: self.y_title_color,
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Top,
                    font: Font::DEFAULT,
                    ..canvas::Text::default()
                });
            });
        }

        // ---- Legend ----
        if let Some(ref config) = self.legend_config {
            let hidden = self.hidden_series.borrow();
            let mut toggle_rects: Vec<LegendToggleRect> = Vec::new();
            let mut legend_bg_rect: Option<iced::Rectangle> = None;

            let row_height = config.toggle_size.max(config.text_size) + 4.0;
            let num_entries = self.legend_entries.len();
            if num_entries > 0 {
                // Estimate legend box dimensions
                // Each row: [toggle_square] [gap] [label] [gap] [value]
                let gap = 6.0;
                let value_format = &config.value_format;
                let mut max_text_width: f32 = 0.0;
                for entry in &self.legend_entries {
                    // Rough character width estimate: text_size * 0.6 per char (monospace)
                    let char_width = config.text_size * 0.6;
                    let label_width = entry.label.len() as f32 * char_width;
                    let value_width = if config.show_value {
                        if let Some(v) = entry.latest_value {
                            let formatted = (value_format)(v);
                            (formatted.len() as f32 + 1.0) * char_width // +1 for space
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    max_text_width = max_text_width.max(label_width + value_width);
                }

                let legend_width = config.padding * 2.0 + config.toggle_size + gap + max_text_width;
                let legend_height = config.padding * 2.0 + num_entries as f32 * row_height - 4.0;

                // Position based on legend position
                let (legend_x, legend_y) = match config.position {
                    LegendPosition::TopRight => (
                        self.padding + plot_width - config.margin - legend_width,
                        self.padding + config.margin,
                    ),
                    LegendPosition::TopLeft => {
                        (self.padding + config.margin, self.padding + config.margin)
                    }
                    LegendPosition::BottomRight => (
                        self.padding + plot_width - config.margin - legend_width,
                        self.padding + plot_height - config.margin - legend_height,
                    ),
                    LegendPosition::BottomLeft => (
                        self.padding + config.margin,
                        self.padding + plot_height - config.margin - legend_height,
                    ),
                };

                // Draw background
                let bg_rect = iced::Rectangle::new(
                    Point::new(legend_x, legend_y),
                    iced::Size::new(legend_width, legend_height),
                );
                legend_bg_rect = Some(bg_rect);
                frame.fill_rectangle(bg_rect.position(), bg_rect.size(), config.background_color);

                // Draw border
                frame.stroke_rectangle(
                    bg_rect.position(),
                    bg_rect.size(),
                    canvas::Stroke::default()
                        .with_color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.2))
                        .with_width(1.0),
                );

                // Draw entries
                for (i, entry) in self.legend_entries.iter().enumerate() {
                    let is_hidden = hidden.contains(&i);
                    let entry_y = legend_y + config.padding + i as f32 * row_height;

                    // Toggle square (rounded rect)
                    let toggle_x = legend_x + config.padding;
                    let toggle_y = entry_y + (row_height - 4.0 - config.toggle_size) / 2.0;
                    let toggle_rect = iced::Rectangle::new(
                        Point::new(toggle_x, toggle_y),
                        iced::Size::new(config.toggle_size, config.toggle_size),
                    );

                    // Store for hit testing
                    toggle_rects.push(LegendToggleRect {
                        series_index: i,
                        rect: toggle_rect,
                    });

                    let toggle_color = if is_hidden {
                        // Dimmed version of the color
                        iced::Color::from_rgba(
                            entry.color.r * 0.3,
                            entry.color.g * 0.3,
                            entry.color.b * 0.3,
                            0.5,
                        )
                    } else {
                        entry.color
                    };

                    // Draw rounded rectangle for toggle
                    let corner_radius: f32 = 3.0;
                    let rounded_path = canvas::path::Builder::new();
                    let mut builder = rounded_path;
                    // Build a rounded rect path
                    let rx = toggle_x;
                    let ry = toggle_y;
                    let rw = config.toggle_size;
                    let rh = config.toggle_size;
                    let r = corner_radius.min(rw / 2.0).min(rh / 2.0);
                    builder.move_to(Point::new(rx + r, ry));
                    builder.line_to(Point::new(rx + rw - r, ry));
                    builder.arc_to(Point::new(rx + rw, ry), Point::new(rx + rw, ry + r), r);
                    builder.line_to(Point::new(rx + rw, ry + rh - r));
                    builder.arc_to(
                        Point::new(rx + rw, ry + rh),
                        Point::new(rx + rw - r, ry + rh),
                        r,
                    );
                    builder.line_to(Point::new(rx + r, ry + rh));
                    builder.arc_to(Point::new(rx, ry + rh), Point::new(rx, ry + rh - r), r);
                    builder.line_to(Point::new(rx, ry + r));
                    builder.arc_to(Point::new(rx, ry), Point::new(rx + r, ry), r);
                    builder.close();
                    let path = builder.build();
                    frame.fill(&path, toggle_color);

                    // Draw border on toggle
                    frame.stroke(
                        &path,
                        canvas::Stroke::default()
                            .with_color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.3))
                            .with_width(1.0),
                    );

                    // Label text
                    let text_x = toggle_x + config.toggle_size + gap;
                    let text_y = entry_y + (row_height - 4.0) / 2.0;
                    let text_color = if is_hidden {
                        iced::Color::from_rgba(
                            config.text_color.r,
                            config.text_color.g,
                            config.text_color.b,
                            config.text_color.a * 0.4,
                        )
                    } else {
                        config.text_color
                    };

                    let mut display_text = entry.label.clone();
                    if config.show_value
                        && let Some(v) = entry.latest_value
                    {
                        display_text.push_str(&format!(" {}", (value_format)(v)));
                    }

                    frame.fill_text(canvas::Text {
                        content: display_text,
                        size: iced::Pixels(config.text_size),
                        position: Point::new(text_x, text_y),
                        color: text_color,
                        align_x: iced::alignment::Horizontal::Left.into(),
                        align_y: iced::alignment::Vertical::Center,
                        font: Font::MONOSPACE,
                        ..canvas::Text::default()
                    });
                }
            }

            // Update shared legend layout for hit testing
            *self.legend_layout.borrow_mut() = LegendLayout {
                bounds: legend_bg_rect,
                toggles: toggle_rects,
            };
        }

        // ---- Tooltip ----
        if let Some(ref config) = self.tooltip_config {
            let hovered = self.tooltip_state.hovered.borrow();
            if let Some(ref hp) = *hovered {
                let format_x = &config.format_x;
                let format_y = &config.format_y;
                let text = format!(
                    "{}: ({}, {})",
                    hp.series_label,
                    (format_x)(hp.x),
                    (format_y)(hp.y)
                );

                // Estimate text dimensions
                let char_width = config.text_size * 0.6;
                let text_width = text.len() as f32 * char_width;
                let text_height = config.text_size;

                let box_width = text_width + config.padding * 2.0;
                let box_height = text_height + config.padding * 2.0;

                // Position tooltip above and to the right of the point, with clamping
                let offset_x = 12.0;
                let offset_y = -12.0;

                let mut tooltip_x = hp.screen_pos.x + offset_x;
                let mut tooltip_y = hp.screen_pos.y + offset_y - box_height;

                // Clamp to widget bounds
                if tooltip_x + box_width > bounds.width {
                    tooltip_x = hp.screen_pos.x - offset_x - box_width;
                }
                if tooltip_x < 0.0 {
                    tooltip_x = 0.0;
                }
                if tooltip_y < 0.0 {
                    tooltip_y = hp.screen_pos.y + offset_x; // flip below
                }
                if tooltip_y + box_height > bounds.height {
                    tooltip_y = bounds.height - box_height;
                }

                // Draw background
                frame.fill_rectangle(
                    Point::new(tooltip_x, tooltip_y),
                    iced::Size::new(box_width, box_height),
                    config.background_color,
                );

                // Draw border
                frame.stroke_rectangle(
                    Point::new(tooltip_x, tooltip_y),
                    iced::Size::new(box_width, box_height),
                    canvas::Stroke::default()
                        .with_color(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.3))
                        .with_width(1.0),
                );

                // Draw text
                frame.fill_text(canvas::Text {
                    content: text,
                    size: iced::Pixels(config.text_size),
                    position: Point::new(
                        tooltip_x + config.padding,
                        tooltip_y + config.padding + text_height / 2.0,
                    ),
                    color: config.text_color,
                    align_x: iced::alignment::Horizontal::Left.into(),
                    align_y: iced::alignment::Vertical::Center,
                    font: Font::MONOSPACE,
                    ..canvas::Text::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }
}
