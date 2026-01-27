//! Application state and Elm Architecture implementation.
//!
//! This module implements The Elm Architecture (TEA) pattern:
//! Model (AppState) → Message (Events) → Update (State Transitions) → View (Rendering)

#![allow(dead_code)]

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::api::GeocodingResult;
use crate::config::AppConfig;
use crate::ui::{
    footer::{self, FooterData},
    graphs::{
        alerts::{self, AlertsData},
        humidity::{self, HumidityData},
        precipitation::{self, PrecipitationData},
        temperature::{self, TemperatureData},
    },
    header::{self, HeaderData},
    layout::{render_size_warning, ScreenLayout, SizeValidation},
    theme::Theme,
};

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
    pub weather_code: u8,
}

/// Hourly forecast data point.
#[derive(Debug, Clone)]
pub struct HourlyForecast {
    pub time: NaiveDateTime,
    pub temperature: f64,
    pub precipitation_probability: u8,
    pub humidity: u8,
    pub wind_speed: f64,
}

/// Daily forecast data point.
#[derive(Debug, Clone)]
pub struct DailyForecast {
    pub date: NaiveDate,
    pub temp_high: f64,
    pub temp_low: f64,
    pub precipitation_sum: f64,
    pub sunrise: NaiveDateTime,
    pub sunset: NaiveDateTime,
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
#[derive(Debug, Clone, Default)]
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
    pub fetch_in_progress: bool,
    pub pending_search: Option<String>,
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
    /// Location search result received.
    LocationReceived(Result<GeocodingResult, String>),
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

impl AppState {
    /// Create a new application state with the given configuration.
    ///
    /// If no default location is configured, starts in search mode.
    pub fn new(config: AppConfig) -> Self {
        let has_default_location = !config.location.default.is_empty();
        let mut state = Self {
            config: config.clone(),
            ..Default::default()
        };

        if has_default_location {
            // Trigger search for default location on startup
            state.pending_search = Some(config.location.default.clone());
        } else {
            // Start in search mode if no default location
            state.input_mode = InputMode::Search;
        }

        state
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
            Message::LocationReceived(result) => self.handle_location_received(result),
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
                    if !self.input_buffer.is_empty() {
                        self.pending_search = Some(self.input_buffer.clone());
                    }
                    self.input_mode = InputMode::Normal;
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

    fn handle_location_received(&mut self, result: Result<GeocodingResult, String>) {
        self.fetch_in_progress = false;
        match result {
            Ok(loc) => {
                self.location = Location {
                    name: loc.display_name(),
                    latitude: loc.latitude,
                    longitude: loc.longitude,
                };
                self.loading = true;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e);
            }
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
        let area = frame.area();
        let theme = Theme::from_name(&self.config.display.theme);

        // Check terminal size
        if let SizeValidation::TooSmall { .. } = ScreenLayout::validate_size(area) {
            render_size_warning(frame, area, &theme);
            return;
        }

        let layout = ScreenLayout::new(area);

        self.render_header(frame, layout.header, &theme);
        self.render_main(frame, layout.main, &theme);
        self.render_footer(frame, layout.footer, &theme);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let data = HeaderData::from_state(self);
        header::render(&data, theme, frame, area);
    }

    fn render_main(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        use ratatui::layout::{Constraint, Direction, Layout};

        // Split area: tabs row (3 lines) + content area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Render tab bar
        self.render_tabs(frame, chunks[0], theme);

        // Render selected content
        match self.selected_tab {
            Tab::Temperature => {
                let data = TemperatureData::from_hourly(&self.hourly);
                temperature::render(&data, theme, frame, chunks[1]);
            }
            Tab::Precipitation => {
                let data = PrecipitationData::from_hourly(&self.hourly);
                precipitation::render(&data, theme, frame, chunks[1]);
            }
            Tab::Humidity => {
                let data = HumidityData::from_hourly(&self.hourly);
                humidity::render(&data, theme, frame, chunks[1]);
            }
            Tab::Alerts => {
                let data = AlertsData::from_state(self);
                alerts::render(&data, theme, frame, chunks[1]);
            }
        }
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
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
                    Span::styled(format!("[{}]", t), Style::default().fg(theme.accent))
                } else {
                    Span::styled(format!(" {} ", t), Style::default().fg(theme.muted))
                }
            })
            .collect();

