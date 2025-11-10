
use ratatui::{
    style::{Style, Stylize},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Widget},
};

use crate::graph::Graph;

impl Widget for &Graph {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        const BAR_WIDTH: u16 = 3;
        const BAR_GAP: u16 = 2;
        const PADDING: u16 = 4;
        const MIN_BARS_TO_DISPLAY: u16 = 7;
        const UPPER_PADDING: u64 = 2;

        let available_width = area.width.saturating_sub(PADDING);
        let max_bars = (available_width / (BAR_WIDTH + BAR_GAP)).max(MIN_BARS_TO_DISPLAY) as usize;

        let all_bars = self.into_bars();
        let bars_that_fit: Vec<_> = all_bars.into_iter().rev().take(max_bars).rev().collect();

        // invisible bars for padding
        let mut datapoints: Vec<Bar> = vec![
            Bar::default().value(0).label("".into()).style(
                Style::new().fg(ratatui::style::Color::Reset)
            );
            2
        ];
        datapoints.extend(bars_that_fit);
        let bar_chart = BarChart::default()
            .block(
                Block::default().title_top(
                    Line::from(self.subject().graph_name().to_string())
                        .bold()
                        .centered()
                        .fg(self.subject().color().to_ratatui_color()),
                ),
            )
            .data(BarGroup::default().bars(&datapoints))
            .bar_width(BAR_WIDTH)
            .bar_gap(BAR_GAP)
            .max(self.get_max_quantity() + UPPER_PADDING);
        bar_chart.render(area, buf);
    }
}
