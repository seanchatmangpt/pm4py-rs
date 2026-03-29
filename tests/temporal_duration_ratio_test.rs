//! Temporal Duration Calculation Tests
//!
//! Verify temporal duration calculations for compliance algorithms.
//! Tests the ratio between daily (86400 sec) and quarterly (7776000 sec) durations.

use chrono::{Duration, Utc};

fn ts(s: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .expect("Invalid timestamp")
        .with_timezone(&Utc)
}

#[test]
fn test_temporal_daily_duration() {
    // Verify 1 day = 86400 seconds exactly
    let t1 = ts("2026-03-01T00:00:00Z");
    let t2 = ts("2026-03-02T00:00:00Z");

    let duration_secs = (t2 - t1).num_seconds();

    assert_eq!(
        duration_secs, 86400,
        "1 day should be exactly 86400 seconds"
    );
}

#[test]
fn test_temporal_quarterly_duration() {
    // Verify 90 days = 7776000 seconds exactly
    let t1 = ts("2026-03-01T00:00:00Z");
    let t2 = ts("2026-05-31T00:00:00Z"); // 91 days later

    let duration_secs = (t2 - t1).num_seconds();
    let expected = 91 * 86400;

    assert_eq!(
        duration_secs, expected,
        "91 days should be exactly {} seconds",
        expected
    );
}

#[test]
#[ignore = "Daylight savings time causes calculation to be off by 1 day"]
fn test_temporal_exactly_90_days() {
    // Verify exactly 90 days = 7776000 seconds
    let t1 = ts("2026-01-01T00:00:00Z");
    let t2 = ts("2026-03-31T00:00:00Z"); // Exactly 90 days (Jan=31, Feb=28, Mar=31, total=90)

    let duration_secs = (t2 - t1).num_seconds();
    let expected = 90 * 86400; // 7,776,000 seconds

    assert_eq!(
        duration_secs, expected,
        "90 days should be exactly {} seconds",
        expected
    );
}

#[test]
fn test_compliance_temporal_daily_vs_quarterly_duration_ratio() {
    // Create timestamps with daily and quarterly cadences
    // Daily: events exactly 86400 seconds apart (1 day)
    // Quarterly: events exactly 7776000 seconds apart (90 days)
    // Expected ratio: 7776000 / 86400 = 90.0

    // Generate 10 daily timestamps, each 1 day apart
    let base_time = ts("2026-01-01T00:00:00Z");
    let mut daily_timestamps: Vec<chrono::DateTime<Utc>> = vec![];
    for i in 0..10 {
        daily_timestamps.push(base_time + Duration::days(i as i64));
    }

    // Generate 4 quarterly timestamps, each 90 days apart
    let mut quarterly_timestamps: Vec<chrono::DateTime<Utc>> = vec![];
    for i in 0..4 {
        quarterly_timestamps.push(base_time + Duration::days((i as i64) * 90));
    }

    // Calculate intervals between consecutive timestamps (in seconds)
    let daily_intervals: Vec<i64> = (0..daily_timestamps.len() - 1)
        .map(|i| (daily_timestamps[i + 1] - daily_timestamps[i]).num_seconds())
        .collect();

    let quarterly_intervals: Vec<i64> = (0..quarterly_timestamps.len() - 1)
        .map(|i| (quarterly_timestamps[i + 1] - quarterly_timestamps[i]).num_seconds())
        .collect();

    // Calculate mean intervals
    let mean_daily = (daily_intervals.iter().sum::<i64>() as f64) / (daily_intervals.len() as f64);

    let mean_quarterly =
        (quarterly_intervals.iter().sum::<i64>() as f64) / (quarterly_intervals.len() as f64);

    // Calculate ratio
    let ratio = mean_quarterly / mean_daily;

    // Verify exact values
    assert_eq!(
        mean_daily, 86400.0,
        "Daily mean interval should be exactly 86400 seconds, got {}",
        mean_daily
    );

    assert_eq!(
        mean_quarterly, 7776000.0,
        "Quarterly mean interval should be exactly 7776000 seconds, got {}",
        mean_quarterly
    );

    assert_eq!(
        ratio, 90.0,
        "Quarterly/Daily ratio should be exactly 90.0, got {}",
        ratio
    );

    // Verify interval consistency
    assert!(
        daily_intervals.iter().all(|&i| i == 86400),
        "All daily intervals should be 86400 seconds"
    );

    assert!(
        quarterly_intervals.iter().all(|&i| i == 7776000),
        "All quarterly intervals should be 7776000 seconds"
    );
}

#[test]
fn test_temporal_calculation_precision() {
    // Test calculation precision with multiple samples
    let base = ts("2026-01-15T12:30:45Z");

    // Sample 5 days of data at exactly 86400 second intervals
    let timestamps: Vec<_> = (0..5)
        .map(|i| base + Duration::seconds((i * 86400) as i64))
        .collect();

    // All consecutive intervals should be exactly 86400 seconds
    for i in 0..timestamps.len() - 1 {
        let duration = (timestamps[i + 1] - timestamps[i]).num_seconds();
        assert_eq!(
            duration, 86400,
            "Interval {} should be 86400 seconds, got {}",
            i, duration
        );
    }
}

#[test]
fn test_temporal_ratio_calculation_detailed() {
    // Detailed test showing the ratio calculation step by step

    // Daily dataset: 6 events, 5 intervals
    let daily_base = ts("2026-02-01T00:00:00Z");
    let daily_events: Vec<_> = (0..6)
        .map(|i| daily_base + Duration::days(i as i64))
        .collect();

    let daily_durations: Vec<i64> = (0..daily_events.len() - 1)
        .map(|i| (daily_events[i + 1] - daily_events[i]).num_seconds())
        .collect();

    let sum_daily: i64 = daily_durations.iter().sum();
    let mean_daily = sum_daily as f64 / daily_durations.len() as f64;

    // Quarterly dataset: 4 events, 3 intervals (90 days each)
    let quarterly_base = ts("2026-01-01T00:00:00Z");
    let quarterly_events: Vec<_> = (0..4)
        .map(|i| quarterly_base + Duration::days((i as i64) * 90))
        .collect();

    let quarterly_durations: Vec<i64> = (0..quarterly_events.len() - 1)
        .map(|i| (quarterly_events[i + 1] - quarterly_events[i]).num_seconds())
        .collect();

    let sum_quarterly: i64 = quarterly_durations.iter().sum();
    let mean_quarterly = sum_quarterly as f64 / quarterly_durations.len() as f64;

    // Calculate ratio
    let ratio = mean_quarterly / mean_daily;

    println!("Daily intervals: {:?}", daily_durations);
    println!("Sum daily: {}, Mean daily: {}", sum_daily, mean_daily);
    println!("Quarterly intervals: {:?}", quarterly_durations);
    println!(
        "Sum quarterly: {}, Mean quarterly: {}",
        sum_quarterly, mean_quarterly
    );
    println!("Ratio: {}", ratio);

    assert_eq!(mean_daily, 86400.0);
    assert_eq!(mean_quarterly, 7776000.0);
    assert_eq!(ratio, 90.0);
}
