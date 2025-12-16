#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PomodoroState {
    Work(i64),
    Break(i64),
}
pub enum TimerCommand {
    Start,
    NextIteration(PomodoroState),
    Stop,
    Restart(PomodoroState),
}
