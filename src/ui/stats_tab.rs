use crate::stats::pixel::Pixel;
use crate::stats::pixela::complex_pixel::IsRounded;
use crate::stats::pixela::pixela_client::{PixelaClient, PixelaTabs};
use crate::stats::pixela::subjects::{Subject, SubjectUnit};
use crate::ui::ui_utils::FooterHint;
use crate::ui::{BLUE, GREEN, ORANGE, RED, YELLOW};
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, List};
use ratatui::widgets::{BorderType, HighlightSpacing};
use ratatui::widgets::{Borders, ListItem};
use ratatui::widgets::{Paragraph, StatefulWidget};
use ratatui::widgets::{Widget, Wrap};

use super::helpers;
use super::ui_utils::HintProvider;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
pub struct StatsTab<'a> {
    pixela_client: &'a mut PixelaClient,
}
impl Widget for &mut StatsTab<'_> {
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
                Constraint::Max(6),  // Sync area
                Constraint::Fill(1), // Subjects area
            ])
            .split(top_layout[1]);

        self.render_pixels(top_layout[0], buf);

        self.render_sync(right_column_layout[0], buf);

        self.render_subjects(right_column_layout[1], buf);

        self.render_graph(main_layout[1], buf);
    }
}

impl<'a> StatsTab<'a> {
    pub fn new(pixela_client: &'a mut PixelaClient) -> Self {
        Self { pixela_client }
    }

    fn render_pixels(&mut self, area: Rect, buf: &mut Buffer) {
        let border_type = match self.pixela_client.focused_pane() == PixelaTabs::Pixels {
            true => BorderType::Double,
            false => BorderType::Rounded,
        };
        let block = Block::default()
            .title(" Pixels ")
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(BLUE));

        let inner_area = block.inner(area);

