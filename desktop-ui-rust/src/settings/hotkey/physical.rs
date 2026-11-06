use std::fmt;
use std::str::FromStr;

use iced::keyboard::key::{Code, NativeCode, Physical};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PhysicalKeyId {
    Code(Code),
    Xkb(u32),
}

impl PhysicalKeyId {
    pub fn from_physical(physical: Physical) -> Option<Self> {
        match physical {
            Physical::Code(code) => Some(Self::Code(code)),
            Physical::Unidentified(NativeCode::Xkb(value)) => Some(Self::Xkb(value)),
            Physical::Unidentified(_) => None,
        }
    }

    pub fn to_physical(self) -> Physical {
        match self {
            Self::Code(code) => Physical::Code(code),
            Self::Xkb(value) => Physical::Unidentified(NativeCode::Xkb(value)),
        }
    }

    pub fn display_name(self) -> String {
        match self {
            Self::Code(code) => code_name(code).to_owned(),
            Self::Xkb(value) => format!("XKB {value}"),
        }
    }
}

impl fmt::Display for PhysicalKeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Code(code) => write!(f, "code:{}", code_token(*code)),
            Self::Xkb(value) => write!(f, "xkb:{value}"),
        }
    }
}

impl FromStr for PhysicalKeyId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(rest) = s.strip_prefix("code:") {
            let code = code_from_token(rest).ok_or_else(|| format!("неизвестный код: {rest}"))?;
            return Ok(Self::Code(code));
        }
        if let Some(rest) = s.strip_prefix("xkb:") {
            let value = rest
                .parse::<u32>()
                .map_err(|_| format!("неверный xkb: {rest}"))?;
            return Ok(Self::Xkb(value));
        }
        Err(format!("неизвестный идентификатор клавиши: {s}"))
    }
}

impl Serialize for PhysicalKeyId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PhysicalKeyId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

