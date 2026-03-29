# pm4py-rust vs pm4py Python: Comprehensive Parity Gap Analysis

**Date:** 2026-03-24
**Python source:** `/tmp/pm4py-python/` (latest main branch)
**Rust source:** `/Users/sac/chatmangpt/pm4py-rust/`

---

## Summary

| Category | Python Functions | Rust Implemented | Missing | Parity % |
|----------|-----------------|-----------------|---------|----------|
| **discovery** | 25 | 9 | 16 | 36% |
| **conformance** | 19 | 6 | 13 | 32% |
| **filtering** | 38 | 15 | 23 | 39% |
| **statistics** | 23 | 12 | 11 | 52% |
| **read (I/O)** | 13 | 3 | 10 | 23% |
| **write (I/O)** | 13 | 3 | 10 | 23% |
| **utils** | 10 | 3 | 7 | 30% |
| **org** | 6 | 2 | 4 | 33% |
| **analysis** | 15 | 0 | 15 | 0% |
| **convert** | 11 | 0 | 11 | 0% |
| **sim** | 2 | 0 | 2 | 0% |
| **ml** | 7 | 0 | 7 | 0% |
| **ocel** | 20 | 3 | 17 | 15% |
| **visualization** | 26 | 0 | 26 | 0% |
| **TOTAL** | **228** | **56** | **172** | **25%** |

---

## 1. DISCOVERY (25 Python → 9 Rust)

### Present in Rust ✅
| Python Function | Rust Location | Notes |
|----------------|---------------|-------|
| `discover_dfg` | `discovery::graphs::directly_follows_graph` | Returns DFG, but missing start/end activities dict |
| `discover_directly_follows_graph` | Same as above | Alias for discover_dfg |
| `discover_performance_dfg` | `discovery::graphs::discover_performance_dfg` | Has mean/min/max/median, missing start/end activities |
| `discover_petri_net_alpha` | `discovery::alpha_miner::AlphaMiner` | Simplified implementation |
| `discover_petri_net_alpha_plus` | `discovery::alpha_miner::AlphaPlusMiner` | Present |
| `discover_process_tree_inductive` | `discovery::inductive_miner::TreeMiner` | Simplified (sequence-only fallback) |
| `discover_petri_net_heuristics` | `discovery::heuristics_miner::HeuristicsMiner` | Present |
| `discover_eventually_follows_graph` | `discovery::graphs::eventually_follows_graph` | Present |
| `discover_footprints` | `conformance::footprints::FootprintsConformanceChecker` | Partial — log footprints only |
| `discover_log_skeleton` | `discovery::log_skeleton::LogSkeletonMiner` | Present |
| `discover_temporal_profile` | `statistics::temporal_profile` | Present |

### Missing in Rust ❌
| Python Function | Description | Priority |
|----------------|-------------|----------|
| `discover_petri_net_inductive` | Inductive Miner → Petri net (converts tree to net) | HIGH |
| `discover_petri_net_ilp` | ILP Miner discovery | MEDIUM |
| `discover_petri_net_genetic` | Genetic Miner discovery | LOW |
| `discover_bpmn_inductive` | Inductive Miner → BPMN | HIGH |
| `discover_heuristics_net` | Heuristics Net object (not Petri net) | MEDIUM |
| `discover_dfg_typed` | Typed DFG object (DFG class) | MEDIUM |
| `discover_transition_system` | Transition System discovery | LOW |
| `discover_prefix_tree` | Prefix Tree (Trie) discovery | LOW |
| `discover_declare` | DECLARE declarative model discovery | HIGH |
| `discover_powl` | POWL model discovery | LOW |
| `discover_batches` | Batch detection in logs | MEDIUM |
| `derive_minimum_self_distance` | Minimum self-distance per activity | MEDIUM |
| `correlation_miner` | Correlation miner (no case ID) | LOW |
| `discover_otg` | Object Type Graph (OCEL) | LOW |
| `discover_etot` | Event Type - Object Type graph | LOW |

---

