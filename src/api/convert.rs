//! Convert API response types to application domain types.

use crate::api::types::{CurrentResponse, DailyResponse, ForecastResponse, HourlyResponse};
use crate::app::{CurrentWeather, DailyForecast, HourlyForecast, PrecipType, WeatherData};

/// Convert a forecast API response to the app's WeatherData type.
pub fn forecast_to_weather_data(resp: ForecastResponse) -> WeatherData {
    let hourly = resp.hourly.map(convert_hourly).unwrap_or_default();

    // Extract current UV from first hourly entry
    let current_uv = hourly.first().map(|h| h.uv_index).unwrap_or(0.0);

    let mut current = resp.current.map(convert_current).unwrap_or_default();
    current.uv_index = current_uv;

    WeatherData {
        current,
        hourly,
        daily: resp.daily.map(convert_daily).unwrap_or_default(),
        timezone: resp.timezone,
    }
}

/// Convert current weather API response to domain type.
fn convert_current(c: CurrentResponse) -> CurrentWeather {
    CurrentWeather {
        temperature: c.temperature_2m,
        feels_like: c.apparent_temperature,
        humidity: c.relative_humidity_2m,
        wind_speed: c.wind_speed_10m,
        wind_direction: degrees_to_direction(c.wind_direction_10m),
        pressure: c.surface_pressure,
        description: weather_code_to_description(c.weather_code),
        weather_code: c.weather_code,
        uv_index: 0.0, // Will be set from hourly data
    }
}

/// Convert hourly forecast API response to domain type.
fn convert_hourly(h: HourlyResponse) -> Vec<HourlyForecast> {
    h.time
        .into_iter()
        .enumerate()
        .map(|(i, time)| {
            let weather_code = h.weather_code.get(i).copied().unwrap_or(0);
            HourlyForecast {
                time,
                temperature: h.temperature_2m.get(i).copied().unwrap_or(0.0),
                precipitation_probability: h.precipitation_probability.get(i).copied().unwrap_or(0),
                humidity: h.relative_humidity_2m.get(i).copied().unwrap_or(0),
                wind_speed: h.wind_speed_10m.get(i).copied().unwrap_or(0.0),
                precip_type: weather_code_to_precip_type(weather_code),
                uv_index: h.uv_index.get(i).copied().unwrap_or(0.0),
            }
        })
        .collect()
}

/// Convert WMO weather code to precipitation type.
fn weather_code_to_precip_type(code: u8) -> PrecipType {
    match code {
        // Rain codes: slight/moderate/heavy rain, rain showers
        61..=65 | 80..=82 => PrecipType::Rain,
        // Snow codes: slight/moderate/heavy snow, snow grains, snow showers
        71..=77 | 85..=86 => PrecipType::Snow,
        // Mixed: drizzle, freezing drizzle, freezing rain, thunderstorms
        51..=57 | 66..=67 | 95..=99 => PrecipType::Mixed,
        // No precipitation or other (clear, cloudy, fog)
        _ => PrecipType::None,
    }
}

/// Convert daily forecast API response to domain type.
fn convert_daily(d: DailyResponse) -> Vec<DailyForecast> {
    d.time
        .into_iter()
        .enumerate()
        .map(|(i, date)| {
            let default_sunrise = date.and_hms_opt(6, 0, 0).unwrap();
            let default_sunset = date.and_hms_opt(18, 0, 0).unwrap();
            DailyForecast {
                date,
                temp_high: d.temperature_2m_max.get(i).copied().unwrap_or(0.0),
                temp_low: d.temperature_2m_min.get(i).copied().unwrap_or(0.0),
                precipitation_sum: d.precipitation_sum.get(i).copied().unwrap_or(0.0),
                sunrise: d.sunrise.get(i).copied().unwrap_or(default_sunrise),
                sunset: d.sunset.get(i).copied().unwrap_or(default_sunset),
                uv_index_max: d.uv_index_max.get(i).copied().unwrap_or(0.0),
                daylight_duration: d.daylight_duration.get(i).copied().unwrap_or(0.0),
            }
        })
        .collect()
}

/// Convert wind direction in degrees to compass direction string.
fn degrees_to_direction(deg: f64) -> String {
    let directions = ["N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE",
                      "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW"];
    let normalized = ((deg % 360.0) + 360.0) % 360.0;
    let index = ((normalized + 11.25) / 22.5) as usize % 16;
    directions[index].to_string()
}

