use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyAction {
    Shuffle,
    Previous,
    PlayPause,
    Next,
    Repeat,
    Rewind,
    Forward,
    VolumeDown,
    VolumeUp,
}

impl HotkeyAction {
    pub const ALL: [Self; 9] = [
        Self::Shuffle,
        Self::Previous,
        Self::PlayPause,
        Self::Next,
        Self::Repeat,
        Self::Rewind,
        Self::Forward,
        Self::VolumeDown,
        Self::VolumeUp,
    ];

    pub fn repeats(self) -> bool {
        matches!(
            self,
            Self::Rewind | Self::Forward | Self::VolumeDown | Self::VolumeUp
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_continuous_actions_repeat() {
        assert!(HotkeyAction::Rewind.repeats());
        assert!(HotkeyAction::Forward.repeats());
        assert!(HotkeyAction::VolumeDown.repeats());
        assert!(HotkeyAction::VolumeUp.repeats());
        assert!(!HotkeyAction::PlayPause.repeats());
        assert!(!HotkeyAction::Next.repeats());
    }

    #[test]
    fn all_contains_every_action_once() {
        assert_eq!(HotkeyAction::ALL.len(), 9);

        for action in HotkeyAction::ALL {
            assert_eq!(
                HotkeyAction::ALL
                    .iter()
                    .filter(|candidate| **candidate == action)
                    .count(),
                1
            );
        }
    }
}
