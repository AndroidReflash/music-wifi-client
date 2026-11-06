use iced::mouse;
use iced::widget::canvas::{self, Canvas, Event, Frame, Geometry, Path, Stroke};
use iced::widget::{button, column, row, text, Space};
use iced::{Alignment, Border, Color, Element, Length, Point, Rectangle, Renderer, Theme};

use crate::app::dialog::EqualizerState;
use crate::app::{App, Message};

pub fn view<'a>(app: &'a App, state: &'a EqualizerState) -> Element<'a, Message> {
    let interface = interface_color(app);
    let mut bands = row![].spacing(4).align_y(Alignment::End).width(Length::Fill);

    for (i, (hz, gain)) in app.eq_bands_hz.iter().zip(state.gains_db.iter()).enumerate() {
        let max = app.eq_max_db;
        let g = *gain;
        let col = column![
            text(db_label(g)).size(10).color(Color::WHITE),
            band_slider(i, g, max, interface),
            text(hz_label(*hz)).size(10).color(Color::WHITE),
        ]
        .spacing(4)
        .align_x(Alignment::Center)
        .width(Length::FillPortion(1));
        bands = bands.push(col);
    }

    let close = button(crate::ui::icons::close::close(18.0, Color::WHITE))
        .on_press(Message::DialogDismiss)
        .padding(4)
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)))
                }
                _ => None,
            };
            button::Style { background: bg, border: Border::default(), ..button::Style::default() }
        });
    let header = row![Space::with_width(Length::Fill), close]
        .width(Length::Fill)
        .align_y(Alignment::Center);

    column![
        header,
        text("Эквалайзер").color(Color::WHITE),
        Space::with_height(4),
        bands,
        Space::with_height(8),
        button(text("Сбросить").color(Color::WHITE))
            .on_press(Message::EqualizerResetPressed)
            .padding([4, 12])
            .style(|_theme, _status| button::Style {
                border: Border { color: Color::WHITE, width: 1.0, radius: 0.0.into() },
                background: None,
                ..button::Style::default()
            }),
        text("применяется к плееру сразу (если плеер готов)").size(11).color(Color::from_rgb(0.6, 0.6, 0.6)),
    ]
    .spacing(6)
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn band_slider<'a>(index: usize, gain: f32, max_db: f32, interface: Color) -> Element<'a, Message> {
    Canvas::new(Band { index, gain, max_db, interface })
        .width(Length::Fixed(28.0))
        .height(Length::Fixed(200.0))
        .into()
}

struct Band {
    index: usize,
    gain: f32,
    max_db: f32,
    interface: Color,
}

#[derive(Default)]
struct BandState {
    dragging: bool,
}

impl canvas::Program<Message> for Band {
    type State = BandState;

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let gain_at = |pos_y: f32| {
            let share = ((pos_y - bounds.y) / bounds.height).clamp(0.0, 1.0);
            (self.max_db - share * 2.0 * self.max_db).round()
        };
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position() {
                    if bounds.contains(pos) {
                        state.dragging = true;
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::EqualizerBandChanged(self.index, gain_at(pos.y))),
                        );
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.dragging {
                    if let Some(pos) = cursor.position() {
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::EqualizerBandChanged(self.index, gain_at(pos.y))),
                        );
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.dragging {
                    state.dragging = false;
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::EqualizerBandReleased(self.index)),
                    );
                }
            }
            _ => {}
        }
        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let x = bounds.width / 2.0;
        let mid = bounds.height / 2.0;
        let thumb = bounds.height * (self.max_db - self.gain) / (2.0 * self.max_db);
        let line = Stroke::default().with_color(Color::from_rgba(0.6, 0.6, 0.6, 0.5)).with_width(2.0);
        frame.stroke(&Path::line(Point::new(x, 0.0), Point::new(x, bounds.height)), line);
        let zero = Stroke::default().with_color(Color::from_rgb(0.6, 0.6, 0.6)).with_width(2.0);
        frame.stroke(&Path::line(Point::new(x - 6.0, mid), Point::new(x + 6.0, mid)), zero);
        let active = Stroke::default().with_color(self.interface).with_width(2.0);
        frame.stroke(&Path::line(Point::new(x, mid), Point::new(x, thumb)), active);
        frame.fill(&Path::circle(Point::new(x, thumb), 6.0), self.interface);
        vec![frame.into_geometry()]
    }
}

fn db_label(gain: f32) -> String {
    let whole = gain.round() as i32;
    match whole {
        g if g > 0 => format!("+{g}"),
        g if g < 0 => format!("-{}", -g),
        _ => "0".to_owned(),
    }
}

fn hz_label(hz: f32) -> String {
    if hz >= 1000.0 {
        format!("{}k", (hz / 1000.0).round() as i32)
    } else {
        format!("{}", hz as i32)
    }
}

fn interface_color(app: &App) -> Color {
    let (r, g, b, a) = app.settings.theme_color.rgba();
    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}
