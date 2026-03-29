//! Temporal Duration Calculation Verification
//!
//! Standalone example demonstrating proper temporal duration calculations
//! for compliance algorithms without depending on pm4py library compilation.

use chrono::{Duration, Utc};

fn ts(s: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .expect("Invalid timestamp")
        .with_timezone(&Utc)
}

fn main() {
    println!("=== Temporal Duration Calculation Test ===\n");

    // Test 1: Basic daily duration
    println!("Test 1: Daily Duration (1 day)");
    let t1 = ts("2026-03-01T00:00:00Z");
    let t2 = ts("2026-03-02T00:00:00Z");
    let daily_duration = (t2 - t1).num_seconds();
    println!("  t1: {}", t1);
    println!("  t2: {}", t2);
    println!("  Duration: {} seconds (expected: 86400)", daily_duration);
    assert_eq!(daily_duration, 86400, "Daily duration mismatch!");
    println!("  ✓ PASS\n");

    // Test 2: 90 days duration
    println!("Test 2: Quarterly Duration (90 days)");
    let q1 = ts("2026-01-01T00:00:00Z");
    let q2 = ts("2026-03-31T00:00:00Z");
    let quarterly_duration = (q2 - q1).num_seconds();
    let expected_quarterly = 90 * 86400;
    println!("  q1: {}", q1);
    println!("  q2: {}", q2);
    println!(
        "  Duration: {} seconds (expected: {})",
        quarterly_duration, expected_quarterly
    );
    assert_eq!(
        quarterly_duration, expected_quarterly,
        "Quarterly duration mismatch!"
    );
    println!("  ✓ PASS\n");

    // Test 3: Daily vs Quarterly Ratio
    println!("Test 3: Daily vs Quarterly Ratio");

    // Generate 10 daily events (9 intervals)
    let base_time = ts("2026-01-01T00:00:00Z");
    let mut daily_timestamps = vec![];
    for i in 0..10 {
        daily_timestamps.push(base_time + Duration::days(i as i64));
    }

    // Generate 4 quarterly events (3 intervals)
    let mut quarterly_timestamps = vec![];
    for i in 0..4 {
        quarterly_timestamps.push(base_time + Duration::days((i as i64) * 90));
    }

    // Calculate intervals
    let daily_intervals: Vec<i64> = (0..daily_timestamps.len() - 1)
        .map(|i| (daily_timestamps[i + 1] - daily_timestamps[i]).num_seconds())
        .collect();

    let quarterly_intervals: Vec<i64> = (0..quarterly_timestamps.len() - 1)
        .map(|i| (quarterly_timestamps[i + 1] - quarterly_timestamps[i]).num_seconds())
        .collect();

    println!(
        "  Daily timestamps: {} events, {} intervals",
        daily_timestamps.len(),
        daily_intervals.len()
    );
    println!("  Daily intervals (seconds): {:?}", daily_intervals);

    println!(
        "  Quarterly timestamps: {} events, {} intervals",
        quarterly_timestamps.len(),
        quarterly_intervals.len()
    );
    println!("  Quarterly intervals (seconds): {:?}", quarterly_intervals);

    // Calculate means
    let mean_daily = (daily_intervals.iter().sum::<i64>() as f64) / (daily_intervals.len() as f64);
    let mean_quarterly =
        (quarterly_intervals.iter().sum::<i64>() as f64) / (quarterly_intervals.len() as f64);
    let ratio = mean_quarterly / mean_daily;

    println!("\n  Mean daily interval: {} seconds", mean_daily);
    println!("  Mean quarterly interval: {} seconds", mean_quarterly);
    println!("  Ratio (quarterly / daily): {}", ratio);

    assert_eq!(mean_daily, 86400.0, "Mean daily mismatch!");
    assert_eq!(mean_quarterly, 7776000.0, "Mean quarterly mismatch!");
    assert_eq!(ratio, 90.0, "Ratio mismatch!");
    println!("  ✓ PASS\n");

    // Test 4: Detailed calculation breakdown
    println!("Test 4: Detailed Calculation Breakdown");
    println!("  Daily calculation:");
    println!(
        "    Intervals: {} × 86400 sec = {} sec total",
        daily_intervals.len(),
        daily_intervals.iter().sum::<i64>()
    );
    println!(
        "    Mean: {} / {} = {} sec/interval",
        daily_intervals.iter().sum::<i64>(),
        daily_intervals.len(),
        mean_daily
    );

    println!("  Quarterly calculation:");
    println!(
        "    Intervals: {} × 7776000 sec = {} sec total",
        quarterly_intervals.len(),
        quarterly_intervals.iter().sum::<i64>()
    );
    println!(
        "    Mean: {} / {} = {} sec/interval",
        quarterly_intervals.iter().sum::<i64>(),
        quarterly_intervals.len(),
        mean_quarterly
    );

    println!(
        "  Ratio: {} / {} = {}",
        mean_quarterly as i64, mean_daily as i64, ratio
    );
    println!("  ✓ PASS\n");

    println!("=== All Tests Passed ===");
    println!("\nSummary:");
    println!("  - Daily mean interval: 86,400 seconds (1 day)");
    println!("  - Quarterly mean interval: 7,776,000 seconds (90 days)");
    println!("  - Ratio: 90.0 (perfectly aligned)");
}
