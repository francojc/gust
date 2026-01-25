//! Cache storage implementation.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use color_eyre::eyre::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};

/// Disk-based cache for weather data.
#[derive(Debug)]
pub struct Cache {
    cache_dir: PathBuf,
    ttl: Duration,
}

impl Cache {
    /// Create a new cache with the given TTL.
    pub fn new(ttl_seconds: u64) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .map(|p| p.join("gust"))
            .unwrap_or_else(|| PathBuf::from(".cache/gust"));

        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

        Ok(Self {
            cache_dir,
            ttl: Duration::from_secs(ttl_seconds),
        })
    }

    /// Generate a cache key for coordinates.
    fn cache_key(&self, lat: f64, lon: f64) -> PathBuf {
        let key = format!("{}_{}.json", lat, lon);
        self.cache_dir.join(key)
    }

    /// Get cached data if it exists and is not expired.
    pub fn get<T: DeserializeOwned>(&self, lat: f64, lon: f64) -> Option<T> {
        let path = self.cache_key(lat, lon);

        if !path.exists() {
            return None;
        }

        // Check if cache is expired
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    if elapsed > self.ttl {
                        return None;
                    }
                }
            }
        }

        // Read and parse cached data
        let contents = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    /// Get cached data even if expired (for offline mode).
    pub fn get_stale<T: DeserializeOwned>(&self, lat: f64, lon: f64) -> Option<T> {
        let path = self.cache_key(lat, lon);

        if !path.exists() {
            return None;
        }

        let contents = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    /// Store data in the cache.
    pub fn set<T: Serialize>(&self, lat: f64, lon: f64, data: &T) -> Result<()> {
        let path = self.cache_key(lat, lon);
        let contents = serde_json::to_string_pretty(data)
            .context("Failed to serialize cache data")?;

        fs::write(&path, contents)
            .with_context(|| format!("Failed to write cache file: {}", path.display()))?;

        Ok(())
    }

    /// Check if cached data is stale (expired).
    pub fn is_stale(&self, lat: f64, lon: f64) -> bool {
        let path = self.cache_key(lat, lon);

        if !path.exists() {
            return true;
        }

        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    return elapsed > self.ttl;
                }
            }
        }

        true
    }

    /// Clear all cached data.
    pub fn clear(&self) -> Result<()> {
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |e| e == "json") {
                fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let cache = Cache::new(1800).unwrap();
        let key = cache.cache_key(40.7128, -74.0060);
        assert!(key.to_string_lossy().contains("40.7128_-74.006"));
    }
}
