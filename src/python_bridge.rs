//! Python subprocess bridge for heavy pm4py operations not yet ported to Rust.
//!
//! Spawns a Python process with a 30 s WvdA liveness budget (30 000 ms).
//! Used for: pm4py-native DFG rendering, OCEL validation, conformance algorithms.
//!
//! # Design
//!
//! This bridge avoids a compile-time PyO3 dependency: it simply forks `python3`
//! as a child process, feeds a script via stdin, and reads stdout.
//! Timeout is enforced via a background thread that sends SIGKILL after the
//! budget expires — WvdA liveness guarantee.
//!
//! # Example
//!
//! ```rust
//! use pm4py::python_bridge::PythonBridge;
//!
//! let bridge = PythonBridge::new();
//! if bridge.is_available() {
//!     let result = bridge.execute("print('hello')");
//!     assert!(result.is_ok());
//! }
//! ```

use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Python subprocess bridge.
///
/// All operations are bounded by `DEFAULT_TIMEOUT_MS` (WvdA liveness property).
pub struct PythonBridge {
    /// Path or name of the Python executable (default: `"python3"`).
    python_path: String,
    /// Execution timeout in milliseconds (default: 30 000 ms).
    timeout_ms: u64,
}

/// Default execution timeout — WvdA bounded execution (30 s).
const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// Availability check timeout — short probe (5 s).
const AVAILABILITY_TIMEOUT_MS: u64 = 5_000;

impl Default for PythonBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonBridge {
    /// Create a new bridge with default settings.
    ///
    /// Uses `python3` from `PATH` with a 30 000 ms timeout.
    pub fn new() -> Self {
        Self {
            python_path: "python3".to_string(),
            timeout_ms: DEFAULT_TIMEOUT_MS,
        }
    }

    /// Create a bridge with a custom Python path and timeout.
    ///
    /// # Arguments
    ///
    /// * `python_path` — Path to the Python executable (e.g. `/usr/bin/python3`).
    /// * `timeout_ms`  — Maximum execution time in milliseconds.
    pub fn with_config(python_path: impl Into<String>, timeout_ms: u64) -> Self {
        Self {
            python_path: python_path.into(),
            timeout_ms,
        }
    }

    /// Execute a Python snippet and return stdout as a `String`.
    ///
    /// Enforces `self.timeout_ms` via a background watchdog thread.
    /// If the process does not finish in time, it is killed and
    /// `Err("timeout: process exceeded <N>ms")` is returned.
    ///
    /// # Arguments
    ///
    /// * `script` — Arbitrary Python source code.
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` describing the failure:
    /// - `"timeout: process exceeded <N>ms"` — budget exceeded
    /// - `"spawn failed: <io-error>"` — could not start Python
    /// - `"python error (exit <code>): <stderr>"` — non-zero exit
    pub fn execute(&self, script: &str) -> Result<String, String> {
        let child = Command::new(&self.python_path)
            .arg("-c")
            .arg(script)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn failed: {}", e))?;

        let timeout = Duration::from_millis(self.timeout_ms);

        // Watchdog: if the child does not finish within the budget, kill it.
        let (tx, rx) = mpsc::channel::<()>();
        let child_id = child.id();
        let timeout_copy = timeout;
        thread::spawn(move || {
            thread::sleep(timeout_copy);
            // If the main thread hasn't sent the "done" signal, kill the process.
            if rx.try_recv().is_err() {
                // Best-effort kill — ignore errors (process may have already exited).
                let _ = Command::new("kill")
                    .args(["-9", &child_id.to_string()])
                    .status();
            }
        });

        let output = child
            .wait_with_output()
            .map_err(|e| format!("wait failed: {}", e))?;

        // Signal watchdog that we finished in time.
        let _ = tx.send(());

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            // Distinguish timeout kills (SIGKILL → exit code 137 or -9) from
            // normal Python errors.
            let code = output.status.code().unwrap_or(-1);
            if code == 137 || code == -1 {
                return Err(format!("timeout: process exceeded {}ms", self.timeout_ms));
            }
            return Err(format!("python error (exit {}): {}", code, stderr.trim()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Check whether Python and the `pm4py` package are available.
    ///
    /// Runs `python3 -c "import pm4py; print('ok')"` with a 5 s probe timeout.
    /// Returns `true` if the import succeeds, `false` otherwise.
    /// Does NOT panic — safe to call in CI environments where Python is absent.
    pub fn is_available(&self) -> bool {
        let probe = PythonBridge::with_config(&self.python_path, AVAILABILITY_TIMEOUT_MS);
        probe
            .execute("import pm4py; print('ok')")
            .map(|out| out.trim() == "ok")
            .unwrap_or(false)
    }

    /// Execute a pm4py process-discovery script and return stdout.
    ///
    /// Convenience wrapper that sets up a standard Python script preamble
    /// (imports, JSON serialisation) before running `discovery_script`.
    ///
    /// # Arguments
    ///
    /// * `discovery_script` — Python statements that produce JSON on stdout.
    ///
    /// The caller is responsible for printing valid JSON as the last line.
    pub fn discover(&self, discovery_script: &str) -> Result<String, String> {
        let full_script = format!("import json\nimport sys\n{}\n", discovery_script);
        self.execute(&full_script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify bridge construction uses sensible defaults.
    #[test]
    fn test_python_bridge_new_has_default_timeout() {
        let bridge = PythonBridge::new();
        assert_eq!(bridge.python_path, "python3");
        assert_eq!(bridge.timeout_ms, DEFAULT_TIMEOUT_MS);
    }

    /// Verify `with_config` stores custom values.
    #[test]
    fn test_python_bridge_with_config_stores_values() {
        let bridge = PythonBridge::with_config("/usr/bin/python3", 1_000);
        assert_eq!(bridge.python_path, "/usr/bin/python3");
        assert_eq!(bridge.timeout_ms, 1_000);
    }
}
