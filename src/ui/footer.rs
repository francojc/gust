//! Footer widget for status bar and input.

use chrono::{DateTime, Utc};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::graphs::format_local_time;
use super::theme::Theme;
use crate::app::{AppState, InputMode};

/// Data needed to render the footer.
#[derive(Debug, Clone, Default)]
pub struct FooterData {
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub loading: bool,
    pub error: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
    pub time_format: String,
}

impl FooterData {
    /// Create footer data from the application state.
    pub fn from_state(state: &AppState) -> Self {
        Self {
            input_mode: state.input_mode,
            input_buffer: state.input_buffer.clone(),
            loading: state.loading,
            error: state.error.clone(),
            last_updated: state.last_updated,
            timezone: state.timezone.clone(),
            time_format: state.config.display.time_format.clone(),
        }
    }
}

/// Render the footer widget.
pub fn render(data: &FooterData, theme: &Theme, frame: &mut Frame, area: Rect) {
    let input_line = build_input_line(data, theme);
    let status_line = build_status_line(data, theme);

    let text = vec![input_line, Line::from(""), status_line];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Status ")
        .border_style(Style::default().fg(theme.muted));

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

/// Build the input line based on current mode.
fn build_input_line(data: &FooterData, theme: &Theme) -> Line<'static> {
    match data.input_mode {
        InputMode::Normal => Line::from(Span::raw("Press '/' to search, 'q' to quit")),
        InputMode::Search => Line::from(vec![
            Span::raw("> "),
            Span::styled(data.input_buffer.clone(), Style::default().fg(theme.accent)),
            Span::styled("_", Style::default().fg(theme.foreground)),
        ]),
    }
}

/// Build the status line showing loading, error, or last updated.
fn build_status_line(data: &FooterData, theme: &Theme) -> Line<'static> {
    if data.loading {
        Line::from(Span::styled(
            "Loading...".to_string(),
            Style::default().fg(theme.accent),
        ))
    } else if let Some(ref err) = data.error {
        Line::from(Span::styled(
            err.clone(),
            Style::default().fg(theme.error),
        ))
    } else {
        let use_24h = data.time_format == "24h";
        let updated = data.last_updated.map_or_else(
            || "Never".to_string(),
            |t| format_local_time(t, data.timezone.as_deref(), use_24h),
        );
        Line::from(format!(
            "Updated: {} | q:quit r:refresh /:search",
            updated
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use ratatui::{backend::TestBackend, Terminal};

    fn render_footer_to_string(data: &FooterData, theme: &Theme) -> String {
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
    fn test_footer_normal_mode() {
        let data = FooterData {
            input_mode: InputMode::Normal,
            last_updated: None,
            ..Default::default()
        };
        let theme = Theme::dark();
        let output = render_footer_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_footer_search_mode() {
        let data = FooterData {
            input_mode: InputMode::Search,
            input_buffer: "New York".to_string(),
            ..Default::default()
        };
        let theme = Theme::dark();
        let output = render_footer_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_footer_loading() {
        let data = FooterData {
            input_mode: InputMode::Normal,
            loading: true,
            ..Default::default()
        };
        let theme = Theme::dark();
        let output = render_footer_to_string(&data, &theme);
        assert_snapshot!(output);
    }

    #[test]
    fn test_footer_error() {
        let data = FooterData {
            input_mode: InputMode::Normal,
            error: Some("Network error: connection refused".to_string()),
            ..Default::default()
        };
        let theme = Theme::dark();
        let output = render_footer_to_string(&data, &theme);
        assert_snapshot!(output);
    }
}
