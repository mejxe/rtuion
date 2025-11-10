use crate::{
    settings::{*},
    DEFAULT_BREAK, DEFAULT_ITERATIONS, DEFAULT_WORK,
};
use std::{cell::RefCell, fmt::Display, rc::Rc, time::Duration};
use tokio_util::sync::CancellationToken;
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PomodoroState {
    Work(i64),
    Break(i64),
}

#[derive(Debug, Clone)]
struct Countdown {
    time_left: i64,
    state: PomodoroState,
    running: bool,
}
#[derive(Debug, Clone)]
pub struct Timer {
    countdown_running: bool,
    iteration: u8,
    time_left: i64,
    total_iterations: u8,
    total_time: i64,
    total_elapsed: i64,
    work_state: PomodoroState,
    break_state: PomodoroState,
    current_state: PomodoroState,
    next_state: PomodoroState,
    settings: Rc<RefCell<SettingsTab>>,

    pub countdown_command_tx: Option<tokio::sync::mpsc::Sender<TimerCommand>>,
}
pub enum TimerCommand {
    Start,
    NextIteration(PomodoroState),
    Stop,
    Restart(PomodoroState),
}
impl Timer {
    pub fn get_duration(pomodoro_state: &PomodoroState) -> i64 {
        match pomodoro_state {
            PomodoroState::Work(dur) | PomodoroState::Break(dur) => return *dur,
        };
    }
    pub fn set_total_time(&mut self) {
        let duration = Timer::get_duration(&self.work_state);
        self.total_time = duration * self.total_iterations as i64;
    }

    pub async fn run(
        &mut self,
        sender: tokio::sync::mpsc::Sender<i64>,
        command_rx: tokio::sync::mpsc::Receiver<TimerCommand>,
        close: CancellationToken,
    ) {
        let mut countdown = Countdown::from(self.settings.borrow().timer_settings.clone());
        tokio::task::spawn({
            async move {
                countdown.run(sender, command_rx, close).await;
            }
        });
    }
    pub async fn start(&mut self) {
        self.countdown_running = true;
        self.send_countdown_commands(TimerCommand::Start).await;
    }
    pub async fn stop(&mut self) {
        self.countdown_running = false;
        self.send_countdown_commands(TimerCommand::Stop).await;
    }

    pub fn swap_states(&mut self) {
        if !self.countdown_running {
            match self.current_state {
                PomodoroState::Work(_) => {
                    self.current_state = self.break_state;
                    self.next_state = self.work_state;
                }
                PomodoroState::Break(_) => {
                    self.current_state = self.work_state;
                    self.next_state = self.break_state;
                }
            }
            let duration = Timer::get_duration(&self.current_state);
            self.time_left = duration;
        }
    }
    pub async fn next_iteration(&mut self) {
        self.countdown_running = false;
        self.swap_states();
        if let PomodoroState::Work(_) = self.current_state {
            self.iteration += 1;
        }
        self.stop().await;
        self.send_countdown_commands(TimerCommand::NextIteration(self.current_state))
            .await;
        if !self.settings.borrow().ui_settings.pause_after_state_change {
            self.start().await;
        }
    }
    pub async fn send_countdown_commands(&self, command: TimerCommand) {
        if let Some(tx) = &self.countdown_command_tx {
            let _ = tx.send(command).await;
        }
    }

    pub async fn restart(&mut self) {
        self.current_state = self.work_state;
        self.next_state = self.break_state;
        self.iteration = 1;
        self.time_left = Timer::get_duration(&self.work_state);
        self.total_elapsed = 0;
        self.countdown_running = false;
        self.set_total_time();
        self.send_countdown_commands(TimerCommand::Restart(self.work_state))
            .await;
    }
    pub async fn set_setting(&mut self, setting: PomodoroSettings) -> Option<()> {
        if self.get_running() {
            return None;
        };
        match setting {
            PomodoroSettings::Iterations(iterations) => {
                self.set_total_iterations(iterations.unwrap())
            }
            PomodoroSettings::WorkTime(_) => self.set_work_state(PomodoroState::from(setting)),
            PomodoroSettings::BreakTime(_) => self.set_break_state(PomodoroState::from(setting)),
        }
        self.restart().await;
        Some(())
    }
    pub fn get_next_state(&self) -> PomodoroState {
        self.next_state
    }
    pub fn get_timeleft(&self) -> i64 {
        self.time_left
    }
    pub fn get_work_state(&self) -> PomodoroState {
        self.work_state
    }

    pub fn get_break_state(&self) -> PomodoroState {
        self.break_state
    }

