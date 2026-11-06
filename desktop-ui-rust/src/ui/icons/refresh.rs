use iced::widget::canvas::{self, path, Canvas, Geometry, Path, Stroke};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn refresh<'a>(size: f32, color: Color) -> Element<'a, Message> {
    Canvas::new(Icon { size, color }).width(size).height(size).into()
}

struct Icon { size: f32, color: Color }

impl canvas::Program<Message> for Icon {
    type State = ();
    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let s = self.size;
        let c = Point::new(s * 0.5, s * 0.5);
        let r = s * 0.28;
        let w = Stroke::default().with_color(self.color).with_width((s * 0.1).max(1.2));
        let arc = Path::new(|b| {
            b.arc(path::Arc {
                center: c,
                radius: r,
                start_angle: iced::Radians(-0.6),
                end_angle: iced::Radians(std::f32::consts::PI * 1.55),
            });
        });
        frame.stroke(&arc, w);
        let tip = Point::new(c.x + r, c.y - r * 0.15);
        let ah = s * 0.12;
        frame.stroke(&Path::line(tip, Point::new(tip.x - ah, tip.y - ah * 0.4)), w);
        frame.stroke(&Path::line(tip, Point::new(tip.x - ah * 0.2, tip.y + ah)), w);
        vec![frame.into_geometry()]
    }
}
