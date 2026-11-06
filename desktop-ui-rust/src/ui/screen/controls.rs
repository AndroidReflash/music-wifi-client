use iced::widget::{button, container, row};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::app::{App, Message};
use crate::ui::icons;

pub fn controls<'a>(app: &'a App, interface: Color) -> Element<'a, Message> {
    let shuffle = app.settings.shuffle;
    let repeat = app.settings.repeat;
    let playing = app
        .snapshot
        .as_ref()
        .map(|s| s.state == player_core::PlayerState::Playing)
        .unwrap_or(false);

    let shuffle_color = if shuffle { interface } else { Color::WHITE };
    let repeat_color = if repeat { interface } else { Color::WHITE };
    let play_icon = if playing {
        icons::pause::pause(40.0, Color::WHITE)
    } else {
        icons::play::play(40.0, Color::WHITE)
    };

    row![
        cell(icon_btn(icons::shuffle::shuffle(20.0, shuffle_color), Message::ShufflePressed)),
        cell(icon_btn(icons::skip_back::skip_back(25.0, Color::WHITE), Message::PreviousPressed)),
        cell(icon_btn(play_icon, Message::PlayPausePressed)),
        cell(icon_btn(icons::skip_forward::skip_forward(25.0, Color::WHITE), Message::NextPressed)),
        cell(icon_btn(icons::repeat::repeat(20.0, repeat_color), Message::RepeatPressed)),
    ]
    .width(Length::Fill)
    .height(Length::Fixed(60.0))
    .align_y(Alignment::Center)
    .padding([0, 20])
    .into()
}

fn icon_btn<'a>(content: Element<'a, Message>, message: Message) -> Element<'a, Message> {
    button(content)
        .on_press(message)
        .padding(8)
        .style(|_theme, status| button::Style {
            background: match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06)))
                }
                _ => None,
            },
            border: Border {
                radius: 6.0.into(),
                ..Border::default()
            },
            ..button::Style::default()
        })
        .into()
}

fn cell<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    container(content)
        .width(Length::FillPortion(1))
        .height(Length::Fill)
        .center_x(Length::FillPortion(1))
        .center_y(Length::Fill)
        .into()
}
