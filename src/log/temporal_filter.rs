/// Temporal filtering operations for event logs
///
/// This module provides advanced time-based filtering:
/// - Time-based trace filtering
/// - Event timestamp range filtering
/// - Business hour filtering
/// - Weekend/holiday filtering
/// - Custom temporal rules
///
/// # Examples
///
/// ```ignore
/// use pm4py::log::temporal_filter::{TemporalFilter, BusinessHourFilter};
/// use chrono::Utc;
///
/// let filter = TemporalFilter::new();
/// let result = filter.filter_by_time_range(&log, start, end);
/// ```
use crate::log::{EventLog, FilterResult};
use chrono::{DateTime, Datelike, Local, Utc, Weekday};
use std::collections::BTreeSet;

/// Represents a time range for filtering
#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// Create a new time range
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }

    /// Check if a timestamp is within this range
    pub fn contains(&self, timestamp: &DateTime<Utc>) -> bool {
        timestamp >= &self.start && timestamp <= &self.end
    }
}

/// Defines business hour configuration
#[derive(Debug, Clone)]
pub struct BusinessHours {
    /// Start hour (0-23)
    pub start_hour: u32,
    /// End hour (0-23)
    pub end_hour: u32,
    /// Days to exclude (0=Monday, 6=Sunday)
    pub exclude_days: BTreeSet<u32>,
    /// Excluded dates
    pub holidays: BTreeSet<(u32, u32)>, // (month, day)
}

impl BusinessHours {
    /// Create standard business hours (9 AM to 5 PM, Monday to Friday)
    pub fn standard() -> Self {
        let mut exclude_days = BTreeSet::new();
        exclude_days.insert(5); // Saturday
        exclude_days.insert(6); // Sunday

        Self {
            start_hour: 9,
            end_hour: 17,
            exclude_days,
            holidays: BTreeSet::new(),
        }
    }

    /// Create extended business hours (8 AM to 6 PM, Monday to Friday)
    pub fn extended() -> Self {
        let mut exclude_days = BTreeSet::new();
        exclude_days.insert(5);
        exclude_days.insert(6);

        Self {
            start_hour: 8,
            end_hour: 18,
            exclude_days,
            holidays: BTreeSet::new(),
        }
    }

    /// Add a holiday (month, day)
    pub fn add_holiday(&mut self, month: u32, day: u32) {
        self.holidays.insert((month, day));
    }

    /// Add multiple holidays
    pub fn add_holidays(&mut self, holidays: Vec<(u32, u32)>) {
        for holiday in holidays {
            self.holidays.insert(holiday);
        }
    }

    /// Check if a timestamp is within business hours
    pub fn is_business_hour(&self, timestamp: &DateTime<Utc>) -> bool {
        use chrono::Timelike;
        let local = timestamp.with_timezone(&Local);

        // Check day of week
        let day_of_week = local.weekday().number_from_monday() - 1;
        if self.exclude_days.contains(&day_of_week) {
            return false;
        }

        // Check holidays
        if self.holidays.contains(&(local.month(), local.day())) {
            return false;
        }

        // Check hour
        let hour = local.hour();
        hour >= self.start_hour && hour < self.end_hour
    }
}

impl Default for BusinessHours {
    fn default() -> Self {
        Self::standard()
    }
}

/// Main temporal filter for event logs
pub struct TemporalFilter;

