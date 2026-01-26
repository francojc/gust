//! API client for Open-Meteo weather data.

#![allow(dead_code)]

mod client;
pub mod convert;
mod geocoding;
pub mod mock;
mod types;

#[allow(unused_imports)]
pub use client::{Units, WeatherClient};
#[allow(unused_imports)]
pub use geocoding::GeocodingClient;
#[allow(unused_imports)]
pub use types::*;
