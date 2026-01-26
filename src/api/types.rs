//! API response types for Open-Meteo.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Response from the Open-Meteo forecast API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForecastResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub current: Option<CurrentResponse>,
    pub hourly: Option<HourlyResponse>,
    pub daily: Option<DailyResponse>,
}

/// Current weather data from API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CurrentResponse {
    pub time: NaiveDateTime,
    pub temperature_2m: f64,
    pub apparent_temperature: f64,
    pub relative_humidity_2m: u8,
    pub wind_speed_10m: f64,
    pub wind_direction_10m: f64,
    pub surface_pressure: f64,
    pub weather_code: u8,
}

/// Hourly forecast data from API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HourlyResponse {
    pub time: Vec<NaiveDateTime>,
    pub temperature_2m: Vec<f64>,
    pub precipitation_probability: Vec<u8>,
    pub relative_humidity_2m: Vec<u8>,
    pub wind_speed_10m: Vec<f64>,
}

/// Daily forecast data from API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DailyResponse {
    pub time: Vec<NaiveDateTime>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub precipitation_sum: Vec<f64>,
    pub sunrise: Vec<NaiveDateTime>,
    pub sunset: Vec<NaiveDateTime>,
}

/// Response from the Open-Meteo geocoding API.
#[derive(Debug, Clone, Deserialize)]
pub struct GeocodingResponse {
    pub results: Option<Vec<GeocodingResult>>,
}

/// Single geocoding result.
#[derive(Debug, Clone, Deserialize)]
pub struct GeocodingResult {
    pub id: u64,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country: Option<String>,
    pub admin1: Option<String>,
}

impl GeocodingResult {
    /// Format the location as a display string.
    pub fn display_name(&self) -> String {
        let mut parts = vec![self.name.clone()];
        if let Some(ref admin) = self.admin1 {
            parts.push(admin.clone());
        }
        if let Some(ref country) = self.country {
            parts.push(country.clone());
        }
        parts.join(", ")
    }
}
