use iced::time::{self, Duration};
use iced::widget::{column, row, text, Container};
use iced::{Color, Element, Length, Subscription, Theme};
use iced_plotter::plotter::{
    AxisConfig, ColorMode, InteractionConfig, LegendConfig, LegendPosition, LegendState, PlotPoint,
    PlotPoints, PlotSeries, Plotter, PlotterOptions, SeriesStyle, ViewState,
};

pub fn main() {
    iced::application(SensorData::new, SensorData::update, SensorData::view)
        .subscription(SensorData::subscription)
        .theme(Theme::GruvboxDark)
        .run()
        .unwrap()
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    ViewChanged(ViewState),
}

/// Simple LCG pseudo-random number generator (no external deps).
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Returns a pseudo-random f32 in [-1.0, 1.0].
    fn next_f32(&mut self) -> f32 {
        // LCG parameters from Numerical Recipes
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        // Take upper bits, map to [-1, 1]
        let bits = (self.state >> 33) as f32 / (1u64 << 31) as f32;
        bits * 2.0 - 1.0
    }
}

const MAX_POINTS: usize = 500;

struct SensorData {
    temperature: Vec<PlotPoint>,
    humidity: Vec<PlotPoint>,
    time: f32,
    current_temp: f32,
    current_humidity: f32,
    rng: SimpleRng,
    view_state: ViewState,
    legend_state: LegendState,
}

impl SensorData {
    pub fn new() -> Self {
        Self {
            temperature: Vec::new(),
            humidity: Vec::new(),
            time: 0.0,
            current_temp: 22.0,
            current_humidity: 55.0,
            rng: SimpleRng::new(42),
            view_state: ViewState::auto_fit(),
            legend_state: LegendState::default(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(200)).map(|_| Message::Tick)
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                self.time += 0.2; // 200ms in seconds

                // Random walk for temperature: drift toward 22°C with noise
                let temp_noise = self.rng.next_f32() * 0.3;
                let temp_drift = (22.0 - self.current_temp) * 0.02;
                self.current_temp += temp_drift + temp_noise;

                // Random walk for humidity: drift toward 55% with noise
                let hum_noise = self.rng.next_f32() * 0.5;
                let hum_drift = (55.0 - self.current_humidity) * 0.02;
                self.current_humidity += hum_drift + hum_noise;

                self.temperature.push(PlotPoint {
                    x: self.time,
                    y: self.current_temp,
                });
                self.humidity.push(PlotPoint {
                    x: self.time,
                    y: self.current_humidity,
                });

                // Rolling window
                if self.temperature.len() > MAX_POINTS {
                    self.temperature.remove(0);
                }
                if self.humidity.len() > MAX_POINTS {
                    self.humidity.remove(0);
                }
            }
            Message::ViewChanged(new_view) => {
                self.view_state = new_view;
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let info = column![
            text("Sensor Data Simulation"),
            text(""),
            text(format!("Temperature: {:.1} °C", self.current_temp)),
            text(format!("Humidity:    {:.1} %", self.current_humidity)),
            text(""),
            text(format!("Points/series: {}", self.temperature.len())),
            text(format!("Time: {:.1}s", self.time)),
            text(""),
            text("Controls:"),
            text("  Drag X: Pan"),
            text("  Double-click: Reset"),
            text(""),
            text("Y-axis auto-fits to data"),
        ]
        .spacing(5);

        let plotter = Plotter::new(
            vec![
                PlotSeries::new("Temperature", PlotPoints::borrowed(&self.temperature)).with_style(
                    SeriesStyle::new(ColorMode::solid(Color::from_rgb(0.9, 0.3, 0.2))),
                ),
                PlotSeries::new("Humidity", PlotPoints::borrowed(&self.humidity)).with_style(
                    SeriesStyle::new(ColorMode::solid(Color::from_rgb(0.2, 0.6, 0.9))),
                ),
            ],
            &self.view_state,
        )
        .with_options(PlotterOptions {
            legend: Some(LegendConfig {
                position: LegendPosition::TopLeft,
                ..LegendConfig::default()
            }),
            x_axis: AxisConfig::default().with_title("Time (s)"),
            y_axis: AxisConfig::default().with_title("Value"),
            ..PlotterOptions::default()
        })
        .with_legend_state(self.legend_state.clone())
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
