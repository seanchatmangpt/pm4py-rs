/// Test helpers implementing Chicago TDD + WvdA + Armstrong standards.
///
/// Five helpers prevent 80% of test bugs:
/// 1. TimeoutHelper: WvdA deadlock-freedom
/// 2. BoundedLoopVerifier: WvdA liveness
/// 3. MessagePassingValidator: Armstrong no-shared-state
/// 4. BudgetEnforcer: Armstrong resource limits
/// 5. (Supervision tracking in Rust: channels replace processes)

pub mod bounded_loop_verifier;
pub mod budget_enforcer;
pub mod message_passing_validator;
pub mod timeout_helper;

// Re-export commonly used items
pub use bounded_loop_verifier::{assert_max_iterations, BoundedIterator, RecursionGuard};
pub use budget_enforcer::{assert_tier_compliant, Budget, BudgetMonitor, BudgetTier};
pub use message_passing_validator::{assert_receives_message, BoundedQueue, MessageChannel};
pub use timeout_helper::assert_completes_with_timeout;
