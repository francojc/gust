//! Screen layout management.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::theme::Theme;

/// Minimum terminal width for proper display.
pub const MIN_WIDTH: u16 = 80;

/// Minimum terminal height for proper display.
pub const MIN_HEIGHT: u16 = 24;

/// Result of terminal size validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeValidation {
    /// Terminal size is adequate.
    Ok,
    /// Terminal is too small.
    TooSmall { width: u16, height: u16 },
}

/// Layout constraints for the main screen areas.
pub struct ScreenLayout {
    pub header: Rect,
    pub main: Rect,
    pub footer: Rect,
}

impl ScreenLayout {
    /// Create a new screen layout from the terminal size.
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(area);

        Self {
            header: chunks[0],
            main: chunks[1],
            footer: chunks[2],
        }
    }

    /// Validate that the terminal size is adequate.
    pub fn validate_size(area: Rect) -> SizeValidation {
        if area.width >= MIN_WIDTH && area.height >= MIN_HEIGHT {
            SizeValidation::Ok
        } else {
            SizeValidation::TooSmall {
                width: area.width,
                height: area.height,
            }
        }
    }
}

/// Render a warning message when the terminal is too small.
pub fn render_size_warning(frame: &mut Frame, area: Rect, theme: &Theme) {
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Terminal too small",
            Style::default().fg(theme.warning),
        )),
        Line::from(""),
        Line::from(format!(
            "Current: {}x{}",
            area.width, area.height
        )),
        Line::from(format!(
            "Required: {}x{}",
            MIN_WIDTH, MIN_HEIGHT
        )),
        Line::from(""),
        Line::from("Please resize your terminal."),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Warning ")
        .border_style(Style::default().fg(theme.warning));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}
