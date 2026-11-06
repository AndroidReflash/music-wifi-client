use std::f32::consts::PI;

use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn rays<'a>(size: f32, color: Color) -> Element<'a, Message> {
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
        let middle = Point::new(s / 2.0, s / 2.0);
        let stroke = Stroke::default().with_color(self.color).with_width((s * 0.07).max(1.0)).with_line_cap(canvas::LineCap::Round);
        for index in 0..12 {
            let radians = index as f32 * (360.0 / 12.0) * PI / 180.0;
            let direction = Point::new(radians.cos(), radians.sin());
            let outer = if index % 2 == 0 { 0.46 } else { 0.34 };
            frame.stroke(
                &Path::line(
                    Point::new(middle.x + direction.x * s * 0.18, middle.y + direction.y * s * 0.18),
                    Point::new(middle.x + direction.x * s * outer, middle.y + direction.y * s * outer),
                ),
                stroke,
            );
        }
        vec![frame.into_geometry()]
    }
}
