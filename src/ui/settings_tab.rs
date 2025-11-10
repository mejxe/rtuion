use crate::settings::SettingsTab;
use crate::ui::{BG, BLUE, RED, YELLOW};
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::{Color, Stylize};
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

impl Widget for &SettingsTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create vertical layout with three panels
        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(5),  // Top gap
                Constraint::Percentage(25), // Pomodoro Settings
                Constraint::Percentage(5),  // Gap
                Constraint::Percentage(30), // Stats Settings
                Constraint::Percentage(5),  // Gap
                Constraint::Percentage(25), // Other Settings
                Constraint::Percentage(5),  // Bottom gap
            ])
            .split(area);

        // Create horizontal layout for centering panels
        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Left margin
                Constraint::Percentage(40), // Panel width
                Constraint::Percentage(30), // Right margin
            ]);

        let pomodoro_layout = horizontal_layout.split(outer_layout[1]);
        let stats_layout = horizontal_layout.split(outer_layout[3]);
        let other_layout = horizontal_layout.split(outer_layout[5]);

        let pomodoro_settings_area = pomodoro_layout[1];
        let stats_settings_area = stats_layout[1];
        let other_settings_area = other_layout[1];

        // Create panel blocks
        let pomodoro_box =
            SettingsBlock::create_settings_block("Pomodoro Settings", BLUE, BorderType::Rounded);

        let stats_box =
            SettingsBlock::create_settings_block("Stats Settings", YELLOW, BorderType::Rounded);

        let other_settings_box =
            SettingsBlock::create_settings_block("Other Settings", RED, BorderType::Rounded);

        let pomodoro_inner_area = pomodoro_box.inner(pomodoro_settings_area);
        let stats_inner_area = stats_box.inner(stats_settings_area);
        let other_inner_area = other_settings_box.inner(other_settings_area);

        // Create inner layouts
        let pomodoro_inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(5),  // Top margin
                Constraint::Percentage(15), // Work Time label
                Constraint::Percentage(15), // Work Time value
                Constraint::Percentage(15), // Break Time label
                Constraint::Percentage(15), // Break Time value
                Constraint::Percentage(15), // Iterations label
                Constraint::Percentage(15), // Iterations value
                Constraint::Percentage(5),  // Bottom margin
            ])
            .split(pomodoro_inner_area);

        let stats_inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(5),  // Top margin
                Constraint::Percentage(12), // Stats on label
                Constraint::Percentage(12), // Stats on value
                Constraint::Percentage(12), // Username label
                Constraint::Percentage(12), // Username value
                Constraint::Percentage(12), // Token label
                Constraint::Percentage(12), // Token value
                Constraint::Percentage(5),  // Bottom margin
            ])
            .split(stats_inner_area);

        let other_inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(5),  // Top margin
                Constraint::Percentage(15), // Pause setting label
                Constraint::Percentage(15), // Pause setting value
                Constraint::Percentage(15), // Hide clock label
                Constraint::Percentage(15), // Hide clock value
                Constraint::Percentage(5),  // Bottom margin
            ])
            .split(other_inner_area);

        // Render the panel blocks
        pomodoro_box.render(pomodoro_settings_area, buf);
        stats_box.render(stats_settings_area, buf);
        other_settings_box.render(other_settings_area, buf);

        // Pomodoro Settings content
        let pomodoro_settings = PomodoroTabSettings::new(self);

        // Stats Settings content
        let stats_settings = StatsTabSettings::new(self);

        // Other Settings content
        let other_settings = OtherTabSettings::new(self);

        pomodoro_settings
            .work_time_text
            .render(pomodoro_inner_layout[1], buf);
        pomodoro_settings
            .work_time_value
            .render(pomodoro_inner_layout[2], buf);
        pomodoro_settings
            .break_time_text
            .render(pomodoro_inner_layout[3], buf);
        pomodoro_settings
            .break_time_value
            .render(pomodoro_inner_layout[4], buf);
        pomodoro_settings
            .iterations_text
            .render(pomodoro_inner_layout[5], buf);
        pomodoro_settings
            .iterations_value
            .render(pomodoro_inner_layout[6], buf);

        stats_settings
            .stats_on_text
            .render(stats_inner_layout[1], buf);
        stats_settings
            .stats_on_value
            .render(stats_inner_layout[2], buf);
        stats_settings
            .username_text
            .render(stats_inner_layout[3], buf);
        stats_settings
            .username_value
            .render(stats_inner_layout[4], buf);
        stats_settings.token_text.render(stats_inner_layout[5], buf);
        stats_settings
            .token_value
            .render(stats_inner_layout[6], buf);

        other_settings
            .pause_change_state_text
            .render(other_inner_layout[1], buf);
        other_settings
            .pause_change_state_value
            .render(other_inner_layout[2], buf);
        other_settings
            .hide_clock_text
            .render(other_inner_layout[3], buf);
        other_settings
            .hide_clock_value
            .render(other_inner_layout[4], buf);

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
pub struct PomodoroTabSettings<'a> {
    pub work_time_text: Paragraph<'a>,
    pub work_time_value: Paragraph<'a>,
    pub break_time_text: Paragraph<'a>,
    pub break_time_value: Paragraph<'a>,
    pub iterations_text: Paragraph<'a>,
    pub iterations_value: Paragraph<'a>,
}
impl<'a> PomodoroTabSettings<'a> {
    pub fn new(settings: &SettingsTab) -> PomodoroTabSettings<'a> {
        let work_time_text = SettingsBlock::create_settings_paragraph("Work Time", None);
        let work_time_value = SettingsBlock::create_settings_paragraph(
            &format!("{} min", (settings.timer_settings.work_time / 60)),
            Some(settings.highlight_selected(0)),
        );
        let break_time_text = SettingsBlock::create_settings_paragraph("Break Time", None);
        let break_time_value = SettingsBlock::create_settings_paragraph(
            &format!("{} min", (settings.timer_settings.break_time / 60)),
            Some(settings.highlight_selected(1)),
        );

