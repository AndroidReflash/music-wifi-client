use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use iced::keyboard;
use iced::widget::scrollable;
use iced::window;
use player_core::{Player, PlayerError, PlayerEvent, TrackData};

use crate::settings::{HotkeyAction, HotkeyBinding};
use crate::platform::hotkeys::x11::GlobalHotkeyNotice;
use crate::repository::NetworkNotice;

use super::player::PlayerNotice;

#[derive(Clone)]
pub enum Message {
    PlayerReady(Arc<Player>),
    PlayerFailed(PlayerError),
    PlayerEvent(PlayerEvent),
    Bands(Vec<f64>),
    Playlist(Vec<TrackData>),
    PlaylistFailed(String),
    Tick,
    ListScrolled(scrollable::Viewport),
    ListScrollTo(f32),
    SearchToggled,
    SearchQueryChanged(String),
    TrackPressed(usize),
    TitlePressed,
    PlayPausePressed,
    PreviousPressed,
    NextPressed,
    ShufflePressed,
    RepeatPressed,
    SeekChanged(f32),
    SeekReleased,
    MenuPressed,
    EqualizerPressed,
    DialogDismiss,
    ColorPickerOpen,
    ColorPickerCancel,
    ColorPickerApply,
    ColorPickerSvChanged(f32, f32),
    ColorPickerHueChanged(f32),
    ColorPickerDragEnd,

    IpInputChanged(String),
    SwitchLocalToggled,
    ApplyServerPressed,
    ShowVisualizerToggled(bool),
    ShowCoverToggled(bool),
    ShowCoverImageToggled(bool),
    VisualizerRingToggled(bool),
    CoverSmallToggled(bool),
    AutoplayToggled(bool),

    CoverLoaded {
        index: usize,
        rgba: Option<(u32, u32, Vec<u8>)>,
    },

    SoundRefreshRequested,
    OutputDevicesLoaded(Result<Vec<String>, String>),
    OutputDevicePicked(String),
    OutputDeviceApplied(Result<(), String>),
    VolumeChanged(f32),
    VolumeReleased,

    EqualizerBandChanged(usize, f32),
    EqualizerBandReleased(usize),
    EqualizerResetPressed,

    FolderPickerOpen,
    FolderPickerUp,
    FolderPickerEnter(PathBuf),
    FolderPickerLoaded { path: PathBuf, entries: Result<Vec<PathBuf>, String> },
    FolderPickerApply,

    HotkeyCaptureOpen(HotkeyAction),
    HotkeyCaptured(Option<HotkeyBinding>),
    HotkeyGlobalToggled(bool),

    KeyPressed {
        physical_key: keyboard::key::Physical,
        modifiers: keyboard::Modifiers,
    },
    KeyReleased,
    WindowUnfocused,
    GlobalHotkey(GlobalHotkeyNotice),

    Network(NetworkNotice),

    CloseRequested(window::Id),
}

