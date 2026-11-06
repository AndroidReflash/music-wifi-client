use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn disc<'a>(size: f32, color: Color, radius: f32, shift: f32) -> Element<'a, Message> {
    Canvas::new(Icon { size, color, radius, shift }).width(size).height(size).into()
}

struct Icon { size: f32, color: Color, radius: f32, shift: f32 }

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
        let middle = Point::new(s / 2.0 + s * self.shift, s / 2.0);
        frame.stroke(
            &Path::circle(middle, s * self.radius),
            Stroke::default().with_color(self.color).with_width((s * 0.07).max(1.0)),
        );
        frame.fill(&Path::circle(middle, s * 0.06), self.color);
        vec![frame.into_geometry()]
    }
}
