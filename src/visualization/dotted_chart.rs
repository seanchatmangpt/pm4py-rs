//! Dotted Chart Visualization

use crate::log::{EventLog, Trace};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Duration is used in tests
#[allow(unused_imports)]
use chrono::Duration;

/// Options for dotted chart visualization
#[derive(Debug, Clone)]
pub struct DottedChartOptions {
    pub width: usize,
    pub height: usize,
    pub dot_radius: f64,
    pub show_resources: bool,
    pub show_durations: bool,
    pub show_heatmap: bool,
    pub highlight_anomalies: bool,
    pub anomaly_threshold: f64,
}

impl DottedChartOptions {
    pub fn new() -> Self {
        Self {
            width: 1400,
            height: 600,
            dot_radius: 3.0,
            show_resources: true,
            show_durations: true,
            show_heatmap: true,
            highlight_anomalies: true,
            anomaly_threshold: 2.0,
        }
    }

    pub fn with_resources(mut self, enable: bool) -> Self {
        self.show_resources = enable;
        self
    }

    pub fn with_durations(mut self, enable: bool) -> Self {
        self.show_durations = enable;
        self
    }

    pub fn with_heatmap(mut self, enable: bool) -> Self {
        self.show_heatmap = enable;
        self
    }

    pub fn with_anomalies(mut self, enable: bool) -> Self {
        self.highlight_anomalies = enable;
        self
    }
}

