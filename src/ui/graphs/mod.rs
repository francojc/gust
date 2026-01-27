//! Graph widgets for weather data visualization.

pub mod alerts;
pub mod humidity;
pub mod precipitation;
pub mod temperature;

use chrono::NaiveDateTime;
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
}
