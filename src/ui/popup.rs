use std::process::exit;

use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Widget, Wrap,
    },
};

use crate::{
    popup::{Popup, PopupKind},
    stats::Pixel,
};

use super::{stats_tab::PixelToListWrapper, BG, GREEN, RED, YELLOW};

impl Widget for &mut Popup {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let popup_area = popup_area(area, 50, 35);

        Clear.render(popup_area, buf);

        match &mut self.kind {
            PopupKind::YesNoPopup(_) => {
                Popup::render_yes_no_popup(&self.message, popup_area, buf);
            }
            PopupKind::ErrorPopup(error) => {
                Popup::render_error_popup(&self.message, popup_area, buf, error);
            }
            PopupKind::SendPixelsPopup(_, pixels, ref mut list_state) => {
                Popup::render_confirm_list_popup(
                    &self.message,
                    popup_area,
                    buf,
                    pixels,
                    list_state,
                );
            }
            PopupKind::ListPopup(pixels, ref mut list_state) => {
                Popup::render_list_popup(&self.message, popup_area, buf, pixels, list_state);
            }
        }
    }
}
impl Popup {
    fn render_yes_no_popup(
        message: &str,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) where
        Self: Sized,
    {
        let popup_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(BG));
        popup_block.render(area, buf);

        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Message area
                Constraint::Percentage(40), // Buttons area
            ])
            .split(area);

        let question_paragraph = Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(YELLOW))
                    .title(" Confirmation ")
                    .title_style(Style::default().fg(YELLOW)),
            );
        question_paragraph.render(layout[0], buf);

        let button_layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[1]);

        let yes_paragraph = Paragraph::new("<y>es")
            .alignment(Alignment::Center)
            .style(Style::default().fg(GREEN))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(GREEN)),
            );
        yes_paragraph.render(button_layout[0], buf);

        let no_paragraph = Paragraph::new("<n>o")
            .alignment(Alignment::Center)
            .style(Style::default().fg(RED))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(RED)),
            );
        no_paragraph.render(button_layout[1], buf);
    }
    fn render_error_popup(
        message: &str,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer,
        error: &crate::error::Error,
    ) {
        let popup_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(BG));
        popup_block.render(area, buf);

        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        let error_paragraph = Paragraph::new(format!("{}", error))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(RED))
                    .title(" Error ")
                    .title_style(Style::default().fg(RED)),
            );
        error_paragraph.render(layout[0], buf);

        let ok_paragraph = Paragraph::new("Press any key to continue")
            .alignment(Alignment::Center)
            .style(Style::default().fg(YELLOW))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(YELLOW)),
            );
        ok_paragraph.render(layout[1], buf);
    }
    fn render_confirm_list_popup(
        message: &str,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer,
        pixels: &[Pixel],
        list_state: &mut ListState,
    ) {
        let popup_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(BG));
        popup_block.render(area, buf);

        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Percentage(20), // Message area
                Constraint::Percentage(60), // List area
                Constraint::Percentage(30), // Buttons area
            ])
            .split(area);

        // Question/message paragraph
        let question_paragraph = Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(YELLOW))
                    .title(" Confirmation ")
                    .title_style(Style::default().fg(YELLOW)),
            );
        question_paragraph.render(layout[0], buf);

        // List area
        let pixel_items: Vec<ListItem> = pixels
            .iter()
            .map(|pix| {
                PixelToListWrapper {
                    pixel: pix,
                    selected: true,
                }
                .into()
            })
            .collect();

        let list = List::new(pixel_items)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Items ")
                    .title_style(Style::default().fg(Color::Cyan)),
            );

        use ratatui::widgets::StatefulWidget;
        StatefulWidget::render(list, layout[1], buf, list_state);
        render_scroll_indicators(layout[1], buf, pixels, list_state, GREEN);

        // Buttons
        let button_layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[2]);

        let yes_paragraph = Paragraph::new("<y>es")
            .alignment(Alignment::Center)
            .style(Style::default().fg(GREEN))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(GREEN)),
            );
        yes_paragraph.render(button_layout[0], buf);

        let no_paragraph = Paragraph::new("<n>o")
            .alignment(Alignment::Center)
            .style(Style::default().fg(RED))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(RED)),
            );
        no_paragraph.render(button_layout[1], buf);
    }
    fn render_list_popup(
        message: &str,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer,
        pixels: &[Pixel],
        list_state: &mut ListState,
    ) {
        let popup_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(BG));
        popup_block.render(area, buf);

        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Percentage(20), // Message area
                Constraint::Percentage(60), // List area
                Constraint::Percentage(30), // Buttons area
            ])
            .split(area);

        let question_paragraph = Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(YELLOW))
                    .title(" Confirmation ")
                    .title_style(Style::default().fg(YELLOW)),
            );
        question_paragraph.render(layout[0], buf);

        // List area
        let pixel_items: Vec<ListItem> = pixels
            .iter()
            .map(|pix| {
                PixelToListWrapper {
                    pixel: pix,
                    selected: true,
                }
                .into()
            })
            .collect();

        let list = List::new(pixel_items)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(YELLOW))
                    .title(" Items ")
                    .title_style(Style::default().fg(YELLOW)),
            );

        use ratatui::widgets::StatefulWidget;
        StatefulWidget::render(list, layout[1], buf, list_state);
        render_scroll_indicators(layout[1], buf, pixels, list_state, GREEN);

        let ok_paragraph = Paragraph::new("Press any key to continue")
            .alignment(Alignment::Center)
            .style(Style::default().fg(YELLOW))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(YELLOW)),
            );
        ok_paragraph.render(layout[2], buf);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect
pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
pub fn list_height(area: &Rect) -> usize {
    ((area.height * 60) / 100).saturating_sub(3) as usize
}
fn render_scroll_indicators(
    list_area: Rect,
    buf: &mut ratatui::prelude::Buffer,
    pixels: &[Pixel],
    list_state: &ListState,
    color: Color,
) {
    let list_inner = list_area.inner(Margin {
        horizontal: 0,
        vertical: 1,
    });
    let offset = list_state.offset();
    let list_height = list_inner.height as usize;
    let total_items = pixels.len();
    let selected = list_state.selected().unwrap_or(0);

    if offset > 0 {
        let up_arrow = Paragraph::new("▲")
            .style(Style::default().fg(color))
            .alignment(Alignment::Center);
        let arrow_area = Rect {
            x: list_area.x,
            y: list_area.y,
            width: list_area.width,
            height: 1,
        };
        up_arrow.render(arrow_area, buf);
    }

    if total_items > list_height && offset < total_items.saturating_sub(list_height) {
        let down_arrow = Paragraph::new("▼")
            .style(Style::default().fg(color))
            .alignment(Alignment::Center);
        let arrow_area = Rect {
            x: list_area.x,
            y: list_area.y + list_area.height - 1,
            width: list_area.width,
            height: 1,
        };
        down_arrow.render(arrow_area, buf);
    }
}
