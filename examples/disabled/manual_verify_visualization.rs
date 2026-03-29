use pm4py::discovery::AlphaMiner;
/// Manual verification of pm4py-rust visualization
use pm4py::io::XESReader;
use pm4py::visualization::{dotted_chart::DottedChart, svg_renderer::SvgRenderer};
use std::path::Path;

fn main() {
    println!("=== MANUAL VERIFICATION OF PM4PY-RUST VISUALIZATION ===\n");

    // Load the log
    let path = Path::new("/Users/sac/chatmangpt/test_simple.xes");
    let reader = XESReader::new();
    let log = reader.read(path).expect("Failed to load XES");

    // Discover a model
    let miner = AlphaMiner::new();
    let net = miner.discover(&log);

    // 1. PETRI NET VISUALIZATION - MANUAL VERIFICATION
    println!("1. PETRI NET SVG RENDERING");
    let renderer = SvgRenderer::new();
    let svg_result = renderer.render_petri_net(&net);

    match &svg_result {
        Ok(svg) => {
            println!("  - SVG generated: {} bytes", svg.len());
            println!("  - Contains <?xml: {:?}", svg.contains("<?xml"));
            println!("  - Contains <svg: {:?}", svg.contains("<svg"));
            println!("  - Contains <circle: {:?}", svg.contains("<circle"));
            println!("  - Contains <text: {:?}", svg.contains("<text"));

            if svg.len() > 100 && svg.contains("<svg") {
                println!("  ✓ PETRI NET VISUALIZATION WORKS\n");
            } else {
                println!("  ✗ PETRI NET VISUALIZATION FAILED\n");
            }
        }
        Err(e) => {
            println!("  ✗ PETRI NET VISUALIZATION FAILED: {:?}\n", e);
        }
    }

    // 2. DOTTED CHART VISUALIZATION - MANUAL VERIFICATION
    println!("2. DOTTED CHART SVG RENDERING");
    let dotted_chart = DottedChart::new();
    let chart_result = dotted_chart.generate(&log);

    match &chart_result {
        Ok(chart) => {
            println!("  - Chart generated: {} dots", chart.dots.len());
            println!("  - SVG available: {}", chart.svg.is_some());

            if chart.dots.len() == 15 {
                println!("  ✓ DOTTED CHART WORKS\n");
            } else {
                println!(
                    "  ✗ DOTTED CHART FAILED: Expected 15 dots, got {}\n",
                    chart.dots.len()
                );
            }
        }
        Err(e) => {
            println!("  ✗ DOTTED CHART FAILED: {:?}\n", e);
        }
    }

    // 3. PROCESS TREE VISUALIZATION - MANUAL VERIFICATION
    println!("3. PROCESS TREE VISUALIZATION");
    use pm4py::discovery::TreeMiner;
    let tree_miner = TreeMiner::new();
    let tree = tree_miner.discover(&log);

    let tree_svg_result = renderer.render_process_tree(&tree);
    match &tree_svg_result {
        Ok(svg) => {
            println!("  - Tree SVG generated: {} bytes", svg.len());
            println!("  - Contains <svg: {:?}", svg.contains("<svg"));

            if svg.len() > 100 && svg.contains("<svg") {
                println!("  ✓ PROCESS TREE VISUALIZATION WORKS\n");
            } else {
                println!("  ✗ PROCESS TREE VISUALIZATION FAILED\n");
            }
        }
        Err(e) => {
            println!("  ✗ PROCESS TREE VISUALIZATION FAILED: {:?}\n", e);
        }
    }

    println!("=== VISUALIZATION VERIFIED ===");
}
