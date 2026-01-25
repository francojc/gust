//! Open-Meteo geocoding API client.

use color_eyre::eyre::{Context, Result};

use super::types::{GeocodingResponse, GeocodingResult};

const GEOCODING_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";

/// Client for geocoding location searches.
#[derive(Debug, Clone)]
pub struct GeocodingClient {
    client: reqwest::Client,
}

impl Default for GeocodingClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GeocodingClient {
    /// Create a new geocoding client.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Search for locations by name.
    pub async fn search(&self, query: &str) -> Result<Vec<GeocodingResult>> {
        let url = format!(
            "{}?name={}&count=5&language=en&format=json",
            GEOCODING_URL,
            urlencoding::encode(query)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send geocoding request")?;

        let geocoding = response
            .json::<GeocodingResponse>()
            .await
            .context("Failed to parse geocoding response")?;

        Ok(geocoding.results.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geocoding_client_creation() {
        let _client = GeocodingClient::new();
    }
}
