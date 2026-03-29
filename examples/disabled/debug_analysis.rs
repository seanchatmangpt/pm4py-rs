use pm4py::discovery::AlphaMiner;
use pm4py::io::XESReader;
use std::path::Path;

fn main() {
    let log = XESReader::new()
        .read(Path::new("/Users/sac/chatmangpt/test_simple.xes"))
        .unwrap();
    let net = AlphaMiner::new().discover(&log);

    let soundness = pm4py::models::petri_net_analysis::PetriNetAnalyzer::check_soundness(&net);
    println!("is_sound: {}", soundness.is_sound);
    println!("option_to_complete: {}", soundness.option_to_complete);
    println!("proper_completion: {}", soundness.proper_completion);
    println!("no_dead_transitions: {}", soundness.no_dead_transitions);
}
