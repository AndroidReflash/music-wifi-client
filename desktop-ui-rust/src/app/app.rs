use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use iced::widget::{image, scrollable};
use player_core::{Player, PlayerSnapshot, TrackData};

use crate::settings::{self, PhysicalKeyId, Settings};
use crate::platform::hotkeys::x11::{GlobalHotkeyNotice, GlobalHotkeysHandle, GlobalHotkeysState};
use crate::repository::{NetworkHandle, NetworkNotice, NetworkStatus};

use super::dialog::Dialog;
use super::player::{ChannelStream, PlayerHandle};

pub struct App {
    pub settings: Settings,
    pub playback: PlayerHandle,
    pub network: NetworkHandle,
    pub network_notices: ChannelStream<NetworkNotice>,
    pub player: Option<Arc<Player>>,
    pub tracks: Vec<TrackData>,
    pub snapshot: Option<PlayerSnapshot>,
    pub bands: Vec<f64>,
    pub smoothed_bands: Vec<f32>,
    pub cover: Option<image::Handle>,
    pub cover_for_index: Option<usize>,
    pub cover_loading: Option<usize>,
    pub spin_deg: f32,
    pub last_tick: Option<Instant>,
    pub player_error: Option<String>,
    pub playlist_error: Option<String>,
    pub network_status: NetworkStatus,
    pub network_error: Option<String>,
    pub network_generation: u64,
    pub list_scrollable: scrollable::Id,
    pub list_viewport: Option<scrollable::Viewport>,
    pub search_active: bool,
    pub search_query: String,
    pub selected_track: Option<usize>,
    pub seek_drag: Option<f32>,
    pub marquee_scroll: f32,
    pub dialog: Option<Dialog>,
    pub output_devices: Vec<String>,
    pub sound_error: Option<String>,
    pub volume_drag: Option<f32>,
    pub eq_bands_hz: Vec<f32>,
    pub eq_max_db: f32,
    pub pressed_keys: HashSet<PhysicalKeyId>,
    pub global_hotkeys: GlobalHotkeysHandle,
    pub global_hotkeys_notices: ChannelStream<GlobalHotkeyNotice>,
    pub global_hotkeys_state: GlobalHotkeysState,
    pub autoplay_pending: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::load()
    }
}

impl App {
    pub fn load() -> Self {
        let settings = match settings::load() {
            Ok(settings) => settings,
            Err(error) => {
                eprintln!("musicwifi: {error}");
                Settings::default()
            }
        };

        let playback = PlayerHandle::start(settings.folder_address.clone());
        let network = NetworkHandle::start();
        let network_notices = network.notices();

        let eq_bands_hz = vec![
            31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
        ];

        let (global_hotkeys, global_notices_rx) = crate::platform::hotkeys::x11::spawn();
        let global_hotkeys_notices = ChannelStream::new(std::sync::Arc::new(std::sync::Mutex::new(global_notices_rx)));
        let global_hotkeys_state = if settings.hotkeys_global && crate::platform::hotkeys::wayland_session() {
            GlobalHotkeysState::Unavailable(crate::platform::hotkeys::wayland::unavailable_reason())
        } else {
            GlobalHotkeysState::Off
        };
        if !(settings.hotkeys_global && crate::platform::hotkeys::wayland_session()) {
            global_hotkeys.apply(settings.hotkeys_global, &settings.hotkeys);
        }

        let autoplay_pending = settings.autoplay;
        let mut app = Self {
            settings,
            playback,
            network,
            network_notices,
            player: None,
            tracks: Vec::new(),
            snapshot: None,
            bands: Vec::new(),
            smoothed_bands: Vec::new(),
            cover: None,
            cover_for_index: None,
            cover_loading: None,
            spin_deg: 0.0,
            last_tick: None,
            player_error: None,
            playlist_error: None,
            network_status: NetworkStatus::Disconnected,
            network_error: None,
            network_generation: 0,
            list_scrollable: scrollable::Id::unique(),
            list_viewport: None,
            search_active: false,
            search_query: String::new(),
            selected_track: None,
            seek_drag: None,
            marquee_scroll: 0.0,
            dialog: None,
            output_devices: Vec::new(),
            sound_error: None,
            volume_drag: None,
            eq_bands_hz,
            eq_max_db: 12.0,
            pressed_keys: HashSet::new(),
            global_hotkeys,
            global_hotkeys_notices,
            global_hotkeys_state,
            autoplay_pending,
        };
        app.sync_source();
        app
    }

