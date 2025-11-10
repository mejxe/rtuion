use crate::error::Error;
use crate::graph::Graph;
use crate::pixela_client::PixelaClient;
use crate::popup::{Popup, PopupKind};
use crate::romodoro::Pomodoro;
use crate::settings::*;
use arboard::Clipboard;
use core::panic;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use derivative::Derivative;
use ratatui::DefaultTerminal;
use std::cell::RefCell;
use std::io;
use std::process::exit;
use std::rc::Rc;
use tokio_util::sync::CancellationToken;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct App {
    exit: bool,
    pomodoro: Pomodoro,
    selected_tab: u8,
    settings: Rc<RefCell<SettingsTab>>,
    popup: Option<Popup>, // TODO: popup queue maybe so they don't overwrite each other?
    #[derivative(Debug = "ignore")]
    clipboard: Option<Clipboard>,
    event_tx: tokio::sync::mpsc::Sender<Event>,
}
pub enum Event {
    TimerTick(i64),
    KeyPress(KeyEvent),
    TerminalEvent,
    OverwriteTimerSettings,
    OverwriteTimerForSubject(usize),
    SendPixels,
    DeletePixel,
    RequestGraph,
    GraphReceived(Result<Graph, Error>),
}
impl App {
    pub fn new(
        pomodoro: Pomodoro,
        settings: Rc<RefCell<SettingsTab>>,
        event_tx: tokio::sync::mpsc::Sender<Event>,
    ) -> Self {
        App {
            pomodoro,
            exit: false,
            selected_tab: 0,
            settings,
            popup: None,
            clipboard: Clipboard::new().ok(),
            event_tx,
        }
    }
    pub async fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        mut rx: tokio::sync::mpsc::Receiver<Event>,
        mut time_rx: tokio::sync::mpsc::Receiver<i64>,
    ) -> io::Result<()> {
        let tx_inputs = self.event_tx.clone();
        let tx_timer = self.event_tx.clone();

        let cancelation_token = tokio_util::sync::CancellationToken::new();
        let input_cancel = cancelation_token.clone();
        let timer_comm_cancel = cancelation_token.clone();
        let timer_cancel = cancelation_token.clone();

        let input_task = tokio::spawn(async move {
            match App::handle_inputs(tx_inputs, input_cancel).await {
                Ok(_) => {}
                Err(e) => {
                    panic!("{e}")
                }
            }
        });

        let timer_task = tokio::spawn(async move {
            Pomodoro::handle_timer(&mut time_rx, tx_timer, timer_comm_cancel).await;
        });
        self.pomodoro.create_countdown(timer_cancel).await;

        terminal.draw(|frame| self.draw(frame))?;
        while !self.exit {
            if let Some(event) = rx.recv().await {
                match event {
                    Event::KeyPress(key) => {
                        self.handle_key_event(key).await;
                    }
                    Event::TimerTick(time) => {
                        if let Err(e) = self.pomodoro.handle_timer_responses(time).await {
                            self.set_popup(e.into());
                        };
                    }
                    Event::TerminalEvent => {}
                    Event::OverwriteTimerSettings => self.overwrite_timer_settings().await,
                    Event::OverwriteTimerForSubject(index) => {
                        self.overwrite_timer_for_subject(index).await
                    }
                    Event::SendPixels => {
                        if let Some(pixela) = self.get_pomodoro_ref_mut().pixela_client_as_mut() {
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
                        if let Some(pixela_client) = self.pomodoro.pixela_client_as_mut() {
                            if let Err(e) = pixela_client.delete_pixel() {
                                self.set_popup(e.into());
                            }
                        }
                    }
                    Event::RequestGraph => {
                        if let Some(client) = self.pomodoro.pixela_client_as_mut() {
                            if let Some(index) = client.subjects.state().selected() {
                                let user = client.user.clone();
                                let rq_client = client.client.clone();
                                let tx_clone = self.event_tx.clone();
                                let subject = client.get_subject(index).unwrap();

                                tokio::spawn(async move {
                                    let result =
                                        Graph::download_graph(user, rq_client, subject).await;

                                    if let Err(e) =
                                        tx_clone.send(Event::GraphReceived(result)).await
                                    {
                                        eprintln!("Failed to send GraphDataReceived event: {}", e);
                                    }
                                });
                            }
                        }
                    }
                    Event::GraphReceived(res) => {
                        if let Some(client) = self.pomodoro.pixela_client_as_mut() {
                            match res {
                                Ok(g) => client.set_current_graph(Some(g)),
                                Err(e) => self.set_popup(e.into()),
                            }
                        }
                    }
                }
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        cancelation_token.cancel();
        timer_task.await?;
        input_task.abort();
        Ok(())
    }
    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        //global
        match key_event.code {
            KeyCode::Char('Q') => self.exit(),
            KeyCode::Tab
                if self.popup().is_none() && !(self.settings.borrow().mode() == &Mode::Input) =>
            {
                self.change_tab()
            }
            _ => {}
        }
        // popup
        if let Some(popup) = self.popup.take() {
            match popup.kind {
                PopupKind::YesNoPopup(callback) | PopupKind::SendPixelsPopup(callback, _) => {
                    match key_event.code {
                        KeyCode::Char('y') => {
                            callback(self);
                            self.clear_popup()
                        }
                        KeyCode::Char('n') => self.clear_popup(),
                        _ => {}
                    }
                }
                PopupKind::ErrorPopup(_) | PopupKind::ListPopup(_) => self.clear_popup(),
            }
            return;
        };
        match self.selected_tab {
            0 => {
                // timer
                if let KeyCode::Char(' ') = key_event.code {
                    self.pomodoro.cycle().await
                }
            }
            // settings
            1 if self.settings.borrow().mode() == &Mode::Input => match key_event.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.settings.borrow_mut().change_mode(Mode::Normal)
                }
                key => {
                    if key == KeyCode::Char('v') && key_event.modifiers == KeyModifiers::CONTROL {
                        if let Some(clip) = self.clipboard.as_mut() {
                            self.settings
                                .borrow_mut()
                                .input(key_event, Some(clip.get_text().unwrap()));
                        }
                    } else {
                        self.settings.borrow_mut().input(key_event, None);
                    }
                }
            },
            1 => match key_event.code {
                KeyCode::Down | KeyCode::Char('j') => self.settings.borrow_mut().select_down(),
                KeyCode::Up | KeyCode::Char('k') => self.settings.borrow_mut().select_up(),
                KeyCode::Right | KeyCode::Char('l') => self.settings.borrow_mut().increment(),
                KeyCode::Left | KeyCode::Char('h') => self.settings.borrow_mut().decrement(),
                KeyCode::Char(' ') => self.update_settings().await,
                KeyCode::Char('r') => self.settings.borrow_mut().restore_defaults(),
                KeyCode::Enter if [4, 5].contains(&self.settings.borrow().selected_setting) => {
                    self.settings.borrow_mut().change_mode(Mode::Input)
                }
                _ => {}
            },
            2 => {
                if let Some(pixela_client) = self.pomodoro.pixela_client_as_mut() {
                    match key_event.code {
                        KeyCode::Char('L') if !pixela_client.logged_in() => {
                            let res = pixela_client.log_in().await;
                            self.set_popup_opt(Error::handle_error_and_consume_data(res));
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            pixela_client.change_focused_pane(true)
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            pixela_client.change_focused_pane(false)
                        }
                        KeyCode::Down | KeyCode::Char('j') => pixela_client.select_next(),
                        KeyCode::Up | KeyCode::Char('k') => pixela_client.select_previous(),
                        KeyCode::Char(' ') if pixela_client.focused_pane() == 2 => {
                            if let Some(index) = pixela_client.subjects.state().selected() {
                                if !self.pomodoro.is_duration_saved() {
                                    self.set_popup(Popup::yes_no(
                                        "Your current progress will be reset".into(),
                                        Box::new(move |app: &mut App| {
                                            app.ask_overwrite_subject(index);
                                        }),
                                    ));
                                } else {
                                    self.pomodoro.set_current_subject_index(index);
                                }
                            }
                        }
                        KeyCode::Char('G') => {
                            let _ = self.event_tx.send(Event::RequestGraph).await;
                        }
                        KeyCode::Char(' ') if pixela_client.focused_pane() == 0 => {
                            if let Some(index) = pixela_client.pixels.state().selected() {
                                if pixela_client.selected_to_send(index) {
                                    pixela_client.unselect_pixel(index);
                                } else {
                                    pixela_client.select_pixel(index);
                                }
                            }
                        }
                        KeyCode::Char('p') if pixela_client.focused_pane() == 0 => {
                            todo!("Push pixel")
                        }
                        KeyCode::Char('P')
                            if (pixela_client.focused_pane() == 0
                                && !pixela_client.pixels_to_send_is_empty()
                                && pixela_client.logged_in) =>
                        {
                            if let Some(pixels) = self.pomodoro.pixela_client() {
                                let pixels = pixels.get_selected_pixels();
                                self.set_popup(Popup::pixel_confirm_list(
                                    "These pixels will be sent".into(),
                                    Box::new(App::ask_send_pixels),
                                    pixels,
                                ));
                            }
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
            _ => {}
        }
    }

    async fn handle_inputs(
        tx: tokio::sync::mpsc::Sender<Event>,
        cancel_token: CancellationToken,
    ) -> std::io::Result<()> {
        loop {
            match event::read()? {
                crossterm::event::Event::Key(key_event)
                    if key_event.kind == KeyEventKind::Press =>
                {
                    let _ = tx.send(Event::KeyPress(key_event)).await;
                    if cancel_token.is_cancelled() {
                        return Ok(());
                    }
                }
                event::Event::Resize(_, _) => {
                    let _ = tx.send(Event::TerminalEvent).await;
                    if cancel_token.is_cancelled() {
                        return Ok(());
                    }
                }
                _ => {
                    if cancel_token.is_cancelled() {
                        return Ok(());
                    }
                }
            };
        }
    }
    fn ask_overwrite_timer(&mut self) {
        let _ = self.event_tx.try_send(Event::OverwriteTimerSettings);
    }
    fn ask_send_pixels(&mut self) {
        let _ = self.event_tx.try_send(Event::SendPixels);
    }
    fn ask_delete_pixel(&mut self) {
        if self.pomodoro.pixela_client_as_mut().is_some() {
            let _ = self.event_tx.try_send(Event::DeletePixel);
        }
    }
    fn ask_overwrite_subject(&mut self, index: usize) {
        let _ = self
            .event_tx
            .try_send(Event::OverwriteTimerForSubject(index));
    }
    async fn update_settings(&mut self) {
        if self.pomodoro.timer.get_running() {
            self.set_popup(Popup::yes_no(
                "This will overwrite current progress".into(),
                Box::new(App::ask_overwrite_timer),
            ));
            return;
        }
        let break_time = self
            .settings
            .borrow()
            .get_pomodoro_setting(PomodoroSettings::BreakTime(None));
        let work_time = self
            .settings
            .borrow()
            .get_pomodoro_setting(PomodoroSettings::WorkTime(None));
        let iterations = self
            .settings
            .borrow()
            .get_pomodoro_setting(PomodoroSettings::Iterations(None));
        let current_break_time: PomodoroSettings = self.pomodoro.timer.get_break_state().into();
        let current_work_time: PomodoroSettings = self.pomodoro.timer.get_work_state().into();
        let current_iterations: PomodoroSettings =
            PomodoroSettings::Iterations(Some(self.pomodoro.timer.get_total_iterations()));
        if current_break_time != break_time {
            self.pomodoro.set_setting(break_time).await;
        }
        if current_work_time != work_time {
            self.pomodoro.set_setting(work_time).await;
        }
        if current_iterations != iterations {
            self.pomodoro.set_setting(iterations).await;
        }
        // save settings and overwrite existing pixela_client
        let mut popup = Error::handle_error_and_consume_data(self.settings.borrow().save_to_file());
        self.set_popup_opt(popup);
        popup = Error::handle_error_and_consume_data(self.pomodoro.try_init_pixela_client());
        self.set_popup_opt(popup);
    }
    pub async fn overwrite_timer_settings(&mut self) {
        self.pomodoro.timer.stop().await;
        self.update_settings().await;
    }
    async fn overwrite_timer_for_subject(&mut self, index: usize) {
        self.pomodoro.timer.restart().await;
        self.pomodoro.set_current_subject_index(index);
    }

    pub fn get_selected_tab(&self) -> u8 {
        self.selected_tab
    }
    pub fn get_settings_ref(&self) -> Rc<RefCell<SettingsTab>> {
        self.settings.clone()
    }
    pub fn get_pomodoro_ref(&self) -> &Pomodoro {
        &self.pomodoro
    }
    pub fn get_pomodoro_ref_mut(&mut self) -> &mut Pomodoro {
        &mut self.pomodoro
    }

    fn change_tab(&mut self) {
        if self.selected_tab == 2 {
            self.selected_tab = 0;
        } else {
            self.selected_tab += 1;
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn popup(&self) -> Option<&Popup> {
        self.popup.as_ref()
    }
    pub fn clear_popup(&mut self) {
        self.popup = None;
    }

    pub fn set_popup(&mut self, popup: Popup) {
        self.popup = Some(popup);
    }
    pub fn set_popup_opt(&mut self, popup: Option<Popup>) {
        self.popup = popup;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    //  #[test]
    //   #[ignore]
    //   fn multithread_works() {
    //       let (tx, rx) = tokio::sync::mpsc::channel(4);
    //       let mut app = App::new(Timer::new(PomodoroState::Work(2), PomodoroState::Break(1), 2), rx, tx);
    //       app.start_timer();

    //   }
} //
