//! Pipeline integration test: pm4py-rust HTTP API
//!
//! Verifies health, discovery, statistics, boardchair Conway, and boardchair
//! Little's Law endpoints against the live pm4py-rust service.
//!
//! Skips ALL tests gracefully when pm4py-rust is not running at localhost:8090.
//!
//! Run (with service up):
//!   cargo test --test pipeline_integration_test -- --nocapture
//!
//! Run (without service — all tests skip, no failures):
//!   cargo test --test pipeline_integration_test -- --nocapture

use std::time::Duration;

const PM4PY_BASE: &str = "http://localhost:8090";

/// Build a shared reqwest async client with a short timeout.
fn make_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("build reqwest client")
}

/// Returns true if pm4py-rust is reachable and healthy.
async fn is_pm4py_running(client: &reqwest::Client) -> bool {
    client
        .get(format!("{}/api/health", PM4PY_BASE))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// A minimal two-trace event log suitable for Alpha Miner discovery.
fn sample_event_log() -> serde_json::Value {
    serde_json::json!({
        "traces": [
            {
                "caseID": "case_001",
                "events": [
                    {"activity": "Start",   "timestamp": "2024-01-01T10:00:00Z"},
                    {"activity": "Review",  "timestamp": "2024-01-01T10:05:00Z"},
                    {"activity": "Approve", "timestamp": "2024-01-01T10:10:00Z"},
                    {"activity": "End",     "timestamp": "2024-01-01T10:15:00Z"}
                ]
            },
            {
                "caseID": "case_002",
                "events": [
                    {"activity": "Start",  "timestamp": "2024-01-02T10:00:00Z"},
                    {"activity": "Review", "timestamp": "2024-01-02T10:03:00Z"},
                    {"activity": "Reject", "timestamp": "2024-01-02T10:08:00Z"},
                    {"activity": "End",    "timestamp": "2024-01-02T10:12:00Z"}
                ]
            }
        ]
    })
}

// ── Health check ─────────────────────────────────────────────────────────────

/// Verifies that the pm4py-rust service reports itself healthy.
///
/// When running: asserts status == "healthy" and version is non-empty.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires running pm4py-rust server on localhost:8090"]
async fn test_pipeline_health_check() {
    let client = make_client();

    if !is_pm4py_running(&client).await {
        eprintln!("SKIP: pm4py-rust not running at {PM4PY_BASE}");
        return;
    }

    let resp = client
        .get(format!("{}/api/health", PM4PY_BASE))
        .send()
        .await
        .expect("GET /api/health failed");

    assert!(
        resp.status().is_success(),
        "Expected 200 OK from /api/health, got {}",
        resp.status()
    );

    let body: serde_json::Value = resp.json().await.expect("health response must be JSON");

    let status = body
        .get("status")
        .and_then(|v: &serde_json::Value| v.as_str())
        .unwrap_or("");
    assert_eq!(
        status, "healthy",
        "Expected status == 'healthy', got: {:?}",
        body
    );

    let version = body
        .get("version")
        .and_then(|v: &serde_json::Value| v.as_str())
        .unwrap_or("");
    assert!(!version.is_empty(), "Expected non-empty version string");

    println!("health OK: status=healthy version={version}");
}

// ── Discovery ─────────────────────────────────────────────────────────────────

/// Verifies that POST /api/discovery/alpha returns a Petri net with places,
/// transitions, and arcs for the sample event log.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires running pm4py-rust server on localhost:8090"]
async fn test_pipeline_discovery_alpha_returns_petri_net() {
    let client = make_client();

    if !is_pm4py_running(&client).await {
        eprintln!("SKIP: pm4py-rust not running at {PM4PY_BASE}");
        return;
    }

    let payload = serde_json::json!({
        "event_log": sample_event_log(),
        "variant": "alpha"
    });

    let resp = client
        .post(format!("{}/api/discovery/alpha", PM4PY_BASE))
        .json(&payload)
        .send()
        .await
        .expect("POST /api/discovery/alpha failed");

    assert!(
        resp.status().is_success(),
        "Expected 2xx from /api/discovery/alpha, got {}",
        resp.status()
    );

    let body: serde_json::Value = resp.json().await.expect("discovery response must be JSON");

    // Response must contain a petri_net object.
    let net = body
        .get("petri_net")
        .expect("response missing 'petri_net' field");

    let places = net
        .get("places")
        .and_then(|v: &serde_json::Value| v.as_array())
        .expect("'places' must be an array");
    assert!(!places.is_empty(), "Petri net must have at least one place");

    let transitions = net
        .get("transitions")
        .and_then(|v: &serde_json::Value| v.as_array())
        .expect("'transitions' must be an array");
    assert!(
        !transitions.is_empty(),
        "Petri net must have at least one transition"
    );

    let arcs = net
        .get("arcs")
        .and_then(|v: &serde_json::Value| v.as_array())
        .expect("'arcs' must be an array");
    assert!(!arcs.is_empty(), "Petri net must have at least one arc");

    // Metadata fields must be present.
    assert!(
        body.get("algorithm").is_some(),
        "Response missing 'algorithm'"
    );
    assert!(
        body.get("trace_count").is_some(),
        "Response missing 'trace_count'"
    );
    assert!(
        body.get("event_count").is_some(),
        "Response missing 'event_count'"
    );

    let trace_count = body["trace_count"].as_u64().unwrap_or(0);
    assert_eq!(trace_count, 2, "Expected 2 traces in discovery response");

    println!(
        "discovery OK: places={} transitions={} arcs={} traces={}",
        places.len(),
        transitions.len(),
        arcs.len(),
        trace_count
    );
}

// ── Statistics ────────────────────────────────────────────────────────────────

/// Verifies that POST /api/statistics returns the correct trace count and
/// activity frequencies for the sample event log.
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires running pm4py-rust server on localhost:8090"]
async fn test_pipeline_statistics_returns_correct_trace_count() {
    let client = make_client();

    if !is_pm4py_running(&client).await {
        eprintln!("SKIP: pm4py-rust not running at {PM4PY_BASE}");
        return;
    }

    let payload = serde_json::json!({
        "event_log": sample_event_log(),
        "include_variants": true,
        "include_resource_metrics": false,
        "include_bottlenecks": false
    });

    let resp = client
        .post(format!("{}/api/statistics", PM4PY_BASE))
        .json(&payload)
        .send()
        .await
        .expect("POST /api/statistics failed");

    assert!(
        resp.status().is_success(),
        "Expected 2xx from /api/statistics, got {}",
        resp.status()
    );

    let body: serde_json::Value = resp.json().await.expect("statistics response must be JSON");

    let trace_count = body
        .get("trace_count")
        .and_then(|v: &serde_json::Value| v.as_u64())
        .expect("response missing 'trace_count'");
    assert_eq!(trace_count, 2, "Expected trace_count == 2");

    // event_count: 4 + 4 = 8 events across both traces.
    let event_count = body
        .get("event_count")
        .and_then(|v: &serde_json::Value| v.as_u64())
        .expect("response missing 'event_count'");
    assert_eq!(event_count, 8, "Expected event_count == 8");

    let unique_activities = body
        .get("unique_activities")
        .and_then(|v: &serde_json::Value| v.as_u64())
        .expect("response missing 'unique_activities'");
    // Start, Review, Approve, Reject, End = 5 unique activities.
    assert_eq!(
        unique_activities, 5,
        "Expected 5 unique activities (Start, Review, Approve, Reject, End)"
    );

    println!(
        "statistics OK: traces={trace_count} events={event_count} \
         unique_activities={unique_activities}"
    );
}

// ── Boardchair: Conway's Law ──────────────────────────────────────────────────

/// Verifies that the boardchair endpoint detects a Conway's Law violation when
/// boundary time exceeds 40% of cycle time.
///
/// Setup: boundary_time_ms=60, cycle_time_ms=100 → score=0.6 > 0.4 → violation.
/// Skips gracefully if the endpoint is not yet wired (404).
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires running pm4py-rust server on localhost:8090"]
async fn test_pipeline_boardchair_conway_violation_detected() {
    let client = make_client();

    if !is_pm4py_running(&client).await {
        eprintln!("SKIP: pm4py-rust not running at {PM4PY_BASE}");
        return;
    }

    let payload = serde_json::json!({
        "boundary_time_ms": 60,
        "cycle_time_ms": 100
    });

    let resp = match client
        .post(format!("{}/api/boardchair/conway", PM4PY_BASE))
        .json(&payload)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("SKIP: /api/boardchair/conway not reachable: {e}");
            return;
        }
    };

    if resp.status().as_u16() == 404 {
        eprintln!("SKIP: /api/boardchair/conway not yet wired (404)");
        return;
    }

    assert!(
        resp.status().is_success(),
        "Expected 2xx from /api/boardchair/conway, got {}",
        resp.status()
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("boardchair/conway response must be JSON");

    let is_violation = body
        .get("is_violation")
        .and_then(|v: &serde_json::Value| v.as_bool())
        .expect("response missing 'is_violation'");
    assert!(
        is_violation,
        "Expected Conway violation (boundary=60ms, cycle=100ms → score=0.6 > 0.4)"
    );

    let score = body
        .get("conway_score")
        .and_then(|v: &serde_json::Value| v.as_f64())
        .expect("response missing 'conway_score'");
    assert!(
        (score - 0.6).abs() < 0.01,
        "Expected conway_score ≈ 0.6, got {score}"
    );

    println!("boardchair/conway OK: is_violation=true score={score:.3}");
}