## 2. CONFORMANCE (19 Python → 6 Rust)

### Present in Rust ✅
| Python Function | Rust Location | Notes |
|----------------|---------------|-------|
| `conformance_diagnostics_token_based_replay` | `conformance::token_replay` | Present |
| `conformance_diagnostics_alignments` | `conformance::alignments` | Present |
| `conformance_diagnostics_footprints` | `conformance::footprints` | Present |
| `conformance_temporal_profile` | `conformance::temporal_profile` | Present |
| `conformance_log_skeleton` | `discovery::log_skeleton` | Present (in discovery module) |
| `check_is_fitting` | `conformance` module | Partial |

### Missing in Rust ❌
| Python Function | Description | Priority |
|----------------|-------------|----------|
| `fitness_token_based_replay` | Aggregate fitness score (TBR) | HIGH |
| `fitness_alignments` | Aggregate fitness score (alignments) | HIGH |
| `precision_token_based_replay` | Precision metric (TBR) | HIGH |
| `precision_alignments` | Precision metric (alignments) | HIGH |
| `fitness_footprints` | Fitness metric (footprints) | MEDIUM |
| `precision_footprints` | Precision metric (footprints) | MEDIUM |
| `generalization_tbr` | Generalization metric | MEDIUM |
| `replay_prefix_tbr` | Replay a prefix on a Petri net | MEDIUM |
| `conformance_declare` | DECLARE conformance checking | HIGH |
| `conformance_ocdfg` | OC-DFG conformance (OCEL) | LOW |
| `conformance_otg` | OTG conformance (OCEL) | LOW |
| `conformance_etot` | ET-OT conformance (OCEL) | LOW |

---

## 3. FILTERING (38 Python → 15 Rust)

### Present in Rust ✅
| Python Function | Rust Location |
|----------------|---------------|
| `filter_start_activities` | `statistics::log_stats` |
| `filter_end_activities` | `statistics::log_stats` |
| `filter_event_attribute_values` | `log::filtering` |
| `filter_trace_attribute_values` | `log::filtering` |
| `filter_directly_follows_relation` | `log::filtering` |
| `filter_eventually_follows_relation` | `log::filtering` |
| `filter_time_range` | `log::filtering` |
| `filter_between` | `log::filtering` |
| `filter_case_size` | `log::filtering` |
| `filter_case_performance` | `log::filtering` |
| `filter_activities_rework` | `log::filtering` |
| `filter_paths_performance` | `log::filtering` |
| `filter_variants_top_k` | `log::filtering` |
| `filter_four_eyes_principle` | `log::filtering` |
| `filter_activity_done_different_resources` | `log::filtering` |

### Missing in Rust ❌
| Python Function | Description | Priority |
|----------------|-------------|----------|
| `filter_log_relative_occurrence_event_attribute` | Filter by relative frequency | MEDIUM |
| `filter_variants` | Filter by specific variants | HIGH |
| `filter_variants_by_coverage_percentage` | Filter variants by coverage | MEDIUM |
| `filter_prefixes` | Keep prefixes up to activity | MEDIUM |
| `filter_suffixes` | Keep suffixes from activity | MEDIUM |
| `filter_trace_segments` | Filter by trace segments with wildcards | MEDIUM |
| `filter_ocel_event_attribute` | OCEL event attribute filter | LOW |
| `filter_ocel_object_attribute` | OCEL object attribute filter | LOW |
| `filter_ocel_object_types_allowed_activities` | OCEL activity-type matching | LOW |
| `filter_ocel_object_per_type_count` | OCEL object count filter | LOW |
| `filter_ocel_start_events_per_object_type` | OCEL start events filter | LOW |
| `filter_ocel_end_events_per_object_type` | OCEL end events filter | LOW |
| `filter_ocel_events_timestamp` | OCEL timestamp filter | LOW |
| `filter_ocel_events` | OCEL event ID filter | LOW |
| `filter_ocel_objects` | OCEL object ID filter | LOW |
| `filter_ocel_object_types` | OCEL object type filter | LOW |
| `filter_ocel_cc_object` | OCEL connected component filter | LOW |
| `filter_ocel_cc_length` | OCEL CC length filter | LOW |
| `filter_ocel_cc_otype` | OCEL CC object type filter | LOW |
| `filter_ocel_cc_activity` | OCEL CC activity filter | LOW |
| `filter_ocel_activities_connected_object_type` | OCEL activity-object type filter | LOW |
| `filter_dfg_activities_percentage` | DFG activity percentage filter | MEDIUM |
| `filter_dfg_paths_percentage` | DFG path percentage filter | MEDIUM |

