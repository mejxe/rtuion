use std::fs;
use std::process::exit;

use crate::error::{Result, SettingsError};
use crate::stats::pixela::pixela_client::PixelaClient;
use crate::timers::counters::CounterMode;
use crate::timers::timer::Timer;
use crate::utils::settings_helper_structs::SettingsTabs;
use crate::{
    timers::*, BREAK_TIME_INCR, DEFAULT_BREAK, DEFAULT_ITERATIONS, DEFAULT_WORK, WORK_TIME_INCR,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use directories::ProjectDirs;
use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Copy)]
pub enum Mode {
    Input,
    #[default]
    Normal,
    Modify,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(skip)]
    pub selected_setting: u8,
    #[serde(skip)]
    pub selected_tab: SettingsTabs,
    #[serde(skip)]
    mode: Mode,
    pub ui_settings: UISettings,
    pub timer_settings: TimerSettings,
    pub stats_setting: StatsSettings,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UISettings {
    pub pause_after_state_change: bool,
    pub hide_work_countdown: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSettings {
    pub mode: CounterMode,
    pub work_time: i64,  // seconds
    pub break_time: i64, // seconds
    pub iterations: u8,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PomodoroSettings {
    WorkTime(i64),
    BreakTime(i64),
    Iterations(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatsSettings {
    pub stats_on: bool,
    pub pixela_username: Option<String>,
    pub pixela_token: Option<String>,
}
impl StatsSettings {
    pub fn init_stats(&mut self) {
        if self.pixela_username.is_none() {
            self.pixela_username = Some(String::new());
        }
        if self.pixela_token.is_none() {
            self.pixela_token = Some(String::new());
        }
    }
}

impl Settings {
    pub fn save_to_file(&self) -> Result<()> {
        let path = ProjectDirs::from("romodoro", "romodoro", "romodoro")
            .ok_or(SettingsError::HomeDirNotFound)?;
        let path = path.config_dir();
        if !path.exists() {
            fs::create_dir_all(path)?
        }
        let toml_cfg: String =
            toml::to_string(&self).expect("Settings should be instantiated correctly");
        fs::write(path.join("config.toml"), toml_cfg)?;
        Ok(())
    }
    pub fn new() -> Result<Settings> {
        let path = ProjectDirs::from("romodoro", "romodoro", "romodoro")
            .ok_or(SettingsError::HomeDirNotFound)?;
        let path = path.config_dir();

        Ok(match &fs::read_to_string(path.join("config.toml")) {
            Ok(data) => toml::from_str(data).unwrap_or_default(),
            Err(_) => Settings::default(),
        })
    }
    pub fn restore_defaults(&mut self) {
        self.timer_settings = TimerSettings::default();
        self.ui_settings = UISettings::default();
    }
    pub fn iterations(&self) -> usize {
        self.timer_settings.iterations as usize
    }
    pub fn work_time(&self) -> usize {
        self.timer_settings.work_time as usize
    }
    pub fn break_time(&self) -> usize {
        self.timer_settings.break_time as usize
    }
    pub fn counter_mode(&self) -> CounterMode {
        self.timer_settings.mode
    }
    pub fn tab_right(&mut self) {
        self.selected_tab = self.selected_tab.right()
    }
    pub fn tab_left(&mut self) {
        self.selected_tab = self.selected_tab.left()
    }
    pub fn tab_up(&mut self) {
        let pomodoro_off = self.timer_settings.mode == CounterMode::Countup;
        self.selected_tab = self.selected_tab.up(pomodoro_off)
    }
    pub fn tab_down(&mut self) {
        let pomodoro_off = self.timer_settings.mode == CounterMode::Countup;
        self.selected_tab = self.selected_tab.down(pomodoro_off)
    }

    pub fn select_down(&mut self) {
        if self.selected_tab == SettingsTabs::Stats && !self.stats_setting.stats_on {
            return;
        }
        let num_of_settings = self.selected_tab.amount();
        if self.selected_setting < num_of_settings - 1 {
            self.selected_setting += 1;
        }
    }
    pub fn select_up(&mut self) {
        if self.selected_setting > 0 {
            self.selected_setting -= 1;
        }
    }
    pub fn decrement(&mut self) {
        if self.mode != Mode::Modify {
            return;
        }
        match self.selected_tab {
            SettingsTabs::Mode => self.timer_settings.mode.next(),
            SettingsTabs::Pomodoro => match self.selected_setting {
                0 if self.timer_settings.work_time - WORK_TIME_INCR != 0 => {
                    self.timer_settings.work_time = 10;
                }
                1 if self.timer_settings.break_time - BREAK_TIME_INCR != 0 => {
                    self.timer_settings.break_time -= BREAK_TIME_INCR
                }
                2 if self.timer_settings.iterations - 1 > 0 => self.timer_settings.iterations -= 1,
                _ => {}
            },
            SettingsTabs::Preferences => match self.selected_setting {
                0 => self.ui_settings.pause_after_state_change = true,
                1 => self.ui_settings.hide_work_countdown = true,
                _ => {}
            },
            SettingsTabs::Stats => {
                if let 0 = self.selected_setting {
                    self.set_stats(true);
                }
            }
        }
    }
    pub fn increment(&mut self) {
        if self.mode != Mode::Modify {
            return;
        }
        match self.selected_tab {
            SettingsTabs::Mode => self.timer_settings.mode.next(),
            SettingsTabs::Pomodoro => match self.selected_setting {
                0 => self.timer_settings.work_time += WORK_TIME_INCR,
                1 => self.timer_settings.break_time += BREAK_TIME_INCR,
                2 => self.timer_settings.iterations += 1,
                _ => {}
            },
            SettingsTabs::Preferences => match self.selected_setting {
                0 => self.ui_settings.pause_after_state_change = false,
                1 => self.ui_settings.hide_work_countdown = false,
                _ => {}
            },
            SettingsTabs::Stats => {
                if let 0 = self.selected_setting {
                    self.set_stats(false);
                }
            }
        }
    }
    fn do_pomodoro_settings_match(&self, timer: &Timer) -> bool {
        self.timer_settings.iterations == timer.total_iterations()
            && self.timer_settings.work_time == timer.work_time()
            && timer.counter_mode() == self.timer_settings.mode
            && self.timer_settings.break_time == timer.break_time()
    }
    pub fn do_settings_match(&self, timer: &Timer, stats: Option<&PixelaClient>) -> bool {
        let pomodoro_match = match timer.counter_mode() {
            CounterMode::Countup => self.timer_settings.mode == CounterMode::Countup,
            CounterMode::Countdown => self.do_pomodoro_settings_match(timer),
        };
        let stats_match = if let Some(pixela) = stats {
            self.stats_setting.stats_on
                && self
                    .stats_setting
                    .pixela_username
                    .clone()
                    .unwrap_or("".to_string())
                    == pixela.user.username()
                && self
                    .stats_setting
                    .pixela_token
                    .clone()
                    .unwrap_or("".to_string())
                    == pixela.user.token()
        } else {
            !self.stats_setting.stats_on
        };
        pomodoro_match && stats_match
    }
    pub fn set_stats(&mut self, on: bool) {
        self.stats_setting.stats_on = on;
        if self.stats_setting.stats_on {
            self.stats_setting.init_stats();
        }
    }
    pub fn change_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn input(&mut self, key: KeyEvent, clipboard_response: Option<String>) {
        match self.selected_setting {
            1 => Settings::process_input(
                key,
                self.stats_setting.pixela_username.as_mut().unwrap(),
                clipboard_response,
            ),
            2 => Settings::process_input(
                key,
                self.stats_setting.pixela_token.as_mut().unwrap(),
                clipboard_response,
            ),
            _ => {}
        }
    }
    pub fn process_input(
        key: KeyEvent,
        value_to_modify: &mut String,
        clipboard_response: Option<String>,
    ) {
        match key.code {
            KeyCode::Backspace if !value_to_modify.is_empty() => {
                value_to_modify.pop();
            }
            KeyCode::Char('v') if key.modifiers == KeyModifiers::CONTROL => {
                if let Some(clip) = clipboard_response {
                    value_to_modify.clear();
                    value_to_modify.push_str(&clip);
                }
            }
            KeyCode::Char(letter) => {
                value_to_modify.push(letter);
            }
            _ => {}
        };
    }
}
#[cfg(test)]
mod tests {
    use super::Settings;

    #[test]
    fn save_to_file() {
        let settings = Settings::default();
        settings.save_to_file();
    }
    #[test]
    fn read() {
        let settings = Settings::new();
        dbg!(settings);
    }
}
