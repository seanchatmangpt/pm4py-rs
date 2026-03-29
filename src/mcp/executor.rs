/// Shared tool execution logic — called by both the MCP handler and the A2A skill dispatcher.
///
/// WvdA boundedness guards:
/// - Event log max 10,000 events enforced before heavy algorithms
/// - Caller wraps with tokio::time::timeout (25 s for MCP, 60 s for A2A)
use serde_json::{json, Value};

pub const MAX_EVENTS: usize = 10_000;

/// Execute a named pm4py tool with the given arguments.
/// Returns the JSON result as a pretty-printed string, or an error message.
pub(crate) async fn execute_tool(name: &str, args: Value) -> Result<String, String> {
    match name {
        "pm4py_discover_alpha" => {
            let event_log_json = args.get("event_log").ok_or("Missing event_log argument")?;
            let event_log = parse_event_log(event_log_json)?;
            check_event_count(&event_log)?;

            use crate::conformance::TokenReplay;
            use crate::discovery::AlphaMiner;
            use crate::http::businessos_api::PetriNetJson;

            let miner = AlphaMiner::new();
            let petri_net = miner.discover(&event_log);
            let checker = TokenReplay::new();
            let conformance = checker.check(&event_log, &petri_net);
            let net_json = PetriNetJson::from_petri_net(&petri_net);

            let result = json!({
                "algorithm": "alpha_miner",
                "petri_net": net_json,
                "fitness": conformance.fitness,
                "place_count": petri_net.places.len(),
                "transition_count": petri_net.transitions.len()
            });
            Ok(serde_json::to_string_pretty(&result).unwrap_or_default())
        }

        "pm4py_conformance_token_replay" => {
            let event_log_json = args.get("event_log").ok_or("Missing event_log argument")?;
            let petri_net_json = args.get("petri_net").ok_or("Missing petri_net argument")?;

            let event_log = parse_event_log(event_log_json)?;
            check_event_count(&event_log)?;

            use crate::conformance::TokenReplay;
            use crate::http::businessos_api::PetriNetJson;

            let net_parsed: PetriNetJson = serde_json::from_value(petri_net_json.clone())
                .map_err(|e| format!("Failed to parse petri_net: {}", e))?;
            let petri_net = parse_petri_net_json(&net_parsed)?;

            let checker = TokenReplay::new();
            let result = checker.check(&event_log, &petri_net);

            let out = json!({
                "is_conformant": result.is_conformant,
                "fitness": result.fitness,
                "precision": result.precision,
                "generalization": result.generalization,
                "method": "token_replay"
            });
            Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
        }

        "pm4py_statistics" => {
            let event_log_json = args.get("event_log").ok_or("Missing event_log argument")?;
            let event_log = parse_event_log(event_log_json)?;
            check_event_count(&event_log)?;

            use crate::statistics::advanced::{
                get_activity_frequency, get_bottleneck_activities, get_variant_frequency,
            };

            let trace_count = event_log.traces.len();
            let event_count: usize = event_log.traces.iter().map(|t| t.events.len()).sum();

            let mut activities = std::collections::HashSet::new();
            for trace in &event_log.traces {
                for event in &trace.events {
                    activities.insert(event.activity.clone());
                }
            }

            let activity_freqs: std::collections::HashMap<String, usize> =
                get_activity_frequency(&event_log)
                    .into_iter()
                    .map(|f| (f.activity, f.total_count))
                    .collect();

            let variant_freqs: std::collections::HashMap<String, usize> =
                get_variant_frequency(&event_log)
                    .into_iter()
                    .map(|f| (f.variant, f.count))
                    .collect();

            let bottlenecks: Vec<String> = get_bottleneck_activities(&event_log, 5)
                .into_iter()
                .map(|(a, _)| a)
                .collect();

            let out = json!({
                "trace_count": trace_count,
                "event_count": event_count,
                "unique_activities": activities.len(),
                "activity_frequencies": activity_freqs,
                "variant_count": variant_freqs.len(),
                "bottleneck_activities": bottlenecks
            });
            Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
        }

        "pm4py_parse_xes" => {
            let xes_xml = args
                .get("xes_xml")
                .and_then(|v| v.as_str())
                .ok_or("Missing or non-string xes_xml argument")?;

            use crate::io::xes::XESReader;
            let reader = XESReader::new();
            let log = reader
                .parse_str(xes_xml)
                .map_err(|e| format!("XES parse failed: {}", e))?;

            let v =
                serde_json::to_value(&log).map_err(|e| format!("Serialization failed: {}", e))?;
            Ok(serde_json::to_string_pretty(&v).unwrap_or_default())
        }

        "pm4py_detect_drift" => {
            let baseline_json = args.get("baseline").ok_or("Missing baseline argument")?;
            let recent_json = args.get("recent").ok_or("Missing recent argument")?;

            let baseline: std::collections::HashMap<String, f64> =
                serde_json::from_value(baseline_json.clone())
                    .map_err(|e| format!("Failed to parse baseline: {}", e))?;
            let recent: std::collections::HashMap<String, f64> =
                serde_json::from_value(recent_json.clone())
                    .map_err(|e| format!("Failed to parse recent: {}", e))?;

            use crate::monitoring::DriftCalculator;
            let calculator = DriftCalculator::new();
            let drift_score = calculator.calculate_drift(&baseline, &recent);
            let is_drifted = calculator.is_drift_detected(drift_score);
            let changed = calculator.identify_changed_metrics(&baseline, &recent);

            let out = json!({
                "drift_detected": is_drifted,
                "drift_score": drift_score,
                "changed_metrics": changed
            });
            Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
        }

        "pm4py_abstract_petri_net" => {
            let petri_net_json = args.get("petri_net").ok_or("Missing petri_net argument")?;
            use crate::http::businessos_api::PetriNetJson;
            let net: PetriNetJson = serde_json::from_value(petri_net_json.clone())
                .map_err(|e| format!("Failed to parse petri_net: {}", e))?;
            let petri_net = parse_petri_net_json(&net)?;

            use crate::llm::abstract_petri_net;
            let description = abstract_petri_net(&petri_net);
            Ok(description)
        }

        "pm4py_abstract_event_log" => {
            let event_log_json = args.get("event_log").ok_or("Missing event_log argument")?;
            let event_log = parse_event_log(event_log_json)?;
            check_event_count(&event_log)?;

            use crate::llm::abstract_event_log;
            let description = abstract_event_log(&event_log);
            Ok(description)
        }

        "pm4py_abstract_dfg" => {
            let dfg_json = args.get("dfg").ok_or("Missing dfg argument")?;
            use crate::models::DirectlyFollowsGraph;
            let dfg: DirectlyFollowsGraph = serde_json::from_value(dfg_json.clone())
                .map_err(|e| format!("Failed to parse dfg: {}", e))?;

            use crate::llm::abstract_dfg;
            let description = abstract_dfg(&dfg);
            Ok(description)
        }

        "pm4py_query" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or("Missing or non-string query argument")?;

            let event_log_json = args.get("event_log");
            let api_key = std::env::var("GROQ_API_KEY").ok();

            use crate::llm::domain2_intelligence::IntelligenceContext;
            use crate::llm::{
                answer_causal_question, format_intelligence_answer, groq_chat_with_span,
                GroqMessage,
            };

            match (event_log_json, api_key) {
                // Path A: event_log + GROQ_API_KEY → statistical analysis + Groq enrichment
                (Some(el_json), Some(key)) => {
                    let event_log = parse_event_log(el_json)?;
                    check_event_count(&event_log)?;
                    let ctx = IntelligenceContext {
                        log: Some(&event_log),
                        petri_net: None,
                    };
                    let answer = answer_causal_question(query, &ctx);
                    let base_text = format_intelligence_answer(&answer);
                    let messages = vec![
                        GroqMessage { role: "system".to_string(), content: "You are an expert in process mining and business process analysis (van der Aalst methodology). Answer questions about process performance, bottlenecks, conformance, and improvement. Be concise and data-oriented. Max 200 words.".to_string() },
                        GroqMessage { role: "user".to_string(), content: format!("Statistical analysis:\n{}\n\nQuestion: {}", base_text, query) },
                    ];
                    match groq_chat_with_span(&key, messages).await {
                        Ok(resp) if !resp.choices.is_empty() => {
                            let enriched = resp.choices[0].message.content.clone();
                            let out = json!({ "query": query, "response": enriched, "path": "groq_augmented", "statistical_base": base_text });
                            Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
                        }
                        _ => {
                            let out = json!({ "query": query, "response": base_text, "path": "statistical" });
                            Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
                        }
                    }
                }
                // Path B: event_log only → pure statistical
                (Some(el_json), None) => {
                    let event_log = parse_event_log(el_json)?;
                    check_event_count(&event_log)?;
                    let ctx = IntelligenceContext {
                        log: Some(&event_log),
                        petri_net: None,
                    };
                    let answer = answer_causal_question(query, &ctx);
                    let text = format_intelligence_answer(&answer);
                    let out = json!({ "query": query, "response": text, "path": "statistical" });
                    Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
                }
                // Path C: GROQ_API_KEY only → pure Groq
                (None, Some(key)) => {
                    let messages = vec![
                        GroqMessage { role: "system".to_string(), content: "You are an expert in process mining and business process analysis (van der Aalst methodology). Answer questions about process performance, bottlenecks, conformance, and improvement. Be concise and data-oriented. Max 200 words.".to_string() },
                        GroqMessage { role: "user".to_string(), content: query.to_string() },
                    ];
                    match groq_chat_with_span(&key, messages).await {
                        Ok(resp) if !resp.choices.is_empty() => {
                            let answer_text = resp.choices[0].message.content.clone();
                            let out =
                                json!({ "query": query, "response": answer_text, "path": "groq" });
                            Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
                        }
                        Ok(_) => {
                            let out = json!({ "query": query, "response": "Groq returned no choices.", "path": "groq" });
                            Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
                        }
                        Err(e) => Err(format!("Groq call failed: {}", e)),
                    }
                }
                // Path D: fallback — no event_log, no GROQ_API_KEY
                (None, None) => {
                    let out = json!({
                        "query": query,
                        "response": "Set GROQ_API_KEY env var or provide event_log for analysis.",
                        "path": "fallback"
                    });
                    Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
                }
            }
        }

        "pm4py_decision_mine" => {
            let event_log_json = args.get("event_log").ok_or("Missing event_log argument")?;
            let event_log = parse_event_log(event_log_json)?;
            check_event_count(&event_log)?;

            use crate::discovery::decision_mining::mine_decision_rules;
            let model = mine_decision_rules(&event_log);

            let rules_json: Vec<serde_json::Value> = model
                .rules
                .iter()
                .map(|r| {
                    json!({
                        "split_activity": r.split_activity,
                        "condition": r.condition,
                        "confidence": r.confidence,
                        "support": r.support
                    })
                })
                .collect();

            let out = json!({
                "rule_count": rules_json.len(),
                "rules": rules_json
            });
            Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
        }

        "pm4py_predict_remaining_time" => {
            let training_log_json = args
                .get("training_log")
                .ok_or("Missing training_log argument")?;
            let partial_trace_json = args
                .get("partial_trace")
                .ok_or("Missing partial_trace argument")?;

            let training_log = parse_event_log(training_log_json)?;
            check_event_count(&training_log)?;

            let partial_events: Vec<crate::log::Event> =
                serde_json::from_value(partial_trace_json.clone())
                    .map_err(|e| format!("Failed to parse partial_trace: {}", e))?;

            use crate::predictive::remaining_time::predict_remaining_time_from_log;
            let resp = predict_remaining_time_from_log(&training_log, &partial_events)
                .ok_or("Cannot predict: insufficient training data or empty partial trace")?;

            let out = json!({
                "predicted_remaining_seconds": resp.predicted_remaining_seconds,
                "confidence": resp.confidence,
                "similar_cases_count": resp.similar_cases_count,
                "percentile_10": resp.percentile_10,
                "percentile_90": resp.percentile_90
            });
            Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
        }

        "pm4py_ocel_ingest" => {
            let ocel_json = args.get("ocel").ok_or("Missing ocel argument")?;
            use crate::ocpm::object_log::ObjectCentricEventLog;

            let ocel: ObjectCentricEventLog = serde_json::from_value(ocel_json.clone())
                .map_err(|e| format!("Failed to parse OCEL: {}", e))?;

            let object_type_count = ocel.object_types.len();
            let event_count = ocel.events.len();
            let object_count = ocel.objects.len();

            let out = json!({
                "object_type_count": object_type_count,
                "event_count": event_count,
                "object_count": object_count,
                "object_types": ocel.object_types
            });
            Ok(serde_json::to_string_pretty(&out).unwrap_or_default())
        }

        unknown => Err(format!("Unknown tool: {}", unknown)),
    }
}

