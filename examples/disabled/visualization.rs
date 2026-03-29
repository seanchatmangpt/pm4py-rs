use chrono::Utc;
use pm4py::discovery::AlphaMiner;
/// Visualization Example
///
/// This example demonstrates creating visual process models.
///
/// Run with: cargo run --example visualization
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("═══════════════════════════════════════════════════════");
    println!("         Process Model Visualization");
    println!("═══════════════════════════════════════════════════════\n");

    let log = create_sample_log();

    println!("Discovering process model...");
    let miner = AlphaMiner::new();
    let petri_net = miner.discover(&log);

    println!("✓ Discovered Petri Net");
    println!("  Places: {}", petri_net.places.len());
    println!("  Transitions: {}", petri_net.transitions.len());

    println!("\nPetri Net Components:");
    println!("  Places (●):");
    for (i, place) in petri_net.places.iter().enumerate().take(5) {
        println!("    {} → {}", i + 1, place.name);
    }

    println!("\n  Transitions (□):");
    for (i, transition) in petri_net.transitions.iter().enumerate().take(5) {
        println!("    {} → {}", i + 1, transition.name);
    }

    println!("\nVisualization Options:");
    println!("  • SVG export (scalable vector graphics)");
    println!("  • PNG export (raster image)");
    println!("  • DOT format (Graphviz)");

    println!("\n✓ Visualization example complete");
}

fn create_sample_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    for i in 0..5 {
        let mut trace = Trace::new(format!("case_{:03}", i + 1));
        let offset = base + chrono::Duration::hours(i as i64);

        trace.add_event(Event::new("Start", offset));
        trace.add_event(Event::new("Check", offset + chrono::Duration::minutes(5)));
        trace.add_event(Event::new("Review", offset + chrono::Duration::minutes(15)));
        trace.add_event(Event::new(
            "Approve",
            offset + chrono::Duration::minutes(25),
        ));
        trace.add_event(Event::new(
            "Execute",
            offset + chrono::Duration::minutes(35),
        ));
        trace.add_event(Event::new("End", offset + chrono::Duration::minutes(45)));

        log.add_trace(trace);
    }

    // Alternative path
    for i in 0..2 {
        let mut trace = Trace::new(format!("case_{:03}", i + 6));
        let offset = base + chrono::Duration::hours((i + 5) as i64);

        trace.add_event(Event::new("Start", offset));
        trace.add_event(Event::new("Check", offset + chrono::Duration::minutes(5)));
        trace.add_event(Event::new("Review", offset + chrono::Duration::minutes(15)));
        trace.add_event(Event::new("Reject", offset + chrono::Duration::minutes(25)));
        trace.add_event(Event::new("End", offset + chrono::Duration::minutes(35)));

        log.add_trace(trace);
    }

    log
}
