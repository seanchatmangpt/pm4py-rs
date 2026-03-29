//! Error handling with actionable context for pm4py-rust
//!
//! Provides structured error types with:
//! - What operation failed
//! - What input caused the failure
//! - How to fix it
//! - Relevant error chain for debugging

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// Main error type for pm4py-rust operations
#[derive(Debug)]
pub enum Pm4PyError {
    /// Input/Output errors with file paths and context
    Io(IoError),

    /// Parsing errors (CSV, XES, JSON, Parquet) with location details
    Parse(ParseError),

    /// Validation errors (empty logs, missing required fields)
    Validation(ValidationError),

    /// Discovery algorithm errors (invalid input, algorithm-specific issues)
    Discovery(DiscoveryError),

    /// Conformance checking errors (model mismatch, replay failures)
    Conformance(ConformanceError),

    /// Connector errors (webhook, CSV extraction, schema mapping)
    Connector(ConnectorError),

    /// Python bridge errors (pm4py not installed, GIL issues)
    PythonBridge(PythonBridgeError),

    /// HTTP API errors (invalid request payload, missing fields)
    Api(ApiError),
}

impl fmt::Display for Pm4PyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pm4PyError::Io(e) => write!(f, "{}", e),
            Pm4PyError::Parse(e) => write!(f, "{}", e),
            Pm4PyError::Validation(e) => write!(f, "{}", e),
            Pm4PyError::Discovery(e) => write!(f, "{}", e),
            Pm4PyError::Conformance(e) => write!(f, "{}", e),
            Pm4PyError::Connector(e) => write!(f, "{}", e),
            Pm4PyError::PythonBridge(e) => write!(f, "{}", e),
            Pm4PyError::Api(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Pm4PyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Pm4PyError::Io(e) => Some(e),
            Pm4PyError::Parse(e) => Some(e),
            Pm4PyError::Validation(e) => Some(e),
            Pm4PyError::Discovery(e) => Some(e),
            Pm4PyError::Conformance(e) => Some(e),
            Pm4PyError::Connector(e) => Some(e),
            Pm4PyError::PythonBridge(e) => Some(e),
            Pm4PyError::Api(e) => Some(e),
        }
    }
}

/// File I/O errors with actionable context
#[derive(Debug)]
pub struct IoError {
    pub operation: String,
    pub path: Option<PathBuf>,
    pub kind: IoErrorKind,
    pub fix: String,
}

#[derive(Debug)]
pub enum IoErrorKind {
    FileNotFound,
    PermissionDenied,
    InvalidData,
    ConnectionRefused,
    TimedOut,
    Other(io::ErrorKind),
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "I/O error in '{}': {:?}. {}",
            self.operation, self.kind, self.fix
        )
    }
}

impl std::error::Error for IoError {}

/// Parse errors with location and recovery hints
#[derive(Debug)]
pub struct ParseError {
    pub format: ParseFormat,
    pub location: String,
    pub reason: String,
    pub line: Option<usize>,
    pub fix: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ParseFormat {
    Csv,
    Xes,
    Json,
    Parquet,
    Ocel2,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let line_info = self
            .line
            .map(|l| format!(" at line {}", l))
            .unwrap_or_default();
        write!(
            f,
            "Failed to parse {:?}{}: '{}'. {}",
            self.format, line_info, self.reason, self.fix
        )
    }
}

impl std::error::Error for ParseError {}

/// Validation errors for invalid input
#[derive(Debug)]
pub struct ValidationError {
    pub what: String,
    pub why: String,
    pub fix: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Validation failed: {}. {}. Fix: {}",
            self.what, self.why, self.fix
        )
    }
}

impl std::error::Error for ValidationError {}

/// Discovery algorithm errors
#[derive(Debug)]
pub struct DiscoveryError {
    pub algorithm: String,
    pub reason: String,
    pub fix: String,
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Discovery error ({}): {}. {}",
            self.algorithm, self.reason, self.fix
        )
    }
}

impl std::error::Error for DiscoveryError {}

/// Conformance checking errors
#[derive(Debug)]
pub struct ConformanceError {
    pub checker: String,
    pub reason: String,
    pub fix: String,
}

impl fmt::Display for ConformanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Conformance error ({}): {}. {}",
            self.checker, self.reason, self.fix
        )
    }
}

impl std::error::Error for ConformanceError {}

/// Connector errors
#[derive(Debug)]
pub struct ConnectorError {
    pub connector_type: String,
    pub operation: String,
    pub reason: String,
    pub fix: String,
}