impl Default for DottedChartOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DottedChartDot {
    x: f64,
    y: f64,
    activity: String,
    resource: Option<String>,
    color: String,
    is_anomaly: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TraceStats {
    duration_hours: f64,
    event_count: usize,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DottedChart {
    dots: Vec<DottedChartDot>,
    options: DottedChartOptions,
    min_time: Option<DateTime<Utc>>,
    max_time: Option<DateTime<Utc>>,
    trace_stats: Vec<TraceStats>,
    activity_colors: HashMap<String, String>,
}

impl DottedChart {
    pub fn new(options: DottedChartOptions) -> Self {
        Self {
            dots: Vec::new(),
            options,
            min_time: None,
            max_time: None,
            trace_stats: Vec::new(),
            activity_colors: HashMap::new(),
        }
    }

    fn add_dot(
        &mut self,
        x: f64,
        y: f64,
        activity: impl Into<String>,
        resource: Option<String>,
        is_anomaly: bool,
    ) {
        let activity_str = activity.into();
        let color = self
            .activity_colors
            .get(&activity_str)
            .cloned()
            .unwrap_or_else(|| self.generate_activity_color(&activity_str));
        self.dots.push(DottedChartDot {
            x,
            y,
            activity: activity_str,
            resource,
            color,
            is_anomaly,
        });
    }

    fn generate_activity_color(&mut self, activity: &str) -> String {
        let hash = activity
            .bytes()
            .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        let hue = (hash % 360) as f64;
        let color = format!("hsl({:.0}, 70%, 50%)", hue);
        self.activity_colors
            .insert(activity.to_string(), color.clone());
        color
    }

    fn calculate_trace_stats(&self, trace: &Trace) -> TraceStats {
        if trace.events.is_empty() {
            return TraceStats {
                duration_hours: 0.0,
                event_count: 0,
                start_time: Utc::now(),
                end_time: Utc::now(),
            };
        }
        let start_time = trace.events[0].timestamp;
        let end_time = trace.events[trace.events.len() - 1].timestamp;
        let duration = end_time.signed_duration_since(start_time);
        TraceStats {
            duration_hours: duration.num_seconds() as f64 / 3600.0,
            event_count: trace.events.len(),
            start_time,
            end_time,
        }
    }

    fn detect_anomalies(durations: &[f64], threshold: f64) -> Vec<bool> {
        if durations.is_empty() {
            return Vec::new();
        }
        let mean = durations.iter().sum::<f64>() / durations.len() as f64;
        let variance =
            durations.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / durations.len() as f64;
        let std_dev = variance.sqrt();
        if std_dev == 0.0 {
            return vec![false; durations.len()];
        }
        durations
            .iter()
            .map(|d| ((d - mean).abs() / std_dev) > threshold)
            .collect()
    }

    pub fn generate_svg(&self) -> String {
        let mut svg = String::new();
        svg.push_str(&format!(
            "<svg xmlns='http://www.w3.org/2000/svg' width='{}' height='{}' viewBox='0 0 {} {}'>",
            self.options.width, self.options.height, self.options.width, self.options.height
        ));
        svg.push_str("<defs><style>");
        svg.push_str(".dotted-chart-dot { cursor: pointer; }");
        svg.push_str(".dotted-chart-label { font-size: 12px; fill: black; }");
        svg.push_str(".dotted-chart-grid { stroke: #EEE; stroke-width: 0.5; }");
        svg.push_str(".dotted-chart-axis { stroke: black; stroke-width: 1; }");
        svg.push_str("</style></defs>");
        svg.push_str(&format!(
            "<rect width='{}' height='{}' fill='white' stroke='black' stroke-width='2'/>",
            self.options.width, self.options.height
        ));

        let grid_spacing = 100;
        for x in (0..self.options.width).step_by(grid_spacing) {
            svg.push_str(&format!(
                "<line x1='{}' y1='0' x2='{}' y2='{}' class='dotted-chart-grid'/>",
                x, x, self.options.height
            ));
        }
        for y in (0..self.options.height).step_by(grid_spacing) {
            svg.push_str(&format!(
                "<line x1='0' y1='{}' x2='{}' y2='{}' class='dotted-chart-grid'/>",
                y, self.options.width, y
            ));
        }

        svg.push_str(&format!(
            "<line x1='40' y1='0' x2='40' y2='{}' class='dotted-chart-axis'/>",
            self.options.height
        ));
        svg.push_str(&format!(
            "<line x1='40' y1='{}' x2='{}' y2='{}' class='dotted-chart-axis'/>",
            self.options.height - 40,
            self.options.width,
            self.options.height - 40
        ));

        for dot in &self.dots {
            svg.push_str(&format!(
                "<circle cx='{:.1}' cy='{:.1}' r='{}' fill='{}' class='dotted-chart-dot'/>",
                dot.x, dot.y, self.options.dot_radius, dot.color
            ));
        }
        svg.push_str("</svg>");
        svg
    }

    pub fn generate_heatmap_overlay(&self) -> String {
        let mut heatmap = String::new();
        let mut resource_load: HashMap<String, usize> = HashMap::new();
        for dot in &self.dots {
            if let Some(ref resource) = dot.resource {
                *resource_load.entry(resource.clone()).or_insert(0) += 1;
            }
        }
        let max_load = *resource_load.values().max().unwrap_or(&1);
        heatmap.push_str("<svg xmlns='http://www.w3.org/2000/svg' width='100%' height='100%'>");
        heatmap.push_str("<defs><style>.heatmap-cell { cursor: pointer; }</style></defs>");
        let mut y_pos = 20.0;
        for (resource, load) in &resource_load {
            let intensity = (*load as f64 / max_load as f64) * 100.0;
            let color = format!("hsl(0, 100%, {:.0}%)", 100.0 - intensity);
            heatmap.push_str(&format!(
                "<rect x='50' y='{:.1}' width='200' height='20' fill='{}' class='heatmap-cell'/>",
                y_pos, color
            ));
            heatmap.push_str(&format!(
                "<text x='60' y='{:.1}' font-size='12' fill='black'>{}</text>",
                y_pos + 15.0,
                resource
            ));
            y_pos += 25.0;
        }
        heatmap.push_str("</svg>");
        heatmap
    }
}

pub fn create_dotted_chart(event_log: &EventLog, options: DottedChartOptions) -> DottedChart {
    let mut chart = DottedChart::new(options);
    if event_log.traces.is_empty() {
        return chart;
    }

    let mut durations = Vec::new();
    for trace in &event_log.traces {
        let stats = chart.calculate_trace_stats(trace);
        durations.push(stats.duration_hours);
        chart.trace_stats.push(stats);
    }

    let anomalies = DottedChart::detect_anomalies(&durations, chart.options.anomaly_threshold);

    let mut min_time = None;
    let mut max_time = None;
    for trace in &event_log.traces {
        if !trace.events.is_empty() {
            let start = trace.events[0].timestamp;
            let end = trace.events[trace.events.len() - 1].timestamp;
            min_time = Some(
                min_time
                    .map(|t: DateTime<Utc>| t.min(start))
                    .unwrap_or(start),
            );
            max_time = Some(max_time.map(|t: DateTime<Utc>| t.max(end)).unwrap_or(end));
        }
    }

    chart.min_time = min_time;
    chart.max_time = max_time;

    let time_range = if let (Some(min), Some(max)) = (min_time, max_time) {
        max.signed_duration_since(min).num_seconds() as f64
    } else {
        1.0
    };

    let num_traces = event_log.traces.len() as f64;
    for (trace_idx, trace) in event_log.traces.iter().enumerate() {
        let y = 50.0 + (trace_idx as f64 * ((chart.options.height as f64 - 90.0) / num_traces));
        if !trace.events.is_empty() && time_range > 0.0 {
            for event in &trace.events {
                if let Some(min_time) = min_time {
                    let elapsed = event
                        .timestamp
                        .signed_duration_since(min_time)
                        .num_seconds() as f64;
                    let x = 50.0 + (elapsed / time_range) * (chart.options.width as f64 - 90.0);
                    let is_anomaly = anomalies.get(trace_idx).copied().unwrap_or(false);
                    chart.add_dot(
                        x,
                        y,
                        &event.activity,
                        event.resource.clone(),
                        is_anomaly && chart.options.highlight_anomalies,
                    );
                }
            }
        }
    }
    chart
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::Event;
    use chrono::Utc;

    #[test]
    fn test_dotted_chart_options_creation() {
        let opts = DottedChartOptions::new();
        assert_eq!(opts.width, 1400);
        assert_eq!(opts.height, 600);
        assert!(opts.show_resources);
    }

    #[test]
    fn test_dotted_chart_options_builder() {
        let opts = DottedChartOptions::new()
            .with_resources(false)
            .with_durations(false);
        assert!(!opts.show_resources);
        assert!(!opts.show_durations);
    }

    #[test]
    fn test_create_empty_chart() {
        let chart = DottedChart::new(DottedChartOptions::new());
        assert_eq!(chart.dots.len(), 0);
    }

    #[test]
    fn test_anomaly_detection() {
        let durations = vec![1.0, 1.1, 1.0, 5.0, 1.0, 1.1];
        let anomalies = DottedChart::detect_anomalies(&durations, 2.0);
        assert_eq!(anomalies.len(), 6);
        assert!(anomalies[3]);
    }

    #[test]
    fn test_generate_svg_structure() {
        let chart = DottedChart::new(DottedChartOptions::new());
        let svg = chart.generate_svg();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_create_chart_from_log() {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case1");
        let now = Utc::now();
        trace.add_event(Event::new("Activity A", now).with_resource("Resource1"));
        trace.add_event(
            Event::new("Activity B", now + Duration::hours(1)).with_resource("Resource2"),
        );
        log.add_trace(trace);
        let chart = create_dotted_chart(&log, DottedChartOptions::new());
        assert_eq!(chart.dots.len(), 2);
    }

    #[test]
    fn test_heatmap_generation() {
        let chart = DottedChart::new(DottedChartOptions::new());
        let heatmap = chart.generate_heatmap_overlay();
        assert!(heatmap.contains("<svg"));
    }

    #[test]
    fn test_duration_calculation() {
        let mut trace = Trace::new("case1");
        let now = Utc::now();
        trace.add_event(Event::new("A", now));
        trace.add_event(Event::new("B", now + Duration::hours(2)));
        let chart = DottedChart::new(DottedChartOptions::new());
        let stats = chart.calculate_trace_stats(&trace);
        assert!(stats.duration_hours >= 1.99 && stats.duration_hours <= 2.01);
    }

    #[test]
    fn test_activity_color_generation() {
        let mut chart = DottedChart::new(DottedChartOptions::new());
        let color1 = chart.generate_activity_color("Activity A");
        let color2 = chart.generate_activity_color("Activity A");
        assert_eq!(color1, color2);
    }

    #[test]
    fn test_color_persistence() {
        let mut chart = DottedChart::new(DottedChartOptions::new());
        chart.generate_activity_color("Activity A");
        assert!(chart.activity_colors.contains_key("Activity A"));
    }
}
