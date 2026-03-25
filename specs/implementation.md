# Development Implementation Details

**Project:** gust
**Status:** Active
**Last Updated:** 2026-03-24

## Architecture

### System Design

- **Architecture Pattern:** Single-process TUI, async data layer
- **Primary Language:** Rust (edition 2021, stable toolchain)
- **Framework:** ratatui 0.30 + crossterm 0.28
- **Build System:** Cargo + Nix flake (rust-overlay)

### Component Overview

```
gust/
├── src/
│   ├── main.rs          # Entry point, tokio loop, channel routing
│   ├── app.rs           # AppState, Message enum, update/view
│   ├── astro.rs         # Astronomical data (sunrise/sunset)
│   ├── error.rs         # color-eyre init
│   ├── api/
│   │   ├── mod.rs
│   │   ├── client.rs    # WeatherClient (Open-Meteo + AQI)
│   │   ├── alerts.rs    # AlertsClient (NWS)
│   │   ├── geocoding.rs # GeocodingClient
│   │   ├── convert.rs   # ForecastResponse → WeatherData
│   │   ├── types.rs     # Serde API response types
│   │   └── mock.rs      # Test fixture data
│   ├── cache/
│   │   ├── mod.rs
│   │   └── storage.rs   # Disk cache (lat/lon keyed JSON)
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs  # AppConfig from ~/.config/gust/config.toml
│   └── ui/
│       ├── mod.rs        # View dispatch (routes active tab)
│       ├── header.rs     # Location, AQI, time bar
│       ├── footer.rs     # Key hint bar
│       ├── layout.rs     # Layout helpers
│       ├── theme.rs      # Color palette from theme setting
│       └── graphs/
│           ├── mod.rs
│           ├── temperature.rs
│           ├── precipitation.rs
│           ├── humidity.rs
│           └── alerts.rs
├── specs/               # Project planning and tracking
├── logs/                # Weekly reviews and session logs
├── Cargo.toml
├── flake.nix
└── .gitignore
```

### Key Modules

1. **`app`**
   - **Purpose:** Central state machine. Owns all mutable app state
     (`AppState`). Dispatches `Message` variants from the event loop.
   - **Public Interface:** `AppState::new()`, `state.update(msg)`,
     `state.view(frame)`
   - **Dependencies:** `api`, `config`, `cache`, `ui`

2. **`api`**
   - **Purpose:** HTTP clients for Open-Meteo (weather + AQI),
     Open-Meteo geocoding, and NWS alerts. `convert` module maps API
     responses to internal types.
   - **Public Interface:** `WeatherClient`, `GeocodingClient`,
     `AlertsClient`, `convert::forecast_to_weather_data()`
   - **Dependencies:** `reqwest`, `serde`, `tokio`

3. **`ui/graphs`**
   - **Purpose:** Renders Braille-based graphs for temperature,
     precipitation, and humidity using ratatui canvas. Handles
     timezone-aware X-axis with 6-hour tick marks and day labels.
   - **Public Interface:** `render_temperature()`,
     `render_precipitation()`, `render_humidity()`, `render_alerts()`
   - **Dependencies:** `ratatui`, `chrono`, `chrono-tz`

4. **`cache`**
   - **Purpose:** Disk-based JSON cache keyed by `(lat, lon)`. Reads
     within TTL return fresh data; beyond TTL returns stale data as
     fallback on network failure.
   - **Public Interface:** `Cache::new(ttl)`, `cache.get()`,
     `cache.set()`, `cache.get_stale()`
   - **Dependencies:** `serde_json`, `dirs`, `chrono`

5. **`config`**
   - **Purpose:** Loads `~/.config/gust/config.toml` at startup.
     Provides `AppConfig` with defaults if the file is absent.
   - **Public Interface:** `AppConfig::load()`
   - **Dependencies:** `toml`, `serde`, `dirs`

### Data Model

- **Primary Types:** `WeatherData` (hourly arrays), `WeatherAlert`,
  `AirQuality`, `GeocodingResult`, `AppConfig`
- **Persistence:** Disk cache in user data dir; config in user config
  dir (both via `dirs` crate)
