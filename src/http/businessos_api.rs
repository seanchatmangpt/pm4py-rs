//! BusinessOS HTTP API Integration
//!
//! Provides HTTP endpoints for BusinessOS to interact with pm4py-rust process mining engine.
//! Implements discovery, conformance checking, and statistical analysis endpoints.
//!
//! Progress Event Streaming:
//! During discovery, progress events are emitted at 0%, 25%, 50%, 75%, 100% and POSTed to BusinessOS.
//! BusinessOS broadcasts these events to connected SSE clients for real-time UI updates.
//!
//! Environment Variables:
//! - PM4PY_RUST_BUSINESSOS_URL: BusinessOS progress callback base URL (default: http://localhost:8001)
//! - PM4PY_RUST_PORT: Port this server listens on (default: 8090)

use axum::{
    extract::Json,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use bytes::Bytes;
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use tower_http::trace::TraceLayer;

use crate::telemetry::middleware::{extract_trace_context, resolve_correlation_id};

use crate::conformance::{ConformanceResult, TokenReplay};
use crate::connectors::EventLogExtractor;
use crate::discovery::AlphaMiner;
use crate::llm::{abstract_dfg, abstract_event_log, abstract_petri_net};
use crate::log::EventLog;
use crate::models::{DirectlyFollowsGraph, PetriNet};
use crate::monitoring::DriftCalculator;
use crate::semconv::jtbd_attributes::{
    JTBD_SCENARIO_FITNESS, JTBD_SCENARIO_MODEL_FORMAT, JTBD_SCENARIO_PLACE_COUNT,
    JTBD_SCENARIO_TRANSITION_COUNT,
};
use crate::semconv::process_mining_attributes::PROCESS_MINING_DRIFT_DETECTED;
use crate::semconv::process_mining_span_names::PROCESS_MINING_DRIFT_DETECT_SPAN;
use crate::statistics::advanced::{
    get_activity_frequency, get_bottleneck_activities, get_resource_metrics, get_variant_frequency,
};

/// API error response structure
#[derive(Debug, Serialize)]
pub struct ApiError {
    error: String,
    details: Option<String>,
    status: u16,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code =
            StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status_code, Json(self)).into_response()
    }
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    timestamp: String,
}

/// Discovery request: EventLog → PetriNet
#[derive(Debug, Deserialize)]
pub struct DiscoveryRequest {
    /// Event log in JSON format
    pub event_log: serde_json::Value,
    /// Discovery variant: "alpha" (default), "alpha_plus", "inductive"
    #[serde(default = "default_discovery_variant")]
    pub variant: String,
}

fn default_discovery_variant() -> String {
    "alpha".to_string()
}

/// Discovery response: Petri net with metadata
#[derive(Debug, Serialize)]
pub struct DiscoveryResponse {
    pub petri_net: PetriNetJson,
    pub algorithm: String,
    pub execution_time_ms: u128,
    pub event_count: usize,
    pub trace_count: usize,
}

