//! Precipitation bar chart.

use ratatui::{
    layout::Rect,
    style::Style,
    symbols::Marker,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use super::{create_time_labels_with_dates, get_day_boundary_positions, x_position};
use crate::app::HourlyForecast;
use crate::ui::theme::Theme;

/// Data structure for precipitation graph rendering.
#[derive(Debug, Clone)]
pub struct PrecipitationData {
    /// Data points as (x, y) tuples where x is time index and y is precipitation probability.
    pub points: Vec<(f64, f64)>,
    /// X-axis labels.
    pub x_labels: Vec<String>,
    /// X-positions of day boundaries for rendering separator lines.
    pub day_boundaries: Vec<f64>,
}

impl PrecipitationData {
    /// Create precipitation data from hourly forecast.
    pub fn from_hourly(hourly: &[HourlyForecast], timezone: Option<&str>) -> Self {
        if hourly.is_empty() {
            return Self::empty();
        }

        let points: Vec<(f64, f64)> = hourly
            .iter()
            .enumerate()
            .map(|(i, h)| (x_position(i, hourly.len()), h.precipitation_probability as f64))
            .collect();

        let x_labels = create_time_labels_with_dates(hourly, timezone);
        let day_boundaries = get_day_boundary_positions(hourly);

        Self {
            points,
            x_labels,
            day_boundaries,
        }
    }

    /// Create empty precipitation data.
    pub fn empty() -> Self {
        Self {
            points: vec![],
            x_labels: vec![],
            day_boundaries: vec![],
        }
    }
}

/// Render the precipitation graph.
pub fn render(data: &PrecipitationData, theme: &Theme, frame: &mut Frame, area: Rect) {
    if data.points.is_empty() {
        render_empty(theme, frame, area);
        return;
    }

    let x_bounds = [0.0, (data.points.len() - 1).max(1) as f64];
    let y_min = 0.0;
    let y_max = 100.0;

    // Create datasets: main precipitation scatter + day boundary separators
    let mut datasets = Vec::new();

    // Add day boundary separator lines first (so they render behind the data)
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

    // Add main precipitation scatter
    datasets.push(
        Dataset::default()
            .name("Precipitation %")
            .marker(Marker::Block)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(ratatui::style::Color::Blue))
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
                .bounds([y_min, y_max])
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
        let data = PrecipitationData::from_hourly(&hourly, None);
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
        let data = PrecipitationData::from_hourly(&hourly, None);

        assert_eq!(data.points.len(), 24);
        assert_eq!(data.points[0].1, 0.0);
    }
}
