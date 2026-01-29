use crate::plotter::Plotter;
use iced::widget::canvas;
use iced::{mouse, Rectangle, Renderer, Theme};

#[derive(Default)]
pub struct PlotterState {
    pub is_dragging: bool,
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
}

impl<Message> canvas::Program<Message> for Plotter<'_> {
    type State = PlotterState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let padding = 50.0;

        self.draw_series(
            &mut frame,
            state,
            bounds.width,
            bounds.height,
            padding,
            theme.palette().primary,
        );

        self.draw_legend();

        self.draw_axes(
            &mut frame,
            bounds.width,
            bounds.height,
            padding,
            theme.palette().text,
        );

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &iced::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        None
    }
}
