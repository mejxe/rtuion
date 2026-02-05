use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{settings::Mode, utils::settings_helper_structs::SettingsTabs};

use super::{app_ui::AppWidget, settings_tab::SettingsTab, BLUE, GREEN, YELLOW};
#[derive(Clone, Copy)]
pub struct UISettingsTabData {
    pub selected_setting: u8,
    pub selected_tab: SettingsTabs,
    pub current_mode: Mode,
}

impl SettingsTab<'_> {
    pub fn highlight_selected(
        selected_num: u8,
        selected_tab: SettingsTabs,
        setting_tab: SettingsTabs,
        setting_num: u8,
        current_mode: Mode,
    ) -> Style {
        if setting_num == selected_num && selected_tab == setting_tab {
            match current_mode {
                Mode::Modify => Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
                Mode::Input => Style::default().fg(BLUE).add_modifier(Modifier::REVERSED),
                Mode::Normal => Style::default().fg(GREEN),
            }
        } else {
            Style::default().fg(GREEN)
        }
    }
    pub fn highlight_selected_tab(
        selected_tab: SettingsTabs,
        setting_tab: SettingsTabs,
        current_mode: Mode,
    ) -> BorderType {
        if selected_tab == setting_tab
            && (current_mode == Mode::Modify || current_mode == Mode::Input)
        {
            BorderType::QuadrantInside
        } else if selected_tab == setting_tab {
            BorderType::Double
        } else {
            BorderType::Rounded
        }
    }
}
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
pub struct UIHelper {}
impl<'a> UIHelper {
    pub fn create_settings_block(
        title: &str,
        color: Color,
        selected_tab: SettingsTabs,
        setting_tab: SettingsTabs,
        current_mode: Mode,
    ) -> Block<'a> {
        Block::default()
            .title(format!(" {} ", title))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(SettingsTab::highlight_selected_tab(
                selected_tab,
                setting_tab,
                current_mode,
            ))
            .border_style(Style::default().fg(color))
    }
    pub fn create_settings_paragraph(title: &str, highlight_style: Option<Style>) -> Paragraph<'a> {
        let style = if let Some(hi) = highlight_style {
            hi
        } else {
            Style::default().fg(Color::White)
        };
        Paragraph::new(Span::styled(title.to_string(), style))
            .alignment(Alignment::Center)
            .add_modifier(Modifier::BOLD)
    }
    pub fn format_yes_no(
        this_setting: u8,
        this_tab: SettingsTabs,
        tab_data: UISettingsTabData,
        value_that_highlights: bool,
        current_value: bool,
    ) -> Style {
        if value_that_highlights == current_value
            && tab_data.selected_setting == this_setting
            && tab_data.selected_tab == this_tab
            && tab_data.current_mode == Mode::Modify
        {
            Style::default().fg(YELLOW).underlined()
        } else if value_that_highlights == current_value {
            Style::default().fg(GREEN).underlined()
        } else {
            Style::default().fg(Color::White)
        }
    }
}
pub struct FooterHint {
    pub(crate) key: &'static str,
    pub(crate) hint: &'static str,
}

impl FooterHint {
    pub fn new(key: &'static str, hint: &'static str) -> Self {
        Self { key, hint }
    }
}
pub trait HintProvider {
    fn provide_hints(&self) -> Vec<FooterHint>;
}
impl HintProvider for AppWidget<'_> {
    fn provide_hints(&self) -> Vec<FooterHint> {
        vec![
            FooterHint::new("Tab", "Next tab"),
            FooterHint::new("S-Tab", "Prev tab"),
            FooterHint::new("Q", "Quit"),
        ]
    }
}
