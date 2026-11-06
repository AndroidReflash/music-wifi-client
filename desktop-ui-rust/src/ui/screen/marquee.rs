use iced::alignment;
use iced::mouse;
use iced::widget::canvas::{self, Canvas, Geometry, Text};
use iced::widget::{container, mouse_area};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

use crate::app::{App, Message};

pub fn marquee<'a>(app: &'a App) -> Element<'a, Message> {
    let title = app.current_title();
    let scroll = app.marquee_scroll;
    mouse_area(
        container(
            Canvas::new(Marquee { title, scroll })
                .width(Length::Fill)
                .height(Length::Fixed(30.0)),
        )
        .width(Length::Fill)
        .height(Length::Fixed(30.0))
        .clip(true),
    )
    .on_press(Message::TitlePressed)
    .into()
}

struct Marquee {
    title: String,
    scroll: f32,
}

impl canvas::Program<Message> for Marquee {
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
        if self.title.is_empty() {
            return vec![frame.into_geometry()];
        }
        let size = 18.0;
        let approx_width = self.title.chars().count() as f32 * size * 0.58;
        let view_w = bounds.width;
        let speed = 25.0;
        let pause = 2.0;
        let y = bounds.height / 2.0;

        let fits = approx_width <= view_w || view_w <= 1.0;
        let (x, align) = if fits {
            (view_w / 2.0, alignment::Horizontal::Center)
        } else {
            let max_scroll = (approx_width - view_w).max(0.0);
            let dur = (max_scroll / speed).max(0.001);
            let period = pause + dur + pause + dur;
            let t = self.scroll.rem_euclid(period);
            let offset = if t < pause {
                0.0
            } else if t < pause + dur {
                let p = (t - pause) / dur;
                let e = 1.0 - (1.0 - p) * (1.0 - p);
                -e * max_scroll
            } else if t < pause + dur + pause {
                -max_scroll
            } else {
                let p = (t - (pause + dur + pause)) / dur;
                let e = 1.0 - (1.0 - p) * (1.0 - p);
                -max_scroll + e * max_scroll
            };
            (offset, alignment::Horizontal::Left)
        };

        let base = Point::new(x, y);
        let outline = Text {
            content: self.title.clone(),
            position: base,
            color: Color::BLACK,
            size: iced::Pixels(size),
            horizontal_alignment: align,
            vertical_alignment: alignment::Vertical::Center,
            ..Text::default()
        };
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
            let mut t = outline.clone();
            t.position = Point::new(base.x + dx, base.y + dy);
            frame.fill_text(t);
        }
        frame.fill_text(Text {
            content: self.title.clone(),
            position: base,
            color: Color::WHITE,
            size: iced::Pixels(size),
            horizontal_alignment: align,
            vertical_alignment: alignment::Vertical::Center,
            ..Text::default()
        });
        vec![frame.into_geometry()]
    }
}
