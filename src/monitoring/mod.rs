/// Monitoring and drift detection for process models
///
/// Provides real-time detection of process model staleness by monitoring
/// key metrics and identifying when models have drifted beyond acceptable thresholds.
pub mod drift;

pub use drift::DriftCalculator;
