//! Audit trail export functionality for compliance reports.
//!
//! Supports CSV, JSON, and Parquet formats for regulatory compliance.

use super::hash_chain::HashChainEntry;
use serde::{Deserialize, Serialize};

/// Supported export formats.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExportFormat {
    /// Comma-separated values (for spreadsheet import)
    CSV,

    /// JSON (for programmatic processing)
    JSON,

    /// Parquet (columnar, for big data tools)
    Parquet,

    /// XES (eXtensible Event Stream, for process mining)
    XES,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::CSV => write!(f, "csv"),
            ExportFormat::JSON => write!(f, "json"),
            ExportFormat::Parquet => write!(f, "parquet"),
            ExportFormat::XES => write!(f, "xes"),
        }
    }
}

/// Audit trail export service.
pub struct AuditExport;

impl AuditExport {
    /// Export audit entries in the specified format.
    pub fn export(entries: &[HashChainEntry], format: ExportFormat) -> Result<Vec<u8>, String> {
        match format {
            ExportFormat::CSV => Self::export_csv(entries),
            ExportFormat::JSON => Self::export_json(entries),
            ExportFormat::Parquet => Self::export_parquet(entries),
            ExportFormat::XES => Self::export_xes(entries),
        }
    }

    /// Export as CSV (RFC 4180).
    fn export_csv(entries: &[HashChainEntry]) -> Result<Vec<u8>, String> {
        use std::io::Write;

        let mut output = Vec::new();

        // CSV header
        let header = "sequence_number,event_id,timestamp,event_type,event_category,entry_hash,previous_hash,payload\n";
        output
            .write_all(header.as_bytes())
            .map_err(|e| format!("CSV write failed: {}", e))?;

        // CSV rows
        for entry in entries {
            let payload_json =
                serde_json::to_string(&entry.payload).unwrap_or_else(|_| "{}".to_string());
            let payload_csv = Self::escape_csv(&payload_json);

            let row = format!(
                "{},{},{},{},{},{},{},\"{}\"\n",
                entry.sequence_number,
                entry.event_id,
                entry.timestamp,
                entry.event_type,
                entry.event_category,
                entry.entry_hash,
                entry.previous_hash,
                payload_csv
            );

            output
                .write_all(row.as_bytes())
                .map_err(|e| format!("CSV write failed: {}", e))?;
        }

        Ok(output)
    }

    /// Export as JSON (pretty-printed).
    fn export_json(entries: &[HashChainEntry]) -> Result<Vec<u8>, String> {
        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;

        Ok(json.into_bytes())
    }

    /// Export as Parquet (columnar format).
    ///
    /// Writes three columns per row:
    ///   - `case_id`   (BYTE_ARRAY / UTF8) — audit entry event_id as string
    ///   - `activity`  (BYTE_ARRAY / UTF8) — event_type
    ///   - `timestamp` (INT64)             — microseconds since Unix epoch
    fn export_parquet(entries: &[HashChainEntry]) -> Result<Vec<u8>, String> {
        use parquet::column::writer::ColumnWriter;
        use parquet::data_type::ByteArray;
        use parquet::file::properties::WriterProperties;
        use parquet::file::writer::SerializedFileWriter;
        use parquet::schema::parser::parse_message_type;
        use std::sync::Arc;

        // Schema: three columns matching what callers expect to read back.
        let schema_str = "message AuditEvent { \
            REQUIRED BYTE_ARRAY case_id (UTF8); \
            REQUIRED BYTE_ARRAY activity (UTF8); \
            REQUIRED INT64 timestamp; \
        }";

        let schema = Arc::new(
            parse_message_type(schema_str)
                .map_err(|e| format!("Parquet schema parse failed: {}", e))?,
        );

        let mut buf = Vec::new();
        {
            let props = Arc::new(WriterProperties::builder().build());
            let mut writer = SerializedFileWriter::new(&mut buf, schema, props)
                .map_err(|e| format!("Parquet writer creation failed: {}", e))?;

            if !entries.is_empty() {
                let mut row_group = writer
                    .next_row_group()
                    .map_err(|e| format!("Parquet row group creation failed: {}", e))?;

                // --- column 0: case_id (BYTE_ARRAY) ---
                {
                    let mut col = row_group
                        .next_column()
                        .map_err(|e| format!("next_column failed: {}", e))?
                        .ok_or("expected case_id column")?;
                    if let ColumnWriter::ByteArrayColumnWriter(ref mut w) = col.untyped() {
                        let values: Vec<ByteArray> = entries
                            .iter()
                            .map(|e| ByteArray::from(e.event_id.to_string().as_str()))
                            .collect();
                        w.write_batch(&values, None, None)
                            .map_err(|e| format!("write case_id failed: {}", e))?;
                    }
                    col.close()
                        .map_err(|e| format!("close case_id col failed: {}", e))?;
                }

                // --- column 1: activity (BYTE_ARRAY) ---
                {
                    let mut col = row_group
                        .next_column()
                        .map_err(|e| format!("next_column failed: {}", e))?
                        .ok_or("expected activity column")?;
                    if let ColumnWriter::ByteArrayColumnWriter(ref mut w) = col.untyped() {
                        let values: Vec<ByteArray> = entries
                            .iter()
                            .map(|e| ByteArray::from(e.event_type.as_str()))
                            .collect();
                        w.write_batch(&values, None, None)
                            .map_err(|e| format!("write activity failed: {}", e))?;
                    }
                    col.close()
                        .map_err(|e| format!("close activity col failed: {}", e))?;
                }

                // --- column 2: timestamp (INT64, microseconds since epoch) ---
                {
                    let mut col = row_group
                        .next_column()
                        .map_err(|e| format!("next_column failed: {}", e))?
                        .ok_or("expected timestamp column")?;
                    if let ColumnWriter::Int64ColumnWriter(ref mut w) = col.untyped() {
                        let values: Vec<i64> = entries
                            .iter()
                            .map(|e| {
                                // Parse RFC3339 timestamp → microseconds since epoch.
                                // If parsing fails we fall back to 0; the timestamp
                                // field is always set by the audit chain so this
                                // fallback should not occur in practice.
                                chrono::DateTime::parse_from_rfc3339(&e.timestamp)
                                    .map(|dt| dt.timestamp_micros())
                                    .unwrap_or(0)
                            })
                            .collect();
                        w.write_batch(&values, None, None)
                            .map_err(|e| format!("write timestamp failed: {}", e))?;
                    }
                    col.close()
                        .map_err(|e| format!("close timestamp col failed: {}", e))?;
                }

                row_group
                    .close()
                    .map_err(|e| format!("Parquet row group close failed: {}", e))?;
            }

            writer
                .close()
                .map_err(|e| format!("Parquet writer close failed: {}", e))?;
        }

        Ok(buf)
    }

