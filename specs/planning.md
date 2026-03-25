# Development Project Planning

**Project:** gust
**Status:** Active Development
**Last Updated:** 2026-03-24

## Project Overview

### Software Description

- **Application Type:** CLI tool / TUI application
- **Target Platform:** macOS, Linux (cross-platform terminal)
- **Primary Language:** Rust (edition 2021)
- **Key Libraries/Frameworks:** ratatui 0.30, crossterm 0.28, tokio
  1.x, reqwest 0.12, clap 4, chrono 0.4, insta 1

### Problem Statement

- Terminal weather apps tend to be either data-dense but ugly, or
  visually polished but low on information. gust aims for both.
- Target users: developers and power users who live in the terminal
  and want weather at a glance without leaving it.
- Existing tools (wttr.in, wego, weather-rs) lack Braille-based
  high-resolution graphing and the Vim-style interaction model.

### Goals and Non-Goals

#### Goals

- [x] Real-time weather fetch from Open-Meteo (no API key)
- [x] Braille-based graphs for temperature, precipitation, humidity
- [x] Vim-style keyboard navigation
- [x] Imperial/metric unit switching
- [x] Disk-based caching with configurable TTL
- [x] Weather alerts via NWS (US only)
- [x] Air quality index display
- [x] Astronomical data (sunrise/sunset)
- [ ] Nix package publish (`nix run github:francojc/gust`)
- [ ] Multi-location comparison view
- [ ] Configurable graph time window

#### Non-Goals

- GUI or web frontend
- Push notifications
- Paid API integration
- Windows support (crossterm supports it but not a priority)

## Architecture and Design

### High-Level Architecture

- **Pattern:** Single-process TUI with async data layer
- **Data Flow:** CLI args / keypress → `AppState` update → tokio
  channel fetch → `AppState` update → ratatui render loop
- **Key Components:** `app` (state machine), `api` (data clients),
  `cache` (disk), `config` (TOML settings), `ui` (render modules)

### External Dependencies

- **APIs:** Open-Meteo forecast, Open-Meteo geocoding, NWS alerts
- **Data Sources:** HTTP endpoints (no authentication required)
- **Build Tools:** Cargo, Nix flake with rust-overlay for stable
  Rust toolchain

### Technical Constraints

- Terminal must support Unicode/Braille (Alacritty, Kitty, WezTerm,
  iTerm2 confirmed)
- No API keys required — all data sources are public
- Cache stored in user data dir (`dirs` crate for XDG compliance)

## Timeline and Milestones

### Phase 1: Foundation (complete)

- [x] Project scaffolding (Cargo, flake.nix, .gitignore)
- [x] Basic TUI loop with ratatui + crossterm
- [x] Open-Meteo API client
- [x] Disk cache with stale-data fallback
- [x] TOML configuration loading

### Phase 2: Core Features (complete)

- [x] Braille graphs for temperature, precipitation, humidity
- [x] Timezone support and day-boundary rendering on graph X-axes
- [x] 6-hour tick marks and day labels
- [x] Vim-style key bindings and view switching
- [x] Location search via geocoding API
- [x] Weather alerts display (NWS)
- [x] Air quality index (Open-Meteo AQI endpoint)
- [x] Astronomical data (sunrise/sunset)

### Phase 3: Polish and Release (current)

- [ ] Snapshot test coverage for graph render modules
- [ ] CLAUDE.md, specs/, and project documentation
- [ ] Nix package attribute for `nix run` install path
- [ ] v0.1.0 release tag and changelog

### Phase 4: Future Enhancements

- [ ] Multi-location tabs or split view
- [ ] Configurable graph time window (24h / 48h / 7-day)
- [ ] Hourly detail popup on cursor hover

## Resources and Requirements

### Development Environment

- Rust stable (managed via rust-overlay in flake.nix)
- Cargo with cargo-watch, cargo-edit, cargo-insta
- Nix + direnv for environment activation

### Collaboration

- Solo project; no external contributors at this stage
- Feature branches, conventional commits, squash merges to main

## Risk Assessment

### Technical Risks

- **API availability:** Open-Meteo and NWS are public/free but have
  no SLA. Mitigation: disk cache with stale-data fallback.
- **Terminal compatibility:** Braille rendering depends on font
  support. Mitigation: document tested terminals; graceful fallback
  if characters don't render.
- **Snapshot drift:** insta snapshots can silently diverge from UI
  changes. Mitigation: run `cargo insta review` in CI or pre-commit.

### Scope Risks

- Feature creep into GUI or multi-pane layouts before core is
  stable. Guardrail: defer Phase 4 items until v0.1.0 ships.

## Success Metrics

### Functional Criteria

- [ ] All three graph views render correctly for arbitrary lat/lon
- [ ] Cache hit avoids network on relaunch within TTL
- [ ] Alerts view displays NWS data for US locations

### Quality Criteria

- [ ] Snapshot tests cover graph render functions
- [ ] `cargo clippy` reports zero warnings
- [ ] `cargo fmt --check` passes in CI

### Adoption Criteria

- [ ] `nix run github:francojc/gust` installs and launches
- [ ] README accurately reflects current feature set
