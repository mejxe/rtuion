use crate::error::Error;
use crate::popup::{Popup, PopupKind};
use crate::romodoro::Pomodoro;
use crate::settings::*;
use crate::stats::pixela::graph::Graph;
use crate::timers::counters::CounterMode;
use crate::ui::app_ui::AppWidget;
use crate::ui::popup::popup_area;
use crate::utils::tabs::{self, Tabs};
use arboard::Clipboard;
use core::panic;
use crossterm::event::{self, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use derivative::Derivative;
use futures::StreamExt;
use ratatui::layout::Rect;
use ratatui::DefaultTerminal;
use std::cell::RefCell;
use std::io;
use std::process::exit;
use std::rc::Rc;
use std::time::Duration;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct App {
    exit: bool,
    pomodoro: Pomodoro,
    selected_tab: Tabs,
    settings: Rc<RefCell<Settings>>,
    popup: Option<Popup>, // TODO: popup queue maybe so they don't overwrite each other?
    popup_size: Rect,
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
    RestartTimer,
    GraphReceived(Result<Graph, Error>),
}
impl App {
    pub fn new(
        pomodoro: Pomodoro,
        settings: Rc<RefCell<Settings>>,
        event_tx: tokio::sync::mpsc::Sender<Event>,
    ) -> Self {
        App {
            pomodoro,
            exit: false,
            selected_tab: Tabs::TimerTab,
            settings,
            popup: None,
            clipboard: Clipboard::new().ok(),
            event_tx,
            popup_size: Rect::default(),
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
            match App::convert_events(tx_inputs, input_cancel).await {
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

        let _ = self.event_tx.send(Event::TerminalEvent).await; // prompt the handler to draw

        while !self.exit {
            if let Some(event) = rx.recv().await {
                self.handle_event(event).await;
            }
            let mut widget = AppWidget::new(self);
            terminal.draw(|frame| widget.draw(frame))?;
            self.popup_size = popup_area(terminal.get_frame().area(), 50, 35);
        }
        cancelation_token.cancel();
        timer_task.await?;
        input_task.abort();
        Ok(())
    }

    async fn convert_events(
        tx: tokio::sync::mpsc::Sender<Event>,
        cancel_token: CancellationToken,
    ) -> std::io::Result<()> {
        let mut reader = EventStream::new();
        let mut last_event_time = Instant::now();
        let debounce_time = Duration::from_millis(100);
        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    return Ok(());
                }
                event_opt = reader.next() => {
                    match event_opt {
                        Some(Ok(event)) => {
                            match event {
                                crossterm::event::Event::Key(key_event)
                                    if key_event.kind == KeyEventKind::Press =>
                                {
                                    if last_event_time.elapsed() >= debounce_time {
                                        let _ = tx.send(Event::KeyPress(key_event)).await;
                                            last_event_time = Instant::now();
                                        }
                                    else {};
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
                                        _ => {}

                                    }
                                }
                        Some(Err(e)) => return Err(e),
                        None => return Ok(()),
                    }
                }
            }
        }
    }
    fn ask_overwrite_timer(&mut self) {
        let _ = self.event_tx.try_send(Event::OverwriteTimerSettings);
    }
    pub async fn update_settings(&mut self) {
        if self.pomodoro.timer.timer_started() {
            self.set_popup(Popup::yes_no(
                "This will overwrite current progress".into(),
                Box::new(App::ask_overwrite_timer),
            ));
            return;
        }
        let break_time = self.settings.borrow().break_time();
        let work_time = self.settings.borrow().work_time();
        let iterations = self.settings.borrow().iterations();
        let mode = self.settings.borrow().counter_mode();
        if let Some(pixela) = self.pomodoro.pixela_client() {
            if mode == CounterMode::Countdown {
                if let Some(subject) = pixela.get_current_subject() {
                    let min_incr = subject.get_min_increment();
                    if work_time < min_incr * 60 {
                        let err: Error = crate::error::SettingsError::WrongSetting(
                        format!("Your work time cannot be smaller than selected subjects smallest increment: {min_incr}")).into();
                        self.set_popup(Popup::from(err));
                        return;
                    }
                }
            }
        }
        self.pomodoro
            .set_setting(PomodoroSettings::BreakTime(break_time as i64))
            .await;
        self.pomodoro
            .set_setting(PomodoroSettings::WorkTime(work_time as i64))
            .await;
        self.pomodoro
            .set_setting(PomodoroSettings::Iterations(iterations as u8))
            .await;
        self.pomodoro.set_counter_mode(mode).await;

        // save settings and overwrite existing pixela_client
        let mut popup = Error::handle_error_and_consume_data(self.settings.borrow().save_to_file());
        self.set_popup_opt(popup);
        let stats_on = self.settings().borrow().stats_setting.stats_on;
        match stats_on {
            true => {
                let (current_pixela_name, current_pixela_token) =
                    match self.pomodoro.pixela_client() {
                        Some(pixela_client) => {
                            (pixela_client.user.username(), pixela_client.user.token())
                        }

                        None => ("", ""),
                    };
                if self
                    .pomodoro()
                    .check_pixela_changed(current_pixela_name, current_pixela_token)
                {
                    popup = Error::handle_error_and_consume_data(
                        self.pomodoro.try_init_pixela_client(),
                    );
                    self.set_popup_opt(popup);
                }
            }
            false => {
                self.pomodoro.clear_pixela();
            }
        }
    }
    pub async fn overwrite_timer_settings(&mut self) {
        self.pomodoro.timer.restart().await;
        self.update_settings().await;
    }

    pub fn get_settings_ref(&self) -> Rc<RefCell<Settings>> {
        self.settings.clone()
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }

    pub fn popup(&self) -> Option<&Popup> {
        self.popup.as_ref()
    }
    pub fn popup_as_mut(&mut self) -> Option<&mut Popup> {
        self.popup.as_mut()
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

    pub fn event_tx(&self) -> &tokio::sync::mpsc::Sender<Event> {
        &self.event_tx
    }

    pub fn pomodoro(&self) -> &Pomodoro {
        &self.pomodoro
    }

    pub fn pomodoro_mut(&mut self) -> &mut Pomodoro {
        &mut self.pomodoro
    }

    pub fn settings(&self) -> &RefCell<Settings> {
        &self.settings
    }

    pub fn clipboard(&self) -> Option<&Clipboard> {
        self.clipboard.as_ref()
    }

    pub fn clipboard_mut(&mut self) -> &mut Option<Clipboard> {
        &mut self.clipboard
    }

    pub const fn take_popup(&mut self) -> Option<Popup> {
        self.popup.take()
    }

    pub fn selected_tab(&self) -> Tabs {
        self.selected_tab
    }

    pub fn selected_tab_mut(&mut self) -> &mut Tabs {
        &mut self.selected_tab
    }

    pub fn popup_size(&self) -> Rect {
        self.popup_size
    }
}
