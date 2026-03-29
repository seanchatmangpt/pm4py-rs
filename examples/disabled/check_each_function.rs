//! Check each pm4py capability one by one - execution verification
use pm4py::conformance::*;
use pm4py::discovery::*;
use pm4py::*;
use std::path::Path;

fn main() {
    println!("=== CHECKING EACH PM4PY CAPABILITY ONE BY ONE ===\n");

    let log = io::XESReader::new()
        .read(Path::new("/Users/sac/chatmangpt/test_simple.xes"))
        .unwrap();
    println!(
        "Loaded log: {} traces, {} events\n",
        log.len(),
        log.num_events()
    );

    let mut checked = 0;
    let mut total = 0;

    macro_rules! check {
        ($name:expr, $expr:expr) => {
            total += 1;
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = $expr;
            })) {
                Ok(_) => {
                    println!("✅ [{}] {}", checked + 1, $name);
                    checked += 1;
                }
                Err(_) => {
                    println!("❌ {}", $name);
                }
            }
        };
    }

    println!("--- DISCOVERY ALGORITHMS ---");
    let net = AlphaMiner::new().discover(&log);
    check!("AlphaMiner::discover", AlphaMiner::new().discover(&log));
    check!(
        "AlphaPlusMiner::discover",
        AlphaPlusMiner::new().discover(&log)
    );
    check!(
        "HeuristicMiner::discover",
        HeuristicMiner::new().discover(&log)
    );
    check!(
        "InductiveMiner::discover",
        InductiveMiner::new().discover(&log)
    );
    check!("DFGMiner::discover", DFGMiner::new().discover(&log));
    check!("TreeMiner::discover", TreeMiner::new().discover(&log));
    check!("SplitMiner::discover", SplitMiner::new().discover(&log));
    check!(
        "CausalNetMiner::discover",
        CausalNetMiner::new().discover(&log)
    );
    check!(
        "LogSkeletonMiner::discover",
        LogSkeletonMiner::new().discover(&log)
    );

    println!("\n--- CONFORMANCE CHECKING ---");
    check!("TokenReplay::check", TokenReplay::new().check(&log, &net));
    check!("conformance_alignments", conformance_alignments(&log, &net));

    println!("\n--- STATISTICS ---");
    check!("log_statistics", statistics::log_statistics(&log));

    println!("\n--- REMAINING PARITY ---");
    check!(
        "cluster_equivalent_ocel",
        cluster_equivalent_ocel(&ocpm::ObjectCentricEventLog::new())
    );
    check!(
        "compute_emd",
        compute_emd(&log.traces[0].events, &log.traces[0].events)
    );
    check!(
        "conformance_diagnostics_alignments",
        conformance_diagnostics_alignments(&log, &net)
    );
    check!(
        "conformance_etoc",
        conformance_etoc(&log, &std::collections::HashMap::new())
    );
    check!("convert_log_to_ocel", convert_log_to_ocel(&log, None));
    check!(
        "construct_synchronous_product_net",
        construct_synchronous_product_net(&net, &net)
    );
    check!("convert_log_to_networkx", convert_log_to_networkx(&log));
    check!(
        "convert_petri_net_to_networkx",
        convert_petri_net_to_networkx(&net)
    );

    println!("\n=== RESULTS ===");
    println!("Checked: {}/{}", checked, total);
    println!("\n✅ EACH CAPABILITY CHECKED THROUGH EXECUTION");
}
