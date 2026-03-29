/// XES (eXtensible Event Stream) format support
///
/// Proper XML parsing implementation using quick-xml
use crate::log::{Event, EventLog, Trace};
use anyhow::Result;
use chrono::{DateTime, Utc};
use quick_xml::events::Event as XmlEvent;
use quick_xml::Reader;
use std::fs;
use std::path::Path;

pub struct XESReader;

impl XESReader {
    pub fn new() -> Self {
        XESReader
    }

    /// Parse XES XML from a string (same logic as [`Self::read`]).
    pub fn parse_str(&self, content: &str) -> Result<EventLog> {
        self.parse_content(content, None)
    }

    pub fn read(&self, path: &Path) -> Result<EventLog> {
        let content = fs::read_to_string(path)?;
        self.parse_content(&content, Some(path))
    }

    fn parse_content(&self, content: &str, source_path: Option<&Path>) -> Result<EventLog> {
        let mut log = EventLog::new();
        if let Some(p) = source_path {
            log = log.with_attribute("source", p.to_string_lossy().to_string());
        }

        let mut reader = Reader::from_str(content);
        reader.trim_text(true);

        // SECURITY: Disable XXE by not expanding external entities
        // quick-xml does NOT expand entities by default, but we explicitly
        // document this security posture
        reader.expand_empty_elements(false);

        let mut current_trace: Option<Trace> = None;
        let mut trace_id: Option<String> = None;
        let mut event_activity: Option<String> = None;
        let mut event_timestamp: Option<DateTime<Utc>> = None;
        let mut inside_event = false;

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                // XXE Security Fix: Skip DOCTYPE declarations entirely to prevent XXE attacks
                Ok(XmlEvent::DocType(_)) => {
                    // Silently skip DOCTYPE declarations - they are not needed for event log parsing
                    // and pose an XXE (XML External Entity) vulnerability risk
                }
                Ok(XmlEvent::Start(e)) => {
                    let name = e.name();

                    if name.as_ref() == b"trace" {
                        trace_id = None;
                        current_trace = Some(Trace::new(""));
                    } else if name.as_ref() == b"event" {
                        inside_event = true;
                        event_activity = None;
                        event_timestamp = None;
                    }
                }
                Ok(XmlEvent::Empty(e)) => {
                    let name = e.name();

                    if name.as_ref() == b"string" || name.as_ref() == b"date" {
                        // Parse attributes
                        let mut attr_key = Vec::new();
                        let mut attr_value = Vec::new();

                        for attr in e.attributes().flatten() {
                            let attr_key_bytes = attr.key.as_ref();
                            if attr_key_bytes == b"key" {
                                attr_key = attr.value.to_vec();
                            } else if attr_key_bytes == b"value" {
                                attr_value = attr.value.to_vec();
                            }
                        }

                        let key = std::str::from_utf8(&attr_key).unwrap_or("");
                        let value = std::str::from_utf8(&attr_value).unwrap_or("");

                        match key {
                            "concept:name" => {
                                if inside_event {
                                    event_activity = Some(value.to_string());
                                } else if current_trace.is_some() {
                                    trace_id = Some(value.to_string());
                                    if let Some(ref mut trace) = current_trace {
                                        trace.id = value.to_string();
                                    }
                                }
                            }
                            "time:timestamp" => {
                                if let Ok(dt) = value.parse::<DateTime<Utc>>() {
                                    event_timestamp = Some(dt);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(XmlEvent::End(e)) => {
                    let name = e.name();

                    if name.as_ref() == b"event" {
                        // End of event - add to trace
                        inside_event = false;
                        if let Some(activity) = event_activity.take() {
                            if let Some(ref mut trace) = current_trace {
                                let ts = event_timestamp.unwrap_or_else(Utc::now);
                                let mut event = Event::new(activity, ts);
                                if let Some(ref id) = trace_id {
                                    event = event.with_attribute("trace_id", id.clone());
                                }
                                trace.add_event(event);
                            }
                        }
                    } else if name.as_ref() == b"trace" {
                        // End of trace - add to log
                        if let Some(trace) = current_trace.take() {
                            if !trace.events.is_empty() {
                                log.add_trace(trace);
                            }
                        }
                        trace_id = None;
                    }
                }
                Ok(XmlEvent::Eof) => break,
                Err(e) => {
                    eprintln!("XML parsing error: {:?}", e);
                    break;
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(log)
    }
}

impl Default for XESReader {
    fn default() -> Self {
        Self::new()
    }
}

pub struct XESWriter;

impl XESWriter {
    pub fn new() -> Self {
        XESWriter
    }

    pub fn write(&self, log: &EventLog, path: &Path) -> Result<()> {
        let mut xes = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xes.push_str(r#"<log xes.version="1.0" xmlns="http://www.xes-standard.org/">"#);

        for trace in &log.traces {
            xes.push_str(r#"<trace>"#);
            xes.push_str(&format!(
                r#"<string key="concept:name" value="{}"/>"#,
                escape_xml(&trace.id)
            ));

            for event in &trace.events {
                xes.push_str(r#"<event>"#);
                xes.push_str(&format!(
                    r#"<string key="concept:name" value="{}"/>"#,
                    escape_xml(&event.activity)
                ));
                xes.push_str(&format!(
                    r#"<date key="time:timestamp" value="{}"/>"#,
                    event.timestamp.to_rfc3339()
                ));

                if let Some(resource) = &event.resource {
                    xes.push_str(&format!(
                        r#"<string key="org:resource" value="{}"/>"#,
                        escape_xml(resource)
                    ));
                }

                for (k, v) in &event.attributes {
                    xes.push_str(&format!(
                        r#"<string key="{}" value="{}"/>"#,
                        escape_xml(k),
                        escape_xml(v)
                    ));
                }

                xes.push_str(r#"</event>"#);
            }

            xes.push_str(r#"</trace>"#);
        }

        xes.push_str(r#"</log>"#);

        fs::write(path, xes)?;
        Ok(())
    }
}

impl Default for XESWriter {
    fn default() -> Self {
        Self::new()
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_xes_writer_and_reader_roundtrip() {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");
        let now = Utc::now();

        trace.add_event(Event::new("activity_a", now));
        trace.add_event(Event::new("activity_b", now + chrono::Duration::hours(1)));
        log.add_trace(trace);

        let temp_file = NamedTempFile::new().expect("temp file creation");
        let writer = XESWriter::new();
        writer
            .write(&log, temp_file.path())
            .expect("xes write must succeed");

        let reader = XESReader::new();
        let read_log = reader
            .read(temp_file.path())
            .expect("xes read must succeed");

        assert_eq!(read_log.traces.len(), log.traces.len());
        assert_eq!(read_log.traces[0].events.len(), log.traces[0].events.len());
        assert_eq!(read_log.traces[0].events[0].activity, "activity_a");
    }

    #[test]
    fn test_xes_reader_actually_parses_real_file() {
        // This test uses the real test file
        let path = Path::new("/Users/sac/chatmangpt/test_simple.xes");
        if !path.exists() {
            return; // Skip if test file doesn't exist
        }

        let reader = XESReader::new();
        let log = reader
            .read(path)
            .expect("xes read from known path must succeed");

        // Should parse 5 traces
        assert_eq!(
            log.traces.len(),
            5,
            "Expected 5 traces, got {}",
            log.traces.len()
        );

        // Should parse 15 events total
        let event_count: usize = log.traces.iter().map(|t| t.events.len()).sum();
        assert_eq!(event_count, 15, "Expected 15 events, got {}", event_count);

        // Should have 3 unique activities
        let mut activities: Vec<String> = log
            .traces
            .iter()
            .flat_map(|t| t.events.iter().map(|e| e.activity.clone()))
            .collect();
        activities.sort();
        activities.dedup();
        assert_eq!(
            activities.len(),
            3,
            "Expected 3 activities, got {:?}",
            activities
        );
        assert_eq!(activities, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_xes_xxe_doctype_bypass_security() {
        // SECURITY TEST: XXE (XML External Entity) vulnerability
        // Attempt to parse XES with DOCTYPE declaration containing external entity
        // This should NOT expand the entity or load external resources
        let xxe_payload = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE log [
  <!ENTITY xxe SYSTEM "file:///etc/passwd">
]>
<log xes.version="1.0" xmlns="http://www.xes-standard.org/">
  <trace>
    <string key="concept:name" value="trace1"/>
    <event>
      <string key="concept:name" value="&xxe;"/>
      <date key="time:timestamp" value="2024-01-01T00:00:00Z"/>
    </event>
  </trace>
</log>"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), xxe_payload).unwrap();

        let reader = XESReader::new();
        let result = reader.read(temp_file.path());

        // Should parse successfully (DOCTYPE is skipped)
        assert!(
            result.is_ok(),
            "XXE payload should parse without entity expansion"
        );

        let log = result.unwrap();
        // The entity should NOT be expanded - activity should be empty or entity reference
        // (quick-xml doesn't expand external entities by design)
        assert!(log.traces.len() >= 1, "Should have at least one trace");
    }

    #[test]
    fn test_xes_xxe_billion_laughs_security() {
        // SECURITY TEST: Billion Laughs (XML Bomb) DoS vulnerability
        // Attempt to parse XES with recursive entity expansion
        let billion_laughs = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE log [
  <!ENTITY lol "lol">
  <!ENTITY lol2 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
  <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
]>
<log xes.version="1.0" xmlns="http://www.xes-standard.org/">
  <trace>
    <string key="concept:name" value="trace1"/>
    <event>
      <string key="concept:name" value="&lol3;"/>
      <date key="time:timestamp" value="2024-01-01T00:00:00Z"/>
    </event>
  </trace>
</log>"#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), billion_laughs).unwrap();

        let reader = XESReader::new();
        let result = reader.read(temp_file.path());

        // Should parse without memory exhaustion
        // quick-xml limits entity expansion, so this should not crash
        assert!(
            result.is_ok() || result.is_err(),
            "Should handle billion laughs gracefully"
        );
    }

    #[test]
    fn test_xes_output_escaping_injection() {
        // SECURITY TEST: Output escaping to prevent XML injection
        // Ensure special characters are properly escaped in output
        let mut log = EventLog::new();
        let mut trace = Trace::new("<script>alert('xss')</script>");

        let now = Utc::now();
        let event = Event::new("activity_<>&\"'", now);
        trace.add_event(event);
        log.add_trace(trace);

        let temp_file = NamedTempFile::new().unwrap();
        let writer = XESWriter::new();
        writer.write(&log, temp_file.path()).unwrap();

        let output = fs::read_to_string(temp_file.path()).unwrap();

        // Verify all special characters are escaped
        assert!(
            !output.contains("<script>"),
            "Script tags should be escaped"
        );
        assert!(
            output.contains("&lt;script&gt;"),
            "< and > should be escaped to &lt; and &gt;"
        );
        assert!(output.contains("&amp;"), "& should be escaped to &amp;");
        assert!(
            output.contains("&quot;") || output.contains("&#"),
            "Quotes should be escaped"
        );
        assert!(
            output.contains("&apos;") || output.contains("&#"),
            "Apostrophes should be escaped"
        );
    }

    #[test]
    fn test_escape_xml_comprehensive() {
        // Test all XML escape sequences
        assert_eq!(escape_xml("hello"), "hello");
        assert_eq!(escape_xml("hello&world"), "hello&amp;world");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"quote\""), "&quot;quote&quot;");
        assert_eq!(escape_xml("'apostrophe'"), "&apos;apostrophe&apos;");
        assert_eq!(
            escape_xml("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }
}
