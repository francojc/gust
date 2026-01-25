//! Application state and Elm Architecture implementation.
//!
//! This module implements The Elm Architecture (TEA) pattern:
//! Model (AppState) → Message (Events) → Update (State Transitions) → View (Rendering)

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::config::AppConfig;

/// Location data.
#[derive(Debug, Clone, Default)]
pub struct Location {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

/// Current weather conditions.
#[derive(Debug, Clone, Default)]
pub struct CurrentWeather {
    pub temperature: f64,
    pub feels_like: f64,
    pub humidity: u8,
    pub wind_speed: f64,
    pub wind_direction: String,
    pub pressure: f64,
    pub description: String,
}

/// Hourly forecast data point.
#[derive(Debug, Clone)]
pub struct HourlyForecast {
    pub time: DateTime<Utc>,
    pub temperature: f64,
    pub precipitation_probability: u8,
    pub humidity: u8,
    pub wind_speed: f64,
}

/// Daily forecast data point.
#[derive(Debug, Clone)]
pub struct DailyForecast {
    pub date: DateTime<Utc>,
    pub temp_high: f64,
    pub temp_low: f64,
    pub precipitation_sum: f64,
    pub sunrise: DateTime<Utc>,
    pub sunset: DateTime<Utc>,
}

/// Weather alert.
#[derive(Debug, Clone)]
pub struct WeatherAlert {
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub expires: DateTime<Utc>,
}

/// Alert severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Advisory,
    Watch,
    Warning,
}

/// Active tab in the main view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Temperature,
    Precipitation,
    Humidity,
    Alerts,
}

/// Input mode for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Search,
}

/// Main application state.
#[derive(Debug, Clone)]
pub struct AppState {
    pub location: Location,
    pub current: Option<CurrentWeather>,
    pub hourly: Vec<HourlyForecast>,
    pub daily: Vec<DailyForecast>,
    pub alerts: Vec<WeatherAlert>,
    pub selected_tab: Tab,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub loading: bool,
    pub error: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
    pub config: AppConfig,
    pub should_quit: bool,
}

/// Messages that can be sent to update the application state.
#[derive(Debug)]
pub enum Message {
    /// Keyboard input event.
    Input(KeyEvent),
    /// Weather data received from API.
    WeatherReceived(Result<WeatherData, String>),
    /// Weather alerts received from API.
    AlertsReceived(Result<Vec<WeatherAlert>, String>),
    /// Timer tick for auto-refresh.
    Tick,
    /// Manual refresh requested.
    Refresh,
    /// Quit the application.
    Quit,
}

/// Combined weather data from API response.
#[derive(Debug, Clone)]
pub struct WeatherData {
    pub current: CurrentWeather,
    pub hourly: Vec<HourlyForecast>,
    pub daily: Vec<DailyForecast>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            location: Location::default(),
            current: None,
            hourly: Vec::new(),
            daily: Vec::new(),
            alerts: Vec::new(),
            selected_tab: Tab::default(),
            input_mode: InputMode::default(),
            input_buffer: String::new(),
            loading: false,
            error: None,
            last_updated: None,
            config: AppConfig::default(),
            should_quit: false,
        }
    }
}

