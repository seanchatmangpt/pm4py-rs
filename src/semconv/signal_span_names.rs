/// Batch aggregation of signals — collecting signals within a time window and processing them as a group.
///
/// Span: `span.signal.batch.aggregate`
/// Kind: `internal`
/// Stability: `development`
pub const SIGNAL_BATCH_AGGREGATE_SPAN: &str = "signal.batch.aggregate";
/// Classifies a signal's mode, genre, and type according to Signal Theory S=(M,G,T,F,W).
///
/// Span: `span.signal.classify`
/// Kind: `internal`
/// Stability: `development`
pub const SIGNAL_CLASSIFY_SPAN: &str = "signal.classify";
/// Compressing a signal payload before transmission — bandwidth optimization.
///
/// Span: `span.signal.compress`
/// Kind: `internal`
/// Stability: `development`
pub const SIGNAL_COMPRESS_SPAN: &str = "signal.compress";
/// Signal deserialization — decoding a received signal payload from its wire format.
///
/// Span: `span.signal.decode`
/// Kind: `internal`
/// Stability: `development`
pub const SIGNAL_DECODE_SPAN: &str = "signal.decode";
/// Encoding of a signal using the S=(M,G,T,F,W) Signal Theory model.
///
/// Span: `span.signal.encode`
/// Kind: `internal`
/// Stability: `development`
pub const SIGNAL_ENCODE_SPAN: &str = "signal.encode";
/// Applies the S/N gate to filter noise — signals below the weight threshold are rejected.
///
/// Span: `span.signal.filter`
/// Kind: `internal`
/// Stability: `development`
pub const SIGNAL_FILTER_SPAN: &str = "signal.filter";
/// Assessing the composite quality of a signal against acceptance thresholds.
///
/// Span: `span.signal.quality.assess`
/// Kind: `internal`
/// Stability: `development`
pub const SIGNAL_QUALITY_ASSESS_SPAN: &str = "signal.quality.assess";
/// Signal routing decision — determining which service or agent receives this signal.
///
/// Span: `span.signal.route`
/// Kind: `internal`
/// Stability: `development`
pub const SIGNAL_ROUTE_SPAN: &str = "signal.route";
/// Signal quality gate — filters signals below S/N ratio threshold.
///
/// Span: `span.signal.sn_gate`
/// Kind: `internal`
/// Stability: `development`
pub const SIGNAL_SN_GATE_SPAN: &str = "signal.sn_gate";
