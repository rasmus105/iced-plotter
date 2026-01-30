use iced::widget::{column, row, text, Container};
use iced::{Color, Element, Length, Theme};
use iced_plotter::colormap::ColormapName;
use iced_plotter::plotter::{
    ColorMode, ExplicitGenerator, PlotPoints, PlotSeries, Plotter, PlotterOptions, SeriesStyle,
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

#[derive(Debug)]
enum Message {}

struct ColoringExample<'a> {
    plotter: Plotter<'a>,
}

impl ColoringExample<'_> {
    pub fn default() -> Self {
        Self {
            plotter: Plotter {
                series: vec![
                    // Value gradient based on Y (colored by the output of the function)
                    PlotSeries {
                        label: "sin(x) - Value Gradient".to_string(),
                        style: SeriesStyle::new(ColorMode::ValueGradient {
                            low: Color::from_rgb(0.2, 0.2, 0.8),
                            high: Color::from_rgb(0.8, 0.2, 0.2),
                            values: None,
                        }),
                        points: PlotPoints::Generator(ExplicitGenerator {
                            function: Box::new(f32::sin),
                            x_range: (0., 10.),
                            points: 500,
                        }),
                    },
                    // Index gradient (blue to yellow based on point position)
                    PlotSeries {
                        label: "cos(x) - Index Gradient".to_string(),
                        style: SeriesStyle::new(ColorMode::IndexGradient {
                            start: Color::from_rgb(0.2, 0.4, 0.8),
                            end: Color::from_rgb(0.8, 0.8, 0.2),
                        }),
                        points: PlotPoints::Generator(ExplicitGenerator {
                            function: Box::new(f32::cos),
                            x_range: (0., 10.),
                            points: 500,
                        }),
                    },
                    // Using a colormap (viridis)
                    PlotSeries {
                        label: "sin(2x) - Viridis Colormap".to_string(),
                        style: SeriesStyle::new(ColorMode::Colormap {
                            name: ColormapName::Viridis,
                            values: None,
                        }),
                        points: PlotPoints::Generator(ExplicitGenerator {
                            function: Box::new(|x| (2.0 * x).sin()),
                            x_range: (0., 10.),
                            points: 500,
                        }),
                    },
                ],
                options: PlotterOptions::default(),
            },
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
