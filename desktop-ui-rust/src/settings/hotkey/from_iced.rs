use iced::keyboard::key::Code;
use iced::keyboard::{Modifiers, key::Physical};

use super::binding::HotkeyBinding;
use super::physical::PhysicalKeyId;

pub fn binding_from_iced(physical: Physical, modifiers: Modifiers) -> Option<HotkeyBinding> {
    if is_modifier_code(physical) {
        return None;
    }
    let key = PhysicalKeyId::from_physical(physical)?;
    Some(HotkeyBinding::from_physical(key, modifiers))
}

fn is_modifier_code(physical: Physical) -> bool {
    matches!(
        physical,
        Physical::Code(
            Code::ShiftLeft
                | Code::ShiftRight
                | Code::ControlLeft
                | Code::ControlRight
                | Code::AltLeft
                | Code::AltRight
                | Code::SuperLeft
                | Code::SuperRight
                | Code::CapsLock
                | Code::NumLock
                | Code::ScrollLock
        )
    )
}