fn code_token(code: Code) -> &'static str {
    match code {
        Code::Backquote => "Backquote",
        Code::Backslash => "Backslash",
        Code::BracketLeft => "BracketLeft",
        Code::BracketRight => "BracketRight",
        Code::Comma => "Comma",
        Code::Digit0 => "Digit0",
        Code::Digit1 => "Digit1",
        Code::Digit2 => "Digit2",
        Code::Digit3 => "Digit3",
        Code::Digit4 => "Digit4",
        Code::Digit5 => "Digit5",
        Code::Digit6 => "Digit6",
        Code::Digit7 => "Digit7",
        Code::Digit8 => "Digit8",
        Code::Digit9 => "Digit9",
        Code::Equal => "Equal",
        Code::IntlBackslash => "IntlBackslash",
        Code::IntlRo => "IntlRo",
        Code::IntlYen => "IntlYen",
        Code::KeyA => "KeyA",
        Code::KeyB => "KeyB",
        Code::KeyC => "KeyC",
        Code::KeyD => "KeyD",
        Code::KeyE => "KeyE",
        Code::KeyF => "KeyF",
        Code::KeyG => "KeyG",
        Code::KeyH => "KeyH",
        Code::KeyI => "KeyI",
        Code::KeyJ => "KeyJ",
        Code::KeyK => "KeyK",
        Code::KeyL => "KeyL",
        Code::KeyM => "KeyM",
        Code::KeyN => "KeyN",
        Code::KeyO => "KeyO",
        Code::KeyP => "KeyP",
        Code::KeyQ => "KeyQ",
        Code::KeyR => "KeyR",
        Code::KeyS => "KeyS",
        Code::KeyT => "KeyT",
        Code::KeyU => "KeyU",
        Code::KeyV => "KeyV",
        Code::KeyW => "KeyW",
        Code::KeyX => "KeyX",
        Code::KeyY => "KeyY",
        Code::KeyZ => "KeyZ",
        Code::Minus => "Minus",
        Code::Period => "Period",
        Code::Quote => "Quote",
        Code::Semicolon => "Semicolon",
        Code::Slash => "Slash",
        Code::AltLeft => "AltLeft",
        Code::AltRight => "AltRight",
        Code::Backspace => "Backspace",
        Code::CapsLock => "CapsLock",
        Code::ContextMenu => "ContextMenu",
        Code::ControlLeft => "ControlLeft",
        Code::ControlRight => "ControlRight",
        Code::Enter => "Enter",
        Code::SuperLeft => "SuperLeft",
        Code::SuperRight => "SuperRight",
        Code::ShiftLeft => "ShiftLeft",
        Code::ShiftRight => "ShiftRight",
        Code::Space => "Space",
        Code::Tab => "Tab",
        Code::Convert => "Convert",
        Code::KanaMode => "KanaMode",
        Code::Lang1 => "Lang1",
        Code::Lang2 => "Lang2",
        Code::Lang3 => "Lang3",
        Code::Lang4 => "Lang4",
        Code::Lang5 => "Lang5",
        Code::NonConvert => "NonConvert",
        Code::Delete => "Delete",
        Code::End => "End",
        Code::Help => "Help",
        Code::Home => "Home",
        Code::Insert => "Insert",
        Code::PageDown => "PageDown",
        Code::PageUp => "PageUp",
        Code::ArrowDown => "ArrowDown",
        Code::ArrowLeft => "ArrowLeft",
        Code::ArrowRight => "ArrowRight",
        Code::ArrowUp => "ArrowUp",
        Code::NumLock => "NumLock",
        Code::Numpad0 => "Numpad0",
        Code::Numpad1 => "Numpad1",
        Code::Numpad2 => "Numpad2",
        Code::Numpad3 => "Numpad3",
        Code::Numpad4 => "Numpad4",
        Code::Numpad5 => "Numpad5",
        Code::Numpad6 => "Numpad6",
        Code::Numpad7 => "Numpad7",
        Code::Numpad8 => "Numpad8",
        Code::Numpad9 => "Numpad9",
        Code::NumpadAdd => "NumpadAdd",
        Code::NumpadBackspace => "NumpadBackspace",
        Code::NumpadClear => "NumpadClear",
        Code::NumpadClearEntry => "NumpadClearEntry",
        Code::NumpadComma => "NumpadComma",
        Code::NumpadDecimal => "NumpadDecimal",
        Code::NumpadDivide => "NumpadDivide",
        Code::NumpadEnter => "NumpadEnter",
        Code::NumpadEqual => "NumpadEqual",
        Code::NumpadHash => "NumpadHash",
        Code::NumpadMemoryAdd => "NumpadMemoryAdd",
        Code::NumpadMemoryClear => "NumpadMemoryClear",
        Code::NumpadMemoryRecall => "NumpadMemoryRecall",
        Code::NumpadMemoryStore => "NumpadMemoryStore",
        Code::NumpadMemorySubtract => "NumpadMemorySubtract",
        Code::NumpadMultiply => "NumpadMultiply",
        Code::NumpadParenLeft => "NumpadParenLeft",
        Code::NumpadParenRight => "NumpadParenRight",
        Code::NumpadStar => "NumpadStar",
        Code::NumpadSubtract => "NumpadSubtract",
        Code::Escape => "Escape",
        Code::Fn => "Fn",
        Code::FnLock => "FnLock",
        Code::PrintScreen => "PrintScreen",
        Code::ScrollLock => "ScrollLock",
        Code::Pause => "Pause",
        Code::BrowserBack => "BrowserBack",
        Code::BrowserFavorites => "BrowserFavorites",
        Code::BrowserForward => "BrowserForward",
        Code::BrowserHome => "BrowserHome",
        Code::BrowserRefresh => "BrowserRefresh",
        Code::BrowserSearch => "BrowserSearch",
        Code::BrowserStop => "BrowserStop",
        Code::Eject => "Eject",
        Code::LaunchApp1 => "LaunchApp1",
        Code::LaunchApp2 => "LaunchApp2",
        Code::LaunchMail => "LaunchMail",
        Code::MediaPlayPause => "MediaPlayPause",
        Code::MediaSelect => "MediaSelect",
        Code::MediaStop => "MediaStop",
        Code::MediaTrackNext => "MediaTrackNext",
        Code::MediaTrackPrevious => "MediaTrackPrevious",
        Code::Power => "Power",
        Code::Sleep => "Sleep",
        Code::AudioVolumeDown => "AudioVolumeDown",
        Code::AudioVolumeMute => "AudioVolumeMute",
        Code::AudioVolumeUp => "AudioVolumeUp",
        Code::WakeUp => "WakeUp",
        Code::Meta => "Meta",
        Code::Hyper => "Hyper",
        Code::Turbo => "Turbo",
        Code::Abort => "Abort",
        Code::Resume => "Resume",
        Code::Suspend => "Suspend",
        Code::Again => "Again",
        Code::Copy => "Copy",
        Code::Cut => "Cut",
        Code::Find => "Find",
        Code::Open => "Open",
        Code::Paste => "Paste",
        Code::Props => "Props",
        Code::Select => "Select",
        Code::Undo => "Undo",
        Code::Hiragana => "Hiragana",
        Code::Katakana => "Katakana",
        Code::F1 => "F1",
        Code::F2 => "F2",
        Code::F3 => "F3",
        Code::F4 => "F4",
        Code::F5 => "F5",
        Code::F6 => "F6",
        Code::F7 => "F7",
        Code::F8 => "F8",
        Code::F9 => "F9",
        Code::F10 => "F10",
        Code::F11 => "F11",
        Code::F12 => "F12",
        Code::F13 => "F13",
        Code::F14 => "F14",
        Code::F15 => "F15",
        Code::F16 => "F16",
        Code::F17 => "F17",
        Code::F18 => "F18",
        Code::F19 => "F19",
        Code::F20 => "F20",
        Code::F21 => "F21",
        Code::F22 => "F22",
        Code::F23 => "F23",
        Code::F24 => "F24",
        Code::F25 => "F25",
        Code::F26 => "F26",
        Code::F27 => "F27",
        Code::F28 => "F28",
        Code::F29 => "F29",
        Code::F30 => "F30",
        Code::F31 => "F31",
        Code::F32 => "F32",
        Code::F33 => "F33",
        Code::F34 => "F34",
        Code::F35 => "F35",
        _ => "Unknown",
    }
}

