//! Screen layout management.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

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
}
