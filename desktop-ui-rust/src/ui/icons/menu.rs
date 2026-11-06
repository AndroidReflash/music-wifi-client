use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn menu<'a>(size: f32, color: Color) -> Element<'a, Message> {
    Canvas::new(Icon { size, color }).width(size).height(size).into()
}

struct Icon { size: f32, color: Color }

impl canvas::Program<Message> for Icon {
    type State = ();
    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let s = self.size;
        let x0 = s * 0.2;
        let x1 = s * 0.8;
        let w = Stroke::default().with_color(self.color).with_width((s * 0.09).max(1.0));
        for y in [s * 0.3, s * 0.5, s * 0.7] {
            frame.stroke(&Path::line(Point::new(x0, y), Point::new(x1, y)), w);
        }
        vec![frame.into_geometry()]
    }
}
