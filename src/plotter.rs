use iced::widget::shader;
use iced::{Color, Element, Length};

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
    pub color: Color,
    pub points: PlotPoints<'a>,
}

// ================================================================================
// Plotter
// ================================================================================

#[derive(Default)]
pub struct PlotterOptions {
    pub show_legend: bool,
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
