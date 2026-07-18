use crate::dsp::state::AppState;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Widget};

pub fn render(
    f: &mut Frame,
    app_state: &AppState,
    devices: &[(usize, String)],
    list_state: &mut ListState,
) {
    let area = f.area();

    match app_state.get_view() {
        crate::dsp::state::AppView::SelectDevice => {
            let items: Vec<ListItem> = devices
                .iter()
                .map(|(idx, name)| ListItem::new(format!("[{}] {}", idx, name)))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(" Select Audio Input Device ")
                        .borders(Borders::ALL),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Cyan)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            let popup_area = centered_rect(60, 40, area);
            f.render_stateful_widget(list, popup_area, list_state);
        }
        crate::dsp::state::AppView::Visualizer => render_visualizer(f, area, app_state),
    }
}

fn render_visualizer(f: &mut Frame, area: Rect, app_state: &AppState) {
    let bar_count = app_state.get_num_bars();
    let speed = (app_state.get_response() * 100.0).round() as u8;
    let block = Block::default()
        .title(format!(
            " Live Audio Spectrum | Bars: {bar_count} Up/Down | Speed: {speed}% Left/Right | q Quit "
        ))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let regions = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);
    let current_data = app_state.get_bins();

    f.render_widget(SpectrumBars::new(&current_data, bar_count), regions[0]);
    f.render_widget(FrequencyAxis, regions[1]);
}

struct SpectrumBars<'a> {
    values: &'a [f32],
    requested_bars: usize,
}

impl<'a> SpectrumBars<'a> {
    fn new(values: &'a [f32], requested_bars: usize) -> Self {
        Self {
            values,
            requested_bars,
        }
    }
}

impl Widget for SpectrumBars<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if area.is_empty() || self.requested_bars == 0 {
            return;
        }

        let visible_bars = self.requested_bars.min(area.width as usize);
        let max_value = self.values.iter().copied().fold(0.0f32, f32::max);
        if visible_bars == 0 || max_value <= f32::EPSILON {
            return;
        }

        for index in 0..visible_bars {
            let start_x = area.x + (index as u16 * area.width / visible_bars as u16);
            let end_x = area.x + ((index as u16 + 1) * area.width / visible_bars as u16);
            let value = aggregated_value(self.values, index, visible_bars);
            let filled = value / max_value * area.height as f32;

            for row in 0..area.height {
                let level = (filled - (area.height - row - 1) as f32).clamp(0.0, 1.0);
                if level > 0.0 {
                    let symbol = bar_symbol(level);
                    for x in start_x..end_x {
                        buffer[(x, area.y + row)]
                            .set_symbol(symbol)
                            .set_style(Style::default().fg(Color::Cyan));
                    }
                }
            }
        }
    }
}

fn bar_symbol(level: f32) -> &'static str {
    match (level * 8.0).ceil() as u8 {
        1 => symbols::bar::ONE_EIGHTH,
        2 => symbols::bar::ONE_QUARTER,
        3 => symbols::bar::THREE_EIGHTHS,
        4 => symbols::bar::HALF,
        5 => symbols::bar::FIVE_EIGHTHS,
        6 => symbols::bar::THREE_QUARTERS,
        7 => symbols::bar::SEVEN_EIGHTHS,
        _ => symbols::bar::FULL,
    }
}

fn aggregated_value(values: &[f32], index: usize, bar_count: usize) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    let start = index * values.len() / bar_count;
    let end = ((index + 1) * values.len() / bar_count)
        .max(start + 1)
        .min(values.len());
    values[start..end].iter().copied().fold(0.0, f32::max)
}

struct FrequencyAxis;

impl Widget for FrequencyAxis {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        let labels = [
            ("20Hz", 20.0f32),
            ("250Hz", 250.0f32),
            ("2kHz", 2_000.0f32),
            ("10kHz", 10_000.0f32),
            ("20kHz", 20_000.0f32),
        ];
        let min_log = 20.0f32.log10();
        let log_span = 20_000.0f32.log10() - min_log;
        let mut next_available = 0usize;

        for (label, frequency) in labels {
            if label.len() > area.width as usize {
                continue;
            }
            let position = ((frequency.log10() - min_log) / log_span * (area.width - 1) as f32)
                .round() as usize;
            let x = position
                .saturating_sub(label.len() / 2)
                .min(area.width as usize - label.len());
            if x < next_available {
                continue;
            }
            buffer.set_string(
                area.x + x as u16,
                area.y,
                label,
                Style::default().fg(Color::DarkGray),
            );
            next_available = x + label.len() + 1;
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::aggregated_value;

    #[test]
    fn aggregation_preserves_the_peak_in_each_visible_bar() {
        assert_eq!(aggregated_value(&[0.1, 0.8, 0.3, 0.5], 0, 2), 0.8);
        assert_eq!(aggregated_value(&[0.1, 0.8, 0.3, 0.5], 1, 2), 0.5);
    }
}
