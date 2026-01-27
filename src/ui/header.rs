//! Header widget for current conditions.

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::theme::Theme;
use crate::app::AppState;

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
}

impl HeaderData {
    /// Create header data from the application state.
    pub fn from_state(state: &AppState) -> Self {
        let location_name = if state.location.name.is_empty() {
            "No location selected".to_string()
        } else {
            state.location.name.clone()
        };

        match &state.current {
            Some(current) => Self {
                location_name,
                temperature: Some(current.temperature),
                feels_like: Some(current.feels_like),
                wind_speed: Some(current.wind_speed),
                wind_direction: Some(current.wind_direction.clone()),
                pressure: Some(current.pressure),
                weather_code: Some(current.weather_code),
            },
            None => Self {
                location_name,
                ..Default::default()
            },
        }
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

    let header_text = vec![
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

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Current Conditions ")
        .border_style(Style::default().fg(theme.muted));

    let paragraph = Paragraph::new(header_text).block(block);
    frame.render_widget(paragraph, area);
}

/// Build the secondary conditions line.
fn build_conditions_line(data: &HeaderData) -> Span<'static> {
    if data.temperature.is_none() {
        return Span::raw("Loading...");
    }

    let feels_like = data.feels_like.map_or(0, |f| f as i32);
    let wind_speed = data.wind_speed.map_or(0, |w| w as i32);
    let wind_dir = data.wind_direction.as_deref().unwrap_or("--");
    let pressure = data.pressure.unwrap_or(0.0);

    Span::raw(format!(
        "Feels like {}\u{00B0}F | Wind {} mph {} | {:.2} in",
        feels_like, wind_speed, wind_dir, pressure
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    fn render_header_to_string(data: &HeaderData, theme: &Theme) -> String {
        let backend = TestBackend::new(80, 6);
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
        let data = HeaderData {
            location_name: "New York, NY".to_string(),
            temperature: Some(72.0),
            feels_like: Some(75.0),
            wind_speed: Some(10.0),
            wind_direction: Some("SW".to_string()),
            pressure: Some(30.12),
            weather_code: Some(2),
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
}