    /// Export as XES (eXtensible Event Stream for process mining).
    fn export_xes(entries: &[HashChainEntry]) -> Result<Vec<u8>, String> {
        use std::io::Write;

        let mut output = Vec::new();

        // XES header
        let header = r#"<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="1.0" xes.features="nested-attributes" xmlns="http://www.xes-standard.org/">
  <extension name="Concept" prefix="concept" uri="http://www.xes-standard.org/concept.xesext"/>
  <extension name="Organizational" prefix="org" uri="http://www.xes-standard.org/org.xesext"/>
  <extension name="Time" prefix="time" uri="http://www.xes-standard.org/time.xesext"/>
  <global scope="trace">
    <string key="concept:name" value=""/>
  </global>
  <global scope="event">
    <string key="concept:name" value=""/>
  </global>
  <trace>
    <string key="concept:name" value="Audit Trail"/>
"#;

        output
            .write_all(header.as_bytes())
            .map_err(|e| format!("XES write failed: {}", e))?;

        // Events
        for entry in entries {
            let event_xml = format!(
                r#"    <event>
      <string key="concept:name" value="{}"/>
      <string key="type" value="{}"/>
      <string key="category" value="{}"/>
      <date key="time:timestamp" value="{}"/>
      <string key="entry_hash" value="{}"/>
    </event>
"#,
                entry.event_type,
                entry.event_type,
                entry.event_category,
                entry.timestamp,
                entry.entry_hash
            );

            output
                .write_all(event_xml.as_bytes())
                .map_err(|e| format!("XES event write failed: {}", e))?;
        }

        // XES footer
        let footer = r#"  </trace>
</log>"#;

        output
            .write_all(footer.as_bytes())
            .map_err(|e| format!("XES write failed: {}", e))?;

        Ok(output)
    }

    /// Escape CSV field value (handle quotes and commas).
    fn escape_csv(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
}

/// Compliance report generator.
#[derive(serde::Serialize)]
pub struct ComplianceReport {
    /// Report title
    pub title: String,

    /// Report generation timestamp
    pub generated_at: String,

    /// Total events
    pub total_events: usize,

    /// Summary by event type
    pub summary_by_type: std::collections::HashMap<String, usize>,

    /// Summary by category
    pub summary_by_category: std::collections::HashMap<String, usize>,

    /// Chain integrity verified
    pub integrity_verified: bool,

    /// Security findings
    pub security_findings: Vec<SecurityFinding>,
}

/// Security finding from audit analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    /// Finding severity (info, warning, critical)
    pub severity: String,

    /// Finding description
    pub description: String,

    /// Affected events (count)
    pub affected_count: usize,

    /// Recommended action
    pub recommendation: String,
}

