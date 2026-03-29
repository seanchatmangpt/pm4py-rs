/// Layout algorithms for graph visualization
use std::collections::HashMap;
use std::f64::consts::PI;

/// Represents a 2D point for layout
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Layout result containing node positions and bounds
#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub positions: HashMap<String, Point>,
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub width: f64,
    pub height: f64,
}

impl LayoutResult {
    /// Translate all positions to origin and scale to fit in bounds
    pub fn normalize(&mut self, width: f64, height: f64, padding: f64) {
        // Translate to origin
        for pos in self.positions.values_mut() {
            pos.x -= self.min_x - padding;
            pos.y -= self.min_y - padding;
        }

        // Recalculate bounds
        self.min_x = padding;
        self.min_y = padding;
        self.max_x = width - padding;
        self.max_y = height - padding;
        self.width = self.max_x - self.min_x;
        self.height = self.max_y - self.min_y;

        // Scale if necessary
        let current_width = self.positions.values().map(|p| p.x).fold(0.0, f64::max) + padding;
        let current_height = self.positions.values().map(|p| p.y).fold(0.0, f64::max) + padding;

        if current_width > 0.0 && current_height > 0.0 {
            let scale_x = if current_width > width - 2.0 * padding {
                (width - 2.0 * padding) / current_width
            } else {
                1.0
            };
            let scale_y = if current_height > height - 2.0 * padding {
                (height - 2.0 * padding) / current_height
            } else {
                1.0
            };

            let scale = scale_x.min(scale_y);
            for pos in self.positions.values_mut() {
                pos.x *= scale;
                pos.y *= scale;
                pos.x += padding;
                pos.y += padding;
            }
        }
    }
}

/// Layout algorithm trait
pub trait LayoutAlgorithm {
    fn layout(&self, nodes: &[String], edges: &[(String, String)]) -> LayoutResult;
}

/// Force-directed layout (spring embedding)
pub struct ForceDirectedLayout {
    pub iterations: usize,
    pub cooling_rate: f64,
    pub optimal_distance: f64,
    pub repulsion: f64,
}

impl ForceDirectedLayout {
    pub fn new() -> Self {
        Self {
            iterations: 100,
            cooling_rate: 0.95,
            optimal_distance: 50.0,
            repulsion: 5000.0,
        }
    }

    pub fn with_iterations(mut self, iter: usize) -> Self {
        self.iterations = iter;
        self
    }

    pub fn with_optimal_distance(mut self, dist: f64) -> Self {
        self.optimal_distance = dist;
        self
    }

    pub fn with_repulsion(mut self, rep: f64) -> Self {
        self.repulsion = rep;
        self
    }
}

