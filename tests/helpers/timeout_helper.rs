/// WvdA Deadlock-Freedom Helper (Rust).
/// Enforces all blocking operations complete within timeout.

use std::time::{Duration, Instant};

/// Asserts operation completes within timeout or panics.
///
/// # Panics
///
/// Panics if timeout_ms is None or operation exceeds boundary.
///
/// # Example
///
/// ```ignore
/// assert_completes_with_timeout(Some(5000), || {
///     expensive_operation()
/// });
/// ```
pub fn assert_completes_with_timeout<F>(timeout_ms: Option<u64>, operation: F) -> ()
where
    F: FnOnce() -> (),
{
    let timeout_ms = timeout_ms.expect("timeout_ms is required (WvdA deadlock-freedom)");

    let start = Instant::now();
    operation();
    let elapsed = start.elapsed();

    let timeout_dur = Duration::from_millis(timeout_ms);
    assert!(
        elapsed <= timeout_dur,
        "operation exceeded timeout: {:?} > {:?}",
        elapsed,
        timeout_dur
    );
}

/// Asserts async operation completes within timeout.
///
/// # Example
///
/// ```ignore
/// assert_completes_with_timeout_async(5000, async {
///     expensive_async_op().await
/// }).await;
/// ```
#[cfg(feature = "tokio")]
pub async fn assert_completes_with_timeout_async<F>(
    timeout_ms: u64,
    operation: F,
) -> ()
where
    F: std::future::Future<Output = ()>,
{
    let timeout = Duration::from_millis(timeout_ms);
    let result = tokio::time::timeout(timeout, operation).await;

    match result {
        Ok(_) => {}
        Err(tokio::time::error::Elapsed { .. }) => {
            panic!("async operation exceeded timeout: {:?}", timeout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completes_within_timeout() {
        assert_completes_with_timeout(Some(100), || {
            std::thread::sleep(Duration::from_millis(10));
        });
    }

    #[test]
    #[should_panic(expected = "operation exceeded timeout")]
    fn test_panics_on_timeout_exceeded() {
        assert_completes_with_timeout(Some(10), || {
            std::thread::sleep(Duration::from_millis(100));
        });
    }

    #[test]
    #[should_panic(expected = "timeout_ms is required")]
    fn test_panics_on_missing_timeout() {
        assert_completes_with_timeout(None, || {});
    }
}
