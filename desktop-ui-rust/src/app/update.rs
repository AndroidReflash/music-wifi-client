use std::path::{Path, PathBuf};
use std::thread;

use iced::keyboard::key::{Code, Physical};
use iced::widget::scrollable;
use iced::{exit, Task};
use player_core::{PlayerEvent, PlayerState, RepeatMode};

use crate::platform::hotkeys;
use crate::platform::hotkeys::x11::{GlobalHotkeyNotice, GlobalHotkeysState};
use crate::repository::{NetworkNotice, NetworkStatus};
use crate::settings::{self, binding_from_iced, HotkeyAction};

use super::dialog::{
    ChangeIpState, ColorPickerState, Dialog, EqualizerState, FolderPickerState, HotkeysCaptureState,
};
use super::{App, Message};

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::PlayerReady(player) => {
            player.set_volume(app.settings.volume);
            let gains = eq_gains(app);
            if let Err(error) = player.set_equalizer(gains) {
                app.sound_error = Some(error.to_string());
            }
            let device = if app.settings.output_device.trim().is_empty() {
                None
            } else {
                Some(app.settings.output_device.clone())
            };
            if let Err(error) = player.set_output_device(device) {
                app.sound_error = Some(error.to_string());
            }
            if let Err(error) = player.set_shuffle(app.settings.shuffle) {
                app.player_error = Some(error.to_string());
            }
            if let Err(error) = player.set_repeat(settings_repeat(app.settings.repeat)) {
                app.player_error = Some(error.to_string());
            }
            if !app.settings.is_local {
                let session = app.network.session_for(&app.settings.server_address);
                if let Err(error) = player.set_session(session) {
                    app.player_error = Some(error.to_string());
                }
            }
            app.player = Some(player);
            app.player_error = None;
            apply_queue(app);
            app.refresh_snapshot();
            log::info!(
                "autoplay: player ready pending={} tracks={}",
                app.autoplay_pending,
                app.tracks.len()
            );
            if let Some(task) = maybe_autoplay(app) {
                return task;
            }
        }
        Message::PlayerFailed(error) => {
            app.player = None;
            app.snapshot = None;
            app.player_error = Some(error.to_string());
        }
        Message::PlayerEvent(event) => {
            if let PlayerEvent::Failed { error } = &event {
                app.player_error = Some(error.to_string());
            }
            if matches!(
                event,
                PlayerEvent::PositionChanged | PlayerEvent::TrackStarted { .. } | PlayerEvent::TrackFinished
            ) {
                app.seek_drag = None;
            }
            app.refresh_snapshot();
            if matches!(event, PlayerEvent::TrackStarted { .. } | PlayerEvent::TrackFinished | PlayerEvent::QueueFinished) {
                return maybe_load_cover(app);
            }
        }
        Message::Bands(bands) => {
            app.bands = bands;
        }
        Message::Playlist(tracks) => {
            if app.settings.is_local {
                log::info!("autoplay: local playlist len={} pending={} player={} track={:?}", tracks.len(), app.autoplay_pending, app.player.is_some(), app.snapshot.as_ref().and_then(|s| s.track_index));
                set_playlist(app, tracks, None);
                return maybe_autoplay(app).unwrap_or_else(|| maybe_load_cover(app));
            }
        }
        Message::PlaylistFailed(reason) => {
            if app.settings.is_local {
                app.playlist_error = Some(reason);
            }
        }
        Message::Tick => {
            let now = std::time::Instant::now();
            let dt = app
                .last_tick
                .map(|prev| (now - prev).as_secs_f32())
                .unwrap_or(0.0);
            app.last_tick = Some(now);

            app.refresh_snapshot();
            app.marquee_scroll = (app.marquee_scroll + dt).rem_euclid(10000.0);
            if app.is_playing() {
                app.spin_deg = (app.spin_deg + 200.0 * dt.min(0.25)) % 360.0;
            }
            if app.settings.show_visualizer && !app.bands.is_empty() {
                crate::ui::screen::visualizer::smooth_bands(&mut app.smoothed_bands, &app.bands);
            }
            return maybe_load_cover(app);
        }
        Message::ListScrolled(viewport) => {
            app.list_viewport = Some(viewport);
        }
        Message::ListScrollTo(relative) => {
            return scrollable::snap_to(
                app.list_scrollable.clone(),
                scrollable::RelativeOffset {
                    x: 0.0,
                    y: relative.clamp(0.0, 1.0),
                },
            );
        }
        Message::SearchToggled => {
            app.search_active = !app.search_active;
            if !app.search_active {
                app.search_query.clear();
            }
        }
        Message::SearchQueryChanged(value) => {
            app.search_query = value;
        }
        Message::TrackPressed(index) => {
            app.autoplay_pending = false;
            app.selected_track = Some(index);
            app.seek_drag = None;
            if let Some(player) = &app.player {
                if let Err(error) = player.play_track(index as u32) {
                    app.player_error = Some(error.to_string());
                }
            }
            return maybe_load_cover(app);
        }
        Message::TitlePressed => {
            if let Some(index) = app.current_track_index() {
                let total = app.tracks.len();
                if total == 0 {
                    return Task::none();
                }
                let relative = if let Some(viewport) = app.list_viewport {
                    let content_h = viewport.content_bounds().height;
                    let view_h = viewport.bounds().height;
                    let scrollable_h = (content_h - view_h).max(0.0);
                    if scrollable_h > 0.0 {
                        let row_h = content_h / total as f32;
                        let target = index as f32 * row_h;
                        (target / scrollable_h).clamp(0.0, 1.0)
                    } else {
                        0.0
                    }
                } else if total <= 1 {
                    0.0
                } else {
                    index as f32 / (total - 1) as f32
                };
                return scrollable::snap_to(
                    app.list_scrollable.clone(),
                    scrollable::RelativeOffset { x: 0.0, y: relative },
                );
            }
        }
        Message::PlayPausePressed => {
            app.autoplay_pending = false;
            if let Some(player) = &app.player {
                let playing = app
                    .snapshot
                    .as_ref()
                    .map(|s| s.state == PlayerState::Playing)
                    .unwrap_or(false);
                let result = if playing { player.pause() } else { player.play() };
                if let Err(error) = result {
                    app.player_error = Some(error.to_string());
                }
            }
        }
        Message::PreviousPressed => {
            app.autoplay_pending = false;
            if let Some(player) = &app.player {
                if let Err(error) = player.previous() {
                    app.player_error = Some(error.to_string());
                }
            }
        }
        Message::NextPressed => {
            app.autoplay_pending = false;
            if let Some(player) = &app.player {
                if let Err(error) = player.next() {
                    app.player_error = Some(error.to_string());
                }
            }
        }
        Message::ShufflePressed => {
            app.autoplay_pending = false;
            app.settings.shuffle = !app.settings.shuffle;
            save(app);
            if let Some(player) = &app.player {
                if let Err(error) = player.set_shuffle(app.settings.shuffle) {
                    app.player_error = Some(error.to_string());
                }
            }
        }
        Message::RepeatPressed => {
            app.autoplay_pending = false;
            app.settings.repeat = !app.settings.repeat;
            save(app);
            if let Some(player) = &app.player {
                if let Err(error) = player.set_repeat(settings_repeat(app.settings.repeat)) {
                    app.player_error = Some(error.to_string());
                }
            }
        }
        Message::SeekChanged(value) => {
            app.seek_drag = Some(value.clamp(0.0, 1.0));
        }
        Message::SeekReleased => {
            app.autoplay_pending = false;
            if let (Some(fraction), Some(player)) = (app.seek_drag, &app.player) {
                let duration_ms = app
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.duration_ms)
                    .unwrap_or(0);
                if duration_ms > 0 {
                    let position_ms = (duration_ms as f64 * fraction as f64) as u64;
                    if let Err(error) = player.seek(position_ms) {
                        app.player_error = Some(error.to_string());
                        app.seek_drag = None;
                    }
                } else {
                    app.seek_drag = None;
                }
            } else {
                app.seek_drag = None;
            }
        }

        Message::MenuPressed => {
            app.dialog = Some(Dialog::ChangeIp(ChangeIpState::new(
                app.settings.server_address.clone(),
            )));
            return refresh_output_devices();
        }
        Message::EqualizerPressed => {
            app.dialog = Some(Dialog::Equalizer(EqualizerState {
                gains_db: eq_gains(app),
            }));
        }
        Message::DialogDismiss => {
            app.dialog = None;
            app.pressed_keys.clear();
        }
        Message::ColorPickerOpen => {
            app.dialog = Some(Dialog::ColorPicker(ColorPickerState::new(
                app.settings.theme_color,
            )));
        }
        Message::ColorPickerCancel => {
            if let Some(Dialog::ColorPicker(state)) = &app.dialog {
                app.settings.theme_color = state.original;
                save(app);
            }
            app.dialog = Some(Dialog::ChangeIp(ChangeIpState::new(
                app.settings.server_address.clone(),
            )));
        }
        Message::ColorPickerApply => {
            app.dialog = Some(Dialog::ChangeIp(ChangeIpState::new(
                app.settings.server_address.clone(),
            )));
        }
        Message::ColorPickerSvChanged(s, v) => {
            if let Some(Dialog::ColorPicker(state)) = &mut app.dialog {
                state.hsv.s = s.clamp(0.0, 1.0);
                state.hsv.v = v.clamp(0.0, 1.0);
                state.dragging_sv = true;
                app.settings.theme_color = state.hsv.to_theme();
                save(app);
            }
        }
        Message::ColorPickerHueChanged(h) => {
            if let Some(Dialog::ColorPicker(state)) = &mut app.dialog {
                state.hsv.h = h.rem_euclid(360.0);
                state.dragging_hue = true;
                app.settings.theme_color = state.hsv.to_theme();
                save(app);
            }
        }
        Message::ColorPickerDragEnd => {
            if let Some(Dialog::ColorPicker(state)) = &mut app.dialog {
                state.dragging_sv = false;
                state.dragging_hue = false;
            }
        }

        Message::IpInputChanged(value) => {
            if let Some(Dialog::ChangeIp(state)) = &mut app.dialog {
                state.ip_input = value;
            }
        }
        Message::SwitchLocalToggled => {
            app.settings.is_local = !app.settings.is_local;
            save(app);
            app.sync_source();
        }
        Message::ApplyServerPressed => {
            if let Some(Dialog::ChangeIp(state)) = &app.dialog {
                app.settings.server_address = state.ip_input.clone();
                save(app);
                if !app.settings.is_local {
                    app.sync_source();
                }
            }
        }
        Message::ShowVisualizerToggled(value) => {
            app.settings.show_visualizer = value;
            save(app);
        }
        Message::ShowCoverToggled(value) => {
            app.settings.show_cover = value;
            save(app);
        }
        Message::ShowCoverImageToggled(value) => {
            app.settings.show_cover_image = value;
            save(app);
        }
        Message::VisualizerRingToggled(value) => {
            app.settings.visualizer_ring = value;
            save(app);
        }
        Message::CoverSmallToggled(value) => {
            app.settings.cover_small = value;
            save(app);
        }
        Message::AutoplayToggled(value) => {
            app.settings.autoplay = value;
            save(app);
            app.autoplay_pending = value;
            if value {
                if let Some(task) = maybe_autoplay(app) {
                    return task;
                }
            }
        }

        Message::CoverLoaded { index, bytes } => {
            if app.cover_loading == Some(index) {
                app.cover_loading = None;
                app.cover_for_index = Some(index);
                app.cover = bytes.map(iced::widget::image::Handle::from_bytes);
            }
        }

        Message::SoundRefreshRequested => {
            return refresh_output_devices();
        }
        Message::OutputDevicesLoaded(result) => match result {
            Ok(devices) => {
                app.output_devices = devices;
                app.sound_error = None;
            }
            Err(error) => {
                app.sound_error = Some(error);
            }
        },
        Message::OutputDevicePicked(name) => {
            app.settings.output_device = name.clone();
            save(app);
            let player = app.player.clone();
            return Task::perform(
                async move { set_output_device_task(player, name).await },
                Message::OutputDeviceApplied,
            );
        }
        Message::OutputDeviceApplied(result) => {
            if let Err(error) = result {
                app.sound_error = Some(error);
            } else {
                app.sound_error = None;
            }
        }
        Message::VolumeChanged(value) => {
            let value = value.clamp(0.0, 1.0);
            app.volume_drag = Some(value);
            if let Some(player) = &app.player {
                player.set_volume(value);
            }
        }
        Message::VolumeReleased => {
            if let Some(value) = app.volume_drag.take() {
                app.settings.volume = value;
                save(app);
                if let Some(player) = &app.player {
                    player.set_volume(value);
                }
            }
        }

        Message::EqualizerBandChanged(index, value) => {
            if let Some(Dialog::Equalizer(state)) = &mut app.dialog {
                if let Some(gain) = state.gains_db.get_mut(index) {
                    *gain = value;
                }
            }
        }
        Message::EqualizerBandReleased(_) => {
            if let Some(Dialog::Equalizer(state)) = &app.dialog {
                let gains = state.gains_db.clone();
                app.settings.equalizer = gains.clone();
                save(app);
                if let Some(player) = &app.player {
                    let _ = player.set_equalizer(gains);
                }
            }
        }
        Message::EqualizerResetPressed => {
            if let Some(Dialog::Equalizer(state)) = &mut app.dialog {
                state.gains_db.fill(0.0);
                let gains = state.gains_db.clone();
                app.settings.equalizer = gains.clone();
                save(app);
                if let Some(player) = &app.player {
                    let _ = player.set_equalizer(gains);
                }
            }
        }

        Message::FolderPickerOpen => {
            let start = folder_start(app);
            app.dialog = Some(Dialog::FolderPicker(FolderPickerState::loading(start.clone())));
            return load_folder(start);
        }
        Message::FolderPickerUp => {
            if let Some(Dialog::FolderPicker(state)) = &app.dialog {
                let parent = state
                    .current
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| state.current.clone());
                app.dialog = Some(Dialog::FolderPicker(FolderPickerState::loading(parent.clone())));
                return load_folder(parent);
            }
        }
        Message::FolderPickerEnter(path) => {
            app.dialog = Some(Dialog::FolderPicker(FolderPickerState::loading(path.clone())));
            return load_folder(path);
        }
        Message::FolderPickerLoaded { path, entries } => {
            if let Some(Dialog::FolderPicker(state)) = &mut app.dialog {
                if state.current == path {
                    state.loading = false;
                    match entries {
                        Ok(items) => {
                            state.entries = items;
                            state.error = None;
                        }
                        Err(error) => {
                            state.entries.clear();
                            state.error = Some(error);
                        }
                    }
                }
            }
        }
        Message::FolderPickerApply => {
            if let Some(Dialog::FolderPicker(state)) = &app.dialog {
                app.settings.folder_address = state.current.to_string_lossy().into_owned();
                save(app);
                restart_watcher(app);
            }
            app.dialog = None;
        }

        Message::HotkeyCaptureOpen(action) => {
            app.dialog = Some(Dialog::HotkeysCapture(HotkeysCaptureState { action }));
        }
        Message::HotkeyCaptured(binding) => {
            if let Some(Dialog::HotkeysCapture(state)) = &app.dialog {
                let action = state.action;
                if let Some(binding) = binding {
                    app.settings.hotkeys.insert(action, binding);
                } else {
                    app.settings.hotkeys.remove(&action);
                }
                save(app);
                sync_global_hotkeys(app);
            }
            app.dialog = Some(Dialog::ChangeIp(ChangeIpState::new(
                app.settings.server_address.clone(),
            )));
            app.pressed_keys.clear();
        }
        Message::HotkeyGlobalToggled(value) => {
            app.settings.hotkeys_global = value;
            save(app);
            if value && hotkeys::wayland_session() {
                app.global_hotkeys_state = GlobalHotkeysState::Unavailable(
                    hotkeys::wayland::unavailable_reason(),
                );
            } else {
                sync_global_hotkeys(app);
            }
        }

        Message::KeyPressed {
            physical_key,
            modifiers,
        } => {
            return handle_key_press(app, physical_key, modifiers);
        }
        Message::KeyReleased => {
            app.pressed_keys.clear();
        }
        Message::WindowUnfocused => {
            app.pressed_keys.clear();
        }
        Message::GlobalHotkey(notice) => match notice {
            GlobalHotkeyNotice::Status(state) => {
                app.global_hotkeys_state = state;
            }
            GlobalHotkeyNotice::Action(action) => {
                return perform_action(app, action);
            }
        },

        Message::Network(notice) => match notice {
            NetworkNotice::Status {
                generation,
                address,
                status,
                error,
            } => {
                if generation != app.network_generation || app.settings.is_local || address != app.settings.server_address.trim() {
                    return Task::none();
                }
                app.network_status = status.clone();
                app.network_error = error.clone();
                if matches!(status, NetworkStatus::Connected) {
                    let session = app.network.session_for(&address);
                    if let Some(player) = &app.player {
                        if let Err(err) = player.set_session(session) {
                            app.player_error = Some(err.to_string());
                        }
                    }
                } else {
                    if let Some(player) = &app.player {
                        let _ = player.set_session(None);
                    }
                    if !app.tracks.is_empty() {
                        set_playlist(app, Vec::new(), error.clone());
                    } else {
                        app.playlist_error = error;
                    }
                }
            }
            NetworkNotice::Playlist {
                generation,
                address,
                tracks,
            } => {
                if generation != app.network_generation || app.settings.is_local || address != app.settings.server_address.trim() {
                    return Task::none();
                }
                log::info!("autoplay: network playlist len={} pending={} player={} track={:?}", tracks.len(), app.autoplay_pending, app.player.is_some(), app.snapshot.as_ref().and_then(|s| s.track_index));
                set_playlist(app, tracks, None);
                return maybe_autoplay(app).unwrap_or_else(|| maybe_load_cover(app));
            }
            NetworkNotice::SseRefresh {
                generation,
                address,
            } => {
                if generation != app.network_generation || app.settings.is_local || address != app.settings.server_address.trim() {
                    return Task::none();
                }
                app.network.refresh_playlist();
            }
        },

        Message::CloseRequested(id) => {
            app.global_hotkeys.stop();
            app.network.stop();
            let watcher = app.playback.take_watcher();
            let player = app.player.take();
            thread::spawn(move || {
                if let Some(watcher) = watcher {
                    watcher.stop();
                }
                if let Some(player) = player {
                    let _ = player.shutdown();
                }
            });
            return Task::batch([iced::window::close(id), exit()]);
        }
    }
    Task::none()
}

