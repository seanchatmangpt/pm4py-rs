/// Object-Centric token replay conformance check
///
/// Span: `span.ocpm.conformance.check`
/// Kind: `internal`
/// Stability: `development`
pub const OCPM_CONFORMANCE_CHECK_SPAN: &str = "ocpm.conformance.check";
/// Object-Centric DFG discovery from an OCEL 2.0 log
///
/// Span: `span.ocpm.discovery.dfg`
/// Kind: `internal`
/// Stability: `development`
pub const OCPM_DISCOVERY_DFG_SPAN: &str = "ocpm.discovery.dfg";
/// Object-Centric Petri Net discovery from an OCEL 2.0 log
///
/// Span: `span.ocpm.discovery.petri_net`
/// Kind: `internal`
/// Stability: `development`
pub const OCPM_DISCOVERY_PETRI_NET_SPAN: &str = "ocpm.discovery.petri_net";
/// OCEL-grounded LLM query — RAG over real process data (Connection 4)
///
/// Span: `span.ocpm.llm.query`
/// Kind: `client`
/// Stability: `development`
pub const OCPM_LLM_QUERY_SPAN: &str = "ocpm.llm.query";
/// OCEL 2.0 log ingestion — parse and load into ObjectCentricEventLog
///
/// Span: `span.ocpm.ocel.ingest`
/// Kind: `internal`
/// Stability: `development`
pub const OCPM_OCEL_INGEST_SPAN: &str = "ocpm.ocel.ingest";
/// Object-Centric bottleneck detection — top-N edges by severity score
///
/// Span: `span.ocpm.performance.bottleneck`
/// Kind: `internal`
/// Stability: `development`
pub const OCPM_PERFORMANCE_BOTTLENECK_SPAN: &str = "ocpm.performance.bottleneck";
/// Object-Centric throughput computation — end-to-end duration per object type
///
/// Span: `span.ocpm.performance.throughput`
/// Kind: `internal`
/// Stability: `development`
pub const OCPM_PERFORMANCE_THROUGHPUT_SPAN: &str = "ocpm.performance.throughput";
