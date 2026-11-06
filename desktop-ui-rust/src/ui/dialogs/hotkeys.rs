use iced::widget::{button, column, text, Space};
use iced::{Alignment, Border, Color, Element, Length};

use crate::app::dialog::HotkeysCaptureState;
use crate::app::{App, Message};
use crate::settings::HotkeyAction;

pub fn view<'a>(app: &'a App, state: &'a HotkeysCaptureState) -> Element<'a, Message> {
    let current = app
        .settings
        .hotkeys
        .get(&state.action)
        .map(|b| b.to_string())
        .unwrap_or_else(|| "не назначена".to_owned());

    column![
        text("Нажмите любую клавишу/сочетание клавиш").color(Color::WHITE),
        text(format!("Действие: {}", label(state.action))).color(Color::WHITE),
        text(format!("Сейчас: {current}")).color(interface_color(app)),
        text("Esc — отмена, любая другая — назначить").size(12).color(Color::from_rgb(0.7, 0.7, 0.7)),
        Space::with_height(8),
        button(text("Убрать").color(Color::WHITE))
            .on_press(Message::HotkeyCaptured(None))
            .padding([5, 12])
            .style(move |_theme, _status| button::Style {
                border: Border { color: interface_color(app), width: 1.0, radius: 0.0.into() },
                background: None,
                ..button::Style::default()
            }),
        button(text("Отмена").color(Color::WHITE))
            .on_press(Message::DialogDismiss)
            .padding([5, 12]),
    ]
    .spacing(8)
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn label(action: HotkeyAction) -> &'static str {
    match action {
        HotkeyAction::Shuffle => "Перемешать",
        HotkeyAction::Previous => "Предыдущий",
        HotkeyAction::PlayPause => "Воспроизведение",
        HotkeyAction::Next => "Следующий",
        HotkeyAction::Repeat => "Повторить",
        HotkeyAction::Rewind => "Назад на 5 секунд",
        HotkeyAction::Forward => "Вперёд на 5 секунд",
        HotkeyAction::VolumeDown => "Тише",
        HotkeyAction::VolumeUp => "Громче",
    }
}

fn interface_color(app: &App) -> Color {
    let (r, g, b, a) = app.settings.theme_color.rgba();
    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}
