//! Header widget for current conditions.

use chrono::{NaiveDate, NaiveDateTime};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::theme::Theme;
use crate::app::AppState;
use crate::astro::{format_daylight_duration, moon_phase};

/// Data needed to render the header.
#[derive(Debug, Clone, Default)]
pub struct HeaderData {
    pub location_name: String,
    pub temperature: Option<f64>,
    pub feels_like: Option<f64>,
    pub wind_speed: Option<f64>,
    pub wind_direction: Option<String>,
    pub pressure: Option<f64>,
    pub weather_code: Option<u8>,
    pub uv_index: Option<f64>,
    pub aqi: Option<u16>,
    pub sunrise: Option<NaiveDateTime>,
    pub sunset: Option<NaiveDateTime>,
    pub daylight_duration: Option<f64>,
    pub date: Option<NaiveDate>,
}

impl HeaderData {
    /// Create header data from the application state.
    pub fn from_state(state: &AppState) -> Self {
        let location_name = if state.location.name.is_empty() {
            "No location selected".to_string()
        } else {
            state.location.name.clone()
        };

        // Get today's data from daily forecast
        let today = state.daily.first();

        match &state.current {
            Some(current) => Self {
                location_name,
                temperature: Some(current.temperature),
                feels_like: Some(current.feels_like),
                wind_speed: Some(current.wind_speed),
                wind_direction: Some(current.wind_direction.clone()),
                pressure: Some(current.pressure),
                weather_code: Some(current.weather_code),
                uv_index: Some(current.uv_index),
                aqi: state.air_quality.as_ref().map(|aq| aq.aqi),
                sunrise: today.map(|d| d.sunrise),
                sunset: today.map(|d| d.sunset),
                daylight_duration: today.map(|d| d.daylight_duration),
                date: today.map(|d| d.date),
            },
            None => Self {
                location_name,
                ..Default::default()
            },
        }
    }
}

/// Get EPA color for UV index.
fn uv_color(uv: f64) -> Color {
    match uv as u8 {
        0..=2 => Color::Green,                  // Low
        3..=5 => Color::Yellow,                 // Moderate
        6..=7 => Color::Rgb(255, 165, 0),       // High (Orange)
        8..=10 => Color::Red,                   // Very High
        _ => Color::Magenta,                    // Extreme (11+)
    }
}

/// Get EPA label for UV index.
fn uv_label(uv: f64) -> &'static str {
    match uv as u8 {
        0..=2 => "Low",
        3..=5 => "Moderate",
        6..=7 => "High",
        8..=10 => "Very High",
        _ => "Extreme",
    }
}

/// Get EPA color for AQI.
fn aqi_color(aqi: u16) -> Color {
    match aqi {
        0..=50 => Color::Green,                 // Good
        51..=100 => Color::Yellow,              // Moderate
        101..=150 => Color::Rgb(255, 165, 0),   // Unhealthy for Sensitive
        151..=200 => Color::Red,                // Unhealthy
        201..=300 => Color::Magenta,            // Very Unhealthy
        _ => Color::Rgb(128, 0, 0),             // Hazardous (Maroon)
    }
}

/// Get EPA label for AQI.
fn aqi_label(aqi: u16) -> &'static str {
    match aqi {
        0..=50 => "Good",
        51..=100 => "Moderate",
        101..=150 => "Sensitive",
        151..=200 => "Unhealthy",
        201..=300 => "Very Unhealthy",
        _ => "Hazardous",
    }
}

/// Convert WMO weather code to an icon character.
///
/// WMO Weather interpretation codes (WW):
/// - 0: Clear sky
/// - 1-2: Mainly clear, partly cloudy
/// - 3: Overcast
/// - 45, 48: Fog
/// - 51-57: Drizzle
/// - 61-67: Rain
/// - 71-77: Snow
/// - 80-82: Rain showers
/// - 85-86: Snow showers
/// - 95-99: Thunderstorm
pub fn weather_code_to_icon(code: u8) -> &'static str {
    match code {
        0 => "\u{2600}\u{FE0F}",           // ☀️ Clear sky
        1 => "\u{1F324}\u{FE0F}",          // 🌤️ Mainly clear
        2 => "\u{26C5}",                   // ⛅ Partly cloudy
        3 => "\u{2601}\u{FE0F}",           // ☁️ Overcast
        45 | 48 => "\u{1F32B}\u{FE0F}",    // 🌫️ Fog
        51..=57 => "\u{1F327}\u{FE0F}",    // 🌧️ Drizzle
        61..=67 => "\u{1F327}\u{FE0F}",    // 🌧️ Rain
        71..=77 => "\u{1F328}\u{FE0F}",    // 🌨️ Snow
        80..=82 => "\u{1F327}\u{FE0F}",    // 🌧️ Rain showers
        85..=86 => "\u{1F328}\u{FE0F}",    // 🌨️ Snow showers
        95..=99 => "\u{26C8}\u{FE0F}",     // ⛈️ Thunderstorm
        _ => "\u{2753}",                   // ❓ Unknown
    }
}