    pub fn get_running(&self) -> bool {
        self.countdown_running
    }
    pub fn get_total_iterations(&self) -> u8 {
        self.total_iterations
    }
    pub fn get_iteration(&self) -> u8 {
        self.iteration
    }
    pub fn get_total_time(&self) -> i64 {
        self.total_time
    }
    pub fn get_total_elapsed_time(&self) -> i64 {
        self.total_elapsed
    }
    pub fn get_current_state(&self) -> PomodoroState {
        self.current_state
    }
    pub fn set_running(&mut self, state: bool) {
        self.countdown_running = state;
    }
    pub fn set_elapsed_time(&mut self, elapsed: i64) {
        self.total_elapsed = elapsed;
    }
    pub fn set_time_left(&mut self, time: i64) {
        self.time_left = time;
    }
    pub fn set_work_state(&mut self, work_state: PomodoroState) {
        self.work_state = work_state;
    }
    pub fn set_break_state(&mut self, break_state: PomodoroState) {
        self.break_state = break_state;
    }
    pub fn set_total_iterations(&mut self, total_iterations: u8) {
        self.total_iterations = total_iterations;
    }
    pub fn set_config(&mut self, settings: Rc<RefCell<SettingsTab>>) {
        self.settings = settings;
    }
    pub fn get_timer_started(&self) -> bool {
        self.get_total_elapsed_time() != 0
    }
}
impl Countdown {
    pub async fn run(
        &mut self,
        sender: tokio::sync::mpsc::Sender<i64>,
        mut command_rx: tokio::sync::mpsc::Receiver<TimerCommand>,
        close: CancellationToken,
    ) {
        loop {
            tokio::select! {
            Some(command) = command_rx.recv() => {
                match command {
                    TimerCommand::Start => {self.running = true},
                    TimerCommand::Stop => {self.running = false},
                    TimerCommand::NextIteration(state) => {self.next_iteration(state)},
                    TimerCommand::Restart(state) => self.restart(state),
                };
            }
            _ = tokio::time::sleep(Duration::from_secs(1)), if self.running && self.time_left >= 0 => {
                self.time_left -= 1;
                sender.send(self.time_left).await.unwrap();
            }
            _ = close.cancelled() => {break}
            }
        }
    }
    pub fn next_iteration(&mut self, state: PomodoroState) {
        self.state = state;
        self.time_left = Timer::get_duration(&state);
    }

    pub fn restart(&mut self, state: PomodoroState) {
        self.time_left = Timer::get_duration(&state);
        self.state = state;
        self.running = false;
    }
}

// traits
impl Into<PomodoroSettings> for &PomodoroState {
    fn into(self) -> PomodoroSettings {
        match self {
            PomodoroState::Break(time) => PomodoroSettings::BreakTime(Some(*time)),
            PomodoroState::Work(time) => PomodoroSettings::BreakTime(Some(*time)),
        }
    }
}
impl From<PomodoroSettings> for PomodoroState {
    fn from(value: PomodoroSettings) -> Self {
        match value {
            PomodoroSettings::WorkTime(Some(time)) => PomodoroState::Work(time),
            PomodoroSettings::BreakTime(Some(time)) => PomodoroState::Break(time),
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
        let duration = Timer::get_duration(&work_state);
        let total_time: i64 = duration * total_iterations as i64;
        let settings = Rc::new(RefCell::new(SettingsTab::default()));
        Timer {
            time_left: duration,
            countdown_running: false,
            countdown_command_tx: None,
            total_iterations,
            current_state: work_state,
            next_state: break_state,
            iteration: 1,
            total_time,
            total_elapsed: 0,
            work_state,
            break_state,
            settings,
        }
    }
}
impl Default for Countdown {
    fn default() -> Self {
        Countdown {
            running: false,
            time_left: DEFAULT_WORK,
            state: PomodoroState::Work(DEFAULT_WORK),
        }
    }
}
impl From<TimerSettings> for Timer {
    fn from(value: TimerSettings) -> Self {
        let mut timer = Timer::default();
        timer.work_state = PomodoroState::Work(value.work_time);
        timer.break_state = PomodoroState::Break(value.break_time);
        timer.total_iterations = value.iterations;
        timer.current_state = timer.work_state;
        timer.next_state = timer.break_state;
        timer.time_left = Timer::get_duration(&timer.current_state);
        timer.total_elapsed = 0;
        timer.set_total_time();
        timer
    }
}

impl From<TimerSettings> for Countdown {
    fn from(value: TimerSettings) -> Self {
        let state = PomodoroState::Work(value.work_time);
        let running = false;
        let time_left = Timer::get_duration(&state);
        Countdown {
            time_left,
            state,
            running,
        }
    }
}
#[cfg(test)]
mod tests {}
