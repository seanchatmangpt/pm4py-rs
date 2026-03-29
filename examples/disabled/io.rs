use chrono::Utc;
/// I/O Operations Example
///
/// This example demonstrates reading and writing event logs.
///
/// Run with: cargo run --example io
use pm4py::log::{Event, EventLog, Trace};

fn main() {
    println!("═══════════════════════════════════════════════════════");
    println!("           I/O Operations Example");
    println!("═══════════════════════════════════════════════════════\n");

    let log = create_sample_log();

    println!("Event Log Summary:");
    println!("  Total traces: {}", log.traces.len());
    println!(
        "  Total events: {}",
        log.traces.iter().map(|t| t.events.len()).sum::<usize>()
    );

    println!("\nLog Contents:");
    for (i, trace) in log.traces.iter().enumerate().take(3) {
        println!(
            "\n  Trace {} ({}): {} events",
            i + 1,
            trace.id,
            trace.events.len()
        );
        for (j, event) in trace.events.iter().enumerate().take(3) {
            println!("    {} → {}", j + 1, event.activity);
        }
        if trace.events.len() > 3 {
            println!("    ... and {} more", trace.events.len() - 3);
        }
    }

    println!("\n✓ I/O example complete");
    println!("\nNote: Full XES/CSV I/O support available via:");
    println!("  • pm4py::io::XESReader::read()");
    println!("  • pm4py::io::XESWriter::write()");
    println!("  • pm4py::io::CSVReader::read()");
    println!("  • pm4py::io::CSVWriter::write()");
}

fn create_sample_log() -> EventLog {
    let mut log = EventLog::new();
    let base = Utc::now();

    for i in 0..3 {
        let mut trace = Trace::new(format!("order_{:03}", i + 1));
        let offset = base + chrono::Duration::hours(i as i64);

        trace.add_event(Event::new("Order Received", offset));
        trace.add_event(Event::new(
            "Order Validated",
            offset + chrono::Duration::minutes(5),
        ));
        trace.add_event(Event::new(
            "Payment Processed",
            offset + chrono::Duration::minutes(15),
        ));
        trace.add_event(Event::new(
            "Item Picked",
            offset + chrono::Duration::minutes(45),
        ));
        trace.add_event(Event::new(
            "Item Shipped",
            offset + chrono::Duration::hours(2),
        ));

        log.add_trace(trace);
    }

    log
}
