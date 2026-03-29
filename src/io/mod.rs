pub mod csv;
pub mod database;
pub mod extended_io;
pub mod json;
pub mod ocel2;
pub mod ocel2_io;
pub mod parquet;
/// Input/Output for various log formats
///
/// This module provides readers and writers for various event log formats used in process mining.
///
/// # Supported Formats
///
/// - **XES**: eXtensible Event Stream (standard XML-based format)
/// - **CSV**: Comma-separated values (simple table format)
/// - **JSON**: JavaScript Object Notation (flexible format)
/// - **Parquet**: Columnar storage (efficient for large logs)
/// - **OCEL**: Object-Centric Event Log (for object-centric process mining)
/// - **OCEL2**: Next-generation object-centric format
///
/// # Reading Event Logs
///
/// ## XES Format (Standard)
///
/// ```rust,ignore
/// use pm4py::io::XESReader;
/// use pm4py::io::LogReader;
/// use std::path::Path;
///
/// let reader = XESReader::new();
/// let event_log = reader.read(Path::new("event_log.xes"))?;
/// ```
///
/// ## CSV Format
///
/// ```rust,ignore
/// use pm4py::io::CSVReader;
/// use pm4py::io::LogReader;
/// use std::path::Path;
///
/// let reader = CSVReader::new();
/// let event_log = reader.read(Path::new("event_log.csv"))?;
/// ```
///
/// ## Auto-detect Format
///
/// ```rust,ignore
/// use pm4py::io::read_log;
/// use std::path::Path;
///
/// // Automatically detects format from file extension
/// let event_log = read_log(Path::new("event_log.xes"))?;
/// ```
///
/// # Writing Event Logs
///
/// ```rust,ignore
/// use pm4py::io::{XESWriter, LogWriter};
/// use pm4py::EventLog;
/// use std::path::Path;
///
/// let event_log = EventLog::new();
/// let writer = XESWriter::new();
/// writer.write(&event_log, Path::new("output.xes"))?;
/// ```
///
/// # CSV Configuration
///
/// For CSV files, you can configure the column mapping:
///
/// ```rust
/// use pm4py::io::CSVReader;
///
/// let reader = CSVReader::new()
///     .with_case_column("case_id")
///     .with_activity_column("activity")
///     .with_timestamp_column("timestamp")
///     .with_resource_column(Some("resource"));
/// ```
pub mod streaming_json;
pub mod xes;

pub use csv::{CSVReader, CSVWriter};
pub use extended_io::*;
pub use json::{JsonEventLogReader, JsonEventLogWriter, OcelReader, OcelWriter};
pub use ocel2::Ocel2Reader;
pub use ocel2_io::*;
pub use parquet::{ParquetReader, ParquetWriter};
pub use streaming_json::{StreamingJsonReader, StreamingJsonWriter};
pub use xes::{XESReader, XESWriter};

use crate::log::EventLog;
use anyhow::Result;
use std::path::Path;

/// Trait for log readers
///
/// All log readers implement this trait, providing a unified interface
/// for reading event logs from different formats.
///
/// # Example
///
/// ```rust,no_run
/// use pm4py::io::{XESReader, CSVReader};
/// use pm4py::EventLog;
/// use std::path::Path;
///
/// // Read XES files with XESReader
/// let xes_log = XESReader::new().read(Path::new("log.xes")).unwrap();
///
/// // Read CSV files with CSVReader
/// let csv_log = CSVReader::new().read(Path::new("log.csv")).unwrap();
/// ```
pub trait LogReader {
    /// Read an event log from a file
    ///
    /// # Parameters
    ///
    /// - `path`: Path to the log file
    ///
    /// # Returns
    ///
    /// A populated [`EventLog`] instance
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file doesn't exist
    /// - The file format is invalid
    /// - Required columns are missing (for CSV)
    fn read(&self, path: &Path) -> Result<EventLog>;
}

/// Trait for log writers
///
/// All log writers implement this trait, providing a unified interface
/// for writing event logs to different formats.
///
/// # Example
///
/// ```rust
/// use pm4py::io::{LogWriter, XESWriter};
/// use pm4py::EventLog;
/// use std::path::Path;
///
/// fn save_log(event_log: &EventLog, path: &Path) -> anyhow::Result<()> {
///     XESWriter::new().write(event_log, path)
/// }
/// ```
pub trait LogWriter {
    /// Write an event log to a file
    ///
    /// # Parameters
    ///
    /// - `log`: The event log to write
    /// - `path`: Destination file path
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The directory doesn't exist
    /// - File cannot be created
    /// - Data cannot be serialized
    fn write(&self, log: &EventLog, path: &Path) -> Result<()>;
}

/// Auto-detect and read log file by extension
///
/// This convenience function automatically selects the appropriate reader
/// based on the file extension.
///
/// # Supported Extensions
///
/// - `.xes` → XESReader
/// - `.csv` → CSVReader
/// - `.json` → JsonEventLogReader
/// - `.parquet` → ParquetReader
///
/// # Example
///
/// ```rust,ignore
/// use pm4py::io::read_log;
/// use std::path::Path;
///
/// let event_log = read_log(Path::new("event_log.xes"))?;
/// ```
///
/// # Errors
///
/// Returns an error if the file extension is not supported
pub fn read_log(path: &Path) -> Result<EventLog> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("xes") => {
            let reader = XESReader::new();
            Ok(reader.read(path)?)
        }
        Some("csv") => {
            let reader = CSVReader::new();
            Ok(reader.read(path)?)
        }
        _ => Err(anyhow::anyhow!("Unsupported file format")),
    }
}
