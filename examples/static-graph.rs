use iced::widget::{Container, column, row, text};
use iced::{Color, Element, Length, Theme};
use iced_plotter::plotter::{ColorMode, PlotPoints, PlotSeries, Plotter, SeriesStyle};

pub fn main() {
    iced::application(StaticGraph::default, StaticGraph::update, StaticGraph::view)
        .theme(Theme::GruvboxDark)
        .run()
        .unwrap()
}

#[derive(Debug)]
enum Message {}

struct StaticGraph<'a> {
    plotter: Plotter<'a>,
}

impl StaticGraph<'_> {
    pub fn default() -> Self {
        Self {
            plotter: Plotter::new(vec![
                PlotSeries::new("sin(x)", PlotPoints::generator(f32::sin, (0.0, 10.0), 1000))
                    .with_style(SeriesStyle::new(ColorMode::solid(Color::from_rgb(
                        0.8, 0.4, 0.2,
                    )))),
            ]),
        }
    }

    pub fn update(&mut self, _message: Message) {}

    pub fn view(&self) -> Element<'_, Message> {
        let panel = column![text("Column 1"), text("Column 2"), text("Column 3"),];

        row![
            Container::new(self.plotter.draw())
                .width(Length::FillPortion(3)) // 3/4 of space
                .height(Length::Fill),
            Container::new(panel)
                .width(Length::FillPortion(1)) // 1/4 of space
                .height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
