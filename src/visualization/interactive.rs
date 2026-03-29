//! Interactive Model Visualization

use crate::models::dfg::DirectlyFollowsGraph;
use crate::models::petri_net::PetriNet;
use std::collections::HashMap;

/// Interactive visualization options
#[derive(Debug, Clone)]
pub struct InteractiveOptions {
    pub width: usize,
    pub height: usize,
    pub enable_zoom: bool,
    pub enable_pan: bool,
    pub enable_tooltips: bool,
    pub enable_animation: bool,
    pub zoom_factor: f64,
    pub initial_zoom: f64,
}

impl InteractiveOptions {
    pub fn new() -> Self {
        Self {
            width: 1200,
            height: 800,
            enable_zoom: true,
            enable_pan: true,
            enable_tooltips: true,
            enable_animation: true,
            zoom_factor: 1.2,
            initial_zoom: 1.0,
        }
    }

    pub fn with_zoom(mut self, enable: bool) -> Self {
        self.enable_zoom = enable;
        self
    }

    pub fn with_pan(mut self, enable: bool) -> Self {
        self.enable_pan = enable;
        self
    }

    pub fn with_tooltips(mut self, enable: bool) -> Self {
        self.enable_tooltips = enable;
        self
    }

    pub fn with_animation(mut self, enable: bool) -> Self {
        self.enable_animation = enable;
        self
    }
}

impl Default for InteractiveOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct InteractiveNode {
    id: String,
    label: String,
    x: f64,
    y: f64,
    radius: f64,
    color: String,
    tooltip: String,
}

#[derive(Debug, Clone)]
struct InteractiveEdge {
    from: String,
    to: String,
    label: String,
    color: String,
    weight: f64,
}

#[derive(Debug, Clone)]
pub struct InteractiveVisualization {
    nodes: Vec<InteractiveNode>,
    edges: Vec<InteractiveEdge>,
    options: InteractiveOptions,
    filter_state: HashMap<String, bool>,
}

impl InteractiveVisualization {
    pub fn new(options: InteractiveOptions) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            options,
            filter_state: HashMap::new(),
        }
    }

    pub fn add_node(
        &mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        x: f64,
        y: f64,
        color: impl Into<String>,
    ) -> &mut Self {
        let id = id.into();
        let label_str = label.into();
        self.nodes.push(InteractiveNode {
            id: id.clone(),
            label: label_str.clone(),
            x,
            y,
            radius: 25.0,
            color: color.into(),
            tooltip: format!("Activity: {}", label_str),
        });
        self
    }

    pub fn add_edge(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        label: impl Into<String>,
        color: impl Into<String>,
    ) -> &mut Self {
        self.edges.push(InteractiveEdge {
            from: from.into(),
            to: to.into(),
            label: label.into(),
            color: color.into(),
            weight: 2.0,
        });
        self
    }

    pub fn generate_svg(&self) -> String {
        let mut svg = String::new();
        svg.push_str(&format!(
            "<svg xmlns='http://www.w3.org/2000/svg' width='{}' height='{}' viewBox='0 0 {} {}'>",
            self.options.width, self.options.height, self.options.width, self.options.height
        ));

        svg.push_str("<defs><style>");
        svg.push_str(".interactive-node { cursor: pointer; fill: steelblue; }");
        svg.push_str(".interactive-node:hover { filter: brightness(1.2); }");
        svg.push_str(".interactive-edge { stroke: gray; stroke-width: 2; pointer-events: none; }");
        svg.push_str("</style></defs>");

        if self.options.enable_zoom || self.options.enable_pan {
            svg.push_str("<g id='viewport' transform='scale(1, 1) translate(0, 0)'>");
        }

        for edge in &self.edges {
            let from_node = self.nodes.iter().find(|n| n.id == edge.from);
            let to_node = self.nodes.iter().find(|n| n.id == edge.to);
            if let (Some(from), Some(to)) = (from_node, to_node) {
                svg.push_str(&format!("<line x1='{:.1}' y1='{:.1}' x2='{:.1}' y2='{:.1}' stroke='{}' stroke-width='{:.1}' class='interactive-edge'/>",
                    from.x, from.y, to.x, to.y, edge.color, edge.weight));
                let mid_x = (from.x + to.x) / 2.0;
                let mid_y = (from.y + to.y) / 2.0;
                svg.push_str(&format!("<text x='{:.1}' y='{:.1}' font-size='12' text-anchor='middle' fill='black' pointer-events='none'>{}</text>",
                    mid_x, mid_y, edge.label));
            }
        }

        for node in &self.nodes {
            svg.push_str(&format!("<circle cx='{:.1}' cy='{:.1}' r='{:.1}' fill='{}' stroke='black' stroke-width='2' class='interactive-node' data-id='{}'/>",
                node.x, node.y, node.radius, node.color, node.id));
            svg.push_str(&format!("<text x='{:.1}' y='{:.1}' text-anchor='middle' dominant-baseline='middle' font-size='14' fill='white' pointer-events='none' font-weight='bold'>{}</text>",
                node.x, node.y, node.label));
            if self.options.enable_tooltips {
                svg.push_str(&format!("<title>{}</title>", node.tooltip));
            }
        }

        if self.options.enable_zoom || self.options.enable_pan {
            svg.push_str("</g>");
        }
        svg.push_str("</svg>");
        svg
    }

    pub fn set_filter(&mut self, node_id: impl Into<String>, visible: bool) {
        self.filter_state.insert(node_id.into(), visible);
    }

    pub fn apply_filters(&mut self) {
        for (node_id, visible) in &self.filter_state {
            if let Some(node) = self.nodes.iter_mut().find(|n| &n.id == node_id) {
                node.color = if *visible {
                    "#4CAF50".to_string()
                } else {
                    "#CCCCCC".to_string()
                };
            }
        }
    }
}