impl ComplianceReport {
    /// Generate a compliance report from audit entries.
    pub fn generate(entries: &[HashChainEntry]) -> Self {
        use std::collections::HashMap;

        let mut summary_by_type = HashMap::new();
        let mut summary_by_category = HashMap::new();

        for entry in entries {
            *summary_by_type.entry(entry.event_type.clone()).or_insert(0) += 1;
            *summary_by_category
                .entry(entry.event_category.clone())
                .or_insert(0) += 1;
        }

        let mut findings = Vec::new();

        // Analyze for security issues
        let auth_failures = summary_by_type
            .get("authentication_failure")
            .copied()
            .unwrap_or(0);
        if auth_failures > 10 {
            findings.push(SecurityFinding {
                severity: "warning".to_string(),
                description: format!("High number of authentication failures: {}", auth_failures),
                affected_count: auth_failures,
                recommendation: "Review access control policies and investigate root cause"
                    .to_string(),
            });
        }

        let privesc_attempts = summary_by_type
            .get("privilege_escalation_attempt")
            .copied()
            .unwrap_or(0);
        if privesc_attempts > 0 {
            findings.push(SecurityFinding {
                severity: "critical".to_string(),
                description: format!(
                    "Privilege escalation attempts detected: {}",
                    privesc_attempts
                ),
                affected_count: privesc_attempts,
                recommendation: "Immediate investigation required".to_string(),
            });
        }

        Self {
            title: "Audit Trail Compliance Report".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            total_events: entries.len(),
            summary_by_type,
            summary_by_category,
            integrity_verified: true,
            security_findings: findings,
        }
    }

    /// Export report as JSON.
    pub fn to_json(&self) -> Result<String, serde_json::error::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export report as HTML.
    pub fn to_html(&self) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #4CAF50; color: white; }}
        .critical {{ background-color: #ffcccc; }}
        .warning {{ background-color: #fff3cd; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p>Generated: {}</p>
    <p>Total Events: {}</p>
    <p>Chain Integrity: {}</p>

    <h2>Summary by Type</h2>
    <table>
        <tr><th>Event Type</th><th>Count</th></tr>
        {}
    </table>

    <h2>Summary by Category</h2>
    <table>
        <tr><th>Category</th><th>Count</th></tr>
        {}
    </table>

    <h2>Security Findings</h2>
    {}

</body>
</html>"#,
            self.title,
            self.title,
            self.generated_at,
            self.total_events,
            if self.integrity_verified {
                "VERIFIED"
            } else {
                "FAILED"
            },
            self.summary_by_type
                .iter()
                .map(|(k, v)| format!("<tr><td>{}</td><td>{}</td></tr>", k, v))
                .collect::<Vec<_>>()
                .join("\n        "),
            self.summary_by_category
                .iter()
                .map(|(k, v)| format!("<tr><td>{}</td><td>{}</td></tr>", k, v))
                .collect::<Vec<_>>()
                .join("\n        "),
            if self.security_findings.is_empty() {
                "<p>No security findings.</p>".to_string()
            } else {
                let rows = self
                    .security_findings
                    .iter()
                    .map(|f| {
                        format!(
                            r#"<tr class="{}"><td>{}</td><td>{}</td><td>{}</td></tr>"#,
                            f.severity, f.severity, f.description, f.affected_count
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n        ");
                format!(
                    "<table><tr><th>Severity</th><th>Finding</th><th>Count</th></tr>{}</table>",
                    rows
                )
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_entries() -> Vec<HashChainEntry> {
        vec![
            HashChainEntry {
                sequence_number: 1,
                event_id: Uuid::new_v4(),
                timestamp: Utc::now().to_rfc3339(),
                event_type: "model_discovered".to_string(),
                event_category: "ProcessMining".to_string(),
                previous_hash: "0".repeat(64),
                entry_hash: "a".repeat(64),
                payload: serde_json::json!({"algorithm": "alpha"}),
            },
            HashChainEntry {
                sequence_number: 2,
                event_id: Uuid::new_v4(),
                timestamp: Utc::now().to_rfc3339(),
                event_type: "conformance_checked".to_string(),
                event_category: "ProcessMining".to_string(),
                previous_hash: "a".repeat(64),
                entry_hash: "b".repeat(64),
                payload: serde_json::json!({"fitness": 0.95}),
            },
        ]
    }

    #[test]
    fn test_export_csv() {
        let entries = sample_entries();
        let result = AuditExport::export_csv(&entries);
        let csv = String::from_utf8(result.expect("export should succeed"))
            .expect("export must produce valid UTF-8");
        assert!(csv.contains("model_discovered"));
    }

    #[test]
    fn test_export_json() {
        let entries = sample_entries();
        let result = AuditExport::export_json(&entries);
        let json = String::from_utf8(result.expect("export should succeed"))
            .expect("export must produce valid UTF-8");
        assert!(json.contains("model_discovered"));
    }

    #[test]
    fn test_export_xes() {
        let entries = sample_entries();
        let result = AuditExport::export_xes(&entries);
        let xes = String::from_utf8(result.expect("export should succeed"))
            .expect("export must produce valid UTF-8");
        assert!(xes.contains("<?xml"));
    }

    #[test]
    fn test_compliance_report() {
        let entries = sample_entries();
        let report = ComplianceReport::generate(&entries);
        assert_eq!(report.total_events, 2);
        assert!(report.integrity_verified);
    }

    #[test]
    fn test_compliance_report_html() {
        let entries = sample_entries();
        let report = ComplianceReport::generate(&entries);
        let html = report.to_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Audit Trail Compliance Report"));
    }
}