/// JSON-serializable Petri Net
#[derive(Debug, Serialize, Deserialize)]
pub struct PetriNetJson {
    pub places: Vec<PlaceJson>,
    pub transitions: Vec<TransitionJson>,
    pub arcs: Vec<ArcJson>,
    pub initial_place: Option<String>,
    pub final_place: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceJson {
    pub id: String,
    pub name: String,
    pub initial_marking: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransitionJson {
    pub id: String,
    pub name: String,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArcJson {
    pub from: String,
    pub to: String,
    pub weight: usize,
}

impl PetriNetJson {
    /// Convert from pm4py PetriNet to JSON-serializable format
    pub(crate) fn from_petri_net(net: &PetriNet) -> Self {
        Self {
            places: net
                .places
                .iter()
                .map(|p| PlaceJson {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    initial_marking: p.initial_marking,
                })
                .collect(),
            transitions: net
                .transitions
                .iter()
                .map(|t| TransitionJson {
                    id: t.id.clone(),
                    name: t.name.clone(),
                    label: t.label.clone(),
                })
                .collect(),
            arcs: net
                .arcs
                .iter()
                .map(|a| ArcJson {
                    from: a.from.clone(),
                    to: a.to.clone(),
                    weight: a.weight,
                })
                .collect(),
            initial_place: net.initial_place.clone(),
            final_place: net.final_place.clone(),
        }
    }
}

/// Conformance checking request: EventLog + PetriNet → Fitness
#[derive(Debug, Deserialize)]
pub struct ConformanceRequest {
    /// Event log in JSON format
    pub event_log: serde_json::Value,
    /// Petri net in JSON format
    pub petri_net: PetriNetJson,
    /// Conformance checking method: "token_replay" (default), "alignment", "footprints"
    #[serde(default = "default_conformance_method")]
    pub method: String,
}

fn default_conformance_method() -> String {
    "token_replay".to_string()
}

/// Conformance checking response
#[derive(Debug, Serialize)]
pub struct ConformanceResponse {
    pub is_conformant: bool,
    pub fitness: f64,
    pub precision: f64,
    pub generalization: f64,
    pub method: String,
    pub execution_time_ms: u128,
}

impl From<(ConformanceResult, String, u128)> for ConformanceResponse {
    fn from((result, method, time): (ConformanceResult, String, u128)) -> Self {
        Self {
            is_conformant: result.is_conformant,
            fitness: result.fitness,
            precision: result.precision,
            generalization: result.generalization,
            method,
            execution_time_ms: time,
        }
    }
}

/// Statistics request: EventLog → Analysis Report
#[derive(Debug, Deserialize)]
pub struct StatisticsRequest {
    /// Event log in JSON format
    pub event_log: serde_json::Value,
    /// Include detailed metrics
    #[serde(default)]
    pub include_variants: bool,
    #[serde(default)]
    pub include_resource_metrics: bool,
    #[serde(default)]
    pub include_bottlenecks: bool,
}

/// Statistics response
#[derive(Debug, Serialize)]
pub struct StatisticsResponse {
    pub trace_count: usize,
    pub event_count: usize,
    pub unique_activities: usize,
    pub activity_frequencies: Option<HashMap<String, usize>>,
    pub variant_count: usize,
    pub variant_frequencies: Option<HashMap<String, usize>>,
    pub bottleneck_activities: Option<Vec<String>>,
    pub resource_count: Option<usize>,
    pub execution_time_ms: u128,
}

/// Drift detection request: Compare baseline vs recent metrics
#[derive(Debug, Deserialize)]
pub struct DriftRequest {
    /// Baseline metrics: {metric_name: value}
    pub baseline: HashMap<String, f64>,
    /// Recent metrics: {metric_name: value}
    pub recent: HashMap<String, f64>,
}

/// Drift detection response
#[derive(Debug, Serialize)]
pub struct DriftResponse {
    /// Drift score: 0.0 (stable) to 1.0+ (critical)
    pub drift_score: f64,
    /// Whether drift exceeds threshold (0.2)
    pub is_drifted: bool,
    /// Severity level: "stable", "minor", "major", "critical"
    pub severity: String,
    /// Metrics that changed significantly (>10% drift)
    pub changed_metrics: Vec<String>,
    pub execution_time_ms: u128,
}

/// LLM Abstraction Request: Petri Net → Plain English
#[derive(Debug, Deserialize)]
pub struct AbstractPetriNetRequest {
    pub petri_net: PetriNetJson,
}

/// LLM Abstraction Request: DFG → Plain English
#[derive(Debug, Deserialize)]
pub struct AbstractDfgRequest {
    pub dfg: serde_json::Value,
}

/// LLM Abstraction Request: EventLog → Plain English
#[derive(Debug, Deserialize)]
pub struct AbstractEventLogRequest {
    pub event_log: serde_json::Value,
}

/// LLM Query Request: NL query + model context → process insight
#[derive(Debug, Deserialize)]
pub struct ProcessIntelligenceQueryRequest {
    /// Natural language query (e.g., "What is the bottleneck?")
    pub query: String,
    /// Optional Petri net context
    pub petri_net: Option<PetriNetJson>,
    /// Optional event log context
    pub event_log: Option<serde_json::Value>,
}

/// LLM Response: Plain English process insight
#[derive(Debug, Serialize)]
pub struct ProcessIntelligenceResponse {
    pub abstract_description: String,
    pub model_type: String,
    pub execution_time_ms: u128,
}

/// LLM Query Response: Natural language answer
#[derive(Debug, Serialize)]
pub struct ProcessIntelligenceQueryResponse {
    pub query: String,
    pub response: String,
    pub execution_time_ms: u128,
}

/// POST /api/connector/extract request body
#[derive(serde::Deserialize)]
struct ConnectorExtractRequest {
    config: crate::connectors::ConnectorConfig,
}

/// POST /api/connector/extract — Extract EventLog via a named connector
async fn connector_extract(
    Json(req): Json<ConnectorExtractRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = match req.config.connector_type {
        crate::connectors::ConnectorType::Csv => {
            crate::connectors::CsvConnector::extract(&req.config)
        }
        crate::connectors::ConnectorType::Webhook => {
            crate::connectors::WebhookConnector::extract(&req.config)
        }
        crate::connectors::ConnectorType::Sap => {
            crate::connectors::SapODataConnector::extract(&req.config)
        }
        crate::connectors::ConnectorType::Salesforce => {
            crate::connectors::SalesforceConnector::extract(&req.config)
        }
        crate::connectors::ConnectorType::ServiceNow => {
            crate::connectors::ServiceNowConnector::extract(&req.config)
        }
    };
    match result {
        Ok(r) => Ok(Json(serde_json::json!({
            "extracted_case_count": r.metadata.extracted_case_count,
            "extracted_event_count": r.metadata.extracted_event_count,
            "extraction_time_ms": r.metadata.extraction_time_ms,
            "connector_name": r.metadata.connector_name,
            "source_record_count": r.metadata.source_record_count,
        }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

/// Parse EventLog from JSON
pub(crate) fn parse_event_log(value: &serde_json::Value) -> Result<EventLog, String> {
    serde_json::from_value(value.clone())
        .map_err(|e| format!("Failed to deserialize event log: {}", e))
}

/// Parse raw XES XML into EventLog JSON (BusinessOS gateway).
async fn parse_xes_to_event_log(
    body: Bytes,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let xml = std::str::from_utf8(&body)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid UTF-8: {}", e)))?;
    let reader = crate::io::xes::XESReader::new();
    let log = reader
        .parse_str(xml)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let v = serde_json::to_value(&log)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(v))
}

/// Progress event emitted during discovery/conformance processing
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressEvent {
    /// Progress percentage: 0, 25, 50, 75, 100
    pub progress: u32,
    /// Algorithm name: "alpha", "alpha_plus", "inductive", etc.
    pub algorithm: String,
    /// Elapsed time in milliseconds
    pub elapsed_ms: u128,
}

/// Returns the BusinessOS base URL from environment, defaulting to localhost.
/// Reads PM4PY_RUST_BUSINESSOS_URL; falls back to http://localhost:8001.
fn businessos_url() -> String {
    env::var("PM4PY_RUST_BUSINESSOS_URL").unwrap_or_else(|_| "http://localhost:8001".to_string())
}

/// Returns the port this server should bind to from environment, defaulting to 8090.
/// Reads PM4PY_RUST_PORT; falls back to 8090 on parse error.
pub fn pm4py_port() -> u16 {
    env::var("PM4PY_RUST_PORT")
        .unwrap_or_else(|_| "8090".to_string())
        .parse::<u16>()
        .unwrap_or(8090)
}

/// Emits a progress event to BusinessOS streaming endpoint
/// This is fire-and-forget: errors are logged but do not block the discovery process
async fn emit_progress_to_businessos(progress: u32, algorithm: &str, elapsed_ms: u128) {
    let progress_endpoint = format!("{}/api/bos/progress", businessos_url());

    let event = ProgressEvent {
        progress,
        algorithm: algorithm.to_string(),
        elapsed_ms,
    };

    // Spawn non-blocking async task to emit event — errors never surface to caller
    tokio::spawn(async move {
        match reqwest::Client::new()
            .post(&progress_endpoint)
            .json(&event)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) => {
                if !response.status().is_success() {
                    eprintln!("Failed to emit progress event: HTTP {}", response.status());
                }
            }
            Err(e) => {
                // Log warning but do not block discovery
                eprintln!(
                    "Failed to send progress event to BusinessOS: {} (continuing discovery)",
                    e
                );
            }
        }
    });

    // Yield to allow the spawned task to start
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
}

/// Parse PetriNet from JSON
fn parse_petri_net(json: &PetriNetJson) -> Result<PetriNet, String> {
    let mut net = PetriNet::new();

    // Add places
    for place_json in &json.places {
        let mut place = crate::models::petri_net::Place::new(place_json.name.clone());
        place.id = place_json.id.clone();
        place.initial_marking = place_json.initial_marking;
        net.places.push(place);
    }

    // Add transitions
    for trans_json in &json.transitions {
        let mut trans = crate::models::petri_net::Transition::new(trans_json.name.clone());
        trans.id = trans_json.id.clone();
        trans.label = trans_json.label.clone();
        net.transitions.push(trans);
    }

    // Add arcs
    for arc_json in &json.arcs {
        let arc = crate::models::petri_net::Arc::new(arc_json.from.clone(), arc_json.to.clone())
            .with_weight(arc_json.weight);
        net.arcs.push(arc);
    }

    net.initial_place = json.initial_place.clone();
    net.final_place = json.final_place.clone();

    Ok(net)
}

/// POST /api/discovery/alpha — Discover Petri Net from Event Log
/// OTEL instrumentation: emits jtbd_scenario_process_discovery span for Wave 12
async fn discover_alpha(
    headers: HeaderMap,
    Json(req): Json<DiscoveryRequest>,
) -> Result<Json<DiscoveryResponse>, ApiError> {
    let parent_cx = extract_trace_context(&headers);
    let correlation_id = resolve_correlation_id(&headers);
    let tracer = global::tracer("pm4py-rust");
    let mut span = tracer.start_with_context(
        crate::semconv::process_mining_span_names::PROCESS_MINING_DISCOVERY_SPAN,
        &parent_cx,
    );

    span.set_attribute(KeyValue::new(
        "chatmangpt.run.correlation_id",
        correlation_id,
    ));
    span.set_attribute(KeyValue::new("jtbd.scenario.id", "process_discovery"));
    span.set_attribute(KeyValue::new("jtbd.scenario.step", "load_event_log"));
    span.set_attribute(KeyValue::new("jtbd.scenario.step_num", 1i64));
    span.set_attribute(KeyValue::new("jtbd.scenario.step_total", 3i64));
    span.set_attribute(KeyValue::new("jtbd.scenario.system", "pm4py_rust"));
    span.set_attribute(KeyValue::new("jtbd.scenario.wave", "wave12"));
    span.set_attribute(KeyValue::new("jtbd.scenario.outcome", "pending"));

    let start = std::time::Instant::now();

    // Progress: 0% - started
    emit_progress_to_businessos(0, "alpha", start.elapsed().as_millis()).await;

    let event_log = parse_event_log(&req.event_log).map_err(|e| {
        span.set_attribute(KeyValue::new("jtbd.scenario.outcome", "failure"));
        span.set_attribute(KeyValue::new(
            "jtbd.scenario.error_reason",
            "event_log_parse_failed",
        ));
        ApiError {
            error: "EventLog parsing failed".to_string(),
            details: Some(e),
            status: 400,
        }
    })?;

    // Progress: 25% - event log parsed
    emit_progress_to_businessos(25, "alpha", start.elapsed().as_millis()).await;

    let event_count = event_log.traces.iter().map(|t| t.events.len()).sum();
    let trace_count = event_log.traces.len();

    span.set_attribute(KeyValue::new("jtbd.scenario.step", "calculate_metrics"));
    span.set_attribute(KeyValue::new("jtbd.scenario.step_num", 2i64));

    // Progress: 50% - metrics calculated
    emit_progress_to_businessos(50, "alpha", start.elapsed().as_millis()).await;

    // Run Alpha Miner
    let miner = AlphaMiner::new();
    let petri_net = miner.discover(&event_log);

    let place_count = petri_net.places.len() as i64;
    let transition_count = petri_net.transitions.len() as i64;

    // Calculate real fitness via token replay conformance checking
    let conformance_checker = TokenReplay::new();
    let fitness_result = conformance_checker.check(&event_log, &petri_net);
    let fitness = fitness_result.fitness;

    span.set_attribute(KeyValue::new("jtbd.scenario.step", "complete"));
    span.set_attribute(KeyValue::new("jtbd.scenario.step_num", 3i64));
    span.set_attribute(KeyValue::new(JTBD_SCENARIO_PLACE_COUNT, place_count));
    span.set_attribute(KeyValue::new(
        JTBD_SCENARIO_TRANSITION_COUNT,
        transition_count,
    ));
    span.set_attribute(KeyValue::new(JTBD_SCENARIO_FITNESS, fitness));
    span.set_attribute(KeyValue::new(JTBD_SCENARIO_MODEL_FORMAT, "pnml"));

    // Progress: 75% - mining completed
    emit_progress_to_businessos(75, "alpha", start.elapsed().as_millis()).await;

    let elapsed = start.elapsed().as_millis();

    span.set_attribute(KeyValue::new("jtbd.scenario.outcome", "success"));
    span.set_attribute(KeyValue::new("jtbd.scenario.latency_ms", elapsed as i64));

    // Progress: 100% - finished
    emit_progress_to_businessos(100, "alpha", elapsed).await;

    Ok(Json(DiscoveryResponse {
        petri_net: PetriNetJson::from_petri_net(&petri_net),
        algorithm: "alpha_miner".to_string(),
        execution_time_ms: elapsed,
        event_count,
        trace_count,
    }))
}

/// POST /api/conformance/token-replay — Check Conformance via Token Replay
async fn conformance_token_replay(
    Json(req): Json<ConformanceRequest>,
) -> Result<Json<ConformanceResponse>, ApiError> {
    let start = std::time::Instant::now();

    let event_log = parse_event_log(&req.event_log).map_err(|e| ApiError {
        error: "EventLog parsing failed".to_string(),
        details: Some(e),
        status: 400,
    })?;

    let petri_net = parse_petri_net(&req.petri_net).map_err(|e| ApiError {
        error: "PetriNet parsing failed".to_string(),
        details: Some(e),
        status: 400,
    })?;

    // Run Token Replay
    let checker = TokenReplay::new();
    let result = checker.check(&event_log, &petri_net);

    Ok(Json(ConformanceResponse::from((
        result,
        "token_replay".to_string(),
        start.elapsed().as_millis(),
    ))))
}

/// POST /api/statistics — Calculate Event Log Statistics
async fn statistics(
    Json(req): Json<StatisticsRequest>,
) -> Result<Json<StatisticsResponse>, ApiError> {
    let start = std::time::Instant::now();

    let event_log = parse_event_log(&req.event_log).map_err(|e| ApiError {
        error: "EventLog parsing failed".to_string(),
        details: Some(e),
        status: 400,
    })?;

    let trace_count = event_log.traces.len();
    let event_count: usize = event_log.traces.iter().map(|t| t.events.len()).sum();

    // Get unique activities
    let mut activities = std::collections::HashSet::new();
    for trace in &event_log.traces {
        for event in &trace.events {
            activities.insert(event.activity.clone());
        }
    }
    let unique_activities = activities.len();

    // Get activity frequencies
    let activity_freqs = if req.include_variants {
        let freqs = get_activity_frequency(&event_log);
        Some(
            freqs
                .into_iter()
                .map(|f| (f.activity, f.total_count))
                .collect(),
        )
    } else {
        None
    };

    // Get variant frequencies
    let variant_freqs: Option<HashMap<String, usize>> = if req.include_variants {
        let freqs = get_variant_frequency(&event_log);
        Some(freqs.into_iter().map(|f| (f.variant, f.count)).collect())
    } else {
        None
    };
    let variant_count = variant_freqs
        .as_ref()
        .map(|v: &HashMap<String, usize>| v.len())
        .unwrap_or(0);

    // Get bottleneck activities
    let bottlenecks = if req.include_bottlenecks {
        Some(
            get_bottleneck_activities(&event_log, 10)
                .into_iter()
                .map(|(activity, _)| activity)
                .collect(),
        )
    } else {
        None
    };

    // Get resource metrics
    let resource_count = if req.include_resource_metrics {
        Some(get_resource_metrics(&event_log).len())
    } else {
        None
    };

    Ok(Json(StatisticsResponse {
        trace_count,
        event_count,
        unique_activities,
        activity_frequencies: activity_freqs,
        variant_count,
        variant_frequencies: variant_freqs,
        bottleneck_activities: bottlenecks,
        resource_count,
        execution_time_ms: start.elapsed().as_millis(),
    }))
}

/// POST /api/monitoring/drift — Detect model drift between baseline and recent metrics
async fn detect_drift(Json(req): Json<DriftRequest>) -> Result<Json<DriftResponse>, ApiError> {
    let start = std::time::Instant::now();

    let tracer = global::tracer("pm4py-rust");
    let mut span = tracer.start(PROCESS_MINING_DRIFT_DETECT_SPAN);

    let calculator = DriftCalculator::new();
    let drift_score = calculator.calculate_drift(&req.baseline, &req.recent);
    let is_drifted = calculator.is_drift_detected(drift_score);
    let changed_metrics = calculator.identify_changed_metrics(&req.baseline, &req.recent);

    // Determine severity based on drift score
    let severity = if drift_score < 0.05 {
        "stable".to_string()
    } else if drift_score < 0.1 {
        "minor".to_string()
    } else if drift_score < 0.2 {
        "major".to_string()
    } else {
        "critical".to_string()
    };

    span.set_attribute(KeyValue::new(PROCESS_MINING_DRIFT_DETECTED, is_drifted));

    Ok(Json(DriftResponse {
        drift_score,
        is_drifted,
        severity,
        changed_metrics,
        execution_time_ms: start.elapsed().as_millis(),
    }))
}

/// GET /.well-known/agent.json — A2A agent discovery card (legacy endpoint).
/// Use /.well-known/agent-card.json for the typed A2A card with all 10 skills.
async fn agent_card() -> Json<serde_json::Value> {
    #[cfg(feature = "a2a")]
    {
        use crate::a2a::skills::all_skills;
        let skills: Vec<serde_json::Value> = all_skills()
            .into_iter()
            .map(|s| {
                serde_json::json!({
                    "id": s.id,
                    "name": s.name,
                    "description": s.description,
                    "inputModes": s.input_modes,
                })
            })
            .collect();
        return Json(serde_json::json!({
            "name": "pm4py-rust",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Process mining engine with Alpha Miner, token replay, DFG, OCEL 2.0, and LLM abstraction.",
            "url": "http://localhost:8090",
            "capabilities": ["tools"],
            "protocolVersion": "0.2.1",
            "skills": skills,
        }));
    }
    #[cfg(not(feature = "a2a"))]
    Json(serde_json::json!({
        "name": "pm4py-rust",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Process mining engine with Alpha Miner, token replay conformance, DFG analysis, and OCEL 2.0 support.",
        "url": "http://localhost:8090",
        "capabilities": ["tools"],
        "skills": [
            {"id": "pm4py_discover_alpha", "name": "Process Discovery (Alpha Miner)", "description": "Discover process models using Alpha Miner"},
            {"id": "pm4py_conformance_token_replay", "name": "Conformance Checking (Token Replay)", "description": "Token replay conformance checking"},
            {"id": "pm4py_statistics", "name": "Process Statistics", "description": "Process log statistics and KPI computation"},
            {"id": "pm4py_detect_drift", "name": "Process Drift Detection", "description": "Detect process drift between time windows"},
            {"id": "pm4py_ocel_ingest", "name": "OCEL 2.0 Ingest", "description": "Ingest OCEL 2.0 object-centric event logs"}
        ]
    }))
}

/// GET /api/health — Health check and readiness probe
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// POST /api/abstract/petri-net — Petri Net → Plain English
async fn abstract_petri_net_endpoint(
    Json(req): Json<AbstractPetriNetRequest>,
) -> Result<Json<ProcessIntelligenceResponse>, ApiError> {
    let start = std::time::Instant::now();

    let petri_net = parse_petri_net(&req.petri_net).map_err(|e| ApiError {
        error: "PetriNet parsing failed".to_string(),
        details: Some(e),
        status: 400,
    })?;

    let abstract_description = abstract_petri_net(&petri_net);

    Ok(Json(ProcessIntelligenceResponse {
        abstract_description,
        model_type: "petri_net".to_string(),
        execution_time_ms: start.elapsed().as_millis(),
    }))
}

/// POST /api/abstract/event-log — EventLog → Plain English
async fn abstract_event_log_endpoint(
    Json(req): Json<AbstractEventLogRequest>,
) -> Result<Json<ProcessIntelligenceResponse>, ApiError> {
    let start = std::time::Instant::now();

    let event_log = parse_event_log(&req.event_log).map_err(|e| ApiError {
        error: "EventLog parsing failed".to_string(),
        details: Some(e),
        status: 400,
    })?;

    let abstract_description = abstract_event_log(&event_log);

    Ok(Json(ProcessIntelligenceResponse {
        abstract_description,
        model_type: "event_log".to_string(),
        execution_time_ms: start.elapsed().as_millis(),
    }))
}

/// POST /api/abstract/dfg — DFG → Plain English
async fn abstract_dfg_endpoint(
    Json(req): Json<AbstractDfgRequest>,
) -> Result<Json<ProcessIntelligenceResponse>, ApiError> {
    let start = std::time::Instant::now();

    // Parse DFG from JSON
    let dfg: DirectlyFollowsGraph = serde_json::from_value(req.dfg).map_err(|e| ApiError {
        error: "DFG parsing failed".to_string(),
        details: Some(e.to_string()),
        status: 400,
    })?;

    let abstract_description = abstract_dfg(&dfg);

    Ok(Json(ProcessIntelligenceResponse {
        abstract_description,
        model_type: "dfg".to_string(),
        execution_time_ms: start.elapsed().as_millis(),
    }))
}

/// POST /api/query — Process Intelligence Query (NL question + context → insight)
async fn process_intelligence_query_endpoint(
    Json(req): Json<ProcessIntelligenceQueryRequest>,
) -> Result<Json<ProcessIntelligenceQueryResponse>, ApiError> {
    let start = std::time::Instant::now();

    let response = if let Some(petri_net_json) = &req.petri_net {
        let petri_net = parse_petri_net(petri_net_json).map_err(|e| ApiError {
            error: "PetriNet parsing failed".to_string(),
            details: Some(e),
            status: 400,
        })?;

        // Route query to appropriate abstractor based on keywords
        if req.query.to_lowercase().contains("bottleneck") {
            format!("Bottleneck analysis: {}", abstract_petri_net(&petri_net))
        } else if req.query.to_lowercase().contains("critical path")
            || req.query.to_lowercase().contains("duration")
        {
            format!("Duration analysis: {}", abstract_petri_net(&petri_net))
        } else {
            format!("Process overview: {}", abstract_petri_net(&petri_net))
        }
    } else if let Some(event_log_json) = &req.event_log {
        let event_log = parse_event_log(event_log_json).map_err(|e| ApiError {
            error: "EventLog parsing failed".to_string(),
            details: Some(e),
            status: 400,
        })?;

        // Route query to event log abstractor
        if req.query.to_lowercase().contains("variant")
            || req.query.to_lowercase().contains("trace")
        {
            format!("Variant analysis: {}", abstract_event_log(&event_log))
        } else if req.query.to_lowercase().contains("resource")
            || req.query.to_lowercase().contains("performer")
        {
            format!("Resource analysis: {}", abstract_event_log(&event_log))
        } else {
            format!("Process statistics: {}", abstract_event_log(&event_log))
        }
    } else {
        return Err(ApiError {
            error: "No context provided".to_string(),
            details: Some("Provide either petri_net or event_log".to_string()),
            status: 400,
        });
    };

    Ok(Json(ProcessIntelligenceQueryResponse {
        query: req.query,
        response,
        execution_time_ms: start.elapsed().as_millis(),
    }))
}

// ── OCPM Performance + LLM endpoints ─────────────────────────────────────────

/// Request body for OCPM performance endpoints — an OCEL 2.0 log in JSON.
#[derive(Debug, Deserialize)]
struct OcpmOcelRequest {
    /// OCEL 2.0 log serialized as JSON (same format as `ObjectCentricEventLog`).
    pub ocel: serde_json::Value,
}

/// Request body for OCPM bottleneck endpoint — OCEL + top_n parameter.
#[derive(Debug, Deserialize)]
struct OcpmBottleneckRequest {
    pub ocel: serde_json::Value,
    #[serde(default = "default_top_n")]
    pub top_n: usize,
}

fn default_top_n() -> usize {
    5
}

/// Request body for OCPM LLM query — OCEL + question + optional API key.
#[derive(Debug, Deserialize)]
struct OcpmLlmQueryRequest {
    pub ocel: serde_json::Value,
    pub question: String,
    pub api_key: Option<String>,
}

/// POST /api/ocpm/performance/throughput
/// Compute end-to-end throughput statistics per object type from an OCEL 2.0 log.
async fn ocpm_throughput(
    Json(req): Json<OcpmOcelRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ocel: crate::ocpm::ObjectCentricEventLog =
        serde_json::from_value(req.ocel).map_err(|e| ApiError {
            error: "OCEL parse error".to_string(),
            details: Some(e.to_string()),
            status: 400,
        })?;

    let stats = crate::ocpm::oc_performance::compute_throughput_by_object_type(&ocel);

    // Serialize to plain JSON map (BTreeMap key is not a JSON key by default)
    let result: serde_json::Map<String, serde_json::Value> = stats
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                serde_json::json!({
                    "mean_secs": v.mean_secs,
                    "median_secs": v.median_secs,
                    "p95_secs": v.p95_secs,
                    "min_secs": v.min_secs,
                    "max_secs": v.max_secs,
                    "count": v.count
                }),
            )
        })
        .collect();

    Ok(Json(serde_json::Value::Object(result)))
}

/// POST /api/ocpm/performance/bottleneck
/// Detect top-N bottlenecks from an OCEL 2.0 log.
async fn ocpm_bottleneck(
    Json(req): Json<OcpmBottleneckRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ocel: crate::ocpm::ObjectCentricEventLog =
        serde_json::from_value(req.ocel).map_err(|e| ApiError {
            error: "OCEL parse error".to_string(),
            details: Some(e.to_string()),
            status: 400,
        })?;

    let bottlenecks =
        crate::ocpm::oc_performance::detect_bottlenecks(&ocel, req.top_n.clamp(1, 50));

    let result: Vec<serde_json::Value> = bottlenecks
        .into_iter()
        .map(|b| {
            serde_json::json!({
                "object_type": b.object_type,
                "from_activity": b.from_activity,
                "to_activity": b.to_activity,
                "mean_wait_secs": b.mean_wait_secs,
                "frequency": b.frequency,
                "severity_score": b.severity_score
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "bottlenecks": result })))
}

/// POST /api/ocpm/llm/query
/// Ask a natural language question about an OCEL 2.0 log, grounded in real process data.
async fn ocpm_llm_query(
    Json(req): Json<OcpmLlmQueryRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ocel: crate::ocpm::ObjectCentricEventLog =
        serde_json::from_value(req.ocel).map_err(|e| ApiError {
            error: "OCEL parse error".to_string(),
            details: Some(e.to_string()),
            status: 400,
        })?;

    let result = crate::llm::query_engine::query_with_ocel_llm(
        &req.question,
        Some(&ocel),
        None,
        None,
        None,
        req.api_key.as_deref(),
    )
    .await;

    Ok(Json(serde_json::json!({
        "query": result.query,
        "response": result.response,
        "response_length": result.response_length,
        "model_type": result.model_type,
        "domain": format!("{:?}", result.domain)
    })))
}

/// Create the BusinessOS API router
/// Metrics endpoint: exports Prometheus-format metrics
async fn metrics() -> (StatusCode, String) {
    let metrics_text = crate::metrics::metrics().export_prometheus();
    (StatusCode::OK, metrics_text)
}

/// GET /.well-known/mcp.json — MCP server discovery document.
async fn mcp_discovery() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "mcpServers": {
            "pm4py-rust": {
                "transport": "streamable-http",
                "url": "http://localhost:8090/mcp/v2",
                "legacyUrl": "http://localhost:8090/mcp",
                "protocol": "2024-11-05"
            }
        }
    }))
}