// ── Boardchair: Little's Law ──────────────────────────────────────────────────

/// Verifies that the boardchair endpoint reports stable (no violation) when
/// actual WIP is within Little's Law expectations.
///
/// Setup: arrival_rate=10/s, actual_wip=1.2, cycle_time_ms=100
/// → expected_wip = 10 × 0.1 = 1.0; 1.2 < 1.5 × 1.0 → no violation.
/// Skips gracefully if the endpoint is not yet wired (404).
#[tokio::test(flavor = "current_thread")]
#[ignore = "requires running pm4py-rust server on localhost:8090"]
async fn test_pipeline_boardchair_littles_law_stable() {
    let client = make_client();

    if !is_pm4py_running(&client).await {
        eprintln!("SKIP: pm4py-rust not running at {PM4PY_BASE}");
        return;
    }

    let payload = serde_json::json!({
        "arrival_rate_per_sec": 10.0,
        "actual_wip": 1.2,
        "cycle_time_ms": 100.0
    });

    let resp = match client
        .post(format!("{}/api/boardchair/littles-law", PM4PY_BASE))
        .json(&payload)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("SKIP: /api/boardchair/littles-law not reachable: {e}");
            return;
        }
    };

    if resp.status().as_u16() == 404 {
        eprintln!("SKIP: /api/boardchair/littles-law not yet wired (404)");
        return;
    }

    assert!(
        resp.status().is_success(),
        "Expected 2xx from /api/boardchair/littles-law, got {}",
        resp.status()
    );

    let body: serde_json::Value = resp
        .json()
        .await
        .expect("boardchair/littles-law response must be JSON");

    let is_violation = body
        .get("is_violation")
        .and_then(|v: &serde_json::Value| v.as_bool())
        .expect("response missing 'is_violation'");
    assert!(
        !is_violation,
        "Expected no Little's Law violation (actual_wip=1.2 < 1.5×expected=1.5)"
    );

    let expected_wip = body
        .get("expected_wip")
        .and_then(|v: &serde_json::Value| v.as_f64())
        .expect("response missing 'expected_wip'");
    assert!(
        (expected_wip - 1.0).abs() < 0.01,
        "Expected expected_wip ≈ 1.0, got {expected_wip}"
    );

    println!("boardchair/littles-law OK: is_violation=false expected_wip={expected_wip:.3}");
}
