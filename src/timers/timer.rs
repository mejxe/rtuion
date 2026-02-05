use std::{cell::RefCell, rc::Rc};

use tokio_util::sync::CancellationToken;

use crate::{
    settings::{PomodoroSettings, Settings},
    utils::beeps::{beep_break, beep_finish, beep_work},
};

use super::{
    counters::{Counter, CounterMode},
    helper_structs::{TimerCommand, TimerData, TimerState},
};

#[derive(Debug, Clone)]
pub struct Timer {
    countdown_running: bool,
    counter_mode: CounterMode,
    timer_data: TimerData,
    duration_since_last_save: usize,
    settings: Rc<RefCell<Settings>>,

    pub countdown_command_tx: Option<tokio::sync::mpsc::Sender<TimerCommand>>,
}
impl Timer {
    pub fn new(
        time_left: i64,
        total_iterations: u8,
        total_time: i64,
        work_state: TimerState,
        break_state: TimerState,
        settings: Rc<RefCell<Settings>>,
        counter_mode: CounterMode,
    ) -> Self {
        Self {
            counter_mode,
            countdown_running: false,
            timer_data: TimerData::new(
                time_left,
                total_iterations,
                total_time,
                work_state,
                break_state,
            ),
            settings,
            countdown_command_tx: None,
            duration_since_last_save: 0,
        }
    }

