/// WvdA Liveness Helper (Rust).
/// Validates loops have explicit max_iterations boundary.

/// Tracks iteration count and enforces bounds.
pub struct BoundedIterator {
    count: usize,
    max_iterations: usize,
}

impl BoundedIterator {
    /// Creates a new bounded iterator.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut iter = BoundedIterator::new(100);
    /// for item in items {
    ///     iter.advance().expect("exceeded max iterations");
    ///     process(item);
    /// }
    /// ```
    pub fn new(max_iterations: usize) -> Self {
        BoundedIterator {
            count: 0,
            max_iterations,
        }
    }

    /// Increments counter and returns error if limit exceeded.
    pub fn advance(&mut self) -> Result<(), String> {
        self.count += 1;
        if self.count > self.max_iterations {
            Err(format!(
                "iteration exceeded max: {} > {}",
                self.count, self.max_iterations
            ))
        } else {
            Ok(())
        }
    }

    /// Returns current iteration count.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Returns true if limit reached.
    pub fn is_exhausted(&self) -> bool {
        self.count >= self.max_iterations
    }
}

/// Asserts loop iteration count is bounded.
///
/// # Panics
///
/// Panics if operation returns count > max_iterations.
///
/// # Example
///
/// ```ignore
/// assert_max_iterations(100, || {
///     let mut count = 0;
///     for _ in items {
///         count += 1;
///     }
///     count
/// });
/// ```
pub fn assert_max_iterations<F>(max_iterations: usize, operation: F)
where
    F: FnOnce() -> usize,
{
    let count = operation();
    assert!(
        count <= max_iterations,
        "loop exceeded max iterations: {} > {}",
        count,
        max_iterations
    );
}

/// Asserts recursion depth doesn't exceed limit.
pub struct RecursionGuard {
    depth: usize,
    max_depth: usize,
}

impl RecursionGuard {
    /// Creates new recursion guard.
    pub fn new(max_depth: usize) -> Self {
        RecursionGuard {
            depth: 0,
            max_depth,
        }
    }

    /// Enters recursive call (increments depth).
    pub fn enter(&mut self) -> Result<(), String> {
        self.depth += 1;
        if self.depth > self.max_depth {
            Err(format!(
                "recursion exceeded max depth: {} > {}",
                self.depth, self.max_depth
            ))
        } else {
            Ok(())
        }
    }

    /// Exits recursive call (decrements depth).
    pub fn exit(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    /// Current recursion depth.
    pub fn depth(&self) -> usize {
        self.depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_iterator_allows_valid_count() {
        let mut iter = BoundedIterator::new(10);
        for _ in 0..10 {
            assert!(iter.advance().is_ok());
        }
        assert_eq!(iter.count(), 10);
    }

    #[test]
    fn test_bounded_iterator_rejects_excess() {
        let mut iter = BoundedIterator::new(5);
        for _ in 0..5 {
            let _ = iter.advance();
        }
        assert!(iter.advance().is_err());
    }

    #[test]
    fn test_assert_max_iterations() {
        assert_max_iterations(100, || 50);
    }

    #[test]
    #[should_panic(expected = "exceeded max iterations")]
    fn test_assert_max_iterations_fails() {
        assert_max_iterations(10, || 20);
    }

    #[test]
    fn test_recursion_guard() {
        let mut guard = RecursionGuard::new(100);
        for _ in 0..50 {
            guard.enter().unwrap();
        }
        assert_eq!(guard.depth(), 50);
        for _ in 0..50 {
            guard.exit();
        }
        assert_eq!(guard.depth(), 0);
    }
}
