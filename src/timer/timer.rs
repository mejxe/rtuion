use std::{cell::RefCell, rc::Rc};

use tokio_util::sync::CancellationToken;

use crate::{
    settings::{PomodoroSettings, SettingsTab},
    utils::beeps::{beep_break, beep_finish, beep_work},
};

use super::{
    countdown::Countdown,
    helper_structs::{PomodoroState, TimerCommand},
};

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
impl Timer {
    pub fn new(
        time_left: i64,
        total_iterations: u8,
        total_time: i64,
        work_state: PomodoroState,
        break_state: PomodoroState,
        settings: Rc<RefCell<SettingsTab>>,
    ) -> Self {
        Self {
            countdown_running: false,
            iteration: 1,
            time_left,
            total_iterations,
            total_time,
            total_elapsed: 0,
            work_state,
            break_state,
            current_state: work_state,
            next_state: break_state,
            settings,
            countdown_command_tx: None,
        }
    }

    pub fn get_duration(pomodoro_state: &PomodoroState) -> i64 {
        match pomodoro_state {
            PomodoroState::Work(dur) | PomodoroState::Break(dur) => *dur,
        }
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
        match self.get_current_state() {
            PomodoroState::Work(_) => beep_work(),
            PomodoroState::Break(_) => beep_break(),
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
    pub async fn finish(&mut self) {
        beep_finish();
        self.restart().await;
    }
    pub async fn set_setting(&mut self, setting: PomodoroSettings) -> Option<()> {
        if self.get_running() {
            return None;
        };
        match setting {
            PomodoroSettings::Iterations(iterations) => self.set_total_iterations(iterations),
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
#[cfg(test)]
mod tests {}