---

## 4. STATISTICS (23 Python → 12 Rust)

### Present in Rust ✅
| Python Function | Rust Location |
|----------------|---------------|
| `get_start_activities` | `statistics::log_stats` |
| `get_end_activities` | `statistics::log_stats` |
| `get_event_attributes` | `log::EventLog` methods |
| `get_event_attribute_values` | `statistics::log_stats` |
| `get_trace_attributes` | `log::Trace` methods |
| `get_trace_attribute_values` | `statistics::log_stats` |
| `get_variants` | `statistics::log_stats` |
| `get_case_arrival_average` | `statistics::log_stats` |
| `get_all_case_durations` | `statistics::log_stats` |
| `get_case_overlap` | `statistics::log_stats` |
| `get_cycle_time` | `statistics::log_stats` |
| `get_variants_paths_duration` | `statistics::log_stats` |
| `get_rework_cases_per_activity` | `statistics::log_stats` |

### Missing in Rust ❌
| Python Function | Description | Priority |
|----------------|-------------|----------|
| `get_variants_as_tuples` | Variants as tuples | LOW |
| `get_process_cube` | Process cube computation | LOW |
| `get_minimum_self_distances` | Min self-distances | MEDIUM |
| `get_minimum_self_distance_witnesses` | MSD witnesses | LOW |
| `get_frequent_trace_segments` | Frequent trace segments | MEDIUM |
| `get_case_duration` | Single case duration | LOW |
| `get_activity_position_summary` | Activity position in trace | MEDIUM |
| `get_stochastic_language` | Stochastic map from log | HIGH |
| `split_by_process_variant` | Split log by variant | LOW |
| `get_service_time` | Service time statistics | MEDIUM |

---

## 5. READ / I/O (13 Python → 3 Rust)

### Present in Rust ✅
| Python Function | Rust Location |
|----------------|---------------|
| `read_xes` | `io::XESReader` |
| `read_pnml` | `io::PNMLReader` |
| `read_ptml` | `io::PTMLReader` |

### Missing in Rust ❌
| Python Function | Description | Priority |
|----------------|-------------|----------|
| `read_dfg` | Read DFG file | MEDIUM |
| `read_bpmn` | Read BPMN file | MEDIUM |
| `read_ocel` | Read OCEL (auto-detect format) | LOW |
| `read_ocel_csv` | Read OCEL from CSV | LOW |
| `read_ocel_xml` | Read OCEL from XML | LOW |
| `read_ocel_json` | Read OCEL from JSON | LOW |
| `read_ocel_sqlite` | Read OCEL from SQLite | LOW |
| `read_ocel2` | Read OCEL2 (auto-detect) | LOW |
| `read_ocel2_sqlite` | Read OCEL2 from SQLite | LOW |
| `read_ocel2_json` | Read OCEL2 from JSON | LOW |
| `read_ocel2_xml` | Read OCEL2 from XML | LOW |

---

## 6. WRITE / I/O (13 Python → 3 Rust)

### Present in Rust ✅
| Python Function | Rust Location |
|----------------|---------------|
| `write_xes` | `io::XESWriter` |
| `write_pnml` | `io::PNMLWriter` |
| `write_ptml` | `io::PTMLWriter` |

