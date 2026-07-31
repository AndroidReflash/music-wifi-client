use std::fs;
use std::path::Path;

/// Что кладётся в список: расширения, которые умеет открыть декодер. Всё остальное в
/// каталоге — не музыка для этого приложения.
const AUDIO: [&str; 6] = ["mp3", "flac", "ogg", "wav", "m4a", "aac"];

#[derive(Debug, Clone, uniffi::Record)]
pub struct TrackData {
    pub track_url: String,
    pub track_name: String,
    pub track_index: i32,
    pub is_local: bool,
}

pub fn scan_music_folder(folder_path: String) -> Vec<TrackData> {
    let mut tracks = Vec::new();
    if folder_path.is_empty() {
        return tracks;
    }

    let path = Path::new(&folder_path);
    if !path.exists() || !path.is_dir() {
        return tracks;
    }

    if let Ok(entries) = fs::read_dir(path) {
        let mut index = 0;
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    let ext = ext.to_string_lossy();
                    if AUDIO.iter().any(|known| ext.eq_ignore_ascii_case(known)) {
                        if let Some(path_str) = entry_path.to_str() {
                            let name = entry_path
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());

                            tracks.push(TrackData {
                                track_url: path_str.to_string(),
                                track_name: name,
                                track_index: index,
                                is_local: true,
                            });
                            index += 1;
                        }
                    }
                }
            }
        }
    }

    tracks
}
