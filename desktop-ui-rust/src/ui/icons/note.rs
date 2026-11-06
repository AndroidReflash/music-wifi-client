use std::f32::consts::PI;

use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn note<'a>(color: Color) -> Element<'a, Message> {
    Canvas::new(Icon { color })
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

struct Icon {
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
        let side = bounds.width.min(bounds.height);
        let cx = bounds.width / 2.0;
        let cy = bounds.height / 2.0;
        let c = self.color;

        let radius = side * 0.48;
        let thin = (side * 0.012).max(1.0);
        frame.stroke(
            &Path::circle(Point::new(cx, cy), radius),
            Stroke::default().with_color(c).with_width(thin),
        );

        let n = radius * 0.95;
        let hw = n * 0.20;
        let hh = n * 0.125;
        let oval_shift = (1.0 * hw) / 9.0;
        let base_lx = cx - n * 0.28;
        let base_rx = cx + n * 0.28;
        let note_w = (base_rx - base_lx) + 2.0 * hw;
        let note_shift = note_w / 8.0;
        let lx = base_lx - note_shift;
        let rx = base_rx - note_shift;
        let hy = cy + n * 0.28;
        let ly = cy - n * 0.42;
        let ry = cy - n * 0.52;
        let stem_x_l = lx + hw;
        let stem_x_r = rx + hw;
        let beam_h = n * 0.12;

        let beam_angle = (ry - ly).atan2(stem_x_r - stem_x_l);
        let ox_l = lx + oval_shift;
        let ox_r = rx + oval_shift;

        frame.fill(&rotated_oval(ox_l, hy, hw, hh, beam_angle), c);
        frame.fill(&rotated_oval(ox_r, hy, hw, hh, beam_angle), c);

        let stroke = Stroke::default()
            .with_color(c)
            .with_width((side * 0.014).max(1.0));
        frame.stroke(&Path::line(Point::new(stem_x_l, hy), Point::new(stem_x_l, ly)), stroke);
        frame.stroke(&Path::line(Point::new(stem_x_r, hy), Point::new(stem_x_r, ry)), stroke);

        let beam = Path::new(|b| {
            b.move_to(Point::new(stem_x_l, ly));
            b.line_to(Point::new(stem_x_r, ry));
            b.line_to(Point::new(stem_x_r, ry + beam_h));
            b.line_to(Point::new(stem_x_l, ly + beam_h));
            b.close();
        });
        frame.fill(&beam, c);

        vec![frame.into_geometry()]
    }
}

fn rotated_oval(x: f32, y: f32, rx: f32, ry: f32, angle: f32) -> Path {
    Path::new(|b| {
        let steps = 28;
        let (sin_a, cos_a) = angle.sin_cos();
        for i in 0..=steps {
            let a = i as f32 * 2.0 * PI / steps as f32;
            let dx = rx * a.cos();
            let dy = ry * a.sin();
            let p = Point::new(x + dx * cos_a - dy * sin_a, y + dx * sin_a + dy * cos_a);
            if i == 0 {
                b.move_to(p);
            } else {
                b.line_to(p);
            }
        }
        b.close();
    })
}
