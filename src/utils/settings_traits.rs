use crate::{
    settings::{PomodoroSettings, TimerSettings},
    timers::helper_structs::TimerState,
    DEFAULT_BREAK, DEFAULT_ITERATIONS, DEFAULT_WORK,
};

impl Default for TimerSettings {
    fn default() -> Self {
        TimerSettings {
            work_time: DEFAULT_WORK,
            break_time: DEFAULT_BREAK,
            iterations: DEFAULT_ITERATIONS,
            mode: crate::timers::counters::CounterMode::Countdown,
        }
    }
}
impl From<TimerState> for PomodoroSettings {
    fn from(value: TimerState) -> Self {
        match value {
            TimerState::Work(time) => PomodoroSettings::WorkTime(time),
            TimerState::Break(time) => PomodoroSettings::BreakTime(time),
        }
    }
}
impl From<u8> for PomodoroSettings {
    fn from(value: u8) -> Self {
        PomodoroSettings::Iterations(value)
    }
}
