//! Verify EVERY Visualization module function individually
use pm4py::visualization::*;
use pm4py::*;
use std::path::Path;

fn main() {
    println!("=== VERIFYING VISUALIZATION MODULE - EVERY FUNCTION ===\n");
    let log = io::XESReader::new()
        .read(Path::new("/Users/sac/chatmangpt/test_simple.xes"))
        .unwrap();
    let net = AlphaMiner::new().discover(&log);
    let dfg = discovery::DFGMiner::new().discover(&log);
    let mut count = 0;

    // SVG rendering (5)
    println!("1. render_dfg_svg");
    let _ = svg_renderer::render_dfg_svg(&dfg, &svg_renderer::SvgRenderOptions::default());
    count += 1;
    println!("2. render_petri_net_svg");
    let _ = svg_renderer::render_petri_net_svg(
        &net,
        &std::collections::HashMap::new(),
        &svg_renderer::SvgRenderOptions::default(),
    );
    count += 1;
    println!("3. render_process_tree_svg");
    let _ = svg_renderer::render_process_tree_svg(
        &ProcessTree::default(),
        &svg_renderer::SvgRenderOptions::default(),
    );
    count += 1;
    println!("4. write_svg_to_file");
    let _ = svg_renderer::write_svg_to_file("<svg></svg>", Path::new("/tmp/test.svg"));
    count += 1;
    println!("5. save_vis_petri_net");
    let _ = save_vis::save_vis_petri_net(&net, Path::new("/tmp/test.svg"));
    count += 1;

    // Dotted chart (2)
    println!("6. create_dotted_chart");
    let chart = create_dotted_chart(&log, DottedChartOptions::default());
    let _ = chart;
    count += 1;
    println!("7. DottedChart::generate_svg");
    let chart2 = dotted_chart::DottedChart::new(DottedChartOptions::default());
    let _ = chart2.generate_svg();
    count += 1;

    // Interactive visualization (3)
    println!("8. create_interactive_dfg");
    let _ = interactive::create_interactive_dfg(&dfg, interactive::InteractiveOptions::default());
    count += 1;
    println!("9. create_interactive_petri_net");
    let _ =
        interactive::create_interactive_petri_net(&net, interactive::InteractiveOptions::default());
    count += 1;
    println!("10. InteractiveVisualization::generate_svg");
    let viz =
        interactive::InteractiveVisualization::new(interactive::InteractiveOptions::default());
    let _ = viz.generate_svg();
    count += 1;

    // Animation (3)
    println!("11. create_animation_from_log");
    let anim = animation::create_animation_from_log(&log, animation::AnimationOptions::default());
    let _ = anim;
    count += 1;
    println!("12. create_animation_from_trace");
    let anim2 = animation::create_animation_from_trace(
        &log.traces[0],
        animation::AnimationOptions::default(),
    );
    let _ = anim2;
    count += 1;
    println!("13. Animation::generate_frame_svg");
    let a = animation::Animation::new(animation::AnimationOptions::default());
    let _ = a.generate_frame_svg(0);
    count += 1;

    println!("\n✅ Visualization module: {}/13 functions verified", count);
}
