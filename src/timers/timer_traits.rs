use std::{cell::RefCell, fmt::Display, rc::Rc, time};

use futures::stream::Count;

use crate::{
    settings::{PomodoroSettings, Settings, TimerSettings},
    DEFAULT_BREAK, DEFAULT_ITERATIONS, DEFAULT_WORK,
};

use super::{
    counters::{Counter, CounterMode},
    helper_structs::TimerState,
    timer::Timer,
};

impl From<&TimerState> for PomodoroSettings {
    fn from(val: &TimerState) -> Self {
        match val {
            TimerState::Break(time) => PomodoroSettings::BreakTime(*time),
            TimerState::Work(time) => PomodoroSettings::BreakTime(*time),
        }
    }
}
impl From<PomodoroSettings> for TimerState {
    fn from(value: PomodoroSettings) -> Self {
        match value {
            PomodoroSettings::WorkTime(time) => TimerState::Work(time),
            PomodoroSettings::BreakTime(time) => TimerState::Break(time),
            _ => TimerState::Break(-1),
        }
    }
}

impl Display for TimerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimerState::Work(_) => {
                write!(f, "Work")
            }
            TimerState::Break(_) => {
                write!(f, "Break")
            }
        }
    }
}
impl Default for Timer {
    fn default() -> Self {
        let work_state = TimerState::Work(DEFAULT_WORK);
        let break_state = TimerState::Break(DEFAULT_BREAK);
        let total_iterations = DEFAULT_ITERATIONS;
        let time_left = Timer::get_duration(&work_state);
        let total_time: i64 = time_left * total_iterations as i64;
        let settings = Rc::new(RefCell::new(Settings::default()));
        Timer::new(
            time_left,
            total_iterations,
            total_time,
            work_state,
            break_state,
            settings,
            CounterMode::Countdown,
        )
    }
}
impl Default for Counter {
    fn default() -> Self {
        Counter::new(CounterMode::Countdown, TimerState::Work(DEFAULT_WORK))
    }
}
impl From<TimerSettings> for Timer {
    fn from(value: TimerSettings) -> Self {
        let work_state = TimerState::Work(value.work_time);
        let break_state = TimerState::Break(value.break_time);
        let total_iterations = value.iterations;
        let time_left = match value.mode {
            CounterMode::Countup => 0,
            CounterMode::Countdown => Timer::get_duration(&work_state),
        };
        let total_time: i64 = time_left * total_iterations as i64;
        let settings = Rc::new(RefCell::new(Settings::default()));

        let mut timer = Timer::new(
            time_left,
            total_iterations,
            total_time,
            work_state,
            break_state,
            settings,
            value.mode,
        );
        timer.init_total_time();
        timer
    }
}

impl From<TimerSettings> for Counter {
    fn from(value: TimerSettings) -> Self {
        let state = TimerState::Work(value.work_time);
        let mode = value.mode;
        Counter::new(mode, state)
    }
}
impl From<TimerState> for i64 {
    fn from(value: TimerState) -> Self {
        match value {
            TimerState::Work(time) => time,
            TimerState::Break(time) => time,
        }
    }
}
