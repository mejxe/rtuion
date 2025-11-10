use std::ops::Sub;
use std::process::exit;

use crate::pixela_client::PixelaClient;
use crate::stats::{Pixel, Subject};
use crate::ui::{BG, BLUE, GREEN, RED, YELLOW};
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, List};
use ratatui::widgets::{BorderType, HighlightSpacing};
use ratatui::widgets::{Borders, ListItem};
use ratatui::widgets::{Paragraph, StatefulWidget};
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
impl Widget for &mut PixelaClient {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // pixels and right column
                Constraint::Percentage(40), // Bottom graph
            ])
            .split(area);

        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Pixels area
                Constraint::Percentage(40), // Sync + Subjects area
            ])
            .split(main_layout[0]);

        let right_column_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20), // Sync area
                Constraint::Percentage(80), // Subjects area
            ])
            .split(top_layout[1]);

        self.render_pixels(top_layout[0], buf);

        self.render_sync(right_column_layout[0], buf);

        self.render_subjects(right_column_layout[1], buf);

        self.render_graph(main_layout[1], buf);

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

impl PixelaClient {
    fn render_pixels(&mut self, area: Rect, buf: &mut Buffer) {
        let border_type = if self.focused_pane() == 0 {
            BorderType::Double
        } else {
            BorderType::Rounded
        };
        let block = Block::default()
            .title(" Pixels ")
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(BLUE));

        let inner_area = block.inner(area);

        if self.pixels.is_empty() {
            block.render(area, buf);
            let no_pixels_text = Paragraph::new("No pixels saved")
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                );
            no_pixels_text.render(inner_area, buf);
        } else {
            if self.focused_pane() == 0 && self.pixels.state().selected().is_none() {
                self.pixels.state_mut().select_first();
            } else if self.focused_pane() != 0 {
                self.pixels.state_mut().select(None);
            }

            let list = List::new(
                self.pixels
                    .iter()
                    .enumerate()
                    .map(|(index, pix)| {
                        PixelToListWrapper {
                            pixel: pix,
                            selected: self.selected_to_send(index),
                        }
                        .into()
                    })
                    .collect::<Vec<ListItem>>(),
            )
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);
            StatefulWidget::render(list, area, buf, self.pixels.state_mut());
        }
    }

    fn render_sync(&self, area: Rect, buf: &mut Buffer) {
        let border_type = if self.focused_pane() == 1 {
            BorderType::Double
        } else {
            BorderType::Rounded
        };
        let block = Block::default()
            .title(" Sync ")
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(GREEN));

        let inner_area = block.inner(area);
        block.render(area, buf);

        let sync_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Status line
                Constraint::Min(1),    // Message area
            ])
            .split(inner_area);

        let (status, message) = match self.logged_in() {
            false => (
                "Status: Not Logged In to Pixela".into(),
                format!("Press 'L' to sync account ({})", self.user.username()),
            ),
            true => (
                format!("Status: Logged in as {}", self.user.username()),
                "You can use complex stats tracking for this session!".into(),
            ),
        };
        let status_text = Paragraph::new(status)
            .style(Style::default().fg(YELLOW))
            .alignment(Alignment::Center);
        status_text.render(sync_layout[0], buf);

        // Sync message area
        let message_text = Paragraph::new(message)
            .style(
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )
            .alignment(Alignment::Center);
        message_text.render(sync_layout[1], buf);
    }

    fn render_subjects(&mut self, area: Rect, buf: &mut Buffer) {
        let border_type = if self.focused_pane() == 2 {
            BorderType::Double
        } else {
            BorderType::Rounded
        };
        let block = Block::default()
            .title(" Subjects ")
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(YELLOW));

        let inner_area = block.inner(area);

        if self.subjects().is_empty() {
            block.render(area, buf);
            let no_subjects_text = Paragraph::new("Subjects appear after logging in...")
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                );
            no_subjects_text.render(inner_area, buf);
        } else {
            if self.focused_pane() == 2 && self.subjects.state().selected().is_none() {
                self.subjects.state_mut().select_first();
            } else if self.focused_pane() != 2 {
                self.subjects.state_mut().select(None);
            }
            let list = List::new(
                self.subjects()
                    .iter()
                    .map(|subject| {
                        SubjectToListWrapper {
                            subject,
                            selected: self.get_current_subject().unwrap() == **subject,
                        }
                        .into()
                    })
                    .collect::<Vec<ListItem>>(),
            )
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);
            StatefulWidget::render(list, area, buf, self.subjects.state_mut());
        }
    }

    fn render_graph(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Graph ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(RED));

        let inner_area = block.inner(area);
        block.render(area, buf);

        if let Some(g) = self.current_graph() {
            g.render(inner_area, buf);
        } else {
            let placeholder_text = Paragraph::new("Press G to render a graph.")
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                );
            placeholder_text.render(inner_area, buf);
        }
    }
}
pub(crate) struct PixelToListWrapper<'a> {
    pub(crate) pixel: &'a Pixel,
    pub(crate) selected: bool,
}
impl From<PixelToListWrapper<'_>> for ListItem<'_> {
    fn from(value: PixelToListWrapper) -> Self {
        let pixel_text = match value.pixel {
            Pixel::Complex(complex) => {
                format!(
                    " {} | {} | {}m",
                    complex.subject().graph_name(),
                    complex.date().split('T').next().unwrap_or("N/A"),
                    complex.progress().minutes(complex.subject().unit())
                )
            }
            Pixel::Simple(simple) => {
                format!(
                    " {} | {}m",
                    simple.date().split('T').next().unwrap_or("N/A"),
                    simple.progress()
                )
            }
        };
        let line = match value.selected {
            false if matches!(value.pixel, Pixel::Simple(_)) => Line::styled(
                format!("  {}", pixel_text),
                Style::default().fg(Color::Gray),
            ),
            true => Line::styled(format!(" ✓ {}", pixel_text), Style::default().fg(GREEN)),
            false => Line::styled(
                format!(" ☐ {}", pixel_text),
                Style::default().fg(Color::White),
            ),
        };
        ListItem::new(line)
    }
}
pub(crate) struct SubjectToListWrapper<'a> {
    pub(crate) subject: &'a Subject,
    pub(crate) selected: bool,
}
impl From<SubjectToListWrapper<'_>> for ListItem<'_> {
    fn from(value: SubjectToListWrapper) -> Self {
        let subject_text = value.subject.graph_name().to_string();

        let line = match value.selected {
            true => Line::styled(
                format!("{} [Tracking]", subject_text),
                Style::default().fg(GREEN),
            ),
            false => Line::styled(subject_text.to_string(), Style::default().fg(Color::White)),
        };
        ListItem::new(line)
    }
}