impl fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Connector error ({}): {} failed. {}. {}",
            self.connector_type, self.operation, self.reason, self.fix
        )
    }
}

impl std::error::Error for ConnectorError {}

/// Python bridge errors
#[derive(Debug)]
pub struct PythonBridgeError {
    pub operation: String,
    pub reason: String,
    pub fix: String,
}

impl fmt::Display for PythonBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Python bridge error ({}): {}. {}",
            self.operation, self.reason, self.fix
        )
    }
}

impl std::error::Error for PythonBridgeError {}

/// HTTP API errors
#[derive(Debug)]
pub struct ApiError {
    pub endpoint: String,
    pub reason: String,
    pub fix: String,
    pub status_code: Option<u16>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = self
            .status_code
            .map(|s| format!(" (HTTP {})", s))
            .unwrap_or_default();
        write!(
            f,
            "API error{} ({}): {}. {}",
            status, self.endpoint, self.reason, self.fix
        )
    }
}

impl std::error::Error for ApiError {}

/// Helper functions for creating common errors
impl Pm4PyError {
    /// CSV missing column error
    pub fn csv_missing_column(column: &str, path: &Path) -> Self {
        Pm4PyError::Parse(ParseError {
            format: ParseFormat::Csv,
            location: path.display().to_string(),
            reason: format!("required column '{}' not found", column),
            line: None,
            fix: format!(
                "Add column '{}' to CSV or use CSVReader::with_{}_column() to map to existing column",
                column, column
            ),
        })
    }

    /// CSV invalid delimiter error
    pub fn csv_invalid_delimiter(delimiter: u8) -> Self {
        Pm4PyError::Parse(ParseError {
            format: ParseFormat::Csv,
            location: "CSVReader::with_delimiter()".to_string(),
            reason: format!("invalid delimiter '{:?}'", delimiter as char),
            line: None,
            fix: "Only comma (','), semicolon (';'), tab ('\\t'), and pipe ('|') are allowed"
                .to_string(),
        })
    }

    /// CSV timestamp parse error
    pub fn csv_timestamp_parse(column: &str, value: &str, path: &Path) -> Self {
        Pm4PyError::Parse(ParseError {
            format: ParseFormat::Csv,
            location: path.display().to_string(),
            reason: format!(
                "timestamp column '{}' has invalid value '{}'. Expected RFC3339 or ISO8601 format",
                column, value
            ),
            line: None,
            fix: "Use format: 2024-01-01T00:00:00Z or 2024-01-01T00:00:00.000Z".to_string(),
        })
    }

    /// Empty event log error
    pub fn empty_log(operation: &str) -> Self {
        Pm4PyError::Validation(ValidationError {
            what: format!("Event log is empty in '{}'", operation),
            why: "Cannot perform discovery or conformance checking on an empty log".to_string(),
            fix: "Ensure the log contains at least one trace with one event".to_string(),
        })
    }

    /// File not found error
    pub fn file_not_found(operation: &str, path: &PathBuf) -> Self {
        Pm4PyError::Io(IoError {
            operation: operation.to_string(),
            path: Some(path.clone()),
            kind: IoErrorKind::FileNotFound,
            fix: format!(
                "Check file exists at path: {:?}. Use absolute path if relative path fails.",
                path
            ),
        })
    }

    /// Discovery error with empty log
    pub fn discovery_empty_log(algorithm: &str) -> Self {
        Pm4PyError::Discovery(DiscoveryError {
            algorithm: algorithm.to_string(),
            reason: "event log contains no traces or events".to_string(),
            fix: "Ensure log has at least 1 trace with 2+ events for process discovery".to_string(),
        })
    }

    /// XES parse error
    pub fn xes_parse_error(location: &str, reason: &str) -> Self {
        Pm4PyError::Parse(ParseError {
            format: ParseFormat::Xes,
            location: location.to_string(),
            reason: reason.to_string(),
            line: None,
            fix: "Validate XML structure. Ensure xmlns='http://www.xes-standard.org/' is present"
                .to_string(),
        })
    }

    /// JSON parse error
    pub fn json_parse_error(location: &str, reason: &str) -> Self {
        Pm4PyError::Parse(ParseError {
            format: ParseFormat::Json,
            location: location.to_string(),
            reason: reason.to_string(),
            line: None,
            fix: "Validate JSON with jq or https://jsonlint.com".to_string(),
        })
    }

