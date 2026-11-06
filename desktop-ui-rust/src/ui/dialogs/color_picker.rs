use iced::mouse;
use iced::widget::canvas::{self, Canvas, Geometry, Path, Stroke};
use iced::widget::{button, column, container, row, text, Space};
use iced::{Background, Border, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use crate::app::dialog::ColorPickerState;
use crate::app::{App, Message};
use crate::settings::palette::{hsv_to_rgb, Hsv};
use crate::ui::icons;

const SV_WIDTH: f32 = 260.0;
const SV_HEIGHT: f32 = 180.0;
const HUE_WIDTH: f32 = 22.0;

pub fn view<'a>(app: &'a App, state: &'a ColorPickerState) -> Element<'a, Message> {
    let interface = interface_color(app);
    let preview = state.hsv.to_theme();
    let (pr, pg, pb, pa) = preview.rgba();
    let preview_color = Color::from_rgba8(pr, pg, pb, pa as f32 / 255.0);

    let sv = Canvas::new(SvCanvas { hsv: state.hsv, dragging: state.dragging_sv })
        .width(Length::Fixed(SV_WIDTH))
        .height(Length::Fixed(SV_HEIGHT));

    let hue = Canvas::new(HueCanvas { hue: state.hsv.h, dragging: state.dragging_hue })
        .width(Length::Fixed(HUE_WIDTH))
        .height(Length::Fixed(SV_HEIGHT));

    let picker = row![sv, Space::with_width(8), hue].align_y(iced::Alignment::Start);

    let close = button(icons::close::close(18.0, Color::WHITE))
        .on_press(Message::DialogDismiss)
        .padding(4)
        .style(icon_button);

    let header = row![
        button(text("Назад").color(Color::WHITE))
            .on_press(Message::ColorPickerCancel)
            .padding([4, 8])
            .style(icon_button),
        Space::with_width(Length::Fill),
        text("Цветовая гамма").color(Color::WHITE),
        Space::with_width(Length::Fill),
        close,
    ]
    .width(Length::Fill)
    .align_y(iced::Alignment::Center);

    let hex = preview.to_hex();
    let info = row![
        container(Space::new(Length::Fixed(36.0), Length::Fixed(36.0)))
            .style(move |_theme| container::Style {
                background: Some(Background::Color(preview_color)),
                border: Border { color: Color::WHITE, width: 1.0, radius: 0.0.into() },
                ..container::Style::default()
            }),
        Space::with_width(10),
        text(hex).color(Color::WHITE),
    ]
    .align_y(iced::Alignment::Center);

    let actions = row![
        button(text("Применить").color(Color::WHITE))
            .on_press(Message::ColorPickerApply)
            .padding([6, 12])
            .style(move |_theme, _status| action_style(interface)),
        Space::with_width(8),
        button(text("Отмена").color(Color::WHITE))
            .on_press(Message::ColorPickerCancel)
            .padding([6, 12])
            .style(icon_button),
    ];

    column![header, Space::with_height(8), picker, Space::with_height(10), info, Space::with_height(10), actions]
        .spacing(2)
        .align_x(iced::Alignment::Center)
        .width(Length::Fill)
        .into()
}

#[derive(Debug, Clone, Copy)]
enum SvEvent {
    Press(Point),
    Drag(Point),
    Release,
}

struct SvCanvas {
    hsv: Hsv,
    dragging: bool,
}

impl canvas::Program<Message> for SvCanvas {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let sv_event = match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                cursor.position_in(bounds).map(SvEvent::Press)
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if self.dragging => {
                cursor.position_in(bounds).map(SvEvent::Drag)
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if self.dragging => {
                Some(SvEvent::Release)
            }
            _ => None,
        };

        match sv_event {
            Some(SvEvent::Press(pos)) | Some(SvEvent::Drag(pos)) => {
                let s = (pos.x / bounds.width).clamp(0.0, 1.0);
                let v = 1.0 - (pos.y / bounds.height).clamp(0.0, 1.0);
                (
                    canvas::event::Status::Captured,
                    Some(Message::ColorPickerSvChanged(s, v)),
                )
            }
            Some(SvEvent::Release) => (
                canvas::event::Status::Captured,
                Some(Message::ColorPickerDragEnd),
            ),
            None => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;

        let (hr, hg, hb) = hsv_to_rgb(self.hsv.h, 1.0, 1.0);
        let hue = Color::from_rgb8(hr, hg, hb);
        frame.fill_rectangle(Point::ORIGIN, Size::new(w, h), hue);

        let cols = 90usize;
        let rows = 65usize;
        for y in 0..rows {
            let v = 1.0 - y as f32 / (rows.saturating_sub(1) as f32);
            for x in 0..cols {
                let s = x as f32 / (cols.saturating_sub(1) as f32);
                let (r, g, b) = hsv_to_rgb(self.hsv.h, s, v);
                frame.fill_rectangle(
                    Point::new(w * x as f32 / cols as f32, h * y as f32 / rows as f32),
                    Size::new(w / cols as f32 + 1.0, h / rows as f32 + 1.0),
                    Color::from_rgb8(r, g, b),
                );
            }
        }

        let x = self.hsv.s.clamp(0.0, 1.0) * w;
        let y = (1.0 - self.hsv.v.clamp(0.0, 1.0)) * h;
        frame.stroke(
            &Path::circle(Point::new(x, y), 6.0),
            Stroke::default().with_color(Color::WHITE).with_width(2.0),
        );
        frame.stroke(
            &Path::circle(Point::new(x, y), 7.0),
            Stroke::default().with_color(Color::BLACK).with_width(1.0),
        );

        vec![frame.into_geometry()]
    }
}

#[derive(Debug, Clone, Copy)]
enum HueEvent {
    Press(f32),
    Drag(f32),
    Release,
}

struct HueCanvas {
    hue: f32,
    dragging: bool,
}

impl canvas::Program<Message> for HueCanvas {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let hue_event = match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                cursor.position_in(bounds).map(|p| HueEvent::Press(p.y))
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if self.dragging => {
                cursor.position_in(bounds).map(|p| HueEvent::Drag(p.y))
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if self.dragging => {
                Some(HueEvent::Release)
            }
            _ => None,
        };

        match hue_event {
            Some(HueEvent::Press(y)) | Some(HueEvent::Drag(y)) => {
                let h = (y / bounds.height).clamp(0.0, 1.0) * 360.0;
                (
                    canvas::event::Status::Captured,
                    Some(Message::ColorPickerHueChanged(h)),
                )
            }
            Some(HueEvent::Release) => (
                canvas::event::Status::Captured,
                Some(Message::ColorPickerDragEnd),
            ),
            None => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;
        let strips = 120usize;
        for i in 0..strips {
            let y0 = h * i as f32 / strips as f32;
            let y1 = h * (i + 1) as f32 / strips as f32;
            let hue = 360.0 * i as f32 / strips as f32;
            let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
            frame.fill_rectangle(
                Point::new(0.0, y0),
                Size::new(w, (y1 - y0).ceil()),
                Color::from_rgb8(r, g, b),
            );
        }

        let y = (self.hue.rem_euclid(360.0) / 360.0) * h;
        let marker = Path::rectangle(Point::new(0.0, y - 2.0), Size::new(w, 4.0));
        frame.stroke(&marker, Stroke::default().with_color(Color::WHITE).with_width(1.5));

        vec![frame.into_geometry()]
    }
}

fn icon_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => {
            Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)))
        }
        _ => None,
    };
    button::Style { background: bg, border: Border::default(), ..button::Style::default() }
}

fn action_style(interface: Color) -> button::Style {
    button::Style {
        border: Border { color: interface, width: 1.0, radius: 0.0.into() },
        background: None,
        ..button::Style::default()
    }
}

fn interface_color(app: &App) -> Color {
    let (r, g, b, a) = app.settings.theme_color.rgba();
    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}
