use crate::models::dfg::DirectlyFollowsGraph;
/// SVG rendering for process models
use crate::models::petri_net::PetriNet;
use crate::models::process_tree::{ProcessTree, ProcessTreeNode, TreeOperator};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::layout::{ForceDirectedLayout, HierarchicalLayout, LayoutAlgorithm};

/// Color scheme for frequency-based coloring
#[derive(Debug, Clone)]
pub struct FrequencyColorScheme {
    pub min_color: String,
    pub max_color: String,
    pub min_frequency: usize,
    pub max_frequency: usize,
}

impl FrequencyColorScheme {
    pub fn new() -> Self {
        Self {
            min_color: "#FFF9E6".to_string(),
            max_color: "#E60000".to_string(),
            min_frequency: 1,
            max_frequency: 100,
        }
    }

    pub fn get_color(&self, frequency: usize) -> String {
        let normalized = ((frequency - self.min_frequency) as f64)
            / ((self.max_frequency - self.min_frequency).max(1) as f64);
        let normalized = normalized.clamp(0.0, 1.0);
        let hue = 60.0 - (normalized * 60.0);
        let saturation = 100.0 * normalized;
        let lightness = 50.0 - (normalized * 25.0);
        format!("hsl({:.0}, {:.0}%, {:.0}%)", hue, saturation, lightness)
    }
}

impl Default for FrequencyColorScheme {
    fn default() -> Self {
        Self::new()
    }
}

/// Color scheme for performance-based coloring
#[derive(Debug, Clone)]
pub struct PerformanceColorScheme {
    pub min_color: String,
    pub max_color: String,
    pub min_duration: f64,
    pub max_duration: f64,
}

impl PerformanceColorScheme {
    pub fn new() -> Self {
        Self {
            min_color: "#00CC00".to_string(),
            max_color: "#FF0000".to_string(),
            min_duration: 0.0,
            max_duration: 86400.0,
        }
    }

    pub fn get_color(&self, duration: f64) -> String {
        let normalized = (duration - self.min_duration) / (self.max_duration - self.min_duration);
        let normalized = normalized.clamp(0.0, 1.0);
        let hue = 120.0 - (normalized * 120.0);
        format!("hsl({:.0}, 100%, 50%)", hue)
    }
}

impl Default for PerformanceColorScheme {
    fn default() -> Self {
        Self::new()
    }
}

/// SVG rendering options
#[derive(Debug, Clone)]
pub struct SvgRenderOptions {
    pub width: usize,
    pub height: usize,
    pub padding: f64,
    pub node_radius: f64,
    pub edge_width: f64,
    pub font_size: usize,
    pub show_labels: bool,
    pub frequency_threshold: usize,
    pub use_frequency_colors: bool,
    pub use_performance_colors: bool,
}

impl SvgRenderOptions {
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            padding: 20.0,
            node_radius: 25.0,
            edge_width: 2.0,
            font_size: 12,
            show_labels: true,
            frequency_threshold: 0,
            use_frequency_colors: false,
            use_performance_colors: false,
        }
    }

    pub fn with_dimensions(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_frequency_colors(mut self, enabled: bool) -> Self {
        self.use_frequency_colors = enabled;
        self
    }

    pub fn with_threshold(mut self, threshold: usize) -> Self {
        self.frequency_threshold = threshold;
        self
    }
}