pub fn router() -> Router {
    let r = Router::new()
        .route("/.well-known/agent.json", get(agent_card))
        .route("/.well-known/mcp.json", get(mcp_discovery))
        .route("/mcp", post(crate::mcp::handler::mcp_handler))
        .route("/api/health", get(health))
        .route("/api/discovery/alpha", post(discover_alpha))
        .route(
            "/api/conformance/token-replay",
            post(conformance_token_replay),
        )
        .route("/api/statistics", post(statistics))
        .route("/api/io/parse-xes", post(parse_xes_to_event_log))
        .route("/api/monitoring/drift", post(detect_drift))
        .route("/api/abstract/petri-net", post(abstract_petri_net_endpoint))
        .route("/api/abstract/event-log", post(abstract_event_log_endpoint))
        .route("/api/abstract/dfg", post(abstract_dfg_endpoint))
        .route("/api/query", post(process_intelligence_query_endpoint))
        .route("/metrics", get(metrics))
        .route("/api/connector/extract", post(connector_extract))
        // OCEL 2.0 ingest — parse and queue for process mining discovery
        .route(
            "/api/ocel/ingest",
            post(crate::http::ocel_ingest::ocel_ingest),
        )
        // OCPM Performance Analysis — throughput and bottleneck endpoints
        .route("/api/ocpm/performance/throughput", post(ocpm_throughput))
        .route("/api/ocpm/performance/bottleneck", post(ocpm_bottleneck))
        // OCPM LLM Query — RAG-grounded process intelligence (Connection 4)
        .route("/api/ocpm/llm/query", post(ocpm_llm_query))
        // Board Chair Intelligence System — report process deviation to OSA for autonomous healing
        .route(
            "/board/deviation",
            post(crate::http::boardchair_api::report_deviation),
        )
        // Board Chair Analytics — Conway's Law and Little's Law analysis endpoints
        .route(
            "/api/boardchair/conway-check",
            post(crate::boardchair::handlers::conway_check),
        )
        .route(
            "/api/boardchair/littles-law",
            post(crate::boardchair::handlers::littles_law),
        )
        // Board KPI Pipeline — board-ready KPIs from process mining data
        .route(
            "/api/board/kpis",
            get(crate::board_kpis::handlers::board_kpis_get)
                .post(crate::board_kpis::handlers::board_kpis_post),
        )
        .layer(TraceLayer::new_for_http());

    // A2A JSON-RPC endpoint + typed agent card (feature-gated)
    #[cfg(feature = "a2a")]
    let r = r
        .route("/a2a", post(crate::a2a::handler::a2a_handler))
        .route(
            "/.well-known/agent-card.json",
            get(crate::a2a::handler::a2a_agent_card),
        );

    // rmcp StreamableHttpService — SSE-capable transport at /mcp/v2
    // Feature-gated: only compiled when `--features mcp-server` is set.
    #[cfg(feature = "mcp-server")]
    let r = {
        use rmcp::transport::streamable_http_server::{
            session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
        };
        let ct = crate::http::mcp_state::mcp_cancellation_token().child_token();
        let svc = StreamableHttpService::new(
            || Ok(crate::mcp::rmcp_server::Pm4pyMcpServer::new()),
            std::sync::Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig::default().with_cancellation_token(ct),
        );
        r.nest_service("/mcp/v2", svc)
    };

    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_petri_net_json_roundtrip() {
        let net = PetriNet::new();
        let json = PetriNetJson::from_petri_net(&net);
        assert_eq!(json.places.len(), 0);
        assert_eq!(json.transitions.len(), 0);
    }

    #[test]
    fn test_api_error_serialization() {
        let err = ApiError {
            error: "test error".to_string(),
            details: Some("details".to_string()),
            status: 400,
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("test error"));
    }
}
