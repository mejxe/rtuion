use super::{
    pomodoro_tab::PomodoroTab, settings_tab::SettingsTab, stats_tab::StatsTab,
    ui_utils::FooterHint, YELLOW,
};
use crate::{app::App, ui::ui_utils::HintProvider, ui::BG, utils::tabs};
use ratatui::{
    self,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph, Tabs, Widget},
    Frame,
};
pub struct AppWidget<'a> {
    app_context: &'a mut App,
}

impl Widget for &mut AppWidget<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let tabs = ["Timer", "Settings", "Stats"];
        let tab_titles: Vec<Span> = tabs
            .iter()
            .map(|t| Span::styled(*t, Style::default().fg(Color::White)))
            .collect();
        let selected_tab = self.app_context.selected_tab();

        let tabs_widget = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Menu ")
                    .border_style(Style::default().fg(YELLOW)),
            )
            .highlight_style(Style::default().fg(Color::Rgb(240, 94, 90)))
            .select::<usize>(selected_tab.into());

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tab titles
                Constraint::Min(1),    // Main content area
                Constraint::Length(1), // footer
            ])
            .split(area);

        let tab_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(29), Constraint::Min(1)])
            .split(layout[0]);

        if let Some(popup) = self.app_context.popup_as_mut() {
            popup.render(area, buf);
            return;
        }
        match selected_tab {
            tabs::Tabs::TimerTab => {
                if window_too_small(20, 10, area, buf) {
                    return;
                }
                let pomodoro_tab = PomodoroTab::new(self.app_context.pomodoro());
                let mut hints = pomodoro_tab.provide_hints();
                hints.append(&mut self.provide_hints());
                pomodoro_tab.render(layout[1], buf);
                self.render_footer(layout[2], buf, hints);
            }
            tabs::Tabs::SettingsTab => {
                if window_too_small(65, 30, area, buf) {
                    return;
                }
                let settings_guard = self.app_context.get_settings_ref();
                let settings = settings_guard.borrow();
                let timer = &self.app_context.pomodoro().timer;
                let settings_tab = SettingsTab::new(
                    &settings,
                    timer,
                    self.app_context.pomodoro().pixela_client(),
                );
                let hints = settings_tab.provide_hints();

                settings_tab.render(layout[1], buf);
                self.render_footer(layout[2], buf, hints);
            }
            tabs::Tabs::StatsTab => {
                if window_too_small(65, 30, area, buf) {
                    return;
                }
                let (rendered_stats, hints) = if let Some(stats_client) =
                    self.app_context.pomodoro_mut().pixela_client_as_mut()
                {
                    let mut stats = StatsTab::new(stats_client);
                    let hints = stats.provide_hints();
                    stats.render(layout[1], buf);
                    (true, hints)
                } else {
                    (false, self.provide_hints())
                };
                if !rendered_stats {
                    self.render_stats(layout[1], buf);
                }
                self.render_footer(layout[2], buf, hints);
            }
        }
        tabs_widget.render(tab_layout[0], buf);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = buf.cell_mut((x, y)).expect("Should work");
                if cell.style().bg == Some(Color::Reset) {
                    cell.set_style(cell.style().bg(BG));
                }
            }
        }
    }
}
fn window_too_small(
    min_width: u16,
    min_height: u16,
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
) -> bool {
    if !(area.width < min_width || area.height < min_height) {
        return false;
    }
    let warning = Paragraph::new(format!(
        "Terminal too small {}x{}, please resize to at least {}x{}",
        area.width, area.height, min_width, min_height
    ))
    .centered();
    warning.render(area, buf);
    return true;
}
impl<'a> AppWidget<'a> {
    pub fn new(app_context: &'a mut App) -> Self {
        Self { app_context }
    }

    pub fn render_footer(&self, area: Rect, buf: &mut Buffer, hints: Vec<FooterHint>) {
        let footer_text = hints
            .iter()
            .map(|hint| format!("{}: {}", hint.key, hint.hint))
            .collect::<Vec<_>>()
            .join(" | ");

        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            );

        footer.render(area, buf);
    }
    fn render_stats(&self, area: Rect, buf: &mut Buffer) {
        let text = Paragraph::new("Stats are turned off".to_string())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightGreen));
        text.render(area, buf);
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}
