/// Span emitted when CSV event data is ingested from Canopy
///
/// Span: `process.mining.canopy.ingest`
/// Kind: `consumer`
/// Stability: `development`
pub const PROCESS_MINING_CANOPY_INGEST_SPAN: &str = "process.mining.canopy.ingest";
/// Span emitted when declare constraint conformance is checked
///
/// Span: `process.mining.declare.check`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_DECLARE_CHECK_SPAN: &str = "process.mining.declare.check";
/// Span emitted when predictive analytics (next activity, remaining time, outcome) is computed
///
/// Span: `process.mining.prediction.make`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_PREDICTION_MAKE_SPAN: &str = "process.mining.prediction.make";
/// Span emitted when organizational/social network analysis is performed
///
/// Span: `process.mining.social_network.analyze`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_SOCIAL_NETWORK_ANALYZE_SPAN: &str =
    "process.mining.social_network.analyze";
