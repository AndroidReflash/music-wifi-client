use iced::widget::{button, column, container, row, scrollable, slider, text, text_input, toggler, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::app::dialog::ChangeIpState;
use crate::app::{App, Message};
use crate::ui::icons;

pub fn view<'a>(app: &'a App, state: &'a ChangeIpState) -> Element<'a, Message> {
    let interface = interface_color(app);

    let mut body = column![
        hotkeys_section(app, interface),
        Space::with_height(4),
        action_button("Выбрать цветовую гамму", interface, Message::ColorPickerOpen),
        Space::with_height(4),
        switch_row(if app.settings.is_local { "Локально" } else { "По сети" }, app.settings.is_local, interface, |_| Message::SwitchLocalToggled),
        switch_row("Автоплей", app.settings.autoplay, interface, Message::AutoplayToggled),
        glyphs_row(app, interface),
        sound_section(app, interface),
    ]
    .spacing(6)
    .align_x(Alignment::Center)
    .width(Length::Fill);

    if !app.settings.is_local {
        body = body.push(text(format!("Текущий айпи: {}", app.settings.server_address)).color(Color::WHITE));
        body = body.push(text("Введите айпи сервера").color(Color::WHITE));
        body = body.push(text("(пример: 192.168.100.9:9090)").color(Color::WHITE));
        body = body.push(
            text_input("", &state.ip_input)
                .on_input(Message::IpInputChanged)
                .padding(8),
        );
        body = body.push(action_button("Применить", interface, Message::ApplyServerPressed));
    } else {
        body = body.push(text(" ").color(Color::WHITE));
        body = body.push(text(format!("Текущая папка:\n {}", app.settings.folder_address)).color(Color::WHITE));
        body = body.push(action_button("Выбрать папку с треками", interface, Message::FolderPickerOpen));
    }

    let close = button(icons::close::close(18.0, Color::WHITE))
        .on_press(Message::DialogDismiss)
        .padding(4)
        .style(icon_button);
    let header = row![Space::with_width(Length::Fill), close]
        .width(Length::Fill)
        .align_y(Alignment::Center);

    column![
        header,
        scrollable(body.padding(iced::Padding { top: 4.0, right: 20.0, bottom: 4.0, left: 4.0 }))
            .height(Length::Fill)
    ]
    .spacing(2)
    .width(Length::Fill)
    .into()
}

fn hotkeys_section<'a>(app: &'a App, interface: Color) -> Element<'a, Message> {
    use crate::settings::HotkeyAction;
    let mut r = row![].spacing(2).align_y(Alignment::Center);
    for action in HotkeyAction::ALL {
        let assigned = app.settings.hotkeys.contains_key(&action);
        let color = if assigned { interface } else { Color::from_rgb(0.5, 0.5, 0.5) };
        let icon = match action {
            HotkeyAction::Shuffle => icons::shuffle::shuffle(22.0, color),
            HotkeyAction::Previous => icons::skip_back::skip_back(22.0, color),
            HotkeyAction::PlayPause => icons::play::play(22.0, color),
            HotkeyAction::Next => icons::skip_forward::skip_forward(22.0, color),
            HotkeyAction::Repeat => icons::repeat::repeat(22.0, color),
            _ => text(label(action)).size(10).color(color).into(),
        };
        r = r.push(
            button(icon)
                .on_press(Message::HotkeyCaptureOpen(action))
                .padding(4)
                .style(icon_button),
        );
    }
    column![
        text("Горячие клавиши").color(Color::WHITE),
        r,
        switch_row("Вне окна (глобальные X11)", app.settings.hotkeys_global, interface, Message::HotkeyGlobalToggled),
        text(global_hotkeys_status(app)).size(11).color(Color::from_rgb(0.6, 0.6, 0.6)),
    ]
    .spacing(4)
    .align_x(Alignment::Center)
    .into()
}

fn label(action: crate::settings::HotkeyAction) -> &'static str {
    use crate::settings::HotkeyAction;
    match action {
        HotkeyAction::Shuffle => "shuffle",
        HotkeyAction::Previous => "prev",
        HotkeyAction::PlayPause => "play",
        HotkeyAction::Next => "next",
        HotkeyAction::Repeat => "repeat",
        HotkeyAction::Rewind => "-5s",
        HotkeyAction::Forward => "+5s",
        HotkeyAction::VolumeDown => "vol-",
        HotkeyAction::VolumeUp => "vol+",
    }
}

fn global_hotkeys_status(app: &App) -> String {
    use crate::platform::hotkeys::x11::GlobalHotkeysState;
    match &app.global_hotkeys_state {
        GlobalHotkeysState::Off => "глобальные: выключено".to_owned(),
        GlobalHotkeysState::Unavailable(reason) => format!("глобальные: недоступно — {reason}"),
        GlobalHotkeysState::Active { rejected } if rejected.is_empty() => {
            "глобальные: активно".to_owned()
        }
        GlobalHotkeysState::Active { rejected } => {
            let names = rejected.iter().map(|a| label(*a)).collect::<Vec<_>>().join(", ");
            format!("глобальные: активно; не захвачены: {names}")
        }
    }
}

