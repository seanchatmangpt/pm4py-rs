/// All 10 pm4py process mining tools exposed as A2A skills.
///
/// Skill IDs match the MCP tool names exactly — the A2A dispatch layer
/// passes these IDs directly to `mcp::executor::execute_tool`.
use crate::a2a::protocol::AgentSkill;

pub fn all_skills() -> Vec<AgentSkill> {
    vec![
        AgentSkill {
            id: "pm4py_discover_alpha".to_string(),
            name: "Process Discovery (Alpha Miner)".to_string(),
            description: "Discover a Petri net process model from an event log using the Alpha Miner algorithm. Returns petri_net JSON with fitness score.".to_string(),
            input_modes: vec!["data".to_string()],
        },
        AgentSkill {
            id: "pm4py_conformance_token_replay".to_string(),
            name: "Conformance Checking (Token Replay)".to_string(),
            description: "Check conformance of an event log against a Petri net using token replay. Returns fitness, precision, and generalization scores.".to_string(),
            input_modes: vec!["data".to_string()],
        },
        AgentSkill {
            id: "pm4py_statistics".to_string(),
            name: "Process Statistics".to_string(),
            description: "Compute trace count, event count, activity frequencies, variant distribution, and bottleneck activities from an event log.".to_string(),
            input_modes: vec!["data".to_string()],
        },
        AgentSkill {
            id: "pm4py_parse_xes".to_string(),
            name: "Parse XES Event Log".to_string(),
            description: "Parse an XES XML string into a structured event log JSON representation.".to_string(),
            input_modes: vec!["data".to_string(), "text".to_string()],
        },
        AgentSkill {
            id: "pm4py_detect_drift".to_string(),
            name: "Process Drift Detection".to_string(),
            description: "Detect statistical drift between baseline and recent process behavior. Returns drift score and changed metrics.".to_string(),
            input_modes: vec!["data".to_string()],
        },
        AgentSkill {
            id: "pm4py_abstract_petri_net".to_string(),
            name: "Abstract Petri Net".to_string(),
            description: "Generate a plain-English description of a Petri net for LLM consumption or board-level reporting.".to_string(),
            input_modes: vec!["data".to_string()],
        },
        AgentSkill {
            id: "pm4py_abstract_event_log".to_string(),
            name: "Abstract Event Log".to_string(),
            description: "Generate a plain-English summary of an event log highlighting key patterns and statistics.".to_string(),
            input_modes: vec!["data".to_string()],
        },
        AgentSkill {
            id: "pm4py_abstract_dfg".to_string(),
            name: "Abstract Directly-Follows Graph".to_string(),
            description: "Generate a plain-English description of a Directly-Follows Graph for process intelligence reporting.".to_string(),
            input_modes: vec!["data".to_string()],
        },
        AgentSkill {
            id: "pm4py_query".to_string(),
            name: "Process Intelligence Query".to_string(),
            description: "Answer causal questions about process data. Uses statistical analysis; enriches with Groq LLM when GROQ_API_KEY is available.".to_string(),
            input_modes: vec!["data".to_string(), "text".to_string()],
        },
        AgentSkill {
            id: "pm4py_ocel_ingest".to_string(),
            name: "OCEL 2.0 Ingest".to_string(),
            description: "Ingest and validate an OCEL 2.0 object-centric event log. Returns object type counts and event summary.".to_string(),
            input_modes: vec!["data".to_string()],
        },
    ]
}