    pub async fn run(
        &mut self,
        sender: tokio::sync::mpsc::Sender<i64>,
        command_rx: tokio::sync::mpsc::Receiver<TimerCommand>,
        close: CancellationToken,
    ) {
        let mut counter = Counter::from(self.settings.borrow().timer_settings.clone());
        self.set_counter_mode(self.counter_mode).await;
        tokio::task::spawn({
            async move {
                counter.run(sender, command_rx, close).await;
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

    pub async fn next_iteration(&mut self) {
        self.countdown_running = false;
        self.reset_time_states().await;
        self.swap_state();
        self.handle_mode_specific_iteration_stuff().await;
        self.stop().await;
        self.send_countdown_commands(TimerCommand::NextIteration(self.current_state()))
            .await;
        if let TimerState::Work(_) = self.current_state() {
            self.increment_iteration();
        }
        if !self.settings.borrow().ui_settings.pause_after_state_change {
            self.start().await;
        }
        match self.current_state() {
            TimerState::Work(_) => beep_work(),
            TimerState::Break(_) => beep_break(),
        }
    }
    async fn handle_mode_specific_iteration_stuff(&mut self) {
        match (self.current_state(), self.counter_mode()) {
            (TimerState::Break(_), CounterMode::Countup) => {
                self.send_countdown_commands(TimerCommand::ChangeMode(CounterMode::Countdown))
                    .await
            }
            (TimerState::Work(_), CounterMode::Countup) => {
                self.send_countdown_commands(TimerCommand::ChangeMode(CounterMode::Countup))
                    .await
            }
            _ => {}
        }
    }
    async fn reset_time_states(&mut self) {
        match self.counter_mode() {
            CounterMode::Countup => {
                self.set_break_state(TimerState::Break(self.timer_data.time_left() / 5));
                self.set_work_state(TimerState::Work(0));
            }
            CounterMode::Countdown => {
                let work_time = self.settings.borrow().work_time() as i64;
                let break_time = self.settings.borrow().break_time() as i64;
                self.set_work_state(TimerState::Work(work_time));
                self.set_break_state(TimerState::Break(break_time));
            }
        }
    }
    pub async fn send_countdown_commands(&self, command: TimerCommand) {
        if let Some(tx) = &self.countdown_command_tx {
            let _ = tx.send(command).await;
        }
    }

    pub async fn restart(&mut self) {
        self.countdown_running = false;
        self.reset_time_states().await;
        self.set_current_state(self.work_state());
        self.set_iteration(1);
        self.set_time_left(Timer::get_duration(&self.work_state()));
        self.set_total_elapsed(0);
        self.init_total_time();
        self.send_countdown_commands(TimerCommand::Restart(self.work_state()))
            .await;
    }
    pub async fn finish(&mut self) {
        beep_finish();
        self.restart().await;
    }
    pub async fn set_setting(&mut self, setting: PomodoroSettings) -> Option<()> {
        if self.timer_started() {
            return None;
        };
        match setting {
            PomodoroSettings::Iterations(iterations) => self.set_total_iterations(iterations),
            PomodoroSettings::WorkTime(_) => self.set_work_state(TimerState::from(setting)),
            PomodoroSettings::BreakTime(_) => self.set_break_state(TimerState::from(setting)),
        }
        self.restart().await;
        Some(())
    }
    fn swap_state(&mut self) {
        self.set_current_state(match self.current_state() {
            TimerState::Work(_) => self.break_state(),
            TimerState::Break(_) => self.work_state(),
        });
        let duration = Timer::get_duration(&self.current_state());
        self.handle_time_left(duration);
    }
    pub fn handle_time_left(&mut self, time: i64) {
        self.set_time_left(time);
        self.duration_since_last_save += 1;
        if let TimerState::Work(_) = self.current_state() {
            self.set_total_elapsed(self.total_elapsed() + 1);
        }
    }
    pub async fn set_counter_mode(&mut self, mode: CounterMode) {
        self.send_countdown_commands(TimerCommand::ChangeMode(mode))
            .await;
        self.counter_mode = mode;
        self.restart().await;
    }
    pub async fn handle_timer_response(&mut self, time: i64) {
        if time == -1 && self.iteration() < self.total_iterations() {
            self.next_iteration().await;
        } else if time == -1 && self.iteration() >= self.total_iterations() {
            self.stop().await;
            self.finish().await;
        } else {
            self.handle_time_left(time);
        }
    }
    pub fn get_duration(pomodoro_state: &TimerState) -> i64 {
        match pomodoro_state {
            TimerState::Work(dur) | TimerState::Break(dur) => *dur,
        }
    }
    pub fn init_total_time(&mut self) {
        let duration = Timer::get_duration(&self.work_state());
        self.set_total_time(duration * self.total_iterations() as i64);
    }

    pub fn get_running(&self) -> bool {
        self.countdown_running
    }
    pub fn set_running(&mut self, state: bool) {
        self.countdown_running = state;
    }
    pub fn set_config(&mut self, settings: Rc<RefCell<Settings>>) {
        self.settings = settings;
    }
    pub fn timer_started(&self) -> bool {
        self.timer_data.total_elapsed() != 0
    }

    pub fn break_state(&self) -> TimerState {
        self.timer_data.break_state()
    }

    pub fn current_state(&self) -> TimerState {
        self.timer_data.current_state()
    }

    pub fn iteration(&self) -> u8 {
        self.timer_data.iteration()
    }
    pub fn increment_iteration(&mut self) {
        self.set_iteration(self.iteration() + 1);
    }

    pub fn set_current_state(&mut self, current_state: TimerState) {
        self.timer_data.set_current_state(current_state)
    }

    pub fn set_iteration(&mut self, iteration: u8) {
        self.timer_data.set_iteration(iteration)
    }

    pub fn set_total_elapsed(&mut self, total_elapsed: i64) {
        self.timer_data.set_total_elapsed(total_elapsed)
    }

    pub fn time_left(&self) -> i64 {
        self.timer_data.time_left()
    }

    pub fn total_elapsed(&self) -> i64 {
        self.timer_data.total_elapsed()
    }

    pub fn total_iterations(&self) -> u8 {
        self.timer_data.total_iterations()
    }

    pub fn total_time(&self) -> i64 {
        self.timer_data.total_time()
    }

    pub fn work_state(&self) -> TimerState {
        self.timer_data.work_state()
    }
    pub fn work_time(&self) -> i64 {
        self.timer_data.work_state().into()
    }
    pub fn break_time(&self) -> i64 {
        match self.counter_mode() {
            CounterMode::Countdown => self.timer_data.break_state().into(),
            CounterMode::Countup => self.time_left() / 5,
        }
    }

    pub fn set_break_state(&mut self, break_state: TimerState) {
        self.timer_data.set_break_state(break_state)
    }

    pub fn set_time_left(&mut self, time_left: i64) {
        self.timer_data.set_time_left(time_left)
    }

    pub fn set_total_iterations(&mut self, total_iterations: u8) {
        self.timer_data.set_total_iterations(total_iterations)
    }

    pub fn set_work_state(&mut self, work_state: TimerState) {
        self.timer_data.set_work_state(work_state)
    }

    pub fn set_total_time(&mut self, total_time: i64) {
        self.timer_data.set_total_time(total_time)
    }

    pub fn counter_mode(&self) -> CounterMode {
        self.counter_mode
    }
    pub fn in_work_state(&self) -> bool {
        if let TimerState::Work(_) = self.current_state() {
            return true;
        }
        return false;
    }
}
#[cfg(test)]
mod tests {}