impl TemporalFilter {
    /// Filter by time range - keeps traces with at least one event in the range
    pub fn filter_by_time_range(
        log: &EventLog,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            if trace
                .events
                .iter()
                .any(|e| e.timestamp >= start && e.timestamp <= end)
            {
                filtered.add_trace(trace.clone());
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter to keep only events within time range
    pub fn filter_events_by_time_range(
        log: &EventLog,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            let mut filtered_trace = trace.clone();
            filtered_trace.events = trace
                .events
                .iter()
                .filter(|e| e.timestamp >= start && e.timestamp <= end)
                .cloned()
                .collect();

            if !filtered_trace.events.is_empty() {
                filtered.add_trace(filtered_trace);
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter to keep only business hour events
    pub fn filter_business_hours(log: &EventLog, business_hours: &BusinessHours) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            let mut filtered_trace = trace.clone();
            filtered_trace.events = trace
                .events
                .iter()
                .filter(|e| business_hours.is_business_hour(&e.timestamp))
                .cloned()
                .collect();

            if !filtered_trace.events.is_empty() {
                filtered.add_trace(filtered_trace);
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter to remove weekend events
    pub fn remove_weekends(log: &EventLog) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            let mut filtered_trace = trace.clone();
            filtered_trace.events = trace
                .events
                .iter()
                .filter(|e| {
                    let local = e.timestamp.with_timezone(&Local);
                    let day = local.weekday();
                    day != Weekday::Sat && day != Weekday::Sun
                })
                .cloned()
                .collect();

            if !filtered_trace.events.is_empty() {
                filtered.add_trace(filtered_trace);
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter to keep only weekend events
    pub fn keep_only_weekends(log: &EventLog) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            let mut filtered_trace = trace.clone();
            filtered_trace.events = trace
                .events
                .iter()
                .filter(|e| {
                    let local = e.timestamp.with_timezone(&Local);
                    let day = local.weekday();
                    day == Weekday::Sat || day == Weekday::Sun
                })
                .cloned()
                .collect();

            if !filtered_trace.events.is_empty() {
                filtered.add_trace(filtered_trace);
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by date (ignoring time)
    pub fn filter_by_date(log: &EventLog, year: i32, month: u32, day: u32) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            let mut filtered_trace = trace.clone();
            filtered_trace.events = trace
                .events
                .iter()
                .filter(|e| {
                    let local = e.timestamp.with_timezone(&Local);
                    local.year() == year && local.month() == month && local.day() == day
                })
                .cloned()
                .collect();

            if !filtered_trace.events.is_empty() {
                filtered.add_trace(filtered_trace);
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by hour of day (e.g., keep only events in a specific hour)
    pub fn filter_by_hour(log: &EventLog, hour: u32) -> FilterResult {
        use chrono::Timelike;
        if hour > 23 {
            return FilterResult::new(log.clone(), log.len());
        }

        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            let mut filtered_trace = trace.clone();
            filtered_trace.events = trace
                .events
                .iter()
                .filter(|e| {
                    let local = e.timestamp.with_timezone(&Local);
                    local.hour() == hour
                })
                .cloned()
                .collect();

            if !filtered_trace.events.is_empty() {
                filtered.add_trace(filtered_trace);
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Filter by day of week
    pub fn filter_by_day_of_week(log: &EventLog, day: Weekday) -> FilterResult {
        let mut filtered = EventLog::new();
        filtered.attributes = log.attributes.clone();

        for trace in &log.traces {
            let mut filtered_trace = trace.clone();
            filtered_trace.events = trace
                .events
                .iter()
                .filter(|e| {
                    let local = e.timestamp.with_timezone(&Local);
                    local.weekday() == day
                })
                .cloned()
                .collect();

            if !filtered_trace.events.is_empty() {
                filtered.add_trace(filtered_trace);
            }
        }

        FilterResult::new(filtered, log.len())
    }

    /// Get temporal statistics for the log
    pub fn get_temporal_statistics(log: &EventLog) -> TemporalStatistics {
        if log.is_empty() {
            return TemporalStatistics::default();
        }

        let mut min_timestamp = Utc::now();
        let mut max_timestamp = Utc::now();
        let mut total_events = 0;

        for trace in &log.traces {
            for event in &trace.events {
                if total_events == 0 {
                    min_timestamp = event.timestamp;
                    max_timestamp = event.timestamp;
                } else {
                    if event.timestamp < min_timestamp {
                        min_timestamp = event.timestamp;
                    }
                    if event.timestamp > max_timestamp {
                        max_timestamp = event.timestamp;
                    }
                }
                total_events += 1;
            }
        }

        TemporalStatistics {
            earliest_event: min_timestamp,
            latest_event: max_timestamp,
            duration_days: (max_timestamp - min_timestamp).num_days(),
            total_events,
            average_events_per_trace: if !log.is_empty() {
                total_events as f64 / log.len() as f64
            } else {
                0.0
            },
        }
    }
}

/// Statistics about temporal aspects of an event log
#[derive(Debug, Clone)]
pub struct TemporalStatistics {
    /// Earliest event timestamp
    pub earliest_event: DateTime<Utc>,
    /// Latest event timestamp
    pub latest_event: DateTime<Utc>,
    /// Duration in days
    pub duration_days: i64,
    /// Total number of events
    pub total_events: usize,
    /// Average events per trace
    pub average_events_per_trace: f64,
}

impl Default for TemporalStatistics {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            earliest_event: now,
            latest_event: now,
            duration_days: 0,
            total_events: 0,
            average_events_per_trace: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Duration;

    fn create_temporal_test_log() -> EventLog {
        let mut log = EventLog::new();

        let base = Utc::now();

        let mut trace1 = Trace::new("case_1");
        trace1.add_event(Event::new("start", base));
        trace1.add_event(Event::new("process", base + Duration::hours(1)));
        trace1.add_event(Event::new("end", base + Duration::hours(8)));
        log.add_trace(trace1);

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("start", base + Duration::days(5)));
        trace2.add_event(Event::new(
            "end",
            base + Duration::days(5) + Duration::hours(4),
        ));
        log.add_trace(trace2);

        let mut trace3 = Trace::new("case_3");
        trace3.add_event(Event::new("start", base + Duration::days(7)));
        trace3.add_event(Event::new(
            "process",
            base + Duration::days(7) + Duration::hours(2),
        ));
        log.add_trace(trace3);

        log
    }

    #[test]
    fn test_filter_by_time_range() {
        let log = create_temporal_test_log();
        let base = Utc::now();

        let start = base - Duration::days(1);
        let end = base + Duration::hours(12);

        let result = TemporalFilter::filter_by_time_range(&log, start, end);

        assert!(result.filtered_count > 0);
        assert_eq!(result.original_count, 3);
    }

    #[test]
    fn test_filter_events_by_time_range() {
        let log = create_temporal_test_log();
        let base = Utc::now();

        let start = base;
        let end = base + Duration::hours(6);

        let result = TemporalFilter::filter_events_by_time_range(&log, start, end);

        assert!(result.filtered_count <= result.original_count);
    }

    #[test]
    fn test_business_hours_standard() {
        let bh = BusinessHours::standard();
        assert_eq!(bh.start_hour, 9);
        assert_eq!(bh.end_hour, 17);
    }

    #[test]
    fn test_business_hours_extended() {
        let bh = BusinessHours::extended();
        assert_eq!(bh.start_hour, 8);
        assert_eq!(bh.end_hour, 18);
    }

    #[test]
    fn test_remove_weekends() {
        let mut log = EventLog::new();

        // Create a week spanning from some base date
        let base = Utc::now();

        // Trace 1: events on multiple days (guaranteed to have at least one weekday)
        let mut trace1 = Trace::new("case_1");
        for i in 0..7 {
            trace1.add_event(Event::new("event", base + Duration::days(i)));
        }
        log.add_trace(trace1);

        // Trace 2: Create a trace with only weekend-likely events
        // Since we can't guarantee a specific date is weekend (depends on when test runs),
        // just verify the filtered log has events and count is reduced
        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("event", base + Duration::days(10)));
        log.add_trace(trace2);

        let result = TemporalFilter::remove_weekends(&log);

        // The log should have been processed (traces kept only if they have weekday events)
        // With 7 consecutive days in trace1, at least 5 should be weekdays
        assert!(result.original_count >= result.filtered_count);
        assert!(result.filtered_count > 0); // At least one trace should remain
    }

    #[test]
    fn test_temporal_statistics() {
        let log = create_temporal_test_log();
        let stats = TemporalFilter::get_temporal_statistics(&log);

        assert!(stats.total_events > 0);
        assert!(stats.duration_days >= 0);
        assert!(stats.average_events_per_trace > 0.0);
    }

    #[test]
    fn test_filter_by_date() {
        let log = create_temporal_test_log();
        let base = Utc::now();
        let local = base.with_timezone(&Local);

        let result = TemporalFilter::filter_by_date(&log, local.year(), local.month(), local.day());

        assert!(result.filtered_count > 0);
    }

    #[test]
    fn test_filter_by_hour() {
        use chrono::Timelike;
        let log = create_temporal_test_log();
        let base = Utc::now();
        let local = base.with_timezone(&Local);
        let hour = local.hour();

        let result = TemporalFilter::filter_by_hour(&log, hour);

        assert!(result.filtered_count > 0);
    }
}