impl AppState {
    /// Create a new application state with the given configuration.
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Update the application state based on a message.
    ///
    /// This is the core of the Elm Architecture - a pure function that takes
    /// the current state and a message, and returns the new state.
    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::Input(key) => self.handle_input(key),
            Message::WeatherReceived(result) => self.handle_weather_received(result),
            Message::AlertsReceived(result) => self.handle_alerts_received(result),
            Message::Tick => self.handle_tick(),
            Message::Refresh => self.handle_refresh(),
            Message::Quit => self.should_quit = true,
        }
    }

    fn handle_input(&mut self, key: KeyEvent) {
        use crossterm::event::KeyCode;

        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char('r') => self.loading = true,
                KeyCode::Char('/') => {
                    self.input_mode = InputMode::Search;
                    self.input_buffer.clear();
                }
                KeyCode::Char('1') => self.selected_tab = Tab::Temperature,
                KeyCode::Char('2') => self.selected_tab = Tab::Precipitation,
                KeyCode::Char('3') => self.selected_tab = Tab::Humidity,
                KeyCode::Char('4') => self.selected_tab = Tab::Alerts,
                KeyCode::Tab => self.cycle_tab(),
                _ => {}
            },
            InputMode::Search => match key.code {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input_buffer.clear();
                }
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    // TODO: Trigger location search
                    self.input_buffer.clear();
                }
                KeyCode::Char(c) => self.input_buffer.push(c),
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                _ => {}
            },
        }
    }

    fn handle_weather_received(&mut self, result: Result<WeatherData, String>) {
        self.loading = false;
        match result {
            Ok(data) => {
                self.current = Some(data.current);
                self.hourly = data.hourly;
                self.daily = data.daily;
                self.last_updated = Some(Utc::now());
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e);
            }
        }
    }

    fn handle_alerts_received(&mut self, result: Result<Vec<WeatherAlert>, String>) {
        match result {
            Ok(alerts) => self.alerts = alerts,
            Err(e) => self.error = Some(e),
        }
    }

    fn handle_tick(&mut self) {
        // Check if auto-refresh is needed
        if let Some(last) = self.last_updated {
            let elapsed = Utc::now().signed_duration_since(last);
            if elapsed.num_seconds() as u64 >= self.config.behavior.refresh_interval {
                self.loading = true;
            }
        }
    }

    fn handle_refresh(&mut self) {
        self.loading = true;
    }

    fn cycle_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            Tab::Temperature => Tab::Precipitation,
            Tab::Precipitation => Tab::Humidity,
            Tab::Humidity => Tab::Alerts,
            Tab::Alerts => Tab::Temperature,
        };
    }

    /// Render the application UI.
    pub fn view(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(frame.area());

        self.render_header(frame, chunks[0]);
        self.render_main(frame, chunks[1]);
        self.render_footer(frame, chunks[2]);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let location_name = if self.location.name.is_empty() {
            "No location selected"
        } else {
            &self.location.name
        };

        let temp_str = self.current.as_ref().map_or_else(
            || "--°".to_string(),
            |c| format!("{}°F", c.temperature as i32),
        );

        let header_text = vec![
            Line::from(vec![
                Span::styled(
                    location_name.to_uppercase(),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  "),
                Span::styled(temp_str, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(self.current.as_ref().map_or_else(
                || Span::raw("Loading..."),
                |c| {
                    Span::raw(format!(
                        "Feels like {}°F | Wind {} mph {} | {} in",
                        c.feels_like as i32, c.wind_speed as i32, c.wind_direction, c.pressure
                    ))
                },
            )),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Current Conditions ");

        let paragraph = Paragraph::new(header_text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_main(&self, frame: &mut Frame, area: Rect) {
        let tab_titles = ["1:Temp", "2:Precip", "3:Humidity", "4:Alerts"];
        let selected = match self.selected_tab {
            Tab::Temperature => 0,
            Tab::Precipitation => 1,
            Tab::Humidity => 2,
            Tab::Alerts => 3,
        };

        let tabs: Vec<Span> = tab_titles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                if i == selected {
                    Span::styled(format!("[{}]", t), Style::default().fg(Color::Yellow))
                } else {
                    Span::styled(format!(" {} ", t), Style::default().fg(Color::DarkGray))
                }
            })
            .collect();

        let content = match self.selected_tab {
            Tab::Temperature => "Temperature graph will be rendered here",
            Tab::Precipitation => "Precipitation graph will be rendered here",
            Tab::Humidity => "Humidity graph will be rendered here",
            Tab::Alerts => "Weather alerts will be displayed here",
        };

        let text = vec![
            Line::from(tabs),
            Line::from(""),
            Line::from(content),
        ];

        let block = Block::default().borders(Borders::ALL).title(" Forecast ");

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let input_line = match self.input_mode {
            InputMode::Normal => Line::from(Span::raw("Press '/' to search, 'q' to quit")),
            InputMode::Search => Line::from(vec![
                Span::raw("> "),
                Span::styled(&self.input_buffer, Style::default().fg(Color::Yellow)),
                Span::styled("_", Style::default().fg(Color::White)),
            ]),
        };

        let status_line = if self.loading {
            Line::from(Span::styled("Loading...", Style::default().fg(Color::Cyan)))
        } else if let Some(ref err) = self.error {
            Line::from(Span::styled(err.as_str(), Style::default().fg(Color::Red)))
        } else {
            let updated = self.last_updated.map_or_else(
                || "Never".to_string(),
                |t| t.format("%I:%M %p").to_string(),
            );
            Line::from(format!("Updated: {} | q:quit r:refresh /:search", updated))
        };

        let text = vec![input_line, Line::from(""), status_line];

        let block = Block::default().borders(Borders::ALL).title(" Status ");

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    #[test]
    fn test_default_state() {
        let state = AppState::default();
        assert!(!state.should_quit);
        assert_eq!(state.selected_tab, Tab::Temperature);
        assert_eq!(state.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_quit_message() {
        let mut state = AppState::default();
        state.update(Message::Quit);
        assert!(state.should_quit);
    }

    #[test]
    fn test_tab_cycling() {
        let mut state = AppState::default();
        assert_eq!(state.selected_tab, Tab::Temperature);

        state.cycle_tab();
        assert_eq!(state.selected_tab, Tab::Precipitation);

        state.cycle_tab();
        assert_eq!(state.selected_tab, Tab::Humidity);

        state.cycle_tab();
        assert_eq!(state.selected_tab, Tab::Alerts);

        state.cycle_tab();
        assert_eq!(state.selected_tab, Tab::Temperature);
    }

    #[test]
    fn test_search_mode_toggle() {
        let mut state = AppState::default();
        assert_eq!(state.input_mode, InputMode::Normal);

        // Enter search mode
        let key = KeyEvent::new(KeyCode::Char('/'), crossterm::event::KeyModifiers::NONE);
        state.handle_input(key);
        assert_eq!(state.input_mode, InputMode::Search);

        // Exit with Escape
        let key = KeyEvent::new(KeyCode::Esc, crossterm::event::KeyModifiers::NONE);
        state.handle_input(key);
        assert_eq!(state.input_mode, InputMode::Normal);
    }
}
