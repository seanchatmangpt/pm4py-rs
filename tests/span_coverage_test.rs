/// Span coverage regression gate for pm4py-rust.
///
/// Every span name constant defined in semconv modules must:
///   1. Be a non-empty string.
///   2. Follow the dotted namespace format (e.g., "healing.diagnosis").
///   3. Match the expected domain prefix.
///
/// Armstrong: if a constant is renamed or removed in semconv YAML and
/// the Rust constants are regenerated, the compile error here is the first
/// signal that dead semconv has accumulated.
///
/// Run: cargo test span_coverage

// ─────────────────────────────────────────────────────────────────────────────
// Process Mining spans
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod mining_span_coverage {
    // Only import mining_span_names here to avoid ambiguity with process_mining_span_names
    use pm4py::semconv::mining_span_names::{
        PROCESS_MINING_CANOPY_INGEST_SPAN, PROCESS_MINING_DECLARE_CHECK_SPAN,
        PROCESS_MINING_PREDICTION_MAKE_SPAN, PROCESS_MINING_SOCIAL_NETWORK_ANALYZE_SPAN,
    };

    #[test]
    fn mining_span_constants_are_non_empty_dotted_strings() {
        let spans = [
            PROCESS_MINING_CANOPY_INGEST_SPAN,
            PROCESS_MINING_DECLARE_CHECK_SPAN,
            PROCESS_MINING_PREDICTION_MAKE_SPAN,
            PROCESS_MINING_SOCIAL_NETWORK_ANALYZE_SPAN,
        ];
        for span in &spans {
            let span: &str = span;
            assert!(!span.is_empty(), "Mining span must not be empty: {span:?}");
            assert!(
                span.contains('.'),
                "Mining span '{span}' must use dotted namespace format"
            );
        }
    }

    #[test]
    fn process_mining_discovery_span_name_matches_semconv() {
        use pm4py::semconv::spans::PROCESS_MINING_DISCOVERY;
        assert_eq!(PROCESS_MINING_DISCOVERY, "process.mining.discovery");
    }

    #[test]
    fn mining_prediction_make_matches_semconv() {
        assert_eq!(
            PROCESS_MINING_PREDICTION_MAKE_SPAN,
            "process.mining.prediction.make"
        );
    }
}

#[cfg(test)]
mod process_mining_span_coverage {
    use pm4py::semconv::process_mining_span_names::{
        PROCESS_MINING_ALIGNMENT_ANALYZE_SPAN, PROCESS_MINING_BOTTLENECK_ANALYZE_SPAN,
        PROCESS_MINING_BOTTLENECK_DETECTION_SPAN, PROCESS_MINING_CASE_CLUSTER_SPAN,
        PROCESS_MINING_COMPLEXITY_MEASURE_SPAN, PROCESS_MINING_CONFORMANCE_DEVIATION_SPAN,
        PROCESS_MINING_CONFORMANCE_REPAIR_SPAN, PROCESS_MINING_CONFORMANCE_THRESHOLD_SPAN,
        PROCESS_MINING_CONFORMANCE_VISUALIZE_SPAN, PROCESS_MINING_DECISION_MINE_SPAN,
    };

    #[test]
    fn process_mining_span_constants_are_non_empty_dotted_strings() {
        let spans = [
            PROCESS_MINING_ALIGNMENT_ANALYZE_SPAN,
            PROCESS_MINING_BOTTLENECK_ANALYZE_SPAN,
            PROCESS_MINING_BOTTLENECK_DETECTION_SPAN,
            PROCESS_MINING_CASE_CLUSTER_SPAN,
            PROCESS_MINING_COMPLEXITY_MEASURE_SPAN,
            PROCESS_MINING_CONFORMANCE_DEVIATION_SPAN,
            PROCESS_MINING_CONFORMANCE_REPAIR_SPAN,
            PROCESS_MINING_CONFORMANCE_THRESHOLD_SPAN,
            PROCESS_MINING_CONFORMANCE_VISUALIZE_SPAN,
            PROCESS_MINING_DECISION_MINE_SPAN,
        ];
        for span in &spans {
            assert!(
                !span.is_empty(),
                "Process mining span must not be empty: {span:?}"
            );
            assert!(
                span.contains('.'),
                "Process mining span '{span}' must use dotted namespace format"
            );
        }
    }

