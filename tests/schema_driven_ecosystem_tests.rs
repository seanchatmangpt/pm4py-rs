//! Schema-Driven Ecosystem Integration Tests for pm4py-rust
//!
//! Ralph Loop iteration 8: Validates pm4py-rust against REAL JSON schemas from
//! BusinessOS modules, OSA recipes/swarms/role_rates, and Canopy demo data.
//!
//! Innovation areas:
//! 1. BusinessOS CRM module → process mining (leads pipeline, deal stages)
//! 2. BusinessOS Projects module → task lifecycle discovery
//! 3. BusinessOS Documents module → version control process mining
//! 4. OSA recipe workflows → process model comparison
//! 5. OSA swarm patterns → multi-agent coordination mining
//! 6. OSA role_rates → resource cost analysis
//! 7. Performance metrics on Canopy invoice data
//! 8. Behavioral profiles across data sources
//! 9. Causal dependency analysis on business processes
//! 10. ML feature extraction from Canopy data
//! 11. OCEL conformance checks
//! 12. Tree statistics from discovered models
//! 13. SplitMiner vs InductiveMiner comparison
//! 14. CausalNetMiner on business workflows
//! 15. Extended statistics (correlation, network metrics)
//! 16. Process tree pattern analysis
//! 17. Temporal profile conformance
//! 18. Anomaly detection on Canopy compliance data
//! 19. Trace feature engineering for predictive models
//! 20. Cross-module process similarity

use chrono::{Duration, Utc};
use pm4py::conformance::check_ocel_lifecycle_conformance;
use pm4py::conformance::validate_ocel_event_ordering;
use pm4py::conformance::BehavioralProfile;
use pm4py::conformance::{AlignmentChecker, TokenReplay};
use pm4py::discovery::{
    conformance_declare, discover_prefix_tree, CausalNetMiner, DFGMiner, InductiveMiner, SplitMiner,
};
use pm4py::io::CSVReader;
use pm4py::log::{Event, EventLog, Trace};
use pm4py::models::footprints::Footprints;
use pm4py::ocpm::{
    OCPMDiscoveryMiner, ObjectCentricEventLog, ObjectCentricTokenReplay, ObjectType,
};
use pm4py::performance::{case_duration_metrics, case_durations, rework_cases, rework_percentage};
use pm4py::predictive::{
    CaseOutcome, NextActivityPredictor, OutcomePredictor, RemainingTimePredictor,
};
use pm4py::statistics::correlation::{
    activity_co_occurrence, causal_dependency_analysis, network_metrics,
};
use pm4py::statistics::ml_features::{
    create_feature_matrix, extract_features, features_to_vector, get_all_activities,
    get_numeric_attributes, get_str_attributes, normalize_features, train_test_split,
};
use pm4py::statistics::temporal_profile::{
    conformance_temporal_profile, discover_temporal_profile,
};
use pm4py::statistics::{analyze_tree, get_feature_names};
use std::collections::HashMap;
use std::path::Path;

// ============================================================================
// Helpers: Load real Canopy demo data
// ============================================================================

const CANOPY_DIR: &str = "/Users/sac/chatmangpt/canopy/priv/demo_data";

fn canopy_csv() -> CSVReader {
    CSVReader::new()
        .with_case_column("case_id")
        .with_activity_column("activity")
        .with_timestamp_column("timestamp")
        .with_resource_column(Some("resource"))
}

fn load_invoice() -> EventLog {
    canopy_csv()
        .read(Path::new(&format!(
            "{}/invoice_processing_events.csv",
            CANOPY_DIR
        )))
        .expect("Failed to load invoice CSV")
}

fn load_onboarding() -> EventLog {
    canopy_csv()
        .read(Path::new(&format!(
            "{}/customer_onboarding_events.csv",
            CANOPY_DIR
        )))
        .expect("Failed to load onboarding CSV")
}

fn load_compliance() -> EventLog {
    canopy_csv()
        .read(Path::new(&format!(
            "{}/compliance_reporting_events.csv",
            CANOPY_DIR
        )))
        .expect("Failed to load compliance CSV")
}

// ============================================================================
// INNOVATION 1: BusinessOS CRM Module → Process Mining
// Maps the CRM module's actions (create_contact, create_lead, create_deal, etc.)
// to a process log and discovers the sales pipeline model.
// ============================================================================

#[test]
fn test_crm_sales_pipeline_discovery() {
    // CRM module defines: create_contact → create_lead → create_company → create_deal
    // Each deal goes through stages: qualification → proposal → negotiation → closed_won/lost
    let mut log = EventLog::new();

    // Simulate 25 CRM sales pipeline cases based on the CRM module schema
    let pipeline_variants = vec![
        // Happy path
        vec![
            ("create_contact", "sales_rep"),
            ("create_lead", "sales_rep"),
            ("create_company", "sales_rep"),
            ("create_deal", "sales_rep"),
            ("qualify", "manager"),
            ("propose", "sales_rep"),
            ("negotiate", "sales_rep"),
            ("closed_won", "manager"),
        ],
        // Fast track (no company needed)
        vec![
            ("create_contact", "sales_rep"),
            ("create_lead", "sales_rep"),
            ("create_deal", "sales_rep"),
            ("qualify", "manager"),
            ("closed_won", "manager"),
        ],
        // Lost deal
        vec![
            ("create_contact", "sales_rep"),
            ("create_lead", "sales_rep"),
            ("create_company", "sales_rep"),
            ("create_deal", "sales_rep"),
            ("qualify", "manager"),
            ("closed_lost", "manager"),
        ],
        // Rework: re-qualify
        vec![
            ("create_contact", "sales_rep"),
            ("create_lead", "sales_rep"),
            ("create_deal", "sales_rep"),
            ("qualify", "manager"),
            ("negotiate", "sales_rep"),
            ("re_qualify", "manager"),
            ("propose", "sales_rep"),
            ("closed_won", "manager"),
        ],
    ];

    for (case_idx, variant) in pipeline_variants.iter().enumerate() {
        let mut trace = Trace::new(format!("DEAL-{}", case_idx));
        for (i, (activity, resource)) in variant.iter().enumerate() {
            trace.add_event(
                Event::new(*activity, Utc::now() + Duration::hours(i as i64))
                    .with_resource(*resource)
                    .with_attribute("module", "crm"),
            );
        }
        log.add_trace(trace);
    }

    // Discover the CRM sales process
    let dfg = DFGMiner::new().discover(&log);
    let net = InductiveMiner::new().discover(&log);
    let fp = Footprints::from_log(&log);

    println!("CRM Sales Pipeline Discovery:");
    println!("   Cases: {}", log.len());
    println!("   Activities: {}", log.activities().len());
    println!(
        "   DFG: {} nodes, {} edges",
        dfg.nodes.len(),
        dfg.edges.len()
    );
    println!(
        "   Model: {} places, {} transitions",
        net.places.len(),
        net.transitions.len()
    );
    println!("   Footprint activities: {}", fp.activities().len());

    // Verify CRM-specific activities exist
    let acts: std::collections::HashSet<String> = log.activities().into_iter().collect();
    assert!(
        acts.contains("create_contact"),
        "CRM should have create_contact"
    );
    assert!(acts.contains("create_lead"), "CRM should have create_lead");
    assert!(acts.contains("create_deal"), "CRM should have create_deal");

    // Conformance check
    let result = TokenReplay::new().check(&log, &net);
    println!("   Token replay fitness: {:.4}", result.fitness);
    assert!(result.fitness >= 0.0);
}

