//! Animation and Playback Visualization
//!
//! Token replay and frame-by-frame animation support

use crate::log::{EventLog, Trace};
use std::collections::HashMap;

/// Animation playback speed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationSpeed {
    /// Very slow: 0.25x
    VerySlow,
    /// Slow: 0.5x
    Slow,
    /// Normal: 1.0x
    Normal,
    /// Fast: 2.0x
    Fast,
    /// Very fast: 4.0x
    VeryFast,
}

impl AnimationSpeed {
    /// Get speed multiplier
    pub fn multiplier(self) -> f64 {
        match self {
            AnimationSpeed::VerySlow => 0.25,
            AnimationSpeed::Slow => 0.5,
            AnimationSpeed::Normal => 1.0,
            AnimationSpeed::Fast => 2.0,
            AnimationSpeed::VeryFast => 4.0,
        }
    }
}

/// Animation options
#[derive(Debug, Clone)]
pub struct AnimationOptions {
    /// Width
    pub width: usize,
    /// Height
    pub height: usize,
    /// Speed
    pub speed: AnimationSpeed,
    /// Frame duration
    pub frame_duration_ms: usize,
    /// Show labels
    pub show_labels: bool,
    /// Show token count
    pub show_token_count: bool,
    /// Highlight current
    pub highlight_current: bool,
    /// Trail length
    pub trail_length: i32,
}

impl AnimationOptions {
    /// Create new animation options
    pub fn new() -> Self {
        Self {
            width: 1000,
            height: 800,
            speed: AnimationSpeed::Normal,
            frame_duration_ms: 500,
            show_labels: true,
            show_token_count: true,
            highlight_current: true,
            trail_length: 10,
        }
    }

    /// Set speed
    pub fn with_speed(mut self, speed: AnimationSpeed) -> Self {
        self.speed = speed;
        self
    }

    /// Set frame duration
    pub fn with_frame_duration(mut self, ms: usize) -> Self {
        self.frame_duration_ms = ms;
        self
    }

    /// Set labels
    pub fn with_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Set token count
    pub fn with_token_count(mut self, show: bool) -> Self {
        self.show_token_count = show;
        self
    }
}

impl Default for AnimationOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Animation frame
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    /// Frame number
    pub frame_number: usize,
    /// Timestamp
    pub timestamp_ms: usize,
    /// Event index
    pub event_index: usize,
    /// Active transition
    pub active_transition: Option<String>,
    /// Token state
    pub token_state: HashMap<String, usize>,
    /// Event label
    pub event_label: String,
    /// Highlight color
    pub highlight_color: String,
}

impl AnimationFrame {
    /// Create new frame
    pub fn new(frame_number: usize, event_index: usize) -> Self {
        Self {
            frame_number,
            timestamp_ms: frame_number * 500,
            event_index,
            active_transition: None,
            token_state: HashMap::new(),
            event_label: String::new(),
            highlight_color: "#FF6B6B".to_string(),
        }
    }

    /// Set transition
    pub fn with_transition(mut self, transition_id: String) -> Self {
        self.active_transition = Some(transition_id);
        self
    }

    /// Set token state
    pub fn with_token_state(mut self, state: HashMap<String, usize>) -> Self {
        self.token_state = state;
        self
    }

    /// Set label
    pub fn with_label(mut self, label: String) -> Self {
        self.event_label = label;
        self
    }
}

/// Animation visualization
#[derive(Debug, Clone)]
pub struct Animation {
    frames: Vec<AnimationFrame>,
    options: AnimationOptions,
    /// Total duration
    pub total_duration_ms: usize,
}

impl Animation {
    /// Create new animation
    pub fn new(options: AnimationOptions) -> Self {
        Self {
            frames: Vec::new(),
            options,
            total_duration_ms: 0,
        }
    }

    /// Add frame
    pub fn add_frame(&mut self, frame: AnimationFrame) {
        self.total_duration_ms = frame.timestamp_ms.max(self.total_duration_ms);
        self.frames.push(frame);
    }

    /// Get frame at timestamp
    pub fn frame_at(&self, timestamp_ms: usize) -> Option<&AnimationFrame> {
        self.frames
            .binary_search_by(|f| f.timestamp_ms.cmp(&timestamp_ms))
            .ok()
            .and_then(|idx| self.frames.get(idx))
    }

