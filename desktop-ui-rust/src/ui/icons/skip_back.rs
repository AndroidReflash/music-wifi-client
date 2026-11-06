use iced::widget::canvas::{self, Canvas, Geometry, Path};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Size, Theme};

use crate::app::Message;

pub fn skip_back<'a>(size: f32, color: Color) -> Element<'a, Message> {
    Canvas::new(Icon { size, color }).width(size).height(size).into()
}

struct Icon { size: f32, color: Color }

impl canvas::Program<Message> for Icon {
    type State = ();
    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let s = self.size;
        let bar_w = s * 0.12;
        let x0 = s * 0.2;
        frame.fill(&Path::rectangle(Point::new(x0, s * 0.22), Size::new(bar_w, s * 0.56)), self.color);
        let tri = Path::new(|b| {
            b.move_to(Point::new(s * 0.82, s * 0.22));
            b.line_to(Point::new(x0 + bar_w + s * 0.04, s * 0.5));
            b.line_to(Point::new(s * 0.82, s * 0.78));
            b.close();
        });
        frame.fill(&tri, self.color);
        vec![frame.into_geometry()]
    }
}
