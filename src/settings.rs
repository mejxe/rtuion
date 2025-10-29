use std::process::exit;
use std::{fs, io};

use crate::error::{Result, SettingsError};
use crate::stats::Subject;
use crate::{
    timer::*, BREAK_TIME_INCR, DEFAULT_BREAK, DEFAULT_ITERATIONS, DEFAULT_WORK, WORK_TIME_INCR,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use directories::ProjectDirs;
use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum Mode {
    Input,
    #[default]
    Normal,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsTab {
    #[serde(skip)]
    pub selected_setting: u8,
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
    pub work_time: i64,
    pub break_time: i64,
    pub iterations: u8,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PomodoroSettings {
    WorkTime(Option<i64>),
    BreakTime(Option<i64>),
    Iterations(Option<u8>),
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

impl SettingsTab {
    pub fn save_to_file(&self) -> Result<()> {
        let path = ProjectDirs::from("romodoro", "mejxedev", "romodoro")
            .ok_or(SettingsError::HomeDirNotFound)?;
        let path = path.config_dir();
        if !path.exists() {
            fs::create_dir(path)?
        }
        let toml_cfg: String =
            toml::to_string(&self).expect("Settings should be instantiated correctly");
        fs::write(path.join("config.toml"), toml_cfg)?;
        Ok(())
    }
    pub fn new() -> Result<SettingsTab> {
        let path = ProjectDirs::from("romodoro", "mejxedev", "romodoro")
            .ok_or(SettingsError::HomeDirNotFound)?;
        let path = path.config_dir();

        Ok(match &fs::read_to_string(path.join("config.toml")) {
            Ok(data) => toml::from_str(data).unwrap_or_default(),
            Err(_) => SettingsTab::default(),
        })
    }
    pub fn restore_defaults(&mut self) {
        self.timer_settings = TimerSettings::default();
        self.ui_settings = UISettings::default();
    }

    pub fn get_pomodoro_setting(&self, setting: PomodoroSettings) -> PomodoroSettings {
        match setting {
            PomodoroSettings::BreakTime(_) => {
                PomodoroSettings::BreakTime(Some(self.timer_settings.break_time))
            }
            PomodoroSettings::WorkTime(_) => {
                PomodoroSettings::WorkTime(Some(self.timer_settings.work_time))
            }
            PomodoroSettings::Iterations(_) => {
                PomodoroSettings::Iterations(Some(self.timer_settings.iterations))
            }
        }
    }

    pub fn select_down(&mut self) {
        if !self.stats_setting.stats_on && (self.selected_setting == 3) {
            self.selected_setting = 5;
        }

        if self.selected_setting == 7 {
            self.selected_setting = 0;
        } else {
            self.selected_setting += 1
        }
    }
    pub fn select_up(&mut self) {
        if !self.stats_setting.stats_on && (self.selected_setting == 6) {
            self.selected_setting = 4;
        }
        if self.selected_setting == 0 {
            self.selected_setting = 7;
        } else {
            self.selected_setting -= 1
        }
    }
    pub fn decrement(&mut self) {
        match self.selected_setting {
            0 if self.timer_settings.work_time - WORK_TIME_INCR != 0 => {
                self.timer_settings.work_time = 10;
            }
            1 if self.timer_settings.break_time - BREAK_TIME_INCR != 0 => {
                self.timer_settings.break_time -= BREAK_TIME_INCR
            }
            2 if self.timer_settings.iterations - 1 > 0 => self.timer_settings.iterations -= 1,
            3 => self.stats_setting.stats_on = !self.stats_setting.stats_on,
            6 => {
                self.ui_settings.pause_after_state_change =
                    !self.ui_settings.pause_after_state_change
            }
            7 => self.ui_settings.hide_work_countdown = !self.ui_settings.hide_work_countdown,
            _ => {}
        }
    }
    pub fn increment(&mut self) {
        match self.selected_setting {
            0 => self.timer_settings.work_time += WORK_TIME_INCR,
            1 => self.timer_settings.break_time += BREAK_TIME_INCR,
            2 => self.timer_settings.iterations += 1,
            3 => self.set_stats(),
            6 => {
                self.ui_settings.pause_after_state_change =
                    !self.ui_settings.pause_after_state_change
            }
            7 => self.ui_settings.hide_work_countdown = !self.ui_settings.hide_work_countdown,
            _ => {}
        }
    }
    pub fn set_stats(&mut self) {
        self.stats_setting.stats_on = !self.stats_setting.stats_on;
        if self.stats_setting.stats_on {
            self.stats_setting.init_stats();
        }
    }
    pub fn change_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
    pub fn mode(&self) -> &Mode {
        &self.mode
    }
    pub fn input(&mut self, key: KeyEvent, clipboard_response: Option<String>) {
        match self.selected_setting {
            4 => SettingsTab::process_input(
                key,
                self.stats_setting.pixela_username.as_mut(),
                clipboard_response,
            ),
            5 => SettingsTab::process_input(
                key,
                self.stats_setting.pixela_token.as_mut(),
                clipboard_response,
            ),
            _ => {}
        }
    }
    pub fn process_input(
        key: KeyEvent,
        value_to_modify: Option<&mut String>,
        clipboard_response: Option<String>,
    ) {
        if let Some(val_to_modify) = value_to_modify {
            match key.code {
                KeyCode::Backspace if !val_to_modify.is_empty() => {
                    val_to_modify.pop();
                }
                KeyCode::Char('v') if key.modifiers == KeyModifiers::CONTROL => {
                    if let Some(clip) = clipboard_response {
                        val_to_modify.clear();
                        val_to_modify.push_str(&clip);
                    }
                }
                KeyCode::Char(letter) => {
                    val_to_modify.push(letter);
                }
                _ => {}
            }
        }
    }
}
impl Default for TimerSettings {
    fn default() -> Self {
        TimerSettings {
            work_time: DEFAULT_WORK,
            break_time: DEFAULT_BREAK,
            iterations: DEFAULT_ITERATIONS,
        }
    }
}
impl From<PomodoroState> for PomodoroSettings {
    fn from(value: PomodoroState) -> Self {
        match value {
            PomodoroState::Work(time) => PomodoroSettings::WorkTime(Some(time)),
            PomodoroState::Break(time) => PomodoroSettings::BreakTime(Some(time)),
        }
    }
}
impl From<u8> for PomodoroSettings {
    fn from(value: u8) -> Self {
        PomodoroSettings::Iterations(Some(value))
    }
}
#[cfg(test)]
mod tests {
    use super::SettingsTab;

    #[test]
    fn save_to_file() {
        let settings = SettingsTab::default();
        settings.save_to_file();
    }
    #[test]
    fn read() {
        let settings = SettingsTab::new();
        dbg!(settings);
    }
}
