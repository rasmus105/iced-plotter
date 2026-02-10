use iced::widget::{column, row, text, Container};
use iced::{Color, Element, Length, Theme};
use iced_plotter::colormap::ColormapName;
use iced_plotter::plotter::{
    ColorMode, InteractionConfig, PlotPoints, PlotSeries, Plotter, SeriesStyle, ViewState,
};

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

#[derive(Debug, Clone)]
enum Message {
    ViewChanged(ViewState),
}

struct ColoringExample {
    view_state: ViewState,
}

impl ColoringExample {
    pub fn default() -> Self {
        Self {
            view_state: ViewState::auto_fit(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::ViewChanged(new_view) => {
                self.view_state = new_view;
            }
        }
    }

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

        let plotter = Plotter::new(
            vec![
                // Value gradient based on Y
                PlotSeries::new(
                    "sin(x) - Value Gradient",
                    PlotPoints::generator(f32::sin, (0.0, 10.0), 500),
                )
                .with_style(SeriesStyle::new(ColorMode::value_gradient(
                    Color::from_rgb(0.2, 0.2, 0.8),
                    Color::from_rgb(0.8, 0.2, 0.2),
                ))),
                // Index gradient
                PlotSeries::new(
                    "cos(x) - Index Gradient",
                    PlotPoints::generator(f32::cos, (0.0, 10.0), 500),
                )
                .with_style(SeriesStyle::new(ColorMode::index_gradient(
                    Color::from_rgb(0.2, 0.4, 0.8),
                    Color::from_rgb(0.8, 0.8, 0.2),
                ))),
                // Colormap
                PlotSeries::new(
                    "sin(2x) - Viridis Colormap",
                    PlotPoints::generator(|x| (2.0 * x).sin(), (0.0, 10.0), 500),
                )
                .with_style(SeriesStyle::new(ColorMode::colormap(ColormapName::Viridis))),
            ],
            &self.view_state,
        )
        .with_interaction(InteractionConfig::full())
        .on_view_change(Message::ViewChanged);

        row![
            Container::new(plotter.draw())
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