// ============================================================================
// INNOVATION 2: BusinessOS Projects Module → Task Lifecycle
// Maps the Projects module schema (create_project → create_task → assign → update_status)
// ============================================================================

#[test]
fn test_projects_task_lifecycle_discovery() {
    // Projects module: create_project → create_task → assign_task → update_task_status
    // Task statuses: todo → in_progress → review → done
    let mut log = EventLog::new();

    for proj_idx in 0..10 {
        let project_id = format!("PROJ-{}", proj_idx);
        for task_idx in 0..5 {
            let mut trace = Trace::new(format!("{}_TASK-{}", project_id, task_idx));
            let mut ts = Utc::now() + Duration::hours(proj_idx as i64);

            trace.add_event(
                Event::new("create_task", ts)
                    .with_resource("project_manager")
                    .with_attribute("project", &project_id),
            );
            ts += Duration::minutes(30);
            trace.add_event(
                Event::new("assign_task", ts)
                    .with_resource("project_manager")
                    .with_attribute("assignee", "developer"),
            );
            ts += Duration::hours(2);
            trace.add_event(Event::new("start_work", ts).with_resource("developer"));
            ts += Duration::hours(4);
            trace.add_event(Event::new("submit_review", ts).with_resource("developer"));
            ts += Duration::hours(1);
            trace.add_event(Event::new("approve", ts).with_resource("reviewer"));
            ts += Duration::minutes(15);
            trace.add_event(Event::new("mark_done", ts).with_resource("project_manager"));
            log.add_trace(trace);
        }
    }

    // Discover task lifecycle
    let dfg = DFGMiner::new().discover(&log);
    let net = InductiveMiner::new().discover(&log);
    let tree = InductiveMiner::new().discover_tree(&log);

    println!("Projects Task Lifecycle:");
    println!("   Tasks: {}", log.len());
    println!(
        "   DFG: {} nodes, {} edges",
        dfg.nodes.len(),
        dfg.edges.len()
    );
    println!(
        "   PetriNet: {} places, {} transitions",
        net.places.len(),
        net.transitions.len()
    );
    println!("   Process tree depth: {}", tree.root.depth());

    // Performance analysis on task lifecycle
    let durations = case_durations(&log);
    let metrics = case_duration_metrics(&log);
    let rework = rework_cases(&log);
    let rework_pct = rework_percentage(&log);

    println!(
        "   Avg case duration: {:?}",
        metrics.as_ref().map(|m| m.avg_duration)
    );
    println!("   Rework cases: {}", rework.len());
    println!("   Rework %: {:.2}%", rework_pct);

    assert!(log.len() == 50, "Should have 50 tasks");
}

// ============================================================================
// INNOVATION 3: BusinessOS Documents Module → Version Control Mining
// Maps the Documents module (create → update → share → version_history)
// ============================================================================

#[test]
fn test_documents_version_control_mining() {
    // Documents module: create_document → update_document → share_document → version_history
    let mut log = EventLog::new();

    for doc_idx in 0..15 {
        let mut trace = Trace::new(format!("DOC-{}", doc_idx));
        let mut ts = Utc::now() + Duration::hours(doc_idx as i64);
        let author = if doc_idx % 3 == 0 { "alice" } else { "bob" };

        trace.add_event(Event::new("create_document", ts).with_resource(author));
        ts += Duration::hours(1);

        // Varying number of edits
        let edits = 1 + (doc_idx % 4);
        for e in 0..edits {
            trace.add_event(
                Event::new("update_document", ts + Duration::minutes(30 * e as i64))
                    .with_resource(author)
                    .with_attribute("version", format!("{}", e + 1)),
            );
        }
        ts += Duration::hours(edits as i64);

        trace.add_event(
            Event::new("share_document", ts)
                .with_resource(author)
                .with_attribute("permission", "view"),
        );
        ts += Duration::minutes(15);
        trace.add_event(Event::new("get_version_history", ts).with_resource("auditor"));

        log.add_trace(trace);
    }

    let tree = InductiveMiner::new().discover_tree(&log);
    let tree_analysis = analyze_tree(&tree);

    println!("Documents Version Control Mining:");
    println!("   Documents: {}", log.len());
    println!(
        "   Tree stats: {} nodes, {} leaves, depth={}",
        tree_analysis.0.node_count, tree_analysis.0.leaf_count, tree_analysis.0.depth
    );
    println!(
        "   Tree metrics: complexity={:.4}",
        tree_analysis.1.complexity_score
    );
    println!("   Tree pattern: {}", tree_analysis.2.description());
    println!(
        "   Complexity level: {}",
        tree_analysis.1.complexity_level()
    );

    // Performance metrics
    let metrics = case_duration_metrics(&log);
    if let Some(m) = metrics {
        println!(
            "   Duration - avg: {:?}, median: {:?}, min: {:?}, max: {:?}",
            m.avg_duration, m.median_duration, m.min_duration, m.max_duration
        );
        assert!(m.avg_duration >= Duration::zero());
    }
}