fn glyphs_row<'a>(app: &'a App, interface: Color) -> Element<'a, Message> {
    let off = Color::from_rgb(0.5, 0.5, 0.5);
    row![
        toggle_glyph(
            icons::bars::bars(32.0, if app.settings.show_visualizer { interface } else { off }),
            Message::ShowVisualizerToggled(!app.settings.show_visualizer),
        ),
        toggle_glyph(
            icons::disc::disc(32.0, if app.settings.show_cover { interface } else { off }, 0.40, 0.0),
            Message::ShowCoverToggled(!app.settings.show_cover),
        ),
        toggle_glyph(
            icons::picture::picture(32.0, if app.settings.show_cover_image { interface } else { off }),
            Message::ShowCoverImageToggled(!app.settings.show_cover_image),
        ),
        toggle_glyph(
            icons::rays::rays(
                32.0,
                if app.settings.visualizer_ring { interface } else { off },
            ),
            Message::VisualizerRingToggled(!app.settings.visualizer_ring),
        ),
        toggle_glyph(
            icons::disc_size::disc_size(32.0, interface, app.settings.cover_small),
            Message::CoverSmallToggled(!app.settings.cover_small),
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn toggle_glyph<'a>(icon: Element<'a, Message>, message: Message) -> Element<'a, Message> {
    button(icon)
        .on_press(message)
        .padding(2)
        .style(icon_button)
        .into()
}

fn sound_section<'a>(app: &'a App, interface: Color) -> Element<'a, Message> {
    let mut c = column![].spacing(4).width(Length::Fill);

    let header = row![
        text("Вывод звука").color(Color::WHITE),
        button(icons::refresh::refresh(18.0, Color::WHITE))
            .on_press(Message::SoundRefreshRequested)
            .padding(4)
            .style(icon_button),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    c = c.push(
        container(header)
            .width(Length::Fill)
            .center_x(Length::Fill),
    );

    let selected = app.settings.output_device.clone();
    let mut list = column![device_row(
        "Системное".into(),
        selected.is_empty(),
        interface,
        String::new()
    )]
    .spacing(2);
    for d in &app.output_devices {
        list = list.push(device_row(d.clone(), *d == selected, interface, d.clone()));
    }
    c = c.push(container(scrollable(list).height(Length::Fixed(100.0))).width(Length::Fill));
    if let Some(err) = &app.sound_error {
        c = c.push(text(err).size(11).color(Color::from_rgb(1.0, 0.5, 0.5)));
    }
    c = c.push(text("Громкость").color(Color::WHITE));
    c = c.push(
        slider(0.0..=1.0, app.effective_volume(), Message::VolumeChanged)
            .on_release(Message::VolumeReleased)
            .step(0.01_f32),
    );
    c.into()
}

fn device_row(label_text: String, selected: bool, interface: Color, value: String) -> Element<'static, Message> {
    let color = if selected { interface } else { Color::WHITE };
    button(
        container(text(label_text).color(color).size(13))
            .width(Length::Fill)
            .center_x(Length::Fill),
    )
        .on_press(Message::OutputDevicePicked(value))
        .width(Length::Fill)
        .padding(4)
        .style(move |_theme, status| {
            let bg = if matches!(status, button::Status::Hovered | button::Status::Pressed) {
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
            button::Style { background: bg, text_color: Color::WHITE, border, ..button::Style::default() }
        })
        .into()
}

fn switch_row<'a, F>(label_text: &'a str, checked: bool, interface: Color, f: F) -> Element<'a, Message>
where
    F: 'a + Fn(bool) -> Message,
{
    row![
        text(label_text).color(Color::WHITE).width(Length::Fill),
        toggler(checked).on_toggle(f).style(move |_theme, status| toggler_style(interface, status)),
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn toggler_style(interface: Color, status: toggler::Status) -> toggler::Style {
    let active = matches!(status, toggler::Status::Active { is_toggled: true } | toggler::Status::Hovered { is_toggled: true });
    toggler::Style {
        background: if active { interface } else { Color::from_rgb(0.4, 0.4, 0.4) },
        background_border_width: 0.0,
        background_border_color: Color::TRANSPARENT,
        foreground: Color::WHITE,
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
    }
}

fn action_button<'a>(label_text: &'a str, interface: Color, msg: Message) -> Element<'a, Message> {
    button(text(label_text).color(Color::WHITE))
        .on_press(msg)
        .padding([5, 10])
        .style(move |_theme, _status| button::Style {
            border: Border { color: interface, width: 1.0, radius: 0.0.into() },
            background: None,
            ..button::Style::default()
        })
        .into()
}

fn icon_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.08))),
        _ => None,
    };
    button::Style { background: bg, border: Border::default(), ..button::Style::default() }
}

fn interface_color(app: &App) -> Color {
    let (r, g, b, a) = app.settings.theme_color.rgba();
    Color::from_rgba8(r, g, b, a as f32 / 255.0)
}