fn parse_event_log(value: &Value) -> Result<crate::log::EventLog, String> {
    serde_json::from_value(value.clone())
        .map_err(|e| format!("Failed to deserialize event log: {}", e))
}

fn check_event_count(log: &crate::log::EventLog) -> Result<(), String> {
    let count: usize = log.traces.iter().map(|t| t.events.len()).sum();
    if count > MAX_EVENTS {
        return Err(format!(
            "Event log too large: {} events (max {}). Reduce the log size before processing.",
            count, MAX_EVENTS
        ));
    }
    Ok(())
}

fn parse_petri_net_json(
    json: &crate::http::businessos_api::PetriNetJson,
) -> Result<crate::models::PetriNet, String> {
    use crate::models::{Arc, PetriNet, Place, Transition};

    let places: Vec<Place> = json
        .places
        .iter()
        .map(|p| Place {
            id: p.id.clone(),
            name: p.name.clone(),
            initial_marking: p.initial_marking,
            final_marking: None,
        })
        .collect();

    let transitions: Vec<Transition> = json
        .transitions
        .iter()
        .map(|t| Transition {
            id: t.id.clone(),
            name: t.name.clone(),
            label: t.label.clone(),
        })
        .collect();

    let arcs: Vec<Arc> = json
        .arcs
        .iter()
        .map(|a| Arc {
            from: a.from.clone(),
            to: a.to.clone(),
            weight: a.weight,
        })
        .collect();

    Ok(PetriNet {
        places,
        transitions,
        arcs,
        initial_place: json.initial_place.clone(),
        final_place: json.final_place.clone(),
    })
}