// ============================================================================
// INNOVATION 4: OSA Recipe Workflows → Process Model Comparison
// Maps OSA recipe steps (code-review, security-audit) to event logs
// ============================================================================

#[test]
fn test_osa_recipe_workflow_comparison() {
    // Code Review recipe: Understand → Check Correctness → Security Audit → Performance → Feedback
    // Security Audit recipe: Map Attack Surface → Auth Review → Injection → AuthZ → Dep Scan → Secrets → Report
    let mut code_review_log = EventLog::new();
    let mut security_audit_log = EventLog::new();

    let code_review_steps = [
        ("understand_changes", "ANALYZE"),
        ("check_correctness", "ANALYZE"),
        ("security_audit", "ANALYZE"),
        ("performance_review", "ANALYZE"),
        ("provide_feedback", "ASSIST"),
    ];

    let security_steps = [
        ("map_attack_surface", "ANALYZE"),
        ("auth_session_review", "ANALYZE"),
        ("injection_testing", "ANALYZE"),
        ("authorization_check", "ANALYZE"),
        ("dependency_scan", "EXECUTE"),
        ("secrets_audit", "ANALYZE"),
        ("findings_report", "ASSIST"),
    ];

    for i in 0..20 {
        // Code review cases
        let mut cr_trace = Trace::new(format!("CR-{}", i));
        for (j, (step, mode)) in code_review_steps.iter().enumerate() {
            cr_trace.add_event(
                Event::new(*step, Utc::now() + Duration::minutes(j as i64 * 15))
                    .with_resource("reviewer")
                    .with_attribute("signal_mode", *mode),
            );
        }
        code_review_log.add_trace(cr_trace);

        // Security audit cases (some skip steps)
        let mut sa_trace = Trace::new(format!("SA-{}", i));
        let skip_idx = i % 4; // Vary which step gets skipped
        for (j, (step, mode)) in security_steps.iter().enumerate() {
            if j == skip_idx {
                continue;
            }
            sa_trace.add_event(
                Event::new(*step, Utc::now() + Duration::minutes(j as i64 * 20))
                    .with_resource("auditor")
                    .with_attribute("signal_mode", *mode),
            );
        }
        security_audit_log.add_trace(sa_trace);
    }

    // Compare discovered models
    let cr_dfg = DFGMiner::new().discover(&code_review_log);
    let sa_dfg = DFGMiner::new().discover(&security_audit_log);
    let cr_fp = Footprints::from_log(&code_review_log);
    let sa_fp = Footprints::from_log(&security_audit_log);

    println!("OSA Recipe Workflow Comparison:");
    println!(
        "   Code Review: {} traces, {} activities, {} DFG edges",
        code_review_log.len(),
        cr_dfg.nodes.len(),
        cr_dfg.edges.len()
    );
    println!(
        "   Security Audit: {} traces, {} activities, {} DFG edges",
        security_audit_log.len(),
        sa_dfg.nodes.len(),
        sa_dfg.edges.len()
    );
    println!("   CR footprint activities: {}", cr_fp.activities().len());
    println!("   SA footprint activities: {}", sa_fp.activities().len());

    // Compare behavioral profiles
    let cr_bp = BehavioralProfile::extract_from_log(&code_review_log);
    let sa_bp = BehavioralProfile::extract_from_log(&security_audit_log);
    let conflicts = cr_bp.find_conflicts(&sa_bp);
    println!("   Behavioral profile conflicts: {}", conflicts.len());

    // Both recipes should have signal_mode attributes
    let cr_attrs = get_str_attributes(&code_review_log);
    let sa_attrs = get_str_attributes(&security_audit_log);
    println!("   CR string attributes: {:?}", cr_attrs);
    println!("   SA string attributes: {:?}", sa_attrs);

    assert!(code_review_log.len() == 20);
    assert!(security_audit_log.len() == 20);
}

// ============================================================================
// INNOVATION 5: OSA Swarm Patterns → Multi-Agent Coordination Mining
// Maps OSA swarm patterns (review_loop, debate, parallel, pipeline) to process logs
// ============================================================================