pub fn create_interactive_petri_net(
    petri_net: &PetriNet,
    options: InteractiveOptions,
) -> InteractiveVisualization {
    let mut viz = InteractiveVisualization::new(options);
    let num_places = petri_net.places.len();
    let num_transitions = petri_net.transitions.len();
    let total_nodes = num_places + num_transitions;
    let angle_step = 2.0 * std::f64::consts::PI / total_nodes.max(1) as f64;
    let radius = 200.0;
    let center_x = viz.options.width as f64 / 2.0;
    let center_y = viz.options.height as f64 / 2.0;

    for (idx, place) in petri_net.places.iter().enumerate() {
        let angle = angle_step * idx as f64;
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        viz.add_node(&place.id, &place.name, x, y, "#2196F3");
    }

    for (idx, transition) in petri_net.transitions.iter().enumerate() {
        let angle = angle_step * (idx + num_places) as f64;
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        let color = if transition.is_invisible() {
            "#9C27B0"
        } else {
            "#FF9800"
        };
        viz.add_node(
            &transition.id,
            transition.label.as_ref().unwrap_or(&transition.name),
            x,
            y,
            color,
        );
    }

    for arc in &petri_net.arcs {
        viz.add_edge(&arc.from, &arc.to, arc.weight.to_string(), "#666666");
    }
    viz
}

pub fn create_interactive_dfg(
    dfg: &DirectlyFollowsGraph,
    options: InteractiveOptions,
) -> InteractiveVisualization {
    let mut viz = InteractiveVisualization::new(options);
    let num_activities = dfg.nodes.len();
    let angle_step = 2.0 * std::f64::consts::PI / num_activities.max(1) as f64;
    let radius = 200.0;
    let center_x = viz.options.width as f64 / 2.0;
    let center_y = viz.options.height as f64 / 2.0;

    for (idx, activity) in dfg.nodes.iter().enumerate() {
        let angle = angle_step * idx as f64;
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        viz.add_node(activity, activity, x, y, "#4CAF50");
    }

    for edge in &dfg.edges {
        viz.add_edge(
            &edge.from,
            &edge.to,
            format!("({})", edge.frequency),
            "#999999",
        );
    }
    viz
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_options_creation() {
        let opts = InteractiveOptions::new();
        assert_eq!(opts.width, 1200);
        assert_eq!(opts.height, 800);
        assert!(opts.enable_zoom);
        assert!(opts.enable_pan);
    }

    #[test]
    fn test_interactive_options_builder() {
        let opts = InteractiveOptions::new().with_zoom(false).with_pan(false);
        assert!(!opts.enable_zoom);
        assert!(!opts.enable_pan);
    }

    #[test]
    fn test_add_node_and_edge() {
        let mut viz = InteractiveVisualization::new(InteractiveOptions::new());
        viz.add_node("n1", "Node 1", 100.0, 100.0, "#FF0000");
        viz.add_node("n2", "Node 2", 200.0, 200.0, "#00FF00");
        viz.add_edge("n1", "n2", "edge1", "#000000");
        assert_eq!(viz.nodes.len(), 2);
        assert_eq!(viz.edges.len(), 1);
    }

    #[test]
    fn test_generate_svg_contains_elements() {
        let mut viz = InteractiveVisualization::new(InteractiveOptions::new());
        viz.add_node("n1", "Activity A", 100.0, 100.0, "#FF0000");
        viz.add_edge("n1", "n1", "self-loop", "#000000");
        let svg = viz.generate_svg();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("circle"));
        assert!(svg.contains("Activity A"));
    }

    #[test]
    fn test_filter_state() {
        let mut viz = InteractiveVisualization::new(InteractiveOptions::new());
        viz.add_node("n1", "Activity A", 100.0, 100.0, "#FF0000");
        viz.set_filter("n1", true);
        assert_eq!(viz.filter_state.get("n1"), Some(&true));
        viz.set_filter("n1", false);
        assert_eq!(viz.filter_state.get("n1"), Some(&false));
    }

    #[test]
    fn test_apply_filters() {
        let mut viz = InteractiveVisualization::new(InteractiveOptions::new());
        viz.add_node("n1", "Activity A", 100.0, 100.0, "#FF0000");
        viz.set_filter("n1", false);
        viz.apply_filters();
        let node = viz
            .nodes
            .iter()
            .find(|n| n.id == "n1")
            .expect("test node n1 must exist");
        assert_eq!(node.color, "#CCCCCC");
    }

    #[test]
    fn test_default_options() {
        let opts1 = InteractiveOptions::default();
        let opts2 = InteractiveOptions::new();
        assert_eq!(opts1.width, opts2.width);
    }

    #[test]
    fn test_svg_generation() {
        let mut viz = InteractiveVisualization::new(InteractiveOptions::new());
        viz.add_node("n1", "Test Node", 400.0, 300.0, "#2196F3");
        let svg = viz.generate_svg();
        assert!(svg.contains("xmlns"));
        assert!(svg.contains("viewport"));
    }
}