        if self.pixela_client.pixels.is_empty() {
            block.render(area, buf);
            let no_pixels_text = Paragraph::new("No pixels saved")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray))
                .bold();
            no_pixels_text.render(inner_area, buf);
        } else {
            if self.pixela_client.focused_pane() == PixelaTabs::Pixels
                && self.pixela_client.pixels.state().selected().is_none()
            {
                self.pixela_client.pixels.state_mut().select_first();
            } else if self.pixela_client.focused_pane() != PixelaTabs::Pixels {
                self.pixela_client.pixels.state_mut().select(None);
            }

            let list = List::new(
                self.pixela_client
                    .pixels
                    .iter()
                    .enumerate()
                    .map(|(index, pix)| {
                        PixelToListWrapper {
                            pixel: pix,
                            selected: self.pixela_client.selected_to_send(index),
                            rounded: IsRounded::No,
                            available_size: area.width,
                        }
                        .into()
                    })
                    .collect::<Vec<ListItem>>(),
            )
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);
            StatefulWidget::render(list, area, buf, self.pixela_client.pixels.state_mut());
        }
        let pixels = &self.pixela_client.subjects;
        helpers::render_scroll_indicators(
            inner_area,
            buf,
            pixels.items().len(),
            pixels.state(),
            GREEN,
        );
    }

    fn render_sync(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Sync ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(GREEN));

        let inner_area = block.inner(area);
        block.render(area, buf);

        let sync_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Max(2),  // Status line
                Constraint::Fill(1), // Message area
            ])
            .split(inner_area);

        let (status, message) = match self.pixela_client.logged_in() {
            false => (
                "Status: Logged out".into(),
                format!(
                    "Press 'L' to sync account ({})",
                    self.pixela_client.user.username()
                ),
            ),
            true => (
                format!(
                    "Status: Logged in as {}",
                    self.pixela_client.user.username()
                ),
                "You can use complex stats tracking!".into(),
            ),
        };
        let status_text = Paragraph::new(Text::from(status))
            .style(Style::default().fg(YELLOW))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        status_text.render(sync_layout[0], buf);

        // Sync message area
        let message_text = Paragraph::new(Text::from(message))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .bold()
            .wrap(Wrap { trim: true });
        message_text.render(sync_layout[1], buf);
    }

    fn render_subjects(&mut self, area: Rect, buf: &mut Buffer) {
        let border_type = if self.pixela_client.focused_pane() == PixelaTabs::Subject {
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

        if self.pixela_client.subjects().len() < 2 {
            block.render(area, buf);
            let no_subjects_text = Paragraph::new("Subjects appear after logging in...")
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            no_subjects_text.render(inner_area, buf);
        } else {
            if self.pixela_client.focused_pane() == PixelaTabs::Subject
                && self.pixela_client.subjects.state().selected().is_none()
            {
                self.pixela_client.subjects.state_mut().select_first();
            } else if self.pixela_client.focused_pane() != PixelaTabs::Subject {
                self.pixela_client.subjects.state_mut().select(None);
            }
            let list = List::new(
                self.pixela_client
                    .subjects()
                    .iter()
                    .map(|subject| {
                        SubjectToListWrapper {
                            subject,
                            selected: self
                                .pixela_client
                                .get_current_subject()
                                .unwrap_or_else(|| self.pixela_client.get_subject(0).unwrap())
                                == **subject,
                            available_size: area.width,
                        }
                        .into()
                    })
                    .collect::<Vec<ListItem>>(),
            )
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);
            let subjects = &mut self.pixela_client.subjects;
            StatefulWidget::render(list, area, buf, subjects.state_mut());
            helpers::render_scroll_indicators(
                area,
                buf,
                subjects.items().len(),
                subjects.state(),
                GREEN,
            );
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

        if let Some(g) = self.pixela_client.current_graph() {
            g.render(inner_area, buf);
        } else {
            let placeholder_text = Paragraph::new("Press G to render a graph.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            placeholder_text.render(inner_area, buf);
        }
    }
}
pub(crate) struct PixelToListWrapper<'a> {
    pub(crate) pixel: &'a Pixel,
    pub(crate) selected: bool,
    pub(crate) rounded: IsRounded,
    pub(crate) available_size: u16,
}
impl From<PixelToListWrapper<'_>> for ListItem<'_> {
    fn from(value: PixelToListWrapper) -> Self {
        let (rounded, rounded_to) = if let IsRounded::Yes(rounded_to) = value.rounded {
            (true, rounded_to)
        } else {
            (false, 0)
        };
        let (pixel_text, unit) = match value.pixel {
            Pixel::Complex(complex) => {
                let mut text = format!(
                    " {} | {} | {}",
                    complex.subject().graph_name(),
                    complex.date(),
                    complex.display_string(),
                );
                if text.len() + 5 >= value.available_size.into() {
                    text = format!(
                        " {} | {} | {}",
                        complex.subject().shortened_graph_name(),
                        complex.date(),
                        complex.display_string()
                    )
                };
                (text, complex.subject().unit().clone())
            }
            Pixel::Simple(simple) => (
                format!(" {} | {}m", simple.date(), simple.progress()),
                SubjectUnit::Minutes,
            ),
        };
        let pixel_text = Text::from(pixel_text);
        let unit_display = unit.short_string();
        let line = match (value.selected, rounded) {
            (false, _) if matches!(value.pixel, Pixel::Simple(_)) => Line::styled(
                format!("  {}", pixel_text),
                Style::default().fg(Color::Gray),
            ),
            (true, false) => Line::styled(format!(" ✓ {}", pixel_text), Style::default().fg(GREEN)),
            (true, true) => Line::styled(
                format!(
                    " ✓ {}| rounded: {}{}!",
                    pixel_text, rounded_to, unit_display
                ),
                Style::default().fg(ORANGE),
            ),
            (false, _) => Line::styled(
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
    pub(crate) available_size: u16,
}
impl From<SubjectToListWrapper<'_>> for ListItem<'_> {
    fn from(value: SubjectToListWrapper) -> Self {
        let mut subject_text = format!(
            "{} | unit: {}",
            value.subject.graph_name().to_string(),
            value.subject.unit().short_string()
        );
        if subject_text.len() + 5 > value.available_size.into() {
            subject_text = format!(
                "{} | unit: {}",
                value.subject.shortened_graph_name(),
                value.subject.unit().short_string()
            );
        }

        let line = match (value.selected, value.subject.is_dummy()) {
            (true, true) => Line::styled("None", Style::default().fg(GREEN)).into(),
            (true, false) => Line::styled(
                format!("{} [Tracking]", subject_text),
                Style::default().fg(GREEN),
            )
            .into(),
            (false, false) => {
                Line::styled(subject_text.to_string(), Style::default().fg(Color::White))
            }
            (false, true) => Line::styled("None", Style::default().fg(Color::White)),
        };
        ListItem::new(line)
    }
}

impl HintProvider for StatsTab<'_> {
    fn provide_hints(&self) -> Vec<FooterHint> {
        let mut default = vec![
            FooterHint::new("<>", "Change Tabs"),
            FooterHint::new("↑↓", "Select"),
        ];
        let mut based_on_state = match self.pixela_client.focused_pane() {
            PixelaTabs::Pixels => {
                let mut basic = vec![FooterHint::new("Space|RET", "Select Pixel(s)")];
                if !self.pixela_client.get_selected_pixels().is_empty() {
                    basic.append(&mut vec![
                        FooterHint::new("P", "Push to Pixela"),
                        FooterHint::new("d", "Delete pixel"),
                    ]);
                };
                basic
            }
            PixelaTabs::Subject => {
                if !self.pixela_client.subjects.is_empty() {
                    vec![
                        FooterHint::new("G", "Render graph"),
                        FooterHint::new("Space", "Track"),
                    ]
                } else {
                    vec![]
                }
            }
        };
        default.append(&mut based_on_state);
        default
    }
}
