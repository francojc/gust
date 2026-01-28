//! Temperature graph using Braille rendering.

use ratatui::{
    layout::Rect,
    style::Style,
    symbols::Marker,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use super::{calculate_bounds, create_day_labels, get_day_boundary_positions, get_tick_positions, x_position};
use crate::app::HourlyForecast;
use crate::ui::theme::Theme;

/// Data structure for temperature graph rendering.
#[derive(Debug, Clone)]
pub struct TemperatureData {
    /// Data points as (x, y) tuples where x is time index and y is temperature.
    pub points: Vec<(f64, f64)>,
    /// Y-axis bounds [min, max].
    pub y_bounds: [f64; 2],
    /// X-axis labels (day names only).
    pub x_labels: Vec<String>,
    /// X-positions for day labels.
    pub label_positions: Vec<f64>,
    /// Y-axis labels.
    pub y_labels: Vec<String>,
    /// X-positions of day boundaries for rendering separator lines.
    pub day_boundaries: Vec<f64>,
    /// X-positions for 6-hour tick marks.
    pub tick_positions: Vec<f64>,
}

impl TemperatureData {
    /// Create temperature data from hourly forecast.
    pub fn from_hourly(hourly: &[HourlyForecast], timezone: Option<&str>) -> Self {
        if hourly.is_empty() {
            return Self::empty();
        }

        let temps: Vec<f64> = hourly.iter().map(|h| h.temperature).collect();
        let y_bounds = calculate_bounds(&temps);

        let points: Vec<(f64, f64)> = hourly
            .iter()
            .enumerate()
            .map(|(i, h)| (x_position(i, hourly.len()), h.temperature))
            .collect();

        let (x_labels, label_positions) = create_day_labels(hourly, timezone);
        let y_labels = Self::create_y_labels(y_bounds);
        let day_boundaries = get_day_boundary_positions(hourly);
        let tick_positions = get_tick_positions(hourly);

        Self {
            points,
            y_bounds,
            x_labels,
            label_positions,
            y_labels,
            day_boundaries,
            tick_positions,
        }
    }

    /// Create empty temperature data.
    pub fn empty() -> Self {
        Self {
            points: vec![],
            y_bounds: [0.0, 100.0],
            x_labels: vec![],
            label_positions: vec![],
            y_labels: vec!["0".to_string(), "50".to_string(), "100".to_string()],
            day_boundaries: vec![],
            tick_positions: vec![],
        }
    }

    /// Create Y-axis labels based on bounds.
    fn create_y_labels(bounds: [f64; 2]) -> Vec<String> {
        let min = bounds[0];
        let max = bounds[1];
        let mid = (min + max) / 2.0;

        vec![
            format!("{:.0}", min),
            format!("{:.0}", mid),
            format!("{:.0}", max),
        ]
    }
}

/// Render the temperature graph.
pub fn render(data: &TemperatureData, theme: &Theme, frame: &mut Frame, area: Rect) {
    if data.points.is_empty() {
        render_empty(theme, frame, area);
        return;
    }

    let x_bounds = [0.0, (data.points.len() - 1).max(1) as f64];
    let y_min = data.y_bounds[0];
    let y_max = data.y_bounds[1];

    // Create datasets: main temperature line + day boundary separators + tick marks
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

    // Add main temperature line
    datasets.push(
        Dataset::default()
            .name("Temperature")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.accent))
            .data(&data.points),
    );

    let x_labels: Vec<ratatui::text::Span> = data
        .x_labels
        .iter()
        .map(|s| ratatui::text::Span::raw(s.clone()))
        .collect();

    let y_labels: Vec<ratatui::text::Span> = data
        .y_labels
        .iter()
        .map(|s| ratatui::text::Span::raw(s.clone()))
        .collect();

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Temperature (F) ")
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
                .title("F")
                .style(Style::default().fg(theme.muted))
                .bounds(data.y_bounds)
                .labels(y_labels),
        );

    frame.render_widget(chart, area);
}

/// Render an empty state message.
fn render_empty(theme: &Theme, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Temperature (F) ")
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
                temperature: 32.0 + (i as f64 * 0.5),
                precipitation_probability: 10,
                humidity: 60,
                wind_speed: 5.0,
            })
            .collect()
    }

    fn render_to_string(data: &TemperatureData, theme: &Theme) -> String {
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
    fn test_temperature_graph_renders() {
        let hourly = create_mock_hourly();
        let data = TemperatureData::from_hourly(&hourly, None);
        let theme = Theme::dark();
        let output = render_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_temperature_graph_empty_data() {
        let data = TemperatureData::empty();
        let theme = Theme::dark();
        let output = render_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_temperature_data_from_hourly() {
        let hourly = create_mock_hourly();
        let data = TemperatureData::from_hourly(&hourly, None);

        assert_eq!(data.points.len(), 24);
        assert!(data.y_bounds[0] < 32.0);
        assert!(data.y_bounds[1] > 43.5);
    }

    #[test]
    fn test_temperature_data_empty() {
        let hourly: Vec<HourlyForecast> = vec![];
        let data = TemperatureData::from_hourly(&hourly, None);

        assert!(data.points.is_empty());
        assert_eq!(data.y_bounds, [0.0, 100.0]);
    }
}
