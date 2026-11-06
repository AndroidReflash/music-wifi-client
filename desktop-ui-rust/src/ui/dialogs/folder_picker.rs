use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::app::dialog::FolderPickerState;
use crate::app::{App, Message};

pub fn view<'a>(_app: &'a App, state: &'a FolderPickerState) -> Element<'a, Message> {
    let mut list = column![].spacing(2).width(Length::Fill);
    if state.current.parent().is_some() {
        list = list.push(folder_button(".. (Назад)".to_owned(), false, Message::FolderPickerUp));
    }
    if state.loading {
        list = list.push(text("загрузка...").color(Color::WHITE));
    }
    if let Some(err) = &state.error {
        list = list.push(text(err).size(12).color(Color::from_rgb(1.0, 0.5, 0.5)));
    }
    for path in &state.entries {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_owned();
        let p = path.clone();
        list = list.push(folder_entry(name, Message::FolderPickerEnter(p)));
    }

    column![
        text("Выбери папку с музыкой").color(Color::WHITE),
        text(format!("Текущий путь: {}", state.current.display())).size(12).color(Color::WHITE),
        Space::with_height(4),
        container(scrollable(list).height(Length::Fixed(300.0))).width(Length::Fill),
        row![
            button(text("Применить").color(Color::WHITE))
                .on_press(Message::FolderPickerApply)
                .padding([6, 12])
                .style(|_theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => {
                            Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)))
                        }
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        border: Border {
                            color: Color::WHITE,
                            width: 1.0,
                            radius: 0.0.into(),
                        },
                        text_color: Color::WHITE,
                        ..button::Style::default()
                    }
                }),
            Space::with_width(Length::Fill),
            button(text("Отмена").color(Color::WHITE))
                .on_press(Message::DialogDismiss)
                .padding([6, 12])
                .style(|_theme, status| {
                    let bg = match status {
                        button::Status::Hovered | button::Status::Pressed => {
                            Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08)))
                        }
                        _ => None,
                    };
                    button::Style {
                        background: bg,
                        border: Border::default(),
                        text_color: Color::WHITE,
                        ..button::Style::default()
                    }
                }),
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center),
    ]
    .spacing(6)
    .width(Length::Fill)
    .into()
}

fn folder_button(label: String, _selected: bool, msg: Message) -> Element<'static, Message> {
    button(text(label).color(Color::WHITE).size(14))
        .on_press(msg)
        .width(Length::Fill)
        .padding([10, 8])
        .style(|_theme, status| {
            let bg = matches!(status, button::Status::Hovered).then(|| Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06)));
            button::Style { background: bg, border: Border::default(), ..button::Style::default() }
        })
        .into()
}

fn folder_entry(name: String, msg: Message) -> Element<'static, Message> {
    let content = row![
        crate::ui::icons::folder::folder(18.0, Color::WHITE),
        Space::with_width(8),
        text(name).color(Color::WHITE).size(14),
    ]
    .align_y(Alignment::Center);

    button(content)
        .on_press(msg)
        .width(Length::Fill)
        .padding([10, 8])
        .style(|_theme, status| {
            let bg = matches!(status, button::Status::Hovered)
                .then(|| Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06)));
            button::Style {
                background: bg,
                border: Border::default(),
                ..button::Style::default()
            }
        })
        .into()
}
