use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::{App, Event},
    error::Error,
    popup::Popup,
    settings::PomodoroSettings,
};

impl App {
    pub async fn handle_pixela_keybinds(&mut self, key_event: KeyEvent) {
        if let Some(pixela_client) = self.pomodoro_mut().pixela_client_as_mut() {
            match key_event.code {
                KeyCode::Char('L') if !pixela_client.logged_in() => {
                    let res = pixela_client.log_in().await;
                    self.set_popup_opt(Error::handle_error_and_consume_data(res));
                }
                KeyCode::Right | KeyCode::Char('l') => pixela_client.change_focused_pane(true),
                KeyCode::Left | KeyCode::Char('h') => pixela_client.change_focused_pane(false),
                KeyCode::Down | KeyCode::Char('j') => pixela_client.select_next(),
                KeyCode::Up | KeyCode::Char('k') => pixela_client.select_previous(),
                KeyCode::Char(' ') if pixela_client.focused_pane() == 2 => {
                    if let Some(index) = pixela_client.subjects.state().selected() {
                        if !self.pomodoro().is_duration_saved() {
                            self.set_popup(Popup::yes_no(
                                "Your current progress & time settings will be reset".into(),
                                Box::new(move |app: &mut App| {
                                    app.ask_overwrite_subject(index);
                                }),
                            ));
                        } else {
                            self.pomodoro_mut().set_current_subject_index(index);
                        }
                        if let Some(subject) = self.pomodoro().get_current_subject() {
                            let time = (subject.get_min_increment() * 60) as i64;
                            self.pomodoro_mut()
                                .set_setting(PomodoroSettings::WorkTime(time))
                                .await;
                            self.settings().borrow_mut().timer_settings.work_time = time;
                        }
                    }
                }
                KeyCode::Char('G') => {
                    let _ = self.event_tx().send(Event::RequestGraph).await;
                }
                KeyCode::Char(' ') | KeyCode::Enter if pixela_client.focused_pane() == 0 => {
                    if let Some(index) = pixela_client.pixels.state().selected() {
                        if pixela_client.selected_to_send(index) {
                            pixela_client.unselect_pixel(index);
                        } else {
                            pixela_client.select_pixel(index);
                        }
                    }
                }
                KeyCode::Char('P')
                    if (pixela_client.focused_pane() == 0
                        && !pixela_client.pixels_to_send_is_empty()
                        && pixela_client.logged_in) =>
                {
                    let pixels = pixela_client.get_selected_pixels();
                    self.set_popup(Popup::pixel_confirm_list(
                        "These pixels will be sent".into(),
                        Box::new(App::ask_send_pixels),
                        pixels,
                    ));
                }
                KeyCode::Char('d') if pixela_client.focused_pane() == 0 => {
                    self.set_popup(Popup::yes_no(
                        "This pixel will be deleted, you sure?".into(),
                        Box::new(App::ask_delete_pixel),
                    ));
                }
                _ => {}
            };
        }
    }
    fn ask_send_pixels(&mut self) {
        let _ = self.event_tx().try_send(Event::SendPixels);
    }
    fn ask_delete_pixel(&mut self) {
        if self.pomodoro_mut().pixela_client_as_mut().is_some() {
            let _ = self.event_tx().try_send(Event::DeletePixel);
        }
    }
    fn ask_overwrite_subject(&mut self, index: usize) {
        let _ = self
            .event_tx()
            .try_send(Event::OverwriteTimerForSubject(index));
    }
}
