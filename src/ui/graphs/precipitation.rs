//! Precipitation chart with color-coded precipitation types.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols::Marker,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

use super::{create_day_labels, get_day_boundary_positions, get_tick_positions, x_position};
use crate::app::{HourlyForecast, PrecipType};
use crate::ui::theme::Theme;

/// Data structure for precipitation graph rendering.
#[derive(Debug, Clone)]
pub struct PrecipitationData {
    /// Rain data points (x, y) - green color.
    pub rain_points: Vec<(f64, f64)>,
    /// Snow data points (x, y) - blue/violet color.
    pub snow_points: Vec<(f64, f64)>,
    /// Mixed precipitation data points (x, y) - slate blue color.
    pub mixed_points: Vec<(f64, f64)>,
    /// Total number of data points (for x-axis bounds).
    pub total_points: usize,
    /// X-axis labels (day names only).
    pub x_labels: Vec<String>,
    /// X-positions for day labels.
    pub label_positions: Vec<f64>,
    /// X-positions of day boundaries for rendering separator lines.
    pub day_boundaries: Vec<f64>,
    /// X-positions for 6-hour tick marks.
    pub tick_positions: Vec<f64>,
}

impl PrecipitationData {
    /// Create precipitation data from hourly forecast.
    pub fn from_hourly(hourly: &[HourlyForecast], timezone: Option<&str>, use_24h: bool) -> Self {
        if hourly.is_empty() {
            return Self::empty();
        }

        let mut rain_points = Vec::new();
        let mut snow_points = Vec::new();
        let mut mixed_points = Vec::new();

        for (i, h) in hourly.iter().enumerate() {
            let x = x_position(i, hourly.len());
            let y = h.precipitation_probability as f64;

            // Only add points where there's precipitation probability
            if h.precipitation_probability > 0 {
                match h.precip_type {
                    PrecipType::Rain => rain_points.push((x, y)),
                    PrecipType::Snow => snow_points.push((x, y)),
                    PrecipType::Mixed => mixed_points.push((x, y)),
                    PrecipType::None => {
                        // If probability > 0 but type is None, default to rain
                        rain_points.push((x, y));
                    }
                }
            }
        }

        let (x_labels, label_positions) = create_day_labels(hourly, timezone, use_24h);
        let day_boundaries = get_day_boundary_positions(hourly);
        let tick_positions = get_tick_positions(hourly);

        Self {
            rain_points,
            snow_points,
            mixed_points,
            total_points: hourly.len(),
            x_labels,
            label_positions,
            day_boundaries,
            tick_positions,
        }
    }

    /// Create empty precipitation data.
    pub fn empty() -> Self {
        Self {
            rain_points: vec![],
            snow_points: vec![],
            mixed_points: vec![],
            total_points: 0,
            x_labels: vec![],
            label_positions: vec![],
            day_boundaries: vec![],
            tick_positions: vec![],
        }
    }

    /// Check if there's any precipitation data.
    pub fn has_data(&self) -> bool {
        !self.rain_points.is_empty()
            || !self.snow_points.is_empty()
            || !self.mixed_points.is_empty()
    }
}

/// Render the precipitation graph.
pub fn render(data: &PrecipitationData, theme: &Theme, frame: &mut Frame, area: Rect) {
    if data.total_points == 0 {
        render_empty(theme, frame, area);
        return;
    }

    let x_bounds = [0.0, (data.total_points - 1).max(1) as f64];
    let y_min = 0.0;
    let y_max = 100.0;

    // Create datasets: tick marks + day boundaries + precipitation lines by type
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

    // Add precipitation lines by type with NOAA/NEXRAD-inspired colors
    // Rain - Green (standard radar rain color)
    if !data.rain_points.is_empty() {
        datasets.push(
            Dataset::default()
                .name("Rain")
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Green))
                .data(data.rain_points.clone().leak()),
        );
    }

    // Snow - Blue/Magenta (NWS heavy snow color)
    if !data.snow_points.is_empty() {
        datasets.push(
            Dataset::default()
                .name("Snow")
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Magenta))
                .data(data.snow_points.clone().leak()),
        );
    }

    // Mixed - Cyan (freezing rain/sleet)
    if !data.mixed_points.is_empty() {
        datasets.push(
            Dataset::default()
                .name("Mixed")
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(data.mixed_points.clone().leak()),
        );
    }

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
            .map(|i| {
                // Assign precipitation types: 0-7 none, 8-15 rain, 16-19 snow, 20-23 mixed
                let precip_type = if i < 8 {
                    PrecipType::None
                } else if i < 16 {
                    PrecipType::Rain
                } else if i < 20 {
                    PrecipType::Snow
                } else {
                    PrecipType::Mixed
                };
                HourlyForecast {
                    time: base_date.and_hms_opt(i, 0, 0).unwrap(),
                    temperature: 42.0,
                    precipitation_probability: ((i as f64 * 1.5) as u8).min(100),
                    humidity: 60,
                    wind_speed: 5.0,
                    precip_type,
                    uv_index: 0.0,
                }
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
        let data = PrecipitationData::from_hourly(&hourly, None, false);
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
        let data = PrecipitationData::from_hourly(&hourly, None, false);

        assert_eq!(data.total_points, 24);
        // Hours 0-7 have precip_probability 0-10, but type is None
        // Hours 8-15 are Rain with probability 12-22
        // Hours 16-19 are Snow with probability 24-28
        // Hours 20-23 are Mixed with probability 30-34 (capped at 100)
        assert!(!data.rain_points.is_empty());
        assert!(!data.snow_points.is_empty());
        assert!(!data.mixed_points.is_empty());
    }
}
