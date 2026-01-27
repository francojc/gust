//! Precipitation bar chart.

use ratatui::{
    layout::Rect,
    style::Style,
    symbols::Marker,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use super::{create_time_labels, x_position};
use crate::app::HourlyForecast;
use crate::ui::theme::Theme;

/// Data structure for precipitation graph rendering.
#[derive(Debug, Clone)]
pub struct PrecipitationData {
    /// Data points as (x, y) tuples where x is time index and y is precipitation probability.
    pub points: Vec<(f64, f64)>,
    /// X-axis labels.
    pub x_labels: Vec<String>,
}

impl PrecipitationData {
    /// Create precipitation data from hourly forecast.
    pub fn from_hourly(hourly: &[HourlyForecast]) -> Self {
        if hourly.is_empty() {
            return Self::empty();
        }

        let points: Vec<(f64, f64)> = hourly
            .iter()
            .enumerate()
            .map(|(i, h)| (x_position(i, hourly.len()), h.precipitation_probability as f64))
            .collect();

        let x_labels: Vec<String> = create_time_labels(hourly)
            .iter()
            .map(|s| s.content.to_string())
            .collect();

        Self { points, x_labels }
    }

    /// Create empty precipitation data.
    pub fn empty() -> Self {
        Self {
            points: vec![],
            x_labels: vec![],
        }
    }
}

/// Render the precipitation graph.
pub fn render(data: &PrecipitationData, theme: &Theme, frame: &mut Frame, area: Rect) {
    if data.points.is_empty() {
        render_empty(theme, frame, area);
        return;
    }

    // Use Bar graph type with wider markers to simulate bars
    let dataset = Dataset::default()
        .name("Precipitation %")
        .marker(Marker::Block)
        .graph_type(GraphType::Scatter)
        .style(Style::default().fg(ratatui::style::Color::Blue))
        .data(&data.points);

    let x_bounds = [0.0, (data.points.len() - 1).max(1) as f64];

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

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Precipitation Probability (%) ")
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
                .bounds([0.0, 100.0])
                .labels(y_labels),
        );

    frame.render_widget(chart, area);
}

/// Render an empty state message.
fn render_empty(theme: &Theme, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Precipitation Probability (%) ")
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
                precipitation_probability: ((i as f64 * 1.5) as u8).min(100),
                humidity: 60,
                wind_speed: 5.0,
            })
            .collect()
    }

    fn render_to_string(data: &PrecipitationData, theme: &Theme) -> String {
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
    fn test_precipitation_graph_renders() {
        let hourly = create_mock_hourly();
        let data = PrecipitationData::from_hourly(&hourly);
        let theme = Theme::dark();
        let output = render_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_precipitation_graph_empty_data() {
        let data = PrecipitationData::empty();
        let theme = Theme::dark();
        let output = render_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_precipitation_data_from_hourly() {
        let hourly = create_mock_hourly();
        let data = PrecipitationData::from_hourly(&hourly);

        assert_eq!(data.points.len(), 24);
        assert_eq!(data.points[0].1, 0.0);
    }
}
