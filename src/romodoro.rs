use std::{cell::RefCell, rc::Rc};

use chrono::{DateTime, Local};
use tokio_util::sync::CancellationToken;

use crate::{
    app::Event,
    error::{Error, Result, StatsError},
    pixela_client::{self, PixelaClient},
    settings::{self, PomodoroSettings, SettingsTab},
    stats::{PixelaUser, Subject},
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

        Pomodoro {
            timer,
            command_rx: Some(command_rx),
            time_sender,
            settings,
            pixela_client: None,
            duration_since_last_save: 0,
        }
    }
    pub fn get_current_subject(&self) -> Option<Subject> {
        if let Some(pixela) = &self.pixela_client {
            Some(pixela.get_current_subject()?)
        } else {
            None
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
    pub fn try_init_pixela_client(&mut self) -> Result<()> {
        let px_stats = &mut self.settings.borrow_mut().stats_setting;
        if px_stats.stats_on {
            if px_stats.pixela_username.is_none() || px_stats.pixela_token.is_none() {
                px_stats.init_stats();
                return Err(StatsError::UserNotProvided().into());
            }
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
            if !user.validate_not_empty() {
                Err(StatsError::UserNotProvided().into())
            } else {
                self.pixela_client = Some(PixelaClient::try_new(user)?);
                Ok(())
            }
        } else {
            Err(StatsError::StatsTrackingTurnedOff().into())
        }
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
    pub fn log_pixel_from_duration(&mut self) -> Result<()> {
        if let Some(client) = &self.pixela_client {
            let date = chrono::Local::now();
            let subject = client.get_current_subject();
            let duration = subject
                .as_ref()
                // default simply logs minutes
                .map_or(((self.duration_since_last_save / 60) as i32).into(), |s| {
                    s.transform_progress(self.duration_since_last_save)
                });
            if let Some(client) = self.pixela_client.as_mut() {
                client.add_pixel(date, subject, duration);
                client.save_pixels()?;
            }
            self.duration_since_last_save = 0;
            Ok(())
        } else {
            Err(crate::error::StatsError::StatsTrackingTurnedOff().into())
        }
    }
    pub async fn handle_timer_responses(&mut self, time: i64) -> Result<()> {
        if time == -1 && self.timer.get_iteration() < self.timer.get_total_iterations() {
            if let PomodoroState::Work(_) = self.timer.get_current_state() {
                self.log_pixel_from_duration()?;
            }
            self.timer.next_iteration().await;
        } else if time == -1 && self.timer.get_iteration() >= self.timer.get_total_iterations() {
            self.timer.stop().await;
            self.log_pixel_from_duration()?;
        } else {
            self.set_time_left(time);
        }
        Ok(())
    }
    pub fn is_duration_saved(&self) -> bool {
        self.duration_since_last_save == 0
    }

    pub fn pixela_client(&self) -> Option<&PixelaClient> {
        self.pixela_client.as_ref()
    }
    pub fn pixela_client_as_mut(&mut self) -> Option<&mut PixelaClient> {
        self.pixela_client.as_mut()
    }

    pub fn set_current_subject_index(&mut self, current_subject_index: usize) {
        // TODO: MAKE SO SELECTING A DIFFERENT SUBJECT RESETS THE TIMER (NOTIFICATIONS!)
        if let Some(client) = &mut self.pixela_client {
            client.set_current_subject_index(current_subject_index);
        }
    }
}
