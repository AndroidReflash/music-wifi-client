use std::fmt;
use std::str::FromStr;

use iced::keyboard::Modifiers;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::physical::PhysicalKeyId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HotkeyBinding {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub key: PhysicalKeyId,
}

impl HotkeyBinding {
    pub fn from_physical(key: PhysicalKeyId, modifiers: Modifiers) -> Self {
        Self {
            ctrl: modifiers.control(),
            alt: modifiers.alt(),
            shift: modifiers.shift(),
            key,
        }
    }
}

impl fmt::Display for HotkeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if self.ctrl {
            parts.push("ctrl".to_owned());
        }

        if self.alt {
            parts.push("alt".to_owned());
        }

        if self.shift {
            parts.push("shift".to_owned());
        }

        parts.push(self.key.to_string());

        f.write_str(&parts.join("+"))
    }
}

impl FromStr for HotkeyBinding {
    type Err = HotkeyParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut key = None;

        for part in input.split('+') {
            let part = part.trim();

            if part.is_empty() {
                return Err(HotkeyParseError::Empty);
            }

            match part.to_ascii_lowercase().as_str() {
                "ctrl" => ctrl = true,
                "alt" => alt = true,
                "shift" => shift = true,
                _ => {
                    let parsed = PhysicalKeyId::from_str(part)
                        .map_err(|_| HotkeyParseError::UnknownKey(part.to_owned()))?;

                    if key.replace(parsed).is_some() {
                        return Err(HotkeyParseError::MultipleKeys);
                    }
                }
            }
        }

        Ok(Self {
            ctrl,
            alt,
            shift,
            key: key.ok_or(HotkeyParseError::MissingKey)?,
        })
    }
}

impl Serialize for HotkeyBinding {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for HotkeyBinding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotkeyParseError {
    Empty,
    UnknownKey(String),
    MultipleKeys,
    MissingKey,
}

impl fmt::Display for HotkeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("пустая комбинация"),
            Self::UnknownKey(key) => write!(f, "неизвестная клавиша: {key}"),
            Self::MultipleKeys => f.write_str("в комбинации больше одной клавиши"),
            Self::MissingKey => f.write_str("в комбинации нет клавиши"),
        }
    }
}

impl std::error::Error for HotkeyParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::key::Code;

    fn binding(ctrl: bool, alt: bool, shift: bool, key: PhysicalKeyId) -> HotkeyBinding {
        HotkeyBinding {
            ctrl,
            alt,
            shift,
            key,
        }
    }

    #[test]
    fn formats_modifiers_in_stable_order() {
        assert_eq!(
            binding(true, true, true, PhysicalKeyId::Code(Code::KeyA)).to_string(),
            "ctrl+alt+shift+code:KeyA"
        );
        assert_eq!(
            binding(false, true, false, PhysicalKeyId::Code(Code::Digit7)).to_string(),
            "alt+code:Digit7"
        );
    }

    #[test]
    fn parses_modifiers_in_any_order() {
        let parsed = "shift+code:KeyA+ctrl".parse::<HotkeyBinding>().unwrap();

        assert_eq!(parsed, binding(true, false, true, PhysicalKeyId::Code(Code::KeyA)));
    }

    #[test]
    fn rejects_invalid_bindings() {
        assert_eq!("".parse::<HotkeyBinding>(), Err(HotkeyParseError::Empty));
        assert_eq!(
            "ctrl".parse::<HotkeyBinding>(),
            Err(HotkeyParseError::MissingKey)
        );
        assert_eq!(
            "ctrl+code:KeyA+code:KeyB".parse::<HotkeyBinding>(),
            Err(HotkeyParseError::MultipleKeys)
        );
        assert!(matches!(
            "ctrl+unknown".parse::<HotkeyBinding>(),
            Err(HotkeyParseError::UnknownKey(_))
        ));
    }

    #[test]
    fn serde_roundtrip() {
        let value = binding(true, false, true, PhysicalKeyId::Xkb(38));
        let json = serde_json::to_string(&value).unwrap();

        assert_eq!(json, "\"ctrl+shift+xkb:38\"");
        assert_eq!(
            serde_json::from_str::<HotkeyBinding>(&json).unwrap(),
            value
        );
    }
}
