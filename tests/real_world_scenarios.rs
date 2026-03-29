use chrono::{Duration, Utc};
use pm4py::conformance::FootprintsConformanceChecker;
use pm4py::discovery::{AlphaMiner, SplitMiner, TreeMiner};
/// Real-world scenario integration tests for PM4Py Rust
use pm4py::log::{Event, EventLog, Trace};
use pm4py::models::DirectlyFollowsGraph;
use pm4py::models::Footprints;

fn create_manufacturing_log(num_orders: usize) -> EventLog {
    let mut log = EventLog::new();
    let start_time = Utc::now();

    for order_id in 0..num_orders {
        let mut trace = Trace::new(format!("order_{}", order_id));
        let mut current_time = start_time + Duration::minutes((order_id as i64) * 5);

        trace.add_event(Event::new("receive_raw_material", current_time));
        current_time = current_time + Duration::minutes(2);

        trace.add_event(Event::new("initial_quality_check", current_time));
        current_time = current_time + Duration::minutes(1);

        if order_id % 10 == 0 {
            trace.add_event(Event::new("rework_required", current_time));
            current_time = current_time + Duration::minutes(3);
            trace.add_event(Event::new("initial_quality_check", current_time));
            current_time = current_time + Duration::minutes(1);
        }

        trace.add_event(Event::new("start_assembly", current_time));
        current_time = current_time + Duration::minutes(5);
        trace.add_event(Event::new("assembly_complete", current_time));
        current_time = current_time + Duration::minutes(1);

        if order_id % 3 == 0 {
            trace.add_event(Event::new("performance_test", current_time));
        } else if order_id % 3 == 1 {
            trace.add_event(Event::new("safety_test", current_time));
        } else {
            trace.add_event(Event::new("durability_test", current_time));
        }
        current_time = current_time + Duration::minutes(4);

        trace.add_event(Event::new("prepare_packaging", current_time));
        current_time = current_time + Duration::minutes(2);
        trace.add_event(Event::new("package_complete", current_time));

        log.add_trace(trace);
    }

    log
}

fn create_healthcare_log(num_patients: usize) -> EventLog {
    let mut log = EventLog::new();
    let start_time = Utc::now();

    for patient_id in 0..num_patients {
        let mut trace = Trace::new(format!("patient_{}", patient_id));
        let mut current_time = start_time + Duration::hours((patient_id as i64) % 24);

        trace.add_event(Event::new("patient_checkin", current_time));
        current_time = current_time + Duration::minutes(5);

        trace.add_event(Event::new("registration", current_time));
        current_time = current_time + Duration::minutes(10);

        trace.add_event(Event::new("triage_assessment", current_time));
        current_time = current_time + Duration::minutes(5);

        if patient_id % 4 == 0 {
            trace.add_event(Event::new("preliminary_labs", current_time));
            current_time = current_time + Duration::minutes(15);
        }

        trace.add_event(Event::new("doctor_examination", current_time));
        current_time = current_time + Duration::minutes(20);

        match patient_id % 5 {
            0 => trace.add_event(Event::new("medication_prescribed", current_time)),
            1 => {
                trace.add_event(Event::new("physical_therapy", current_time));
                current_time = current_time + Duration::minutes(30);
            }
            2 => {
                trace.add_event(Event::new("surgical_procedure", current_time));
                current_time = current_time + Duration::minutes(60);
            }
            3 => {
                trace.add_event(Event::new("diagnostic_imaging", current_time));
                current_time = current_time + Duration::minutes(25);
            }
            _ => {
                trace.add_event(Event::new("observation_period", current_time));
                current_time = current_time + Duration::minutes(120);
            }
        }

        current_time = current_time + Duration::minutes(10);
        trace.add_event(Event::new("final_review", current_time));
        current_time = current_time + Duration::minutes(10);
        trace.add_event(Event::new("discharge", current_time));

        log.add_trace(trace);
    }

    log
}

