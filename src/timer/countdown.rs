use std::time::Duration;

use tokio_util::sync::CancellationToken;

use super::{
    helper_structs::{PomodoroState, TimerCommand},
    timer::Timer,
};

#[derive(Debug, Clone)]
pub struct Countdown {
    time_left: i64,
    state: PomodoroState,
    running: bool,
}
impl Countdown {
    pub fn new(time_left: i64, state: PomodoroState, running: bool) -> Self {
        Self {
            time_left,
            state,
            running,
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
