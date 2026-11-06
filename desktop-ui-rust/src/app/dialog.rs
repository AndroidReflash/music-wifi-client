use std::path::PathBuf;

use crate::settings::palette::Hsv;
use crate::settings::{HotkeyAction, ThemeColor};

#[derive(Debug, Clone, PartialEq)]
pub enum Dialog {
    ChangeIp(ChangeIpState),
    Equalizer(EqualizerState),
    FolderPicker(FolderPickerState),
    ColorPicker(ColorPickerState),
    HotkeysCapture(HotkeysCaptureState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeIpState {
    pub ip_input: String,
}

impl ChangeIpState {
    pub fn new(current_ip: String) -> Self {
        Self { ip_input: current_ip }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EqualizerState {
    pub gains_db: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderPickerState {
    pub current: PathBuf,
    pub entries: Vec<PathBuf>,
    pub loading: bool,
    pub error: Option<String>,
}

impl FolderPickerState {
    pub fn loading(current: PathBuf) -> Self {
        Self {
            current,
            entries: Vec::new(),
            loading: true,
            error: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeysCaptureState {
    pub action: HotkeyAction,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorPickerState {
    pub hsv: Hsv,
    pub original: ThemeColor,
    pub dragging_sv: bool,
    pub dragging_hue: bool,
}

impl ColorPickerState {
    pub fn new(color: ThemeColor) -> Self {
        Self {
            hsv: Hsv::from_theme(color),
            original: color,
            dragging_sv: false,
            dragging_hue: false,
        }
    }
}
