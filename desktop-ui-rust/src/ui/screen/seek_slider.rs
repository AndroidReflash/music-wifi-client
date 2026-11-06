use iced::widget::{container, row, slider, text, Space};
use iced::{Alignment, Border, Color, Element, Length};

use crate::app::{App, Message};

pub fn seek_slider<'a>(app: &'a App, interface: Color) -> Element<'a, Message> {
    let (pos, dur) = app.time_labels();
    let value = app.progress();

    let bar = slider(0.0..=1.0, value, Message::SeekChanged)
        .on_release(Message::SeekReleased)
        .step(0.001_f32)
        .width(Length::Fill)
        .style(move |_theme, status| slider_style(interface, status));

    row![
        container(text(pos).size(12).color(Color { a: 0.7, ..Color::WHITE })).width(Length::Fixed(42.0)),
        Space::with_width(8),
        bar,
        Space::with_width(8),
        container(text(dur).size(12).color(Color { a: 0.7, ..Color::WHITE }))
            .width(Length::Fixed(42.0))
            .align_x(iced::alignment::Horizontal::Right),
    ]
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .padding([8, 0])
    .into()
}

fn slider_style(interface: Color, status: slider::Status) -> slider::Style {
    let handle = match status {
        slider::Status::Dragged | slider::Status::Hovered => slider::Handle {
            shape: slider::HandleShape::Circle { radius: 6.0 },
            background: iced::Background::Color(interface),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
        _ => slider::Handle {
            shape: slider::HandleShape::Circle { radius: 5.0 },
            background: iced::Background::Color(interface),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
    };
    slider::Style {
        rail: slider::Rail {
            backgrounds: (
                iced::Background::Color(interface),
                iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.25)),
            ),
            width: 2.0,
            border: Border {
                radius: 2.0.into(),
                ..Border::default()
            },
        },
        handle,
    }
}
