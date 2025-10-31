use crate::ui::{BG, BLUE, GREEN, RED, YELLOW};
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Gauge;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use crate::romodoro::Pomodoro;

impl Widget for &Pomodoro {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let time = self.timer.get_timeleft();
        let total_time = self.timer.get_total_time();
        let elapsed_time = self.timer.get_total_elapsed_time();
        let now_text = format!("Now: {}", self.timer.get_current_state());
        let progress = (elapsed_time) as f64 / total_time as f64;
        let iterations_text = format!(
            "{}/{} iterations",
            self.timer.get_iteration(),
            self.timer.get_total_iterations()
        );

        let outer_block = Block::default()
            .title(" Pomodoro Timer ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(YELLOW)); // Gruvbox yellow for border

        // Determine timer display based on settings and state
        let (timer_style, text_of_timer) = match self
            .get_setting_ref()
            .borrow()
            .ui_settings
            .hide_work_countdown
        {
            true if self.timer.get_running() => {
                (Style::default().fg(RED), format_ascii_time("00:00:00"))
            }
            true => (
                Style::default().fg(RED),
                format_ascii_time(&format!(
                    "{:02}:{:02}:{:02}",
                    time / 3600,
                    (time % 3600) / 60,
                    time % 60
                )),
            ),
            false => (
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
                format_ascii_time(&format!(
                    "{:02}:{:02}:{:02}",
                    time / 3600,
                    (time % 3600) / 60,
                    time % 60
                )),
            ),
        };

        let timer_text = Paragraph::new(text_of_timer)
            .alignment(Alignment::Center)
            .style(timer_style);

        // Style the state indicator based on current state
        let now_paragraph_style = match self.timer.get_current_state() {
            crate::timer::PomodoroState::Work(_) => Style::default().fg(BLUE),
            crate::timer::PomodoroState::Break(_) => Style::default().fg(GREEN),
        };

        let now_paragraph = Paragraph::new(now_text)
            .alignment(Alignment::Center)
            .style(now_paragraph_style);

        let count_paragraph = Paragraph::new(iterations_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(BLUE).add_modifier(Modifier::ITALIC));

        // Create gauge with proper title
        let gauge_style = match self.timer.get_current_state() {
            crate::timer::PomodoroState::Work(_) => BLUE,
            crate::timer::PomodoroState::Break(_) => GREEN,
        };

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(gauge_style)
                    .title(" Progress ")
                    .title_alignment(Alignment::Center),
            )
            .gauge_style(Style::default().fg(gauge_style))
            .ratio(progress);

        let available_height = area.height.saturating_sub(2);

        let top_margin_percent = if available_height > 20 { 12 } else { 5 };
        let bottom_margin_percent = if available_height > 20 { 12 } else { 5 };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(top_margin_percent), // Top margin
                Constraint::Length(1),                      // Now text
                Constraint::Length(5),                      // Small gap
                Constraint::Min(5),                         // ASCII timer
                Constraint::Length(1),                      // count
                Constraint::Length(1),                      // Subjects
                Constraint::Length(1),                      // Small gap
                Constraint::Length(3),                      // Progress bar
                Constraint::Percentage(bottom_margin_percent), // Bottom margin
            ])
            .split(area);

        // Create horizontal layout for centered gauge
        let gauge_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(layout[7]);

        // Render all elements
        outer_block.render(area, buf);
        now_paragraph.render(layout[1], buf);
        timer_text.render(layout[3], buf);
        count_paragraph.render(layout[4], buf);
        self.render_subject_select(layout[5], buf);
        gauge.render(gauge_layout[1], buf);

        // Set background color while preserving existing styles
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = buf.cell_mut((x, y)).expect("should work lmao");
                if cell.style().bg == Some(Color::Reset) {
                    cell.set_style(cell.style().bg(BG));
                }
            }
        }
    }
}
impl Pomodoro {
    fn render_subject_select(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let subject = self.get_current_subject();
        let subject_name: String = match &subject {
            Some(subject) => subject.graph_name().to_string(),
            None => "No subjects".into(),
        };
        let style = match &subject {
            Some(_) if self.timer.get_timer_started() => Style::default().fg(Color::Gray),
            Some(_) => Style::default().fg(YELLOW),
            None => Style::default().fg(Color::Gray),
        };
        let subject_paragraph = Paragraph::new(format!("Tracking: {subject_name}"))
            .alignment(Alignment::Center)
            .style(style);
        subject_paragraph.render(area, buf);
    }
}
fn format_ascii_time(input: &str) -> String {
    let mut output = vec![String::new(); 7];

    for ch in input.chars() {
        let index = match ch {
            '0'..='9' => ch as usize - '0' as usize,
            ':' => 10,
            _ => continue,
        };

        let ascii_lines: Vec<&str> = ASCII_NUMBERS[index].lines().collect();

        for (i, line) in ascii_lines.iter().enumerate() {
            output[i].push_str(line);
            output[i].push_str("  ");
        }
    }

    output.join("\n")
}

const ASCII_NUMBERS: [&str; 11] = [
    "  ███  \n █   █ \n█     █\n█     █\n█     █\n █   █ \n  ███  ", // 0
    "   █   \n  ██   \n █ █   \n   █   \n   █   \n   █   \n ████  ", // 1
    " ███   \n█   █  \n    █  \n   █   \n  █    \n █     \n█████  ", // 2
    " ███   \n█   █  \n    █  \n  ██   \n    █  \n█   █  \n ███   ", // 3
    "   ██  \n  █ █  \n █  █  \n█   █  \n█████  \n    █  \n    █  ", // 4
    "█████  \n█      \n████   \n    █  \n    █  \n█   █  \n ███   ", // 5
    "  ███  \n █     \n█      \n█ ███  \n█    █ \n █   █ \n  ███  ", // 6
    "█████  \n    █  \n   █   \n  █    \n █     \n █     \n █     ", // 7
    "  ███  \n █   █ \n █   █ \n  ███  \n █   █ \n █   █ \n  ███  ", // 8
    "  ███  \n █   █ \n █   █ \n  ████ \n     █ \n    ██ \n  ███  ", // 9
    "        \n   █    \n   █    \n        \n   █    \n   █    \n        ", // :
];
#[cfg(test)]
mod test {
    use super::format_ascii_time;

    #[test]
    fn ascii_text_works() {
        let time = "01:32:29";
        println!("{}", format_ascii_time(time));
    }
}
