use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::color::ThemeColor;
use super::hotkey::{HotkeyAction, HotkeyBinding};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub server_address: String,
    pub is_local: bool,
    pub folder_address: String,
    pub theme_color: ThemeColor,
    pub show_visualizer: bool,
    pub show_cover: bool,
    pub output_device: String,
    pub volume: f32,
    pub equalizer: Vec<f32>,
    pub visualizer_ring: bool,
    pub cover_small: bool,
    pub show_cover_image: bool,
    pub panel_divider: bool,
    pub panel_transparent: bool,
    pub hotkeys_global: bool,
    pub shuffle: bool,
    pub repeat: bool,
    pub autoplay: bool,
    pub hotkeys: BTreeMap<HotkeyAction, HotkeyBinding>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server_address: String::new(),
            is_local: false,
            folder_address: String::new(),
            theme_color: ThemeColor::CYAN,
            show_visualizer: true,
            show_cover: true,
            output_device: String::new(),
            volume: 1.0,
            equalizer: Vec::new(),
            visualizer_ring: false,
            cover_small: false,
            show_cover_image: true,
            panel_divider: false,
            panel_transparent: false,
            hotkeys_global: false,
            shuffle: false,
            repeat: false,
            autoplay: false,
            hotkeys: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::PhysicalKeyId;
    use iced::keyboard::key::Code;

    #[test]
    fn defaults_match_desktop_behavior() {
        let settings = Settings::default();

        assert_eq!(settings.server_address, "");
        assert!(!settings.is_local);
        assert_eq!(settings.theme_color, ThemeColor::CYAN);
        assert!(settings.show_visualizer);
        assert!(settings.show_cover);
        assert!(settings.show_cover_image);
        assert_eq!(settings.volume, 1.0);
        assert!(settings.equalizer.is_empty());
        assert!(!settings.visualizer_ring);
        assert!(!settings.cover_small);
        assert!(!settings.hotkeys_global);
        assert!(!settings.shuffle);
        assert!(!settings.repeat);
        assert!(!settings.autoplay);
        assert!(settings.hotkeys.is_empty());
    }

    #[test]
    fn missing_fields_use_defaults() {
        let decoded = serde_json::from_str::<Settings>("{}").unwrap();

        assert_eq!(decoded, Settings::default());
    }

    #[test]
    fn serde_roundtrip_with_hotkey() {
        let mut settings = Settings::default();
        settings.volume = 0.5;
        settings.hotkeys.insert(
            HotkeyAction::PlayPause,
            HotkeyBinding {
                ctrl: true,
                alt: false,
                shift: false,
                key: PhysicalKeyId::Code(Code::KeyA),
            },
        );

        let json = serde_json::to_string(&settings).unwrap();
        let decoded = serde_json::from_str::<Settings>(&json).unwrap();

        assert_eq!(decoded, settings);
        assert!(json.contains("\"play_pause\":\"ctrl+code:KeyA\""));
    }
}
