use serde::{Deserialize, Serialize};

const MODE_SETTINGS: u8 = 1;
const POMODORO_SETTINGS: u8 = 3;
const PREFERENCES_SETTINGS: u8 = 2;
const STATS_SETTINGS: u8 = 3;
#[derive(Debug, Clone, Deserialize, Serialize, Default, Copy, PartialEq)]
pub enum SettingsTabs {
    #[default]
    Mode,
    Pomodoro,
    Preferences,
    Stats,
}
impl SettingsTabs {
    pub fn amount(&self) -> u8 {
        match self {
            Self::Mode => MODE_SETTINGS,
            Self::Pomodoro => POMODORO_SETTINGS,
            Self::Preferences => PREFERENCES_SETTINGS,
            Self::Stats => STATS_SETTINGS,
        }
    }
    pub fn down(self, pomodoro_off: bool) -> Self {
        match self {
            Self::Mode if pomodoro_off => Self::Preferences,
            Self::Mode => Self::Pomodoro,
            Self::Pomodoro => Self::Preferences,
            Self::Preferences => Self::Preferences,
            Self::Stats => Self::Preferences,
        }
    }
    pub fn up(self, pomodoro_off: bool) -> Self {
        match self {
            Self::Mode => Self::Mode,
            Self::Pomodoro => Self::Mode,
            Self::Preferences if pomodoro_off => Self::Mode,
            Self::Preferences => Self::Pomodoro,
            Self::Stats => Self::Stats,
        }
    }
    pub fn right(self) -> Self {
        Self::Stats
    }
    pub fn left(self) -> Self {
        match self {
            Self::Mode => Self::Mode,
            Self::Pomodoro => Self::Pomodoro,
            Self::Preferences => Self::Preferences,
            Self::Stats => Self::Mode,
        }
    }
}
