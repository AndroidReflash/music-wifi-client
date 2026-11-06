use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::{mouse, Color, Element, Point, Rectangle, Renderer, Theme};

use crate::app::Message;

pub fn repeat<'a>(size: f32, color: Color) -> Element<'a, Message> {
    Canvas::new(Icon { size, color }).width(size).height(size).into()
}

struct Icon { size: f32, color: Color }

impl canvas::Program<Message> for Icon {
    type State = ();
    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let s = self.size;
        let w = Stroke::default().with_color(self.color).with_width((s * 0.08).max(1.0));
        let l = s * 0.22; let r = s * 0.78; let t = s * 0.32; let b = s * 0.68;
        frame.stroke(&Path::new(|p| { p.move_to(Point::new(r-s*0.1,t)); p.line_to(Point::new(l,t)); p.line_to(Point::new(l,b-s*0.1)); }), w);
        frame.stroke(&Path::new(|p| { p.move_to(Point::new(l+s*0.1,b)); p.line_to(Point::new(r,b)); p.line_to(Point::new(r,t+s*0.1)); }), w);
        frame.fill(&Path::new(|p| { p.move_to(Point::new(r-s*0.18,t-s*0.08)); p.line_to(Point::new(r-s*0.02,t)); p.line_to(Point::new(r-s*0.18,t+s*0.08)); p.close(); }), self.color);
        frame.fill(&Path::new(|p| { p.move_to(Point::new(l+s*0.18,b-s*0.08)); p.line_to(Point::new(l+s*0.02,b)); p.line_to(Point::new(l+s*0.18,b+s*0.08)); p.close(); }), self.color);
        vec![frame.into_geometry()]
    }
}
