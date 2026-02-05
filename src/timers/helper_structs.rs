use super::counters::CounterMode;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TimerState {
    Work(i64),
    Break(i64),
}
pub enum TimerCommand {
    Start,
    NextIteration(TimerState),
    Stop,
    Restart(TimerState),
    ChangeMode(CounterMode),
}
#[derive(Debug, Clone)]
pub struct TimerData {
    iteration: u8,
    time_left: i64,
    total_iterations: u8,
    total_time: i64,
    total_elapsed: i64,
    work_state: TimerState,
    break_state: TimerState,
    current_state: TimerState,
}

impl TimerData {
    pub fn new(
        time_left: i64,
        total_iterations: u8,
        total_time: i64,
        work_state: TimerState,
        break_state: TimerState,
    ) -> Self {
        Self {
            iteration: 1,
            time_left,
            total_iterations,
            total_time,
            total_elapsed: 0,
            work_state,
            break_state,
            current_state: work_state,
        }
    }

    pub fn iteration(&self) -> u8 {
        self.iteration
    }

    pub fn time_left(&self) -> i64 {
        self.time_left
    }

    pub fn total_iterations(&self) -> u8 {
        self.total_iterations
    }

    pub fn total_time(&self) -> i64 {
        self.total_time
    }

    pub fn total_elapsed(&self) -> i64 {
        self.total_elapsed
    }

    pub fn work_state(&self) -> TimerState {
        self.work_state
    }

    pub fn break_state(&self) -> TimerState {
        self.break_state
    }

    pub fn current_state(&self) -> TimerState {
        self.current_state
    }

    pub fn set_iteration(&mut self, iteration: u8) {
        self.iteration = iteration;
    }

    pub fn set_time_left(&mut self, time_left: i64) {
        self.time_left = time_left;
    }

    pub fn set_total_iterations(&mut self, total_iterations: u8) {
        self.total_iterations = total_iterations;
    }

    pub fn set_total_time(&mut self, total_time: i64) {
        self.total_time = total_time;
    }

    pub fn set_total_elapsed(&mut self, total_elapsed: i64) {
        self.total_elapsed = total_elapsed;
    }

    pub fn set_work_state(&mut self, work_state: TimerState) {
        self.work_state = work_state;
    }

    pub fn set_break_state(&mut self, break_state: TimerState) {
        self.break_state = break_state;
    }

    pub fn set_current_state(&mut self, current_state: TimerState) {
        self.current_state = current_state;
    }
}