fn handle_key_press(
    app: &mut App,
    physical_key: Physical,
    modifiers: iced::keyboard::Modifiers,
) -> Task<Message> {
    if matches!(&app.dialog, Some(Dialog::HotkeysCapture(_))) {
        if physical_key == Physical::Code(Code::Escape) {
            return Task::done(Message::DialogDismiss);
        }
        if let Some(binding) = binding_from_iced(physical_key, modifiers) {
            return Task::done(Message::HotkeyCaptured(Some(binding)));
        }
        return Task::none();
    }

    if app.search_active {
        return Task::none();
    }

    let Some(binding) = binding_from_iced(physical_key, modifiers) else {
        return Task::none();
    };

    let action = app
        .settings
        .hotkeys
        .iter()
        .find_map(|(action, bound)| if *bound == binding { Some(*action) } else { None });
    let Some(action) = action else {
        return Task::none();
    };

    if app.pressed_keys.contains(&binding.key) && !action.repeats() {
        return Task::none();
    }
    app.pressed_keys.insert(binding.key);
    perform_action(app, action)
}

fn perform_action(app: &mut App, action: HotkeyAction) -> Task<Message> {
    app.autoplay_pending = false;
    match action {
        HotkeyAction::Shuffle => {
            app.settings.shuffle = !app.settings.shuffle;
            save(app);
            if let Some(player) = &app.player {
                if let Err(error) = player.set_shuffle(app.settings.shuffle) {
                    app.player_error = Some(error.to_string());
                }
            }
            Task::none()
        }
        HotkeyAction::Previous => {
            if let Some(player) = &app.player {
                if let Err(error) = player.previous() {
                    app.player_error = Some(error.to_string());
                }
            }
            Task::none()
        }
        HotkeyAction::PlayPause => {
            if let Some(player) = &app.player {
                let playing = app
                    .snapshot
                    .as_ref()
                    .map(|s| s.state == PlayerState::Playing)
                    .unwrap_or(false);
                let result = if playing { player.pause() } else { player.play() };
                if let Err(error) = result {
                    app.player_error = Some(error.to_string());
                }
            }
            Task::none()
        }
        HotkeyAction::Next => {
            if let Some(player) = &app.player {
                if let Err(error) = player.next() {
                    app.player_error = Some(error.to_string());
                }
            }
            Task::none()
        }
        HotkeyAction::Repeat => {
            app.settings.repeat = !app.settings.repeat;
            save(app);
            if let Some(player) = &app.player {
                if let Err(error) = player.set_repeat(settings_repeat(app.settings.repeat)) {
                    app.player_error = Some(error.to_string());
                }
            }
            Task::none()
        }
        HotkeyAction::Rewind => {
            let delta = 5_000i64;
            if let (Some(player), Some(snapshot)) = (&app.player, &app.snapshot) {
                let pos = snapshot.position_ms as i64;
                let target = (pos - delta).max(0) as u64;
                if let Err(error) = player.seek(target) {
                    app.player_error = Some(error.to_string());
                }
            }
            Task::none()
        }
        HotkeyAction::Forward => {
            let delta = 5_000u64;
            if let (Some(player), Some(snapshot)) = (&app.player, &app.snapshot) {
                let dur = snapshot.duration_ms.unwrap_or(u64::MAX);
                let target = (snapshot.position_ms + delta).min(dur);
                if let Err(error) = player.seek(target) {
                    app.player_error = Some(error.to_string());
                }
            }
            Task::none()
        }
        HotkeyAction::VolumeDown => {
            let next = (app.settings.volume - 0.05).max(0.0);
            app.settings.volume = next;
            save(app);
            if let Some(player) = &app.player {
                player.set_volume(next);
            }
            Task::none()
        }
        HotkeyAction::VolumeUp => {
            let next = (app.settings.volume + 0.05).min(1.0);
            app.settings.volume = next;
            save(app);
            if let Some(player) = &app.player {
                player.set_volume(next);
            }
            Task::none()
        }
    }
}

