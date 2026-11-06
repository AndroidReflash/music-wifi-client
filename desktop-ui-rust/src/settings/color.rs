use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ThemeColor {
    pub const CYAN: Self = Self {
        r: 0,
        g: 255,
        b: 255,
        a: 255,
    };

    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };

    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };

    pub const ORANGE: Self = Self {
        r: 255,
        g: 165,
        b: 0,
        a: 255,
    };

    pub const VIOLET: Self = Self {
        r: 180,
        g: 130,
        b: 255,
        a: 255,
    };

    pub fn parse(input: &str) -> Result<Self, ColorError> {
        let hex = input
            .strip_prefix('#')
            .ok_or_else(|| ColorError::new("цвет должен начинаться с #"))?;

        if hex.len() != 6 && hex.len() != 8 {
            return Err(ColorError::new("ожидается #rrggbb или #rrggbbaa"));
        }

        if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ColorError::new("цвет должен быть шестнадцатеричным"));
        }

        let parse = |start: usize| {
            u8::from_str_radix(&hex[start..start + 2], 16)
                .map_err(|_| ColorError::new("цвет должен быть шестнадцатеричным"))
        };

        Ok(Self {
            r: parse(0)?,
            g: parse(2)?,
            b: parse(4)?,
            a: if hex.len() == 8 { parse(6)? } else { 255 },
        })
    }

    pub fn to_hex(self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    pub fn rgba(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }
}

impl Default for ThemeColor {
    fn default() -> Self {
        Self::CYAN
    }
}

impl fmt::Display for ThemeColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl Serialize for ThemeColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for ThemeColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorError(String);

impl ColorError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ColorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_opaque_and_translucent_colors() {
        assert_eq!(ThemeColor::parse("#00ffff"), Ok(ThemeColor::CYAN));
        assert_eq!(
            ThemeColor::parse("#00ffff80"),
            Ok(ThemeColor {
                r: 0,
                g: 255,
                b: 255,
                a: 128
            })
        );
    }

    #[test]
    fn rejects_invalid_colors() {
        assert!(ThemeColor::parse("00ffff").is_err());
        assert!(ThemeColor::parse("#fff").is_err());
        assert!(ThemeColor::parse("#zzzzzz").is_err());
        assert!(ThemeColor::parse("#ééé").is_err());
    }

    #[test]
    fn formats_opaque_and_translucent_colors() {
        assert_eq!(ThemeColor::CYAN.to_hex(), "#00ffff");
        assert_eq!(
            ThemeColor {
                r: 1,
                g: 2,
                b: 3,
                a: 4
            }
            .to_hex(),
            "#01020304"
        );
    }

    #[test]
    fn serde_roundtrip() {
        let json = serde_json::to_string(&ThemeColor::CYAN).unwrap();
        assert_eq!(json, "\"#00ffff\"");
        assert_eq!(
            serde_json::from_str::<ThemeColor>(&json).unwrap(),
            ThemeColor::CYAN
        );
    }
}
