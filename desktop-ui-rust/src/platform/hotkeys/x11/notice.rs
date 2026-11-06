use crate::settings::HotkeyAction;

use super::state::GlobalHotkeysState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalHotkeyNotice {
    Status(GlobalHotkeysState),
    Action(HotkeyAction),
}