        let text = vec![Line::from(tabs)];

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Forecast ")
            .border_style(Style::default().fg(theme.muted));

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let data = FooterData::from_state(self);
        footer::render(&data, theme, frame, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{convert, mock};
    use crossterm::event::KeyCode;

    #[test]
    fn test_default_state() {
        let state = AppState::default();
        assert!(!state.should_quit);
        assert_eq!(state.selected_tab, Tab::Temperature);
        assert_eq!(state.input_mode, InputMode::Normal);
        assert!(!state.fetch_in_progress);
        assert!(state.pending_search.is_none());
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

    #[test]
    fn test_search_submit_sets_pending() {
        let mut state = AppState::default();

        // Enter search mode
        let key = KeyEvent::new(KeyCode::Char('/'), crossterm::event::KeyModifiers::NONE);
        state.handle_input(key);

        // Type query
        for c in "New York".chars() {
            let key = KeyEvent::new(KeyCode::Char(c), crossterm::event::KeyModifiers::NONE);
            state.handle_input(key);
        }

        // Submit search
        let key = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE);
        state.handle_input(key);

        assert_eq!(state.input_mode, InputMode::Normal);
        assert_eq!(state.pending_search, Some("New York".to_string()));
        assert!(state.input_buffer.is_empty());
    }

    #[test]
    fn test_empty_search_not_submitted() {
        let mut state = AppState::default();

        // Enter search mode
        let key = KeyEvent::new(KeyCode::Char('/'), crossterm::event::KeyModifiers::NONE);
        state.handle_input(key);

        // Submit empty search
        let key = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE);
        state.handle_input(key);

        assert_eq!(state.input_mode, InputMode::Normal);
        assert!(state.pending_search.is_none());
    }

    #[test]
    fn test_weather_received_success() {
        let mut state = AppState::default();
        state.loading = true;

        let mock_resp = mock::mock_forecast_response();
        let data = convert::forecast_to_weather_data(mock_resp);

        state.update(Message::WeatherReceived(Ok(data)));

        assert!(!state.loading);
        assert!(state.error.is_none());
        assert!(state.current.is_some());
        assert_eq!(state.current.as_ref().unwrap().temperature, 42.0);
        assert!(!state.hourly.is_empty());
        assert!(!state.daily.is_empty());
        assert!(state.last_updated.is_some());
    }

    #[test]
    fn test_weather_received_error() {
        let mut state = AppState::default();
        state.loading = true;

        state.update(Message::WeatherReceived(Err(
            "Network error".to_string(),
        )));

        assert!(!state.loading);
        assert!(state.error.is_some());
        assert_eq!(state.error.as_ref().unwrap(), "Network error");
    }

    #[test]
    fn test_location_received_success() {
        let mut state = AppState::default();
        state.fetch_in_progress = true;

        let loc = mock::mock_geocoding_result();
        state.update(Message::LocationReceived(Ok(loc)));

        assert!(!state.fetch_in_progress);
        assert!(state.loading);
        assert!(state.error.is_none());
        assert_eq!(state.location.name, "New York, New York, United States");
        assert_eq!(state.location.latitude, 40.7128);
        assert_eq!(state.location.longitude, -74.0060);
    }

    #[test]
    fn test_location_received_error() {
        let mut state = AppState::default();
        state.fetch_in_progress = true;

        state.update(Message::LocationReceived(Err(
            "No locations found".to_string(),
        )));

        assert!(!state.fetch_in_progress);
        assert!(!state.loading);
        assert!(state.error.is_some());
        assert_eq!(state.error.as_ref().unwrap(), "No locations found");
    }

    #[test]
    fn test_refresh_triggers_loading() {
        let mut state = AppState::default();
        assert!(!state.loading);

        state.update(Message::Refresh);

        assert!(state.loading);
    }

    #[test]
    fn test_new_with_default_location() {
        let mut config = AppConfig::default();
        config.location.default = "Chicago".to_string();

        let state = AppState::new(config);

        assert_eq!(state.pending_search, Some("Chicago".to_string()));
        assert_eq!(state.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_new_without_default_location() {
        let config = AppConfig::default();
        let state = AppState::new(config);

        assert!(state.pending_search.is_none());
        assert_eq!(state.input_mode, InputMode::Search);
    }
}