### Missing in Rust ❌
| Python Function | Description | Priority |
|----------------|-------------|----------|
| `write_dfg` | Write DFG file | MEDIUM |
| `write_bpmn` | Write BPMN file | MEDIUM |
| `write_ocel` | Write OCEL (auto-detect) | LOW |
| `write_ocel_json` | Write OCEL JSON | LOW |
| `write_ocel_csv` | Write OCEL CSV | LOW |
| `write_ocel_xml` | Write OCEL XML | LOW |
| `write_ocel_sqlite` | Write OCEL SQLite | LOW |
| `write_ocel2` | Write OCEL2 (auto-detect) | LOW |
| `write_ocel2_sqlite` | Write OCEL2 SQLite | LOW |
| `write_ocel2_xml` | Write OCEL2 XML | LOW |
| `write_ocel2_json` | Write OCEL2 JSON | LOW |

---

## 7. UTILS (10 Python → 3 Rust)

### Present in Rust ✅
| Python Function | Rust Location |
|----------------|---------------|
| `serialize` | `utils::serialization` |
| `deserialize` | `utils::serialization` |
| `parse_process_tree` | `models::process_tree` |

### Missing in Rust ❌
| Python Function | Description | Priority |
|----------------|-------------|----------|
| `format_dataframe` | Format DataFrame columns | N/A (no DataFrame) |
| `set_classifier` | Set log classifier | LOW |
| `parse_event_log_string` | Parse log from string | MEDIUM |
| `project_on_event_attribute` | Project log on attribute | MEDIUM |
| `sample_cases` | Sample cases from log | MEDIUM |
| `sample_events` | Sample events from log | MEDIUM |
| `rebase` | Rebase timestamps | LOW |
| `parse_powl_model_string` | Parse POWL from string | LOW |

---

## 8. ORG (6 Python → 2 Rust)

### Present in Rust ✅
| Python Function | Rust Location |
|----------------|---------------|
| `discover_subcontracting_network` | `statistics::organizational` |
| (partial) handover of work | `statistics::organizational` |

### Missing in Rust ❌
| Python Function | Description | Priority |
|----------------|-------------|----------|
| `discover_handover_of_work_network` | SNA handover network | MEDIUM |
| `discover_activity_based_resource_similarity` | Resource similarity | LOW |
| `discover_working_together_network` | Working together network | MEDIUM |
| `discover_organizational_roles` | Role discovery | MEDIUM |
| `discover_network_analysis` | Network analysis metrics | LOW |

---

## 9. ANALYSIS (15 Python → 0 Rust) — COMPLETELY MISSING

| Python Function | Description | Priority |
|----------------|-------------|----------|
| `cluster_log` | Log clustering | LOW |
| `check_soundness` | Petri net soundness check | HIGH |
| `compute_emd` | Earth Mover's Distance | LOW |
| `solve_marking_equation` | Marking equation solver | MEDIUM |
| `solve_extended_marking_equation` | Extended marking equation | MEDIUM |
| `construct_synchronous_product_net` | Sync product net | MEDIUM |
| `insert_artificial_start_end` | Add start/end to model | MEDIUM |
| `check_is_workflow_net` | Workflow net validation | HIGH |
| `maximal_decomposition` | Maximal decomposition | LOW |
| `generate_marking` | Generate marking from string | LOW |
| `reduce_petri_net_invisibles` | Reduce invisible transitions | MEDIUM |
| `reduce_petri_net_implicit_places` | Reduce implicit places | MEDIUM |
| `get_enabled_transitions` | Get enabled transitions | MEDIUM |
| `simplicity_petri_net` | Simplicity metric | MEDIUM |
| `behavioral_similarity` | Behavioral similarity | LOW |
| `structural_similarity` | Structural similarity | LOW |
| `embeddings_similarity` | Embedding-based similarity | LOW |
| `get_activity_labels` | Get model activity labels | MEDIUM |
| `replace_activity_labels` | Replace labels in model | MEDIUM |
| `label_sets_similarity` | Label set similarity | LOW |
| `map_labels_from_second_model` | Map labels between models | LOW |

---

## 10. CONVERT (11 Python → 0 Rust) — COMPLETELY MISSING

