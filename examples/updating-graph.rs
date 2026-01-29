use iced::time::{self, Duration};
use iced::widget::{column, row, text, Container};
use iced::{Color, Element, Length, Subscription, Theme};
use iced_plotter::plotter::{PlotPoint, PlotPoints, PlotSeries, Plotter, PlotterOptions};

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
}

struct UpdatingGraph<'a> {
    plotter: Plotter<'a>,
    time: f64,
}

impl UpdatingGraph<'_> {
    pub fn new() -> Self {
        Self {
            plotter: Plotter {
                series: vec![PlotSeries {
                    label: "wave".to_string(),
                    color: Color::from_rgb(0.2, 0.8, 0.4),
                    points: PlotPoints::Owned(Vec::new()),
                }],
                options: PlotterOptions::default(),
            },
            time: 0.0,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_micros(10)).map(|_| Message::Tick)
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                // Add a new point based on current time
                let x = self.time;
                let y = (x * 0.001).sin() + (x * 0.000314).cos() * 6.28;

                // Get mutable access to the first series' owned points
                if let Some(series) = self.plotter.series.get_mut(0) && let PlotPoints::Owned(ref mut points) = series.points {
                    points.push(PlotPoint { x, y });

                    // Keep last 200 points for a sliding window effect
                    if points.len() > 500000 {
                        points.remove(0);
                    }
                }

                self.time += 0.1;
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let point_count = self
            .plotter
            .series
            .first()
            .map(|s| match &s.points {
                PlotPoints::Owned(points) => points.len(),
                _ => 0,
            })
            .unwrap_or(0);

        let info = column![
            text("Updating Graph"),
            text(format!("Points: {}", point_count)),
            text(format!("Time: {:.1}", self.time)),
        ]
        .spacing(10);

        row![
            Container::new(self.plotter.draw())
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
