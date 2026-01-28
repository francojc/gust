//! Humidity area chart.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols::Marker,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use super::{create_day_labels, get_day_boundary_positions, get_tick_positions, x_position};
use crate::app::HourlyForecast;
use crate::ui::theme::Theme;

/// Data structure for humidity graph rendering.
#[derive(Debug, Clone)]
pub struct HumidityData {
    /// Data points as (x, y) tuples where x is time index and y is humidity percentage.
    pub points: Vec<(f64, f64)>,
    /// X-axis labels (day names only).
    pub x_labels: Vec<String>,
    /// X-positions for day labels.
    pub label_positions: Vec<f64>,
    /// X-positions of day boundaries for rendering separator lines.
    pub day_boundaries: Vec<f64>,
    /// X-positions for 6-hour tick marks.
    pub tick_positions: Vec<f64>,
}

impl HumidityData {
    /// Create humidity data from hourly forecast.
    pub fn from_hourly(hourly: &[HourlyForecast], timezone: Option<&str>) -> Self {
        if hourly.is_empty() {
            return Self::empty();
        }

        let points: Vec<(f64, f64)> = hourly
            .iter()
            .enumerate()
            .map(|(i, h)| (x_position(i, hourly.len()), h.humidity as f64))
            .collect();

        let (x_labels, label_positions) = create_day_labels(hourly, timezone);
        let day_boundaries = get_day_boundary_positions(hourly);
        let tick_positions = get_tick_positions(hourly);

        Self {
            points,
            x_labels,
            label_positions,
            day_boundaries,
            tick_positions,
        }
    }

    /// Create empty humidity data.
    pub fn empty() -> Self {
        Self {
            points: vec![],
            x_labels: vec![],
            label_positions: vec![],
            day_boundaries: vec![],
            tick_positions: vec![],
        }
    }
}

/// Render the humidity graph.
pub fn render(data: &HumidityData, theme: &Theme, frame: &mut Frame, area: Rect) {
    if data.points.is_empty() {
        render_empty(theme, frame, area);
        return;
    }

    let x_bounds = [0.0, (data.points.len() - 1).max(1) as f64];
    let y_min = 0.0;
    let y_max = 100.0;

    // Create datasets: main humidity line + day boundary separators + tick marks
    let mut datasets = Vec::new();

    // Add 6-hour tick marks at the bottom of the chart
    let tick_height = (y_max - y_min) * 0.05; // 5% of chart height
    let tick_style = Style::default().fg(theme.muted);
    for &x in &data.tick_positions {
        let tick_data: Vec<(f64, f64)> = vec![(x, y_min), (x, y_min + tick_height)];
        datasets.push(
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(tick_style)
                .data(tick_data.leak()),
        );
    }

    // Add day boundary separator lines (so they render behind the data)
    let separator_style = Style::default().fg(theme.muted);
    for &x in &data.day_boundaries {
        let separator_data: Vec<(f64, f64)> = vec![(x, y_min), (x, y_max)];
        datasets.push(
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(separator_style)
                .data(separator_data.leak()),
        );
    }

    // Add main humidity line
    datasets.push(
        Dataset::default()
            .name("Humidity %")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&data.points),
    );

    let x_labels: Vec<ratatui::text::Span> = data
        .x_labels
        .iter()
        .map(|s| ratatui::text::Span::raw(s.clone()))
        .collect();

    let y_labels: Vec<ratatui::text::Span> = vec![
        ratatui::text::Span::raw("0"),
        ratatui::text::Span::raw("50"),
        ratatui::text::Span::raw("100"),
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Humidity (%) ")
                .border_style(Style::default().fg(theme.muted)),
        )
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(theme.muted))
                .bounds(x_bounds)
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("%")
                .style(Style::default().fg(theme.muted))
                .bounds([y_min, y_max])
                .labels(y_labels),
        );

    frame.render_widget(chart, area);
}

/// Render an empty state message.
fn render_empty(theme: &Theme, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Humidity (%) ")
        .border_style(Style::default().fg(theme.muted));

    let paragraph = ratatui::widgets::Paragraph::new("No data available")
        .block(block)
        .style(Style::default().fg(theme.muted));

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    fn create_mock_hourly() -> Vec<HourlyForecast> {
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        (0..24)
            .map(|i| HourlyForecast {
                time: base_date.and_hms_opt(i, 0, 0).unwrap(),
                temperature: 42.0,
                precipitation_probability: 10,
                humidity: 54 + ((i as f64 * 1.0) as u8),
                wind_speed: 5.0,
            })
            .collect()
    }

    fn render_to_string(data: &HumidityData, theme: &Theme) -> String {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                render(data, theme, frame, area);
            })
            .unwrap();

        let backend = terminal.backend();
        let buffer = backend.buffer();
        let mut output = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                output.push_str(&cell.symbol());
            }
            output.push('\n');
        }
        output
    }

    #[test]
    fn test_humidity_graph_renders() {
        let hourly = create_mock_hourly();
        let data = HumidityData::from_hourly(&hourly, None);
        let theme = Theme::dark();
        let output = render_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_humidity_graph_empty_data() {
        let data = HumidityData::empty();
        let theme = Theme::dark();
        let output = render_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_humidity_data_from_hourly() {
        let hourly = create_mock_hourly();
        let data = HumidityData::from_hourly(&hourly, None);

        assert_eq!(data.points.len(), 24);
        assert_eq!(data.points[0].1, 54.0);
    }
}