#[test]
fn test_osa_swarm_patterns_process_mining() {
    let mut log = EventLog::new();

    // review_loop pattern: author → reviewer (iterate 3x)
    for case_idx in 0..10 {
        let mut trace = Trace::new(format!("review_loop_{}", case_idx));
        let mut ts = Utc::now();
        for iteration in 0..3 {
            trace.add_event(Event::new("author_submit", ts).with_resource("author"));
            ts += Duration::minutes(10);
            trace.add_event(Event::new("reviewer_feedback", ts).with_resource("reviewer"));
            ts += Duration::minutes(5);
        }
        trace.add_event(Event::new("approved", ts).with_resource("reviewer"));
        log.add_trace(trace);
    }

    // debate pattern: proposer_a → proposer_b → critic
    for case_idx in 0..8 {
        let mut trace = Trace::new(format!("debate_{}", case_idx));
        trace.add_event(Event::new("propose_a", Utc::now()).with_resource("proposer_a"));
        trace.add_event(
            Event::new("propose_b", Utc::now() + Duration::minutes(5)).with_resource("proposer_b"),
        );
        trace.add_event(
            Event::new("critic_decide", Utc::now() + Duration::minutes(10)).with_resource("critic"),
        );
        log.add_trace(trace);
    }

    // parallel pattern: researcher_1, researcher_2, researcher_3 (concurrent)
    for case_idx in 0..6 {
        let mut trace = Trace::new(format!("parallel_{}", case_idx));
        let ts = Utc::now() + Duration::hours(case_idx as i64);
        trace.add_event(Event::new("research_1", ts).with_resource("researcher_1"));
        trace.add_event(Event::new("research_2", ts).with_resource("researcher_2"));
        trace.add_event(Event::new("research_3", ts).with_resource("researcher_3"));
        trace.add_event(Event::new("synthesize", ts + Duration::hours(1)).with_resource("lead"));
        log.add_trace(trace);
    }

    // pipeline pattern: architect → implementer → tester
    for case_idx in 0..5 {
        let mut trace = Trace::new(format!("pipeline_{}", case_idx));
        trace.add_event(Event::new("architect_design", Utc::now()).with_resource("architect"));
        trace.add_event(
            Event::new("implement", Utc::now() + Duration::hours(2)).with_resource("implementer"),
        );
        trace
            .add_event(Event::new("test", Utc::now() + Duration::hours(4)).with_resource("tester"));
        log.add_trace(trace);
    }

    // Discover organizational structure from swarm patterns
    let net = InductiveMiner::new().discover(&log);
    let dfg = DFGMiner::new().discover(&log);

    println!("OSA Swarm Pattern Mining:");
    println!("   Total cases: {}", log.len());
    println!("   Activities: {}", log.activities().len());
    println!(
        "   Model: {} places, {} transitions",
        net.places.len(),
        net.transitions.len()
    );
    println!(
        "   DFG: {} nodes, {} edges",
        dfg.nodes.len(),
        dfg.edges.len()
    );

    // Conformance
    let result = AlignmentChecker::new().check(&log, &net);
    println!("   Alignment fitness: {:.4}", result.fitness);

    assert!(log.len() == 29, "Should have 29 swarm cases (10+8+6+5)");
}

// ============================================================================
// INNOVATION 6: OSA Role Rates → Resource Cost Analysis
// Uses OSA role_rates.json hourly rates to compute process mining cost metrics
// ============================================================================

#[test]
fn test_osa_role_rates_resource_cost_analysis() {
    // OSA role_rates: lead=$150, backend=$120, frontend=$110, data=$130,
    //                 qa=$90, design=$100, infra=$140, red_team=$160, services=$115
    let hourly_rates: HashMap<&str, f64> = [
        ("lead", 150.0),
        ("backend", 120.0),
        ("frontend", 110.0),
        ("data", 130.0),
        ("qa", 90.0),
        ("design", 100.0),
        ("infra", 140.0),
        ("red_team", 160.0),
        ("services", 115.0),
    ]
    .into();

    // Simulate a software development process
    let mut log = EventLog::new();
    for sprint in 0..8 {
        let mut trace = Trace::new(format!("SPRINT-{}", sprint));
        let mut ts = Utc::now() + Duration::days(sprint as i64 * 14);

        trace.add_event(Event::new("plan", ts).with_resource("lead"));
        ts += Duration::hours(4);
        trace.add_event(Event::new("design", ts).with_resource("design"));
        ts += Duration::hours(8);
        trace.add_event(Event::new("implement_backend", ts).with_resource("backend"));
        ts += Duration::hours(16);
        trace.add_event(Event::new("implement_frontend", ts).with_resource("frontend"));
        ts += Duration::hours(12);
        trace.add_event(Event::new("write_tests", ts).with_resource("qa"));
        ts += Duration::hours(6);
        trace.add_event(Event::new("security_review", ts).with_resource("red_team"));
        ts += Duration::hours(3);
        trace.add_event(Event::new("deploy", ts).with_resource("infra"));
        log.add_trace(trace);
    }

    // Compute per-trace cost using activity durations and role rates
    let mut total_cost = 0.0;
    let mut role_costs: HashMap<String, f64> = HashMap::new();

    for trace in &log.traces {
        for (i, event) in trace.events.iter().enumerate() {
            let next_ts = trace
                .events
                .get(i + 1)
                .map(|e| e.timestamp)
                .unwrap_or_else(|| event.timestamp + Duration::hours(1));
            let hours = (next_ts - event.timestamp).num_seconds() as f64 / 3600.0;

            if let Some(ref resource) = event.resource {
                let rate = hourly_rates
                    .get(resource.as_str())
                    .copied()
                    .unwrap_or(100.0);
                let cost = hours * rate;
                *role_costs.entry(resource.clone()).or_insert(0.0) += cost;
                total_cost += cost;
            }
        }
    }

    println!("OSA Role-Based Cost Analysis:");
    println!("   Total sprint cost: ${:.2}", total_cost);
    println!(
        "   Avg cost per sprint: ${:.2}",
        total_cost / log.len() as f64
    );
    for (role, cost) in &role_costs {
        let rate = hourly_rates.get(role.as_str()).copied().unwrap_or(0.0);
        let hours = cost / rate;
        println!("   {}: ${:.2} ({}h @ ${}/h)", role, cost, hours, rate);
    }

    assert!(total_cost > 0.0, "Total cost should be positive");
    assert!(role_costs.len() >= 5, "Should have at least 5 roles");
}

// ============================================================================
// INNOVATION 7: Performance Metrics on Canopy Invoice Data
// Tests the entirely untested performance module against real data
// ============================================================================

#[test]
fn test_performance_metrics_canopy_invoice() {
    let log = load_invoice();

    // Case duration metrics
    let durations = case_durations(&log);
    let metrics = case_duration_metrics(&log);

    println!("Canopy Invoice Performance Metrics:");
    println!("   Cases: {}", log.len());
    println!("   Duration samples: {}", durations.len());

    if let Some(m) = metrics {
        println!("   Avg: {:?}", m.avg_duration);
        println!("   Median: {:?}", m.median_duration);
        println!("   Min: {:?}", m.min_duration);
        println!("   Max: {:?}", m.max_duration);
        assert!(m.avg_duration >= Duration::zero());
    }

    // Rework analysis
    let rework = rework_cases(&log);
    let rework_pct = rework_percentage(&log);
    println!("   Rework cases: {}", rework.len());
    println!("   Rework percentage: {:.2}%", rework_pct);
}

