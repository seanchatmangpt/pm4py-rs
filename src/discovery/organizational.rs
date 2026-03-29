//! Organizational Mining
//!
//! Discovery of organizational patterns from event logs.

use crate::log::EventLog;
use std::collections::{HashMap, HashSet};

/// Activity-based resource similarity matrix
pub fn discover_activity_based_resource_similarity(
    log: &EventLog,
) -> HashMap<(String, String), f64> {
    let mut activity_resources: HashMap<String, HashSet<String>> = HashMap::new();
    let mut all_resources: HashSet<String> = HashSet::new();

    for trace in &log.traces {
        for event in &trace.events {
            if let Some(resource) = &event.resource {
                activity_resources
                    .entry(event.activity.clone())
                    .or_default()
                    .insert(resource.clone());
                all_resources.insert(resource.clone());
            }
        }
    }

    let mut similarity = HashMap::new();
    let activities: Vec<String> = activity_resources.keys().cloned().collect();

    for i in 0..activities.len() {
        for j in (i + 1)..activities.len() {
            let act1 = &activities[i];
            let act2 = &activities[j];

            let resources1 = activity_resources.get(act1).map(|s| s.len()).unwrap_or(0);
            let resources2 = activity_resources.get(act2).map(|s| s.len()).unwrap_or(0);

            let intersection = activity_resources
                .get(act1)
                .and_then(|s1| {
                    activity_resources
                        .get(act2)
                        .map(|s2| s1.intersection(s2).count())
                })
                .unwrap_or(0);

            let union = resources1 + resources2 - intersection;

            let jaccard = if union > 0 {
                intersection as f64 / union as f64
            } else {
                0.0
            };

            similarity.insert((act1.clone(), act2.clone()), jaccard);
        }
    }

    similarity
}

/// Discover organizational roles from resource-activity associations
pub fn discover_organizational_roles(log: &EventLog) -> HashMap<String, Vec<String>> {
    let mut resource_activities: HashMap<String, HashSet<String>> = HashMap::new();

    for trace in &log.traces {
        for event in &trace.events {
            if let Some(resource) = &event.resource {
                resource_activities
                    .entry(resource.clone())
                    .or_default()
                    .insert(event.activity.clone());
            }
        }
    }

    // Group resources by similar activity patterns
    let mut roles: HashMap<String, Vec<String>> = HashMap::new();

    let resource_list: Vec<String> = resource_activities.keys().cloned().collect();
    let mut assigned = HashSet::new();

    for (i, resource1) in resource_list.iter().enumerate() {
        if assigned.contains(resource1) {
            continue;
        }

        let activities1 = resource_activities
            .get(resource1)
            .cloned()
            .unwrap_or_default();
        let mut role_members = vec![resource1.clone()];
        assigned.insert(resource1.clone());

        // Find resources with similar activity patterns
        for resource2 in resource_list.iter().skip(i + 1) {
            if assigned.contains(resource2) {
                continue;
            }

            let activities2 = resource_activities
                .get(resource2)
                .cloned()
                .unwrap_or_default();

            // Calculate similarity
            let intersection = activities1.intersection(&activities2).count();
            let union = activities1.union(&activities2).count();

            if union > 0 && intersection as f64 / union as f64 > 0.7 {
                role_members.push(resource2.clone());
                assigned.insert(resource2.clone());
            }
        }

        let role_name = format!("Role_{}", roles.len() + 1);
        roles.insert(role_name, role_members);
    }

    roles
}

/// Handover of work network (which resources hand over work to which)
pub fn discover_handover_of_work_network(log: &EventLog) -> HashMap<(String, String), usize> {
    let mut handover: HashMap<(String, String), usize> = HashMap::new();

    for trace in &log.traces {
        for window in trace.events.windows(2) {
            if let (Some(res1), Some(res2)) = (&window[0].resource, &window[1].resource) {
                if res1 != res2 {
                    *handover.entry((res1.clone(), res2.clone())).or_insert(0) += 1;
                }
            }
        }
    }

    handover
}

/// Working together network (which resources work together on cases)
pub fn discover_working_together_network(log: &EventLog) -> HashMap<(String, String), usize> {
    let mut working_together: HashMap<(String, String), usize> = HashMap::new();

    for trace in &log.traces {
        let mut resources_in_trace: HashSet<String> = HashSet::new();

        for event in &trace.events {
            if let Some(resource) = &event.resource {
                resources_in_trace.insert(resource.clone());
            }
        }

        // Count all pairs of resources that worked on the same case
        let resource_list: Vec<String> = resources_in_trace.into_iter().collect();
        for i in 0..resource_list.len() {
            for j in (i + 1)..resource_list.len() {
                let pair = if resource_list[i] < resource_list[j] {
                    (resource_list[i].clone(), resource_list[j].clone())
                } else {
                    (resource_list[j].clone(), resource_list[i].clone())
                };
                *working_together.entry(pair).or_insert(0) += 1;
            }
        }
    }

    working_together
}

/// Subcontracting network (which resources are contracted by which)
pub fn discover_subcontracting_network(
    log: &EventLog,
    main_organization: &str,
) -> HashMap<String, Vec<String>> {
    let mut subcontractors: HashMap<String, Vec<String>> = HashMap::new();

    for trace in &log.traces {
        let mut org_resources: HashMap<String, Vec<String>> = HashMap::new();

        for event in &trace.events {
            let org = event
                .get_attribute("organization")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            if let Some(resource) = &event.resource {
                org_resources.entry(org).or_default().push(resource.clone());
            }
        }

        // Find subcontracting relationships
        if let Some(main_resources) = org_resources.get(main_organization) {
            for (org, resources) in &org_resources {
                if org != main_organization {
                    for resource in resources {
                        for _main_res in main_resources {
                            if !main_resources.contains(resource) {
                                subcontractors
                                    .entry(org.clone())
                                    .or_default()
                                    .push(resource.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    subcontractors
}

/// Network analysis metrics for organizational networks
pub fn discover_network_analysis(log: &EventLog) -> OrganizationalNetworkMetrics {
    let handover = discover_handover_of_work_network(log);
    let working_together = discover_working_together_network(log);

    // Calculate network metrics
    let mut node_degree: HashMap<String, usize> = HashMap::new();

    for ((from, to), count) in &handover {
        *node_degree.entry(from.clone()).or_insert(0) += count;
        *node_degree.entry(to.clone()).or_insert(0) += count;
    }

    for ((r1, r2), count) in &working_together {
        *node_degree.entry(r1.clone()).or_insert(0) += count;
        *node_degree.entry(r2.clone()).or_insert(0) += count;
    }

    let total_nodes = node_degree.len();
    let total_edges = handover.len() + working_together.len();

    OrganizationalNetworkMetrics {
        handover_network: handover,
        working_together_network: working_together,
        node_degree,
        total_nodes,
        total_edges,
    }
}

/// Organizational network metrics
#[derive(Debug, Clone)]
pub struct OrganizationalNetworkMetrics {
    pub handover_network: HashMap<(String, String), usize>,
    pub working_together_network: HashMap<(String, String), usize>,
    pub node_degree: HashMap<String, usize>,
    pub total_nodes: usize,
    pub total_edges: usize,
}
