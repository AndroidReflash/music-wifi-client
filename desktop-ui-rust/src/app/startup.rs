use iced::window;
use iced::Size;

use super::{subscription, update};
use crate::ui;

pub fn run() -> iced::Result {
    iced::application("musicwifi", update, ui::view)
        .subscription(subscription)
        .window(window::Settings {
            size: Size::new(900.0, 600.0),
            min_size: Some(Size::new(900.0, 600.0)),
            ..Default::default()
        })
        .exit_on_close_request(false)
        .theme(ui::theme)
        .centered()
        .run()
}
