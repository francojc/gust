//! Weather alerts display.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{AlertSeverity, AppState, WeatherAlert};
use crate::ui::theme::Theme;

/// Data structure for alerts display.
#[derive(Debug, Clone)]
pub struct AlertsData {
    /// Weather alerts.
    pub alerts: Vec<WeatherAlert>,
}

impl AlertsData {
    /// Create alerts data from application state.
    pub fn from_state(state: &AppState) -> Self {
        Self {
            alerts: state.alerts.clone(),
        }
    }

    /// Create empty alerts data.
    pub fn empty() -> Self {
        Self { alerts: vec![] }
    }
}

/// Get the color for an alert severity.
fn severity_color(severity: AlertSeverity) -> Color {
    match severity {
        AlertSeverity::Warning => Color::Red,
        AlertSeverity::Watch => Color::Rgb(255, 165, 0), // Orange
        AlertSeverity::Advisory => Color::Yellow,
    }
}

/// Get the label for an alert severity.
fn severity_label(severity: AlertSeverity) -> &'static str {
    match severity {
        AlertSeverity::Warning => "WARNING",
        AlertSeverity::Watch => "WATCH",
        AlertSeverity::Advisory => "ADVISORY",
    }
}

/// Render the alerts display.
pub fn render(data: &AlertsData, theme: &Theme, frame: &mut Frame, area: Rect) {
    if data.alerts.is_empty() {
        render_empty(theme, frame, area);
        return;
    }

    let items: Vec<ListItem> = data
        .alerts
        .iter()
        .map(|alert| {
            let color = severity_color(alert.severity);
            let label = severity_label(alert.severity);

            let header = Line::from(vec![
                Span::styled(
                    format!("[{}] ", label),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(&alert.title, Style::default().fg(theme.foreground)),
            ]);

            let description = Line::from(Span::styled(
                truncate_text(&alert.description, 70),
                Style::default().fg(theme.muted),
            ));

            let expires = Line::from(Span::styled(
                format!("Expires: {}", alert.expires.format("%b %d %H:%M")),
                Style::default().fg(theme.muted),
            ));

            ListItem::new(vec![header, description, expires, Line::from("")])
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Weather Alerts ({}) ", data.alerts.len()))
            .border_style(Style::default().fg(theme.muted)),
    );

    frame.render_widget(list, area);
}

/// Render an empty state message.
fn render_empty(theme: &Theme, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Weather Alerts ")
        .border_style(Style::default().fg(theme.muted));

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "No active alerts",
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Weather conditions are normal for your area.",
            Style::default().fg(theme.muted),
        )),
    ];

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

/// Truncate text to a maximum length with ellipsis.
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    fn create_mock_alerts() -> Vec<WeatherAlert> {
        vec![
            WeatherAlert {
                severity: AlertSeverity::Warning,
                title: "Winter Storm Warning".to_string(),
                description: "Heavy snow expected. 8-12 inches accumulation possible.".to_string(),
                expires: Utc.with_ymd_and_hms(2024, 1, 2, 18, 0, 0).unwrap(),
            },
            WeatherAlert {
                severity: AlertSeverity::Watch,
                title: "Wind Advisory".to_string(),
                description: "Gusty winds up to 45 mph expected.".to_string(),
                expires: Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap(),
            },
            WeatherAlert {
                severity: AlertSeverity::Advisory,
                title: "Frost Advisory".to_string(),
                description: "Sub-freezing temperatures expected overnight.".to_string(),
                expires: Utc.with_ymd_and_hms(2024, 1, 2, 8, 0, 0).unwrap(),
            },
        ]
    }

    fn render_to_string(data: &AlertsData, theme: &Theme) -> String {
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
    fn test_alerts_renders_with_alerts() {
        let alerts = create_mock_alerts();
        let data = AlertsData { alerts };
        let theme = Theme::dark();
        let output = render_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_alerts_renders_empty() {
        let data = AlertsData::empty();
        let theme = Theme::dark();
        let output = render_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_severity_color() {
        assert_eq!(severity_color(AlertSeverity::Warning), Color::Red);
        assert_eq!(severity_color(AlertSeverity::Watch), Color::Rgb(255, 165, 0));
        assert_eq!(severity_color(AlertSeverity::Advisory), Color::Yellow);
    }

    #[test]
    fn test_severity_label() {
        assert_eq!(severity_label(AlertSeverity::Warning), "WARNING");
        assert_eq!(severity_label(AlertSeverity::Watch), "WATCH");
        assert_eq!(severity_label(AlertSeverity::Advisory), "ADVISORY");
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("short", 10), "short");
        assert_eq!(truncate_text("this is a long text", 10), "this is...");
    }
}