fn apply_queue(app: &mut App) {
    let Some(player) = app.player.as_ref() else {
        return;
    };
    let result = player.set_queue(app.tracks.clone(), None);
    if let Err(error) = result {
        app.player_error = Some(error.to_string());
    }
}

fn set_playlist(app: &mut App, tracks: Vec<player_core::TrackData>, error: Option<String>) {
    app.tracks = tracks;
    app.playlist_error = error;
    app.selected_track = None;
    app.cover = None;
    app.cover_for_index = None;
    app.cover_loading = None;
    app.smoothed_bands.clear();
    apply_queue(app);
    app.refresh_snapshot();
}

fn maybe_autoplay(app: &mut App) -> Option<Task<Message>> {
    if !app.autoplay_pending || app.tracks.is_empty() {
        log::info!("autoplay: skip pending={} tracks={}", app.autoplay_pending, app.tracks.len());
        return None;
    }
    let Some(player) = app.player.as_ref() else {
        log::info!("autoplay: skip no player");
        return None;
    };
    let track_index = app.snapshot.as_ref().and_then(|s| s.track_index);
    if track_index.is_some() {
        log::info!("autoplay: skip already playing track={track_index:?}");
        app.autoplay_pending = false;
        return None;
    }
    let index = random_index(app.tracks.len())? as u32;
    log::info!("autoplay: start track {index}/{}", app.tracks.len());
    app.autoplay_pending = false;
    if let Err(error) = player.play_track(index) {
        log::info!("autoplay: play_track failed: {error}");
        app.player_error = Some(error.to_string());
        return None;
    }
    Some(maybe_load_cover(app))
}

