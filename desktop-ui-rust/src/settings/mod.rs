mod color;
pub mod hotkey;
pub mod palette;
mod settings;
mod store;

pub use color::{ColorError, ThemeColor};
pub use hotkey::{binding_from_iced, HotkeyAction, HotkeyBinding, HotkeyParseError, PhysicalKeyId};
pub use settings::Settings;
pub use store::{load, save, settings_path, SettingsError};
