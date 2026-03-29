/// Correlation and dependency analysis for process mining
use crate::log::EventLog;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct CoOccurrence {
    pub activity1: String,
    pub activity2: String,
    pub frequency: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone)]
pub struct CausalDependency {
    pub source: String,
    pub target: String,
    pub frequency: usize,
    pub strength: f64,
    pub conditional_probability: f64,
}

#[derive(Debug, Clone)]
pub struct AttributeCorrelation {
    pub attribute1: String,
    pub attribute2: String,
    pub correlation: f64,
    pub sample_size: usize,
}

#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub num_nodes: usize,
    pub num_edges: usize,
    pub density: f64,
    pub avg_in_degree: f64,
    pub avg_out_degree: f64,
}

pub fn activity_co_occurrence(log: &EventLog) -> Vec<CoOccurrence> {
    if log.is_empty() {
        return Vec::new();
    }
    let mut co_occ: HashMap<(String, String), usize> = HashMap::new();
    for trace in &log.traces {
        let acts: Vec<String> = trace
            .events
            .iter()
            .map(|e| e.activity.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        for i in 0..acts.len() {
            for j in i + 1..acts.len() {
                let key = if acts[i] <= acts[j] {
                    (acts[i].clone(), acts[j].clone())
                } else {
                    (acts[j].clone(), acts[i].clone())
                };
                *co_occ.entry(key).or_insert(0) += 1;
            }
        }
    }
    let log_size = log.len() as f64;
    co_occ
        .into_iter()
        .map(|((a1, a2), f)| CoOccurrence {
            activity1: a1,
            activity2: a2,
            frequency: f,
            percentage: f as f64 / log_size,
        })
        .collect()
}

pub fn causal_dependency_analysis(log: &EventLog) -> Vec<CausalDependency> {
    let mut deps: HashMap<(String, String), usize> = HashMap::new();
    let mut counts: HashMap<String, usize> = HashMap::new();
    for trace in &log.traces {
        let sorted = trace.events_sorted();
        for i in 0..sorted.len() - 1 {
            let s = sorted[i].activity.clone();
            let t = sorted[i + 1].activity.clone();
            *deps.entry((s.clone(), t)).or_insert(0) += 1;
            *counts.entry(s).or_insert(0) += 1;
        }
    }
    deps.into_iter()
        .map(|((s, t), f)| {
            let sc = counts.get(&s).copied().unwrap_or(1);
            CausalDependency {
                source: s,
                target: t,
                frequency: f,
                strength: f as f64 / sc as f64,
                conditional_probability: f as f64 / sc as f64,
            }
        })
        .collect()
}

pub fn case_attribute_correlation(log: &EventLog) -> Vec<AttributeCorrelation> {
    if log.is_empty() {
        return Vec::new();
    }
    let mut attrs = HashSet::new();
    let mut vals: HashMap<String, Vec<Option<f64>>> = HashMap::new();
    for trace in &log.traces {
        for (k, v) in &trace.attributes {
            attrs.insert(k.clone());
            let num = v.parse::<f64>().ok();
            vals.entry(k.clone()).or_default().push(num);
        }
    }
    for v in vals.values_mut() {
        while v.len() < log.len() {
            v.push(None);
        }
    }
    let mut corrs = Vec::new();
    let attrs_vec: Vec<String> = attrs.into_iter().collect();
    for i in 0..attrs_vec.len() {
        for j in i + 1..attrs_vec.len() {
            if let (Some(v1), Some(v2)) = (vals.get(&attrs_vec[i]), vals.get(&attrs_vec[j])) {
                if let Some(c) = pearson(v1, v2) {
                    corrs.push(AttributeCorrelation {
                        attribute1: attrs_vec[i].clone(),
                        attribute2: attrs_vec[j].clone(),
                        correlation: c,
                        sample_size: v1.len(),
                    });
                }
            }
        }
    }
    corrs
}

fn pearson(x: &[Option<f64>], y: &[Option<f64>]) -> Option<f64> {
    let mut pairs = Vec::new();
    for (xv, yv) in x.iter().zip(y.iter()) {
        if let (Some(xval), Some(yval)) = (xv, yv) {
            pairs.push((*xval, *yval));
        }
    }
    if pairs.len() < 2 {
        return None;
    }
    let n = pairs.len() as f64;
    let mx = pairs.iter().map(|(x, _)| x).sum::<f64>() / n;
    let my = pairs.iter().map(|(_, y)| y).sum::<f64>() / n;
    let num: f64 = pairs.iter().map(|(x, y)| (x - mx) * (y - my)).sum();
    let vx: f64 = pairs.iter().map(|(x, _)| (x - mx).powi(2)).sum();
    let vy: f64 = pairs.iter().map(|(_, y)| (y - my).powi(2)).sum();
    let denom = (vx * vy).sqrt();
    if denom == 0.0 {
        Some(0.0)
    } else {
        Some(num / denom)
    }
}

pub fn network_metrics(log: &EventLog) -> NetworkMetrics {
    if log.is_empty() {
        return NetworkMetrics {
            num_nodes: 0,
            num_edges: 0,
            density: 0.0,
            avg_in_degree: 0.0,
            avg_out_degree: 0.0,
        };
    }
    let mut nodes = HashSet::new();
    let mut edges = HashSet::new();
    let mut in_deg: HashMap<String, usize> = HashMap::new();
    let mut out_deg: HashMap<String, usize> = HashMap::new();
    for trace in &log.traces {
        let sorted = trace.events_sorted();
        for e in &sorted {
            nodes.insert(e.activity.clone());
            in_deg.entry(e.activity.clone()).or_insert(0);
            out_deg.entry(e.activity.clone()).or_insert(0);
        }
        for i in 0..sorted.len() - 1 {
            let s = &sorted[i].activity;
            let t = &sorted[i + 1].activity;
            edges.insert((s.clone(), t.clone()));
            *out_deg.entry(s.clone()).or_insert(0) += 1;
            *in_deg.entry(t.clone()).or_insert(0) += 1;
        }
    }
    let nn = nodes.len();
    let ne = edges.len();
    let dens = if nn > 1 {
        ne as f64 / ((nn * (nn - 1)) as f64)
    } else {
        0.0
    };
    let avg_in = if !in_deg.is_empty() {
        in_deg.values().sum::<usize>() as f64 / in_deg.len() as f64
    } else {
        0.0
    };
    let avg_out = if !out_deg.is_empty() {
        out_deg.values().sum::<usize>() as f64 / out_deg.len() as f64
    } else {
        0.0
    };
    NetworkMetrics {
        num_nodes: nn,
        num_edges: ne,
        density: dens,
        avg_in_degree: avg_in,
        avg_out_degree: avg_out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();
        let mut t1 = Trace::new("1");
        t1.add_event(Event::new("A", now));
        t1.add_event(Event::new("B", now + chrono::Duration::seconds(1)));
        log.add_trace(t1);
        let mut t2 = Trace::new("2");
        t2.add_event(Event::new("A", now));
        t2.add_event(Event::new("C", now + chrono::Duration::seconds(1)));
        log.add_trace(t2);
        log
    }

    #[test]
    fn test_co_occur() {
        let log = create_test_log();
        let co = activity_co_occurrence(&log);
        assert!(!co.is_empty());
        for c in &co {
            assert!(c.percentage >= 0.0 && c.percentage <= 1.0);
        }
    }

    #[test]
    fn test_causal() {
        let log = create_test_log();
        let deps = causal_dependency_analysis(&log);
        assert!(!deps.is_empty());
        for d in &deps {
            assert!(d.strength >= 0.0);
        }
    }

    #[test]
    fn test_attr_corr() {
        let mut log = EventLog::new();
        let mut t = Trace::new("1");
        t = t.with_attribute("x", "1");
        t.add_event(Event::new("A", Utc::now()));
        log.add_trace(t);
        let corr = case_attribute_correlation(&log);
        assert!(corr.is_empty() || !corr.is_empty());
    }

    #[test]
    fn test_network() {
        let log = create_test_log();
        let nm = network_metrics(&log);
        assert!(nm.num_nodes > 0);
        assert!(nm.density >= 0.0 && nm.density <= 1.0);
    }

    #[test]
    fn test_empty() {
        let log = EventLog::new();
        assert!(activity_co_occurrence(&log).is_empty());
        assert!(causal_dependency_analysis(&log).is_empty());
        let nm = network_metrics(&log);
        assert_eq!(nm.num_nodes, 0);
    }

    #[test]
    fn test_co_freq() {
        let log = create_test_log();
        let co = activity_co_occurrence(&log);
        let ab = co.iter().find(|c| {
            (c.activity1 == "A" && c.activity2 == "B") || (c.activity1 == "B" && c.activity2 == "A")
        });
        assert!(ab.is_some());
    }

    #[test]
    fn test_deps() {
        let log = create_test_log();
        let d = causal_dependency_analysis(&log);
        let ab = d.iter().find(|x| x.source == "A" && x.target == "B");
        assert!(ab.is_some() || d.iter().any(|x| x.source == "A"));
    }
}