    /// Missing required field in JSON
    pub fn json_missing_field(field: &str, location: &str) -> Self {
        Pm4PyError::Parse(ParseError {
            format: ParseFormat::Json,
            location: location.to_string(),
            reason: format!("required field '{}' is missing", field),
            line: None,
            fix: format!(
                "Add '{}' field to JSON payload or check field name spelling",
                field
            ),
        })
    }

    /// Python pm4py not installed
    pub fn pm4py_not_installed(operation: &str) -> Self {
        Pm4PyError::PythonBridge(PythonBridgeError {
            operation: operation.to_string(),
            reason: "pm4py Python package not available".to_string(),
            fix: "Install pm4py: pip install pm4py".to_string(),
        })
    }

    /// Connector config validation error
    pub fn connector_config_invalid(connector_type: &str, reason: &str) -> Self {
        Pm4PyError::Connector(ConnectorError {
            connector_type: connector_type.to_string(),
            operation: "validate_config".to_string(),
            reason: reason.to_string(),
            fix: "Check ConnectorConfig has all required fields and valid values".to_string(),
        })
    }

    /// Webhook payload validation error
    pub fn webhook_invalid_payload(reason: &str) -> Self {
        Pm4PyError::Connector(ConnectorError {
            connector_type: "webhook".to_string(),
            operation: "extract".to_string(),
            reason: reason.to_string(),
            fix: "Ensure webhook JSON has case_id, activity, timestamp fields".to_string(),
        })
    }
}

/// Conversion from std::io::Error
impl From<io::Error> for Pm4PyError {
    fn from(err: io::Error) -> Self {
        Pm4PyError::Io(IoError {
            operation: "file operation".to_string(),
            path: None,
            kind: match err.kind() {
                io::ErrorKind::NotFound => IoErrorKind::FileNotFound,
                io::ErrorKind::PermissionDenied => IoErrorKind::PermissionDenied,
                io::ErrorKind::InvalidData => IoErrorKind::InvalidData,
                io::ErrorKind::ConnectionRefused => IoErrorKind::ConnectionRefused,
                io::ErrorKind::TimedOut => IoErrorKind::TimedOut,
                other => IoErrorKind::Other(other),
            },
            fix: match err.kind() {
                io::ErrorKind::NotFound => "Check file path exists and is readable".to_string(),
                io::ErrorKind::PermissionDenied => {
                    "Check file permissions: chmod 644 for files, 755 for directories".to_string()
                }
                io::ErrorKind::InvalidData => {
                    "Check file format matches expected type (CSV, XES, JSON)".to_string()
                }
                io::ErrorKind::ConnectionRefused => {
                    "Check server is running and port is accessible".to_string()
                }
                io::ErrorKind::TimedOut => "Increase timeout or optimize operation".to_string(),
                _ => format!("See std::io::Error docs for {:?}", err.kind()),
            },
        })
    }
}

/// Conversion from csv::Error
impl From<csv::Error> for Pm4PyError {
    fn from(err: csv::Error) -> Self {
        Pm4PyError::Io(IoError {
            operation: "CSV read/write".to_string(),
            path: None,
            kind: match err.kind() {
                csv::ErrorKind::Io(io_err) => match io_err.kind() {
                    io::ErrorKind::NotFound => IoErrorKind::FileNotFound,
                    io::ErrorKind::PermissionDenied => IoErrorKind::PermissionDenied,
                    io::ErrorKind::InvalidData => IoErrorKind::InvalidData,
                    io::ErrorKind::ConnectionRefused => IoErrorKind::ConnectionRefused,
                    io::ErrorKind::TimedOut => IoErrorKind::TimedOut,
                    other => IoErrorKind::Other(other),
                },
                _ => IoErrorKind::InvalidData,
            },
            fix: "Check CSV file format, delimiter, and column headers".to_string(),
        })
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, Pm4PyError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_missing_column_error() {
        let err = Pm4PyError::csv_missing_column("case_id", &PathBuf::from("/tmp/test.csv"));
        let msg = format!("{}", err);
        assert!(msg.contains("case_id"));
        assert!(msg.contains("not found"));
        assert!(msg.contains("with_case_id_column"));
    }

    #[test]
    fn test_empty_log_error() {
        let err = Pm4PyError::empty_log("AlphaMiner::discover");
        let msg = format!("{}", err);
        assert!(msg.contains("empty"));
        assert!(msg.contains("at least one trace"));
    }

    #[test]
    fn test_pm4py_not_installed_error() {
        let err = Pm4PyError::pm4py_not_installed("TokenReplay::check");
        let msg = format!("{}", err);
        assert!(msg.contains("pm4py"));
        assert!(msg.contains("pip install"));
    }
}
