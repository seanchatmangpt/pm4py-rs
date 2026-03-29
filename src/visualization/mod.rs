pub mod animation;
pub mod dotted_chart;
pub mod interactive;
pub mod layout;
pub mod save_vis;
/// Visualization module for process models
///
/// This module provides SVG rendering for various process models:
/// - Petri nets with markings
/// - Directly-Follows Graphs (DFG)
/// - Process trees
///
/// Features:
/// - Layout algorithms (force-directed, hierarchical)
/// - Frequency-based coloring
/// - Performance-based coloring (duration)
/// - Labels and annotations
/// - Export to SVG files or strings
pub mod svg_renderer;

pub use animation::{
    create_animation_from_log, create_animation_from_trace, Animation, AnimationFrame,
    AnimationOptions, AnimationSpeed,
};
pub use dotted_chart::{create_dotted_chart, DottedChart, DottedChartOptions};
pub use interactive::{
    create_interactive_dfg, create_interactive_petri_net, InteractiveOptions,
    InteractiveVisualization,
};
pub use layout::{ForceDirectedLayout, HierarchicalLayout, LayoutAlgorithm, LayoutResult};
pub use save_vis::*;
pub use svg_renderer::{
    render_dfg_svg, render_petri_net_svg, render_process_tree_svg, FrequencyColorScheme,
    PerformanceColorScheme, SvgRenderOptions,
};
