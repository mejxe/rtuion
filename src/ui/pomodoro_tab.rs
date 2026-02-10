use crate::timers::counters::CounterMode;
use crate::timers::helper_structs::TimerState;
use crate::ui::assets::{ASCII_NUMBERS, BIG_ANIMATION_FRAMES};
use crate::ui::{BLUE, GREEN, RED, YELLOW};
use ratatui::layout::Alignment;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::{Constraint, Flex};
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::{Color, Stylize};
use ratatui::text::Text;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Gauge;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, Padding};

use crate::romodoro::Pomodoro;

use super::ui_utils::{FooterHint, HintProvider, UIHelper};

pub struct PomodoroTab<'a> {
    pomodoro: &'a Pomodoro,
}
impl Widget for PomodoroTab<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let time = self.pomodoro.timer.time_left();
        let now_text = format!("Now: {}", self.pomodoro.timer.current_state());
        let hide_countdown = self
            .pomodoro
            .get_setting_ref()
            .borrow()
            .ui_settings
            .hide_work_countdown;

        let main_title = match self.pomodoro.timer.counter_mode() {
            CounterMode::Countup => " Flowmodoro Timer ",
            CounterMode::Countdown => " Pomodoro Timer ",
        };
        let outer_block = Block::default()
            .title(main_title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(YELLOW));

        if area.width < 75 || area.height < 30 {
            match self.pomodoro.timer.counter_mode() {
                CounterMode::Countdown => self.render_compressed_pomodoro_ui(area, buf),
                CounterMode::Countup => self.render_compressed_flowmodoro_ui(area, buf),
            }
            return;
        }
        let color = match self.pomodoro.timer.current_state() {
            TimerState::Work(_) => BLUE,
            TimerState::Break(_) => GREEN,
        };
        let (timer_style, text_of_timer, animation_style, animation_text) = match hide_countdown {
            true => (
                Style::default().fg(color),
                ascii_animation(&BIG_ANIMATION_FRAMES, self.pomodoro.timer.time_left()),
                Style::default().fg(color),
                format!(
                    "{:02}:{:02}:{:02}",
                    time / 3600,
                    (time % 3600) / 60,
                    time % 60
                ),
            ),
            false => (
                Style::default().fg(color).add_modifier(Modifier::BOLD),
                format_ascii_time(&format!(
                    "\n\n{:02}:{:02}:{:02}",
                    time / 3600,
                    (time % 3600) / 60,
                    time % 60
                )),
                Style::default().fg(color),
                "".to_string(),
            ),
        };
        let formatted_timer_text = Text::from(text_of_timer);

        let timer_text = Paragraph::new(formatted_timer_text.patch_style(timer_style))
            .alignment(Alignment::Center);
        let formatted_animation_text = Text::from(animation_text);
        let animation_text = Paragraph::new(formatted_animation_text.patch_style(animation_style))
            .alignment(Alignment::Center);

        // Style the state indicator based on current state
        let now_paragraph_style = match self.pomodoro.timer.current_state() {
            TimerState::Work(_) => Style::default().fg(BLUE),
            TimerState::Break(_) => Style::default().fg(GREEN),
        };

        let now_paragraph =
            UIHelper::create_settings_paragraph(&now_text, Some(now_paragraph_style))
                .alignment(Alignment::Center);
        let spacer = Block::bordered()
            .borders(Borders::TOP)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(YELLOW));

        let layout_timer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Max(3),     // Small gap
                Constraint::Length(1),  // Now text
                Constraint::Max(5),     // Small gap
                Constraint::Length(10), // ASCII timer
                Constraint::Length(3),  // ASCII Spacer
                Constraint::Length(7),  // ASCII Animation
                Constraint::Length(1),  // ASCII Spacer
                Constraint::Max(3),     // Pomodoro/Flowmodoro layouts
            ])
            .flex(ratatui::layout::Flex::Center)
            .split(outer_block.inner(area));
        let layout_animation = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(0),  // Small gap
                Constraint::Length(1),  // Now text
                Constraint::Max(3),     // Small gap
                Constraint::Length(10), // ASCII timer
                Constraint::Length(2),  // ASCII Spacer
                Constraint::Length(1),  // ASCII Animation
                Constraint::Length(3),  // ASCII Spacer
                Constraint::Max(7),     // Pomodoro/Flowmodoro layouts
            ])
            .flex(ratatui::layout::Flex::Center)
            .split(outer_block.inner(area));
        let layout = match self
            .pomodoro
            .get_setting_ref()
            .borrow()
            .ui_settings
            .hide_work_countdown
        {
            false => layout_timer,
            true => layout_animation,
        };

        let spacer_layout = Layout::horizontal([Constraint::Percentage(50)])
            .flex(Flex::Center)
            .split(layout[4]);
        // Render all elements
        outer_block.render(area, buf);
        now_paragraph.render(layout[1], buf);
        timer_text.render(layout[3], buf);
        spacer.render(spacer_layout[0], buf);
        let layout_spot = if hide_countdown {
            animation_text.render(layout[5], buf);
            7
        } else {
            5
        };
        self.render_subject_select(layout[layout_spot - 1], buf);
        match self.pomodoro.timer.counter_mode() {
            CounterMode::Countdown => self.render_pomodoro_ui(layout[layout_spot], buf),
            CounterMode::Countup => self.render_flowmodoro_ui(layout[layout_spot], buf),
        }
    }
}

