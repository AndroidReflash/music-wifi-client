use iced::widget::{button, column, container, scrollable, text};
use iced::{Background, Border, Color, Element, Length};

use crate::app::{App, Message};

use super::list_scrollbar::list_scrollbar;

pub fn song_list<'a>(app: &'a App, interface: Color) -> Element<'a, Message> {
    let current = app
        .snapshot
        .as_ref()
        .and_then(|s| s.track_index)
        .map(|i| i as usize)
        .or(app.selected_track);
    let items = app.visible_tracks();
    let list_content = column(items.into_iter().map(|(index, track)| {
        let selected = current == Some(index);
        let label = text(track.track_name.clone()).size(16).color(if selected {
            interface
        } else {
            Color::WHITE
        });
        button(container(label).width(Length::Fill).padding([6, 8]))
            .on_press(Message::TrackPressed(index))
            .width(Length::Fill)
            .padding(0)
            .style(move |_theme, status| row_style(selected, interface, status))
            .into()
    }))
    .spacing(0)
    .width(Length::Fill)
    .padding(iced::Padding {
        top: 0.0,
        right: 2.0,
        bottom: 0.0,
        left: 0.0,
    });

    let list = scrollable(list_content)
        .id(app.list_scrollable.clone())
        .width(Length::Fill)
        .height(Length::Fill)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(0).scroller_width(0),
        ))
        .on_scroll(Message::ListScrolled)
        .style(|_theme, _status| scrollable::Style {
            container: container::Style::default(),
            vertical_rail: hidden_rail(),
            horizontal_rail: hidden_rail(),
            gap: None,
        });

    let left = iced::widget::row![
        iced::widget::Space::with_width(Length::Fixed(8.0)),
        list_scrollbar(app.list_viewport, interface),
        iced::widget::Space::with_width(Length::Fixed(16.0)),
        container(list).width(Length::Fill).height(Length::Fill),
        iced::widget::Space::with_width(Length::Fixed(4.0)),
    ]
    .width(Length::FillPortion(1))
    .height(Length::Fill);

    left.into()
}

fn row_style(selected: bool, interface: Color, status: button::Status) -> button::Style {
    let background = if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06)))
    } else {
        None
    };
    let border = if selected {
        Border {
            color: interface,
            width: 1.0,
            radius: 0.0.into(),
        }
    } else {
        Border::default()
    };
    button::Style {
        background,
        text_color: Color::WHITE,
        border,
        ..button::Style::default()
    }
}

fn hidden_rail() -> scrollable::Rail {
    scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            color: Color::TRANSPARENT,
            border: iced::border::rounded(0),
        },
    }
}