    #[test]
    fn process_mining_bottleneck_detection_matches_semconv() {
        assert_eq!(
            PROCESS_MINING_BOTTLENECK_DETECTION_SPAN,
            "process.mining.bottleneck_detection"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Healing spans (Agent 5)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod healing_span_coverage {
    use pm4py::semconv::healing_span_names::*;

    #[test]
    fn healing_diagnosis_span_name_matches_semconv() {
        assert_eq!(HEALING_DIAGNOSIS_SPAN, "healing.diagnosis");
    }

    #[test]
    fn healing_reflex_arc_span_name_matches_semconv() {
        assert_eq!(HEALING_REFLEX_ARC_SPAN, "healing.reflex_arc");
    }

    #[test]
    fn healing_fingerprint_span_name_matches_semconv() {
        assert_eq!(HEALING_FINGERPRINT_SPAN, "healing.fingerprint");
    }

    #[test]
    fn healing_escalation_span_name_matches_semconv() {
        assert_eq!(HEALING_ESCALATION_SPAN, "healing.escalation");
    }

    #[test]
    fn all_healing_span_constants_are_non_empty_dotted_strings() {
        let spans = [
            HEALING_ADAPTIVE_ADJUST_SPAN,
            HEALING_ANOMALY_DETECT_SPAN,
            HEALING_BACKPRESSURE_APPLY_SPAN,
            HEALING_CASCADE_DETECT_SPAN,
            HEALING_CHECKPOINT_CREATE_SPAN,
            HEALING_CIRCUIT_BREAKER_TRIP_SPAN,
            HEALING_COLD_STANDBY_PROMOTE_SPAN,
            HEALING_DIAGNOSIS_SPAN,
            HEALING_ESCALATION_SPAN,
            HEALING_FAILOVER_EXECUTE_SPAN,
            HEALING_FINGERPRINT_SPAN,
            HEALING_INTERVENTION_SCORE_SPAN,
            HEALING_LOAD_SHEDDING_APPLY_SPAN,
            HEALING_MEMORY_SNAPSHOT_SPAN,
            HEALING_MTTR_MEASURE_SPAN,
            HEALING_PATTERN_MATCH_SPAN,
            HEALING_PLAYBOOK_EXECUTE_SPAN,
            HEALING_PREDICTION_MAKE_SPAN,
            HEALING_QUARANTINE_APPLY_SPAN,
            HEALING_RATE_LIMIT_ENFORCE_SPAN,
            HEALING_RECOVERY_SIMULATE_SPAN,
            HEALING_RECOVERY_LOOP_SPAN,
            HEALING_REFLEX_ARC_SPAN,
            HEALING_RETRY_ADAPTIVE_SPAN,
            HEALING_ROLLBACK_EXECUTE_SPAN,
            HEALING_SELF_HEALING_TRIGGER_SPAN,
            HEALING_SURGE_DETECT_SPAN,
            HEALING_WARM_STANDBY_ACTIVATE_SPAN,
        ];
        for span in &spans {
            assert!(!span.is_empty(), "Healing span must not be empty: {span:?}");
            assert!(
                span.contains('.'),
                "Healing span '{span}' must use dotted namespace format"
            );
            assert!(
                span.starts_with("healing."),
                "Healing span '{span}' must start with 'healing.'"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// YAWL spans (Agent 6)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod yawl_span_coverage {
    use pm4py::semconv::yawl_span_names::*;

    #[test]
    fn yawl_case_span_name_matches_semconv() {
        assert_eq!(YAWL_CASE_SPAN, "yawl.case");
    }

    #[test]
    fn yawl_task_execution_span_name_matches_semconv() {
        assert_eq!(YAWL_TASK_EXECUTION_SPAN, "yawl.task.execution");
    }

    #[test]
    fn all_yawl_span_constants_are_non_empty_dotted_strings() {
        let spans = [YAWL_CASE_SPAN, YAWL_TASK_EXECUTION_SPAN];
        for span in &spans {
            assert!(!span.is_empty(), "YAWL span must not be empty: {span:?}");
            assert!(
                span.contains('.'),
                "YAWL span '{span}' must use dotted namespace format"
            );
            assert!(
                span.starts_with("yawl."),
                "YAWL span '{span}' must start with 'yawl.'"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// JTBD spans (Agent 8)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod jtbd_span_coverage {
    use pm4py::semconv::jtbd_span_names::*;

    #[test]
    fn jtbd_loop_span_name_matches_semconv() {
        assert_eq!(JTBD_LOOP_SPAN, "jtbd.loop");
    }

    #[test]
    fn jtbd_scenario_span_name_matches_semconv() {
        assert_eq!(JTBD_SCENARIO_SPAN, "jtbd.scenario");
    }

    #[test]
    fn jtbd_icp_qualification_span_name_matches_semconv() {
        assert_eq!(
            JTBD_SCENARIO_ICP_QUALIFICATION_SPAN,
            "jtbd.scenario.icp_qualification"
        );
    }

    #[test]
    fn jtbd_deal_progression_span_name_matches_semconv() {
        assert_eq!(
            JTBD_SCENARIO_DEAL_PROGRESSION_SPAN,
            "jtbd.scenario.deal_progression"
        );
    }

    #[test]
    fn jtbd_contract_closure_span_name_matches_semconv() {
        assert_eq!(
            JTBD_SCENARIO_CONTRACT_CLOSURE_SPAN,
            "jtbd.scenario.contract_closure"
        );
    }

    #[test]
    fn jtbd_process_intelligence_query_span_name_matches_semconv() {
        assert_eq!(
            JTBD_SCENARIO_PROCESS_INTELLIGENCE_QUERY_SPAN,
            "jtbd.scenario.process_intelligence_query"
        );
    }

    #[test]
    fn jtbd_dmaic_phase_span_name_matches_semconv() {
        assert_eq!(JTBD_DMAIC_PHASE_SPAN, "jtbd.dmaic.phase");
    }

    #[test]
    fn all_jtbd_span_constants_are_non_empty_dotted_strings() {
        let spans = [
            JTBD_LOOP_SPAN,
            JTBD_SCENARIO_SPAN,
            JTBD_SCENARIO_CONTRACT_CLOSURE_SPAN,
            JTBD_SCENARIO_DEAL_PROGRESSION_SPAN,
            JTBD_SCENARIO_ICP_QUALIFICATION_SPAN,
            JTBD_SCENARIO_OUTREACH_SEQUENCE_EXECUTION_SPAN,
            JTBD_SCENARIO_PROCESS_INTELLIGENCE_QUERY_SPAN,
            JTBD_SCENARIO_RETROFIT_COMPLEXITY_SCORING_SPAN,
            JTBD_DMAIC_PHASE_SPAN,
        ];
        for span in &spans {
            assert!(!span.is_empty(), "JTBD span must not be empty: {span:?}");
            assert!(
                span.contains('.'),
                "JTBD span '{span}' must use dotted namespace format"
            );
            assert!(
                span.starts_with("jtbd."),
                "JTBD span '{span}' must start with 'jtbd.'"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// No duplicate span names across domains (regression guard)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod span_name_uniqueness {
    use pm4py::semconv::a2a_span_names::*;
    use pm4py::semconv::healing_span_names::*;
    use pm4py::semconv::jtbd_span_names::*;
    use pm4py::semconv::mcp_span_names::*;
    use pm4py::semconv::yawl_span_names::*;
    use std::collections::HashSet;

    #[test]
    fn healing_yawl_jtbd_mcp_a2a_span_names_are_globally_unique() {
        let healing = vec![
            HEALING_DIAGNOSIS_SPAN,
            HEALING_REFLEX_ARC_SPAN,
            HEALING_FINGERPRINT_SPAN,
            HEALING_ESCALATION_SPAN,
        ];
        let yawl = vec![YAWL_CASE_SPAN, YAWL_TASK_EXECUTION_SPAN];
        let jtbd = vec![
            JTBD_LOOP_SPAN,
            JTBD_SCENARIO_SPAN,
            JTBD_SCENARIO_ICP_QUALIFICATION_SPAN,
        ];
        let mcp = vec![
            MCP_CALL_SPAN,
            MCP_TOOL_EXECUTE_SPAN,
            MCP_CONNECTION_ESTABLISH_SPAN,
            MCP_REGISTRY_DISCOVER_SPAN,
        ];
        let a2a = vec![
            A2A_CALL_SPAN,
            A2A_TASK_DELEGATE_SPAN,
            A2A_NEGOTIATE_SPAN,
            A2A_CAPABILITY_REGISTER_SPAN,
        ];

        let mut all: Vec<&str> = healing;
        all.extend(yawl);
        all.extend(jtbd);
        all.extend(mcp);
        all.extend(a2a);

        let unique: HashSet<&str> = all.iter().copied().collect();
        assert_eq!(
            all.len(),
            unique.len(),
            "Duplicate span names found across healing/yawl/jtbd/mcp/a2a domains"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MCP spans
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod mcp_span_coverage {
    use pm4py::semconv::mcp_span_names::*;

    #[test]
    fn mcp_call_span_name_matches_semconv() {
        assert_eq!(MCP_CALL_SPAN, "mcp.call");
    }

    #[test]
    fn mcp_tool_execute_span_name_matches_semconv() {
        assert_eq!(MCP_TOOL_EXECUTE_SPAN, "mcp.tool_execute");
    }

    #[test]
    fn all_mcp_span_constants_are_non_empty_dotted_strings_with_mcp_prefix() {
        let spans = [
            MCP_CALL_SPAN,
            MCP_CONNECTION_ESTABLISH_SPAN,
            MCP_CONNECTION_POOL_ACQUIRE_SPAN,
            MCP_REGISTRY_DISCOVER_SPAN,
            MCP_RESOURCE_READ_SPAN,
            MCP_SERVER_HEALTH_CHECK_SPAN,
            MCP_SERVER_METRICS_COLLECT_SPAN,
            MCP_TOOL_ANALYTICS_RECORD_SPAN,
            MCP_TOOL_CACHE_LOOKUP_SPAN,
            MCP_TOOL_COMPOSE_SPAN,
            MCP_TOOL_DEPRECATE_SPAN,
            MCP_TOOL_RETRY_SPAN,
            MCP_TOOL_TIMEOUT_SPAN,
            MCP_TOOL_VALIDATE_SPAN,
            MCP_TOOL_VERSION_CHECK_SPAN,
            MCP_TOOL_EXECUTE_SPAN,
            MCP_TRANSPORT_CONNECT_SPAN,
        ];
        assert_eq!(spans.len(), 17, "expected 17 MCP span constants");
        for span in &spans {
            assert!(
                !span.is_empty(),
                "MCP span name must not be empty: {span:?}"
            );
            assert!(
                span.contains('.'),
                "MCP span name must contain a dot (dotted namespace): {span:?}"
            );
            assert!(
                span.starts_with("mcp."),
                "MCP span name must start with 'mcp.': {span:?}"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// A2A spans
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod a2a_span_coverage {
    use pm4py::semconv::a2a_span_names::*;

    #[test]
    fn a2a_call_span_name_matches_semconv() {
        assert_eq!(A2A_CALL_SPAN, "a2a.call");
    }

    #[test]
    fn a2a_task_delegate_span_name_matches_semconv() {
        assert_eq!(A2A_TASK_DELEGATE_SPAN, "a2a.task.delegate");
    }

    #[test]
    fn all_a2a_span_constants_are_non_empty_dotted_strings_with_a2a_prefix() {
        let spans = [
            A2A_AUCTION_RUN_SPAN,
            A2A_BID_EVALUATE_SPAN,
            A2A_CALL_SPAN,
            A2A_CAPABILITY_MATCH_SPAN,
            A2A_CAPABILITY_NEGOTIATE_SPAN,
            A2A_CAPABILITY_REGISTER_SPAN,
            A2A_CONTRACT_AMEND_SPAN,
            A2A_CONTRACT_DISPUTE_SPAN,
            A2A_CONTRACT_EXECUTE_SPAN,
            A2A_CONTRACT_NEGOTIATE_SPAN,
            A2A_CREATE_DEAL_SPAN,
            A2A_DEAL_STATUS_TRANSITION_SPAN,
            A2A_DISPUTE_RESOLVE_SPAN,
            A2A_ESCROW_CREATE_SPAN,
            A2A_ESCROW_RELEASE_SPAN,
            A2A_KNOWLEDGE_TRANSFER_SPAN,
            A2A_MESSAGE_BATCH_SPAN,
            A2A_MESSAGE_ROUTE_SPAN,
            A2A_NEGOTIATE_SPAN,
            A2A_NEGOTIATION_STATE_TRANSITION_SPAN,
            A2A_PENALTY_APPLY_SPAN,
            A2A_PROTOCOL_NEGOTIATE_SPAN,
            A2A_REPUTATION_DECAY_SPAN,
            A2A_REPUTATION_UPDATE_SPAN,
            A2A_SLA_CHECK_SPAN,
            A2A_SLO_EVALUATE_SPAN,
            A2A_TASK_DELEGATE_SPAN,
            A2A_TRUST_EVALUATE_SPAN,
            A2A_TRUST_FEDERATE_SPAN,
        ];
        assert_eq!(spans.len(), 29, "expected 29 A2A span constants");
        for span in &spans {
            assert!(
                !span.is_empty(),
                "A2A span name must not be empty: {span:?}"
            );
            assert!(
                span.contains('.'),
                "A2A span name must contain a dot (dotted namespace): {span:?}"
            );
            assert!(
                span.starts_with("a2a."),
                "A2A span name must start with 'a2a.': {span:?}"
            );
        }
    }
}
