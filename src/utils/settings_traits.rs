use crate::{
    settings::{PomodoroSettings, TimerSettings},
    timer::helper_structs::PomodoroState,
    DEFAULT_BREAK, DEFAULT_ITERATIONS, DEFAULT_WORK,
};

impl Default for TimerSettings {
    fn default() -> Self {
        TimerSettings {
            work_time: DEFAULT_WORK,
            break_time: DEFAULT_BREAK,
            iterations: DEFAULT_ITERATIONS,
        }
    }
}
impl From<PomodoroState> for PomodoroSettings {
    fn from(value: PomodoroState) -> Self {
        match value {
            PomodoroState::Work(time) => PomodoroSettings::WorkTime(time),
            PomodoroState::Break(time) => PomodoroSettings::BreakTime(time),
        }
    }
}
impl From<u8> for PomodoroSettings {
    fn from(value: u8) -> Self {
        PomodoroSettings::Iterations(value)
    }
}
