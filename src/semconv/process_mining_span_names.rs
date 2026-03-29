/// Alignment analysis — examining multiple alignment results to identify common deviation patterns and fitness trends.
///
/// Span: `span.process.mining.alignment.analyze`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_ALIGNMENT_ANALYZE_SPAN: &str = "process.mining.alignment.analyze";
/// Bottleneck analysis — scoring and ranking detected bottlenecks by severity and impact.
///
/// Span: `span.process.mining.bottleneck.analyze`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_BOTTLENECK_ANALYZE_SPAN: &str = "process.mining.bottleneck.analyze";
/// Bottleneck detection — identifying the activity with the highest average waiting time.
///
/// Span: `span.process.mining.bottleneck_detection`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_BOTTLENECK_DETECTION_SPAN: &str = "process.mining.bottleneck_detection";
/// Case clustering — grouping process cases by behavioral similarity using ML clustering algorithms.
///
/// Span: `span.process.mining.case.cluster`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_CASE_CLUSTER_SPAN: &str = "process.mining.case.cluster";
/// Process complexity measurement — computing complexity metrics for a discovered process model.
///
/// Span: `span.process.mining.complexity.measure`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_COMPLEXITY_MEASURE_SPAN: &str = "process.mining.complexity.measure";
/// Detection of a single conformance deviation during trace alignment.
///
/// Span: `span.process.mining.conformance.deviation`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_CONFORMANCE_DEVIATION_SPAN: &str = "process.mining.conformance.deviation";
/// Conformance repair — automatically repairing a non-conformant trace to align with the process model.
///
/// Span: `span.process.mining.conformance.repair`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_CONFORMANCE_REPAIR_SPAN: &str = "process.mining.conformance.repair";
/// Conformance threshold check — evaluates all cases against the defined conformance threshold and reports violations.
///
/// Span: `span.process.mining.conformance.threshold`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_CONFORMANCE_THRESHOLD_SPAN: &str = "process.mining.conformance.threshold";
/// Generating a conformance visualization — token replay, alignment diagram, or footprint matrix.
///
/// Span: `span.process.mining.conformance.visualize`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_CONFORMANCE_VISUALIZE_SPAN: &str = "process.mining.conformance.visualize";
/// Mining decision rules from a process log — discovers conditions that determine process branching.
///
/// Span: `span.process.mining.decision.mine`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_DECISION_MINE_SPAN: &str = "process.mining.decision.mine";
/// Detection of a single conformance deviation during trace alignment.
///
/// Span: `span.process.mining.deviation`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_DEVIATION_SPAN: &str = "process.mining.deviation";
/// Computation of a Directly-Follows Graph (DFG) from an event log.
///
/// Span: `span.process.mining.dfg`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_DFG_SPAN: &str = "process.mining.dfg";
/// Computation of a Directly-Follows Graph from an event log.
///
/// Span: `span.process.mining.dfg.compute`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_DFG_COMPUTE_SPAN: &str = "process.mining.dfg.compute";
/// Process model discovery run — applying a mining algorithm to an event log to produce a Petri net or BPMN model.
///
/// Span: `span.process.mining.discovery`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_DISCOVERY_SPAN: &str = "process.mining.discovery";
/// Process drift correction — applying model adaptation to address detected concept drift.
///
/// Span: `span.process.mining.drift.correct`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_DRIFT_CORRECT_SPAN: &str = "process.mining.drift.correct";
/// Detecting concept drift in a streaming process mining window.
///
/// Span: `span.process.mining.drift.detect`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_DRIFT_DETECT_SPAN: &str = "process.mining.drift.detect";
/// Event abstraction — mapping raw low-level events to higher-level process activities.
///
/// Span: `span.process.mining.event.abstract`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_EVENT_ABSTRACT_SPAN: &str = "process.mining.event.abstract";
/// Building a process hierarchy tree from process mining trace data.
///
/// Span: `span.process.mining.hierarchy.build`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_HIERARCHY_BUILD_SPAN: &str = "process.mining.hierarchy.build";
/// Inductive miner algorithm — discovers a process model by recursively partitioning the event log using cut semantics.
///
/// Span: `span.process.mining.inductive.mine`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_INDUCTIVE_MINE_SPAN: &str = "process.mining.inductive.mine";
/// Preprocessing an event log — filtering, sorting, and preparing for mining or conformance.
///
/// Span: `span.process.mining.log.preprocess`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_LOG_PREPROCESS_SPAN: &str = "process.mining.log.preprocess";
/// Process model enhancement — augmenting a discovered model with performance, conformance, or organizational perspectives.
///
/// Span: `span.process.mining.model.enhance`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_MODEL_ENHANCE_SPAN: &str = "process.mining.model.enhance";
/// Quality assessment of an enhanced process model — measures coverage, fitness improvement, and enhancement perspective.
///
/// Span: `span.process.mining.model.quality`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_MODEL_QUALITY_SPAN: &str = "process.mining.model.quality";
/// Process outcome prediction — forecasting future trace completion, bottlenecks, or deviations using a predictive model.
///
/// Span: `span.process.mining.prediction.make`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_PREDICTION_MAKE_SPAN: &str = "process.mining.prediction.make";
/// Alignment-based conformance checking — computing optimal alignments between log and model.
///
/// Span: `span.process.mining.replay.alignment`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_REPLAY_ALIGNMENT_SPAN: &str = "process.mining.replay.alignment";
/// Token replay conformance check — replaying a trace against a Petri net model to measure fitness.
///
/// Span: `span.process.mining.replay.check`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_REPLAY_CHECK_SPAN: &str = "process.mining.replay.check";
/// Replay comparison — comparing fitness scores between baseline and target process models.
///
/// Span: `span.process.mining.replay.compare`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_REPLAY_COMPARE_SPAN: &str = "process.mining.replay.compare";
/// Root cause analysis of a process anomaly — identifies why a deviation occurred.
///
/// Span: `span.process.mining.root_cause.analyze`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_ROOT_CAUSE_ANALYZE_SPAN: &str = "process.mining.root_cause.analyze";
/// Running a process simulation — generates synthetic event logs from a discovered model.
///
/// Span: `span.process.mining.simulation.run`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_SIMULATION_RUN_SPAN: &str = "process.mining.simulation.run";
/// Social network analysis of a process log — discovering collaboration patterns, handover-of-work, and resource roles.
///
/// Span: `span.process.mining.social_network.analyze`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_SOCIAL_NETWORK_ANALYZE_SPAN: &str =
    "process.mining.social_network.analyze";
/// Ingesting an event batch into the streaming process mining window.
///
/// Span: `span.process.mining.streaming.ingest`
/// Kind: `consumer`
/// Stability: `development`
pub const PROCESS_MINING_STREAMING_INGEST_SPAN: &str = "process.mining.streaming.ingest";
/// Temporal analysis of a process — detecting drift, seasonality, and trend patterns.
///
/// Span: `span.process.mining.temporal.analyze`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_TEMPORAL_ANALYZE_SPAN: &str = "process.mining.temporal.analyze";
/// Analysis of process variants — identifying distinct execution patterns and their frequencies in the event log.
///
/// Span: `span.process.mining.variant.analyze`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_VARIANT_ANALYZE_SPAN: &str = "process.mining.variant.analyze";
/// Process variant analysis — identifying and ranking unique execution paths in the event log.
///
/// Span: `span.process.mining.variant_analysis`
/// Kind: `internal`
/// Stability: `development`
pub const PROCESS_MINING_VARIANT_ANALYSIS_SPAN: &str = "process.mining.variant_analysis";
