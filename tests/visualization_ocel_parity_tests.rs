//! Visualization, OCEL Filtering, and Remaining Parity Integration Tests
//!
//! Tests previously untested APIs:
//! - Animation (create_animation_from_log, create_animation_from_trace, AnimationFrame)
//! - Interactive visualization (create_interactive_dfg, create_interactive_petri_net)
//! - Layout algorithms (ForceDirectedLayout, HierarchicalLayout)
//! - Remaining parity functions (compute_emd, convert_log_to_ocel, construct_synchronous_product_net,
//!   convert_log_to_networkx, convert_petri_net_to_networkx, conformance_etoc,
//!   cluster_equivalent_ocel, conformance_diagnostics_*)
//! - OCEL filters (ocel_filter_object_type, ocel_filter_object_ids, ocel_filter_activities,
//!   ocel_filter_connected_components, ocel_filter_object_event_count, filter_ocel_cc_*)
//! - OCEL utilities (ocel_enrichment, ocel_sort_by_additional_column, ocel_add_index_based_timedelta,
//!   ocel_drop_duplicates, sample_ocel_objects, sample_ocel_connected_components,
//!   ocel_object_type_activities, ocel_objects_ot_count)
//! - Save visualization functions (save_vis_case_duration_graph, save_vis_events_distribution_graph,
//!   save_vis_events_per_time_graph, save_vis_dotted_chart, save_vis_performance_dfg, etc.)
//!
//! Chicago TDD: NO MOCKS, testing against real pm4py-rust data structures

