//! Converts pm4py Petri nets to YAWL XML specification format.

/// Classify join/split codes based on in/out arc counts for a transition.
#[derive(Debug, Clone, PartialEq)]
pub enum GatewayCode {
    Xor,
    And,
    /// OR-join/OR-split: inclusive routing. Currently not produced by
    /// `classify_join_split`; reserved for future OR-gateway support.
    /// See: van der Aalst (2003) YAWL Section 4.3.
    #[allow(dead_code)]
    Or,
}

impl GatewayCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            GatewayCode::Xor => "xor",
            GatewayCode::And => "and",
            GatewayCode::Or => "or",
        }
    }
}

/// Determine join and split codes for a transition based on arc topology.
/// - 1 in, 1 out → (XOR join, AND split) — sequence
/// - 1 in, N out → (XOR join, AND split) — parallel split
/// - N in, 1 out → (AND join, AND split) — synchronization
pub fn classify_join_split(in_count: usize, _out_count: usize) -> (GatewayCode, GatewayCode) {
    let join = if in_count > 1 {
        GatewayCode::And
    } else {
        GatewayCode::Xor
    };
    let split = GatewayCode::And; // Default to AND split regardless of out_count
    (join, split)
}

/// Convert a simple Petri net description to YAWL XML.
/// tasks: list of (task_name, next_task_names)
pub fn build_yawl_xml(uri: &str, first_task: &str, tasks: &[(&str, Vec<&str>)]) -> String {
    let yawl_ns = r#"xmlns="http://www.citi.qut.edu.au/yawl" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.citi.qut.edu.au/yawl YAWL_Schema.xsd""#;

    let mut xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><specificationSet {yawl_ns}><specification uri="{uri}"><metaData/><rootNet id="Net"><processControlElements>"#
    );

    xml.push_str(&format!(
        r#"<inputCondition id="InputCondition"><flowsInto><nextElementRef id="{first_task}"/></flowsInto></inputCondition>"#
    ));

    for (task_name, nexts) in tasks {
        let flows: String = nexts
            .iter()
            .map(|n| format!(r#"<flowsInto><nextElementRef id="{n}"/></flowsInto>"#))
            .collect();

        let (join, split) = classify_join_split(1, nexts.len());
        xml.push_str(&format!(
            r#"<task id="{task_name}">{flows}<join code="{j}"/><split code="{s}"/></task>"#,
            j = join.as_str(),
            s = split.as_str()
        ));
    }

    xml.push_str(r#"<outputCondition id="OutputCondition"/></processControlElements></rootNet></specification></specificationSet>"#);
    xml
}

/// Minimal YAWL XML for a sequence of tasks (WCP-1).
pub fn petri_net_to_yawl_xml(task_names: &[&str]) -> String {
    if task_names.is_empty() {
        return String::new();
    }
    let tasks: Vec<(&str, Vec<&str>)> = task_names
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            let nexts = if i + 1 < task_names.len() {
                vec![task_names[i + 1]]
            } else {
                vec!["OutputCondition"]
            };
            (name, nexts)
        })
        .collect();

    build_yawl_xml("PM4Py_Discovered", task_names[0], &tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn petri_net_to_yawl_xml_contains_task_elements() {
        let xml = petri_net_to_yawl_xml(&["task_a", "task_b", "task_c"]);
        assert!(xml.contains("<task"), "should contain task elements");
        assert!(xml.contains("flowsInto"), "should contain flowsInto");
        assert!(xml.contains("task_a"));
        assert!(xml.contains("task_b"));
    }

    #[test]
    fn petri_net_to_yawl_xml_empty_returns_empty_string() {
        let xml = petri_net_to_yawl_xml(&[]);
        assert!(xml.is_empty());
    }

    #[test]
    fn petri_net_to_yawl_xml_single_task_links_to_output_condition() {
        let xml = petri_net_to_yawl_xml(&["only_task"]);
        assert!(xml.contains("only_task"));
        assert!(xml.contains("OutputCondition"));
    }

    #[test]
    fn classify_join_split_sequence_is_xor_and() {
        let (join, split) = classify_join_split(1, 1);
        assert_eq!(join, GatewayCode::Xor);
        assert_eq!(split, GatewayCode::And);
    }

    #[test]
    fn classify_join_split_fanin_uses_and_join() {
        let (join, _) = classify_join_split(3, 1);
        assert_eq!(join, GatewayCode::And);
    }

    #[test]
    fn classify_join_split_fanout_uses_and_split() {
        let (_, split) = classify_join_split(1, 3);
        assert_eq!(split, GatewayCode::And);
    }

    #[test]
    fn build_yawl_xml_produces_valid_xml_structure() {
        let xml = build_yawl_xml(
            "Test",
            "step1",
            &[("step1", vec!["step2"]), ("step2", vec!["OutputCondition"])],
        );
        assert!(xml.starts_with("<?xml"));
        assert!(xml.contains("specificationSet"));
        assert!(xml.contains("rootNet"));
    }

    #[test]
    fn gateway_code_as_str_returns_correct_values() {
        assert_eq!(GatewayCode::Xor.as_str(), "xor");
        assert_eq!(GatewayCode::And.as_str(), "and");
        assert_eq!(GatewayCode::Or.as_str(), "or");
    }
}
