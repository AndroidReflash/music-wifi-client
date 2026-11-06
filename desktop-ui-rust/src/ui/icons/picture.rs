use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Size, Theme};

use crate::app::Message;

pub fn picture<'a>(size: f32, color: Color) -> Element<'a, Message> {
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
        let thin = (s * 0.07).max(1.0);
        let stroke = Stroke::default().with_color(self.color).with_width(thin);
        let left = s * 0.14;
        let right = s * 0.86;
        let top = s * 0.20;
        let bottom = s * 0.80;
        frame.stroke(
            &Path::rectangle(Point::new(left, top), Size::new(right - left, bottom - top)),
            stroke,
        );
        frame.fill(&Path::circle(Point::new(s * 0.34, s * 0.36), s * 0.07), self.color);
        let mountain = Path::new(|b| {
            b.move_to(Point::new(left + thin, bottom - thin));
            b.line_to(Point::new(s * 0.52, s * 0.44));
            b.line_to(Point::new(right - thin, bottom - thin));
        });
        frame.stroke(&mountain, Stroke::default().with_color(self.color).with_width(thin).with_line_cap(canvas::LineCap::Round).with_line_join(canvas::LineJoin::Round));
        vec![frame.into_geometry()]
    }
}
