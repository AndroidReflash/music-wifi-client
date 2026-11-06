use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn equalizer<'a>(size: f32, color: Color) -> Element<'a, Message> {
    Canvas::new(Icon { size, color }).width(size).height(size).into()
}

struct Icon { size: f32, color: Color }

impl canvas::Program<Message> for Icon {
    type State = ();
    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let s = self.size;
        let w = Stroke::default().with_color(self.color).with_width((s * 0.1).max(1.0));
        let top = s * 0.22;
        let bot = s * 0.78;
        for (x, y) in [(s * 0.3, s * 0.45), (s * 0.5, s * 0.6), (s * 0.7, s * 0.38)] {
            frame.stroke(&Path::line(Point::new(x, top), Point::new(x, bot)), w);
            frame.fill(&Path::circle(Point::new(x, y), s * 0.08), self.color);
        }
        vec![frame.into_geometry()]
    }
}