impl Default for SvgRenderOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a Petri net to SVG
pub fn render_petri_net_svg(
    net: &PetriNet,
    marking: &HashMap<String, usize>,
    options: &SvgRenderOptions,
) -> String {
    let mut svg = String::new();

    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\">",
        options.width, options.height, options.width, options.height
    ));

    svg.push_str(&format!(
        "<rect width=\"{}\" height=\"{}\" fill=\"white\"/>",
        options.width, options.height
    ));

    if net.places.is_empty() {
        svg.push_str("</svg>");
        return svg;
    }

    let mut nodes = vec![];
    for place in &net.places {
        nodes.push(place.id.clone());
    }
    for trans in &net.transitions {
        nodes.push(trans.id.clone());
    }

    let edges: Vec<(String, String)> = net
        .arcs
        .iter()
        .map(|a| (a.from.clone(), a.to.clone()))
        .collect();

    let layout = HierarchicalLayout::new();
    let mut layout_result = layout.layout(&nodes, &edges);
    layout_result.normalize(options.width as f64, options.height as f64, options.padding);

    // Draw edges
    for arc in &net.arcs {
        if let (Some(from_pos), Some(to_pos)) = (
            layout_result.positions.get(&arc.from),
            layout_result.positions.get(&arc.to),
        ) {
            let x1 = from_pos.x;
            let y1 = from_pos.y;
            let x2 = to_pos.x;
            let y2 = to_pos.y;

            let angle = (y2 - y1).atan2(x2 - x1);
            let start_offset = options.node_radius * 1.1;
            let end_offset = options.node_radius * 0.8;

            let x1_adj = x1 + start_offset * angle.cos();
            let y1_adj = y1 + start_offset * angle.sin();
            let x2_adj = x2 - end_offset * angle.cos();
            let y2_adj = y2 - end_offset * angle.sin();

            svg.push_str(&format!(
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"black\" stroke-width=\"{}\" marker-end=\"url(#arrowhead)\"/>",
                x1_adj, y1_adj, x2_adj, y2_adj, options.edge_width
            ));

            if arc.weight > 1 && options.show_labels {
                let mid_x = (x1_adj + x2_adj) / 2.0;
                let mid_y = (y1_adj + y2_adj) / 2.0;
                svg.push_str(&format!(
                    "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"{}\" fill=\"black\" text-anchor=\"middle\" dy=\"-.5em\">{}</text>",
                    mid_x, mid_y, options.font_size, arc.weight
                ));
            }
        }
    }

    svg.push_str("<defs><marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"10\" refX=\"8\" refY=\"3\" orient=\"auto\"><polygon points=\"0 0, 10 3, 0 6\" fill=\"black\"/></marker></defs>");

    // Draw places
    for place in &net.places {
        if let Some(pos) = layout_result.positions.get(&place.id) {
            let tokens = marking.get(&place.id).copied().unwrap_or(0);
            let color = if tokens > 0 { "#E8F4F8" } else { "white" };
            let stroke_color = if Some(&place.id) == net.initial_place.as_ref()
                || Some(&place.id) == net.final_place.as_ref()
            {
                "blue"
            } else {
                "black"
            };

            svg.push_str(&format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"2\"/>",
                pos.x, pos.y, options.node_radius, color, stroke_color
            ));

            if tokens > 0 && tokens <= 3 {
                let token_radius = 3.0;
                let angle_step = std::f64::consts::TAU / (tokens as f64);
                for i in 0..tokens {
                    let angle = i as f64 * angle_step;
                    let tx = pos.x + (options.node_radius * 0.5) * angle.cos();
                    let ty = pos.y + (options.node_radius * 0.5) * angle.sin();
                    svg.push_str(&format!(
                        "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{}\" fill=\"black\"/>",
                        tx, ty, token_radius
                    ));
                }
            } else if tokens > 3 {
                svg.push_str(&format!(
                    "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"{}\" fill=\"black\" text-anchor=\"middle\" dy=\".3em\">{}</text>",
                    pos.x, pos.y, options.font_size, tokens
                ));
            }

            if options.show_labels {
                svg.push_str(&format!(
                    "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"{}\" fill=\"black\" text-anchor=\"middle\" dy=\"1.5em\">{}</text>",
                    pos.x, pos.y, options.font_size, place.name
                ));
            }
        }
    }

    // Draw transitions
    for trans in &net.transitions {
        if let Some(pos) = layout_result.positions.get(&trans.id) {
            let color = if trans.is_invisible() {
                "#CCCCCC"
            } else {
                "lightgreen"
            };

            let size = options.node_radius * 0.8;
            svg.push_str(&format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"black\" stroke-width=\"2\"/>",
                pos.x - size,
                pos.y - size,
                size * 2.0,
                size * 2.0,
                color
            ));

            if options.show_labels && !trans.is_invisible() {
                let label = trans.label.as_ref().unwrap_or(&trans.name);
                svg.push_str(&format!(
                    "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"{}\" fill=\"black\" text-anchor=\"middle\" dy=\"1.5em\">{}</text>",
                    pos.x, pos.y, options.font_size, label
                ));
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

/// Render a DFG to SVG
pub fn render_dfg_svg(dfg: &DirectlyFollowsGraph, options: &SvgRenderOptions) -> String {
    let mut svg = String::new();

    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\">",
        options.width, options.height, options.width, options.height
    ));

    svg.push_str(&format!(
        "<rect width=\"{}\" height=\"{}\" fill=\"white\"/>",
        options.width, options.height
    ));

    if dfg.nodes.is_empty() {
        svg.push_str("</svg>");
        return svg;
    }

    let edges: Vec<(String, String)> = dfg
        .edges
        .iter()
        .filter(|e| e.frequency >= options.frequency_threshold)
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();

    let layout = ForceDirectedLayout::new()
        .with_optimal_distance(80.0)
        .with_repulsion(8000.0);
    let mut layout_result = layout.layout(&dfg.nodes, &edges);
    layout_result.normalize(options.width as f64, options.height as f64, options.padding);

    let frequency_scheme = FrequencyColorScheme::new();
    let max_freq = dfg.edges.iter().map(|e| e.frequency).max().unwrap_or(1);

    // Draw edges
    for edge in &dfg.edges {
        if edge.frequency < options.frequency_threshold {
            continue;
        }

        if let (Some(from_pos), Some(to_pos)) = (
            layout_result.positions.get(&edge.from),
            layout_result.positions.get(&edge.to),
        ) {
            let x1 = from_pos.x;
            let y1 = from_pos.y;
            let x2 = to_pos.x;
            let y2 = to_pos.y;

            let angle = (y2 - y1).atan2(x2 - x1);
            let start_offset = options.node_radius * 1.1;
            let end_offset = options.node_radius * 0.9;

            let x1_adj = x1 + start_offset * angle.cos();
            let y1_adj = y1 + start_offset * angle.sin();
            let x2_adj = x2 - end_offset * angle.cos();
            let y2_adj = y2 - end_offset * angle.sin();

            let edge_color = if options.use_frequency_colors {
                frequency_scheme.get_color(edge.frequency)
            } else {
                "black".to_string()
            };

            let edge_width = ((edge.frequency as f64 / max_freq as f64) * 4.0 + 1.0).min(6.0);

            svg.push_str(&format!(
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\" stroke-width=\"{:.1}\" marker-end=\"url(#arrowhead)\"/>",
                x1_adj, y1_adj, x2_adj, y2_adj, edge_color, edge_width
            ));

            if options.show_labels {
                let mid_x = (x1_adj + x2_adj) / 2.0;
                let mid_y = (y1_adj + y2_adj) / 2.0;
                svg.push_str(&format!(
                    "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"{}\" fill=\"black\" text-anchor=\"middle\" dy=\"-.5em\">({})</text>",
                    mid_x, mid_y, options.font_size - 2, edge.frequency
                ));
            }
        }
    }

    svg.push_str("<defs><marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"10\" refX=\"8\" refY=\"3\" orient=\"auto\"><polygon points=\"0 0, 10 3, 0 6\" fill=\"black\"/></marker></defs>");

    // Draw nodes
    for node in &dfg.nodes {
        if let Some(pos) = layout_result.positions.get(node) {
            let freq = dfg.activity_frequency.get(node).copied().unwrap_or(0);
            let is_start = dfg.start_activities.contains_key(node);
            let is_end = dfg.end_activities.contains_key(node);

            let node_color = if options.use_frequency_colors {
                frequency_scheme.get_color(freq)
            } else if is_start || is_end {
                "#FFE6E6".to_string()
            } else {
                "#E6F2FF".to_string()
            };

            let node_width =
                ((freq as f64 / max_freq as f64) * 20.0 + 20.0).max(options.node_radius);

            svg.push_str(&format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" fill=\"{}\" stroke=\"black\" stroke-width=\"2\" />",
                pos.x, pos.y, node_width, node_color
            ));

            if options.show_labels {
                svg.push_str(&format!(
                    "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"{}\" fill=\"black\" text-anchor=\"middle\" dy=\".3em\">{}</text>",
                    pos.x, pos.y, options.font_size, node
                ));

                if freq > 0 {
                    svg.push_str(&format!(
                        "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"{}\" fill=\"gray\" text-anchor=\"middle\" dy=\"1.3em\">({})</text>",
                        pos.x, pos.y, options.font_size - 2, freq
                    ));
                }
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

/// Render a process tree to SVG
pub fn render_process_tree_svg(tree: &ProcessTree, options: &SvgRenderOptions) -> String {
    let mut svg = String::new();

    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\">",
        options.width, options.height, options.width, options.height
    ));

    svg.push_str(&format!(
        "<rect width=\"{}\" height=\"{}\" fill=\"white\"/>",
        options.width, options.height
    ));

    let y_spacing = 80.0;
    let x_spacing = 60.0;

    render_tree_node(
        &tree.root,
        options.width as f64 / 2.0,
        options.padding,
        &mut svg,
        y_spacing,
        x_spacing,
        options,
    );

    svg.push_str("</svg>");
    svg
}

fn render_tree_node(
    node: &ProcessTreeNode,
    x: f64,
    y: f64,
    svg: &mut String,
    y_spacing: f64,
    x_spacing: f64,
    options: &SvgRenderOptions,
) {
    match node {
        ProcessTreeNode::Activity(name) => {
            let radius = options.node_radius * 0.8;
            svg.push_str(&format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" fill=\"#E6F2FF\" stroke=\"black\" stroke-width=\"2\"/>",
                x, y, radius
            ));

            if options.show_labels {
                svg.push_str(&format!(
                    "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"{}\" fill=\"black\" text-anchor=\"middle\" dy=\".3em\">{}</text>",
                    x, y, options.font_size - 2, name
                ));
            }
        }
        ProcessTreeNode::Operator {
            operator, children, ..
        } => {
            let op_size = options.node_radius * 0.7;
            let op_color = match operator {
                TreeOperator::Sequence => "#FFE6E6",
                TreeOperator::Choice => "#FFFACD",
                TreeOperator::Parallel => "#E6FFE6",
                TreeOperator::Loop => "#E6E6FF",
            };

            svg.push_str(&format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{}\" stroke=\"black\" stroke-width=\"2\" rx=\"5\"/>",
                x - op_size,
                y - op_size,
                op_size * 2.0,
                op_size * 2.0,
                op_color
            ));

            if options.show_labels {
                svg.push_str(&format!(
                    "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"{}\" fill=\"black\" text-anchor=\"middle\" dy=\".35em\" font-weight=\"bold\">{}</text>",
                    x, y, options.font_size, operator.as_str()
                ));
            }

            let child_count = children.len() as f64;
            let total_child_width = child_count * x_spacing * 2.0;
            let mut child_x = x - (total_child_width / 2.0);

            for child in children {
                let child_y = y + y_spacing;

                svg.push_str(&format!(
                    "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"gray\" stroke-width=\"1\"/>",
                    x,
                    y + op_size,
                    child_x,
                    child_y - (op_size / 2.0)
                ));

                render_tree_node(child, child_x, child_y, svg, y_spacing, x_spacing, options);

                child_x += x_spacing * 2.5;
            }
        }
    }
}