| Python Function | Description | Priority |
|----------------|-------------|----------|
| `convert_to_event_log` | Convert to EventLog | MEDIUM |
| `convert_to_event_stream` | Convert to EventStream | LOW |
| `convert_to_dataframe` | Convert to DataFrame | N/A |
| `convert_to_bpmn` | Convert model to BPMN | HIGH |
| `convert_to_petri_net` | Convert model to Petri net | HIGH |
| `convert_to_process_tree` | Convert model to ProcessTree | HIGH |
| `convert_to_reachability_graph` | Reachability graph | LOW |
| `convert_log_to_ocel` | Log to OCEL | LOW |
| `convert_ocel_to_networkx` | OCEL to NetworkX | N/A |
| `convert_log_to_networkx` | Log to NetworkX | N/A |
| `convert_log_to_time_intervals` | Log to time intervals | LOW |
| `convert_petri_net_to_networkx` | Petri net to NetworkX | N/A |
| `convert_petri_net_type` | Convert Petri net type | LOW |
| `convert_to_powl` | Convert to POWL | LOW |

---

## 11. SIM (2 Python → 0 Rust) — COMPLETELY MISSING

| Python Function | Description | Priority |
|----------------|-------------|----------|
| `play_out` | Playout of Petri net / DFG / tree | MEDIUM |
| `generate_process_tree` | Generate random process tree | LOW |

---

## 12. ML (7 Python → 0 Rust) — COMPLETELY MISSING

| Python Function | Description | Priority |
|----------------|-------------|----------|
| `split_train_test` | Train/test split | MEDIUM |
| `get_prefixes_from_log` | Get prefixes for ML | MEDIUM |
| `extract_ocel_features` | OCEL feature extraction | LOW |
| `extract_features_dataframe` | Feature extraction from log | MEDIUM |
| `extract_temporal_features_dataframe` | Temporal features | MEDIUM |
| `extract_outcome_enriched_dataframe` | Outcome features | MEDIUM |
| `extract_target_vector` | Target vector extraction | MEDIUM |

---

## 13. OCEL (20 Python → 3 Rust) — MOSTLY MISSING

### Present in Rust ✅
| Python Function | Rust Location |
|----------------|---------------|
| `ocel_get_object_types` | `ocpm::ObjectType` |
| `ocel_get_attribute_names` | `ocpm` module |
| `discover_oc_petri_net` | `ocpm::OCPMDiscoveryMiner` |

### Missing in Rust ❌
| Python Function | Description | Priority |
|----------------|-------------|----------|
| `ocel_objects_interactions_summary` | Object interactions summary | LOW |
| `ocel_temporal_summary` | Temporal summary | LOW |
| `ocel_objects_summary` | Objects summary | LOW |
| `ocel_flattening` | OCEL to flat log | MEDIUM |
| `ocel_object_type_activities` | Activities per object type | LOW |
| `ocel_objects_ot_count` | Object count per type | LOW |
| `discover_ocdfg` | OC-DFG discovery | LOW |
| `discover_objects_graph` | Objects graph discovery | LOW |
| `sample_ocel_objects` | Sample OCEL objects | LOW |
| `ocel_drop_duplicates` | Drop duplicate events | LOW |
| `ocel_merge_duplicates` | Merge duplicate events | LOW |
| `ocel_sort_by_additional_column` | Sort OCEL | LOW |
| `ocel_add_index_based_timedelta` | Add timedelta column | LOW |
| `sample_ocel_connected_components` | Sample connected components | LOW |
| `ocel_o2o_enrichment` | Object-to-object enrichment | LOW |
| `ocel_e2o_lifecycle_enrichment` | Event-to-object enrichment | LOW |
| `cluster_equivalent_ocel` | Cluster equivalent OCEL | LOW |

---

## 14. VISUALIZATION (26 Python → 0 Rust) — COMPLETELY MISSING

All 26 visualization functions are missing. Rust could generate SVG/HTML output.