/// Convert WMO weather code to human-readable description.
fn weather_code_to_description(code: u8) -> String {
    match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 => "Fog",
        48 => "Depositing rime fog",
        51 => "Light drizzle",
        53 => "Moderate drizzle",
        55 => "Dense drizzle",
        56 => "Light freezing drizzle",
        57 => "Dense freezing drizzle",
        61 => "Slight rain",
        63 => "Moderate rain",
        65 => "Heavy rain",
        66 => "Light freezing rain",
        67 => "Heavy freezing rain",
        71 => "Slight snow fall",
        73 => "Moderate snow fall",
        75 => "Heavy snow fall",
        77 => "Snow grains",
        80 => "Slight rain showers",
        81 => "Moderate rain showers",
        82 => "Violent rain showers",
        85 => "Slight snow showers",
        86 => "Heavy snow showers",
        95 => "Thunderstorm",
        96 => "Thunderstorm with slight hail",
        99 => "Thunderstorm with heavy hail",
        _ => "Unknown",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::mock;

    #[test]
    fn test_degrees_to_direction() {
        assert_eq!(degrees_to_direction(0.0), "N");
        assert_eq!(degrees_to_direction(45.0), "NE");
        assert_eq!(degrees_to_direction(90.0), "E");
        assert_eq!(degrees_to_direction(135.0), "SE");
        assert_eq!(degrees_to_direction(180.0), "S");
        assert_eq!(degrees_to_direction(225.0), "SW");
        assert_eq!(degrees_to_direction(270.0), "W");
        assert_eq!(degrees_to_direction(315.0), "NW");
        assert_eq!(degrees_to_direction(360.0), "N");
        assert_eq!(degrees_to_direction(-45.0), "NW");
    }

    #[test]
    fn test_weather_code_to_description() {
        assert_eq!(weather_code_to_description(0), "Clear sky");
        assert_eq!(weather_code_to_description(3), "Overcast");
        assert_eq!(weather_code_to_description(63), "Moderate rain");
        assert_eq!(weather_code_to_description(95), "Thunderstorm");
        assert_eq!(weather_code_to_description(255), "Unknown");
    }

    #[test]
    fn test_forecast_to_weather_data() {
        let mock_resp = mock::mock_forecast_response();
        let data = forecast_to_weather_data(mock_resp);

        assert_eq!(data.current.temperature, 42.0);
        assert_eq!(data.current.feels_like, 38.0);
        assert_eq!(data.current.humidity, 65);
        assert_eq!(data.current.wind_direction, "SW");
        assert_eq!(data.current.description, "Partly cloudy");
        assert_eq!(data.hourly.len(), 24);
        assert_eq!(data.daily.len(), 7);
    }

    #[test]
    fn test_convert_current() {
        let current = mock::mock_current();
        let converted = convert_current(current);

        assert_eq!(converted.temperature, 42.0);
        assert_eq!(converted.feels_like, 38.0);
        assert_eq!(converted.humidity, 65);
        assert_eq!(converted.wind_speed, 12.0);
        assert_eq!(converted.wind_direction, "SW");
        assert_eq!(converted.pressure, 30.12);
        assert_eq!(converted.description, "Partly cloudy");
    }

    #[test]
    fn test_convert_hourly() {
        let hourly = mock::mock_hourly();
        let converted = convert_hourly(hourly);

        assert_eq!(converted.len(), 24);
        assert_eq!(converted[0].temperature, 35.0);
        assert_eq!(converted[0].precipitation_probability, 0);
        assert_eq!(converted[12].temperature, 48.0);
        assert_eq!(converted[12].precipitation_probability, 40);
    }

    #[test]
    fn test_convert_daily() {
        let daily = mock::mock_daily();
        let converted = convert_daily(daily);

        assert_eq!(converted.len(), 7);
        assert_eq!(converted[0].temp_high, 48.0);
        assert_eq!(converted[0].temp_low, 32.0);
        assert_eq!(converted[0].precipitation_sum, 0.0);
    }

    #[test]
    fn test_forecast_with_missing_data() {
        let resp = ForecastResponse {
            latitude: 0.0,
            longitude: 0.0,
            timezone: "UTC".to_string(),
            current: None,
            hourly: None,
            daily: None,
        };
        let data = forecast_to_weather_data(resp);

        assert_eq!(data.current.temperature, 0.0);
        assert!(data.hourly.is_empty());
        assert!(data.daily.is_empty());
    }
}