    pub fn sync_source(&mut self) {
        if self.settings.is_local {
            self.network.apply(false, String::new());
            self.network_status = NetworkStatus::Disconnected;
            self.network_error = None;
            self.network_generation = self.network.generation();
            self.playback
                .set_folder(self.settings.folder_address.clone());
            if let Some(player) = &self.player {
                let _ = player.set_session(None);
            }
        } else {
            if let Some(watcher) = self.playback.take_watcher() {
                watcher.stop();
            }
            self.network
                .apply(true, self.settings.server_address.clone());
            self.network_generation = self.network.generation();
            self.network_status = NetworkStatus::Connecting;
            self.network_error = None;
            self.tracks.clear();
            self.selected_track = None;
            self.cover = None;
            self.cover_for_index = None;
            self.cover_loading = None;
            self.smoothed_bands.clear();
            if let Some(player) = &self.player {
                let session = self.network.session_for(&self.settings.server_address);
                let _ = player.set_session(session);
            }
            if let Some(player) = &self.player {
                let _ = player.set_queue(Vec::new(), None);
            }
        }
    }

    pub fn refresh_snapshot(&mut self) {
        self.snapshot = self.player.as_ref().map(|player| player.snapshot());
        if self.seek_drag.is_some() {
            if let (Some(drag), Some(snapshot)) = (self.seek_drag, self.snapshot.as_ref()) {
                if let Some(duration) = snapshot.duration_ms {
                    if duration > 0 {
                        let actual = snapshot.position_ms as f32 / duration as f32;
                        if (actual - drag).abs() < 0.02 {
                            self.seek_drag = None;
                        }
                    }
                }
            }
        }
    }

    pub fn current_track_index(&self) -> Option<usize> {
        self.snapshot
            .as_ref()
            .and_then(|s| s.track_index)
            .map(|i| i as usize)
            .or(self.selected_track)
    }

    pub fn is_playing(&self) -> bool {
        self.snapshot
            .as_ref()
            .map(|s| s.state == player_core::PlayerState::Playing)
            .unwrap_or(false)
    }

    pub fn visible_tracks(&self) -> Vec<(usize, &TrackData)> {
        if self.search_query.trim().is_empty() {
            self.tracks.iter().enumerate().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.tracks
                .iter()
                .enumerate()
                .filter(|(_, track)| track.track_name.to_lowercase().contains(&query))
                .collect()
        }
    }

    pub fn current_title(&self) -> String {
        let snapshot_index = self
            .snapshot
            .as_ref()
            .and_then(|s| s.track_index)
            .map(|i| i as usize);
        let index = snapshot_index.or(self.selected_track);
        if let Some(index) = index {
            if let Some(track) = self.tracks.get(index) {
                return track.track_name.clone();
            }
        }
        if self.player.is_none() {
            "плеер не подключён".to_owned()
        } else if !self.settings.is_local {
            match &self.network_status {
                NetworkStatus::Connecting => "подключение к серверу…".to_owned(),
                NetworkStatus::Connected if self.tracks.is_empty() => "плейлист пуст".to_owned(),
                NetworkStatus::Connected => String::new(),
                NetworkStatus::Disconnected => self
                    .network_error
                    .clone()
                    .unwrap_or_else(|| "сервер не подключён".to_owned()),
            }
        } else if self.tracks.is_empty() {
            "папка не выбрана или пуста".to_owned()
        } else {
            String::new()
        }
    }

    pub fn progress(&self) -> f32 {
        if let Some(drag) = self.seek_drag {
            return drag;
        }
        self.snapshot
            .as_ref()
            .and_then(|snapshot| {
                let duration = snapshot.duration_ms?;
                if duration == 0 {
                    None
                } else {
                    Some((snapshot.position_ms as f32 / duration as f32).clamp(0.0, 1.0))
                }
            })
            .unwrap_or(0.0)
    }

    pub fn time_labels(&self) -> (String, String) {
        let snapshot = self.snapshot.as_ref();
        let position = snapshot.map(|s| s.position_ms).unwrap_or(0);
        let duration = snapshot.and_then(|s| s.duration_ms).unwrap_or(0);
        (format_ms(position), format_ms(duration))
    }

    pub fn effective_volume(&self) -> f32 {
        self.volume_drag.unwrap_or(self.settings.volume)
    }
}

fn format_ms(ms: u64) -> String {
    let total = ms / 1000;
    let minutes = total / 60;
    let seconds = total % 60;
    format!("{minutes:02}:{seconds:02}")
}
