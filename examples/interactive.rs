use iced::widget::{column, row, text, Container};
use iced::{Color, Element, Length, Theme};
use iced_plotter::plotter::{
    ColorMode, InteractionConfig, PlotPoints, PlotSeries, Plotter, SeriesStyle, ViewState,
};

pub fn main() {
    iced::application(
        InteractiveExample::default,
        InteractiveExample::update,
        InteractiveExample::view,
    )
    .theme(Theme::GruvboxDark)
    .run()
    .unwrap()
}

#[derive(Debug, Clone)]
enum Message {
    ViewChanged(ViewState),
}

struct InteractiveExample {
    view_state: ViewState,
}

impl InteractiveExample {
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
        let x_info = match self.view_state.x_range {
            Some((lo, hi)) => format!("X: [{:.2}, {:.2}]", lo, hi),
            None => "X: auto-fit".to_string(),
        };
        let y_info = match self.view_state.y_range {
            Some((lo, hi)) => format!("Y: [{:.2}, {:.2}]", lo, hi),
            None => "Y: auto-fit".to_string(),
        };

        let info = column![
            text("Interactive Plot"),
            text(""),
            text("Controls:"),
            text("  Drag: Pan"),
            text("  Scroll: Zoom"),
            text("  Ctrl+Drag: Zoom select"),
            text("  Double-click: Reset"),
            text(""),
            text("Features:"),
            text("  - Elastic over-scroll"),
            text("  - Rectangle zoom select"),
            text("  - Boundary clamping"),
            text("  - X bounds: [0, 20]"),
            text(""),
            text("Current View:"),
            text(x_info),
            text(y_info),
        ]
        .spacing(5);

        let plotter = Plotter::new(
            vec![
                PlotSeries::new("sin(x)", PlotPoints::generator(f32::sin, (0.0, 20.0), 2000))
                    .with_style(SeriesStyle::new(ColorMode::solid(Color::from_rgb(
                        0.8, 0.4, 0.2,
                    )))),
                PlotSeries::new("cos(x)", PlotPoints::generator(f32::cos, (0.0, 20.0), 2000))
                    .with_style(SeriesStyle::new(ColorMode::solid(Color::from_rgb(
                        0.2, 0.6, 0.8,
                    )))),
                PlotSeries::new(
                    "sin(2x)*0.5",
                    PlotPoints::generator(|x| (2.0 * x).sin() * 0.5, (0.0, 20.0), 2000),
                )
                .with_style(SeriesStyle::new(ColorMode::solid(Color::from_rgb(
                    0.4, 0.8, 0.3,
                )))),
            ],
            &self.view_state,
        )
        .with_interaction(InteractionConfig {
            pan_x: true,
            pan_y: true,
            zoom_x: true,
            zoom_y: true,
            // Set X bounds to demonstrate elastic over-scroll and clamping
            x_bounds: Some((0.0, 20.0)),
            y_bounds: Some((-1.5, 1.5)),
            boundary_padding: 0.05,
            zoom_speed: 0.1,
            double_click_to_fit: true,
            zoom_select: true,
            elastic: true,
            elastic_limit: 0.3,
            elastic_duration_ms: 200,
        })
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
