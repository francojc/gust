//! API response types for Open-Meteo.

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// Custom deserializer for NaiveDateTime without seconds (Open-Meteo format).
mod datetime_no_seconds {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }

    pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&dt.format(FORMAT).to_string())
    }
}

/// Custom deserializer for Vec<NaiveDateTime> without seconds.
mod datetime_no_seconds_vec {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let strings: Vec<String> = Vec::deserialize(deserializer)?;
        strings
            .into_iter()
            .map(|s| NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom))
            .collect()
    }

    pub fn serialize<S>(dts: &Vec<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(dts.len()))?;
        for dt in dts {
            seq.serialize_element(&dt.format(FORMAT).to_string())?;
        }
        seq.end()
    }
}

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
    #[serde(with = "datetime_no_seconds")]
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
    #[serde(with = "datetime_no_seconds_vec")]
    pub time: Vec<NaiveDateTime>,
    pub temperature_2m: Vec<f64>,
    pub precipitation_probability: Vec<u8>,
    pub relative_humidity_2m: Vec<u8>,
    pub wind_speed_10m: Vec<f64>,
    pub weather_code: Vec<u8>,
    #[serde(default)]
    pub uv_index: Vec<f64>,
}

/// Daily forecast data from API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DailyResponse {
    pub time: Vec<NaiveDate>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub precipitation_sum: Vec<f64>,
    #[serde(with = "datetime_no_seconds_vec")]
    pub sunrise: Vec<NaiveDateTime>,
    #[serde(with = "datetime_no_seconds_vec")]
    pub sunset: Vec<NaiveDateTime>,
    #[serde(default)]
    pub uv_index_max: Vec<f64>,
    #[serde(default)]
    pub daylight_duration: Vec<f64>,
}

/// Response from the Open-Meteo Air Quality API.
#[derive(Debug, Clone, Deserialize)]
pub struct AirQualityResponse {
    pub current: Option<AirQualityCurrent>,
}

/// Current air quality data from API.
#[derive(Debug, Clone, Deserialize)]
pub struct AirQualityCurrent {
    pub us_aqi: u16,
    pub pm2_5: f64,
    pub pm10: f64,
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
