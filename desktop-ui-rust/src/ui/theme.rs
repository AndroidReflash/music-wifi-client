use iced::theme::Palette;
use iced::{Color, Theme};

use crate::app::App;

pub fn theme(app: &App) -> Theme {
    let (r, g, b, a) = app.settings.theme_color.rgba();

    Theme::custom(
        "musicwifi".to_owned(),
        Palette {
            background: Color::BLACK,
            text: Color::WHITE,
            primary: Color::from_rgba8(r, g, b, a as f32 / 255.0),
            ..Palette::DARK
        },
    )
}