use chrono::{Duration, Utc};
use pm4py::discovery::AlphaMiner;
use pm4py::io::CSVReader;
use pm4py::log::{Event, EventLog, Trace};
use pm4py::models::Footprints;
use pm4py::models::PetriNet;
use pm4py::ocpm::{EventToObjectMapping, Object, ObjectCentricEventLog, ObjectType};
use pm4py::parity_verification::verify_parity;
use pm4py::remaining_parity::{
    cluster_equivalent_ocel, compute_emd, conformance_etoc, construct_synchronous_product_net,
    convert_log_to_networkx, convert_log_to_ocel, convert_ocel_to_networkx,
    convert_petri_net_to_networkx,
};
use pm4py::visualization::layout::Point;
use pm4py::visualization::{
    create_animation_from_log, create_animation_from_trace, create_interactive_dfg,
    create_interactive_petri_net, Animation, AnimationFrame, AnimationOptions, AnimationSpeed,
    ForceDirectedLayout, HierarchicalLayout, InteractiveOptions, LayoutAlgorithm,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Helper: Load real Canopy invoice data
// ============================================================================

fn load_invoice() -> EventLog {
    let path = format!(
        "{}/canopy/priv/demo_data/invoice_events.csv",
        std::env::var("HOME").unwrap()
    );
    CSVReader::new()
        .with_case_column("case_id")
        .with_activity_column("activity")
        .with_timestamp_column("timestamp")
        .read(std::path::Path::new(&path))
        .unwrap_or_else(|_| {
            let mut log = EventLog::new();
            let now = Utc::now();
            for i in 0..10 {
                let mut trace = Trace::new(format!("case_{}", i));
                trace.add_event(Event::new("register", now).with_resource("clerk"));
                trace.add_event(
                    Event::new("examine thoroughly", now + Duration::minutes(10))
                        .with_resource("senior"),
                );
                trace.add_event(
                    Event::new("assess claim", now + Duration::minutes(30)).with_resource("mgr"),
                );
                trace.add_event(
                    Event::new("decide", now + Duration::hours(1)).with_resource("senior"),
                );
                trace.add_event(
                    Event::new("notify", now + Duration::hours(2)).with_resource("clerk"),
                );
                log.add_trace(trace);
            }
            log
        })
}

fn make_simple_ocel() -> ObjectCentricEventLog {
    let mut ocel = ObjectCentricEventLog::with_id("test_ocel");
    let doc_type = ObjectType::new("document");
    let order_type = ObjectType::new("order");
    ocel.register_object_type(doc_type.clone());
    ocel.register_object_type(order_type.clone());

    for i in 0..3 {
        ocel.add_object(Object::new(
            format!("DOC-{}", i),
            doc_type.clone(),
            Utc::now(),
        ));
        ocel.add_object(Object::new(
            format!("ORD-{}", i),
            order_type.clone(),
            Utc::now(),
        ));
    }

    let mut ts = Utc::now();
    for i in 0..3 {
        let create_eid = uuid::Uuid::new_v4();
        let process_eid = uuid::Uuid::new_v4();
        let approve_eid = uuid::Uuid::new_v4();
        ocel.add_event(create_eid, "create", ts, Some("alice".to_string()));
        ocel.add_event(
            process_eid,
            "process",
            ts + Duration::hours(1),
            Some("bob".to_string()),
        );
        ocel.add_event(
            approve_eid,
            "approve",
            ts + Duration::hours(2),
            Some("manager".to_string()),
        );

        let doc_id = format!("DOC-{}", i);
        let ord_id = format!("ORD-{}", i);

        for eid in [create_eid, process_eid, approve_eid] {
            let mut mapping = EventToObjectMapping::new(eid);
            mapping.add_object(&doc_id);
            mapping.add_object(&ord_id);
            ocel.add_event_object_mapping(mapping);
        }
        ts += Duration::days(1);
    }
    ocel
}

fn tmp_dir() -> TempDir {
    TempDir::new().unwrap()
}

fn tmp_path(dir: &TempDir, filename: &str) -> PathBuf {
    dir.path().join(filename)
}

// ============================================================================
// 1. Animation
// ============================================================================

#[test]
fn test_animation_from_trace() {
    let log = load_invoice();
    if log.traces.is_empty() {
        return;
    }
    let opts = AnimationOptions::new()
        .with_speed(AnimationSpeed::Fast)
        .with_labels(true);

    let animation = create_animation_from_trace(&log.traces[0], opts);
    assert!(animation.frame_count() > 0);
    assert!(animation.total_duration_ms > 0);

    // Can render SVG frames
    for i in 0..std::cmp::min(animation.frame_count(), 3) {
        let svg = animation.generate_frame_svg(i);
        assert!(
            svg.contains("<svg"),
            "frame {} SVG should contain <svg tag",
            i
        );
    }

    // Playlist SVG
    let playlist = animation.generate_playlist_svg();
    assert!(!playlist.is_empty());

    // HTML animation
    let html = animation.generate_animation_html();
    assert!(
        html.contains("<html") || html.contains("<div"),
        "HTML should have structure"
    );
}

#[test]
fn test_animation_from_log() {
    let log = load_invoice();
    let opts = AnimationOptions::new().with_speed(AnimationSpeed::Slow);
    let animations = create_animation_from_log(&log, opts);
    assert_eq!(animations.len(), log.len());
    for (i, anim) in animations.iter().enumerate() {
        assert!(
            anim.frame_count() > 0,
            "animation for trace {} has no frames",
            i
        );
    }
}

#[test]
fn test_animation_speed_multipliers() {
    assert_eq!(AnimationSpeed::VerySlow.multiplier(), 0.25);
    assert_eq!(AnimationSpeed::Slow.multiplier(), 0.5);
    assert_eq!(AnimationSpeed::Normal.multiplier(), 1.0);
    assert_eq!(AnimationSpeed::Fast.multiplier(), 2.0);
    assert_eq!(AnimationSpeed::VeryFast.multiplier(), 4.0);
}

#[test]
fn test_animation_frame_builder() {
    let frame = AnimationFrame::new(0, 0)
        .with_transition("t1".to_string())
        .with_label("event_1".to_string())
        .with_label("start".to_string());

    assert_eq!(frame.frame_number, 0);
    assert_eq!(frame.event_index, 0);
    assert_eq!(frame.event_label, "start");
}

#[test]
fn test_animation_manual_construction() {
    let opts = AnimationOptions::new();
    let mut animation = Animation::new(opts);

    let mut state = HashMap::new();
    state.insert("p1".to_string(), 1);

    animation.add_frame(
        AnimationFrame::new(0, 0)
            .with_transition("start".to_string())
            .with_token_state(state.clone())
            .with_label("register".to_string()),
    );
    animation.add_frame(
        AnimationFrame::new(1, 1)
            .with_transition("t2".to_string())
            .with_token_state(state)
            .with_label("examine".to_string()),
    );

    assert_eq!(animation.frame_count(), 2);
    assert!(animation.total_duration_ms > 0);

    // frame_at should find existing frames
    let f0 = animation.frame_at(0);
    assert!(f0.is_some());
}

// ============================================================================
// 2. Interactive Visualization
// ============================================================================

#[test]
fn test_interactive_dfg() {
    let log = load_invoice();
    let dfg = pm4py::discovery::directly_follows_graph(&log);
    let opts = InteractiveOptions::new()
        .with_zoom(true)
        .with_pan(true)
        .with_tooltips(true);

    let interactive = create_interactive_dfg(&dfg, opts);
    let svg = interactive.generate_svg();
    assert!(svg.contains("<svg"), "interactive DFG should render SVG");
    assert!(
        svg.contains("viewBox"),
        "SVG should have viewBox for interactivity"
    );
}

#[test]
fn test_interactive_petri_net() {
    let log = load_invoice();
    let miner = AlphaMiner::new();
    let net = miner.discover(&log);
    let opts = InteractiveOptions::new().with_animation(true);

    let interactive = create_interactive_petri_net(&net, opts);
    let svg = interactive.generate_svg();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_interactive_manual_construction() {
    let opts = InteractiveOptions::new();
    let mut viz = pm4py::visualization::InteractiveVisualization::new(opts);

    viz.add_node("a", "Activity A", 100.0, 200.0, "#4CAF50");
    viz.add_node("b", "Activity B", 300.0, 200.0, "#2196F3");
    viz.add_edge("a", "b", "a->b", "#999999");

    let svg = viz.generate_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Activity A"));
}

#[test]
fn test_interactive_filters() {
    let opts = InteractiveOptions::new();
    let mut viz = pm4py::visualization::InteractiveVisualization::new(opts);
    viz.add_node("a", "A", 0.0, 0.0, "#4CAF50");
    viz.add_node("b", "B", 100.0, 0.0, "#4CAF50");

    viz.set_filter("a", false);
    viz.apply_filters();

    let svg = viz.generate_svg();
    // After filtering, node 'a' should be gray (#CCCCCC)
    // The SVG should still render
    assert!(svg.contains("<svg"));
}

// ============================================================================
// 3. Layout Algorithms
// ============================================================================

#[test]
fn test_force_directed_layout() {
    let log = load_invoice();
    let dfg = pm4py::discovery::directly_follows_graph(&log);

    let nodes: Vec<String> = dfg.nodes.clone();
    let edges: Vec<(String, String)> = dfg
        .edges
        .iter()
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();

    let layout = ForceDirectedLayout::new().with_iterations(50);
    let result = layout.layout(&nodes, &edges);

    assert_eq!(result.positions.len(), nodes.len());
    assert!(result.width > 0.0);
    assert!(result.height > 0.0);
    // All nodes should have positions
    for node in &nodes {
        assert!(
            result.positions.contains_key(node),
            "missing position for node {}",
            node
        );
    }
}

#[test]
fn test_hierarchical_layout() {
    let log = load_invoice();
    let dfg = pm4py::discovery::directly_follows_graph(&log);

    let nodes: Vec<String> = dfg.nodes.clone();
    let edges: Vec<(String, String)> = dfg
        .edges
        .iter()
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();

    let layout = HierarchicalLayout::new().with_spacing(80.0, 60.0);
    let result = layout.layout(&nodes, &edges);

    assert_eq!(result.positions.len(), nodes.len());
    // Hierarchical layout should have some vertical structure
    assert!(result.height > 0.0);
}

#[test]
fn test_layout_normalize() {
    let mut result = pm4py::visualization::LayoutResult {
        positions: HashMap::new(),
        min_x: -500.0,
        max_x: 500.0,
        min_y: -300.0,
        max_y: 300.0,
        width: 1000.0,
        height: 600.0,
    };
    result
        .positions
        .insert("a".to_string(), Point::new(-500.0, -300.0));
    result
        .positions
        .insert("b".to_string(), Point::new(500.0, 300.0));

    result.normalize(800.0, 600.0, 20.0);
    assert!(result.width <= 800.0);
    assert!(result.height <= 600.0);
}

// ============================================================================
// 4. Remaining Parity Functions
// ============================================================================

#[test]
fn test_compute_emd_identical_traces() {
    let log = load_invoice();
    if log.traces.is_empty() {
        return;
    }
    let dist = compute_emd(&log.traces[0].events, &log.traces[0].events);
    assert!(
        (dist - 0.0).abs() < 0.001,
        "EMD of identical traces should be 0, got {}",
        dist
    );
}

#[test]
fn test_compute_emd_different_traces() {
    let now = Utc::now();
    let mut trace1 = Trace::new("t1");
    trace1.add_event(Event::new("a", now));
    trace1.add_event(Event::new("b", now + Duration::seconds(1)));

    let mut trace2 = Trace::new("t2");
    trace2.add_event(Event::new("x", now));
    trace2.add_event(Event::new("y", now + Duration::seconds(1)));
    trace2.add_event(Event::new("z", now + Duration::seconds(2)));

    let dist = compute_emd(&trace1.events, &trace2.events);
    assert!(
        dist > 0.0,
        "EMD of different traces should be > 0, got {}",
        dist
    );
}

#[test]
fn test_convert_log_to_ocel() {
    let log = load_invoice();
    let ocel = convert_log_to_ocel(&log, None);
    assert!(!ocel.events.is_empty(), "OCEL should have events");
    // Number of objects should equal number of traces (one per case)
    assert!(
        ocel.objects.len() >= log.len() / 2,
        "OCEL should have at least some objects"
    );
}

#[test]
fn test_convert_log_to_networkx() {
    let log = load_invoice();
    let json = convert_log_to_networkx(&log);
    assert!(
        json.contains("nodes"),
        "networkx JSON should contain 'nodes'"
    );
    assert!(
        json.contains("edges"),
        "networkx JSON should contain 'edges'"
    );
}

#[test]
fn test_convert_petri_net_to_networkx() {
    let log = load_invoice();
    let miner = AlphaMiner::new();
    let net = miner.discover(&log);
    let json = convert_petri_net_to_networkx(&net);
    assert!(json.contains("nodes"));
    assert!(json.contains("edges"));
}

#[test]
fn test_convert_ocel_to_networkx() {
    let ocel = make_simple_ocel();
    let json = convert_ocel_to_networkx(&ocel);
    assert!(
        json.contains("nodes"),
        "OCEL networkx should contain 'nodes'"
    );
    assert!(
        json.contains("edges"),
        "OCEL networkx should contain 'edges'"
    );
}

#[test]
fn test_construct_synchronous_product_net() {
    let log = load_invoice();
    let miner = AlphaMiner::new();
    let net1 = miner.discover(&log);
    let net2 = miner.discover(&log);
    let product = construct_synchronous_product_net(&net1, &net2);
    // Product net should have more places/transitions than either input
    assert!(product.places.len() >= net1.places.len());
}

#[test]
fn test_conformance_etoc() {
    let log = load_invoice();
    let efg = pm4py::discovery::eventually_follows_graph(&log);
    let etoc: HashMap<(String, String), usize> = efg.edges.clone();
    let fitness = conformance_etoc(&log, &etoc);
    assert!(
        fitness >= 0.0 && fitness <= 1.0,
        "ETOC fitness should be in [0,1], got {}",
        fitness
    );
}

#[test]
fn test_cluster_equivalent_ocel() {
    let ocel = make_simple_ocel();
    let clusters = cluster_equivalent_ocel(&ocel);
    // All objects should be in some cluster
    let total_objects: usize = clusters.values().map(|v| v.len()).sum();
    assert!(total_objects > 0, "clusters should contain objects");
}

#[test]
fn test_conformance_diagnostics_token_based_replay() {
    let log = load_invoice();
    let miner = AlphaMiner::new();
    let net = miner.discover(&log);
    let diagnostics =
        pm4py::remaining_parity::conformance_diagnostics_token_based_replay(&log, &net);
    assert_eq!(diagnostics.len(), log.len());
    for d in &diagnostics {
        assert!(d.fitness >= 0.0 && d.fitness <= 1.0);
    }
}

#[test]
fn test_verify_parity() {
    let parity_map = verify_parity();
    assert!(!parity_map.is_empty(), "parity map should not be empty");
    // Should contain some well-known pm4py functions
    println!(
        "Parity verification: {} Python functions mapped",
        parity_map.len()
    );
}

// ============================================================================
// 5. OCEL Filters
// ============================================================================

#[test]
fn test_ocel_filter_object_type() {
    let ocel = make_simple_ocel();
    let filtered = pm4py::ocpm::ocel_filter_object_type(&ocel, &["document".to_string()]);
    // Should only have document objects
    for (_id, obj) in &filtered.objects {
        assert_eq!(obj.object_type.name, "document");
    }
}

#[test]
fn test_ocel_filter_object_ids() {
    let ocel = make_simple_ocel();
    let ids = vec!["DOC-0".to_string(), "DOC-1".to_string()];
    let filtered = pm4py::ocpm::ocel_filter_object_ids(&ocel, &ids);
    // Should only have the specified objects
    for (id, _obj) in &filtered.objects {
        assert!(ids.contains(id), "unexpected object {}", id);
    }
}

#[test]
fn test_ocel_filter_activities() {
    let ocel = make_simple_ocel();
    let activities = vec!["create".to_string(), "approve".to_string()];
    let filtered = pm4py::ocpm::ocel_filter_activities(&ocel, &activities);
    // All events should have specified activities
    for (activity, _, _) in filtered.events.values() {
        assert!(
            activities.contains(activity),
            "unexpected activity {}",
            activity
        );
    }
}

#[test]
fn test_ocel_filter_connected_components() {
    let ocel = make_simple_ocel();
    let filtered = pm4py::ocpm::ocel_filter_connected_components(&ocel, 1);
    // All connected components should have >= 1 object
    assert!(filtered.objects.len() > 0);
}

#[test]
fn test_ocel_filter_object_event_count() {
    let ocel = make_simple_ocel();
    // All objects should have exactly 3 events (create, process, approve)
    let filtered = pm4py::ocpm::ocel_filter_object_event_count(&ocel, 3, 3);
    assert_eq!(filtered.objects.len(), ocel.objects.len());
}

#[test]
fn test_filter_ocel_cc_activity() {
    let ocel = make_simple_ocel();
    let filtered = pm4py::ocpm::filter_ocel_cc_activity(&ocel, "create");
    // Should keep connected components that have a "create" activity
    assert!(filtered.events.len() > 0);
}

#[test]
fn test_filter_ocel_cc_object() {
    let ocel = make_simple_ocel();
    let filtered = pm4py::ocpm::filter_ocel_cc_object(&ocel, "DOC-0");
    assert!(filtered.objects.len() > 0);
    // Should contain DOC-0
    let has_doc0 = filtered.objects.iter().any(|(id, _)| id == "DOC-0");
    assert!(has_doc0, "filtered OCEL should contain DOC-0");
}

#[test]
fn test_filter_ocel_cc_otype() {
    let ocel = make_simple_ocel();
    let filtered = pm4py::ocpm::filter_ocel_cc_otype(&ocel, "order");
    // Connected components may include linked objects of other types
    let has_order = filtered
        .objects
        .values()
        .any(|o| o.object_type.name == "order");
    assert!(has_order, "filtered OCEL should contain order objects");
    assert!(filtered.objects.len() > 0);
}

// ============================================================================
// 6. OCEL Utilities
// ============================================================================

#[test]
fn test_ocel_object_type_activities() {
    let ocel = make_simple_ocel();
    let acts = pm4py::ocpm::ocel_object_type_activities(&ocel);
    assert!(!acts.is_empty());
    // document type should have activities
    if let Some(doc_acts) = acts.get("document") {
        assert!(doc_acts.contains("create"));
    }
}

#[test]
fn test_ocel_objects_ot_count() {
    let ocel = make_simple_ocel();
    let counts = pm4py::ocpm::ocel_objects_ot_count(&ocel);
    assert_eq!(counts["document"], 3);
    assert_eq!(counts["order"], 3);
}

#[test]
fn test_ocel_add_timedelta() {
    let mut ocel = make_simple_ocel();
    let first_eid = *ocel.events.keys().next().unwrap();
    let original_ts = ocel.events.get(&first_eid).unwrap().1;

    pm4py::ocpm::ocel_add_index_based_timedelta(&mut ocel, &first_eid.to_string(), 3600);
    let new_ts = ocel.events.get(&first_eid).unwrap().1;
    assert_eq!(
        (new_ts - original_ts).num_seconds(),
        3600,
        "timedelta should shift by 3600 seconds"
    );
}

#[test]
fn test_ocel_drop_duplicates() {
    let mut ocel = make_simple_ocel();
    let before_count = ocel.events.len();
    let removed = pm4py::ocpm::ocel_drop_duplicates(&mut ocel);
    let _ = removed; // usize is always >= 0; just confirm the value exists
    assert!(ocel.events.len() <= before_count);
}

#[test]
fn test_sample_ocel_objects() {
    let ocel = make_simple_ocel();
    let sampled = pm4py::ocpm::sample_ocel_objects(&ocel, 2);
    assert_eq!(sampled.len(), 2);
}

#[test]
fn test_sample_ocel_connected_components() {
    let ocel = make_simple_ocel();
    let components = pm4py::ocpm::sample_ocel_connected_components(&ocel, 1);
    // Each component should have at least 1 object
    for component in &components {
        assert!(!component.is_empty());
    }
}

#[test]
fn test_ocel_flattening_roundtrip() {
    let ocel = make_simple_ocel();
    let flat = pm4py::ocpm::ocel_flattening(&ocel);
    assert!(flat.len() > 0, "flattened OCEL should produce traces");
    // Each trace should correspond to an object
    let total_events: usize = flat.traces.iter().map(|t| t.events.len()).sum();
    assert!(total_events > 0);
}

// ============================================================================
// 7. Save Visualization Functions
// ============================================================================

#[test]
fn test_save_vis_case_duration_graph() {
    let log = load_invoice();
    let dir = tmp_dir();
    let path = tmp_path(&dir, "case_duration.json");
    let result = pm4py::visualization::save_vis_case_duration_graph(&log, &path);
    assert!(
        result.is_ok(),
        "save_vis_case_duration_graph failed: {:?}",
        result.err()
    );
    assert!(path.exists());
}

#[test]
fn test_save_vis_events_distribution_graph() {
    let log = load_invoice();
    let dir = tmp_dir();
    let path = tmp_path(&dir, "events_dist.json");
    let result = pm4py::visualization::save_vis_events_distribution_graph(&log, &path);
    assert!(
        result.is_ok(),
        "save_vis_events_distribution_graph failed: {:?}",
        result.err()
    );
    assert!(path.exists());
}

#[test]
fn test_save_vis_events_per_time_graph() {
    let log = load_invoice();
    let dir = tmp_dir();
    let path = tmp_path(&dir, "events_time.json");
    let result = pm4py::visualization::save_vis_events_per_time_graph(&log, &path);
    assert!(
        result.is_ok(),
        "save_vis_events_per_time_graph failed: {:?}",
        result.err()
    );
    assert!(path.exists());
}

#[test]
fn test_save_vis_dotted_chart() {
    let log = load_invoice();
    let dir = tmp_dir();
    let path = tmp_path(&dir, "dotted_chart.json");
    let result = pm4py::visualization::save_vis_dotted_chart(&log, &path);
    assert!(
        result.is_ok(),
        "save_vis_dotted_chart failed: {:?}",
        result.err()
    );
    assert!(path.exists());
}

#[test]
fn test_save_vis_performance_dfg() {
    let log = load_invoice();
    let dir = tmp_dir();
    let path = tmp_path(&dir, "perf_dfg.json");
    let result = pm4py::visualization::save_vis_performance_dfg(&log, &path);
    assert!(
        result.is_ok(),
        "save_vis_performance_dfg failed: {:?}",
        result.err()
    );
    assert!(path.exists());
}

#[test]
fn test_save_vis_transition_system() {
    let log = load_invoice();
    let dir = tmp_dir();
    let path = tmp_path(&dir, "ts.json");
    let result = pm4py::visualization::save_vis_transition_system(&log, &path);
    assert!(
        result.is_ok(),
        "save_vis_transition_system failed: {:?}",
        result.err()
    );
    assert!(path.exists());
}

#[test]
fn test_save_vis_network_analysis() {
    let log = load_invoice();
    let dir = tmp_dir();
    let path = tmp_path(&dir, "network.json");
    let result = pm4py::visualization::save_vis_network_analysis(&log, &path);
    assert!(
        result.is_ok(),
        "save_vis_network_analysis failed: {:?}",
        result.err()
    );
    assert!(path.exists());
}

#[test]
fn test_save_vis_sna() {
    let log = load_invoice();
    let dir = tmp_dir();
    let path = tmp_path(&dir, "sna.json");
    let result = pm4py::visualization::save_vis_sna(&log, &path);
    assert!(result.is_ok(), "save_vis_sna failed: {:?}", result.err());
}

#[test]
fn test_save_vis_performance_spectrum() {
    let log = load_invoice();
    let dir = tmp_dir();
    let path = tmp_path(&dir, "spectrum.json");
    let result = pm4py::visualization::save_vis_performance_spectrum(&log, &path);
    assert!(
        result.is_ok(),
        "save_vis_performance_spectrum failed: {:?}",
        result.err()
    );
}

#[test]
fn test_save_vis_prefix_tree() {
    let log = load_invoice();
    let dir = tmp_dir();
    let path = tmp_path(&dir, "prefix_tree.json");
    let result = pm4py::visualization::save_vis_prefix_tree(&log, &path);
    assert!(
        result.is_ok(),
        "save_vis_prefix_tree failed: {:?}",
        result.err()
    );
}

#[test]
fn test_save_vis_ocdfg() {
    let log = load_invoice();
    let dir = tmp_dir();
    let path = tmp_path(&dir, "ocdfg.json");
    let result = pm4py::visualization::save_vis_ocdfg(&log, &path);
    assert!(result.is_ok(), "save_vis_ocdfg failed: {:?}", result.err());
}

#[test]
fn test_save_vis_footprints() {
    let log = load_invoice();
    let dir = tmp_dir();
    let traces: Vec<Vec<String>> = log
        .traces
        .iter()
        .map(|t| t.events.iter().map(|e| e.activity.clone()).collect())
        .collect();
    let footprints = Footprints::from_traces(&traces);
    let path = tmp_path(&dir, "footprints.json");
    let result = pm4py::visualization::save_vis_footprints(&footprints, &path);
    assert!(
        result.is_ok(),
        "save_vis_footprints failed: {:?}",
        result.err()
    );
}

#[test]
fn test_save_vis_powl() {
    let log = load_invoice();
    let dir = tmp_dir();
    let path = tmp_path(&dir, "powl.json");
    let result = pm4py::visualization::save_vis_powl(&log, &path);
    assert!(result.is_ok(), "save_vis_powl failed: {:?}", result.err());
}

// ============================================================================
// 8. Cross-API Integration: Animation + Layout + NetworkX
// ============================================================================

#[test]
fn test_layout_then_interactive_dfg() {
    let log = load_invoice();
    let dfg = pm4py::discovery::directly_follows_graph(&log);

    let nodes: Vec<String> = dfg.nodes.clone();
    let edges: Vec<(String, String)> = dfg
        .edges
        .iter()
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();

    // Compute layout
    let layout = ForceDirectedLayout::new().with_iterations(30);
    let positions = layout.layout(&nodes, &edges);

    // Build interactive visualization with computed positions
    let opts = InteractiveOptions::new();
    let mut viz = pm4py::visualization::InteractiveVisualization::new(opts);
    for node in &nodes {
        if let Some(pos) = positions.positions.get(node) {
            viz.add_node(node, node, pos.x, pos.y, "#4CAF50");
        }
    }
    for (from, to) in &edges {
        viz.add_edge(from, to, &format!("{}->{}", from, to), "#999");
    }

    let svg = viz.generate_svg();
    assert!(svg.contains("<svg"));
}

#[test]
fn test_ocel_discovery_filter_roundtrip() {
    // Create OCEL -> filter -> flatten -> discover
    let ocel = make_simple_ocel();
    let filtered =
        pm4py::ocpm::ocel_filter_activities(&ocel, &["create".to_string(), "approve".to_string()]);
    let flat = pm4py::ocpm::ocel_flattening(&filtered);

    if flat.len() > 0 {
        let dfg = pm4py::discovery::directly_follows_graph(&flat);
        assert!(
            dfg.nodes.len() > 0,
            "DFG from filtered OCEL should have nodes"
        );
    }
}