#[test]
fn test_manufacturing_discovery_100_orders() {
    let log = create_manufacturing_log(100);

    assert_eq!(log.len(), 100);
    assert!(log.num_events() > 600);

    let dfg = DirectlyFollowsGraph::from_log(&log);
    assert!(!dfg.nodes.is_empty());
    assert!(dfg.nodes.iter().any(|a| a == "receive_raw_material"));
    assert!(dfg.nodes.iter().any(|a| a == "package_complete"));
}

#[test]
fn test_healthcare_workflow_realistic_paths() {
    let log = create_healthcare_log(100);

    assert_eq!(log.len(), 100);
    assert!(log.num_events() > 700);

    let dfg = DirectlyFollowsGraph::from_log(&log);
    assert!(dfg.nodes.iter().any(|a| a == "patient_checkin"));
    assert!(dfg.nodes.iter().any(|a| a == "discharge"));
}

#[test]
fn test_discovery_multiple_miners_same_log() {
    let log = create_manufacturing_log(75);

    let alpha_miner = AlphaMiner::new();
    let alpha_net = alpha_miner.discover(&log);
    assert!(alpha_net.transitions.len() > 0);

    let dfg = DirectlyFollowsGraph::from_log(&log);
    assert!(!dfg.nodes.is_empty());

    let split_miner = SplitMiner::new();
    let split_net = split_miner.discover(&log);
    assert!(split_net.transitions.len() > 0);

    let tree_miner = TreeMiner::new();
    let _tree = tree_miner.discover(&log);
}

#[test]
fn test_healthcare_conformance_checking() {
    let log = create_healthcare_log(50);

    let mut footprints = Footprints::new();
    for trace in &log.traces {
        for i in 0..trace.events.len().saturating_sub(1) {
            let a = &trace.events[i].activity;
            let b = &trace.events[i + 1].activity;
            footprints.set_relationship(a, b, pm4py::models::ActivityRelationship::DirectlyFollows);
        }
    }

    let result = FootprintsConformanceChecker::check_log(&log, &footprints);
    assert!(result.fitness >= 0.0 && result.fitness <= 1.0);
}

#[test]
fn test_large_scale_500_orders() {
    let log = create_manufacturing_log(500);
    assert!(log.num_events() > 3000);

    let dfg = DirectlyFollowsGraph::from_log(&log);
    assert!(!dfg.nodes.is_empty());

    let tree_miner = TreeMiner::new();
    let _tree = tree_miner.discover(&log);
}

#[test]
fn test_multiple_miners_consistency() {
    let log = create_healthcare_log(100);

    let alpha_miner = AlphaMiner::new();
    let alpha_net = alpha_miner.discover(&log);

    let dfg = DirectlyFollowsGraph::from_log(&log);

    let split_miner = SplitMiner::new();
    let split_net = split_miner.discover(&log);

    assert!(alpha_net.transitions.len() > 0);
    assert!(!dfg.nodes.is_empty());
    assert!(split_net.transitions.len() > 0);
}

#[test]
fn test_filtering_and_discovery() {
    let log = create_manufacturing_log(50);

    let filtered_log = log.filter_by_activity("start_assembly");
    assert!(filtered_log.len() <= log.len());

    let dfg = DirectlyFollowsGraph::from_log(&filtered_log);
    assert!(!dfg.nodes.is_empty());
}

#[test]
fn test_workflow_end_to_end_pipeline() {
    let log = create_healthcare_log(75);

    let dfg = DirectlyFollowsGraph::from_log(&log);
    assert!(!dfg.nodes.is_empty());

    let mut footprints = Footprints::new();
    for trace in &log.traces {
        for i in 0..trace.events.len().saturating_sub(1) {
            footprints.set_relationship(
                &trace.events[i].activity,
                &trace.events[i + 1].activity,
                pm4py::models::ActivityRelationship::DirectlyFollows,
            );
        }
    }

    let result = FootprintsConformanceChecker::check_log(&log, &footprints);
    assert!(result.fitness >= 0.0);

    assert_eq!(log.len(), 75);
    assert!(log.num_events() > 500);
}