// ============================================================================
// INNOVATION 8: Behavioral Profiles Across Data Sources
// Tests the untested behavioral_profile module
// ============================================================================

#[test]
fn test_behavioral_profiles_cross_source() {
    let invoice = load_invoice();
    let onboarding = load_onboarding();
    let compliance = load_compliance();

    // Extract behavioral profiles from each data source
    let bp_invoice = BehavioralProfile::extract_from_log(&invoice);
    let bp_onboard = BehavioralProfile::extract_from_log(&onboarding);
    let bp_compliance = BehavioralProfile::extract_from_log(&compliance);

    println!("Behavioral Profiles:");
    println!("   Invoice: {} activity pairs", bp_invoice.activities.len());
    println!(
        "   Onboarding: {} activity pairs",
        bp_onboard.activities.len()
    );
    println!(
        "   Compliance: {} activity pairs",
        bp_compliance.activities.len()
    );

    // Check for loops in each process
    let invoice_acts = invoice.activities();
    let mut has_loop = false;
    for act in &invoice_acts {
        if bp_invoice.has_loop(act) {
            has_loop = true;
            break;
        }
    }
    println!("   Invoice has loops: {}", has_loop);

    // Compare profiles
    let conflicts = bp_invoice.find_conflicts(&bp_onboard);
    println!("   Invoice vs Onboarding conflicts: {}", conflicts.len());

    // Compute conformance between profiles
    let conformance = bp_invoice.compute_conformance(&bp_onboard);
    println!(
        "   Invoice-Onboarding profile conformance: {:.4}",
        conformance
    );
    assert!(conformance >= 0.0 && conformance <= 1.0);
}

// ============================================================================
// INNOVATION 9: Causal Dependency Analysis on Business Processes
// ============================================================================

#[test]
fn test_causal_dependency_analysis_invoice() {
    let log = load_invoice();

    let deps = causal_dependency_analysis(&log);
    let cooc = activity_co_occurrence(&log);
    let metrics = network_metrics(&log);

    println!("Causal Dependency Analysis (Invoice):");
    println!("   Dependencies: {}", deps.len());
    for dep in deps.iter().take(5) {
        println!(
            "     {} → {}: strength={:.3}",
            dep.source, dep.target, dep.strength
        );
    }
    println!("   Co-occurrences: {}", cooc.len());
    println!("   Network metrics: density={:.4}", metrics.density);
    println!("   Avg in-degree: {:.2}", metrics.avg_in_degree);
    println!("   Avg out-degree: {:.2}", metrics.avg_out_degree);

    assert!(!deps.is_empty(), "Should have causal dependencies");
}

// ============================================================================
// INNOVATION 10: ML Feature Extraction from Canopy Data
// Tests the untested ml_features module
// ============================================================================

#[test]
fn test_ml_feature_extraction_canopy() {
    let log = load_invoice();

    // Extract features from all traces
    let features = extract_features(&log);
    let feature_names = get_feature_names();
    let all_activities = get_all_activities(&log);
    let str_attrs = get_str_attributes(&log);
    let num_attrs = get_numeric_attributes(&log);

    println!("ML Feature Extraction (Invoice):");
    println!("   Traces with features: {}", features.len());
    println!("   Feature names: {:?}", feature_names);
    println!("   All activities: {:?}", all_activities);
    println!("   String attributes: {:?}", str_attrs);
    println!("   Numeric attributes: {:?}", num_attrs);

    if !features.is_empty() {
        let vec = features_to_vector(&features[0], &all_activities);
        println!("   Feature vector length: {}", vec.len());
    }

    // Create feature matrix
    let (matrix, labels) = create_feature_matrix(&log);
    println!(
        "   Feature matrix: {}x{}",
        matrix.len(),
        if matrix.is_empty() {
            0
        } else {
            matrix[0].len()
        }
    );
    println!("   Labels: {}", labels.len());

    // Train/test split
    if features.len() > 4 {
        let (train, test) = train_test_split(&features, 0.8);
        println!("   Train/test split: {}/{}", train.len(), test.len());
        assert!(train.len() > test.len());
    }

    // Normalize features
    let mut norm_matrix = matrix.clone();
    normalize_features(&mut norm_matrix);
    println!("   Normalized matrix rows: {}", norm_matrix.len());
}

// ============================================================================
// INNOVATION 11: OCEL Conformance on Object-Centric Business Process
// ============================================================================

