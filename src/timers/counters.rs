use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::{
    helper_structs::{TimerCommand, TimerState},
    timer::Timer,
};
#[derive(Clone, Debug)]
pub struct Counter {
    time_left: i64,
    state: TimerState,
    running: bool,
    mode: CounterMode,
}
impl Counter {
    pub fn new(mode: CounterMode, state: TimerState) -> Self {
        let time = match state {
            _ if mode == CounterMode::Countup => 0,
            TimerState::Work(time) => time,
            TimerState::Break(time) => time,
        };
        Counter {
            time_left: time,
            state,
            running: false,
            mode,
        }
    }
    pub async fn run(
        &mut self,
        sender: tokio::sync::mpsc::Sender<i64>,
        mut command_rx: tokio::sync::mpsc::Receiver<TimerCommand>,
        close: CancellationToken,
    ) {
        loop {
            tokio::select! {
            Some(command) = command_rx.recv() => self.handle_commands(command),

            _ = tokio::time::sleep(Duration::from_secs(1)), if self.mode.should_tick(self.running, self.time_left) => {
                self.time_left = self.mode.tick(self.time_left).await;
                sender.send(self.time_left).await.unwrap();
            }
            _ = close.cancelled() => {break}
            }
        }
    }
    fn set_time_left(&mut self, time_left: i64) {
        self.time_left = time_left;
    }

    fn set_state(&mut self, state: TimerState) {
        self.state = state;
    }

    fn set_running(&mut self, running: bool) {
        self.running = running;
        self.time_left = 3595;
    }
    fn next_iteration(&mut self, state: TimerState) {
        self.set_state(state);
        self.set_time_left(Timer::get_duration(&state));
    }

    fn restart(&mut self, state: TimerState) {
        self.set_time_left(Timer::get_duration(&state));
        self.set_state(state);
        self.set_running(false);
    }
    fn handle_commands(&mut self, command: TimerCommand) {
        match command {
            TimerCommand::Start => self.set_running(true),
            TimerCommand::Stop => self.set_running(false),
            TimerCommand::NextIteration(state) => self.next_iteration(state),
            TimerCommand::ChangeMode(mode) => self.set_mode(mode),
            TimerCommand::Restart(state) => self.restart(state),
        };
    }

    pub fn set_mode(&mut self, mode: CounterMode) {
        self.mode = mode;
        if self.mode == CounterMode::Countup {
            self.time_left = 0;
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum CounterMode {
    Countdown,
    Countup,
}
impl CounterMode {
    pub async fn tick(&self, time: i64) -> i64 {
        match self {
            CounterMode::Countdown => time - 1,
            CounterMode::Countup => time + 1,
        }
    }
    pub fn should_tick(&self, running: bool, time_left: i64) -> bool {
        match self {
            CounterMode::Countdown => running && (time_left >= 0),
            CounterMode::Countup => running,
        }
    }
    pub fn next(&mut self) {
        *self = match self {
            Self::Countdown => Self::Countup,
            Self::Countup => Self::Countdown,
        };
    }
}
