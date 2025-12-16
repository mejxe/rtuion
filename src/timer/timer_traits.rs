use std::{cell::RefCell, fmt::Display, rc::Rc, time};

use crate::{
    settings::{PomodoroSettings, SettingsTab, TimerSettings},
    DEFAULT_BREAK, DEFAULT_ITERATIONS, DEFAULT_WORK,
};

use super::{countdown::Countdown, helper_structs::PomodoroState, timer::Timer};

impl From<&PomodoroState> for PomodoroSettings {
    fn from(val: &PomodoroState) -> Self {
        match val {
            PomodoroState::Break(time) => PomodoroSettings::BreakTime(*time),
            PomodoroState::Work(time) => PomodoroSettings::BreakTime(*time),
        }
    }
}
impl From<PomodoroSettings> for PomodoroState {
    fn from(value: PomodoroSettings) -> Self {
        match value {
            PomodoroSettings::WorkTime(time) => PomodoroState::Work(time),
            PomodoroSettings::BreakTime(time) => PomodoroState::Break(time),
            _ => PomodoroState::Break(-1),
        }
    }
}

impl Display for PomodoroState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PomodoroState::Work(_) => {
                write!(f, "Work")
            }
            PomodoroState::Break(_) => {
                write!(f, "Break")
            }
        }
    }
}
impl Default for Timer {
    fn default() -> Self {
        let work_state = PomodoroState::Work(DEFAULT_WORK);
        let break_state = PomodoroState::Break(DEFAULT_BREAK);
        let total_iterations = DEFAULT_ITERATIONS;
        let time_left = Timer::get_duration(&work_state);
        let total_time: i64 = time_left * total_iterations as i64;
        let settings = Rc::new(RefCell::new(SettingsTab::default()));
        Timer::new(
            time_left,
            total_iterations,
            total_time,
            work_state,
            break_state,
            settings,
        )
    }
}
impl Default for Countdown {
    fn default() -> Self {
        Countdown::new(DEFAULT_WORK, PomodoroState::Work(DEFAULT_WORK), false)
    }
}
impl From<TimerSettings> for Timer {
    fn from(value: TimerSettings) -> Self {
        let work_state = PomodoroState::Work(value.work_time);
        let break_state = PomodoroState::Break(value.break_time);
        let total_iterations = value.iterations;
        let time_left = Timer::get_duration(&work_state);
        let total_time: i64 = time_left * total_iterations as i64;
        let settings = Rc::new(RefCell::new(SettingsTab::default()));

        let mut timer = Timer::new(
            time_left,
            total_iterations,
            total_time,
            work_state,
            break_state,
            settings,
        );
        timer.set_total_time();
        timer
    }
}

impl From<TimerSettings> for Countdown {
    fn from(value: TimerSettings) -> Self {
        let state = PomodoroState::Work(value.work_time);
        let running = false;
        let time_left = Timer::get_duration(&state);
        Countdown::new(time_left, state, running)
    }
}
