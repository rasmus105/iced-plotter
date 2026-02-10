use iced::time::{self, Duration};
use iced::widget::{column, row, text, Container};
use iced::{Color, Element, Length, Subscription, Theme};
use iced_plotter::plotter::{
    ColorMode, InteractionConfig, PlotPoint, PlotPoints, PlotSeries, Plotter, SeriesStyle,
    ViewState,
};

pub fn main() {
    iced::application(
        UpdatingGraph::new,
        UpdatingGraph::update,
        UpdatingGraph::view,
    )
    .subscription(UpdatingGraph::subscription)
    .theme(Theme::GruvboxDark)
    .run()
    .unwrap()
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    ViewChanged(ViewState),
}

struct UpdatingGraph {
    points: Vec<PlotPoint>,
    time: f32,
    view_state: ViewState,
}

impl UpdatingGraph {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            time: 0.0,
            view_state: ViewState::auto_fit(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_micros(10)).map(|_| Message::Tick)
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                let x = self.time;
                let y = (x * 0.001).sin() + (x * 0.000314).cos() * 6.28;

                self.points.push(PlotPoint { x, y });

                if self.points.len() > 100000 {
                    self.points.remove(0);
                }

                self.time += 0.1;
            }
            Message::ViewChanged(new_view) => {
                self.view_state = new_view;
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let point_count = self.points.len();

        let info = column![
            text("Updating Graph"),
            text(format!("Points: {}", point_count)),
            text(format!("Time: {:.1}", self.time)),
        ]
        .spacing(10);

        let plotter = Plotter::new(
            vec![
                PlotSeries::new("wave", PlotPoints::borrowed(&self.points)).with_style(
                    SeriesStyle::new(ColorMode::solid(Color::from_rgb(0.2, 0.8, 0.4))),
                ),
            ],
            &self.view_state,
        )
        .with_interaction(InteractionConfig::pan_x_autofit_y())
        .on_view_change(Message::ViewChanged);

        row![
            Container::new(plotter.draw())
                .width(Length::FillPortion(3))
                .height(Length::Fill),
            Container::new(info)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .padding(20),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