fn code_name(code: Code) -> &'static str {
    match code {
        Code::KeyA => "A",
        Code::KeyB => "B",
        Code::KeyC => "C",
        Code::KeyD => "D",
        Code::KeyE => "E",
        Code::KeyF => "F",
        Code::KeyG => "G",
        Code::KeyH => "H",
        Code::KeyI => "I",
        Code::KeyJ => "J",
        Code::KeyK => "K",
        Code::KeyL => "L",
        Code::KeyM => "M",
        Code::KeyN => "N",
        Code::KeyO => "O",
        Code::KeyP => "P",
        Code::KeyQ => "Q",
        Code::KeyR => "R",
        Code::KeyS => "S",
        Code::KeyT => "T",
        Code::KeyU => "U",
        Code::KeyV => "V",
        Code::KeyW => "W",
        Code::KeyX => "X",
        Code::KeyY => "Y",
        Code::KeyZ => "Z",
        Code::Digit0 => "0",
        Code::Digit1 => "1",
        Code::Digit2 => "2",
        Code::Digit3 => "3",
        Code::Digit4 => "4",
        Code::Digit5 => "5",
        Code::Digit6 => "6",
        Code::Digit7 => "7",
        Code::Digit8 => "8",
        Code::Digit9 => "9",
        Code::Space => "Space",
        Code::Enter => "Enter",
        Code::NumpadEnter => "Numpad Enter",
        Code::Escape => "Esc",
        Code::ArrowLeft => "Left",
        Code::ArrowRight => "Right",
        Code::ArrowUp => "Up",
        Code::ArrowDown => "Down",
        Code::PageUp => "Page Up",
        Code::PageDown => "Page Down",
        Code::MediaPlayPause => "Play/Pause",
        Code::MediaTrackNext => "Next Track",
        Code::MediaTrackPrevious => "Prev Track",
        Code::MediaStop => "Stop",
        Code::AudioVolumeUp => "Volume Up",
        Code::AudioVolumeDown => "Volume Down",
        Code::AudioVolumeMute => "Mute",
        _ => code_token(code),
    }
}

