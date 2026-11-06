use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::settings::Settings;

pub fn settings_path() -> Result<PathBuf, SettingsError> {
    let executable = std::env::current_exe()?;
    let directory = executable.parent().ok_or(SettingsError::NoParent)?;

    Ok(directory.join("database").join("settings.json"))
}

pub fn load() -> Result<Settings, SettingsError> {
    load_from(&settings_path()?)
}

pub fn load_from(path: &Path) -> Result<Settings, SettingsError> {
    match fs::read(path) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(Settings::default())
        }
        Err(error) => Err(SettingsError::Io(error)),
    }
}

pub fn save(settings: &Settings) -> Result<(), SettingsError> {
    save_to(&settings_path()?, settings)
}

pub fn save_to(path: &Path, settings: &Settings) -> Result<(), SettingsError> {
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)?;
    }

    let temporary = path.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(settings)?;

    fs::write(&temporary, bytes)?;
    fs::rename(temporary, path)?;

    Ok(())
}

#[derive(Debug)]
pub enum SettingsError {
    Io(io::Error),
    Json(serde_json::Error),
    NoParent,
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "ошибка файла настроек: {error}"),
            Self::Json(error) => write!(f, "ошибка JSON настроек: {error}"),
            Self::NoParent => f.write_str("у исполняемого файла нет каталога"),
        }
    }
}

impl std::error::Error for SettingsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::NoParent => None,
        }
    }
}

impl From<io::Error> for SettingsError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for SettingsError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "musicwifi-settings-{}-{name}",
            std::process::id()
        ));

        let _ = fs::remove_dir_all(&directory);

        directory.join("database").join("settings.json")
    }

    #[test]
    fn missing_file_returns_defaults() {
        let path = test_path("missing");

        assert_eq!(load_from(&path).unwrap(), Settings::default());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let path = test_path("roundtrip");
        let mut settings = Settings::default();
        settings.volume = 0.25;
        settings.folder_address = "/music".to_owned();

        save_to(&path, &settings).unwrap();

        assert!(path.exists());
        assert!(!path.with_extension("tmp").exists());
        assert_eq!(load_from(&path).unwrap(), settings);
    }

    #[test]
    fn invalid_json_is_reported_and_preserved() {
        let path = test_path("invalid");
        let directory = path.parent().unwrap();
        fs::create_dir_all(directory).unwrap();
        fs::write(&path, b"{").unwrap();

        assert!(load_from(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"{".to_vec());
    }
}
