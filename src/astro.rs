//! Astronomical calculations for moon phase and related data.

use chrono::NaiveDate;

/// Moon phase enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonPhase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl MoonPhase {
    /// Get the emoji icon for this moon phase.
    pub fn icon(&self) -> &'static str {
        match self {
            MoonPhase::New => "\u{1F311}",           // New Moon
            MoonPhase::WaxingCrescent => "\u{1F312}", // Waxing Crescent
            MoonPhase::FirstQuarter => "\u{1F313}",  // First Quarter
            MoonPhase::WaxingGibbous => "\u{1F314}", // Waxing Gibbous
            MoonPhase::Full => "\u{1F315}",          // Full Moon
            MoonPhase::WaningGibbous => "\u{1F316}", // Waning Gibbous
            MoonPhase::LastQuarter => "\u{1F317}",   // Last Quarter
            MoonPhase::WaningCrescent => "\u{1F318}", // Waning Crescent
        }
    }

    /// Get the human-readable name for this moon phase.
    pub fn name(&self) -> &'static str {
        match self {
            MoonPhase::New => "New Moon",
            MoonPhase::WaxingCrescent => "Waxing Crescent",
            MoonPhase::FirstQuarter => "First Quarter",
            MoonPhase::WaxingGibbous => "Waxing Gibbous",
            MoonPhase::Full => "Full Moon",
            MoonPhase::WaningGibbous => "Waning Gibbous",
            MoonPhase::LastQuarter => "Last Quarter",
            MoonPhase::WaningCrescent => "Waning Crescent",
        }
    }
}

/// Calculate the moon phase for a given date.
///
/// Uses a simple synodic month calculation (29.53 days).
/// Reference new moon: January 11, 2024.
pub fn moon_phase(date: NaiveDate) -> MoonPhase {
    // Known new moon: Jan 11, 2024
    let known_new = NaiveDate::from_ymd_opt(2024, 1, 11).unwrap();
    let days_since = (date - known_new).num_days();
    let synodic_month = 29.53;
    let phase_day = ((days_since as f64 % synodic_month) + synodic_month) % synodic_month;

    match phase_day as u8 {
        0..=1 => MoonPhase::New,
        2..=6 => MoonPhase::WaxingCrescent,
        7..=8 => MoonPhase::FirstQuarter,
        9..=13 => MoonPhase::WaxingGibbous,
        14..=16 => MoonPhase::Full,
        17..=21 => MoonPhase::WaningGibbous,
        22..=23 => MoonPhase::LastQuarter,
        _ => MoonPhase::WaningCrescent,
    }
}

/// Format daylight duration in seconds as hours and minutes.
pub fn format_daylight_duration(seconds: f64) -> String {
    let total_minutes = (seconds / 60.0).round() as u32;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{}h {}m", hours, minutes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moon_phase_new() {
        // Jan 11, 2024 was a new moon
        let date = NaiveDate::from_ymd_opt(2024, 1, 11).unwrap();
        assert_eq!(moon_phase(date), MoonPhase::New);
    }

    #[test]
    fn test_moon_phase_full() {
        // ~14-15 days after new moon
        let date = NaiveDate::from_ymd_opt(2024, 1, 25).unwrap();
        assert_eq!(moon_phase(date), MoonPhase::Full);
    }

    #[test]
    fn test_moon_phase_first_quarter() {
        // ~7 days after new moon
        let date = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();
        assert_eq!(moon_phase(date), MoonPhase::FirstQuarter);
    }

    #[test]
    fn test_moon_phase_icon() {
        assert_eq!(MoonPhase::New.icon(), "\u{1F311}");
        assert_eq!(MoonPhase::Full.icon(), "\u{1F315}");
    }

    #[test]
    fn test_moon_phase_name() {
        assert_eq!(MoonPhase::New.name(), "New Moon");
        assert_eq!(MoonPhase::Full.name(), "Full Moon");
        assert_eq!(MoonPhase::FirstQuarter.name(), "First Quarter");
    }

    #[test]
    fn test_format_daylight_duration() {
        assert_eq!(format_daylight_duration(36000.0), "10h 0m");
        assert_eq!(format_daylight_duration(36900.0), "10h 15m");
        assert_eq!(format_daylight_duration(43200.0), "12h 0m");
    }
}