| Python Function | Description |
|----------------|-------------|
| `view_petri_net` | View Petri net |
| `save_vis_petri_net` | Save Petri net visualization |
| `view_dfg` | View DFG |
| `save_vis_dfg` | Save DFG visualization |
| `view_process_tree` | View process tree |
| `save_vis_process_tree` | Save process tree visualization |
| `view_ocdfg` | View OC-DFG |
| `save_vis_ocdfg` | Save OC-DFG visualization |
| `view_heuristics_net` | View heuristics net |
| `save_vis_heuristics_net` | Save heuristics net visualization |
| `view_bpmn` | View BPMN |
| `save_vis_bpmn` | Save BPMN visualization |
| `view_sna` | View SNA |
| `save_vis_sna` | Save SNA visualization |
| `view_dotted_chart` | View dotted chart |
| `save_vis_dotted_chart` | Save dotted chart |
| `view_performance_spectrum` | View performance spectrum |
| `save_vis_performance_spectrum` | Save performance spectrum |
| `view_case_duration_graph` | View case duration graph |
| `view_events_per_time_graph` | View events per time |
| `save_vis_case_duration_graph` | Save case duration graph |
| `save_vis_events_per_time_graph` | Save events per time |
| `view_events_distribution_graph` | View events distribution |
| `save_vis_events_distribution_graph` | Save events distribution |
| `view_performance_dfg` | View performance DFG |
| `save_vis_performance_dfg` | Save performance DFG |
| `view_ocpn` | View object-centric Petri net |
| `save_vis_ocpn` | Save OCPN visualization |
| `view_network_analysis` | View network analysis |
| `save_vis_network_analysis` | Save network analysis |
| `view_transition_system` | View transition system |
| `save_vis_transition_system` | Save transition system |
| `view_prefix_tree` | View prefix tree |
| `save_vis_prefix_tree` | Save prefix tree |
| `view_object_graph` | View object graph |
| `save_vis_object_graph` | Save object graph |
| `view_alignments` | View alignments |
| `save_vis_alignments` | Save alignments |
| `view_footprints` | View footprints |
| `save_vis_footprints` | Save footprints |
| `view_powl` | View POWL |
| `save_vis_powl` | Save POWL |

---

## Priority Roadmap

### Phase 1: Core Gaps (HIGH priority, 30 functions)
These are the most commonly-used process mining functions:

1. **Conformance metrics** — `fitness_*`, `precision_*`, `generalization_tbr`
2. **convert_to_petri_net`, `convert_to_process_tree`, `convert_to_bpmn` — Model conversions
3. **check_is_workflow_net**, `check_soundness` — Model validation
4. **discover_petri_net_inductive** — Full inductive miner (tree → net conversion)
5. **discover_bpmn_inductive** — BPMN from inductive miner
6. **discover_declare** + `conformance_declare` — DECLARE model
7. **filter_variants`** — Variant filtering
8. **get_stochastic_language`** — Stochastic map

### Phase 2: Important Gaps (MEDIUM priority, ~70 functions)
9. Remaining filtering (prefixes, suffixes, trace segments, DFG filters)
10. Remaining statistics (MSD, frequent segments, service time)
11. Remaining I/O (DFG, BPMN read/write)
12. Remaining discovery (transition system, prefix tree, batches)
13. Remaining org (SNA networks, role discovery)
14. Remaining OCEL (flattening, OC-DFG)
15. Analysis helpers (marking equation, sync product net, reduction)
16. ML features (split, feature extraction)

### Phase 3: Advanced (LOW priority, ~72 functions)
17. All OCEL 2.0 (read_ocel2_*, write_ocel2_*)
18. All visualization (26 functions)
19. Simulation (play_out, generate_process_tree)
20. Correlation miner, POWL, genetic miner
21. NetworkX conversions (not applicable to Rust — use petgraph)

### Phase 4: Rust-Only Innovations (not in Python pm4py)
These exist in Rust but not Python:
- `predictive` module (7 types for next activity, outcome, remaining time prediction)
- `performance` module (extended performance analysis)
- `parity_verification` module
- `remaining_parity` module
- Signal Theory integration
