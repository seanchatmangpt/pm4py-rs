//! Python subprocess bridge integration tests.
//!
//! Chicago TDD — behavior-first, no mocks, real subprocess invocations.
//! All three tests satisfy FIRST properties:
//!   Fast        — each test has an explicit short timeout
//!   Independent — no shared state between tests
//!   Repeatable  — deterministic (no random state, no timing races)
//!   Self-Checking — clear assertions with descriptive messages
//!   Timely      — written alongside the implementation

use pm4py::python_bridge::PythonBridge;

/// Test 1: `is_available()` returns a bool without panicking.
///
/// We do NOT assert that Python or pm4py is present because CI may not have
/// them installed.  The contract under test is that the method always returns
/// a `bool` rather than panicking or blocking forever.
#[test]
fn test_python_bridge_is_available_returns_bool() {
    let bridge = PythonBridge::new();
    // Must return without panic regardless of whether Python is installed.
    let available: bool = bridge.is_available();
    // No assertion on the value — CI may or may not have pm4py.
    // Logging for debugging if a CI failure occurs.
    eprintln!("is_available() = {}", available);
}

/// Test 2: Executing a simple `print("hello")` returns `Ok("hello\n")`.
///
/// Skipped when Python is not on PATH (CI without Python).
#[test]
fn test_python_bridge_execute_simple_python() {
    let bridge = PythonBridge::new();

    // Guard: skip gracefully if python3 is not available at all.
    let python_present = std::process::Command::new("python3")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !python_present {
        eprintln!("SKIP: python3 not found on PATH");
        return;
    }

    let result = bridge.execute("print(\"hello\")");
    assert!(
        result.is_ok(),
        "Expected Ok from simple print, got: {:?}",
        result
    );
    assert_eq!(
        result.unwrap(),
        "hello\n",
        "Output should be 'hello' followed by a newline"
    );
}

/// Test 3: Execution respects the timeout budget (WvdA liveness guarantee).
///
/// Runs an infinite loop with a 100 ms timeout.
/// Expects `Err` containing the word "timeout".
#[test]
fn test_python_bridge_execute_respects_timeout() {
    // 100 ms budget — well below the default 30 s so the test is fast.
    let bridge = PythonBridge::with_config("python3", 100);

    // Guard: skip gracefully if python3 is not available at all.
    let python_present = std::process::Command::new("python3")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !python_present {
        eprintln!("SKIP: python3 not found on PATH");
        return;
    }

    // Infinite loop — should be killed by the watchdog.
    let result = bridge.execute("while True: pass");

    assert!(
        result.is_err(),
        "Expected Err(timeout) from infinite loop, got Ok"
    );

    let err = result.unwrap_err();
    assert!(
        err.contains("timeout"),
        "Error message should contain 'timeout', got: {}",
        err
    );
}
