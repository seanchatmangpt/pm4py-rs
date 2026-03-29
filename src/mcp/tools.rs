use crate::mcp::protocol::ToolDefinition;
/// Static tool definitions for the pm4py-rust MCP server.
///
/// 10 process mining tools exposed via MCP protocol.
/// Each definition includes JSON Schema inputSchema for client validation.
use serde_json::json;

pub fn all_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "pm4py_discover_alpha",
            description: "Discover a Petri net process model from an event log using the Alpha Miner algorithm. Returns places, transitions, arcs, and fitness score.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "event_log": {
                        "type": "object",
                        "description": "Event log as JSON (traces array with events)"
                    }
                },
                "required": ["event_log"]
            }),
        },
        ToolDefinition {
            name: "pm4py_conformance_token_replay",
            description: "Check conformance between an event log and a Petri net using token replay. Returns fitness, precision, and per-trace results.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "event_log": {
                        "type": "object",
                        "description": "Event log as JSON (traces array with events)"
                    },
                    "petri_net": {
                        "type": "object",
                        "description": "Petri net as JSON (places, transitions, arcs)"
                    }
                },
                "required": ["event_log", "petri_net"]
            }),
        },
        ToolDefinition {
            name: "pm4py_statistics",
            description: "Calculate statistical metrics from an event log: activity frequencies, variant distribution, bottleneck activities, and resource metrics.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "event_log": {
                        "type": "object",
                        "description": "Event log as JSON (traces array with events)"
                    }
                },
                "required": ["event_log"]
            }),
        },
        ToolDefinition {
            name: "pm4py_parse_xes",
            description: "Parse XES XML format event log into the pm4py JSON event log format. Returns the parsed event log ready for analysis.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "xes_xml": {
                        "type": "string",
                        "description": "XES XML content as a string"
                    }
                },
                "required": ["xes_xml"]
            }),
        },
        ToolDefinition {
            name: "pm4py_detect_drift",
            description: "Detect concept drift between two event log windows (baseline and recent). Returns drift detected flag and statistical metrics.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "baseline": {
                        "type": "object",
                        "description": "Baseline event log window (earlier traces)"
                    },
                    "recent": {
                        "type": "object",
                        "description": "Recent event log window (later traces to compare)"
                    }
                },
                "required": ["baseline", "recent"]
            }),
        },
        ToolDefinition {
            name: "pm4py_abstract_petri_net",
            description: "Generate an LLM-friendly natural language abstraction of a Petri net. Describes the process flow, branching, and key activities.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "petri_net": {
                        "type": "object",
                        "description": "Petri net as JSON (places, transitions, arcs)"
                    }
                },
                "required": ["petri_net"]
            }),
        },
        ToolDefinition {
            name: "pm4py_abstract_event_log",
            description: "Generate an LLM-friendly natural language abstraction of an event log. Summarizes traces, activities, and process behavior.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "event_log": {
                        "type": "object",
                        "description": "Event log as JSON (traces array with events)"
                    }
                },
                "required": ["event_log"]
            }),
        },
        ToolDefinition {
            name: "pm4py_abstract_dfg",
            description: "Generate an LLM-friendly natural language abstraction of a Directly-Follows Graph. Describes activity transitions and frequencies.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "dfg": {
                        "type": "object",
                        "description": "Directly-Follows Graph as JSON (nodes, edges with frequencies)"
                    }
                },
                "required": ["dfg"]
            }),
        },
        ToolDefinition {
            name: "pm4py_query",
            description: "Answer natural language queries about process behavior using process intelligence. Ask questions like 'What are the bottlenecks?' or 'Why is conformance low?'",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language query about process behavior"
                    },
                    "event_log": {
                        "type": "object",
                        "description": "Optional event log for statistical analysis. When provided, enables data-driven root cause analysis."
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "pm4py_ocel_ingest",
            description: "Ingest and analyze an OCEL 2.0 (Object-Centric Event Log) dataset. Returns object types, activities, and inter-object relationships.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "ocel": {
                        "type": "object",
                        "description": "OCEL 2.0 event log as JSON (objectTypes, events, objects)"
                    }
                },
                "required": ["ocel"]
            }),
        },
        ToolDefinition {
            name: "pm4py_decision_mine",
            description: "Discover decision rules at branching points in the process. Identifies split activities and returns rules with conditions, confidence scores, and support counts.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "event_log": {
                        "type": "object",
                        "description": "Event log as JSON (traces array with events)"
                    }
                },
                "required": ["event_log"]
            }),
        },
        ToolDefinition {
            name: "pm4py_predict_remaining_time",
            description: "Predict remaining time for an in-progress case using historical patterns from a training log. Returns predicted seconds, confidence score, and 95% confidence interval.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "training_log": {
                        "type": "object",
                        "description": "Historical event log used to build the prediction model (traces array with events)"
                    },
                    "partial_trace": {
                        "type": "array",
                        "description": "Events observed so far in the current in-progress case",
                        "items": {
                            "type": "object"
                        }
                    }
                },
                "required": ["training_log", "partial_trace"]
            }),
        },
    ]
}

pub fn find_tool(name: &str) -> Option<ToolDefinition> {
    all_tools().into_iter().find(|t| t.name == name)
}