impl<'a> PomodoroTab<'a> {
    pub fn new(pomodoro: &'a Pomodoro) -> Self {
        Self { pomodoro }
    }

    fn render_subject_select(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let subject = self.pomodoro.get_current_subject();
        let subject_name: String = match &subject {
            Some(subject) => subject.graph_name().to_string(),
            None => "No subjects".into(),
        };
        let style = match &subject {
            Some(sub) if self.pomodoro.timer.timer_started() | sub.is_dummy() => {
                Style::default().fg(Color::Gray)
            }
            Some(_) => Style::default().fg(YELLOW),
            None => Style::default().fg(Color::Gray),
        };
        let subject_paragraph =
            UIHelper::create_settings_paragraph(&format!("Tracking: {subject_name}"), Some(style));
        subject_paragraph.render(area, buf);
    }
    fn render_flowmodoro_ui(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let mut center_percentage = 40;
        let mut smol = false;
        let mut text_align = Alignment::Left;
        // smoll mode
        if area.width < 150 {
            smol = true;
            center_percentage = 70;
            text_align = Alignment::Center
        }
        let outer_block = Block::bordered()
            .title(" Flowmodoro Summary ")
            .title_alignment(Alignment::Left)
            .border_type(BorderType::Rounded)
            .padding(Padding::symmetric(5, 0))
            .border_style(Style::default().fg(GREEN));
        let outer_layout = Layout::horizontal([Constraint::Percentage(center_percentage)])
            .flex(Flex::Center)
            .split(area);
        let inside_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Center)
        .split(outer_block.inner(outer_layout[0]));
        outer_block.render(outer_layout[0], buf);
        let elapsed_time = self.pomodoro.timer.total_elapsed();
        let break_time = if let TimerState::Break(_) = self.pomodoro.timer.current_state() {
            0
        } else {
            self.pomodoro.timer.break_time()
        };
        let next_break_time = UIHelper::create_settings_paragraph(
            &format!("Next break time length: {break_time} seconds "),
            None,
        )
        .alignment(text_align);
        let worked_for = match smol {
            false => format!(
                "{:02} hrs, {:02} mins, {:02} secs",
                elapsed_time / 3600,
                (elapsed_time % 3600) / 60,
                elapsed_time % 60
            ),
            true => format!(
                "{:02}:{:02}:{:02}",
                elapsed_time / 3600,
                (elapsed_time % 3600) / 60,
                elapsed_time % 60
            ),
        };

