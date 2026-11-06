use iced::widget::{button, container, row, text, text_input, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::app::{App, Message};
use crate::ui::icons;

pub fn top_bar<'a>(app: &'a App, interface: Color) -> Element<'a, Message> {
    if app.search_active {
        search_bar(app)
    } else {
        status_bar(app, interface)
    }
}

fn search_bar<'a>(app: &'a App) -> Element<'a, Message> {
    let input = text_input("Поиск треков...", &app.search_query)
        .on_input(Message::SearchQueryChanged)
        .padding(8)
        .size(18);
    let close = button(icons::close::close(24.0, Color::WHITE))
        .on_press(Message::SearchToggled)
        .padding(4)
        .style(icon_button);
    container(
        row![
            icons::search::search(24.0, Color::WHITE),
            Space::with_width(12),
            input,
            Space::with_width(8),
            close,
        ]
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(60.0))
    .padding([0, 10])
    .center_y(Length::Fixed(60.0))
    .into()
}

fn status_bar<'a>(app: &'a App, interface: Color) -> Element<'a, Message> {
    let count = app.tracks.len();
    let current_index = app
        .snapshot
        .as_ref()
        .and_then(|s| s.track_index)
        .map(|i| i as usize)
        .or(app.selected_track);
    let current = current_index
        .map(|i| i + 1)
        .map(|n| format!("{n} / {count}"))
        .unwrap_or_else(|| format!("{count}"));

    let left = row![
        button(icons::search::search(24.0, Color::WHITE))
            .on_press(Message::SearchToggled)
            .padding(4)
            .style(icon_button),
    ]
    .align_y(Alignment::Center)
    .width(Length::FillPortion(1));

    let center = container(
        row![
            button(icons::menu::menu(28.0, Color::WHITE))
                .on_press(Message::MenuPressed)
                .padding(4)
                .style(icon_button),
            Space::with_width(10),
            button(icons::equalizer::equalizer(28.0, Color::WHITE))
                .on_press(Message::EqualizerPressed)
                .padding(4)
                .style(icon_button),
        ]
        .align_y(Alignment::Center)
        .spacing(0),
    )
    .width(Length::FillPortion(1))
    .center_x(Length::Fill)
    .center_y(Length::Fill);

    let right = container(text(current).size(20).color(Color::WHITE))
        .width(Length::FillPortion(1))
        .align_x(iced::alignment::Horizontal::Right)
        .center_y(Length::Fill);

    let _ = interface;
    container(
        row![left, center, right]
            .height(Length::Fixed(60.0))
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(60.0))
    .padding([0, 10])
    .into()
}

fn icon_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => {
            Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)))
        }
        _ => None,
    };
    button::Style {
        background: bg,
        text_color: Color::WHITE,
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}
