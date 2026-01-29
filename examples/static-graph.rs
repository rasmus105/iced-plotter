use iced::widget::{column, row, text, Container};
use iced::{Element, Length, Theme};

use iced_graph::chart::{self, Chart};

pub fn main() {
    iced::application(StaticGraph::default, StaticGraph::update, StaticGraph::view)
        .theme(Theme::GruvboxDark)
        .run()
        .unwrap()
}

enum Message {}

#[derive(Default)]
struct StaticGraph {
    chart: Chart,
}

impl StaticGraph {
    pub fn default() -> Self {
        Self {
            chart: Chart::new(400., 300.),
        }
    }
    pub fn update(&mut self, _message: Message) {}

    pub fn view(&self) -> Element<'_, Message> {
        let panel = column![text("Column 1"), text("Column 2"), text("Column 3"),];

        row![
            Container::new(self.chart.draw())
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
