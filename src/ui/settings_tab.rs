use std::mem;
use std::os::linux::raw::stat;
use std::process::exit;

use crate::settings::{Mode, Settings, StatsSettings, TimerSettings, UISettings};
use crate::stats::pixela::pixela_client::PixelaClient;
use crate::timers::counters::CounterMode;
use crate::timers::timer::Timer;
use crate::ui::{BG, BLUE, RED, YELLOW};
use crate::utils::settings_helper_structs::SettingsTabs;
use ratatui::buffer::Buffer;
use ratatui::layout::Layout;
use ratatui::layout::{Alignment, Rect};
use ratatui::layout::{Constraint, Margin};
use ratatui::layout::{Direction, Flex};
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::{Color, Stylize};
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use super::ui_utils::{UIHelper, UISettingsTabData};
use super::GREEN;
pub struct SettingsTab<'a> {
    settings: &'a Settings,
    timer: &'a Timer,
    stats: Option<&'a PixelaClient>,
}

impl<'a> SettingsTab<'a> {
    pub fn new(settings: &'a Settings, timer: &'a Timer, stats: Option<&'a PixelaClient>) -> Self {
        Self {
            settings,
            timer,
            stats,
        }
    }
}
impl Widget for &SettingsTab<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let outer_layout =
            Layout::vertical(vec![Constraint::Length(2), Constraint::Percentage(95)]).split(area);
        let mut layout = SettingsGrid::new(4, 2);
        let tab_data = UISettingsTabData {
            selected_setting: self.settings.selected_setting,
            selected_tab: self.settings.selected_tab,
            current_mode: self.settings.mode(),
        };

        let mode_span = GridSpan {
            row: 0,
            col: 0,
            row_span: 1,
            col_span: 1,
        };
        let inner_area = layout.inner(outer_layout[1]);
        let mode_inner = layout.compute_square_pos(inner_area, mode_span).unwrap();
        let mode_block = mode_tab(mode_inner, buf, self.settings.counter_mode(), tab_data);
        let pomodoro_span = GridSpan {
            row: 1,
            col: 0,
            row_span: 2,
            col_span: 1,
        };
        let pomodoro_inner = layout
            .compute_square_pos(inner_area, pomodoro_span)
            .unwrap();
        let pomodoro_block =
            pomodoro_tab(pomodoro_inner, buf, &self.settings.timer_settings, tab_data);
        let preferences_span = GridSpan {
            row: 3,
            col: 0,
            row_span: 1,
            col_span: 2,
        };
        let preferences_inner = layout
            .compute_square_pos(inner_area, preferences_span)
            .unwrap();
        let preferences_block =
            preferences_tab(preferences_inner, buf, &self.settings.ui_settings, tab_data);
        let stats_span = GridSpan {
            row: 0,
            col: 1,
            row_span: 3,
            col_span: 1,
        };
        let stats_inner = layout.compute_square_pos(inner_area, stats_span).unwrap();
        let stats_block = stats_tab(stats_inner, buf, &self.settings.stats_setting, tab_data);
        let settings_not_saved = Paragraph::new("Settings are not applied!\n(Space to apply)")
            .style(Style::default().bold().fg(RED))
            .centered();
        layout.add(mode_block, mode_span);
        layout.add_spanning(preferences_block, preferences_span);
        layout.add_spanning(pomodoro_block, pomodoro_span);
        layout.add_spanning(stats_block, stats_span);
        layout.render(inner_area, buf);
        if !self.settings.do_settings_match(self.timer, self.stats) {
            settings_not_saved.render(outer_layout[0], buf);
        }
    }
}
#[derive(Clone, Copy)]
struct GridSpan {
    row: u16,
    col: u16,
    row_span: u16,
    col_span: u16,
}
struct SettingsGrid {
    rows: u16,
    cols: u16,
    items: Vec<Option<GridItem>>,
}
type Renderer = Box<dyn FnOnce(Rect, &mut Buffer)>;
struct GridItem {
    renderer: Renderer,
    row_span: u16,
    col_span: u16,
}
impl SettingsGrid {
    fn new(rows: u16, cols: u16) -> SettingsGrid {
        SettingsGrid {
            rows,
            cols,
            items: (0..(cols * rows)).map(|_| None).collect(),
        }
    }
    fn inner(&self, area: Rect) -> Rect {
        let vert = Layout::vertical([Constraint::Percentage(80)])
            .flex(Flex::Center)
            .split(area);
        Layout::horizontal(vec![Constraint::Percentage(50)])
            .flex(Flex::Center)
            .split(vert[0])[0]
    }
    fn add<W: Widget + 'static>(&mut self, item: W, grid_span: GridSpan) {
        let (row, col) = (grid_span.row, grid_span.col);
        let grid_item = GridItem {
            renderer: Box::new(move |area, buf| {
                item.render(area, buf);
            }),
            row_span: 1,
            col_span: 1,
        };
        self.items[(row * self.cols + col) as usize] = Some(grid_item);
    }
    fn add_spanning<W: Widget + 'static>(&mut self, item: W, grid_span: GridSpan) {
        let (row, col, row_span, col_span) = (
            grid_span.row,
            grid_span.col,
            grid_span.row_span,
            grid_span.col_span,
        );
        let grid_item = GridItem {
            renderer: Box::new(move |area, buf| {
                item.render(area, buf);
            }),
            row_span,
            col_span,
        };
        self.items[(row * self.cols + col) as usize] = Some(grid_item);
    }
    fn create_grid(&self, area: Rect) -> Vec<Rect> {
        let col_constraints = (0..self.cols).map(|_| Constraint::Ratio(1, self.cols as u32));
        let row_constraints = (0..self.rows).map(|_| Constraint::Ratio(1, self.rows as u32));
        let horizontal = Layout::horizontal(col_constraints);
        let vertical = Layout::vertical(row_constraints);
        Block::default()
            .inner(area)
            .layout_vec(&vertical)
            .into_iter()
            .flat_map(|row| row.layout_vec(&horizontal))
            .collect()
    }
    fn compute_square_pos(&self, area: Rect, grid_span: GridSpan) -> Option<Rect> {
        let cells = self.create_grid(area);
        self.compute_square_pos_provided_cells(&cells, grid_span)
    }
    fn compute_square_pos_provided_cells(
        &self,
        cells: &[Rect],
        grid_span: GridSpan,
    ) -> Option<Rect> {
        let (row, col, row_span, col_span) = (
            grid_span.row,
            grid_span.col,
            grid_span.row_span,
            grid_span.col_span,
        );
        if row > self.rows || col > self.cols {
            return None;
        }
        let start_index = (row * self.cols + col) as usize;

        let valid_row_span = row_span.min(self.rows - row);
        let valid_col_span = col_span.min(self.cols - col);

        let end_row = (row + valid_row_span).saturating_sub(1);
        let end_col = (col + valid_col_span).saturating_sub(1);
        let end_index = (end_row * self.cols + end_col) as usize;
        if let (Some(&start), Some(&end)) = (cells.get(start_index), cells.get(end_index)) {
            Some(start.union(end))
        } else {
            None
        }
    }
}
impl Widget for SettingsGrid {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let cells = self.create_grid(area);
        let items = mem::take(&mut self.items);
        let mut occupied: Vec<bool> = vec![false; (self.cols * self.rows) as usize];
        for (i, item) in items.into_iter().enumerate() {
            let row = (i as u16) / self.cols;
            let col = (i as u16) % self.cols;
            if let Some(item) = item {
                if let Some(span_area) = self.compute_square_pos_provided_cells(
                    &cells,
                    GridSpan {
                        row,
                        col,
                        row_span: item.row_span,
                        col_span: item.col_span,
                    },
                ) {
                    for r in 0..item.row_span {
                        for c in 0..item.col_span {
                            occupied[i + (c + r * self.cols) as usize] = true;
                        }
                    }
                    (item.renderer)(span_area, buf); // lol
                }
                //} else if let Some(span_area) = self.get_span(row, col, 1, 1) {
                //    if !occupied[i] {
                //        Paragraph::new(format!("Placeholder #{i}"))
                //            .block(Block::bordered())
                //            .centered()
                //            .render(span_area, buf);
                //    }
                //}
            }
        }
    }
}
fn mode_tab(
    inner: Rect,
    buf: &mut Buffer,
    mode: CounterMode,
    tab_data: UISettingsTabData,
) -> Block<'static> {
    let this_tab = SettingsTabs::Mode;
    let selected_tab = tab_data.selected_tab;
    let outer_block = UIHelper::create_settings_block(
        "Mode",
        GREEN,
        selected_tab,
        this_tab,
        tab_data.current_mode,
    );
    let horizontal_layout = Layout::default()
        .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(30)])
        .flex(Flex::Center)
        .direction(Direction::Horizontal)
        .split(outer_block.inner(inner));
    let vertical_layout1 = Layout::default()
        .constraints(vec![Constraint::Percentage(80)])
        .direction(Direction::Vertical)
        .flex(Flex::End)
        .split(horizontal_layout[0]);
    let vertical_layout2 = Layout::default()
        .constraints(vec![Constraint::Percentage(60)])
        .direction(Direction::Vertical)
        .flex(Flex::End)
        .split(horizontal_layout[1]);
    let selected_style = Style::new().bold().fg(YELLOW);
    let not_selected_style = Style::new().fg(Color::White);
    let (pomodoro_text, pomodoro_style) = match mode {
        CounterMode::Countup => ("Pomodoro  ", not_selected_style),
        CounterMode::Countdown => ("Pomodoro <", selected_style),
    };
    let (flowmodoro_text, flowmodoro_style) = match mode {
        CounterMode::Countup => ("> Flowmodoro", selected_style),
        CounterMode::Countdown => ("  Flowmodoro", not_selected_style),
    };
    let pomodoro = Paragraph::new(pomodoro_text)
        .alignment(Alignment::Center)
        .style(pomodoro_style);
    let flowmodoro = Paragraph::new(flowmodoro_text)
        .alignment(Alignment::Center)
        .style(flowmodoro_style);
    pomodoro.render(vertical_layout1[0], buf);
    flowmodoro.render(vertical_layout2[0], buf);
    outer_block
}
fn pomodoro_tab(
    mode_inner: Rect,
    buf: &mut Buffer,
    timer_settings: &TimerSettings,
    tab_data: UISettingsTabData,
) -> Block<'static> {
    let this_tab = SettingsTabs::Pomodoro;
    let (selected_num, selected_tab, current_mode) = (
        tab_data.selected_setting,
        tab_data.selected_tab,
        tab_data.current_mode,
    );
    let outer_block =
        UIHelper::create_settings_block("Pomodoro", BLUE, selected_tab, this_tab, current_mode);

    if timer_settings.mode == CounterMode::Countup {
        let blocked = UIHelper::create_settings_paragraph("Pomodoro settings not available", None);
        blocked.render(outer_block.inner(mode_inner), buf);
        return outer_block
            .border_style(Style::default().fg(Color::Gray))
            .border_type(BorderType::LightQuadrupleDashed);
    }

    let horizontal_layout = Layout::default()
        .constraints(vec![Constraint::Percentage(40), Constraint::Percentage(30)])
        .direction(Direction::Horizontal)
        .flex(Flex::Center)
        .split(outer_block.inner(mode_inner));
    let vertical_layout = Layout::default()
        .constraints(vec![
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .direction(Direction::Vertical)
        .flex(Flex::Center);
    let vertical_layout1 = vertical_layout.split(horizontal_layout[0]);
    let vertical_layout2 = vertical_layout.split(horizontal_layout[1]);
    let work_time_label = Paragraph::new("Work time")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    let work_time_value = UIHelper::create_settings_paragraph(
        &format!("{} min", timer_settings.work_time / 60),
        Some(SettingsTab::highlight_selected(
            selected_num,
            selected_tab,
            this_tab,
            0,
            current_mode,
        )),
    );

    let break_time_label = Paragraph::new("Break time")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    let break_time_value = UIHelper::create_settings_paragraph(
        &format!("{} min", timer_settings.break_time / 60),
        Some(SettingsTab::highlight_selected(
            selected_num,
            selected_tab,
            this_tab,
            1,
            current_mode,
        )),
    );

    let iterations_label = Paragraph::new("Iterations")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    let iterations_value = UIHelper::create_settings_paragraph(
        &format!("{} iter(s)", timer_settings.iterations),
        Some(SettingsTab::highlight_selected(
            selected_num,
            selected_tab,
            this_tab,
            2,
            current_mode,
        )),
    );
    work_time_label.render(vertical_layout1[0], buf);
    break_time_label.render(vertical_layout1[1], buf);
    iterations_label.render(vertical_layout1[2], buf);
    work_time_value.render(vertical_layout2[0], buf);
    break_time_value.render(vertical_layout2[1], buf);
    iterations_value.render(vertical_layout2[2], buf);
    outer_block
}
fn preferences_tab(
    mode_inner: Rect,
    buf: &mut Buffer,
    ui_settings: &UISettings,
    tab_data: UISettingsTabData,
) -> Block<'static> {
    let this_tab = SettingsTabs::Preferences;
    let (selected_num, selected_tab, current_mode) = (
        tab_data.selected_setting,
        tab_data.selected_tab,
        tab_data.current_mode,
    );
    let outer_block =
        UIHelper::create_settings_block(" Preferences ", RED, selected_tab, this_tab, current_mode)
            .title_position(ratatui::widgets::TitlePosition::Bottom);
    let horizontal_layout = Layout::default()
        .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(30)])
        .direction(Direction::Horizontal)
        .flex(Flex::Start)
        .split(outer_block.inner(mode_inner));
    let vertical_layout = Layout::default()
        .constraints(vec![Constraint::Length(2), Constraint::Length(2)])
        .flex(Flex::Center)
        .direction(Direction::Vertical);
    let settings_layout = Layout::default()
        .constraints(vec![Constraint::Length(3), Constraint::Length(3)])
        .direction(Direction::Horizontal)
        .spacing(2)
        .flex(Flex::Center);
    let pause_label = Paragraph::new("Pause after every cycle")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray))
        .right_aligned();
    let pause_value_yes = UIHelper::create_settings_paragraph(
        "yes",
        Some(UIHelper::format_yes_no(
            0,
            this_tab,
            tab_data,
            true,
            ui_settings.pause_after_state_change,
        )),
    );
    let pause_value_no = UIHelper::create_settings_paragraph(
        "no",
        Some(UIHelper::format_yes_no(
            0,
            this_tab,
            tab_data,
            false,
            ui_settings.pause_after_state_change,
        )),
    );

    let hide_label = Paragraph::new("Make the numbers smaller")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray))
        .right_aligned();
    let hide_value_yes = UIHelper::create_settings_paragraph(
        "yes",
        Some(UIHelper::format_yes_no(
            1,
            this_tab,
            tab_data,
            true,
            ui_settings.hide_work_countdown,
        )),
    );
    let hide_value_no = UIHelper::create_settings_paragraph(
        "no",
        Some(UIHelper::format_yes_no(
            1,
            this_tab,
            tab_data,
            false,
            ui_settings.hide_work_countdown,
        )),
    );
    let vertical_layout1 = vertical_layout.split(horizontal_layout[0]);
    let vertical_layout2 = vertical_layout.split(horizontal_layout[1]);
    let settings_layout_vert1 = settings_layout.split(vertical_layout2[0]);
    let settings_layout_vert2 = settings_layout.split(vertical_layout2[1]);

    pause_label.render(vertical_layout1[0], buf);
    hide_label.render(vertical_layout1[1], buf);

    pause_value_yes.render(settings_layout_vert1[0], buf);
    pause_value_no.render(settings_layout_vert1[1], buf);
    hide_value_yes.render(settings_layout_vert2[0], buf);
    hide_value_no.render(settings_layout_vert2[1], buf);
    outer_block
}
fn stats_tab(
    inner: Rect,
    buf: &mut Buffer,
    stats_settings: &StatsSettings,
    tab_data: UISettingsTabData,
) -> Block<'static> {
    let this_tab = SettingsTabs::Stats;
    let (selected_num, selected_tab, current_mode) = (
        tab_data.selected_setting,
        tab_data.selected_tab,
        tab_data.current_mode,
    );
    let outer =
        UIHelper::create_settings_block(" Stats ", YELLOW, selected_tab, this_tab, current_mode);
    let main_vert_layout = Layout::default()
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .flex(Flex::Center)
        .spacing(1)
        .direction(Direction::Vertical)
        .split(outer.inner(inner));
    let option_layout = Layout::default()
        .constraints(vec![Constraint::Length(1), Constraint::Length(1)])
        .direction(Direction::Vertical)
        .flex(Flex::Center);
    let first_row_layout = Layout::vertical(vec![Constraint::Length(2), Constraint::Length(1)])
        .split(main_vert_layout[0]);
    let spacer_border_layout = Layout::horizontal([Constraint::Percentage(50)])
        .flex(Flex::Center)
        .split(first_row_layout[1]);
    let centering_options_layout =
        Layout::horizontal([Constraint::Percentage(40)]).flex(Flex::Center);
    let tracking_label_layout = Layout::default()
        .constraints(vec![Constraint::Length(12), Constraint::Length(7)])
        .direction(Direction::Horizontal)
        .flex(Flex::Center)
        .spacing(8)
        .split(first_row_layout[0]);
    let spacer_block = Block::bordered()
        .borders(Borders::BOTTOM)
        .border_type(BorderType::LightDoubleDashed);
    spacer_block.render(spacer_border_layout[0], buf);
    let yes_no_layout = Layout::default()
        .constraints(vec![Constraint::Length(3), Constraint::Length(3)])
        .direction(Direction::Horizontal)
        .spacing(2)
        .flex(Flex::Center)
        .split(tracking_label_layout[1]);
    let username_box = centering_options_layout.split(main_vert_layout[1]);
    let api_token_box = centering_options_layout.split(main_vert_layout[2]);
    let username_layout = option_layout.split(username_box[0]);
    let api_key_layout = option_layout.split(api_token_box[0]);

    let tracking_label = Paragraph::new("Enable stats")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    let tracking_yes = UIHelper::create_settings_paragraph(
        "yes",
        Some(UIHelper::format_yes_no(
            0,
            this_tab,
            tab_data,
            true,
            stats_settings.stats_on,
        )),
    );
    let tracking_no = UIHelper::create_settings_paragraph(
        "no",
        Some(UIHelper::format_yes_no(
            0,
            this_tab,
            tab_data,
            false,
            stats_settings.stats_on,
        )),
    );

    let username_label = Paragraph::new("Username")
        .alignment(Alignment::Center)
        .style(if stats_settings.stats_on {
            Style::default().fg(Color::Gray)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .left_aligned();
    let mut username_value = UIHelper::create_settings_paragraph(
        stats_settings.pixela_username.as_deref().unwrap_or("—"),
        Some(if stats_settings.stats_on {
            SettingsTab::highlight_selected(selected_num, selected_tab, this_tab, 1, current_mode)
        } else {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        }),
    )
    .left_aligned();

    let token_label = Paragraph::new("Api key")
        .alignment(Alignment::Center)
        .style(if stats_settings.stats_on {
            Style::default().fg(Color::Gray)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .left_aligned();
    let token_display = stats_settings
        .pixela_token
        .as_ref()
        .map(|t| "•".repeat(t.len().min(12)))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "—".to_string());
    let mut token_value = UIHelper::create_settings_paragraph(
        &token_display,
        Some(if stats_settings.stats_on {
            SettingsTab::highlight_selected(selected_num, selected_tab, this_tab, 2, current_mode)
        } else {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        }),
    )
    .left_aligned();
    match selected_num {
        1 if current_mode == Mode::Input => username_value = username_value.rapid_blink(),
        2 if current_mode == Mode::Input => token_value = token_value.slow_blink(),
        _ => {}
    }
    tracking_label.render(tracking_label_layout[0], buf);
    tracking_yes.render(yes_no_layout[0], buf);
    tracking_no.render(yes_no_layout[1], buf);
    username_label.render(username_layout[0], buf);
    username_value.render(username_layout[1], buf);
    token_label.render(api_key_layout[0], buf);
    token_value.render(api_key_layout[1], buf);
    outer
}