        let already_worked_for =
            UIHelper::create_settings_paragraph(&format!("Already worked for: {worked_for}"), None)
                .alignment(text_align);
        let breaks_taken_num = self.pomodoro.timer.iteration().saturating_sub(1);
        let breaks_taken =
            UIHelper::create_settings_paragraph(&format!("Breaks taken: {breaks_taken_num}"), None)
                .alignment(text_align);
        next_break_time.render(inside_layout[0], buf);
        already_worked_for.render(inside_layout[1], buf);
        breaks_taken.render(inside_layout[2], buf);
    }
    fn render_pomodoro_ui(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let total_time = self.pomodoro.timer.total_time();
        let elapsed_time = self.pomodoro.timer.total_elapsed();
        let progress = (elapsed_time) as f64 / total_time as f64;
        let iterations_text = format!(
            "{}/{} iterations",
            self.pomodoro.timer.iteration(),
            self.pomodoro.timer.total_iterations()
        );
        let pomodoro_layout = Layout::vertical(vec![
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .split(area);
        let count_paragraph = UIHelper::create_settings_paragraph(
            &iterations_text,
            Some(Style::default().fg(BLUE).bold()),
        );

        let gauge_style = match self.pomodoro.timer.current_state() {
            TimerState::Work(_) => BLUE,
            TimerState::Break(_) => GREEN,
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
        let gauge_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60)])
            .flex(ratatui::layout::Flex::Center)
            .split(pomodoro_layout[2]);
        count_paragraph.render(pomodoro_layout[1], buf);
        gauge.render(gauge_layout[0], buf);
    }
    fn render_compressed_flowmodoro_ui(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let layout = Layout::vertical(vec![
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Center)
        .split(area);
        let time = self.pomodoro.timer.time_left();
        let timer_paragraph = Paragraph::new(format!(
            "{:02}:{:02}:{:02}",
            time / 3600,
            (time % 3600) / 60,
            time % 60
        ))
        .centered();
        let (current_state, color) = match self.pomodoro.timer.current_state() {
            TimerState::Work(_) => ("Work", BLUE),
            TimerState::Break(_) => ("Break", GREEN),
        };
        let state_paragraph = Paragraph::new(current_state).centered().fg(color);

        let break_time = if let TimerState::Break(_) = self.pomodoro.timer.current_state() {
            0
        } else {
            self.pomodoro.timer.break_time()
        };
        let next_break_time = Paragraph::new(format!("NxtBreak: {break_time}"))
            .centered()
            .fg(YELLOW);
        let time_elapsed = self.pomodoro.timer.total_elapsed();
        let worked_for = format!(
            "{:02}:{:02}:{:02}",
            time_elapsed / 3600,
            (time_elapsed % 3600) / 60,
            time_elapsed % 60
        );
        let already_worked_for = Paragraph::new(format!("TPassed: {worked_for}"))
            .centered()
            .fg(RED);
        let breaks_taken_num = self.pomodoro.timer.iteration().saturating_sub(1);
        let breaks_taken = Paragraph::new(format!("Breaks: {breaks_taken_num}"))
            .centered()
            .fg(BLUE);
        state_paragraph.render(layout[0], buf);
        timer_paragraph.render(layout[1], buf);
        next_break_time.render(layout[2], buf);
        already_worked_for.render(layout[3], buf);
        breaks_taken.render(layout[4], buf);
    }

    fn render_compressed_pomodoro_ui(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let layout = Layout::vertical(vec![
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Center)
        .split(area);
        let time = self.pomodoro.timer.time_left();
        let timer_paragraph = Paragraph::new(format!(
            "{:02}:{:02}:{:02}",
            time / 3600,
            (time % 3600) / 60,
            time % 60
        ))
        .centered();
        let (current_state, color) = match self.pomodoro.timer.current_state() {
            TimerState::Work(_) => ("Work", BLUE),
            TimerState::Break(_) => ("Break", GREEN),
        };
        let iteration_paragraph = Paragraph::new(format!(
            "C_Iteration: {}, Iterations: {}",
            self.pomodoro.timer.iteration(),
            self.pomodoro.timer.total_iterations()
        ))
        .centered()
        .fg(YELLOW);
        let state_paragraph = Paragraph::new(current_state).centered().fg(color);
        timer_paragraph.render(layout[0], buf);
        state_paragraph.render(layout[1], buf);
        iteration_paragraph.render(layout[2], buf);
    }
}
fn ascii_animation(frames: &[&str], time: i64) -> String {
    frames[time as usize % (frames.len() - 1)].to_string()
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
            output[i].push_str(&format!("{:<7}", line));
            output[i].push_str("  ");
        }
    }

    output.join("\n")
}
impl HintProvider for PomodoroTab<'_> {
    fn provide_hints(&self) -> Vec<FooterHint> {
        let mut def = vec![FooterHint::new("r", "Reset timer")];

        let mut state_dependent = match self.pomodoro.timer.counter_mode() {
            CounterMode::Countup if self.pomodoro.timer.in_work_state() => vec![
                FooterHint::new("Space", "Cycle"),
                FooterHint::new("x", "Start Break"),
            ],
            _ => vec![FooterHint::new("Space", "Cycle")],
        };
        state_dependent.append(&mut def);
        state_dependent
    }
}

#[cfg(test)]
mod test {
    use super::format_ascii_time;

    #[test]
    fn ascii_text_works() {
        let time = "01:32:29";
        println!("{}", format_ascii_time(time));
    }
}
