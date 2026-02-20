use ratatui::{
    layout::{Alignment, Margin, Rect},
    prelude::Buffer,
    style::{Color, Style},
    widgets::{ListState, Paragraph, Widget},
};

use crate::stats::pixel::Pixel;

pub fn render_scroll_indicators(
    list_area: Rect,
    buf: &mut Buffer,
    total_items: usize,
    list_state: &ListState,
    color: Color,
) {
    let list_inner = list_area.inner(Margin {
        horizontal: 0,
        vertical: 1,
    });
    let offset = list_state.offset();
    let list_height = list_inner.height as usize;
    let _selected = list_state.selected().unwrap_or(0);

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
