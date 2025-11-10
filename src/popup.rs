
use crate::{app::App, stats::Pixel};

#[derive(Debug)]
pub struct Popup {
    pub message: String,
    pub kind: PopupKind,
    pub interactive: bool,
}
type Callback = Box<dyn FnOnce(&mut App)>;

impl std::fmt::Debug for PopupKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PopupKind::YesNoPopup(_) => write!(f, "YesNoPopup(<callback>)"),
            PopupKind::SendPixelsPopup(_, pixels) => {
                write!(f, "SendPixelsPopup(<callback>, {:?})", pixels)
            }
            PopupKind::ListPopup(pixels) => write!(f, "ListPopup({:?})", pixels),
            PopupKind::ErrorPopup(e) => write!(f, "ErrorPopup({:?})", e),
        }
    }
}
pub enum PopupKind {
    YesNoPopup(Callback),
    SendPixelsPopup(Callback, Vec<Pixel>),
    ListPopup(Vec<Pixel>),
    ErrorPopup(crate::error::Error),
}
impl Popup {
    pub fn new(message: String, kind: PopupKind) -> Self {
        let interactive = match kind {
            PopupKind::YesNoPopup(_) => true,
            PopupKind::ErrorPopup(_) => false,
            PopupKind::SendPixelsPopup(_, _) => true,
            PopupKind::ListPopup(_) => false,
        };
        Self {
            message,
            kind,
            interactive,
        }
    }
    pub fn yes_no(message: String, callback: Callback) -> Self {
        Self {
            message,
            kind: PopupKind::YesNoPopup(callback),
            interactive: true,
        }
    }
    pub fn pixel_confirm_list(message: String, callback: Callback, pixels: Vec<Pixel>) -> Self {
        Self {
            message,
            kind: PopupKind::SendPixelsPopup(callback, pixels),
            interactive: true,
        }
    }
    pub fn pixel_list(message: String, pixels: Vec<Pixel>) -> Self {
        Self {
            message,
            kind: PopupKind::ListPopup(pixels),
            interactive: false,
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
