use std::{cell::RefCell, rc::Rc};

use tokio_util::sync::CancellationToken;

use crate::{
    app::Event,
    error::{Result, StatsError},
    settings::{PomodoroSettings, Settings},
    stats::pixela::{
        pixela_client::PixelaClient,
        pixela_user::PixelaUser,
        subjects::{Progress, Seconds, Subject, TimeUnit},
    },
    timers::{
        helper_structs::{TimerCommand, TimerState},
        timer::Timer,
    },
    MIN_HOUR_INCREMENT,
};

#[derive(Debug)]
pub struct Pomodoro {
    pub timer: Timer,
    time_sender: tokio::sync::mpsc::Sender<Seconds>,
    command_rx: Option<tokio::sync::mpsc::Receiver<TimerCommand>>,
    settings: Rc<RefCell<Settings>>,
    pixela_client: Option<PixelaClient>,
    duration_since_last_save: Seconds,
}
impl Pomodoro {
    pub fn new(
        time_sender: tokio::sync::mpsc::Sender<Seconds>,
        command_rx: tokio::sync::mpsc::Receiver<TimerCommand>,
        command_tx: tokio::sync::mpsc::Sender<TimerCommand>,
        settings: Rc<RefCell<Settings>>,
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
    pub fn try_init_pixela_client(&mut self) -> Result<()> {
        let px_stats = &mut self.settings.borrow_mut().stats_setting;
        if px_stats.stats_on {
            if px_stats.pixela_username.is_none() {
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
    pub async fn set_setting(&mut self, setting: PomodoroSettings) -> Option<()> {
        self.timer.set_setting(setting).await
    }
    pub fn log_pixel_from_duration(&mut self) -> Result<()> {
        if self.duration_since_last_save < 60 {
            return Err(crate::error::Error::StatsError(
                StatsError::QuantityIsNotBigEnough,
            ));
        }

        if let Some(client) = &self.pixela_client {
            let date = chrono::Local::now();
            let subject = client.get_current_subject();
            let duration =
                Progress::new_minutes(TimeUnit::Seconds(self.duration_since_last_save + 1));
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
    pub async fn handle_timer_tick(&mut self, time: Seconds) -> Result<()> {
        self.handle_logging(time)?;
        self.timer.handle_timer_response(time).await;
        self.duration_since_last_save += 1;
        Ok(())
    }
    fn handle_logging(&mut self, time: Seconds) -> Result<()> {
        let increment: Seconds;
        let current_state = self.timer.current_state();
        if let Some(sub) = self.get_current_subject() {
            increment = (sub.get_min_increment() * 60) as Seconds;
        } else {
            increment = (MIN_HOUR_INCREMENT * 60) as Seconds;
        }

        if let TimerState::Work(_) = current_state {
            if time % increment == 0 {
                self.log_pixel_from_duration()?
            }
        }
        Ok(())
    }
    pub fn check_pixela_changed(&self, client_username: &str, client_token: &str) -> bool {
        let settings = &self.settings.borrow().stats_setting;
        let settings_username = settings.pixela_username.as_ref().expect("");
        let settings_token = settings.pixela_token.as_ref().expect("");
        client_token != settings_token || client_username != settings_username
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
        if let Some(client) = &mut self.pixela_client {
            client.set_current_subject_index(current_subject_index);
        }
    }
    pub fn get_setting_ref(&self) -> Rc<RefCell<Settings>> {
        self.settings.clone()
    }

    pub async fn set_counter_mode(&mut self, mode: crate::timers::counters::CounterMode) {
        self.timer.set_counter_mode(mode).await
    }
    pub fn clear_pixela(&mut self) {
        self.pixela_client = None;
    }
    pub async fn restart_timer(&mut self) {
        self.timer.restart().await;
        self.duration_since_last_save = 0;
    }
}