/// Write SVG to file
pub fn write_svg_to_file(svg: &str, path: &Path) -> std::io::Result<()> {
    fs::write(path, svg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, EventLog, Trace};
    use chrono::Utc;

    #[test]
    fn test_frequency_color_scheme() {
        let scheme = FrequencyColorScheme::new();
        let color1 = scheme.get_color(1);
        let color2 = scheme.get_color(50);

        assert!(!color1.is_empty());
        assert!(!color2.is_empty());
    }

    #[test]
    fn test_performance_color_scheme() {
        let scheme = PerformanceColorScheme::new();
        let color1 = scheme.get_color(0.0);
        let color2 = scheme.get_color(43200.0);

        assert!(!color1.is_empty());
        assert!(!color2.is_empty());
    }

    #[test]
    fn test_render_empty_petri_net() {
        let net = PetriNet::new();
        let marking = HashMap::new();
        let options = SvgRenderOptions::new();

        let svg = render_petri_net_svg(&net, &marking, &options);

        assert!(svg.contains("svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_render_dfg() {
        let mut log = EventLog::new();
        let mut trace = Trace::new("case_1");
        let now = Utc::now();

        trace.add_event(Event::new("a", now));
        trace.add_event(Event::new("b", now));
        trace.add_event(Event::new("c", now));

        log.add_trace(trace);

        let dfg = DirectlyFollowsGraph::from_log(&log);
        let options = SvgRenderOptions::new().with_frequency_colors(true);

        let svg = render_dfg_svg(&dfg, &options);

        assert!(svg.contains("svg"));
        assert!(svg.contains("circle"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_render_process_tree() {
        let tree = ProcessTree::new(ProcessTreeNode::sequence(vec![
            ProcessTreeNode::activity("A"),
            ProcessTreeNode::activity("B"),
        ]));

        let options = SvgRenderOptions::new();
        let svg = render_process_tree_svg(&tree, &options);

        assert!(svg.contains("svg"));
        assert!(svg.contains("circle"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_svg_options() {
        let opts = SvgRenderOptions::new()
            .with_dimensions(1000, 800)
            .with_frequency_colors(true)
            .with_threshold(5);

        assert_eq!(opts.width, 1000);
        assert_eq!(opts.height, 800);
        assert!(opts.use_frequency_colors);
        assert_eq!(opts.frequency_threshold, 5);
    }
}