/// Render the header widget.
pub fn render(data: &HeaderData, theme: &Theme, frame: &mut Frame, area: Rect) {
    let icon = data
        .weather_code
        .map(weather_code_to_icon)
        .unwrap_or("");

    let temp_str = data.temperature.map_or_else(
        || "--\u{00B0}".to_string(),
        |t| format!("{}\u{00B0}F", t as i32),
    );

    let mut header_text = vec![
        Line::from(vec![
            Span::styled(
                data.location_name.to_uppercase(),
                Style::default().fg(theme.foreground),
            ),
            Span::raw("  "),
            Span::raw(icon),
            Span::raw(" "),
            Span::styled(temp_str, Style::default().fg(theme.accent)),
        ]),
        Line::from(build_conditions_line(data)),
    ];

    // Add astronomical data line if available
    if let Some(astro_line) = build_astro_line(data) {
        header_text.push(Line::from(astro_line));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Current Conditions ")
        .border_style(Style::default().fg(theme.muted));

    let paragraph = Paragraph::new(header_text).block(block);
    frame.render_widget(paragraph, area);
}

/// Build the astronomical data line (sunrise/sunset, daylight, moon phase).
fn build_astro_line(data: &HeaderData) -> Option<Vec<Span<'static>>> {
    let sunrise = data.sunrise?;
    let sunset = data.sunset?;
    let daylight = data.daylight_duration?;
    let date = data.date?;

    let sunrise_str = format!("{}", sunrise.format("%l:%M%P"));
    let sunset_str = format!("{}", sunset.format("%l:%M%P"));
    let daylight_str = format_daylight_duration(daylight);
    let phase = moon_phase(date);

    Some(vec![
        Span::raw("\u{2600}\u{FE0F} "),  // Sun emoji
        Span::raw(sunrise_str.trim().to_string()),
        Span::raw(" - "),
        Span::raw(sunset_str.trim().to_string()),
        Span::raw(format!(" ({}) | ", daylight_str)),
        Span::raw(phase.icon()),
        Span::raw(format!(" {}", phase.name())),
    ])
}

/// Build the secondary conditions line.
fn build_conditions_line(data: &HeaderData) -> Vec<Span<'static>> {
    if data.temperature.is_none() {
        return vec![Span::raw("Loading...")];
    }

    let feels_like = data.feels_like.map_or(0, |f| f as i32);
    let wind_speed = data.wind_speed.map_or(0, |w| w as i32);
    let wind_dir = data.wind_direction.as_deref().unwrap_or("--");
    let pressure = data.pressure.unwrap_or(0.0);

    let mut spans = vec![Span::raw(format!(
        "Feels like {}\u{00B0}F | Wind {} mph {} | {:.2} in",
        feels_like, wind_speed, wind_dir, pressure
    ))];

    // Add UV index if available
    if let Some(uv) = data.uv_index {
        spans.push(Span::raw(" | UV "));
        spans.push(Span::styled(
            format!("{:.0} {}", uv, uv_label(uv)),
            Style::default().fg(uv_color(uv)),
        ));
    }

    // Add AQI if available
    if let Some(aqi) = data.aqi {
        spans.push(Span::raw(" | AQI "));
        spans.push(Span::styled(
            format!("{} {}", aqi, aqi_label(aqi)),
            Style::default().fg(aqi_color(aqi)),
        ));
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    fn render_header_to_string(data: &HeaderData, theme: &Theme) -> String {
        let backend = TestBackend::new(80, 7);
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
    fn test_header_renders_current_conditions() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let data = HeaderData {
            location_name: "New York, NY".to_string(),
            temperature: Some(72.0),
            feels_like: Some(75.0),
            wind_speed: Some(10.0),
            wind_direction: Some("SW".to_string()),
            pressure: Some(30.12),
            weather_code: Some(2),
            uv_index: Some(5.0),
            aqi: Some(42),
            sunrise: Some(date.and_hms_opt(6, 45, 0).unwrap()),
            sunset: Some(date.and_hms_opt(17, 32, 0).unwrap()),
            daylight_duration: Some(38820.0), // 10h 47m
            date: Some(date),
        };
        let theme = Theme::dark();
        let output = render_header_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_header_no_data() {
        let data = HeaderData {
            location_name: "No location selected".to_string(),
            ..Default::default()
        };
        let theme = Theme::dark();
        let output = render_header_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_weather_code_to_icon() {
        assert_eq!(weather_code_to_icon(0), "\u{2600}\u{FE0F}");
        assert_eq!(weather_code_to_icon(3), "\u{2601}\u{FE0F}");
        assert_eq!(weather_code_to_icon(63), "\u{1F327}\u{FE0F}");
        assert_eq!(weather_code_to_icon(95), "\u{26C8}\u{FE0F}");
        assert_eq!(weather_code_to_icon(255), "\u{2753}");
    }

    #[test]
    fn test_uv_color() {
        assert_eq!(uv_color(1.0), Color::Green);
        assert_eq!(uv_color(4.0), Color::Yellow);
        assert_eq!(uv_color(7.0), Color::Rgb(255, 165, 0));
        assert_eq!(uv_color(9.0), Color::Red);
        assert_eq!(uv_color(11.0), Color::Magenta);
    }

    #[test]
    fn test_uv_label() {
        assert_eq!(uv_label(1.0), "Low");
        assert_eq!(uv_label(4.0), "Moderate");
        assert_eq!(uv_label(7.0), "High");
        assert_eq!(uv_label(9.0), "Very High");
        assert_eq!(uv_label(11.0), "Extreme");
    }

    #[test]
    fn test_aqi_color() {
        assert_eq!(aqi_color(25), Color::Green);
        assert_eq!(aqi_color(75), Color::Yellow);
        assert_eq!(aqi_color(125), Color::Rgb(255, 165, 0));
        assert_eq!(aqi_color(175), Color::Red);
        assert_eq!(aqi_color(250), Color::Magenta);
        assert_eq!(aqi_color(350), Color::Rgb(128, 0, 0));
    }

    #[test]
    fn test_aqi_label() {
        assert_eq!(aqi_label(25), "Good");
        assert_eq!(aqi_label(75), "Moderate");
        assert_eq!(aqi_label(125), "Sensitive");
        assert_eq!(aqi_label(175), "Unhealthy");
        assert_eq!(aqi_label(250), "Very Unhealthy");
        assert_eq!(aqi_label(350), "Hazardous");
    }
}
