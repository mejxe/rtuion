use std::cell::RefCell;
use std::rc::Rc;

use crossterm::terminal;
use pomodoro::app::*;
use pomodoro::error::Result;
use pomodoro::romodoro::*;
use pomodoro::settings::SettingsTab;
// ALPHA 0.1

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = tokio::sync::mpsc::channel(4);
    let (tx_events, rx_events) = tokio::sync::mpsc::channel(32);
    let (tx_commands, rx_commands) = tokio::sync::mpsc::channel(4);
    let settings = Rc::new(RefCell::new(SettingsTab::new()?));
    let pomodoro = Pomodoro::new(tx, rx_commands, tx_commands, settings.clone());
    terminal::enable_raw_mode()?;
    let mut terminal = ratatui::init();
    let mut app = App::new(pomodoro, settings, tx_events);
    let app_result = app.run(&mut terminal, rx_events, rx).await; // mainloop
    terminal::disable_raw_mode()?;

    ratatui::restore();
    app.get_settings_ref().borrow().save_to_file()?;
    Ok(app_result?)
}
