use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn close<'a>(size: f32, color: Color) -> Element<'a, Message> {
    Canvas::new(Icon { size, color }).width(size).height(size).into()
}

struct Icon { size: f32, color: Color }

impl canvas::Program<Message> for Icon {
    type State = ();
    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let s = self.size;
        let p = s * 0.26;
        let w = Stroke::default().with_color(self.color).with_width((s * 0.09).max(1.0));
        frame.stroke(&Path::line(Point::new(p, p), Point::new(s - p, s - p)), w);
        frame.stroke(&Path::line(Point::new(s - p, p), Point::new(p, s - p)), w);
        vec![frame.into_geometry()]
    }
}