impl fmt::Debug for Message {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlayerReady(_) => formatter.write_str("PlayerReady"),
            Self::PlayerFailed(error) => formatter.debug_tuple("PlayerFailed").field(error).finish(),
            Self::PlayerEvent(event) => formatter.debug_tuple("PlayerEvent").field(event).finish(),
            Self::Bands(bands) => formatter.debug_tuple("Bands").field(bands).finish(),
            Self::Playlist(tracks) => formatter.debug_tuple("Playlist").field(tracks).finish(),
            Self::PlaylistFailed(reason) => {
                formatter.debug_tuple("PlaylistFailed").field(reason).finish()
            }
            Self::Tick => formatter.write_str("Tick"),
            Self::ListScrolled(_) => formatter.write_str("ListScrolled"),
            Self::ListScrollTo(v) => formatter.debug_tuple("ListScrollTo").field(v).finish(),
            Self::SearchToggled => formatter.write_str("SearchToggled"),
            Self::SearchQueryChanged(v) => formatter.debug_tuple("SearchQueryChanged").field(v).finish(),
            Self::TrackPressed(i) => formatter.debug_tuple("TrackPressed").field(i).finish(),
            Self::TitlePressed => formatter.write_str("TitlePressed"),
            Self::PlayPausePressed => formatter.write_str("PlayPausePressed"),
            Self::PreviousPressed => formatter.write_str("PreviousPressed"),
            Self::NextPressed => formatter.write_str("NextPressed"),
            Self::ShufflePressed => formatter.write_str("ShufflePressed"),
            Self::RepeatPressed => formatter.write_str("RepeatPressed"),
            Self::SeekChanged(v) => formatter.debug_tuple("SeekChanged").field(v).finish(),
            Self::SeekReleased => formatter.write_str("SeekReleased"),
            Self::MenuPressed => formatter.write_str("MenuPressed"),
            Self::EqualizerPressed => formatter.write_str("EqualizerPressed"),
            Self::DialogDismiss => formatter.write_str("DialogDismiss"),
            Self::ColorPickerOpen => formatter.write_str("ColorPickerOpen"),
            Self::ColorPickerCancel => formatter.write_str("ColorPickerCancel"),
            Self::ColorPickerApply => formatter.write_str("ColorPickerApply"),
            Self::ColorPickerSvChanged(s, v) => formatter.debug_tuple("ColorPickerSvChanged").field(s).field(v).finish(),
            Self::ColorPickerHueChanged(h) => formatter.debug_tuple("ColorPickerHueChanged").field(h).finish(),
            Self::ColorPickerDragEnd => formatter.write_str("ColorPickerDragEnd"),
            Self::IpInputChanged(v) => formatter.debug_tuple("IpInputChanged").field(v).finish(),
            Self::SwitchLocalToggled => formatter.write_str("SwitchLocalToggled"),
            Self::ApplyServerPressed => formatter.write_str("ApplyServerPressed"),
            Self::ShowVisualizerToggled(v) => formatter.debug_tuple("ShowVisualizerToggled").field(v).finish(),
            Self::ShowCoverToggled(v) => formatter.debug_tuple("ShowCoverToggled").field(v).finish(),
            Self::ShowCoverImageToggled(v) => formatter.debug_tuple("ShowCoverImageToggled").field(v).finish(),
            Self::VisualizerRingToggled(v) => formatter.debug_tuple("VisualizerRingToggled").field(v).finish(),
            Self::CoverSmallToggled(v) => formatter.debug_tuple("CoverSmallToggled").field(v).finish(),
            Self::AutoplayToggled(v) => formatter.debug_tuple("AutoplayToggled").field(v).finish(),
            Self::CoverLoaded { index, rgba } => formatter
                .debug_struct("CoverLoaded")
                .field("index", index)
                .field("has_pixels", &rgba.is_some())
                .finish(),
            Self::SoundRefreshRequested => formatter.write_str("SoundRefreshRequested"),
            Self::OutputDevicesLoaded(v) => formatter.debug_tuple("OutputDevicesLoaded").field(&v.as_ref().map(|d| d.len()).map_err(|e| e.clone())).finish(),
            Self::OutputDevicePicked(v) => formatter.debug_tuple("OutputDevicePicked").field(v).finish(),
            Self::OutputDeviceApplied(v) => formatter.debug_tuple("OutputDeviceApplied").field(v).finish(),
            Self::VolumeChanged(v) => formatter.debug_tuple("VolumeChanged").field(v).finish(),
            Self::VolumeReleased => formatter.write_str("VolumeReleased"),
            Self::EqualizerBandChanged(i, v) => formatter.debug_tuple("EqualizerBandChanged").field(i).field(v).finish(),
            Self::EqualizerBandReleased(i) => formatter.debug_tuple("EqualizerBandReleased").field(i).finish(),
            Self::EqualizerResetPressed => formatter.write_str("EqualizerResetPressed"),
            Self::FolderPickerOpen => formatter.write_str("FolderPickerOpen"),
            Self::FolderPickerUp => formatter.write_str("FolderPickerUp"),
            Self::FolderPickerEnter(p) => formatter.debug_tuple("FolderPickerEnter").field(p).finish(),
            Self::FolderPickerLoaded { path, .. } => formatter.debug_tuple("FolderPickerLoaded").field(path).finish(),
            Self::FolderPickerApply => formatter.write_str("FolderPickerApply"),
            Self::HotkeyCaptureOpen(a) => formatter.debug_tuple("HotkeyCaptureOpen").field(a).finish(),
            Self::HotkeyCaptured(v) => formatter.debug_tuple("HotkeyCaptured").field(v).finish(),
            Self::HotkeyGlobalToggled(v) => formatter.debug_tuple("HotkeyGlobalToggled").field(v).finish(),
            Self::KeyPressed { physical_key, modifiers } => formatter
                .debug_struct("KeyPressed")
                .field("physical_key", physical_key)
                .field("modifiers", modifiers)
                .finish(),
            Self::KeyReleased => formatter.write_str("KeyReleased"),
            Self::WindowUnfocused => formatter.write_str("WindowUnfocused"),
            Self::GlobalHotkey(n) => formatter.debug_tuple("GlobalHotkey").field(n).finish(),
            Self::Network(n) => formatter.debug_tuple("Network").field(n).finish(),
            Self::CloseRequested(id) => formatter.debug_tuple("CloseRequested").field(id).finish(),
        }
    }
}

impl From<PlayerNotice> for Message {
    fn from(notice: PlayerNotice) -> Self {
        match notice {
            PlayerNotice::Ready(player) => Self::PlayerReady(player),
            PlayerNotice::Failed(error) => Self::PlayerFailed(error),
            PlayerNotice::Event(event) => Self::PlayerEvent(event),
            PlayerNotice::Playlist(tracks) => Self::Playlist(tracks),
            PlayerNotice::PlaylistFailed(reason) => Self::PlaylistFailed(reason),
        }
    }
}
