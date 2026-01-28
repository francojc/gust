//! Graph widgets for weather data visualization.

pub mod alerts;
pub mod humidity;
pub mod precipitation;
pub mod temperature;

use chrono::{DateTime, NaiveDate, NaiveDateTime, Timelike, Utc};
use chrono_tz::Tz;
use ratatui::text::Span;

use crate::app::HourlyForecast;

/// Calculate Y-axis bounds with 10% padding above and below the data range.
pub fn calculate_bounds(values: &[f64]) -> [f64; 2] {
    if values.is_empty() {
        return [0.0, 100.0];
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let range = max - min;
    let padding = if range < 1.0 { 5.0 } else { range * 0.1 };

    [min - padding, max + padding]
}

/// Format a time as an hour label (e.g., "12am", "6pm").
pub fn format_hour(time: &NaiveDateTime) -> String {
    let hour = time.format("%l%P").to_string();
    hour.trim().to_string()
}

/// Format a time as a 24h hour label (e.g., "6", "12", "18").
pub fn format_hour_24(time: &NaiveDateTime) -> String {
    format!("{}", time.hour())
}

/// Create X-axis time labels from hourly forecast data.
/// Returns labels at 6-hour intervals to avoid crowding.
pub fn create_time_labels(hourly: &[HourlyForecast]) -> Vec<Span<'static>> {
    if hourly.is_empty() {
        return vec![];
    }

    // Select labels at 6-hour intervals (indices 0, 6, 12, 18, 24 if available)
    let indices: Vec<usize> = (0..hourly.len()).step_by(6).collect();

    indices
        .iter()
        .filter_map(|&i| hourly.get(i).map(|h| Span::raw(format_hour(&h.time))))
        .collect()
}

/// Format a date label relative to today.
/// Returns "Today" for the first point, or 3-letter day name (e.g., "Wed", "Thu").
pub fn format_date_label(date: NaiveDate, today: NaiveDate, is_first_point: bool) -> String {
    let _ = today; // Keep for potential future use
    if is_first_point {
        "Today".to_string()
    } else {
        date.format("%a").to_string()
    }
}

/// Find indices in hourly data where the day changes.
/// Returns a vector of (index, date) tuples for each day boundary.
pub fn find_day_boundaries(hourly: &[HourlyForecast]) -> Vec<(usize, NaiveDate)> {
    if hourly.is_empty() {
        return vec![];
    }

    let mut boundaries = vec![(0, hourly[0].time.date())];

    for (i, h) in hourly.iter().enumerate().skip(1) {
        let current_date = h.time.date();
        if let Some((_, last_date)) = boundaries.last() {
            if current_date != *last_date {
                boundaries.push((i, current_date));
            }
        }
    }

    boundaries
}

/// Get today's date in the specified timezone.
/// Falls back to UTC if timezone parsing fails.
pub fn get_local_today(timezone: Option<&str>) -> NaiveDate {
    match timezone.and_then(|tz| tz.parse::<Tz>().ok()) {
        Some(tz) => Utc::now().with_timezone(&tz).date_naive(),
        None => Utc::now().date_naive(),
    }
}

/// Create X-axis labels with date boundaries marked.
/// Shows date labels at day boundaries (midnight) and 24h time labels at 6-hour intervals.
/// Pattern: Today, 6, 12, 18, Wed, 6, 12, 18, Thu, ...
pub fn create_time_labels_with_dates(
    hourly: &[HourlyForecast],
    timezone: Option<&str>,
) -> Vec<String> {
    if hourly.is_empty() {
        return vec![];
    }

    let today = get_local_today(timezone);
    let boundaries = find_day_boundaries(hourly);

    // Create a map of boundary indices to their date labels (excluding index 0)
    let boundary_map: std::collections::HashMap<usize, String> = boundaries
        .iter()
        .enumerate()
        .map(|(i, (idx, date))| (*idx, format_date_label(*date, today, i == 0)))
        .collect();

    // Select labels at 6-hour intervals
    let indices: Vec<usize> = (0..hourly.len()).step_by(6).collect();

    indices
        .iter()
        .filter_map(|&i| {
            hourly.get(i).map(|h| {
                // Check if this index is at or just after a day boundary
                if let Some(label) = boundary_map.get(&i) {
                    label.clone()
                } else {
                    // Check if there's a boundary between this index and the previous one
                    let prev_idx = i.saturating_sub(6);
                    let has_boundary_between = boundaries
                        .iter()
                        .any(|(b_idx, _)| *b_idx > prev_idx && *b_idx <= i && *b_idx != 0);

                    if has_boundary_between {
                        if let Some((idx, (_, date))) = boundaries
                            .iter()
                            .enumerate()
                            .find(|(_, (b_idx, _))| *b_idx > prev_idx && *b_idx <= i && *b_idx != 0)
                        {
                            return format_date_label(*date, today, idx == 0);
                        }
                    }
                    format_hour_24(&h.time)
                }
            })
        })
        .collect()
}