    /// Get frame count
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Generate frame SVG
    pub fn generate_frame_svg(&self, frame_number: usize) -> String {
        let Some(frame) = self.frames.get(frame_number) else {
            return String::new();
        };

        let mut svg = String::new();

        svg.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
            self.options.width, self.options.height, self.options.width, self.options.height
        ));

        svg.push_str("<defs><style>");
        svg.push_str(".animation-frame-bg { fill: white; stroke: black; stroke-width: 2; }");
        svg.push_str(".animation-label { font-size: 14px; fill: black; }");
        svg.push_str(".animation-title { font-size: 20px; font-weight: bold; }");
        svg.push_str("</style></defs>");

        svg.push_str(&format!(
            "<rect class=\"animation-frame-bg\" width=\"{}\" height=\"{}\"/>",
            self.options.width, self.options.height
        ));

        svg.push_str(&format!(
            "<text class=\"animation-title\" x=\"20\" y=\"30\">Frame {}: {}</text>",
            frame.frame_number, frame.event_label
        ));

        let progress_percent = if self.total_duration_ms > 0 {
            (frame.timestamp_ms as f64 / self.total_duration_ms as f64) * 100.0
        } else {
            0.0
        };
        svg.push_str(&format!(
            "<text x=\"20\" y=\"55\" class=\"animation-label\" font-size=\"12\">Progress: {:.1}%</text>",
            progress_percent
        ));

        if self.options.show_token_count {
            self.render_token_state(&mut svg, frame);
        }

        self.render_playback_info(&mut svg, frame);

        svg.push_str("</svg>");
        svg
    }

    /// Render token state
    fn render_token_state(&self, svg: &mut String, frame: &AnimationFrame) {
        let mut y_pos = 100.0;
        svg.push_str("<text class=\"animation-title\" x=\"20\" y=\"85\" font-size=\"16\">Token State:</text>");

        for (place_id, count) in &frame.token_state {
            let color = if *count > 0 { "#4CAF50" } else { "#CCCCCC" };
            svg.push_str(&format!(
                "<text x=\"20\" y=\"{:.1}\" font-size=\"12\" fill=\"{}\">{}: {} tokens</text>",
                y_pos, color, place_id, count
            ));
            y_pos += 20.0;
        }
    }

    /// Render playback info
    fn render_playback_info(&self, svg: &mut String, frame: &AnimationFrame) {
        let y_pos = (self.options.height as f64) - 40.0;

        svg.push_str(&format!(
            "<text x=\"{}\" y=\"{:.1}\" class=\"animation-label\" font-size=\"12\" text-anchor=\"middle\">Frame: {}/{} | Time: {:.1}s | Speed: {:?}</text>",
            self.options.width / 2,
            y_pos,
            frame.frame_number + 1,
            self.frames.len(),
            frame.timestamp_ms as f64 / 1000.0,
            self.options.speed
        ));
    }

    /// Generate HTML animation player
    pub fn generate_animation_html(&self) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html><html><head><title>Process Animation Player</title>");
        html.push_str("<style>");
        html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }");
        html.push_str(".animation-container { max-width: 1200px; margin: 0 auto; }");
        html.push_str(".animation-canvas { border: 2px solid #333; margin-bottom: 20px; }");
        html.push_str(".animation-controls { display: flex; gap: 10px; margin-bottom: 20px; }");
        html.push_str("button { padding: 8px 16px; cursor: pointer; }");
        html.push_str("input[type=\"range\"] { flex: 1; }");
        html.push_str("</style></head><body>");
        html.push_str("<div class=\"animation-container\">");
        html.push_str("<h1>Process Animation Player</h1>");
        html.push_str("<div class=\"animation-canvas\" id=\"canvas\"></div>");
        html.push_str("<div class=\"animation-controls\">");
        html.push_str("<button onclick=\"play()\">Play</button>");
        html.push_str("<button onclick=\"pause()\">Pause</button>");
        html.push_str("<button onclick=\"reset()\">Reset</button>");
        html.push_str("<input type=\"range\" id=\"frameSlider\" min=\"0\" value=\"0\">");
        html.push_str("<select id=\"speedSelect\">");
        html.push_str("<option value=\"0.25\">Very Slow</option>");
        html.push_str("<option value=\"0.5\">Slow</option>");
        html.push_str("<option value=\"1\" selected>Normal</option>");
        html.push_str("<option value=\"2\">Fast</option>");
        html.push_str("<option value=\"4\">Very Fast</option>");
        html.push_str("</select>");
        html.push_str("</div></div></body></html>");

        html
    }

    /// Generate playlist SVG
    pub fn generate_playlist_svg(&self) -> String {
        let mut svg = String::new();

        svg.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1200\" height=\"{}\">",
            self.frames.len() * 50 + 100
        ));

        svg.push_str("<defs><style>.timeline-frame { cursor: pointer; }.timeline-label { font-size: 12px; }</style></defs>");

        svg.push_str(&format!(
            "<text x=\"20\" y=\"30\" font-size=\"18\" font-weight=\"bold\">Animation Timeline ({} frames)</text>",
            self.frames.len()
        ));

        let mut x_pos = 20.0;
        for (idx, _frame) in self.frames.iter().enumerate() {
            let height = 40.0;
            let width = 80.0;

            svg.push_str(&format!(
                "<rect x=\"{:.1}\" y=\"60\" width=\"{:.1}\" height=\"{}\" fill=\"#E8F5E9\" stroke=\"#388E3C\" class=\"timeline-frame\" data-frame=\"{}\"/>",
                x_pos, width, height, idx
            ));

            svg.push_str(&format!(
                "<text x=\"{:.1}\" y=\"95\" text-anchor=\"middle\" class=\"timeline-label\">{}</text>",
                x_pos + width / 2.0,
                idx + 1
            ));

            x_pos += width + 5.0;
        }

        svg.push_str("</svg>");
        svg
    }
}

