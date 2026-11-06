use iced::widget::canvas::{self, Canvas, Geometry, Path};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn play<'a>(size: f32, color: Color) -> Element<'a, Message> {
    Canvas::new(Icon { size, color })
        .width(size)
        .height(size)
        .into()
}

struct Icon {
    size: f32,
    color: Color,
}

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
        let p = s * 0.22;
        let path = Path::new(|b| {
            b.move_to(Point::new(p, p));
            b.line_to(Point::new(s - p, s * 0.5));
            b.line_to(Point::new(p, s - p));
            b.close();
        });
        frame.fill(&path, self.color);
        vec![frame.into_geometry()]
    }
}