impl Default for ForceDirectedLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutAlgorithm for ForceDirectedLayout {
    fn layout(&self, nodes: &[String], edges: &[(String, String)]) -> LayoutResult {
        let mut positions: HashMap<String, Point> = HashMap::new();

        // Initialize random positions
        for (i, node) in nodes.iter().enumerate() {
            let angle = (i as f64) * 2.0 * PI / (nodes.len() as f64);
            let radius = 100.0;
            positions.insert(
                node.clone(),
                Point::new(radius * angle.cos() + 150.0, radius * angle.sin() + 150.0),
            );
        }

        // Simulation loop
        let mut temperature = 100.0;
        for _iter in 0..self.iterations {
            let mut forces: HashMap<String, (f64, f64)> = HashMap::new();

            // Initialize forces
            for node in nodes {
                forces.insert(node.clone(), (0.0, 0.0));
            }

            // Repulsive forces between all nodes
            for i in 0..nodes.len() {
                for j in (i + 1)..nodes.len() {
                    let node1 = &nodes[i];
                    let node2 = &nodes[j];
                    let pos1 = positions[node1];
                    let pos2 = positions[node2];

                    let dx = pos2.x - pos1.x;
                    let dy = pos2.y - pos1.y;
                    let distance = (dx * dx + dy * dy).sqrt().max(1.0);

                    let force = self.repulsion / (distance * distance);
                    let fx = (dx / distance) * force;
                    let fy = (dy / distance) * force;

                    let (f1x, f1y) = forces.entry(node1.clone()).or_insert((0.0, 0.0));
                    *f1x -= fx;
                    *f1y -= fy;

                    let (f2x, f2y) = forces.entry(node2.clone()).or_insert((0.0, 0.0));
                    *f2x += fx;
                    *f2y += fy;
                }
            }

            // Attractive forces along edges
            for (node1, node2) in edges {
                let pos1 = positions[node1];
                let pos2 = positions[node2];

                let dx = pos2.x - pos1.x;
                let dy = pos2.y - pos1.y;
                let distance = (dx * dx + dy * dy).sqrt().max(1.0);

                let force = distance / self.optimal_distance;
                let fx = (dx / distance) * force;
                let fy = (dy / distance) * force;

                let (f1x, f1y) = forces.entry(node1.clone()).or_insert((0.0, 0.0));
                *f1x += fx;
                *f1y += fy;

                let (f2x, f2y) = forces.entry(node2.clone()).or_insert((0.0, 0.0));
                *f2x -= fx;
                *f2y -= fy;
            }

            // Update positions
            for node in nodes {
                if let Some(&(fx, fy)) = forces.get(node) {
                    let force_magnitude = (fx * fx + fy * fy).sqrt();
                    let max_displacement = temperature;
                    let displacement = (force_magnitude).min(max_displacement);

                    let dx = if force_magnitude > 0.0 {
                        (fx / force_magnitude) * displacement
                    } else {
                        0.0
                    };
                    let dy = if force_magnitude > 0.0 {
                        (fy / force_magnitude) * displacement
                    } else {
                        0.0
                    };

                    let pos = positions
                        .entry(node.clone())
                        .or_insert(Point::new(0.0, 0.0));
                    pos.x += dx;
                    pos.y += dy;
                }
            }

            temperature *= self.cooling_rate;
        }

        // Calculate bounds
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for pos in positions.values() {
            min_x = min_x.min(pos.x);
            max_x = max_x.max(pos.x);
            min_y = min_y.min(pos.y);
            max_y = max_y.max(pos.y);
        }

        LayoutResult {
            positions,
            min_x,
            max_x,
            min_y,
            max_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

/// Hierarchical layout (top-down for DAGs)
pub struct HierarchicalLayout {
    pub horizontal_spacing: f64,
    pub vertical_spacing: f64,
}

impl HierarchicalLayout {
    pub fn new() -> Self {
        Self {
            horizontal_spacing: 80.0,
            vertical_spacing: 60.0,
        }
    }

    pub fn with_spacing(mut self, h: f64, v: f64) -> Self {
        self.horizontal_spacing = h;
        self.vertical_spacing = v;
        self
    }
}

impl Default for HierarchicalLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutAlgorithm for HierarchicalLayout {
    fn layout(&self, nodes: &[String], edges: &[(String, String)]) -> LayoutResult {
        let mut positions: HashMap<String, Point> = HashMap::new();

        // Simple hierarchical: group by topological level
        let mut levels: Vec<Vec<String>> = vec![vec![]];
        let mut assigned: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Find root nodes (no incoming edges)
        let outgoing: std::collections::HashSet<String> =
            edges.iter().map(|(_, to)| to.clone()).collect();
        let root_nodes: Vec<String> = nodes
            .iter()
            .filter(|n| !outgoing.contains(*n))
            .cloned()
            .collect();

        if root_nodes.is_empty() {
            // If no clear root, start with first node
            if !nodes.is_empty() {
                levels[0].push(nodes[0].clone());
                assigned.insert(nodes[0].clone());
            }
        } else {
            levels[0] = root_nodes;
            for node in &levels[0] {
                assigned.insert(node.clone());
            }
        }

        // Assign remaining nodes to levels
        while assigned.len() < nodes.len() {
            let mut next_level = vec![];

            for edge in edges {
                if assigned.contains(&edge.0)
                    && !assigned.contains(&edge.1)
                    && !next_level.contains(&edge.1)
                {
                    next_level.push(edge.1.clone());
                }
            }

            if next_level.is_empty() {
                // Fallback: add any unassigned node
                for node in nodes {
                    if !assigned.contains(node) {
                        next_level.push(node.clone());
                        break;
                    }
                }
            }

            for node in &next_level {
                assigned.insert(node.clone());
            }
            if !next_level.is_empty() {
                levels.push(next_level);
            } else {
                break;
            }
        }

        // Assign positions based on levels
        let max_width = levels
            .iter()
            .map(|l| l.len() as f64 * self.horizontal_spacing)
            .fold(0.0, f64::max);

        for (level_idx, level) in levels.iter().enumerate() {
            let y = level_idx as f64 * self.vertical_spacing;
            for (node_idx, node) in level.iter().enumerate() {
                let x = (node_idx as f64) * self.horizontal_spacing
                    + (max_width - (level.len() as f64) * self.horizontal_spacing) / 2.0;
                positions.insert(node.clone(), Point::new(x, y));
            }
        }

        // Calculate bounds
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for pos in positions.values() {
            min_x = min_x.min(pos.x);
            max_x = max_x.max(pos.x);
            min_y = min_y.min(pos.y);
            max_y = max_y.max(pos.y);
        }

        LayoutResult {
            positions,
            min_x,
            max_x,
            min_y,
            max_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_force_directed_layout() {
        let nodes = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let edges = vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
        ];

        let layout = ForceDirectedLayout::new();
        let result = layout.layout(&nodes, &edges);

        assert_eq!(result.positions.len(), 3);
        assert!(result.width > 0.0);
        assert!(result.height > 0.0);
    }

    #[test]
    fn test_hierarchical_layout() {
        let nodes = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let edges = vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
        ];

        let layout = HierarchicalLayout::new();
        let result = layout.layout(&nodes, &edges);

        assert_eq!(result.positions.len(), 3);
        // Hierarchical should create distinct levels
        let y_values: std::collections::HashSet<_> =
            result.positions.values().map(|p| p.y as i32).collect();
        assert!(y_values.len() > 1);
    }

    #[test]
    fn test_layout_normalize() {
        let mut result = LayoutResult {
            positions: {
                let mut m = HashMap::new();
                m.insert("A".to_string(), Point::new(0.0, 0.0));
                m.insert("B".to_string(), Point::new(100.0, 100.0));
                m
            },
            min_x: 0.0,
            max_x: 100.0,
            min_y: 0.0,
            max_y: 100.0,
            width: 100.0,
            height: 100.0,
        };

        result.normalize(400.0, 400.0, 20.0);
        assert!(result
            .positions
            .values()
            .all(|p| p.x >= 20.0 && p.y >= 20.0));
    }
}