        let iterations_text = SettingsBlock::create_settings_paragraph("Iterations", None);
        let iterations_value = SettingsBlock::create_settings_paragraph(
            &format!("{} iters", settings.timer_settings.iterations),
            Some(settings.highlight_selected(2)),
        );
        PomodoroTabSettings {
            work_time_text,
            work_time_value,
            break_time_text,
            break_time_value,
            iterations_text,
            iterations_value,
        }
    }
}
pub struct StatsTabSettings<'a> {
    pub stats_on_text: Paragraph<'a>,
    pub stats_on_value: Paragraph<'a>,
    pub username_text: Paragraph<'a>,
    pub username_value: Paragraph<'a>,
    pub token_text: Paragraph<'a>,
    pub token_value: Paragraph<'a>,
}

impl<'a> StatsTabSettings<'a> {
    pub fn new(settings: &SettingsTab) -> StatsTabSettings<'a> {
        let stats_on_text = SettingsBlock::create_settings_paragraph("Stats Tracking", None);
        let stats_on_val = if settings.stats_setting.stats_on {
            "on"
        } else {
            "off"
        };
        let stats_on_value = SettingsBlock::create_settings_paragraph(
            stats_on_val,
            Some(settings.highlight_selected(3)),
        );

        let mut username_text = SettingsBlock::create_settings_paragraph("Pixela Username", None);
        let mut username_value = SettingsBlock::create_settings_paragraph(
            settings
                .stats_setting
                .pixela_username
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("[not set]"),
            Some(settings.highlight_selected(4)),
        );

        let mut token_text = SettingsBlock::create_settings_paragraph("API Token", None);
        let token_display = settings
            .stats_setting
            .pixela_token
            .as_ref()
            .map(|t| "*".repeat(t.len().min(20)))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "[not set]".to_string());
        let mut token_value = SettingsBlock::create_settings_paragraph(
            &token_display,
            Some(settings.highlight_selected(5)),
        );
        if !settings.stats_setting.stats_on {
            username_text = username_text.dark_gray();
            username_value = username_value.dark_gray();
            token_text = token_text.dark_gray();
            token_value = token_value.dark_gray();
        }

        StatsTabSettings {
            stats_on_text,
            stats_on_value,
            username_text,
            username_value,
            token_text,
            token_value,
        }
    }
}

pub struct OtherTabSettings<'a> {
    pub pause_change_state_text: Paragraph<'a>,
    pub pause_change_state_value: Paragraph<'a>,
    pub hide_clock_text: Paragraph<'a>,
    pub hide_clock_value: Paragraph<'a>,
}

impl<'a> OtherTabSettings<'a> {
    pub fn new(settings: &SettingsTab) -> OtherTabSettings<'a> {
        let pause_change_state_text =
            SettingsBlock::create_settings_paragraph("Pause before new iteration", None);
        let pause_change_state_val = if settings.ui_settings.pause_after_state_change {
            "yes"
        } else {
            "no"
        };
        let pause_change_state_value = SettingsBlock::create_settings_paragraph(
            pause_change_state_val,
            Some(settings.highlight_selected(6)),
        );

        let hide_clock_text =
            SettingsBlock::create_settings_paragraph("Hide clock on work time", None);
        let hide_clock_val = if settings.ui_settings.hide_work_countdown {
            "yes"
        } else {
            "no"
        };
        let hide_clock_value = SettingsBlock::create_settings_paragraph(
            hide_clock_val,
            Some(settings.highlight_selected(7)),
        );

        OtherTabSettings {
            pause_change_state_text,
            pause_change_state_value,
            hide_clock_text,
            hide_clock_value,
        }
    }
}
pub struct SettingsBlock {}
impl<'a> SettingsBlock {
    pub fn create_settings_block(title: &str, color: Color, border: BorderType) -> Block<'a> {
        Block::default()
            .title(format!(" {} ", title))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(border)
            .border_style(Style::default().fg(color))
    }
    pub fn create_settings_paragraph(title: &str, highlight_style: Option<Style>) -> Paragraph<'a> {
        let p = Paragraph::new(title.to_string())
            .alignment(Alignment::Center)
            .add_modifier(Modifier::BOLD);
        if let Some(hi) = highlight_style {
            p.style(hi)
        } else {
            p.style(Style::default().fg(Color::White))
        }
    }
}