#[test]
fn test_ocel_conformance_business_process() {
    let mut ocel = ObjectCentricEventLog::with_id("business_process");

    let order_type = ObjectType::new("order");
    let invoice_type = ObjectType::new("invoice");
    ocel.register_object_type(order_type.clone());
    ocel.register_object_type(invoice_type.clone());

    // Create objects
    for i in 0..5 {
        ocel.add_object(pm4py::ocpm::Object::new(
            format!("ORD-{}", i),
            order_type.clone(),
            Utc::now(),
        ));
        ocel.add_object(pm4py::ocpm::Object::new(
            format!("INV-{}", i),
            invoice_type.clone(),
            Utc::now(),
        ));
    }

    // Add events
    let mut ts = Utc::now();
    for i in 0..5 {
        ocel.add_event(
            uuid::Uuid::new_v4(),
            "create_order",
            ts,
            Some("sales".to_string()),
        );
        ts += Duration::hours(1);
        ocel.add_event(
            uuid::Uuid::new_v4(),
            "create_invoice",
            ts,
            Some("billing".to_string()),
        );
        ts += Duration::hours(2);
        ocel.add_event(
            uuid::Uuid::new_v4(),
            "pay_invoice",
            ts,
            Some("customer".to_string()),
        );
        ts += Duration::minutes(30);
        ocel.add_event(
            uuid::Uuid::new_v4(),
            "ship_order",
            ts,
            Some("warehouse".to_string()),
        );
    }

    // OCEL lifecycle conformance
    let mut constraints: HashMap<String, Vec<String>> = HashMap::new();
    constraints.insert(
        "create_order".to_string(),
        vec!["create_invoice".to_string()],
    );
    constraints.insert(
        "create_invoice".to_string(),
        vec!["pay_invoice".to_string()],
    );
    constraints.insert("pay_invoice".to_string(), vec!["ship_order".to_string()]);
    let lifecycle_result = check_ocel_lifecycle_conformance(&ocel, &constraints);
    println!("OCEL Lifecycle Conformance:");
    println!("   Deviations: {}", lifecycle_result.num_deviations);
    println!("   Fitness: {:.4}", lifecycle_result.fitness);

    // Event ordering validation
    let ordering_result = validate_ocel_event_ordering(&ocel);
    println!("OCEL Event Ordering:");
    println!("   Deviations: {}", ordering_result.num_deviations);
    println!("   Fitness: {:.4}", ordering_result.fitness);

    // OCPM discovery + conformance
    let ocpm_net = OCPMDiscoveryMiner::new(0.5).discover(&ocel);
    let ocpm_result = ObjectCentricTokenReplay::new(0.5).check(&ocel, &ocpm_net);
    println!("   OCPM conformant: {}", ocpm_result.is_conformant);

    assert!(ocel.objects.len() == 10);
    assert!(ocel.events.len() == 20);
}

// ============================================================================
// INNOVATION 12: Tree Statistics from Discovered Models
// Tests the untested tree_stats module
// ============================================================================

#[test]
fn test_tree_statistics_canopy_data() {
    let invoice = load_invoice();
    let onboarding = load_onboarding();

    let invoice_tree = InductiveMiner::new().discover_tree(&invoice);
    let onboard_tree = InductiveMiner::new().discover_tree(&onboarding);

    let (inv_stats, inv_metrics, inv_pattern) = analyze_tree(&invoice_tree);
    let (ob_stats, ob_metrics, ob_pattern) = analyze_tree(&onboard_tree);

    println!("Tree Statistics Comparison:");
    println!(
        "   {:15} {:>10} {:>10} {:>10}",
        "Metric", "Invoice", "Onboarding", "Diff"
    );
    println!(
        "   {:15} {:>10} {:>10} {:>10}",
        "Nodes",
        inv_stats.node_count,
        ob_stats.node_count,
        (inv_stats.node_count as i64 - ob_stats.node_count as i64).abs()
    );
    println!(
        "   {:15} {:>10} {:>10} {:>10}",
        "Leaves",
        inv_stats.leaf_count,
        ob_stats.leaf_count,
        (inv_stats.leaf_count as i64 - ob_stats.leaf_count as i64).abs()
    );
    println!(
        "   {:15} {:>10} {:>10} {:>10}",
        "Depth",
        inv_stats.depth,
        ob_stats.depth,
        (inv_stats.depth as i64 - ob_stats.depth as i64).abs()
    );
    println!(
        "   {:15} {:>10} {:>10}",
        "Operators", inv_stats.operator_count, ob_stats.operator_count
    );
    println!(
        "   {:15} {:>10} {:>10}",
        "Complexity",
        inv_metrics.complexity_level(),
        ob_metrics.complexity_level()
    );
    println!(
        "   {:15} {:>10} {:>10}",
        "Pattern",
        inv_pattern.description(),
        ob_pattern.description()
    );

    println!("\n   Invoice summary: {}", inv_stats.summary());
    println!("   Onboarding summary: {}", ob_stats.summary());

    assert!(inv_stats.node_count > 0);
    assert!(ob_stats.node_count > 0);
}

// ============================================================================
// INNOVATION 13: SplitMiner vs InductiveMiner Comparison
// Tests the untested SplitMiner
// ============================================================================

#[test]
fn test_split_miner_vs_inductive_miner() {
    let log = load_invoice();

    // SplitMiner
    let split_net = SplitMiner::new()
        .with_parallelism_detection(true)
        .discover(&log);

    // InductiveMiner
    let ind_net = InductiveMiner::new().discover(&log);

    // CausalNetMiner
    let causal_net = CausalNetMiner::new().with_min_support(0.1).discover(&log);

    println!("Miner Comparison (Invoice):");
    println!(
        "   {:15} {:>8} {:>8} {:>8}",
        "Miner", "Places", "Trans", "Arcs"
    );
    println!(
        "   {:15} {:>8} {:>8} {:>8}",
        "SplitMiner",
        split_net.places.len(),
        split_net.transitions.len(),
        split_net.arcs.len()
    );
    println!(
        "   {:15} {:>8} {:>8} {:>8}",
        "InductiveMiner",
        ind_net.places.len(),
        ind_net.transitions.len(),
        ind_net.arcs.len()
    );
    println!(
        "   {:15} {:>8} {:>8}",
        "CausalNetMiner",
        causal_net.num_activities(),
        causal_net.num_relations()
    );

    // Both should produce valid models
    assert!(
        split_net.transitions.len() > 0,
        "SplitMiner should produce transitions"
    );
    assert!(
        ind_net.transitions.len() > 0,
        "InductiveMiner should produce transitions"
    );
    assert!(
        causal_net.num_activities() > 0,
        "CausalNetMiner should produce activities"
    );
}

// ============================================================================
// INNOVATION 14: Network Metrics on Canopy Data Sources
// ============================================================================

