//! Color themes for the UI.

use ratatui::style::Color;

/// Color theme for the application.
#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub warning: Color,
    pub error: Color,
    pub muted: Color,
}

impl Theme {
    /// Dark theme (default).
    pub fn dark() -> Self {
        Self {
            background: Color::Reset,
            foreground: Color::White,
            accent: Color::Yellow,
            warning: Color::Rgb(255, 165, 0),
            error: Color::Red,
            muted: Color::DarkGray,
        }
    }

    /// Light theme.
    pub fn light() -> Self {
        Self {
            background: Color::White,
            foreground: Color::Black,
            accent: Color::Blue,
            warning: Color::Rgb(255, 140, 0),
            error: Color::Red,
            muted: Color::Gray,
        }
    }

    /// Solarized theme.
    pub fn solarized() -> Self {
        Self {
            background: Color::Rgb(0, 43, 54),
            foreground: Color::Rgb(131, 148, 150),
            accent: Color::Rgb(181, 137, 0),
            warning: Color::Rgb(203, 75, 22),
            error: Color::Rgb(220, 50, 47),
            muted: Color::Rgb(88, 110, 117),
        }
    }

    /// Nord theme.
    pub fn nord() -> Self {
        Self {
            background: Color::Rgb(46, 52, 64),
            foreground: Color::Rgb(216, 222, 233),
            accent: Color::Rgb(136, 192, 208),
            warning: Color::Rgb(235, 203, 139),
            error: Color::Rgb(191, 97, 106),
            muted: Color::Rgb(76, 86, 106),
        }
    }

    /// Get a theme by name.
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "light" => Self::light(),
            "solarized" => Self::solarized(),
            "nord" => Self::nord(),
            _ => Self::dark(),
        }
    }
}
