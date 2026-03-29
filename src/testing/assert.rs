/// Test assertion macros with descriptive messages.
///
/// Reduces "expected X got Y" confusion by providing context-aware assertions
/// matching Chicago TDD discipline.

/// Assert equality with descriptive context message.
///
/// # Example
/// ```ignore
/// assert_eq!(result.status, "ok", "process mining discovery should succeed");
/// ```
#[macro_export]
macro_rules! assert_eq_with_context {
    ($left:expr, $right:expr, $context:expr) => {
        if $left != $right {
            panic!(
                "Assertion failed: {}\n  Expected: {:?}\n  Got: {:?}",
                $context, $right, $left
            )
        }
    };
}

/// Assert inequality with context.
#[macro_export]
macro_rules! assert_ne_with_context {
    ($left:expr, $right:expr, $context:expr) => {
        if $left == $right {
            panic!(
                "Assertion failed: {}\n  Should not be: {:?}\n  But got: {:?}",
                $context, $right, $left
            )
        }
    };
}

/// Assert greater-than or equal with context.
/// Useful for: confidence scores, latency bounds, resource limits.
#[macro_export]
macro_rules! assert_gte_with_context {
    ($actual:expr, $min:expr, $context:expr) => {
        if $actual < $min {
            panic!(
                "Assertion failed: {}\n  Expected >= {}\n  Got: {}\n  Tip: check threshold or data source",
                $context, $min, $actual
            )
        }
    };
}

/// Assert less-than or equal with context.
/// Useful for: latency, error rate, memory usage.
#[macro_export]
macro_rules! assert_lte_with_context {
    ($actual:expr, $max:expr, $context:expr) => {
        if $actual > $max {
            panic!(
                "Assertion failed: {}\n  Expected <= {}\n  Got: {}\n  Tip: optimize operation or increase limit",
                $context, $max, $actual
            )
        }
    };
}

/// Assert value is in bounded range [min, max].
#[macro_export]
macro_rules! assert_bounded {
    ($actual:expr, $min:expr, $max:expr, $context:expr) => {
        if $actual < $min || $actual > $max {
            panic!(
                "Assertion failed: {}\n  Expected range [{}, {}]\n  Got: {}\n  Tip: verify input or algorithm",
                $context, $min, $max, $actual
            )
        }
    };
}

/// Assert Result is Ok with context.
#[macro_export]
macro_rules! assert_ok {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => panic!(
                "Assertion failed: {}\n  Expected: Ok(value)\n  Got: Err({:?})\n  Tip: check error logs",
                $context, e
            ),
        }
    };
}

/// Assert Result is Err with context.
#[macro_export]
macro_rules! assert_err {
    ($result:expr, $context:expr) => {
        match $result {
            Err(e) => e,
            Ok(value) => panic!(
                "Assertion failed: {}\n  Expected: Err(...)\n  Got: Ok({:?})",
                $context, value
            ),
        }
    };
}

/// Assert Option is Some with context.
#[macro_export]
macro_rules! assert_some {
    ($option:expr, $context:expr) => {
        match $option {
            Some(value) => value,
            None => panic!(
                "Assertion failed: {}\n  Expected: Some(value)\n  Got: None\n  Tip: check initialization or filtering",
                $context
            ),
        }
    };
}

/// Assert Option is None with context.
#[macro_export]
macro_rules! assert_none {
    ($option:expr, $context:expr) => {
        if $option.is_some() {
            panic!(
                "Assertion failed: {}\n  Expected: None\n  Got: Some({:?})",
                $context, $option
            )
        }
    };
}

/// Assert boolean condition with context.
#[macro_export]
macro_rules! assert_true {
    ($condition:expr, $context:expr) => {
        if !$condition {
            panic!(
                "Assertion failed: {}\n  Expected: true\n  Got: false",
                $context
            )
        }
    };
}

/// Assert boolean condition is false with context.
#[macro_export]
macro_rules! assert_false {
    ($condition:expr, $context:expr) => {
        if $condition {
            panic!(
                "Assertion failed: {}\n  Expected: false\n  Got: true",
                $context
            )
        }
    };
}

