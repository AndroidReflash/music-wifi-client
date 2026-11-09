use std::time::Duration;

use iced::event;
use iced::{keyboard, time, window, Event, Subscription};
use super::{App, Message};

pub fn subscription(app: &App) -> Subscription<Message> {
    Subscription::batch([
        Subscription::run_with_id("player-notices", app.playback.notices()).map(Message::from),
        Subscription::run_with_id("player-bands", app.playback.bands()).map(Message::Bands),
        Subscription::run_with_id("network-notices", app.network_notices.clone()).map(Message::Network),
        Subscription::run_with_id("global-hotkeys", app.global_hotkeys_notices.clone())
            .map(Message::GlobalHotkey),
        time::every(Duration::from_millis(10)).map(|_| Message::Tick),
        window::close_requests().map(Message::CloseRequested),
        event::listen_with(|event, status, _window| match (event, status) {
            (Event::Keyboard(keyboard::Event::KeyPressed { physical_key, modifiers, .. }), event::Status::Ignored) => {
                Some(Message::KeyPressed { physical_key, modifiers })
            }
            (Event::Keyboard(keyboard::Event::KeyReleased { .. }), _) => Some(Message::KeyReleased),
            (Event::Window(window::Event::Unfocused), _) => Some(Message::WindowUnfocused),
            _ => None,
        }),
    ])
}