fn random_index(len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0);
    Some((nanos ^ (std::process::id() as usize)) % len)
}

fn settings_repeat(on: bool) -> RepeatMode {
    if on {
        RepeatMode::One
    } else {
        RepeatMode::Off
    }
}

fn save(app: &App) {
    if let Err(error) = settings::save(&app.settings) {
        eprintln!("musicwifi: {error}");
    }
}

fn sync_global_hotkeys(app: &mut App) {
    if app.settings.hotkeys_global && hotkeys::wayland_session() {
        app.global_hotkeys_state = GlobalHotkeysState::Unavailable(
            hotkeys::wayland::unavailable_reason(),
        );
        return;
    }
    app.global_hotkeys
        .apply(app.settings.hotkeys_global, &app.settings.hotkeys);
}

fn eq_gains(app: &App) -> Vec<f32> {
    let len = app.eq_bands_hz.len().max(1);
    let mut gains = app.settings.equalizer.clone();
    gains.resize(len, 0.0);
    gains.truncate(len);
    gains
}

fn refresh_output_devices() -> Task<Message> {
    Task::perform(
        async { list_output_devices() },
        Message::OutputDevicesLoaded,
    )
}

fn list_output_devices() -> Result<Vec<String>, String> {
    player_core::output_devices().map_err(|e| e.to_string())
}

