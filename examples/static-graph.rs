use iced::widget::{column, row, text, Container};
use iced::{Color, Element, Length, Theme};
use iced_plotter::plotter::{
    ColorMode, InteractionConfig, PlotPoints, PlotSeries, Plotter, SeriesStyle, ViewState,
};

pub fn main() {
    iced::application(StaticGraph::default, StaticGraph::update, StaticGraph::view)
        .theme(Theme::GruvboxDark)
        .run()
        .unwrap()
}

#[derive(Debug, Clone)]
enum Message {
    ViewChanged(ViewState),
}

struct StaticGraph {
    view_state: ViewState,
}

impl StaticGraph {
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
        let panel = column![text("Column 1"), text("Column 2"), text("Column 3"),];

        let plotter = Plotter::new(
            vec![
                PlotSeries::new("sin(x)", PlotPoints::generator(f32::sin, (0.0, 10.0), 1000))
                    .with_style(SeriesStyle::new(ColorMode::solid(Color::from_rgb(
                        0.8, 0.4, 0.2,
                    )))),
            ],
            &self.view_state,
        )
        .with_interaction(InteractionConfig::full())
        .on_view_change(Message::ViewChanged);

        row![
            Container::new(plotter.draw())
                .width(Length::FillPortion(3))
                .height(Length::Fill),
            Container::new(panel)
                .width(Length::FillPortion(1))
                .height(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