fn code_from_token(token: &str) -> Option<Code> {
    Some(match token {
        "Backquote" => Code::Backquote,
        "Backslash" => Code::Backslash,
        "BracketLeft" => Code::BracketLeft,
        "BracketRight" => Code::BracketRight,
        "Comma" => Code::Comma,
        "Digit0" => Code::Digit0,
        "Digit1" => Code::Digit1,
        "Digit2" => Code::Digit2,
        "Digit3" => Code::Digit3,
        "Digit4" => Code::Digit4,
        "Digit5" => Code::Digit5,
        "Digit6" => Code::Digit6,
        "Digit7" => Code::Digit7,
        "Digit8" => Code::Digit8,
        "Digit9" => Code::Digit9,
        "Equal" => Code::Equal,
        "IntlBackslash" => Code::IntlBackslash,
        "IntlRo" => Code::IntlRo,
        "IntlYen" => Code::IntlYen,
        "KeyA" => Code::KeyA,
        "KeyB" => Code::KeyB,
        "KeyC" => Code::KeyC,
        "KeyD" => Code::KeyD,
        "KeyE" => Code::KeyE,
        "KeyF" => Code::KeyF,
        "KeyG" => Code::KeyG,
        "KeyH" => Code::KeyH,
        "KeyI" => Code::KeyI,
        "KeyJ" => Code::KeyJ,
        "KeyK" => Code::KeyK,
        "KeyL" => Code::KeyL,
        "KeyM" => Code::KeyM,
        "KeyN" => Code::KeyN,
        "KeyO" => Code::KeyO,
        "KeyP" => Code::KeyP,
        "KeyQ" => Code::KeyQ,
        "KeyR" => Code::KeyR,
        "KeyS" => Code::KeyS,
        "KeyT" => Code::KeyT,
        "KeyU" => Code::KeyU,
        "KeyV" => Code::KeyV,
        "KeyW" => Code::KeyW,
        "KeyX" => Code::KeyX,
        "KeyY" => Code::KeyY,
        "KeyZ" => Code::KeyZ,
        "Minus" => Code::Minus,
        "Period" => Code::Period,
        "Quote" => Code::Quote,
        "Semicolon" => Code::Semicolon,
        "Slash" => Code::Slash,
        "AltLeft" => Code::AltLeft,
        "AltRight" => Code::AltRight,
        "Backspace" => Code::Backspace,
        "CapsLock" => Code::CapsLock,
        "ContextMenu" => Code::ContextMenu,
        "ControlLeft" => Code::ControlLeft,
        "ControlRight" => Code::ControlRight,
        "Enter" => Code::Enter,
        "SuperLeft" => Code::SuperLeft,
        "SuperRight" => Code::SuperRight,
        "ShiftLeft" => Code::ShiftLeft,
        "ShiftRight" => Code::ShiftRight,
        "Space" => Code::Space,
        "Tab" => Code::Tab,
        "Delete" => Code::Delete,
        "End" => Code::End,
        "Home" => Code::Home,
        "Insert" => Code::Insert,
        "PageDown" => Code::PageDown,
        "PageUp" => Code::PageUp,
        "ArrowDown" => Code::ArrowDown,
        "ArrowLeft" => Code::ArrowLeft,
        "ArrowRight" => Code::ArrowRight,
        "ArrowUp" => Code::ArrowUp,
        "NumLock" => Code::NumLock,
        "Numpad0" => Code::Numpad0,
        "Numpad1" => Code::Numpad1,
        "Numpad2" => Code::Numpad2,
        "Numpad3" => Code::Numpad3,
        "Numpad4" => Code::Numpad4,
        "Numpad5" => Code::Numpad5,
        "Numpad6" => Code::Numpad6,
        "Numpad7" => Code::Numpad7,
        "Numpad8" => Code::Numpad8,
        "Numpad9" => Code::Numpad9,
        "NumpadAdd" => Code::NumpadAdd,
        "NumpadDecimal" => Code::NumpadDecimal,
        "NumpadDivide" => Code::NumpadDivide,
        "NumpadEnter" => Code::NumpadEnter,
        "NumpadMultiply" => Code::NumpadMultiply,
        "NumpadSubtract" => Code::NumpadSubtract,
        "Escape" => Code::Escape,
        "PrintScreen" => Code::PrintScreen,
        "ScrollLock" => Code::ScrollLock,
        "Pause" => Code::Pause,
        "MediaPlayPause" => Code::MediaPlayPause,
        "MediaStop" => Code::MediaStop,
        "MediaTrackNext" => Code::MediaTrackNext,
        "MediaTrackPrevious" => Code::MediaTrackPrevious,
        "AudioVolumeDown" => Code::AudioVolumeDown,
        "AudioVolumeMute" => Code::AudioVolumeMute,
        "AudioVolumeUp" => Code::AudioVolumeUp,
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        "F13" => Code::F13,
        "F14" => Code::F14,
        "F15" => Code::F15,
        "F16" => Code::F16,
        "F17" => Code::F17,
        "F18" => Code::F18,
        "F19" => Code::F19,
        "F20" => Code::F20,
        "F21" => Code::F21,
        "F22" => Code::F22,
        "F23" => Code::F23,
        "F24" => Code::F24,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_code() {
        let id = PhysicalKeyId::Code(Code::KeyA);
        assert_eq!(PhysicalKeyId::from_str(&id.to_string()).unwrap(), id);
    }

    #[test]
    fn roundtrip_xkb() {
        let id = PhysicalKeyId::Xkb(38);
        assert_eq!(PhysicalKeyId::from_str(&id.to_string()).unwrap(), id);
    }
}
