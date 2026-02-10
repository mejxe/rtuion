use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::{App, Event},
    error::Error,
    popup::Popup,
    timers::{counters::CounterMode, helper_structs::TimerState},
};

impl App {
    pub async fn handle_timer_tab(&mut self, key_event: KeyEvent) {
        match self.pomodoro().timer.counter_mode() {
            CounterMode::Countdown => {
                if let KeyCode::Char(' ') = key_event.code {
                    self.pomodoro_mut().cycle().await;
                };
            }
            CounterMode::Countup => match key_event.code {
                KeyCode::Char('x')
                    if self.pomodoro().timer.in_work_state()
                        && self.pomodoro().timer.get_running() =>
                {
                    self.pomodoro_mut().timer.next_iteration().await;
                    if let TimerState::Break(_) = self.pomodoro().timer.current_state() {
                        self.handle_flowmodoro_logging();
                    }
                }
                KeyCode::Char(' ') => self.pomodoro_mut().cycle().await,
                _ => {}
            },
        }
        match key_event.code {
            KeyCode::Char('r') => {
                self.set_popup(Popup::yes_no(
                    "Do you want to reset your timer?".to_string(),
                    Box::new(App::ask_restart_timer),
                ));
            }
            _ => {}
        }
    }
    fn handle_flowmodoro_logging(&mut self) {
        if let Some(pixela_client) = self.pomodoro().pixela_client() {
            if pixela_client.get_current_subject().is_some() {
                let popup = Error::handle_error_and_consume_data(
                    self.pomodoro_mut().log_pixel_from_duration(),
                );
                self.set_popup_opt(popup);
            }
        }
    }
    fn ask_restart_timer(&mut self) {
        let _ = self.event_tx().try_send(Event::RestartTimer);
    }
}
