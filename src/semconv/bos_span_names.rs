/// Audit record for configuration change operation.
///
/// Span: `span.bos.audit.config_change`
/// Kind: `server`
/// Stability: `development`
pub const BOS_AUDIT_CONFIG_CHANGE_SPAN: &str = "bos.audit.config_change";
/// Audit record for permission grant or revocation.
///
/// Span: `span.bos.audit.permission_grant`
/// Kind: `server`
/// Stability: `development`
pub const BOS_AUDIT_PERMISSION_GRANT_SPAN: &str = "bos.audit.permission_grant";
/// Recording of a compliance audit trail entry.
///
/// Span: `span.bos.audit.record`
/// Kind: `internal`
/// Stability: `development`
pub const BOS_AUDIT_RECORD_SPAN: &str = "bos.audit.record";
/// Evaluation of a single compliance rule against current workspace state.
///
/// Span: `span.bos.compliance.check`
/// Kind: `internal`
/// Stability: `development`
pub const BOS_COMPLIANCE_CHECK_SPAN: &str = "bos.compliance.check";
/// Evaluation of a compliance control against current system state.
///
/// Span: `span.bos.compliance.evaluate`
/// Kind: `internal`
/// Stability: `development`
pub const BOS_COMPLIANCE_EVALUATE_SPAN: &str = "bos.compliance.evaluate";
/// Recording of an architectural or operational decision in BusinessOS.
///
/// Span: `span.bos.decision.record`
/// Kind: `internal`
/// Stability: `development`
pub const BOS_DECISION_RECORD_SPAN: &str = "bos.decision.record";
/// Detection and classification of a compliance gap.
///
/// Span: `span.bos.gap.detect`
/// Kind: `internal`
/// Stability: `development`
pub const BOS_GAP_DETECT_SPAN: &str = "bos.gap.detect";
/// BOS gateway conformance — forwards conformance check to pm4py-rust.
///
/// Span: `span.bos.gateway.conformance`
/// Kind: `server`
/// Stability: `development`
pub const BOS_GATEWAY_CONFORMANCE_SPAN: &str = "bos.gateway.conformance";
/// BOS gateway discovery — forwards event-log discovery request to pm4py-rust.
///
/// Span: `span.bos.gateway.discover`
/// Kind: `server`
/// Stability: `development`
pub const BOS_GATEWAY_DISCOVER_SPAN: &str = "bos.gateway.discover";
/// BOS gateway statistics — forwards log-statistics extraction to pm4py-rust.
///
/// Span: `span.bos.gateway.statistics`
/// Kind: `server`
/// Stability: `development`
pub const BOS_GATEWAY_STATISTICS_SPAN: &str = "bos.gateway.statistics";
/// bos CLI SPARQL CONSTRUCT pipeline — loads PostgreSQL rows as RDF triples and writes to Oxigraph.
///
/// Span: `span.bos.ontology.execute`
/// Kind: `internal`
/// Stability: `development`
pub const BOS_ONTOLOGY_EXECUTE_SPAN: &str = "bos.ontology.execute";
/// bos CLI SPARQL query — proxies SELECT or CONSTRUCT to Oxigraph /query endpoint.
///
/// Span: `span.bos.rdf.query`
/// Kind: `client`
/// Stability: `development`
pub const BOS_RDF_QUERY_SPAN: &str = "bos.rdf.query";
/// bos CLI RDF write — forwards Turtle or N-Triples to Oxigraph /store via HTTP proxy.
///
/// Span: `span.bos.rdf.write`
/// Kind: `client`
/// Stability: `development`
pub const BOS_RDF_WRITE_SPAN: &str = "bos.rdf.write";
/// An operation performed against a BusinessOS workspace (create, update, query).
///
/// Span: `span.bos.workspace.operation`
/// Kind: `internal`
/// Stability: `development`
pub const BOS_WORKSPACE_OPERATION_SPAN: &str = "bos.workspace.operation";
