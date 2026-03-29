/// Trace-level statistics
use crate::log::Trace;
use std::collections::HashMap;

/// Get trace length distribution
pub fn trace_length_distribution(traces: &[Trace]) -> HashMap<usize, usize> {
    let mut distribution = HashMap::new();

    for trace in traces {
        let len = trace.len();
        *distribution.entry(len).or_insert(0) += 1;
    }

    distribution
}

/// Get unique traces (variants)
pub fn unique_traces(traces: &[Trace]) -> Vec<Vec<String>> {
    let mut unique = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for trace in traces {
        let variant: Vec<String> = trace.events.iter().map(|e| e.activity.clone()).collect();

        if !seen.contains(&variant) {
            seen.insert(variant.clone());
            unique.push(variant);
        }
    }

    unique
}

/// Count variant occurrences
pub fn variant_frequencies(traces: &[Trace]) -> HashMap<String, usize> {
    let mut frequencies = HashMap::new();

    for trace in traces {
        let variant: String = trace
            .events
            .iter()
            .map(|e| e.activity.clone())
            .collect::<Vec<_>>()
            .join(",");

        *frequencies.entry(variant).or_insert(0) += 1;
    }

    frequencies
}

/// Get trace attributes summary
pub fn trace_attribute_stats(traces: &[Trace]) -> HashMap<String, Vec<String>> {
    let mut stats: HashMap<String, Vec<String>> = HashMap::new();

    for trace in traces {
        for (key, value) in &trace.attributes {
            stats.entry(key.clone()).or_default().push(value.clone());
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::Event;
    use chrono::Utc;

    fn create_test_traces() -> Vec<Trace> {
        let now = Utc::now();
        let mut trace1 = Trace::new("case_1");
        trace1.add_event(Event::new("a", now));
        trace1.add_event(Event::new("b", now));

        let mut trace2 = Trace::new("case_2");
        trace2.add_event(Event::new("a", now));
        trace2.add_event(Event::new("b", now));

        vec![trace1, trace2]
    }

    #[test]
    fn test_trace_length_distribution() {
        let traces = create_test_traces();
        let dist = trace_length_distribution(&traces);

        assert_eq!(dist.get(&2), Some(&2));
    }

    #[test]
    fn test_unique_traces() {
        let traces = create_test_traces();
        let unique = unique_traces(&traces);

        assert_eq!(unique.len(), 1);
    }

    #[test]
    fn test_variant_frequencies() {
        let traces = create_test_traces();
        let freq = variant_frequencies(&traces);

        assert_eq!(freq.get("a,b"), Some(&2));
    }
}
