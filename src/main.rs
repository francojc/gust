//! gust - A terminal-based weather dashboard.

mod app;
mod astro;
mod config;
mod error;

mod api;
mod cache;
mod ui;

use std::io;
use std::time::Duration;

use clap::Parser;
use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use tokio::sync::mpsc;
use tokio::time::sleep;

use api::{convert, AlertsClient, GeocodingClient, GeocodingResult, Units, WeatherClient};
use app::{AirQuality, AppState, Message, WeatherAlert, WeatherData};
use cache::Cache;
use config::AppConfig;

/// A terminal-based weather dashboard
#[derive(Parser, Debug)]
#[command(name = "gust")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Location to search for on startup (e.g., "New York, NY")
    location: Option<String>,
}

/// Result type for fetch operations sent through channels.
enum FetchResult {
    Weather(Result<WeatherData, String>),
    Location(Result<GeocodingResult, String>),
    Alerts(Result<Vec<WeatherAlert>, String>),
    AirQuality(Result<AirQuality, String>),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments before anything else
    let cli = Cli::parse();

    // Initialize error handling
    error::init()?;

    // Load configuration
    let config = AppConfig::load()?;

    // Initialize terminal
    let mut terminal = setup_terminal()?;

    // Run the application
    let result = run(&mut terminal, config, cli.location).await;

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

async fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    config: AppConfig,
    initial_location: Option<String>,
) -> Result<()> {
    let mut state = AppState::new(config.clone());

    // If a location was provided via CLI, queue it for search
    if let Some(location) = initial_location {
        state.pending_search = Some(location);
    }

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
                    match event::read()? {
                        Event::Key(key) if key.kind == KeyEventKind::Press => {
                            state.update(Message::Input(key));
                        }
                        Event::Resize(_, _) => {
                            // Terminal resize - ratatui will redraw on next frame
                        }
                        _ => {}
                    }
                }
            }
            Some(result) = rx.recv() => {
                match result {
                    FetchResult::Weather(data) => {
                        state.fetch_in_progress = false;
                        let fetch_extra_needed = data.is_ok() && !state.location.name.is_empty();
                        state.update(Message::WeatherReceived(data));

                        // Fetch alerts and AQI after successful weather data
                        if fetch_extra_needed {
                            let tx_alerts = tx.clone();
                            let tx_aqi = tx.clone();
                            let lat = state.location.latitude;
                            let lon = state.location.longitude;

                            // Fetch NWS alerts (US only)
                            tokio::spawn(async move {
                                let result = fetch_alerts(lat, lon).await;
                                let _ = tx_alerts.send(FetchResult::Alerts(result)).await;
                            });

                            // Fetch AQI
                            tokio::spawn(async move {
                                let result = fetch_air_quality(lat, lon).await;
                                let _ = tx_aqi.send(FetchResult::AirQuality(result)).await;
                            });
                        }
                    }
                    FetchResult::Location(loc) => {
                        state.update(Message::LocationReceived(loc));
                    }
                    FetchResult::Alerts(alerts) => {
                        state.update(Message::AlertsReceived(alerts));
                    }
                    FetchResult::AirQuality(aq) => {
                        state.update(Message::AirQualityReceived(aq));
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

/// Fetch weather alerts from NWS (US only).
async fn fetch_alerts(lat: f64, lon: f64) -> Result<Vec<WeatherAlert>, String> {
    let client = AlertsClient::new();
    client.fetch(lat, lon).await
}

/// Fetch air quality data from Open-Meteo.
async fn fetch_air_quality(lat: f64, lon: f64) -> Result<AirQuality, String> {
    let client = WeatherClient::new(Units::Imperial);
    match client.fetch_air_quality(lat, lon).await {
        Ok(resp) => {
            if let Some(current) = resp.current {
                Ok(AirQuality {
                    aqi: current.us_aqi,
                    pm2_5: current.pm2_5,
                    pm10: current.pm10,
                })
            } else {
                Err("No air quality data available".to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}
