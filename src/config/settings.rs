//! Application settings and TOML configuration.

use color_eyre::eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Main application configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub location: LocationConfig,
    pub display: DisplayConfig,
    pub behavior: BehaviorConfig,
}

/// Location-related settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LocationConfig {
    /// Default location to load on startup.
    pub default: String,
    /// List of favorite locations.
    pub favorites: Vec<String>,
}

/// Display settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    /// Unit system: "imperial" or "metric".
    pub units: String,
    /// Color theme: "dark", "light", "solarized", "nord".
    pub theme: String,
    /// Time format: "12h" or "24h".
    pub time_format: String,
}

/// Behavior settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    /// Auto-refresh interval in seconds.
    pub refresh_interval: u64,
    /// Cache TTL in seconds.
    pub cache_duration: u64,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            units: "imperial".to_string(),
            theme: "dark".to_string(),
            time_format: "12h".to_string(),
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            refresh_interval: 900,  // 15 minutes
            cache_duration: 1800,   // 30 minutes
        }
    }
}

impl AppConfig {
    /// Get the configuration file path.
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("gust").join("config.toml"))
    }

    /// Load configuration from file, or return defaults if not found.
    pub fn load() -> Result<Self> {
        let Some(path) = Self::config_path() else {
            return Ok(Self::default());
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))
    }

    /// Save configuration to file.
    pub fn save(&self) -> Result<()> {
        let Some(path) = Self::config_path() else {
            return Ok(());
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.display.units, "imperial");
        assert_eq!(config.display.theme, "dark");
        assert_eq!(config.behavior.refresh_interval, 900);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.display.units, config.display.units);
    }
}
