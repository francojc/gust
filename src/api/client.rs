//! Open-Meteo weather API client.

use color_eyre::eyre::{Context, Result};

use super::types::{AirQualityResponse, ForecastResponse};

const BASE_URL: &str = "https://api.open-meteo.com/v1/forecast";
const AIR_QUALITY_URL: &str = "https://air-quality-api.open-meteo.com/v1/air-quality";

/// Client for fetching weather data from Open-Meteo.
#[derive(Debug, Clone)]
pub struct WeatherClient {
    client: reqwest::Client,
    units: Units,
}

/// Unit system for weather data.
#[derive(Debug, Clone, Copy, Default)]
pub enum Units {
    #[default]
    Imperial,
    Metric,
}

impl WeatherClient {
    /// Create a new weather client.
    pub fn new(units: Units) -> Self {
        Self {
            client: reqwest::Client::new(),
            units,
        }
    }

    /// Fetch weather forecast for the given coordinates.
    pub async fn fetch_forecast(&self, lat: f64, lon: f64) -> Result<ForecastResponse> {
        let (temp_unit, wind_unit, precip_unit) = match self.units {
            Units::Imperial => ("fahrenheit", "mph", "inch"),
            Units::Metric => ("celsius", "kmh", "mm"),
        };

        let url = format!(
            "{}?latitude={}&longitude={}\
            &current=temperature_2m,apparent_temperature,relative_humidity_2m,\
            wind_speed_10m,wind_direction_10m,surface_pressure,weather_code\
            &hourly=temperature_2m,precipitation_probability,relative_humidity_2m,wind_speed_10m,weather_code,uv_index\
            &daily=temperature_2m_max,temperature_2m_min,precipitation_sum,sunrise,sunset,uv_index_max,daylight_duration\
            &temperature_unit={}&wind_speed_unit={}&precipitation_unit={}\
            &timezone=auto",
            BASE_URL, lat, lon, temp_unit, wind_unit, precip_unit
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to Open-Meteo")?;

        let forecast = response
            .json::<ForecastResponse>()
            .await
            .context("Failed to parse Open-Meteo response")?;

        Ok(forecast)
    }

    /// Fetch air quality data for the given coordinates.
    pub async fn fetch_air_quality(&self, lat: f64, lon: f64) -> Result<AirQualityResponse> {
        let url = format!(
            "{}?latitude={}&longitude={}&current=us_aqi,pm2_5,pm10",
            AIR_QUALITY_URL, lat, lon
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to Open-Meteo Air Quality API")?;

        let air_quality = response
            .json::<AirQualityResponse>()
            .await
            .context("Failed to parse Open-Meteo Air Quality response")?;

        Ok(air_quality)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = WeatherClient::new(Units::Imperial);
        assert!(matches!(client.units, Units::Imperial));
    }
}
