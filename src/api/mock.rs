//! Mock data for testing without network calls.

use chrono::{TimeZone, Utc};

use crate::api::types::{
    CurrentResponse, DailyResponse, ForecastResponse, GeocodingResult, HourlyResponse,
};

/// Create a mock forecast response for testing.
pub fn mock_forecast_response() -> ForecastResponse {
    ForecastResponse {
        latitude: 40.7128,
        longitude: -74.0060,
        timezone: "America/New_York".to_string(),
        current: Some(mock_current()),
        hourly: Some(mock_hourly()),
        daily: Some(mock_daily()),
    }
}

/// Create a mock current weather response.
pub fn mock_current() -> CurrentResponse {
    CurrentResponse {
        time: Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap(),
        temperature_2m: 42.0,
        apparent_temperature: 38.0,
        relative_humidity_2m: 65,
        wind_speed_10m: 12.0,
        wind_direction_10m: 225.0,
        surface_pressure: 30.12,
        weather_code: 2,
    }
}

/// Create a mock hourly forecast response.
pub fn mock_hourly() -> HourlyResponse {
    let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
    let hours: Vec<_> = (0..24)
        .map(|h| base_time + chrono::Duration::hours(h))
        .collect();

    HourlyResponse {
        time: hours,
        temperature_2m: vec![
            35.0, 34.0, 33.0, 32.0, 32.0, 33.0, 35.0, 38.0, 41.0, 44.0, 46.0, 47.0,
            48.0, 47.0, 46.0, 44.0, 42.0, 40.0, 38.0, 37.0, 36.0, 35.0, 34.0, 34.0,
        ],
        precipitation_probability: vec![
            0, 0, 0, 0, 0, 5, 10, 15, 20, 25, 30, 35,
            40, 35, 30, 25, 20, 15, 10, 5, 0, 0, 0, 0,
        ],
        relative_humidity_2m: vec![
            70, 72, 74, 76, 78, 76, 72, 68, 64, 60, 58, 56,
            54, 56, 58, 60, 64, 68, 72, 74, 76, 78, 78, 76,
        ],
        wind_speed_10m: vec![
            8.0, 7.0, 6.0, 5.0, 5.0, 6.0, 8.0, 10.0, 12.0, 14.0, 15.0, 16.0,
            16.0, 15.0, 14.0, 12.0, 10.0, 8.0, 7.0, 6.0, 6.0, 6.0, 7.0, 8.0,
        ],
    }
}

/// Create a mock daily forecast response.
pub fn mock_daily() -> DailyResponse {
    let base_date = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
    let days: Vec<_> = (0..7)
        .map(|d| base_date + chrono::Duration::days(d))
        .collect();

    let sunrises: Vec<_> = days
        .iter()
        .map(|d| *d + chrono::Duration::hours(7) + chrono::Duration::minutes(15))
        .collect();

    let sunsets: Vec<_> = days
        .iter()
        .map(|d| *d + chrono::Duration::hours(17) + chrono::Duration::minutes(30))
        .collect();

    DailyResponse {
        time: days,
        temperature_2m_max: vec![48.0, 52.0, 45.0, 42.0, 38.0, 44.0, 50.0],
        temperature_2m_min: vec![32.0, 36.0, 30.0, 28.0, 25.0, 30.0, 35.0],
        precipitation_sum: vec![0.0, 0.1, 0.5, 0.0, 0.0, 0.2, 0.0],
        sunrise: sunrises,
        sunset: sunsets,
    }
}

/// Create a mock geocoding result for testing.
pub fn mock_geocoding_result() -> GeocodingResult {
    GeocodingResult {
        id: 5128581,
        name: "New York".to_string(),
        latitude: 40.7128,
        longitude: -74.0060,
        country: Some("United States".to_string()),
        admin1: Some("New York".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_forecast_has_all_fields() {
        let forecast = mock_forecast_response();
        assert!(forecast.current.is_some());
        assert!(forecast.hourly.is_some());
        assert!(forecast.daily.is_some());
    }

    #[test]
    fn test_mock_hourly_has_24_hours() {
        let hourly = mock_hourly();
        assert_eq!(hourly.time.len(), 24);
        assert_eq!(hourly.temperature_2m.len(), 24);
    }

    #[test]
    fn test_mock_daily_has_7_days() {
        let daily = mock_daily();
        assert_eq!(daily.time.len(), 7);
        assert_eq!(daily.temperature_2m_max.len(), 7);
    }
}
