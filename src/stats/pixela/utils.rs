use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct PixelaResponse {
    #[serde(alias = "isSuccess")]
    is_success: bool,
    #[serde(alias = "isRejected")]
    is_rejected: Option<bool>,
    message: String,
}
#[derive(Deserialize, Serialize)]
pub struct PixelaValue {
    quantity: String,
}

impl PixelaValue {
    pub fn quantity(&self) -> &str {
        &self.quantity
    }
}
#[derive(Debug)]
pub struct StatefulList<T> {
    items: Vec<T>,
    state: ListState,
}
impl<T> StatefulList<T> {
    pub fn new(items: Vec<T>) -> StatefulList<T> {
        Self {
            items,
            state: ListState::default(),
        }
    }
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
    }

    pub fn push(&mut self, value: T) {
        self.items.push(value)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn state(&self) -> &ListState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ListState {
        &mut self.state
    }

    pub fn select_next(&mut self) {
        self.state.select_next()
    }

    pub fn select_previous(&mut self) {
        self.state.select_previous()
    }
    pub fn refresh_state(&mut self) {
        if let Some(selected) = self.state.selected() {
            if selected >= self.items.len() {
                if !self.items.is_empty() {
                    self.state.select(Some(self.items.len() - 1));
                } else {
                    self.state.select(None);
                }
            }
        }
    }

    pub fn items_mut(&mut self) -> &mut Vec<T> {
        &mut self.items
    }
}
