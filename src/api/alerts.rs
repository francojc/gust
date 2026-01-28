//! NWS Weather Alerts API client.

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::app::{AlertSeverity, WeatherAlert};

const USER_AGENT: &str = "gust-weather-dashboard/0.1.0 (https://github.com/francojc/gust)";

/// Response from NWS alerts API.
#[derive(Debug, Deserialize)]
struct NWSResponse {
    features: Vec<NWSFeature>,
}

/// Single alert feature from NWS.
#[derive(Debug, Deserialize)]
struct NWSFeature {
    properties: NWSProperties,
}

/// Alert properties from NWS.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NWSProperties {
    event: String,
    severity: String,
    headline: Option<String>,
    description: String,
    expires: Option<DateTime<Utc>>,
}

/// Client for fetching weather alerts from NWS.
#[derive(Debug, Clone)]
pub struct AlertsClient {
    client: Client,
}

impl AlertsClient {
    /// Create a new alerts client.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Fetch active weather alerts for the given coordinates.
    ///
    /// Uses the NWS API which is US-only. Returns an empty vector for
    /// non-US locations or on error.
    pub async fn fetch(&self, lat: f64, lon: f64) -> Result<Vec<WeatherAlert>, String> {
        let url = format!(
            "https://api.weather.gov/alerts/active?point={:.4},{:.4}",
            lat, lon
        );

        let resp = self
            .client
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        // NWS returns 404 for non-US coordinates
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(vec![]);
        }

        if !resp.status().is_success() {
            return Err(format!("NWS API error: {}", resp.status()));
        }

        let data: NWSResponse = resp.json().await.map_err(|e| e.to_string())?;

        Ok(data
            .features
            .into_iter()
            .map(|f| {
                let severity = match f.properties.severity.as_str() {
                    "Extreme" | "Severe" => AlertSeverity::Warning,
                    "Moderate" => AlertSeverity::Watch,
                    _ => AlertSeverity::Advisory,
                };

                WeatherAlert {
                    severity,
                    title: f.properties.event,
                    description: f
                        .properties
                        .headline
                        .unwrap_or_else(|| truncate_description(&f.properties.description)),
                    expires: f
                        .properties
                        .expires
                        .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(24)),
                }
            })
            .collect())
    }
}

impl Default for AlertsClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Truncate description to first sentence or 100 characters.
fn truncate_description(desc: &str) -> String {
    // Find first sentence ending
    if let Some(pos) = desc.find(". ") {
        return desc[..=pos].to_string();
    }
    if let Some(pos) = desc.find(".\n") {
        return desc[..=pos].to_string();
    }
    // Truncate if too long
    if desc.len() > 100 {
        format!("{}...", &desc[..97])
    } else {
        desc.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_description_short() {
        let desc = "Short description.";
        assert_eq!(truncate_description(desc), "Short description.");
    }

    #[test]
    fn test_truncate_description_with_sentence() {
        let desc = "First sentence. Second sentence continues here.";
        assert_eq!(truncate_description(desc), "First sentence.");
    }

    #[test]
    fn test_truncate_description_long() {
        let desc = "A".repeat(150);
        let result = truncate_description(&desc);
        assert!(result.ends_with("..."));
        assert_eq!(result.len(), 100);
    }

    #[test]
    fn test_alerts_client_creation() {
        let client = AlertsClient::new();
        assert!(format!("{:?}", client).contains("AlertsClient"));
    }
}
