# gust

A terminal-based weather dashboard that focuses on visual data density and high-resolution aesthetics.

## Features

- Real-time weather data from Open-Meteo (ECMWF/GFS models)
- High-resolution Braille-based graphs for temperature, precipitation, and humidity
- Vim-inspired keyboard navigation
- Configurable units (Imperial/Metric)
- Offline mode with cached data
- Weather alerts display

## Installation

### From source

```bash
cargo install --path .
```

### Using Nix

```bash
nix run github:francojc/gust
```

## Usage

```bash
# Launch with default location (from config)
gust

# Launch with specific location
gust "New York, NY"

# Show help
gust --help
```

## Key Bindings

| Key | Action |
|-----|--------|
| `1` | Temperature graph |
| `2` | Precipitation graph |
| `3` | Humidity graph |
| `/` | Search location |
| `r` | Refresh data |
| `q` | Quit |
| `←/→` | Pan timeline |
| `Tab` | Cycle views |

## Configuration

Configuration is stored at `~/.config/gust/config.toml`:

```toml
[location]
default = "New York, NY"
favorites = ["London, UK", "Tokyo, JP"]

[display]
units = "imperial"  # or "metric"
theme = "dark"      # dark, light, solarized, nord
time_format = "12h" # or "24h"

[behavior]
refresh_interval = 900  # seconds (15 minutes)
cache_duration = 1800   # seconds (30 minutes)
```

## Requirements

- Terminal with Unicode support (Braille characters)
- Recommended: Alacritty, Kitty, WezTerm, iTerm2

## Development

This project uses Nix for reproducible development environments:

```bash
# Enter development shell
direnv allow
# or
nix develop

# Run in development
cargo run

# Run tests
cargo test

# Watch for changes
cargo watch -x run
```

## License

MIT
