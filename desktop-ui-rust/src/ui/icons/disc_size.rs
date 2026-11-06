use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn disc_size<'a>(size: f32, color: Color, small: bool) -> Element<'a, Message> {
    Canvas::new(Icon { size, color, small }).width(size).height(size).into()
}

struct Icon { size: f32, color: Color, small: bool }

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
        let radius = if self.small { 0.20 } else { 0.32 };
        let middle = Point::new(s / 2.0 - s * 0.12, s / 2.0);
        frame.stroke(
            &Path::circle(middle, s * radius),
            Stroke::default().with_color(self.color).with_width((s * 0.07).max(1.0)),
        );
        frame.fill(&Path::circle(middle, s * 0.06), self.color);

        let x = s * 0.82;
        let top = s * 0.22;
        let bottom = s * 0.78;
        let wing = s * 0.12;
        let (tip, toward) = if self.small { (top, wing) } else { (bottom, -wing) };
        let stroke = Stroke::default().with_color(self.color).with_width((s * 0.07).max(1.0)).with_line_cap(canvas::LineCap::Round);
        frame.stroke(&Path::line(Point::new(x, top), Point::new(x, bottom)), stroke);
        frame.stroke(&Path::line(Point::new(x - wing, tip + toward), Point::new(x, tip)), stroke);
        frame.stroke(&Path::line(Point::new(x + wing, tip + toward), Point::new(x, tip)), stroke);
        vec![frame.into_geometry()]
    }
}
