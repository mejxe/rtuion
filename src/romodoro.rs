use std::{cell::RefCell, rc::Rc};

use chrono::{DateTime, Local};
use tokio_util::sync::CancellationToken;

use crate::{
    app::Event,
    pixela_client::{self, PixelaClient},
    settings::{self, PomodoroSettings, SettingsTab},
    stats::PixelaUser,
    timer::{self, *},
};

#[derive(Debug)]
pub struct Pomodoro {
    pub timer: Timer,
    time_sender: tokio::sync::mpsc::Sender<i64>,
    command_rx: Option<tokio::sync::mpsc::Receiver<TimerCommand>>,
    settings: Rc<RefCell<SettingsTab>>,
    pixela_client: Option<PixelaClient>,
    duration_since_last_save: i64,
}
impl Pomodoro {
    pub fn new(
        time_sender: tokio::sync::mpsc::Sender<i64>,
        command_rx: tokio::sync::mpsc::Receiver<TimerCommand>,
        command_tx: tokio::sync::mpsc::Sender<TimerCommand>,
        settings: Rc<RefCell<SettingsTab>>,
    ) -> Self {
        let mut timer = Timer::from(settings.borrow().timer_settings.clone());
        timer.set_config(settings.clone());
        timer.countdown_command_tx = Some(command_tx);

        let mut pixela_client = None; // create PixelaClient for logging progress
        match &mut settings.borrow_mut().stats_setting {
            px_stats if px_stats.stats_on => {
                let user: PixelaUser = PixelaUser::new(
                    px_stats
                        .pixela_username
                        .clone()
                        .expect("Should be initialized"),
                    px_stats
                        .pixela_token
                        .clone()
                        .expect("Should be initialized."),
                );
                if let Ok(px_client) = PixelaClient::try_new(user) {
                    pixela_client = Some(px_client);
                } else {
                    px_stats.stats_on = false;
                    // TODO: add a notification of some kind (err is in try_new)
                }
            }
            _ => {}
        }

        Pomodoro {
            timer,
            command_rx: Some(command_rx),
            time_sender,
            settings,
            pixela_client,
            duration_since_last_save: 0,
        }
    }
    pub async fn create_countdown(&mut self, cancel_token: CancellationToken) {
        let sender = self.time_sender.clone();
        let command_rx = self.command_rx.take().expect("Timer initialized");
        self.timer.set_config(self.settings.clone());
        self.timer.run(sender, command_rx, cancel_token).await;
    }
    pub async fn cycle(&mut self) {
        if self.timer.get_running() {
            self.timer.stop().await;
        } else {
            self.timer.start().await;
        }
    }
    pub fn get_work_state(&self) -> PomodoroState {
        self.timer.get_work_state()
    }
    pub fn get_setting_ref(&self) -> Rc<RefCell<SettingsTab>> {
        self.settings.clone()
    }

    pub async fn handle_timer(
        time_rx: &mut tokio::sync::mpsc::Receiver<i64>,
        tx: tokio::sync::mpsc::Sender<Event>,
        cancel_token: CancellationToken,
    ) {
        loop {
            tokio::select! {
                time = time_rx.recv() => {
                    match time {
                        Some(time) => {
                            let _ = tx.send(Event::TimerTick(time)).await;
                        },
                        None => {break},
                    }
                }
                _ = cancel_token.cancelled() => {
                    break
                }

            }
        }
    }
    pub fn set_time_left(&mut self, time: i64) {
        self.timer.set_time_left(time);
        self.duration_since_last_save += 1;
        if let PomodoroState::Work(_) = self.timer.get_current_state() {
            self.timer.set_elapsed_time(
                (self.timer.get_iteration() - 1) as i64
                    * Timer::get_duration(&self.timer.get_work_state())
                    + Timer::get_duration(&self.get_work_state())
                    - time,
            )
        }
    }
    pub async fn set_setting(&mut self, setting: PomodoroSettings) -> Option<()> {
        self.timer.set_setting(setting).await
    }
    pub async fn log_pixel_from_duration(&mut self) {
        let date = chrono::Local::now();
        let subject = self.settings.borrow().stats_setting.current_subject();
        let duration = subject.transform_progress(self.duration_since_last_save);
        if let Some(client) = self.pixela_client.as_mut() {
            client.add_pixel(date, subject, duration);
        }
    }
    pub async fn handle_timer_responses(&mut self, time: i64) {
        if time == -1 && self.timer.get_iteration() < self.timer.get_total_iterations() {
            self.timer.next_iteration().await;
        } else if time == -1 && self.timer.get_iteration() > self.timer.get_total_iterations() {
            self.timer.stop().await;
        } else if time == -1 && self.timer.get_iteration() == self.timer.get_total_iterations() {
            if let PomodoroState::Break(_) = self.timer.get_current_state() {
                self.timer.stop().await;
            }
        } else {
            self.set_time_left(time);
        }
    }
}
