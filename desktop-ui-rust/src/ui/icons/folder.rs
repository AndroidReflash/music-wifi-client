use iced::widget::canvas::{self, Canvas, Geometry, Path};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Size, Theme};

use crate::app::Message;

pub fn folder<'a>(size: f32, color: Color) -> Element<'a, Message> {
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
        let tab_w = s * 0.38;
        let tab_h = s * 0.18;
        let body_y = s * 0.28;
        let body_h = s - body_y - s * 0.12;

        let tab = Path::rectangle(Point::new(s * 0.12, body_y - tab_h + 2.0), Size::new(tab_w, tab_h));
        frame.fill(&tab, self.color);
        let body = Path::rectangle(Point::new(s * 0.12, body_y), Size::new(s * 0.76, body_h));
        frame.fill(&body, self.color);

        vec![frame.into_geometry()]
    }
}
