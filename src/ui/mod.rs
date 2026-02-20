pub mod app_ui;
mod assets;
pub mod graph;
pub mod helpers;
pub mod pomodoro_tab;
pub mod popup;
pub mod settings_tab;
pub mod stats_tab;
pub mod ui_utils;

use ratatui::style::Color;
pub const YELLOW: Color = Color::Rgb(215, 153, 33);
pub const BLUE: Color = Color::Rgb(69, 133, 136);
pub const GREEN: Color = Color::Rgb(142, 192, 124);
pub const RED: Color = Color::Rgb(204, 36, 29);
pub const ORANGE: Color = Color::Rgb(240, 128, 25);
pub const BG: Color = Color::Rgb(40, 40, 40);