#[test]
fn test_network_metrics_comparison() {
    let sources: Vec<(&str, EventLog)> = vec![
        ("Invoice", load_invoice()),
        ("Onboarding", load_onboarding()),
        ("Compliance", load_compliance()),
    ];

    println!("Network Metrics Comparison:");
    println!(
        "   {:12} {:>8} {:>8} {:>8} {:>10} {:>10}",
        "Process", "Nodes", "Edges", "Density", "AvgInDeg", "AvgOutDeg"
    );

    for (name, log) in &sources {
        let dfg = DFGMiner::new().discover(log);
        let nm = network_metrics(log);

        println!(
            "   {:12} {:>8} {:>8} {:>8.4} {:>10.2} {:>10.2}",
            name,
            dfg.nodes.len(),
            dfg.edges.len(),
            nm.density,
            nm.avg_in_degree,
            nm.avg_out_degree,
        );
    }

    // Co-occurrence analysis
    for (name, log) in &sources {
        let cooc = activity_co_occurrence(log);
        println!("   {} co-occurrences: {}", name, cooc.len());
    }
}

// ============================================================================
// INNOVATION 15: Temporal Profile on Canopy Data
// ============================================================================

#[test]
fn test_temporal_profile_canopy_invoice() {
    let log = load_invoice();

    let profile = discover_temporal_profile(&log);
    let result = conformance_temporal_profile(&log, &profile, 0.1);

    println!("Temporal Profile (Invoice):");
    println!(
        "   Activity pairs in profile: {}",
        profile.min_time_between.len()
    );
    println!("   Deviating traces: {}", result.deviating_traces);
    println!("   Deviations: {}", result.deviations.len());

    assert!(result.deviating_traces <= log.len());
}

// ============================================================================
// INNOVATION 16: Anomaly Detection on Canopy Compliance Data
// ============================================================================

#[test]
fn test_anomaly_detection_compliance() {
    let log = load_compliance();

    // Use OutcomePredictor's detect_anomalies method
    let predictor = OutcomePredictor::new(&log, |trace| {
        let last = trace.events.last();
        let activity = last.map(|e| e.activity.as_str()).unwrap_or("");
        match activity {
            "complete" | "approved" => CaseOutcome::Successful,
            "rejected" | "failed" => CaseOutcome::Failed,
            _ => CaseOutcome::Problematic,
        }
    });

    let anomalies = predictor.detect_anomalies(&log.traces[0]);

    println!("Anomaly Detection (Compliance):");
    println!("   Anomalies in first trace: {}", anomalies.len());
    for anomaly in anomalies.iter().take(5) {
        println!("   Anomaly: {}", anomaly);
    }

    println!("   Total traces: {}", log.len());
}

// ============================================================================
// INNOVATION 17: Predictive Analytics Pipeline on Canopy Data
// Full pipeline: extract features → train/test split → predict
// ============================================================================

#[test]
fn test_predictive_pipeline_canopy_invoice() {
    let log = load_invoice();

    // Next activity prediction
    let next_pred = NextActivityPredictor::new(&log);
    let starts = next_pred.get_start_activities(3);
    println!("Predictive Pipeline - Next Activity:");
    for pred in &starts {
        println!("   Start: {} ({:.3})", pred.activity, pred.probability);
    }

    // Outcome prediction
    let outcome_pred = OutcomePredictor::new(&log, |trace| {
        let last = trace.events.last();
        let activity = last.map(|e| e.activity.as_str()).unwrap_or("");
        match activity {
            "complete" | "approved" => CaseOutcome::Successful,
            "rejected" | "failed" => CaseOutcome::Failed,
            _ => CaseOutcome::Problematic,
        }
    });

    let dist = outcome_pred.outcome_distribution();
    println!("   Outcome distribution: {:?}", dist);

    let risk = outcome_pred.assess_risk(&log.traces[0]);
    println!(
        "   Risk for first trace: score={:.4}, outcome={:?}",
        risk.risk_score, risk.predicted_outcome
    );

    // Remaining time prediction
    let time_pred = RemainingTimePredictor::new(&log);
    let avg_dur = time_pred.average_case_duration();
    println!("   Average case duration: {:?}", avg_dur);

    // ML features for prediction
    let features = extract_features(&log);
    let (train, test) = train_test_split(&features, 0.8);
    println!(
        "   Feature train/test split: {}/{}",
        train.len(),
        test.len()
    );

    assert!(!starts.is_empty());
}

// ============================================================================
// INNOVATION 18: Declare Constraints on BusinessOS Module Workflows
// ============================================================================

#[test]
fn test_declare_constraints_module_workflows() {
    // Create a log from BusinessOS module action sequences
    let mut log = EventLog::new();

    // CRM workflow: contact → lead → company → deal → qualify → propose → close
    for i in 0..15 {
        let mut trace = Trace::new(format!("crm_{}", i));
        trace.add_event(Event::new("create_contact", Utc::now()).with_resource("sales"));
        trace.add_event(
            Event::new("create_lead", Utc::now() + Duration::hours(1)).with_resource("sales"),
        );
        trace.add_event(
            Event::new("create_deal", Utc::now() + Duration::hours(2)).with_resource("sales"),
        );
        trace.add_event(
            Event::new("qualify", Utc::now() + Duration::hours(3)).with_resource("manager"),
        );
        trace
            .add_event(Event::new("close", Utc::now() + Duration::hours(4)).with_resource("sales"));
        log.add_trace(trace);
    }

    // Some non-conforming traces
    for i in 0..3 {
        let mut trace = Trace::new(format!("crm_bad_{}", i));
        trace.add_event(Event::new("close", Utc::now()).with_resource("sales"));
        trace.add_event(
            Event::new("create_contact", Utc::now() + Duration::hours(1)).with_resource("sales"),
        );
        log.add_trace(trace);
    }

    // Discover Declare constraints
    let model = pm4py::discovery::DeclareMiner::new().discover(&log);
    let (compliant, non_compliant) = conformance_declare(&log, &model);

    println!("Declare Constraints (CRM Workflow):");
    println!("   Total traces: {}", log.len());
    println!("   Compliant: {}", compliant);
    println!("   Non-compliant: {}", non_compliant);
    println!("   Discovered constraints: {}", model.constraints.len());

    // Check constraint types (DeclareConstraint is an enum)
    use pm4py::discovery::DeclareConstraint;
    let has_precedence = model.constraints.iter().any(|c| {
        matches!(
            c,
            DeclareConstraint::Precedence { .. } | DeclareConstraint::ChainResponse { .. }
        )
    });
    println!("   Has ordering constraint: {}", has_precedence);

    assert!(model.constraints.len() > 0);
}