- **Serialization:** JSON for cache, TOML for config

## Development Environment

### Setup

```bash
# Activate Nix dev shell (recommended)
direnv allow
# or
nix develop

# Verify Rust toolchain
rustc --version
cargo --version
```

### Build and Run

```bash
# Build debug
cargo build

# Run
cargo run

# Run with location argument
cargo run -- "Austin, TX"

# Watch for changes (requires cargo-watch)
cargo watch -x run

# Release build
cargo build --release

# Install locally
cargo install --path .
```

### Code Standards

- **Formatting:** `cargo fmt` (rustfmt defaults)
- **Linting:** `cargo clippy` — zero warnings policy
- **Naming:** `snake_case` for functions/variables/modules,
  `PascalCase` for types and enums
- **Error Handling:** `color-eyre` for user-facing errors; internal
  errors propagated via `Result`

## Testing Strategy

### Test Levels

- **Snapshot Tests:** insta framework; tests live in the same module
  as the code under test (Rust inline test convention)
- **Unit Tests:** `#[cfg(test)]` modules within source files
- **Integration Tests:** none yet (HTTP clients use `mock.rs` data
  for unit testing)

### Running Tests

```bash
# All tests
cargo test

# Review changed snapshots interactively
cargo insta review

# Accept all pending snapshots (use with care)
cargo insta accept
```

### Coverage Targets

- **Graph render functions:** high priority for snapshot coverage
- **`convert` module:** unit test coverage for data mapping logic
- **Exclusions:** `main.rs` event loop and terminal setup functions

### Test Data

- **Fixtures:** `src/api/mock.rs` provides static `ForecastResponse`
  and related types for use in tests
- **Snapshots:** stored in `src/**/__snapshots__/` alongside source;
  committed to version control

## Deployment

### Target Environment

- **Platform:** Local install via `cargo install` or Nix flake
- **Runtime:** Native binary
- **Configuration:** `~/.config/gust/config.toml` (user-managed)

### CI/CD Pipeline

- Not yet configured. Planned: GitHub Actions on push/PR to run
  `cargo fmt --check`, `cargo clippy`, `cargo test`

### Release Process

- **Versioning:** SemVer (currently pre-1.0, v0.1.0 is first release)
- **Release Steps:**
  1. Update `Cargo.toml` version
  2. Run full test suite
  3. Tag `v0.1.0` with release notes
  4. Verify `nix run github:francojc/gust` works

## Monitoring and Observability

### Logging

- **Framework:** `tracing` + `tracing-subscriber`
- **Usage:** debug-level tracing for API calls and cache hits/misses
- **Log Levels:** errors for network failures, debug for cache and
  fetch lifecycle events

### Error Handling

- **Framework:** `color-eyre` for top-level error presentation
- **User-Facing Errors:** displayed in the TUI status area (e.g.,
  "No locations found for '...'")
- **Non-fatal Errors:** cache failures, NWS 404 — logged, not
  surfaced to user

## Security Considerations

### Input Validation

- Location search strings are URL-encoded via `urlencoding` before
  being passed to the geocoding API
- Config values are validated by type deserialization (TOML)

### Authentication and Authorization

- No authentication required; all APIs are public
- No credentials stored

### Dependency Security

- `cargo audit` can be run manually; not yet automated in CI

## Decision Log

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|------------------------|
| 2026-03-24 | Use Open-Meteo for weather data | No API key required; ECMWF/GFS model quality | OpenWeatherMap (requires key), WeatherAPI (paid) |
| 2026-03-24 | Use ratatui for TUI | Active maintenance, Braille canvas support | tui-rs (archived), termion (less cross-platform) |
| 2026-03-24 | Use insta for snapshot testing | Ergonomic snapshot workflow for render output | Manual string comparison, goldenfile |
| 2026-03-24 | Disk cache keyed by (lat, lon) | Avoid repeat API calls on relaunch | In-memory only (lost on exit), time-based filenames |
| 2026-03-24 | Tokio mpsc channels for fetch results | Keeps async data fetching off the render loop | Blocking fetch in event loop, actor model |
