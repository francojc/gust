# gust Development Plan

## Project Status

- [ ] Stage 1: Project Setup & Architecture
- [ ] Stage 2: Data Layer
- [ ] Stage 3: Static UI
- [ ] Stage 4: Visualization
- [ ] Stage 5: Interaction & Polish
- [ ] Stage 6: Extended Features
- [ ] Stage 7: Release Preparation

---

## Architecture

### Elm Architecture (TEA)

The application follows The Elm Architecture pattern:

```
Model (AppState) → Message (Events) → Update (State Transitions) → View (Rendering)
```

### Module Structure

```
src/
├── main.rs           # Entry point, terminal setup
├── app.rs            # AppState, Message, update(), view()
├── api/
│   ├── mod.rs
│   ├── client.rs     # Open-Meteo HTTP client
│   ├── geocoding.rs  # Location search
│   └── types.rs      # API response structs
├── cache/
│   ├── mod.rs
│   └── storage.rs    # Disk cache implementation
├── config/
│   ├── mod.rs
│   └── settings.rs   # TOML config handling
├── ui/
│   ├── mod.rs
│   ├── layout.rs     # Screen layout (20/60/20 split)
│   ├── header.rs     # Current conditions widget
│   ├── footer.rs     # Status bar, input field
│   ├── graphs/
│   │   ├── mod.rs
│   │   ├── temperature.rs
│   │   ├── precipitation.rs
│   │   └── humidity.rs
│   └── theme.rs      # Color schemes
└── error.rs          # Error types, panic hook
```

---

## Stage 1: Project Setup & Architecture

### Tasks

- [x] Create brainstorm.md
- [x] Create development plan
- [ ] Initialize Cargo project
- [ ] Set up Nix development environment
- [ ] Add dependencies to Cargo.toml
- [ ] Implement Elm Architecture skeleton
- [ ] Set up error handling with color-eyre
- [ ] Create configuration system
- [ ] Write initial test infrastructure

### Dependencies (Cargo.toml)

```toml
[package]
name = "gust"
version = "0.1.0"
edition = "2021"
description = "A terminal-based weather dashboard"
license = "MIT"
repository = "https://github.com/francojc/gust"

[dependencies]
ratatui = "0.30"
crossterm = { version = "0.28", features = ["event-stream"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
color-eyre = "0.6"
dirs = "5"
toml = "0.8"
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
insta = "1"
```

### State Structure

```rust
pub struct AppState {
    pub location: Location,
    pub current: Option<CurrentWeather>,
    pub hourly: Vec<HourlyForecast>,
    pub daily: Vec<DailyForecast>,
    pub alerts: Vec<WeatherAlert>,
    pub selected_tab: Tab,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub loading: bool,
    pub error: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
    pub config: AppConfig,
}

pub enum Message {
    Input(KeyEvent),
    WeatherReceived(Result<WeatherData, Error>),
    AlertsReceived(Result<Vec<WeatherAlert>, Error>),
    Tick,
    Refresh,
    Quit,
}

pub enum Tab {
    Temperature,
    Precipitation,
    Humidity,
    Alerts,
}

pub enum InputMode {
    Normal,
    Search,
}
```

---

## Stage 2: Data Layer

### Tasks

- [ ] Define API response structs with serde
- [ ] Implement Open-Meteo client
- [ ] Implement geocoding (location search)
- [ ] Add caching layer (~/.cache/gust/)
- [ ] Create mock data for testing
- [ ] Unit tests for API client

### Open-Meteo API Endpoints

**Weather Forecast:**

```
https://api.open-meteo.com/v1/forecast
  ?latitude={lat}
  &longitude={lon}
  &hourly=temperature_2m,precipitation_probability,relative_humidity_2m,wind_speed_10m
  &daily=temperature_2m_max,temperature_2m_min,precipitation_sum,sunrise,sunset
  &temperature_unit=fahrenheit
  &wind_speed_unit=mph
  &precipitation_unit=inch
  &timezone=auto
```

**Geocoding:**

```
https://geocoding-api.open-meteo.com/v1/search
  ?name={city}
  &count=5
  &language=en
  &format=json
```

### Caching Strategy

- Cache location: `~/.cache/gust/`
- Cache key: `{lat}_{lon}.json`
- Include timestamp in cached file
- Serve stale data with visual indicator when offline
- Default TTL: 30 minutes (configurable)

---

## Stage 3: Static UI

### Tasks

- [ ] Build responsive layout system
- [ ] Implement Header widget (current conditions)
- [ ] Implement Footer widget (status bar, input)
- [ ] Set up crossterm event loop
- [ ] Add basic key bindings (q, r, /, 1-3)
- [ ] Implement terminal size detection
- [ ] Snapshot tests for widgets

### Layout Structure

