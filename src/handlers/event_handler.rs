use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::{App, Event},
    popup::Popup,
    settings::Mode,
    stats::pixela::graph::Graph,
    timers::{counters::CounterMode, helper_structs::TimerState},
    ui::popup::list_height,
    utils::tabs::Tabs,
};

impl App {
    pub async fn handle_event(&mut self, event: Event) {
        match event {
            Event::KeyPress(key) => {
                self.handle_key_event(key).await;
            }
            Event::TimerTick(time) => {
                if let Err(e) = self.pomodoro_mut().handle_timer_tick(time).await {
                    self.set_popup(e.into());
                };
            }
            Event::TerminalEvent => {}
            Event::OverwriteTimerSettings => self.overwrite_timer_settings().await,
            Event::OverwriteTimerForSubject(index) => self.overwrite_timer_for_subject(index).await,
            Event::SendPixels => {
                if let Some(pixela) = self.pomodoro_mut().pixela_client_as_mut() {
                    match pixela.send_pixels().await {
                        Ok(pixels) => self.set_popup(Popup::pixel_list(
                            "Sucessfuly sent these pixels".into(),
                            pixels,
                        )),
                        Err(e) => self.set_popup(e.into()),
                    }
                }
            }
            Event::DeletePixel => {
                if let Some(pixela_client) = self.pomodoro_mut().pixela_client_as_mut() {
                    if let Err(e) = pixela_client.delete_pixel() {
                        self.set_popup(e.into());
                    }
                }
            }
            Event::RequestGraph => {
                let tx_clone = self.event_tx().clone();
                if let Some(client) = self.pomodoro_mut().pixela_client_as_mut() {
                    if let Some(index) = client.subjects.state().selected() {
                        let user = client.user.clone();
                        let rq_client = client.client.clone();
                        let subject = client.get_subject(index).unwrap();

                        tokio::spawn(async move {
                            let result = Graph::download_graph(user, rq_client, subject).await;

                            if let Err(e) = tx_clone.send(Event::GraphReceived(result)).await {
                                eprintln!("Failed to send GraphDataReceived event: {}", e);
                            }
                        });
                    }
                }
            }
            Event::GraphReceived(res) => {
                if let Some(client) = self.pomodoro_mut().pixela_client_as_mut() {
                    match res {
                        Ok(g) => client.set_current_graph(Some(g)),
                        Err(e) => self.set_popup(e.into()),
                    }
                }
            }
            Event::RestartTimer => {
                self.pomodoro_mut().timer.restart().await;
            }
        }
    }
    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        //global
        match key_event.code {
            KeyCode::Char('Q') => self.exit(),
            KeyCode::Tab
                if self.popup().is_none() && !(self.settings().borrow().mode() == Mode::Input) =>
            {
                self.selected_tab_mut().next();
            }
            _ => {}
        }
        let list_height = list_height(&self.popup_size());
        if let Some(popup) = self.take_popup() {
            self.handle_popups(key_event, popup, list_height).await;
        };
        let settings_mode = self.settings().borrow().mode();
        match self.selected_tab() {
            Tabs::TimerTab => self.handle_timer_tab(key_event).await,
            Tabs::SettingsTab => match settings_mode {
                Mode::Modify => self.handle_settings_modify(key_event).await,
                Mode::Input => self.handle_settings_input(key_event).await,
                Mode::Normal => self.handle_settings_normal(key_event).await,
            },
            Tabs::StatsTab => self.handle_pixela_keybinds(key_event).await,
        }
    }
    async fn overwrite_timer_for_subject(&mut self, index: usize) {
        self.pomodoro_mut().timer.restart().await;
        self.pomodoro_mut().set_current_subject_index(index);
    }
}