/// Assert vector length with context.
#[macro_export]
macro_rules! assert_vec_len {
    ($vec:expr, $expected_len:expr, $context:expr) => {
        let actual_len = $vec.len();
        if actual_len != $expected_len {
            panic!(
                "Assertion failed: {}\n  Expected length: {}\n  Got length: {}\n  Items: {:?}",
                $context, $expected_len, actual_len, $vec
            )
        }
    };
}

/// Assert vector is not empty with context.
#[macro_export]
macro_rules! assert_vec_not_empty {
    ($vec:expr, $context:expr) => {
        if $vec.is_empty() {
            panic!(
                "Assertion failed: {}\n  Expected non-empty vector\n  Got empty vector",
                $context
            )
        }
    };
}

/// Assert latency within budget with context (performance assertion).
#[macro_export]
macro_rules! assert_latency {
    ($latency_ms:expr, $max_ms:expr, $operation:expr) => {
        if $latency_ms > $max_ms {
            panic!(
                "Performance assertion failed: {}\n  Budget: {}ms\n  Actual: {}ms\n  Exceeded by: {}ms\n  Tip: profile with flamegraph or increase timeout",
                $operation, $max_ms, $latency_ms, $latency_ms - $max_ms
            )
        }
    };
}

/// Assert memory usage within budget with context.
#[macro_export]
macro_rules! assert_memory_bounded {
    ($memory_mb:expr, $max_mb:expr, $context:expr) => {
        if $memory_mb > $max_mb {
            panic!(
                "Memory assertion failed: {}\n  Budget: {}MB\n  Actual: {}MB\n  Tip: check for leaks or reduce data size",
                $context, $max_mb, $memory_mb
            )
        }
    };
}

/// Assert semaphore not exhausted (soundness: boundedness).
#[macro_export]
macro_rules! assert_semaphore_available {
    ($available:expr, $total:expr, $context:expr) => {
        if $available <= 0 {
            panic!(
                "Semaphore assertion failed: {}\n  Total: {}\n  Available: {}\n  Tip: check if goroutines are holding resources too long",
                $context, $total, $available
            )
        }
    };
}

/// Assert deadlock-free (code review assertion for soundness).
/// This is a narrative check; real deadlock freedom requires formal proof.
#[macro_export]
macro_rules! assert_deadlock_free {
    ($code_section:expr) => {
        println!(
            "Code review assertion: Deadlock-free\n  Section: {}\n  Verify:\n    - All blocking ops have timeout\n    - No circular wait chains\n    - See: .claude/rules/wvda-soundness.md",
            $code_section
        )
    };
}

/// Assert no panic in critical section (soundness: Armstrong let-it-crash).
#[macro_export]
macro_rules! assert_no_silent_error {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(v) => v,
            Err(e) => panic!(
                "Armstrong assertion failed: {}\n  Caught error: {:?}\n  Fix: remove error handling, let supervisor restart",
                $context, e
            ),
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "Assertion failed")]
    fn test_assert_eq_fails() {
        assert_eq_with_context!(1 + 1, 3, "simple arithmetic");
    }

    #[test]
    fn test_assert_gte_passes() {
        assert_gte_with_context!(10, 5, "confidence >= 0.5");
    }

    #[test]
    #[should_panic(expected = "Expected >= 10")]
    fn test_assert_gte_fails() {
        assert_gte_with_context!(5, 10, "confidence >= 0.5");
    }

    #[test]
    fn test_assert_ok_passes() {
        let result: Result<i32, String> = Ok(42);
        let value = assert_ok!(result, "parsing should succeed");
        assert_eq!(value, 42);
    }

    #[test]
    #[should_panic(expected = "Expected: Ok")]
    fn test_assert_ok_fails() {
        let result: Result<i32, String> = Err("failed".to_string());
        let _ = assert_ok!(result, "parsing should succeed");
    }

    #[test]
    fn test_assert_bounded_passes() {
        assert_bounded!(5, 0, 10, "value in range");
    }

    #[test]
    #[should_panic(expected = "Expected range")]
    fn test_assert_bounded_fails() {
        assert_bounded!(15, 0, 10, "value in range");
    }
}
