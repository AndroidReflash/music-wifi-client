use std::collections::BTreeSet;

use crate::settings::HotkeyAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalHotkeysState {
    Off,
    Unavailable(String),
    Active { rejected: BTreeSet<HotkeyAction> },
}
