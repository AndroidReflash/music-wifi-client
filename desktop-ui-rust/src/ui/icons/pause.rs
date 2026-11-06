use iced::widget::canvas::{self, Canvas, Geometry, Path};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Size, Theme};

use crate::app::Message;

pub fn pause<'a>(size: f32, color: Color) -> Element<'a, Message> {
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
        let w = s * 0.18;
        let gap = s * 0.12;
        let x1 = s * 0.5 - gap - w;
        let x2 = s * 0.5 + gap;
        let y = s * 0.22;
        let h = s - y * 2.0;
        for x in [x1, x2] {
            frame.fill(&Path::rectangle(Point::new(x, y), Size::new(w, h)), self.color);
        }
        vec![frame.into_geometry()]
    }
}
