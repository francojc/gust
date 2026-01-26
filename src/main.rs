//! gust - A terminal-based weather dashboard.

mod app;
mod config;
mod error;

mod api;
mod cache;
mod ui;

use std::io;
use std::time::Duration;

use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use tokio::sync::mpsc;
use tokio::time::sleep;

use api::{convert, GeocodingClient, GeocodingResult, Units, WeatherClient};
use app::{AppState, Message, WeatherData};
use cache::Cache;
use config::AppConfig;

/// Result type for fetch operations sent through channels.
enum FetchResult {
    Weather(Result<WeatherData, String>),
    Location(Result<GeocodingResult, String>),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize error handling
    error::init()?;

    // Load configuration
    let config = AppConfig::load()?;

    // Initialize terminal
    let mut terminal = setup_terminal()?;

    // Run the application
    let result = run(&mut terminal, config).await;

    // Restore terminal state
    restore_terminal(&mut terminal)?;

    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

async fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, config: AppConfig) -> Result<()> {
    let mut state = AppState::new(config.clone());
    let (tx, mut rx) = mpsc::channel::<FetchResult>(32);

    loop {
        // Render
        terminal.draw(|f| state.view(f))?;

        // Check if we need to spawn a location search
        if let Some(query) = state.pending_search.take() {
            if !state.fetch_in_progress {
                state.fetch_in_progress = true;
                let tx = tx.clone();
                tokio::spawn(async move {
                    let result = fetch_location(&query).await;
                    let _ = tx.send(FetchResult::Location(result)).await;
                });
            }
        }

        // Check if we need to spawn a weather fetch
        if state.loading && !state.fetch_in_progress {
            state.fetch_in_progress = true;
            let tx = tx.clone();
            let lat = state.location.latitude;
            let lon = state.location.longitude;
            let cfg = config.clone();
            tokio::spawn(async move {
                let result = fetch_weather(lat, lon, &cfg).await;
                let _ = tx.send(FetchResult::Weather(result)).await;
            });
        }

        // Handle events with async select
        tokio::select! {
            _ = sleep(Duration::from_millis(50)) => {
                if event::poll(Duration::ZERO)? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            state.update(Message::Input(key));
                        }
                    }
                }
            }
            Some(result) = rx.recv() => {
                match result {
                    FetchResult::Weather(data) => {
                        state.fetch_in_progress = false;
                        state.update(Message::WeatherReceived(data));
                    }
                    FetchResult::Location(loc) => {
                        state.update(Message::LocationReceived(loc));
                    }
                }
            }
        }

        // Tick check for auto-refresh
        state.update(Message::Tick);

        if state.should_quit {
            break;
        }
    }

    Ok(())
}

/// Fetch weather data with caching.
async fn fetch_weather(lat: f64, lon: f64, config: &AppConfig) -> Result<WeatherData, String> {
    // Try to create cache (non-fatal if it fails)
    let cache = Cache::new(config.behavior.cache_duration).ok();

    // Try cache first
    if let Some(ref c) = cache {
        if let Some(cached) = c.get::<api::ForecastResponse>(lat, lon) {
            return Ok(convert::forecast_to_weather_data(cached));
        }
    }

    // Fetch from API
    let units = match config.display.units.as_str() {
        "metric" => Units::Metric,
        _ => Units::Imperial,
    };
    let client = WeatherClient::new(units);

    match client.fetch_forecast(lat, lon).await {
        Ok(resp) => {
            // Cache the response
            if let Some(ref c) = cache {
                let _ = c.set(lat, lon, &resp);
            }
            Ok(convert::forecast_to_weather_data(resp))
        }
        Err(e) => {
            // Try stale cache as fallback
            if let Some(ref c) = cache {
                if let Some(stale) = c.get_stale::<api::ForecastResponse>(lat, lon) {
                    return Ok(convert::forecast_to_weather_data(stale));
                }
            }
            Err(e.to_string())
        }
    }
}

/// Fetch location by name using geocoding API.
async fn fetch_location(query: &str) -> Result<GeocodingResult, String> {
    let client = GeocodingClient::new();
    match client.search(query).await {
        Ok(results) => {
            if let Some(first) = results.into_iter().next() {
                Ok(first)
            } else {
                Err(format!(
                    "No locations found for '{}'. Try using a hyphen for hyphenated \
                     city names (e.g., Winston-Salem) or search by ZIP code.",
                    query
                ))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}
