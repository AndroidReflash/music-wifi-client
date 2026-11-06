use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn search<'a>(size: f32, color: Color) -> Element<'a, Message> {
    Canvas::new(Icon { size, color }).width(size).height(size).into()
}

struct Icon { size: f32, color: Color }

impl canvas::Program<Message> for Icon {
    type State = ();
    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let s = self.size;
        let w = Stroke::default().with_color(self.color).with_width((s * 0.09).max(1.0));
        let c = Point::new(s * 0.44, s * 0.44);
        let r = s * 0.22;
        frame.stroke(&Path::circle(c, r), w);
        frame.stroke(&Path::line(Point::new(c.x + r * 0.7, c.y + r * 0.7), Point::new(s * 0.8, s * 0.8)), w);
        vec![frame.into_geometry()]
    }
}
