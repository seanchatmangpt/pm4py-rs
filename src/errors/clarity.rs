/// Error clarity messages for pm4py-rust.
///
/// Provides helpful context for common Rust errors, reducing debugging time
/// by adding root cause + recovery action to error messages.

use std::fmt;

/// Semaphore or resource exhaustion error
#[derive(Debug, Clone)]
pub struct SemaphoreExhaustedError {
    pub resource: String,
    pub limit: usize,
    pub current: usize,
}

impl fmt::Display for SemaphoreExhaustedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Semaphore exhausted: {} limit={}, current={}. \
             Check if all goroutines are properly closing resources. \
             Tip: use RAII (drop trait) to ensure cleanup.",
            self.resource, self.limit, self.current
        )
    }
}

impl std::error::Error for SemaphoreExhaustedError {}

/// Buffer overflow or capacity exceeded
#[derive(Debug, Clone)]
pub struct BufferOverflowError {
    pub buffer_name: String,
    pub capacity: usize,
    pub requested: usize,
}

impl fmt::Display for BufferOverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Buffer overflow in {}: capacity={}, requested={}. \
             Add bounds checking: if data.len() >= capacity {{ trim() }}. \
             Tip: use ringbuf or bounded_vec_deque for fixed-size buffers.",
            self.buffer_name, self.capacity, self.requested
        )
    }
}

impl std::error::Error for BufferOverflowError {}

/// Parsing error with context
#[derive(Debug, Clone)]
pub struct ParseErrorWithContext {
    pub input_type: String,
    pub location: String,
    pub reason: String,
}

impl fmt::Display for ParseErrorWithContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to parse {}: {}. Location: {}. \
             Tip: check format matches schema. Use serde_json::from_str() for JSON.",
            self.input_type, self.reason, self.location
        )
    }
}

impl std::error::Error for ParseErrorWithContext {}

/// Timeout with context
#[derive(Debug, Clone)]
pub struct TimeoutError {
    pub operation: String,
    pub duration_ms: u64,
}

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Operation '{}' timed out after {}ms. \
             Check if blocking operation is truly stuck or if timeout is too short. \
             Tip: increase timeout or optimize the operation. \
             Example: timeout(Duration::from_millis({}), work()).await",
            self.operation, self.duration_ms, self.duration_ms * 2
        )
    }
}

impl std::error::Error for TimeoutError {}

/// Lock contention or mutex poisoned
#[derive(Debug, Clone)]
pub struct LockError {
    pub lock_type: String,
    pub context: String,
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} lock error in {}: \
             Hold lock as briefly as possible. Ensure all panics inside lock are caught. \
             Tip: use parking_lot::Mutex (faster, not poisonable).",
            self.lock_type, self.context
        )
    }
}

impl std::error::Error for LockError {}

/// Channel send/receive error
#[derive(Debug, Clone)]
pub struct ChannelError {
    pub direction: String, // "send" or "receive"
    pub reason: String,
}

impl fmt::Display for ChannelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hint = match self.direction.as_str() {
            "send" => "Receiver dropped or channel closed. Tip: use try_send() or check receiver alive.",
            "receive" => "Sender dropped or channel empty. Tip: use try_recv() or set timeout.",
            _ => "Check channel is properly initialized and both ends exist.",
        };
        write!(
            f,
            "Channel {} failed: {}. {}",
            self.direction, self.reason, hint
        )
    }
}

impl std::error::Error for ChannelError {}

/// Generic error with clarity hint
pub trait ClarityHint {
    /// Return helpful context for this error
    fn clarity_hint(&self) -> String;
}

/// Helper to convert standard errors to clarity messages
pub fn explain_io_error(err: &std::io::Error) -> String {
    match err.kind() {
        std::io::ErrorKind::NotFound => {
            "File not found — check path exists. Tip: use absolute paths, print path before open.".to_string()
        }
        std::io::ErrorKind::PermissionDenied => {
            "Permission denied — check file permissions with `ls -la`. Tip: chmod 644 for files, 755 for dirs.".to_string()
        }
        std::io::ErrorKind::ConnectionRefused => {
            "Connection refused — server not listening. Tip: check if server is running on correct port.".to_string()
        }
        std::io::ErrorKind::ConnectionReset => {
            "Connection reset — server closed unexpectedly. Tip: add retry logic with exponential backoff.".to_string()
        }
        std::io::ErrorKind::TimedOut => {
            "Operation timed out — network is slow. Tip: increase timeout or optimize operation.".to_string()
        }
        std::io::ErrorKind::Interrupted => {
            "Interrupted — try again. Tip: retry with backoff.".to_string()
        }
        std::io::ErrorKind::InvalidData => {
            "Invalid data format — check input matches expected format. Tip: print raw input for debugging.".to_string()
        }
        _ => format!(
            "IO error: {}. Check standard library docs for recovery strategy.",
            err
        ),
    }
}

/// Helper to explain panic causes
pub fn explain_panic_cause(cause: &str) -> String {
    let lower = cause.to_lowercase();
    if lower.contains("index") || lower.contains("bounds") {
        "Index out of bounds — check array/slice length. Tip: use .get(index) instead of [index].".to_string()
    } else if lower.contains("unwrap") || lower.contains("expect") {
        "Called unwrap() on None/Err. Tip: use ? operator or match/if let instead.".to_string()
    } else if lower.contains("overflow") {
        "Integer overflow — check input range. Tip: use checked_add(), checked_mul() methods.".to_string()
    } else if lower.contains("divide by zero") {
        "Divide by zero — check denominator != 0. Tip: if denom == 0 { return Err(...) }".to_string()
    } else if lower.contains("lock") || lower.contains("mutex") {
        "Panic in critical section — lock poisoned. Tip: use parking_lot or catch panic before lock.".to_string()
    } else {
        format!("Panic: {}. Check backtrace for cause. Tip: RUST_BACKTRACE=1 cargo run", cause)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semaphore_exhausted_display() {
        let err = SemaphoreExhaustedError {
            resource: "goroutines".to_string(),
            limit: 100,
            current: 100,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Semaphore exhausted"));
        assert!(msg.contains("goroutines"));
    }

    #[test]
    fn test_timeout_error_display() {
        let err = TimeoutError {
            operation: "fetch_data".to_string(),
            duration_ms: 5000,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("timed out"));
        assert!(msg.contains("5000"));
    }

    #[test]
    fn test_explain_panic_index() {
        let msg = explain_panic_cause("index out of bounds");
        assert!(msg.contains("Index out of bounds"));
        assert!(msg.contains(".get("));
    }

    #[test]
    fn test_explain_io_permission() {
        let io_err = std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "access denied",
        );
        let msg = explain_io_error(&io_err);
        assert!(msg.contains("Permission denied"));
        assert!(msg.contains("chmod"));
    }
}
