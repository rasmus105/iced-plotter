use iced::widget::Container;
use iced::{Element, Length};

pub fn main() {
    iced::application(StaticGraph::default, StaticGraph::update, StaticGraph::view)
        .run()
        .unwrap()
}

enum Message {}

#[derive(Default)]
struct StaticGraph;

impl StaticGraph {
    pub fn update(&mut self, _message: Message) {}
    pub fn view(&self) -> Element<'_, Message> {
        Container::new("")
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
