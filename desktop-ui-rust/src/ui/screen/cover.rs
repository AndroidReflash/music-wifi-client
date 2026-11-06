use std::f32::consts::PI;

use iced::mouse;
use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn view<'a>(color: Color, small: bool, spin_deg: f32) -> Element<'a, Message> {
    Canvas::new(Cover {
        color,
        small,
        spin_deg,
    })
    .width(iced::Length::Fill)
    .height(iced::Length::Fill)
    .into()
}

struct Cover {
    color: Color,
    small: bool,
    spin_deg: f32,
}

impl canvas::Program<Message> for Cover {
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
        let share = if self.small { 0.45 } else { 1.0 };
        let side = bounds.width.min(bounds.height) * share;
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let thin = 1.0;
        let stroke = Stroke::default().with_color(self.color).with_width(thin);

        frame.stroke(&Path::circle(center, side * 0.48), Stroke::default().with_color(self.color).with_width(thin * 1.5));
        frame.stroke(&Path::circle(center, side * 0.16), Stroke::default().with_color(self.color).with_width(thin * 1.5));
        frame.stroke(&Path::circle(center, side * 0.03), stroke);

        let turned = self.spin_deg;
        for share in [0.28_f32, 0.34, 0.40] {
            let radius = side * share;
            draw_arc(&mut frame, center, radius, turned + 225.0 - 22.5, 45.0, stroke);
            draw_arc(&mut frame, center, radius, turned + 45.0 - 22.5, 45.0, stroke);
        }

        vec![frame.into_geometry()]
    }
}

fn draw_arc(frame: &mut canvas::Frame, center: Point, radius: f32, start_deg: f32, sweep_deg: f32, stroke: Stroke) {
    let steps = 24;
    let path = Path::new(|b| {
        for i in 0..=steps {
            let a = (start_deg + sweep_deg * i as f32 / steps as f32) * PI / 180.0;
            let p = Point::new(center.x + radius * a.cos(), center.y + radius * a.sin());
            if i == 0 {
                b.move_to(p);
            } else {
                b.line_to(p);
            }
        }
    });
    frame.stroke(&path, stroke);
}
