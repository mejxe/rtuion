use std::process::exit;

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::App,
    popup::{Popup, PopupKind},
};

impl App {
    pub async fn handle_popups(
        &mut self,
        key_event: KeyEvent,
        mut popup: Popup,
        list_height: usize,
    ) {
        if popup.scrollable {
            match key_event.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    popup.scroll_down(list_height);
                    self.set_popup(popup);
                    return;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    popup.scroll_up(list_height);
                    self.set_popup(popup);
                    return;
                }
                _ => {}
            }
        }
        self.set_popup(popup);
        if let Some(popup) = self.take_popup() {
            match popup.kind {
                PopupKind::YesNoPopup(callback) | PopupKind::SendPixelsPopup(callback, _, _) => {
                    match key_event.code {
                        KeyCode::Char('y') => {
                            callback(self);
                            self.clear_popup()
                        }
                        KeyCode::Char('n') => self.clear_popup(),
                        _ => {}
                    }
                }
                PopupKind::ErrorPopup(_) | PopupKind::ListPopup(_, _) => self.clear_popup(),
            }
        }
    }
}