/// Create X-axis labels with only day names at day boundaries.
/// Returns (labels, label_positions) where positions are x-coordinates.
pub fn create_day_labels(
    hourly: &[HourlyForecast],
    timezone: Option<&str>,
    _use_24h: bool,
) -> (Vec<String>, Vec<f64>) {
    if hourly.is_empty() {
        return (vec![], vec![]);
    }

    let today = get_local_today(timezone);
    let boundaries = find_day_boundaries(hourly);
    let total = hourly.len();

    let labels: Vec<String> = boundaries
        .iter()
        .enumerate()
        .map(|(i, (_, date))| format_date_label(*date, today, i == 0))
        .collect();

    let positions: Vec<f64> = boundaries
        .iter()
        .map(|(idx, _)| x_position(*idx, total))
        .collect();

    (labels, positions)
}

/// Get X-axis positions for 6-hour interval tick marks.
pub fn get_tick_positions(hourly: &[HourlyForecast]) -> Vec<f64> {
    (0..hourly.len())
        .step_by(6)
        .map(|i| x_position(i, hourly.len()))
        .collect()
}

/// Get X-axis positions where day boundaries occur (for rendering separator lines).
/// Returns the x-position of each midnight boundary.
pub fn get_day_boundary_positions(hourly: &[HourlyForecast]) -> Vec<f64> {
    find_day_boundaries(hourly)
        .iter()
        .skip(1) // Skip the first boundary (start of data)
        .map(|(idx, _)| x_position(*idx, hourly.len()))
        .collect()
}

/// Format a UTC DateTime to a local time string in the specified timezone.
/// Uses 12h or 24h format based on the use_24h parameter.
pub fn format_local_time(
    utc_time: DateTime<Utc>,
    timezone: Option<&str>,
    use_24h: bool,
) -> String {
    let format = if use_24h { "%H:%M" } else { "%l:%M %p" };

    match timezone.and_then(|tz| tz.parse::<Tz>().ok()) {
        Some(tz) => {
            let local = utc_time.with_timezone(&tz);
            local.format(format).to_string().trim().to_string()
        }
        None => utc_time.format(format).to_string().trim().to_string(),
    }
}

