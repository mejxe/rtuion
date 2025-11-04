use futures::stream::Any;
use ratatui::widgets::{List, ListState};

use crate::{app::App, pixela_client::StatefulList, stats::Pixel};

#[derive(Debug)]
pub struct Popup {
    pub message: String,
    pub kind: PopupKind,
    pub interactive: bool,
}
#[derive(Debug)]
pub enum PopupKind {
    YesNoPopup(fn(&mut App)),
    SendPixelsPopup(fn(&mut App), Vec<Pixel>),
    ErrorPopup(crate::error::Error),
}
impl PopupKind {
    pub fn yes_no(callback: fn(&mut App)) -> PopupKind {
        PopupKind::YesNoPopup(callback)
    }
}
impl Popup {
    pub fn new(message: String, kind: PopupKind) -> Self {
        let interactive = match kind {
            PopupKind::YesNoPopup(_) => true,
            PopupKind::ErrorPopup(_) => false,
            PopupKind::SendPixelsPopup(_, _) => true,
        };
        Self {
            message,
            kind,
            interactive,
        }
    }
    pub fn yes_no(message: String, callback: fn(&mut App)) -> Self {
        Self {
            message,
            kind: PopupKind::YesNoPopup(callback),
            interactive: true,
        }
    }
    pub fn pixel_list(message: String, callback: fn(&mut App), pixels: Vec<Pixel>) -> Self {
        Self {
            message,
            kind: PopupKind::SendPixelsPopup(callback, pixels),
            interactive: true,
        }
    }
}
impl From<crate::error::Error> for Popup {
    fn from(value: crate::error::Error) -> Self {
        Popup {
            message: value.to_string(),
            kind: PopupKind::ErrorPopup(value),
            interactive: false,
        }
    }
}
