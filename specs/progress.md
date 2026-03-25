# Development Project Progress

**Project:** gust
**Status:** Active — Phase 3 (Polish and Release)
**Last Updated:** 2026-03-24

## Current Status Overview

### Development Phase

- **Current Phase:** Polish and Release
- **Phase Progress:** 20% complete
- **Overall Project Progress:** 75% complete

### Recent Accomplishments

- Weather alerts display via NWS API — 2026-03-24
- Air quality index fetch from Open-Meteo AQI endpoint — 2026-03-24
- Astronomical data (sunrise/sunset) module — 2026-03-24
- 6-hour tick marks and day labels on graph X-axes — 2026-03-24
- Timezone support and day-boundary rendering in graphs — 2026-03-24
- Braille graphs for temperature, precipitation, humidity — 2026-03-24

### Active Work

- [ ] Snapshot test coverage for graph render modules
- [ ] Nix package attribute for `nix run github:francojc/gust`
- [ ] v0.1.0 release tag and changelog

## Milestone Tracking

### Completed Milestones

- [x] ~~Project scaffolding and dev environment~~ — 2026-03-24
- [x] ~~Open-Meteo API client with caching~~ — 2026-03-24
- [x] ~~Braille graph rendering (temp, precip, humidity)~~ — 2026-03-24
- [x] ~~Timezone + day-boundary graph support~~ — 2026-03-24
- [x] ~~Weather alerts, AQI, astronomical data~~ — 2026-03-24

### Upcoming Milestones

- [ ] Snapshot tests passing — Target: 2026-04-07
- [ ] Nix flake package output — Target: 2026-04-14
- [ ] v0.1.0 release — Target: 2026-04-14

### At-Risk Milestones

None currently.

## Build and Test Status

### Build Health

- **Last Successful Build:** 2026-03-24 (b688ad2)
- **Build Warnings:** unknown (run `cargo build` to check)

### Test Results

- **Unit/Snapshot Tests:** insta framework in place; coverage TBD
- **Test Coverage:** not yet measured

### Open Defects

- **Critical:** 0
- **High:** 0
- **Medium:** 0
- **Low:** 0 (known: NWS 404 for non-US coords — handled silently)

## Feature Progress

### Completed Features

- [x] ~~Open-Meteo forecast fetch~~ — 2026-03-24
- [x] ~~Geocoding location search~~ — 2026-03-24
- [x] ~~Disk cache with stale fallback~~ — 2026-03-24
- [x] ~~TOML configuration loading~~ — 2026-03-24
- [x] ~~Braille temperature graph~~ — 2026-03-24
- [x] ~~Braille precipitation graph~~ — 2026-03-24
- [x] ~~Braille humidity graph~~ — 2026-03-24
- [x] ~~Graph X-axis with 6h ticks and day labels~~ — 2026-03-24
- [x] ~~Timezone-aware rendering with day boundaries~~ — 2026-03-24
- [x] ~~Weather alerts view (NWS)~~ — 2026-03-24
- [x] ~~Air quality index (Open-Meteo)~~ — 2026-03-24
- [x] ~~Astronomical data (sunrise/sunset)~~ — 2026-03-24
- [x] ~~Vim-style key bindings~~ — 2026-03-24
- [x] ~~Imperial/metric unit switching~~ — 2026-03-24

### In Progress

- [ ] Snapshot tests for graph render modules — 0% complete
  - insta is in dev-dependencies; test structure to be added to
    graph modules

### Planned

- [ ] Nix flake package output — Phase 3
- [ ] Changelog and release notes — Phase 3
- [ ] Multi-location comparison view — Phase 4 (deferred)
- [ ] Configurable graph time window — Phase 4 (deferred)

### Deferred or Cut

- Multi-location tabs: deferred to Phase 4 after v0.1.0 ships
- Hourly detail popup: deferred to Phase 4

## Technical Debt

### Known Debt

1. **Snapshot test coverage**
   - **Impact:** UI regressions in graph rendering may go undetected
   - **Effort to Resolve:** Medium — insta is wired up, tests need
     writing per graph module
   - **Priority:** High (blocks v0.1.0)

2. **Nix flake package output missing**
   - **Impact:** `nix run github:francojc/gust` does not work yet
   - **Effort to Resolve:** Low — add `packages.default` to flake.nix
   - **Priority:** High (release goal)

3. **AQI always fetches with Imperial units**
   - **Impact:** Cosmetic only; AQI is unit-independent
   - **Effort to Resolve:** Low — pass units from config or use a
     dedicated unit-neutral client
   - **Priority:** Low

## Dependency Status

### External Dependencies

- **ratatui 0.30:** current stable, no concerns
- **crossterm 0.28:** current stable, no concerns
- **tokio 1.x:** current stable, no concerns
- **reqwest 0.12:** current stable, no concerns
- **insta 1:** current stable, snapshot testing

### Pending Updates

None known.

## Challenges and Blockers

### Current Blockers

None.

### Resolved Challenges

- Timezone handling for graph X-axis: resolved by passing tz offset
  to rendering logic and using chrono-tz — 2026-03-24
- NWS 404 for non-US locations: resolved by treating 404 as empty
  alerts rather than an error — 2026-03-24

### Lessons Learned

- Cargo clean resolves stale build artifacts after Nix toolchain
  changes more reliably than targeted rebuilds
- insta snapshot tests need `cargo insta review` after any UI
  change; this should be part of the development workflow

## Next Steps

### Immediate Actions (Next 2 Weeks)

- [ ] Write snapshot tests for temperature, precipitation, and
  humidity graph render functions
- [ ] Add `packages.default` output to `flake.nix` for `nix run`
- [ ] Draft CHANGELOG.md and tag v0.1.0

### Medium-term Goals (Next Month)

- [ ] Publish to crates.io (optional) and verify `nix run` path
- [ ] Begin Phase 4 design for multi-location and configurable
  time-window features

### Decisions Needed

- Whether to publish to crates.io alongside the Nix package path

## Release Planning

### Next Release

- **Version:** v0.1.0
- **Target Date:** 2026-04-14
- **Included Features:** all Phase 1 and Phase 2 features listed
  above; snapshot tests; Nix package output
- **Release Blockers:** snapshot test coverage, Nix flake package

### Release History

| Version | Date | Key Changes |
|---------|------|-------------|
| unreleased | — | Initial feature-complete build |
