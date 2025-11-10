use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap},
};

use crate::{
    app::App,
    popup::{Popup, PopupKind},
    stats::Pixel,
};

use super::{stats_tab::PixelToListWrapper, BG, GREEN, RED, YELLOW};

impl Widget for &Popup {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let popup_area = popup_area(area, 50, 35);

        Clear.render(popup_area, buf);

        match &self.kind {
            PopupKind::YesNoPopup(_) => {
                self.render_yes_no_popup(popup_area, buf);
            }
            PopupKind::ErrorPopup(error) => {
                self.render_error_popup(popup_area, buf, error);
            }
            PopupKind::SendPixelsPopup(_, pixels) => {
                self.render_confirm_list_popup(popup_area, buf, pixels);
            }
            PopupKind::ListPopup(pixels) => {
                self.render_list_popup(popup_area, buf, pixels);
            }
        }
    }
}
impl Popup {
    fn render_yes_no_popup(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
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

        let question_paragraph = Paragraph::new(self.message.clone())
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
        &self,
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

        let error_paragraph = Paragraph::new(format!("{}\n{}", self.message.clone(), error))
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
        &self,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer,
        pixels: &[Pixel],
    ) {
        let popup_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(BG));
        popup_block.render(area, buf);

        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Percentage(30), // Message area
                Constraint::Percentage(40), // List area
                Constraint::Percentage(30), // Buttons area
            ])
            .split(area);

        // Question/message paragraph
        let question_paragraph = Paragraph::new(self.message.clone())
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
        let pixels: Vec<ListItem> = pixels
            .iter()
            .map(|pix| {
                PixelToListWrapper {
                    // DONT SHOW SIMPLE PIXELS
                    pixel: pix,
                    selected: true,
                }
                .into()
            })
            .collect();

        let list_paragraph = List::new(pixels)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Items ")
                    .title_style(Style::default().fg(Color::Cyan)),
            );
        list_paragraph.render(layout[1], buf);

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
    fn render_list_popup(&self, area: Rect, buf: &mut ratatui::prelude::Buffer, pixels: &[Pixel]) {
        let popup_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(BG));
        popup_block.render(area, buf);

        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Percentage(30), // Message area
                Constraint::Percentage(40), // List area
                Constraint::Percentage(30), // Buttons area
            ])
            .split(area);

        // Question/message paragraph
        let question_paragraph = Paragraph::new(self.message.clone())
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
        let pixels: Vec<ListItem> = pixels
            .iter()
            .map(|pix| {
                PixelToListWrapper {
                    pixel: pix,
                    selected: true,
                }
                .into()
            })
            .collect();

        let list_paragraph = List::new(pixels)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(YELLOW))
                    .title(" Items ")
                    .title_style(Style::default().fg(YELLOW)),
            );
        list_paragraph.render(layout[1], buf);
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

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
