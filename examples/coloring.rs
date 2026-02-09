use iced::widget::{Container, column, row, text};
use iced::{Color, Element, Length, Theme};
use iced_plotter::colormap::ColormapName;
use iced_plotter::plotter::{ColorMode, PlotPoints, PlotSeries, Plotter, SeriesStyle};

pub fn main() {
    iced::application(
        ColoringExample::default,
        ColoringExample::update,
        ColoringExample::view,
    )
    .theme(Theme::GruvboxDark)
    .run()
    .unwrap()
}

#[derive(Debug)]
enum Message {}

struct ColoringExample<'a> {
    plotter: Plotter<'a>,
}

impl ColoringExample<'_> {
    pub fn default() -> Self {
        Self {
            plotter: Plotter::new(vec![
                // Value gradient based on Y (colored by the output of the function)
                PlotSeries::new(
                    "sin(x) - Value Gradient",
                    PlotPoints::generator(f32::sin, (0.0, 10.0), 500),
                )
                .with_style(SeriesStyle::new(ColorMode::value_gradient(
                    Color::from_rgb(0.2, 0.2, 0.8),
                    Color::from_rgb(0.8, 0.2, 0.2),
                ))),
                // Index gradient (blue to yellow based on point position)
                PlotSeries::new(
                    "cos(x) - Index Gradient",
                    PlotPoints::generator(f32::cos, (0.0, 10.0), 500),
                )
                .with_style(SeriesStyle::new(ColorMode::index_gradient(
                    Color::from_rgb(0.2, 0.4, 0.8),
                    Color::from_rgb(0.8, 0.8, 0.2),
                ))),
                // Using a colormap (viridis)
                PlotSeries::new(
                    "sin(2x) - Viridis Colormap",
                    PlotPoints::generator(|x| (2.0 * x).sin(), (0.0, 10.0), 500),
                )
                .with_style(SeriesStyle::new(ColorMode::colormap(ColormapName::Viridis))),
            ]),
        }
    }

    pub fn update(&mut self, _message: Message) {}

    pub fn view(&self) -> Element<'_, Message> {
        let legend = column![
            text("Coloring Modes:"),
            text(""),
            text("1. Value Gradient:"),
            text("   Blue (low) → Red (high)"),
            text("   Based on Y value"),
            text(""),
            text("2. Index Gradient:"),
            text("   Blue (start) → Yellow (end)"),
            text("   Based on point order"),
            text(""),
            text("3. Colormap:"),
            text("   Viridis (scientific)"),
            text("   Perceptually uniform"),
        ]
        .spacing(5)
        .width(Length::Shrink);

        row![
            Container::new(self.plotter.draw())
                .width(Length::FillPortion(3))
                .height(Length::Fill),
            Container::new(legend)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .padding(20),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
