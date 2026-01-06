use std::cell::RefCell;
use std::rc::Rc;

use crossterm::terminal;
use pomodoro::app::*;
use pomodoro::error::Result;
use pomodoro::popup::Popup;
use pomodoro::romodoro::*;
use pomodoro::settings::Settings;

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = tokio::sync::mpsc::channel(4);
    let (tx_events, rx_events) = tokio::sync::mpsc::channel(32);
    let (tx_commands, rx_commands) = tokio::sync::mpsc::channel(4);
    let settings_manager = Rc::new(RefCell::new(Settings::new()?));
    let mut pomodoro = Pomodoro::new(tx, rx_commands, tx_commands, settings_manager.clone());

    terminal::enable_raw_mode()?;
    let mut terminal = ratatui::init();

    let mut pixela_failed_popup: Option<Popup> = None;
    if settings_manager.borrow().stats_setting.stats_on {
        // init pixela module
        if let Err(err) = pomodoro.try_init_pixela_client() {
            pixela_failed_popup = Some(err.into());
        }
    }

    let mut app = App::new(pomodoro, settings_manager, tx_events);
    app.set_popup_opt(pixela_failed_popup);
    let app_result = app.run(&mut terminal, rx_events, rx).await; // mainloop
    terminal::disable_raw_mode()?;

    ratatui::restore();
    app.get_settings_ref().borrow().save_to_file()?;
    Ok(app_result?)
}
