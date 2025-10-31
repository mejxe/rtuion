use super::{ui_utils::*, BG, BLUE, GREEN, RED, YELLOW};
use crate::pixela_client::PixelaClient;
use crate::romodoro::Pomodoro;
use crate::settings::SettingsTab;
use crate::stats::{Pixel, Subject};
use crate::{app::App, settings::Mode};
use ratatui::{
    self,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph, Tabs, Widget},
    Frame,
};

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let tabs = ["Pomodoro Timer", "Settings", "Stats"];
        let tab_titles: Vec<Span> = tabs
            .iter()
            .map(|t| Span::styled(*t, Style::default().fg(Color::White)))
            .collect();
        let selected_tab = self.get_selected_tab();

        let tabs_widget = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Menu ")
                    .border_style(Style::default().fg(YELLOW)),
            )
            .highlight_style(Style::default().fg(Color::Rgb(240, 94, 90)))
            .select(selected_tab as usize);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tab titles
                Constraint::Min(1),    // Main content area
                Constraint::Max(1),    // footer
            ])
            .split(area);

        let tab_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(38), Constraint::Min(1)])
            .split(layout[0]);

        tabs_widget.render(tab_layout[0], buf);

        let popup_area = centered_rect(40, 30, area);
        let popup_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(BG));
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(30)])
            .vertical_margin(0)
            .split(popup_area);
        let popup_yes_no_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .horizontal_margin(0)
            .split(popup_layout[1]);
        let yes_paragraph = Paragraph::new("<y>es")
            .alignment(Alignment::Center)
            .centered()
            .style(Style::default().fg(GREEN))
            .block(
                Block::default()
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            );
        let no_paragraph = Paragraph::new("<n>o")
            .alignment(Alignment::Center)
            .style(Style::default().fg(RED))
            .block(
                Block::default()
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            );
        let question_paragraph = Paragraph::new("This will reset your current timer!")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded)
                    .title("You sure?"),
            );
        match selected_tab {
            0 => self.get_pomodoro_ref().render(layout[1], buf),
            1 => self.get_settings_ref().borrow().render(layout[1], buf),
            2 => {
                if let Some(stats_client) = self.get_pomodoro_ref_mut().pixela_client_as_mut() {
                    stats_client.render(layout[1], buf);
                } else {
                    self.render_stats(layout[1], buf);
                }
            }
            _ => {}
        }
        if self.get_show_popup() && selected_tab == 1 {
            Clear.render(area, buf);
            popup_block.render(popup_area, buf);
            question_paragraph.render(popup_layout[0], buf);
            yes_paragraph.render(popup_yes_no_layout[0], buf);
            no_paragraph.render(popup_yes_no_layout[1], buf);
            return;
        }
        self.render_footer(layout[2], buf);
    }
}
impl App {
    pub fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let footer_text = match self.get_selected_tab() {
            0 => "Space: Start/Stop | Tab: Next Tab | Q: Quit",
            1 if self.get_settings_ref().borrow().mode() == &Mode::Input => "Input Mode On | Esc: Normal Mode",
            1 if [4,5].contains( &self.get_settings_ref().borrow().selected_setting )=> "↑↓: Select | Enter: Input Mode | Space: Confirm | Tab: Next Tab | r: Restore Defaults | Q: Quit |" ,
            1 => "↑↓: Select | ←→: Adjust Value | Space: Confirm | Tab: Next Tab | r: Restore Defaults | Q: Quit |" ,
            _ => "Tab: Next Tab | Q: Quit",
        };

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
