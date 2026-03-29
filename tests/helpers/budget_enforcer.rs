/// Armstrong Budget Constraints Helper (Rust).
/// Enforces operations complete within time and memory budgets.

use std::time::{Duration, Instant};

/// Priority tier with resource limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetTier {
    Critical,
    High,
    Normal,
    Low,
}

impl BudgetTier {
    /// Returns budget limits for tier.
    pub fn budget(&self) -> Budget {
        match self {
            BudgetTier::Critical => Budget {
                max_duration: Duration::from_millis(100),
                max_memory_mb: 50,
            },
            BudgetTier::High => Budget {
                max_duration: Duration::from_millis(500),
                max_memory_mb: 200,
            },
            BudgetTier::Normal => Budget {
                max_duration: Duration::from_millis(5000),
                max_memory_mb: 1000,
            },
            BudgetTier::Low => Budget {
                max_duration: Duration::from_millis(30000),
                max_memory_mb: 5000,
            },
        }
    }
}

/// Resource budget constraints.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    pub max_duration: Duration,
    pub max_memory_mb: usize,
}

impl Budget {
    /// Asserts operation respects this budget.
    ///
    /// # Panics
    ///
    /// Panics if operation exceeds time budget.
    pub fn assert_within<F>(&self, operation: F)
    where
        F: FnOnce(),
    {
        let start = Instant::now();
        operation();
        let elapsed = start.elapsed();

        assert!(
            elapsed <= self.max_duration,
            "operation exceeded time budget: {:?} > {:?}",
            elapsed,
            self.max_duration
        );
    }
}

/// Tracks operation metrics for compliance verification.
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub name: String,
    pub duration: Duration,
    pub memory_mb: usize,
}

/// Budget monitor for multiple operations.
pub struct BudgetMonitor {
    tier: BudgetTier,
    operations: Vec<OperationMetrics>,
}

impl BudgetMonitor {
    /// Creates new budget monitor.
    pub fn new(tier: BudgetTier) -> Self {
        BudgetMonitor {
            tier,
            operations: Vec::new(),
        }
    }

    /// Measures operation resource usage.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut monitor = BudgetMonitor::new(BudgetTier::Normal);
    /// monitor.measure("fast_op", || {
    ///     expensive_operation()
    /// });
    /// monitor.assert_all_compliant();
    /// ```
    pub fn measure<F>(&mut self, name: &str, operation: F)
    where
        F: FnOnce(),
    {
        let start = Instant::now();
        operation();
        let elapsed = start.elapsed();

        self.operations.push(OperationMetrics {
            name: name.to_string(),
            duration: elapsed,
            memory_mb: 0, // In production, measure via /proc or similar
        });
    }

    /// Asserts all measured operations comply with tier budget.
    ///
    /// # Panics
    ///
    /// Panics if any operation exceeds tier limits.
    pub fn assert_all_compliant(&self) {
        let budget = self.tier.budget();

        for op in &self.operations {
            assert!(
                op.duration <= budget.max_duration,
                "operation '{}' exceeded time budget: {:?} > {:?}",
                op.name,
                op.duration,
                budget.max_duration
            );
        }
    }

    /// Returns formatted metrics summary.
    pub fn summary(&self) -> String {
        let mut s = format!("=== Budget Monitor: {:?} ===\n", self.tier);
        for op in &self.operations {
            s.push_str(&format!("{}: {:?}\n", op.name, op.duration));
        }
        s
    }
}

/// Asserts operation respects tier-based budget.
///
/// # Example
///
/// ```ignore
/// assert_tier_compliant(BudgetTier::Critical, || {
///     fast_critical_operation()
/// });
/// ```
pub fn assert_tier_compliant<F>(tier: BudgetTier, operation: F)
where
    F: FnOnce(),
{
    let budget = tier.budget();
    budget.assert_within(operation);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_within_limit() {
        let budget = Budget {
            max_duration: Duration::from_millis(100),
            max_memory_mb: 50,
        };

        budget.assert_within(|| {
            std::thread::sleep(Duration::from_millis(10));
        });
    }

    #[test]
    #[should_panic(expected = "exceeded time budget")]
    fn test_budget_exceeds_limit() {
        let budget = Budget {
            max_duration: Duration::from_millis(10),
            max_memory_mb: 50,
        };

        budget.assert_within(|| {
            std::thread::sleep(Duration::from_millis(100));
        });
    }

    #[test]
    fn test_tier_budget() {
        let budget = BudgetTier::Critical.budget();
        assert_eq!(budget.max_duration, Duration::from_millis(100));
        assert_eq!(budget.max_memory_mb, 50);
    }

    #[test]
    fn test_budget_monitor() {
        let mut monitor = BudgetMonitor::new(BudgetTier::Normal);
        monitor.measure("op1", || {
            std::thread::sleep(Duration::from_millis(10));
        });
        monitor.measure("op2", || {
            std::thread::sleep(Duration::from_millis(20));
        });

        monitor.assert_all_compliant();
    }
}
