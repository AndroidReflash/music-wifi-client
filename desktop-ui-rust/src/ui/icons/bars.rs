use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn bars<'a>(size: f32, color: Color) -> Element<'a, Message> {
    Canvas::new(Icon { size, color }).width(size).height(size).into()
}

struct Icon { size: f32, color: Color }

impl canvas::Program<Message> for Icon {
    type State = ();
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let s = self.size;
        let thickness = (s * 0.1).max(1.0);
        let heights = [0.35_f32, 0.6, 0.9, 0.5, 0.3];
        let places = [0.15_f32, 0.325, 0.5, 0.675, 0.85];
        for (share, place) in heights.iter().zip(places.iter()) {
            let x = s * *place;
            let half = s * *share / 2.0;
            frame.stroke(
                &Path::line(Point::new(x, s / 2.0 - half), Point::new(x, s / 2.0 + half)),
                Stroke::default().with_color(self.color).with_width(thickness).with_line_cap(canvas::LineCap::Round),
            );
        }
        vec![frame.into_geometry()]
    }
}
