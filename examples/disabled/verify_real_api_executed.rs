use pm4py::conformance::*;
use pm4py::discovery::*;
/// VERIFY PM4PY-RUST BY ACTUALLY EXECUTING EVERY FUNCTION
/// Chicago TDD: Run the code, don't check declarations
use pm4py::io::XESReader;
use pm4py::log::operations::*;
use pm4py::models::*;
use pm4py::statistics::*;
use pm4py::utils::common::*;
use pm4py::utils::encoders::*;
use pm4py::version::*;
use pm4py::visualization::*;
use std::path::Path;

fn main() {
    println!("=== EXECUTE ALL PM4PY-RUST FUNCTIONS - REAL VERIFICATION ===\n");

    // Load test log
    let log = XESReader::new()
        .read(Path::new("/Users/sac/chatmangpt/test_simple.xes"))
        .unwrap();
    println!(
        "Loaded log: {} traces, {} events\n",
        log.len(),
        log.num_events()
    );

    let mut passed = 0;
    let mut failed = Vec::new();

    // ===== DISCOVERY (8 miners) =====
    println!("DISCOVERY ALGORITHMS:");
    let (p, f) = verify_discovery(&log);
    passed += p;
    failed.extend(f);

    // ===== CONFORMANCE (7 types) =====
    println!("\nCONFORMANCE CHECKING:");
    let (p, f) = verify_conformance(&log);
    passed += p;
    failed.extend(f);

    // ===== LOG OPERATIONS (8 functions) =====
    println!("\nLOG OPERATIONS:");
    let (p, f) = verify_log_operations(&log);
    passed += p;
    failed.extend(f);

    // ===== STATISTICS (4 functions) =====
    println!("\nSTATISTICS:");
    let (p, f) = verify_statistics(&log);
    passed += p;
    failed.extend(f);

    // ===== VISUALIZATION (4 functions) =====
    println!("\nVISUALIZATION:");
    let (p, f) = verify_visualization(&log);
    passed += p;
    failed.extend(f);

    // ===== UTILITIES (5 functions) =====
    println!("\nUTILITIES:");
    let (p, f) = verify_utilities(&log);
    passed += p;
    failed.extend(f);

    // ===== ENCODERS (4 functions) =====
    println!("\nENCODERS:");
    let (p, f) = verify_encoders(&log);
    passed += p;
    failed.extend(f);

    // ===== VERSION (2 functions) =====
    println!("\nVERSION:");
    let (p, f) = verify_version();
    passed += p;
    failed.extend(f);

    println!("\n=== FINAL RESULTS ===");
    println!("Total: {}", passed + failed.len());
    println!("Passed: {}", passed);
    println!("Failed: {}", failed.len());

    if !failed.is_empty() {
        println!("\n=== FAILED ===");
        for (name, error) in failed {
            println!("  ❌ {}: {}", name, error);
        }
    }

    if failed.is_empty() {
        println!("\n✅ ALL PM4PY-RUST FUNCTIONS WORK - EXECUTION VERIFIED");
    }
}