async fn set_output_device_task(
    player: Option<std::sync::Arc<player_core::Player>>,
    name: String,
) -> Result<(), String> {
    let Some(player) = player else {
        return Err("плеер не готов".to_owned());
    };
    let device = if name.trim().is_empty() { None } else { Some(name) };
    player.set_output_device(device).map_err(|e| e.to_string())
}

fn folder_start(app: &App) -> PathBuf {
    let current = if app.settings.folder_address.trim().is_empty() {
        std::env::var_os("HOME").map(PathBuf::from)
    } else {
        Some(PathBuf::from(&app.settings.folder_address))
    };
    current.unwrap_or_else(|| PathBuf::from("/"))
}

fn load_folder(path: PathBuf) -> Task<Message> {
    Task::perform(
        async move { tokio_blocking_read_folder(path).await },
        |(path, entries)| Message::FolderPickerLoaded { path, entries },
    )
}

async fn tokio_blocking_read_folder(path: PathBuf) -> (PathBuf, Result<Vec<PathBuf>, String>) {
    let p = path.clone();
    let result = std::thread::spawn(move || read_folder_sync(p)).join();
    match result {
        Ok(value) => value,
        Err(_) => (path, Err("чтение папки прервано".to_owned())),
    }
}

fn read_folder_sync(path: PathBuf) -> (PathBuf, Result<Vec<PathBuf>, String>) {
    let result = std::fs::read_dir(&path)
        .map(|read| {
            let mut dirs = Vec::new();
            for entry in read.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                        if !name.starts_with('.') {
                            dirs.push(p);
                        }
                    }
                }
            }
            dirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
            dirs
        })
        .map_err(|e| e.to_string());
    (path, result)
}

fn restart_watcher(app: &mut App) {
    if !app.settings.is_local {
        return;
    }
    app.playback.set_folder(app.settings.folder_address.clone());
    app.tracks.clear();
    app.selected_track = None;
    app.cover = None;
    app.cover_for_index = None;
    app.cover_loading = None;
    app.smoothed_bands.clear();
    if let Some(player) = &app.player {
        let _ = player.set_queue(Vec::new(), None);
    }
}

fn maybe_load_cover(app: &mut App) -> Task<Message> {
    if !app.settings.show_cover_image {
        return Task::none();
    }
    let Some(index) = app.current_track_index() else {
        return Task::none();
    };
    if app.cover_for_index == Some(index) || app.cover_loading == Some(index) {
        return Task::none();
    }
    let Some(track) = app.tracks.get(index) else {
        return Task::none();
    };
    let uri = track.track_url.clone();
    app.cover_loading = Some(index);
    Task::perform(
        async move {
            std::thread::spawn(move || player_core::track_cover(uri).unwrap_or(None))
                .join()
                .unwrap_or(None)
        },
        move |bytes| Message::CoverLoaded { index, bytes },
    )
}
