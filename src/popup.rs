use crate::app::App;

#[derive(Debug)]
pub struct Popup {
    pub message: String,
    pub kind: PopupKind,
    pub interactive: bool,
}
#[derive(Debug)]
pub enum PopupKind {
    YesNoPopup(fn(&mut App)),
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