/// Create animation from trace
pub fn create_animation_from_trace(trace: &Trace, options: AnimationOptions) -> Animation {
    let actual_frame_duration =
        (options.frame_duration_ms as f64 / options.speed.multiplier()) as usize;
    let mut animation = Animation::new(options);

    let mut token_state: HashMap<String, usize> = HashMap::new();

    for (event_idx, event) in trace.events.iter().enumerate() {
        let mut frame = AnimationFrame::new(event_idx, event_idx);
        frame.timestamp_ms = event_idx * actual_frame_duration;
        frame.event_label = event.activity.clone();

        let activity_key = format!("place_{}", event_idx);
        token_state.insert(activity_key, event_idx);
        frame.token_state = token_state.clone();

        animation.add_frame(frame);
    }

    animation
}

/// Create animation from log
pub fn create_animation_from_log(
    event_log: &EventLog,
    options: AnimationOptions,
) -> Vec<Animation> {
    event_log
        .traces
        .iter()
        .map(|trace| create_animation_from_trace(trace, options.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::Event;
    use chrono::Utc;

    #[test]
    fn test_animation_speed() {
        assert_eq!(AnimationSpeed::VerySlow.multiplier(), 0.25);
        assert_eq!(AnimationSpeed::Slow.multiplier(), 0.5);
        assert_eq!(AnimationSpeed::Normal.multiplier(), 1.0);
        assert_eq!(AnimationSpeed::Fast.multiplier(), 2.0);
        assert_eq!(AnimationSpeed::VeryFast.multiplier(), 4.0);
    }

    #[test]
    fn test_animation_options_creation() {
        let opts = AnimationOptions::new();
        assert_eq!(opts.width, 1000);
        assert_eq!(opts.height, 800);
        assert_eq!(opts.speed, AnimationSpeed::Normal);
        assert!(opts.show_labels);
    }

    #[test]
    fn test_animation_options_builder() {
        let opts = AnimationOptions::new()
            .with_speed(AnimationSpeed::Fast)
            .with_frame_duration(1000);
        assert_eq!(opts.speed, AnimationSpeed::Fast);
        assert_eq!(opts.frame_duration_ms, 1000);
    }

    #[test]
    fn test_animation_frame_creation() {
        let frame = AnimationFrame::new(0, 0).with_label("Activity A".to_string());
        assert_eq!(frame.event_label, "Activity A");
        assert_eq!(frame.frame_number, 0);
    }

    #[test]
    fn test_animation_add_frames() {
        let mut animation = Animation::new(AnimationOptions::new());
        animation.add_frame(AnimationFrame::new(0, 0).with_label("A".to_string()));
        animation.add_frame(AnimationFrame::new(1, 1).with_label("B".to_string()));

        assert_eq!(animation.frame_count(), 2);
    }

    #[test]
    fn test_generate_frame_svg() {
        let mut animation = Animation::new(AnimationOptions::new());
        animation.add_frame(AnimationFrame::new(0, 0).with_label("Activity A".to_string()));

        let svg = animation.generate_frame_svg(0);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Activity A"));
    }

    #[test]
    fn test_generate_animation_html() {
        let animation = Animation::new(AnimationOptions::new());
        let html = animation.generate_animation_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Animation Player"));
    }

    #[test]
    fn test_generate_playlist_svg() {
        let mut animation = Animation::new(AnimationOptions::new());
        animation.add_frame(AnimationFrame::new(0, 0).with_label("A".to_string()));
        animation.add_frame(AnimationFrame::new(1, 1).with_label("B".to_string()));

        let svg = animation.generate_playlist_svg();
        assert!(svg.contains("Timeline"));
    }

    #[test]
    fn test_create_animation_from_trace() {
        let mut trace = Trace::new("case1");
        let now = Utc::now();
        trace.add_event(Event::new("Activity A", now));
        trace.add_event(Event::new(
            "Activity B",
            now + chrono::Duration::seconds(10),
        ));

        let animation = create_animation_from_trace(&trace, AnimationOptions::new());
        assert_eq!(animation.frame_count(), 2);
    }

    #[test]
    fn test_token_state_tracking() {
        let mut frame = AnimationFrame::new(0, 0);
        let mut state = HashMap::new();
        state.insert("place1".to_string(), 5);
        state.insert("place2".to_string(), 2);

        frame = frame.with_token_state(state);
        assert_eq!(frame.token_state.len(), 2);
        assert_eq!(frame.token_state.get("place1"), Some(&5));
    }

    #[test]
    fn test_animation_frame_at() {
        let mut animation = Animation::new(AnimationOptions::new());
        let mut frame1 = AnimationFrame::new(0, 0);
        frame1.timestamp_ms = 0;
        animation.add_frame(frame1);

        assert!(animation.frame_at(0).is_some());
    }

    #[test]
    fn test_frame_count() {
        let mut animation = Animation::new(AnimationOptions::new());
        animation.add_frame(AnimationFrame::new(0, 0));
        animation.add_frame(AnimationFrame::new(1, 1));
        animation.add_frame(AnimationFrame::new(2, 2));

        assert_eq!(animation.frame_count(), 3);
    }
}