/// Calculate the X-axis position for a given index within the data range.
pub fn x_position(index: usize, total: usize) -> f64 {
    if total <= 1 {
        return 0.0;
    }
    index as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_calculate_bounds_normal() {
        let values = vec![30.0, 40.0, 50.0, 60.0, 70.0];
        let bounds = calculate_bounds(&values);
        assert!(bounds[0] < 30.0);
        assert!(bounds[1] > 70.0);
    }

    #[test]
    fn test_calculate_bounds_empty() {
        let values: Vec<f64> = vec![];
        let bounds = calculate_bounds(&values);
        assert_eq!(bounds, [0.0, 100.0]);
    }

    #[test]
    fn test_calculate_bounds_small_range() {
        let values = vec![50.0, 50.5, 51.0];
        let bounds = calculate_bounds(&values);
        // With small range, should use fixed padding of 5.0
        assert!(bounds[0] < 50.0);
        assert!(bounds[1] > 51.0);
    }

    #[test]
    fn test_format_hour() {
        let time = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap();
        let formatted = format_hour(&time);
        assert_eq!(formatted, "2pm");
    }

    #[test]
    fn test_format_hour_midnight() {
        let time = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let formatted = format_hour(&time);
        assert_eq!(formatted, "12am");
    }

    #[test]
    fn test_create_time_labels_empty() {
        let hourly: Vec<HourlyForecast> = vec![];
        let labels = create_time_labels(&hourly);
        assert!(labels.is_empty());
    }

    #[test]
    fn test_x_position() {
        assert_eq!(x_position(0, 24), 0.0);
        assert_eq!(x_position(12, 24), 12.0);
        assert_eq!(x_position(23, 24), 23.0);
    }

    #[test]
    fn test_format_date_label_first_point() {
        let today = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        // First point should show "Today" regardless of date
        assert_eq!(format_date_label(today, today, true), "Today");
    }

    #[test]
    fn test_format_date_label_not_first_point() {
        let today = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let tomorrow = NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();
        // Non-first points show 3-letter day name
        assert_eq!(format_date_label(tomorrow, today, false), "Tue");
    }

    #[test]
    fn test_format_date_label_future() {
        let today = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let future = NaiveDate::from_ymd_opt(2024, 1, 18).unwrap();
        // Non-first points show only 3-letter day name
        assert_eq!(format_date_label(future, today, false), "Thu");
    }

    #[test]
    fn test_create_day_labels() {
        use crate::app::PrecipType;
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let hourly: Vec<HourlyForecast> = (0..48)
            .map(|i| {
                let day_offset = i / 24;
                let hour = i % 24;
                let date = base_date
                    .checked_add_days(chrono::Days::new(day_offset as u64))
                    .unwrap();
                HourlyForecast {
                    time: date.and_hms_opt(hour as u32, 0, 0).unwrap(),
                    temperature: 42.0,
                    precipitation_probability: 10,
                    humidity: 60,
                    wind_speed: 5.0,
                    precip_type: PrecipType::None,
                    uv_index: 0.0,
                }
            })
            .collect();

        let (labels, positions) = create_day_labels(&hourly, None, false);
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0], "Today"); // First point
        assert_eq!(labels[1], "Tue"); // Second day boundary
        assert_eq!(positions[0], 0.0);
        assert_eq!(positions[1], 24.0);
    }

    #[test]
    fn test_get_tick_positions() {
        use crate::app::PrecipType;
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let hourly: Vec<HourlyForecast> = (0..24)
            .map(|i| HourlyForecast {
                time: base_date.and_hms_opt(i as u32, 0, 0).unwrap(),
                temperature: 42.0,
                precipitation_probability: 10,
                humidity: 60,
                wind_speed: 5.0,
                precip_type: PrecipType::None,
                uv_index: 0.0,
            })
            .collect();

        let positions = get_tick_positions(&hourly);
        assert_eq!(positions.len(), 4); // indices 0, 6, 12, 18
        assert_eq!(positions[0], 0.0);
        assert_eq!(positions[1], 6.0);
        assert_eq!(positions[2], 12.0);
        assert_eq!(positions[3], 18.0);
    }

    #[test]
    fn test_format_hour_24() {
        let time = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(14, 0, 0)
            .unwrap();
        let formatted = format_hour_24(&time);
        assert_eq!(formatted, "14");
    }

    #[test]
    fn test_format_hour_24_midnight() {
        let time = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let formatted = format_hour_24(&time);
        assert_eq!(formatted, "0");
    }

    #[test]
    fn test_format_hour_24_six() {
        let time = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(6, 0, 0)
            .unwrap();
        let formatted = format_hour_24(&time);
        assert_eq!(formatted, "6");
    }

    #[test]
    fn test_get_day_boundary_positions() {
        use crate::app::PrecipType;
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let hourly: Vec<HourlyForecast> = (0..48)
            .map(|i| {
                let day_offset = i / 24;
                let hour = i % 24;
                let date = base_date
                    .checked_add_days(chrono::Days::new(day_offset as u64))
                    .unwrap();
                HourlyForecast {
                    time: date.and_hms_opt(hour as u32, 0, 0).unwrap(),
                    temperature: 42.0,
                    precipitation_probability: 10,
                    humidity: 60,
                    wind_speed: 5.0,
                    precip_type: PrecipType::None,
                    uv_index: 0.0,
                }
            })
            .collect();

        let positions = get_day_boundary_positions(&hourly);
        // Should skip the first boundary (index 0) and return position for index 24
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], 24.0);
    }

    #[test]
    fn test_find_day_boundaries() {
        use crate::app::PrecipType;
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let hourly: Vec<HourlyForecast> = (0..48)
            .map(|i| {
                let day_offset = i / 24;
                let hour = i % 24;
                let date = base_date
                    .checked_add_days(chrono::Days::new(day_offset as u64))
                    .unwrap();
                HourlyForecast {
                    time: date.and_hms_opt(hour as u32, 0, 0).unwrap(),
                    temperature: 42.0,
                    precipitation_probability: 10,
                    humidity: 60,
                    wind_speed: 5.0,
                    precip_type: PrecipType::None,
                    uv_index: 0.0,
                }
            })
            .collect();

        let boundaries = find_day_boundaries(&hourly);
        assert_eq!(boundaries.len(), 2);
        assert_eq!(boundaries[0].0, 0);
        assert_eq!(boundaries[0].1, base_date);
        assert_eq!(boundaries[1].0, 24);
    }

    #[test]
    fn test_find_day_boundaries_empty() {
        let hourly: Vec<HourlyForecast> = vec![];
        let boundaries = find_day_boundaries(&hourly);
        assert!(boundaries.is_empty());
    }

    #[test]
    fn test_format_local_time_12h() {
        let utc_time = DateTime::parse_from_rfc3339("2024-01-15T18:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let result = format_local_time(utc_time, Some("America/New_York"), false);
        assert_eq!(result, "1:30 PM");
    }

    #[test]
    fn test_format_local_time_24h() {
        let utc_time = DateTime::parse_from_rfc3339("2024-01-15T18:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let result = format_local_time(utc_time, Some("America/New_York"), true);
        assert_eq!(result, "13:30");
    }

    #[test]
    fn test_format_local_time_invalid_tz() {
        let utc_time = DateTime::parse_from_rfc3339("2024-01-15T18:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let result = format_local_time(utc_time, Some("Invalid/Timezone"), false);
        assert_eq!(result, "6:30 PM");
    }

    #[test]
    fn test_format_local_time_none_tz() {
        let utc_time = DateTime::parse_from_rfc3339("2024-01-15T18:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let result = format_local_time(utc_time, None, false);
        assert_eq!(result, "6:30 PM");
    }
}
