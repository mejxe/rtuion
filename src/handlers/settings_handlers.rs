
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    app::App,
    settings::Mode,
    utils::settings_helper_structs::SettingsTabs,
};

impl App {
    pub async fn handle_settings_modify(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => self.settings().borrow_mut().select_down(),
            KeyCode::Up | KeyCode::Char('k') => self.settings().borrow_mut().select_up(),
            KeyCode::Right | KeyCode::Char('l') => self.settings().borrow_mut().increment(),
            KeyCode::Left | KeyCode::Char('h') => self.settings().borrow_mut().decrement(),
            KeyCode::Esc => self.settings().borrow_mut().change_mode(Mode::Normal),
            KeyCode::Char(' ') => self.update_settings().await,
            KeyCode::Enter
                if (self.settings().borrow().selected_setting >= 1
                    && self.settings().borrow().selected_tab == SettingsTabs::Stats) =>
            {
                self.settings().borrow_mut().change_mode(Mode::Input)
            }
            _ => {}
        }
    }
    pub async fn handle_settings_normal(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => self.settings().borrow_mut().tab_down(),
            KeyCode::Up | KeyCode::Char('k') => self.settings().borrow_mut().tab_up(),
            KeyCode::Right | KeyCode::Char('l') => self.settings().borrow_mut().tab_right(),
            KeyCode::Left | KeyCode::Char('h') => self.settings().borrow_mut().tab_left(),
            KeyCode::Char('R') => self.settings().borrow_mut().restore_defaults(),
            KeyCode::Char(' ') => self.update_settings().await,
            KeyCode::Enter => {
                self.settings().borrow_mut().change_mode(Mode::Modify);
                self.settings().borrow_mut().selected_setting = 0
            }
            _ => {}
        }
    }
    pub async fn handle_settings_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(' ') => {},
            KeyCode::Esc | KeyCode::Enter => self.settings().borrow_mut().change_mode(Mode::Modify),
            key => {
                let mut text = None;
                if key == KeyCode::Char('v') && key_event.modifiers == KeyModifiers::CONTROL {
                    if let Some(clip) = self.clipboard_mut().as_mut() {
                        text = Some(
                            clip.get_text()
                                .unwrap()
                                .split_whitespace()
                                .next()
                                .unwrap()
                                .to_string(),
                        )
                    }
                }
                self.settings().borrow_mut().input(key_event, text);
            }
        }
    }
}