```
┌─────────────────────────────────────────────────┐
│ Header (20%)                                    │
│ ┌─────────────────────────────────────────────┐ │
│ │ NEW YORK, NY                    72°F  ☀️     │ │
│ │ Feels like 75°F | Wind 5 mph SW | 30.1 in   │ │
│ └─────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────┤
│ Main View (60%)                                 │
│ ┌─────────────────────────────────────────────┐ │
│ │ [1:Temp] [2:Precip] [3:Humidity]            │ │
│ │                                              │ │
│ │         ╭───╮                                │ │
│ │     ╭───╯   ╰───╮         ╭───╮             │ │
│ │ ────╯           ╰─────────╯   ╰────         │ │
│ │                                              │ │
│ │ Mon    Tue    Wed    Thu    Fri    Sat      │ │
│ └─────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────┤
│ Footer (20%)                                    │
│ ┌─────────────────────────────────────────────┐ │
│ │ > _                                          │ │
│ │ Source: ECMWF | Updated: 2:30 PM | q:quit   │ │
│ └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

---

## Stage 4: Visualization

### Tasks

- [ ] Implement Canvas-based temperature graph
- [ ] Add precipitation bar chart
- [ ] Add humidity area chart
- [ ] Implement tab switching logic
- [ ] Add axis labels and legends
- [ ] Handle terminal resize events
- [ ] Fallback rendering for limited terminals

### Graph Specifications

**Temperature Graph:**

- Type: Line graph with Braille rendering
- Color: Yellow (low) → Red (high)
- Y-axis: Temperature range (auto-scaled)
- X-axis: Time (hours/days)

**Precipitation Graph:**

- Type: Bar chart
- Color: Blue gradient by intensity
- Y-axis: Probability % or inches
- X-axis: Time

**Humidity Graph:**

- Type: Area chart
- Color: Cyan
- Y-axis: 0-100%
- X-axis: Time

---

## Stage 5: Interaction & Polish

### Tasks

- [ ] Implement command mode (/ to search)
- [ ] Add loading states and spinners
- [ ] Error display in UI
- [ ] Mouse support (click to select)
- [ ] Theme system (dark, light, solarized, nord)
- [ ] Timeline panning (←/→ arrows)
- [ ] 12h/24h time format toggle

### Async Event Loop

```rust
loop {
    tokio::select! {
        Some(event) = input_rx.recv() => {
            // Handle user input
        }
        Some(data) = weather_rx.recv() => {
            // Handle API response
        }
        _ = tick_interval.tick() => {
            // Auto-refresh check
        }
        _ = render_interval.tick() => {
            // Render frame (~60fps)
        }
    }
}
```

---

## Stage 6: Extended Features

### Tasks

- [ ] Weather alerts integration
- [ ] Multi-location comparison view
- [ ] UV Index display
- [ ] Air Quality (AQI) display
- [ ] Astronomical data (sunrise/sunset, moon phase)
- [ ] Location history (recent searches)
- [ ] Export to JSON/plain text

### Weather Alerts

- Pull from Open-Meteo alerts or NWS API
- Color-coded severity:
  - Yellow: Advisory
  - Orange: Watch
  - Red: Warning
- Dismissible with timestamp

---

## Stage 7: Release Preparation

### Tasks

- [ ] Comprehensive test coverage
- [ ] Documentation (README, man page)
- [ ] Cross-platform testing (Linux, macOS, Windows)
- [ ] Performance optimization
- [ ] Package for cargo install
- [ ] Homebrew formula
- [ ] AUR package
- [ ] GitHub Actions CI/CD

---

## Testing Strategy

### Unit Tests

- API client with mock responses
- State transitions (update function)
- Config parsing
- Cache operations

### Widget Tests

```rust
#[test]
fn test_header_renders_current_conditions() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend)?;
    let state = AppState::with_mock_data();
    terminal.draw(|f| header::render(&state, f))?;
    assert_snapshot!(terminal.backend().buffer());
}
```

### Integration Tests

- Full app lifecycle with mock API
- Key binding behavior
- Error recovery scenarios

---

## Error Handling

### Categories

1. **Recoverable**: Network timeout, API rate limit
   - Show error in UI
   - Serve cached data
   - Retry with backoff

2. **Unrecoverable**: Terminal init failure
   - Log error
   - Restore terminal state
   - Exit gracefully

### Panic Hook

```rust
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal state
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stderr(),
            crossterm::terminal::LeaveAlternateScreen
        );
        original_hook(panic_info);
    }));
}
```

---

## Configuration Reference

### Default Config (~/.config/gust/config.toml)

```toml
[location]
default = ""
favorites = []

[display]
units = "imperial"
theme = "dark"
time_format = "12h"

[behavior]
refresh_interval = 900
cache_duration = 1800

[advanced]
api_base = "https://api.open-meteo.com"
model = "best_match"  # ecmwf, gfs, best_match
```

---

## API Reference

### AppConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `location.default` | String | "" | Default city to load |
| `location.favorites` | Vec<String> | [] | Saved locations |
| `display.units` | "imperial" \| "metric" | "imperial" | Unit system |
| `display.theme` | String | "dark" | Color theme |
| `display.time_format` | "12h" \| "24h" | "12h" | Time display |
| `behavior.refresh_interval` | u64 | 900 | Auto-refresh seconds |
| `behavior.cache_duration` | u64 | 1800 | Cache TTL seconds |

---

## Verification

After implementation, verify:

1. `cargo build` compiles without warnings
2. `cargo test` passes all tests
3. `cargo clippy` has no lints
4. `cargo run` launches TUI
5. Location search works
6. Graphs render correctly
7. Key bindings respond
8. Config persists between sessions
9. Offline mode serves cached data