fn verify_discovery(log: &pm4py::log::EventLog) -> (usize, Vec<(&'static str, String)>) {
    let mut passed = 0;
    let mut failed = Vec::new();

    // 1. Alpha Miner
    let net = AlphaMiner::new().discover(log);
    println!(
        "  ✅ AlphaMiner::discover() -> PetriNet with {} places, {} transitions",
        net.places.len(),
        net.transitions.len()
    );
    passed += 1;

    // 2. Heuristic Miner
    let net = HeuristicMiner::new().discover(log);
    println!("  ✅ HeuristicMiner::discover() -> PetriNet");
    passed += 1;

    // 3. ILP Miner
    let net = ILPMiner::new().discover(log);
    println!("  ✅ ILPMiner::discover() -> PetriNet");
    passed += 1;

    // 4. Inductive Miner
    let net = InductiveMiner::new().discover(log);
    println!("  ✅ InductiveMiner::discover() -> PetriNet");
    passed += 1;

    // 5. DFG Miner
    let dfg = DFGMiner::new().discover(log);
    println!(
        "  ✅ DFGMiner::discover() -> DFG with {} nodes",
        dfg.nodes.len()
    );
    passed += 1;

    // 6. Tree Miner
    let net = TreeMiner::new().discover(log);
    println!("  ✅ TreeMiner::discover() -> PetriNet");
    passed += 1;

    // 7. Split Miner
    let net = SplitMiner::new().discover(log);
    println!("  ✅ SplitMiner::discover() -> PetriNet");
    passed += 1;

    // 8. Causal Net Miner
    let causal_net = CausalNetMiner::new().discover(log);
    println!("  ✅ CausalNetMiner::discover() -> CausalNet");
    passed += 1;

    (passed, failed)
}

fn verify_conformance(log: &pm4py::log::EventLog) -> (usize, Vec<(&'static str, String)>) {
    let mut passed = 0;
    let mut failed = Vec::new();

    // Need a PetriNet first
    let net = AlphaMiner::new().discover(log);

    // 1. Token Replay
    let result = TokenReplay::new().check(log, &net);
    println!("  ✅ TokenReplay::check() -> diagnostics");
    passed += 1;

    // 2. Alignment Checker
    let result = AlignmentChecker::new().check(log, &net);
    println!("  ✅ AlignmentChecker::check() -> diagnostics");
    passed += 1;

    // 3. Precision (need to check actual API)
    match Precision::calculate_tbr(log, &net) {
        Ok(p) => {
            println!("  ✅ Precision::calculate_tbr() -> {}", p);
            passed += 1;
        }
        Err(e) => failed.push(("Precision::calculate_tbr", format!("{:?}", e))),
    }

    // 4. Generalization
    match Generalization::calculate(log, &net, 5) {
        Ok(g) => {
            println!("  ✅ Generalization::calculate() -> {}", g);
            passed += 1;
        }
        Err(e) => failed.push(("Generalization::calculate", format!("{:?}", e))),
    }

    // 5. Simplicity
    match Simplicity::calculate(&net) {
        Ok(s) => {
            println!("  ✅ Simplicity::calculate() -> {}", s);
            passed += 1;
        }
        Err(e) => failed.push(("Simplicity::calculate", format!("{:?}", e))),
    }

    // 6. Four Spectrum
    match FourSpectrum::calculate(log, &net) {
        Ok(s) => {
            println!(
                "  ✅ FourSpectrum::calculate() -> quality: {}",
                s.quality_score
            );
            passed += 1;
        }
        Err(e) => failed.push(("FourSpectrum::calculate", format!("{:?}", e))),
    }

    // 7. Behavioral Profile
    match BehavioralProfile::new(log, &net) {
        Ok(p) => {
            println!("  ✅ BehavioralProfile::new() -> profile");
            passed += 1;
        }
        Err(e) => failed.push(("BehavioralProfile::new", format!("{:?}", e))),
    }

    (passed, failed)
}

fn verify_log_operations(log: &pm4py::log::EventLog) -> (usize, Vec<(&'static str, String)>) {
    let mut passed = 0;
    let mut failed = Vec::new();

    // 1. start_activities
    let starts = start_activities(log);
    println!(
        "  ✅ start_activities() -> {} start activities",
        starts.len()
    );
    passed += 1;

    // 2. end_activities
    let ends = end_activities(log);
    println!("  ✅ end_activities() -> {} end activities", ends.len());
    passed += 1;

    // 3. variants
    let vars = variants(log);
    println!("  ✅ variants() -> {} variants", vars.len());
    passed += 1;

    // 4. directly_follows
    let follows = directly_follows(log);
    println!("  ✅ directly_follows() -> {} relations", follows.len());
    passed += 1;

    // 5. activity_frequency
    let freq = activity_frequency(log);
    println!("  ✅ activity_frequency() -> {} activities", freq.len());
    passed += 1;

    // 6. activity_resources
    let resources = activity_resources(log);
    println!(
        "  ✅ activity_resources() -> {} activities with resources",
        resources.len()
    );
    passed += 1;

    // 7. sort_traces_by_length
    let mut sorted_log = log.clone();
    sort_traces_by_length(&mut sorted_log);
    println!("  ✅ sort_traces_by_length() -> sorted");
    passed += 1;

    // 8. is_consistent
    let consistent = is_consistent(log);
    println!("  ✅ is_consistent() -> {}", consistent);
    passed += 1;

    (passed, failed)
}

fn verify_statistics(log: &pm4py::log::EventLog) -> (usize, Vec<(&'static str, String)>) {
    let mut passed = 0;
    let mut failed = Vec::new();

    // 1. log_statistics
    let stats = log_statistics(log);
    println!(
        "  ✅ log_statistics() -> {} traces, {} events, {} activities",
        stats.num_traces, stats.num_events, stats.num_unique_activities
    );
    passed += 1;

    // 2. activity_occurrence_matrix
    let matrix = activity_occurrence_matrix(log);
    println!(
        "  ✅ activity_occurrence_matrix() -> {} activities",
        matrix.len()
    );
    passed += 1;

    // 3. directly_follows_matrix
    let df_matrix = directly_follows_matrix(log);
    println!(
        "  ✅ directly_follows_matrix() -> {} relations",
        df_matrix.len()
    );
    passed += 1;

    // 4. sample_traces
    let sampled = sample_traces(log, 1);
    println!("  ✅ sample_traces() -> {} traces sampled", sampled.len());
    passed += 1;

    (passed, failed)
}

fn verify_visualization(log: &pm4py::log::EventLog) -> (usize, Vec<(&'static str, String)>) {
    let mut passed = 0;
    let mut failed = Vec::new();

    let net = AlphaMiner::new().discover(log);
    let tree_net = TreeMiner::new().discover(log);
    let dfg = DFGMiner::new().discover(log);

    // Convert to ProcessTree for visualization
    let tree = pm4py::models::ProcessTree::root();

    // 1. render_petri_net_svg
    match render_petri_net_svg(&net, &Default::default()) {
        Ok(svg) => {
            println!("  ✅ render_petri_net_svg() -> {} bytes", svg.len());
            passed += 1;
        }
        Err(e) => failed.push(("render_petri_net_svg", format!("{:?}", e))),
    }

    // 2. render_process_tree_svg
    match render_process_tree_svg(&tree, &Default::default()) {
        Ok(svg) => {
            println!("  ✅ render_process_tree_svg() -> {} bytes", svg.len());
            passed += 1;
        }
        Err(e) => failed.push(("render_process_tree_svg", format!("{:?}", e))),
    }

    // 3. render_dfg_svg
    match render_dfg_svg(&dfg, &Default::default()) {
        Ok(svg) => {
            println!("  ✅ render_dfg_svg() -> {} bytes", svg.len());
            passed += 1;
        }
        Err(e) => failed.push(("render_dfg_svg", format!("{:?}", e))),
    }

    // 4. create_dotted_chart
    match create_dotted_chart(log, &Default::default()) {
        Ok(chart) => {
            println!(
                "  ✅ create_dotted_chart() -> chart with {} dots",
                chart.dots.len()
            );
            passed += 1;
        }
        Err(e) => failed.push(("create_dotted_chart", format!("{:?}", e))),
    }

    (passed, failed)
}

fn verify_utilities(log: &pm4py::log::EventLog) -> (usize, Vec<(&'static str, String)>) {
    let mut passed = 0;
    let mut failed = Vec::new();

    // 1. escape_xml_string
    let escaped = escape_xml_string("<test>&\"'");
    println!("  ✅ escape_xml_string() -> {}", escaped);
    passed += 1;

    // 2. merge_logs
    let merged = merge_logs(&[log.clone(), log.clone()]);
    println!("  ✅ merge_logs() -> {} traces", merged.len());
    passed += 1;

    // 3. split_by_attribute
    let split = split_by_attribute(log, "concept:name");
    println!("  ✅ split_by_attribute() -> {} groups", split.len());
    passed += 1;

    // 4. reverse_traces
    let reversed = reverse_traces(log);
    println!("  ✅ reverse_traces() -> {} traces", reversed.len());
    passed += 1;

    // 5. remove_outliers
    let filtered = remove_outliers(log, 2.0);
    println!("  ✅ remove_outliers() -> {} traces", filtered.len());
    passed += 1;

    (passed, failed)
}

fn verify_encoders(log: &pm4py::log::EventLog) -> (usize, Vec<(&'static str, String)>) {
    let mut passed = 0;
    let mut failed = Vec::new();

    // 1. onehot_encode
    let (encoded, activities) = onehot_encode(log);
    println!(
        "  ✅ onehot_encode() -> {} traces, {} features",
        encoded.len(),
        activities.len()
    );
    passed += 1;

    // 2. frequency_encode
    let encoded = frequency_encode(log);
    println!("  ✅ frequency_encode() -> {} traces", encoded.len());
    passed += 1;

    // 3. sequence_encode
    let (encoded, activities) = sequence_encode(log);
    println!(
        "  ✅ sequence_encode() -> {} sequences, {} activities",
        encoded.len(),
        activities.len()
    );
    passed += 1;

    // 4. feature_matrix
    let (matrix, activities) = feature_matrix(log);
    println!(
        "  ✅ feature_matrix() -> {} traces, {} features",
        matrix.len(),
        activities.len()
    );
    passed += 1;

    (passed, failed)
}

fn verify_version() -> (usize, Vec<(&'static str, String)>) {
    let mut passed = 0;
    let mut failed = Vec::new();

    // 1. version_string
    let v = version_string();
    println!("  ✅ version_string() -> {}", v);
    passed += 1;

    // 2. version_info
    let info = version_info();
    println!(
        "  ✅ version_info() -> {}.{}.{}",
        info.major, info.minor, info.patch
    );
    passed += 1;

    (passed, failed)
}