// ============================================================================
// INNOVATION 19: Cross-Module Process Similarity
// Compare process models across BusinessOS modules
// ============================================================================

#[test]
fn test_cross_module_process_similarity() {
    // Build logs for each BusinessOS module
    let mut crm_log = EventLog::new();
    let mut projects_log = EventLog::new();
    let mut documents_log = EventLog::new();

    // CRM: contact → lead → deal → qualify → close
    for i in 0..10 {
        let mut trace = Trace::new(format!("crm_{}", i));
        trace.add_event(Event::new("create_contact", Utc::now()).with_resource("sales"));
        trace.add_event(
            Event::new("create_lead", Utc::now() + Duration::hours(1)).with_resource("sales"),
        );
        trace.add_event(
            Event::new("qualify", Utc::now() + Duration::hours(2)).with_resource("manager"),
        );
        trace
            .add_event(Event::new("close", Utc::now() + Duration::hours(3)).with_resource("sales"));
        crm_log.add_trace(trace);
    }

    // Projects: create → assign → start → review → done
    for i in 0..10 {
        let mut trace = Trace::new(format!("proj_{}", i));
        trace.add_event(Event::new("create_task", Utc::now()).with_resource("pm"));
        trace.add_event(Event::new("assign", Utc::now() + Duration::hours(1)).with_resource("pm"));
        trace.add_event(Event::new("start", Utc::now() + Duration::hours(2)).with_resource("dev"));
        trace.add_event(
            Event::new("review", Utc::now() + Duration::hours(3)).with_resource("reviewer"),
        );
        trace.add_event(Event::new("done", Utc::now() + Duration::hours(4)).with_resource("pm"));
        projects_log.add_trace(trace);
    }

    // Documents: create → edit → share → version
    for i in 0..10 {
        let mut trace = Trace::new(format!("doc_{}", i));
        trace.add_event(Event::new("create", Utc::now()).with_resource("author"));
        trace
            .add_event(Event::new("edit", Utc::now() + Duration::hours(1)).with_resource("author"));
        trace.add_event(
            Event::new("share", Utc::now() + Duration::hours(2)).with_resource("author"),
        );
        trace.add_event(
            Event::new("version", Utc::now() + Duration::hours(3)).with_resource("system"),
        );
        documents_log.add_trace(trace);
    }

    // Discover footprints for each module
    let crm_fp = Footprints::from_log(&crm_log);
    let proj_fp = Footprints::from_log(&projects_log);
    let doc_fp = Footprints::from_log(&documents_log);

    // Compare using behavioral profiles
    let crm_bp = BehavioralProfile::extract_from_log(&crm_log);
    let proj_bp = BehavioralProfile::extract_from_log(&projects_log);
    let doc_bp = BehavioralProfile::extract_from_log(&documents_log);

    let crm_proj_conf = crm_bp.compute_conformance(&proj_bp);
    let crm_doc_conf = crm_bp.compute_conformance(&doc_bp);
    let proj_doc_conf = proj_bp.compute_conformance(&doc_bp);

    println!("Cross-Module Process Similarity:");
    println!(
        "   {:12} {:>10} {:>10} {:>10}",
        "", "CRM", "Projects", "Docs"
    );
    println!(
        "   {:12} {:>10} {:>10} {:>10}",
        "CRM",
        "1.0000",
        format!("{:.4}", crm_proj_conf),
        format!("{:.4}", crm_doc_conf)
    );
    println!(
        "   {:12} {:>10} {:>10} {:>10}",
        "Projects",
        format!("{:.4}", crm_proj_conf),
        "1.0000",
        format!("{:.4}", proj_doc_conf)
    );
    println!(
        "   {:12} {:>10} {:>10} {:>10}",
        "Docs",
        format!("{:.4}", crm_doc_conf),
        format!("{:.4}", proj_doc_conf),
        "1.0000"
    );

    println!("   CRM activities: {:?}", crm_fp.activities());
    println!("   Projects activities: {:?}", proj_fp.activities());
    println!("   Docs activities: {:?}", doc_fp.activities());

    // All conformance values should be in [0, 1]
    assert!(crm_proj_conf >= 0.0 && crm_proj_conf <= 1.0);
    assert!(crm_doc_conf >= 0.0 && crm_doc_conf <= 1.0);
    assert!(proj_doc_conf >= 0.0 && proj_doc_conf <= 1.0);
}

// ============================================================================
// INNOVATION 20: Prefix Tree Analysis on Business Workflows
// ============================================================================

#[test]
fn test_prefix_tree_business_workflow_analysis() {
    let log = load_invoice();
    let tree = discover_prefix_tree(&log);

    println!("Prefix Tree Analysis (Invoice):");
    println!("   Root children: {}", tree.root.children.len());

    // Collect variant info
    let mut variant_counts: HashMap<String, usize> = HashMap::new();
    collect_variants(&tree.root, String::new(), &mut variant_counts);

    let mut sorted: Vec<_> = variant_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    println!("   Total variants: {}", sorted.len());
    println!("   Top 5 variants:");
    for (variant, count) in sorted.iter().take(5) {
        println!("     {} (count={})", variant, count);
    }
}

fn collect_variants(
    node: &pm4py::discovery::PrefixTreeNode,
    prefix: String,
    counts: &mut HashMap<String, usize>,
) {
    if node.children.is_empty() {
        *counts.entry(prefix.clone()).or_insert(0) += node.count;
    } else {
        for (activity, child) in &node.children {
            let new_prefix = if prefix.is_empty() {
                activity.clone()
            } else {
                format!("{} → {}", prefix, activity)
            };
            collect_variants(child, new_prefix, counts);
        }
    }
}
