use std::process::exit;

use ratatui::widgets::{List, ListState};

use crate::{app::App, stats::Pixel};

#[derive(Debug)]
pub struct Popup {
    pub message: String,
    pub kind: PopupKind,
    pub scrollable: bool,
}
type Callback = Box<dyn FnOnce(&mut App)>;

impl std::fmt::Debug for PopupKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PopupKind::YesNoPopup(_) => write!(f, "YesNoPopup(<callback>)"),
            PopupKind::SendPixelsPopup(_, pixels, _) => {
                write!(f, "SendPixelsPopup(<callback>, {:?})", pixels)
            }
            PopupKind::ListPopup(pixels, _) => write!(f, "ListPopup({:?})", pixels),
            PopupKind::ErrorPopup(e) => write!(f, "ErrorPopup({:?})", e),
        }
    }
}
pub enum PopupKind {
    YesNoPopup(Callback),
    SendPixelsPopup(Callback, Vec<Pixel>, ListState),
    ListPopup(Vec<Pixel>, ListState),
    ErrorPopup(crate::error::Error),
}
impl Popup {
    pub fn new(message: String, kind: PopupKind) -> Self {
        let scrollable = match kind {
            PopupKind::YesNoPopup(_) => false,
            PopupKind::ErrorPopup(_) => false,
            PopupKind::SendPixelsPopup(_, _, _) => true,
            PopupKind::ListPopup(_, _) => true,
        };
        Self {
            message,
            kind,
            scrollable,
        }
    }
    pub fn yes_no(message: String, callback: Callback) -> Self {
        Self {
            message,
            kind: PopupKind::YesNoPopup(callback),
            scrollable: false,
        }
    }
    pub fn pixel_confirm_list(message: String, callback: Callback, pixels: Vec<Pixel>) -> Self {
        Self {
            message,
            kind: PopupKind::SendPixelsPopup(callback, pixels, ListState::default()),
            scrollable: true,
        }
    }
    pub fn pixel_list(message: String, pixels: Vec<Pixel>) -> Self {
        Self {
            message,
            kind: PopupKind::ListPopup(pixels, ListState::default()),
            scrollable: true,
        }
    }
    pub fn scroll_down(&mut self, viewport_height: usize) {
        match &mut self.kind {
            PopupKind::ListPopup(pixels, state) | PopupKind::SendPixelsPopup(_, pixels, state) => {
                let total_items = pixels.len();
                let current_offset = state.offset();
                if current_offset == (total_items.saturating_sub(viewport_height)) {
                    return;
                }
                let new_offset =
                    (current_offset + 1).min(total_items.saturating_sub(viewport_height));

                *state.offset_mut() = new_offset;

                let last_visible = new_offset + viewport_height.saturating_sub(1);
                state.select(Some(last_visible.min(total_items.saturating_sub(1))));
            }
            _ => {}
        }
    }

    pub fn scroll_up(&mut self, viewport_height: usize) {
        match &mut self.kind {
            PopupKind::ListPopup(pixels, state) | PopupKind::SendPixelsPopup(_, pixels, state) => {
                let current_offset = state.offset();
                let new_offset = current_offset.saturating_sub(1);

                *state.offset_mut() = new_offset;

                let last_visible = new_offset + viewport_height.saturating_sub(1);
                state.select(Some(last_visible.min(pixels.len().saturating_sub(1))));
            }
            _ => {}
        }
    }
}
impl From<crate::error::Error> for Popup {
    fn from(value: crate::error::Error) -> Self {
        Popup {
            message: value.to_string(),
            kind: PopupKind::ErrorPopup(value),
            scrollable: false,
        }
    }
}
