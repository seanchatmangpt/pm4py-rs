# pm4py Python Test File Inventory & Porting Status

**Date:** 2026-03-24
**Python source:** `/tmp/pm4py-python/tests/` (59 test files)
**Rust ported tests:** `tests/pm4py_python_ported_tests.rs` (91 tests passing)

---

## Summary

| Category | Test Files | Test Functions | Ported to Rust | Porting % |
|----------|-----------|---------------|----------------|-----------|
| **discovery** | 12 | 68 | 24 | 35% |
| **conformance** | 7 | 42 | 18 | 43% |
| **filtering** | 5 | 39 | 15 | 38% |
| **statistics** | 4 | 33 | 12 | 36% |
| **I/O (read/write)** | 5 | 35 | 12 | 34% |
| **OCEL** | 4 | 35 | 0 | 0% |
| **SNA/org** | 3 | 16 | 4 | 25% |
| **analysis/convert** | 5 | 30 | 3 | 10% |
| **simulation** | 2 | 4 | 0 | 0% |
| **visualization** | 3 | 8 | 0 | 0% |
| **CLI/misc** | 3 | 3 | 0 | 0% |
| **TOTAL** | **53** | **313** | **88** | **28%** |

**Rust tests passing:** 408 (lib) + 91 (ported) + 74 (other) = **573 total**

---

## Test Data Files Available

### In pm4py-rust `test_data/` (8 files)
- `running-example.xes` — 6 traces, 46 events
- `running-example.csv` — same data as XES
- `running-example.pnml` — Alpha miner Petri net
- `running-example.ptml` — Inductive miner process tree
- `running-example.bpmn` — BPMN model
- `running-example.dfg` — Directly-follows graph
- `receipt.xes` — 2338 traces, receipt process
- `roadtraffic100traces.xes` — 100 traces, road traffic

### Available in Python but NOT in Rust test_data/ (22+ files)
- `interval_event_log.xes` / `interval_event_log.csv` — Service time tests
- `receipt.csv` — Receipt as CSV
- `roadtraffic100traces.csv` — Road traffic as CSV
- `reviewing.xes` / `reviewing.csv` — Reviewing process
- `receipt.bpmn` — Receipt BPMN
- `murata1.pnml`, `murata2.pnml`, `murata3.pnml` — Implicit place reduction
- `ocel/example_log.csv` / `.jsonocel` / `.xmlocel` — OCEL data
- `ocel/newocel.sqlite` / `.jsonocel` — New OCEL format
- `ocel/ocel20_example.*` — OCEL 2.0 data
- `recruiting-red.jsonocel` — OCEL recruiting data
- `compressed_input_data/*.xes.gz` — 19 compressed logs (50K-1T traces each)
- `problematic/` — Edge case XES files

---

## Detailed Test File Inventory

### 1. alignment_test.py — 10 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_alignment_alpha` | Alignment on Alpha miner net, verify traces fit | running-example.xes | YES |
| `test_alignment_pnml` | Alignment on inductive miner PNML net | running-example.xes | YES |
| `test_tree_align_receipt` | Process tree alignment on receipt with noise=0.2 | receipt.xes | YES |
| `test_tree_align_reviewing` | Process tree alignment on reviewing log | 04_reviewing.xes.gz | NO |
| `test_tree_align_reviewing_classifier` | Process tree alignment with custom classifier | 04_reviewing.xes.gz | NO |
| `test_tree_align3` | DP variant of process tree alignment | 01_running-example.xes.gz | NO |
| `test_tree_align3_mip` | MILP variant of alignment (skipped in pipeline) | 01_running-example.xes.gz | NO |
| `test_variant_state_eq_a_star` | State equation A* alignment variant | running-example.xes | NO |
| `test_variant_dijkstra_less_memory` | Dijkstra less-memory alignment | running-example.xes | NO |

### 2. alpha_test.py — 3 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_applyAlphaMinerToXES` | Alpha miner on XES, compare nets, export PNML, token replay | running-example.xes | YES |
| `test_applyAlphaMinerToCSV` | Alpha miner on CSV, compare nets, export PNML | running-example.csv | YES |
| `test_alphaMinerVisualizationFromXES` | Alpha miner with Graphviz visualization | running-example.xes | NO (no viz) |

### 3. algorithm_test.py — 14 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_importing_xes` | Import XES with multiple variants (ITERPARSE, LINE_BY_LINE, etc.) | running-example.xes | YES |
| `test_log_skeleton` | Discover log skeleton and conformance | running-example.xes | YES |
| `test_alignment` | Alignment with A* state eq and Dijkstra, fitness/precision | running-example.xes | YES |
| `test_decomp_alignment` | Decomposed alignment with RECOMPOS_MAXIMAL | running-example.xes | NO |
| `test_tokenreplay` | TBR forward/backward, fitness, precision, generalization | running-example.xes | YES |
| `test_evaluation` | Simplicity and overall evaluation on alpha miner | running-example.xes | NO |
| `test_playout` | Petri net playout simulation | running-example.xes | NO |
| `test_tree_generation` | Generate process trees (BASIC, PTANDLOGGENERATOR) | no data file | NO |
| `test_alpha_miner_log` | Alpha miner classic/plus, DFG-to-alpha | running-example.xes | YES |
| `test_alpha_miner_dataframe` | Alpha miner on CSV dataframe | running-example.csv | YES |
| `test_tsystem` | Transition system discovery | running-example.xes | NO |
| `test_inductive_miner` | Inductive miner tree discovery + Petri net conversion | running-example.xes | YES |
| `test_performance_spectrum` | Performance spectrum from XES and CSV | running-example.xes, .csv | NO |

### 4. bpmn_tests.py — 5 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_tree_to_bpmn` | Convert inductive tree to BPMN | running-example.xes | NO |
| `test_bpmn_to_petri_net` | Import BPMN, convert to Petri net, TBR fitness | running-example.xes, .bpmn | NO |
| `test_bpmn_layouting` | Discover tree, convert to BPMN, layout | running-example.xes | NO |
| `test_bpmn_exporting` | Import and re-export BPMN | running-example.bpmn | NO |
| `test_bpmn_importing_and_layouting` | Import BPMN and apply layout | running-example.bpmn | NO |

### 5. csv_impexp_test.py — 2 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_importExportCSVtoXES` | CSV → event log → XES → re-import, verify trace count | running-example.csv | YES |
| `test_importExportCSVtoCSV` | CSV → event log → CSV → re-import, verify trace count | running-example.csv | NO |

### 6. dec_tree_test.py — 2 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_decisiontree_evattrvalue` | Decision tree on event attribute value | roadtraffic50traces.xes | NO |
| `test_decisiontree_traceduration` | Decision tree on trace duration | roadtraffic50traces.xes | NO |

### 7. dfg_tests.py — 2 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_filter_act_percentage` | Filter DFG by activity percentage (0.1) | running-example.xes | YES |
| `test_filter_paths_percentage` | Filter DFG by path percentage (0.3) | running-example.xes | YES |

### 8. diagn_df_conf_checking.py — 6 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_tbr_normal` | TBR conformance diagnostics dataframe | running-example.xes | YES |
| `test_tbr_backwards` | Backwards TBR diagnostics | running-example.xes | NO |
| `test_align` | Alignment diagnostics dataframe | running-example.xes | YES |
| `test_log_skeleton` | Log skeleton conformance diagnostics | running-example.xes | YES |
| `test_footprints_classic` | Footprints conformance (log_model variant) | running-example.xes | YES |
| `test_footprints_extensive` | Footprints conformance (trace_extensive) | running-example.xes | YES |

### 9. doc_tests.py — 57 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_1` | Basic XES import | running-example.xes | YES |
| `test_2` | XES import with ITERPARSE variant | running-example.xes | NO |
| `test_3` | CSV to event log conversion | running-example.csv | YES |
| `test_4` | CSV with custom case ID column | running-example.csv | NO |
| `test_5` | XES log export | running-example.xes | NO |
| `test_6` | XES to dataframe, export CSV | running-example.xes | NO |
| `test_8` | Timestamp filter traces contained (log) | running-example.xes | YES |
| `test_9` | Timestamp filter traces intersecting (df) | running-example.csv | NO |
| `test_10` | Timestamp filter traces intersecting (log) | running-example.xes | YES |
| `test_11` | Timestamp filter traces intersecting (df dup) | running-example.csv | NO |
| `test_12` | Timestamp filter events (log) | running-example.xes | YES |
| `test_13` | Timestamp filter events (df) | running-example.csv | NO |
| `test_14` | Case performance filter (86400-864000ms, log) | running-example.xes | YES |
| `test_15` | Case performance filter (86400-864000ms, df) | running-example.csv | NO |
| `test_22` | Get variants from log | running-example.xes | YES |
| `test_23` | Get variants from df | running-example.csv | NO |
| `test_24` | Variant statistics sorted (log) | running-example.xes | NO |
| `test_25` | Variant statistics sorted (df) | running-example.csv | NO |
| `test_26` | Filter keep variant (log, positive) | running-example.xes | YES |
| `test_27` | Filter keep variant (df, positive) | running-example.csv | NO |
| `test_28` | Filter remove variant (log, negative) | running-example.xes | YES |
| `test_29` | Filter remove variant (df, negative) | running-example.csv | NO |
| `test_32` | Get attribute values (log) | running-example.xes | YES |
| `test_33` | Get attribute values (df) | running-example.csv | NO |
| `test_34` | Attribute trace filtering (log) | receipt.xes | YES |
| `test_35` | Attribute trace filtering (df) | receipt.csv | NO |
| `test_38` | Numeric attribute filter (log, events+cases) | roadtraffic100traces.xes | NO |
| `test_39` | Numeric attribute filter (df, events+cases) | roadtraffic100traces.csv | NO |
| `test_40` | Alpha miner discovery (log) | running-example.xes | YES |
| `test_41` | Inductive miner discovery + tree viz (log) | running-example.xes | YES |
| `test_42` | Heuristics miner + viz (compressed XES) | 09_a32f0n00.xes.gz | NO |
| `test_43` | DFG discovery + frequency viz | running-example.xes | YES |
| `test_44` | DFG performance discovery + viz | running-example.xes | YES |
| `test_45` | DFG performance viz saved to SVG | running-example.xes | NO |
| `test_46` | DFG to Petri net conversion | running-example.xes | NO |
| `test_47` | Inductive net + frequency viz to PNG | running-example.xes | NO |
| `test_48` | Alpha miner with activity key param | running-example.xes | NO |
| `test_49` | Alpha miner with inserted classifier | receipt.xes | NO |
| `test_50` | Alpha miner with custom concatenated classifier | receipt.xes | NO |
| `test_51` | PNML import, export, enabled transitions | running-example.pnml | YES |
| `test_52` | Programmatically create Petri net, export | no data file | NO |
| `test_56` | Alignment conformance (apply_log) | running-example.xes | YES |
| `test_57` | Alignment with custom classifier + fitness | running-example.xes | NO |
| `test_58` | Alignment with custom model cost functions | running-example.xes | NO |
| `test_59` | Process tree generation + log simulation | no data file | NO |
| `test_60` | Decision tree on event attribute value | roadtraffic50traces.xes | NO |
| `test_61` | Decision tree on trace duration | roadtraffic50traces.xes | NO |
| `test_62` | ITERPARSE variant parameters access | no data file | NO |
| `test_63` | LINE_BY_LINE variant parameters access | no data file | NO |
| `test_64` | TO_EVENT_LOG converter parameters | no data file | NO |
| `test_65` | TO_EVENT_STREAM converter parameters | no data file | NO |
| `test_66` | TO_EVENT_STREAM converter (case prefix) | no data file | NO |
| `test_67` | XES exporter ETREE variant compress param | no data file | NO |
| `test_tbr_diagn_1` | TBR diagnostics duration + non-existing activities | receipt.xes | NO |
| `test_tbr_diagn_2` | TBR root cause analysis with decision tree | receipt.xes | NO |
| `test_max_decomp` | Maximal decomposition + visualization | running-example.xes | NO |
| `test_reach_graph` | Reachability graph + transition system viz | running-example.xes | NO |
| `test_decomp` | Decomposed alignments + fitness | running-example.xes | NO |
| `test_footprints` | Footprint discovery + conformance (log+tree) | running-example.xes, receipt.xes | YES |
| `test_log_skeleton` | Log skeleton discovery + conformance (2 noise) | running-example.xes, receipt.xes | YES |
| `test_throughput_time` | All case durations + median | running-example.xes | YES |
| `test_case_arrival` | Case arrival average + dispersion | running-example.xes | YES |
| `test_ps` | Performance spectrum between activities | running-example.xes | NO |
| `test_business_hours` | Business hours calculation | no data file | NO |
| `test_cycle_waiting_time` | Cycle and waiting time with interval lifecycle | receipt.xes | NO |
| `test_distr_case_duration` | KDE case duration distribution | receipt.xes | NO |
| `test_distr_num_attribute` | KDE numeric attribute distribution | roadtraffic100traces.xes | NO |
| `test_evaluation` | Full evaluation (fitness TBR/align, precision, gen, simp) | running-example.xes | NO |
| `test_sna` | SNA (handover, subcontracting, together, roles) + viz | running-example.xes | NO |
| `test_playout` | Petri net simulation (basic + extensive) | running-example.xes | NO |
| `test_ctmc` | CTMC stochastic analysis from DFG performance | running-example.xes | NO |

### 10. etc_tests.py — 1 test

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_etc1` | ETConformance precision (TBR-based) | running-example.xes | NO |

### 11. evaluation_tests.py — 5 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_evaluation_pm1` | Full evaluation (fitness, precision, generalization, simplicity) | running-example.xes | NO |
| `test_evaluation_pm2` | Unified evaluation algorithm | running-example.xes | NO |
| `test_simplicity_arc_degree` | Simplicity using arc degree variant | running-example.pnml | NO |
| `test_simplicity_extended_cardoso` | Simplicity using extended Cardoso | running-example.pnml | NO |
| `test_simplicity_extended_cyclomatic` | Simplicity using extended cyclomatic | running-example.pnml | NO |

### 12. filtering_log_test.py — 20 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_filtering_attributes_events` | Event-level attribute filter (positive) | running-example.xes | YES |
| `test_filtering_attributes_traces` | Trace-level attribute filter (positive) | running-example.xes | YES |
| `test_attribute_selection` | Select attributes for tree analysis | running-example.xes | NO |
| `test_filtering_variants` | Variant filter (positive+negative) | running-example.xes | YES |
| `test_obtaining_variants` | Get variant statistics | running-example.xes | YES |
| `test_casefilter_ncases` | Filter on number of cases | running-example.xes | NO |
| `test_casefilter_casesize` | Filter on case size (3-5 events) | running-example.xes | YES |
| `test_pathsfilter` | Path filter (positive+negative) | running-example.xes | YES |
| `test_AeventuallyB_pos` | LTL eventually-follows (positive) | running-example.xes | YES |
| `test_AeventuallyB_neg` | LTL eventually-follows (negative) | running-example.xes | YES |
| `test_AeventuallyBeventuallyC_pos` | LTL 3-activity chain (positive) | running-example.xes | NO |
| `test_AeventuallyBeventuallyC_neg` | LTL 3-activity chain (negative) | running-example.xes | NO |
| `test_AnextBnextC_pos` | LTL next-activity chain (positive) | running-example.xes | NO |
| `test_AnextBnextC_neg` | LTL next-activity chain (negative) | running-example.xes | NO |
| `test_fourEeyesPrinciple_pos` | Four-eyes principle (positive) | running-example.xes | YES |
| `test_fourEeyesPrinciple_neg` | Four-eyes principle (negative) | running-example.xes | YES |
| `test_attrValueDifferentPersons_pos` | Different persons check (positive) | running-example.xes | YES |
| `test_attrValueDifferentPersons_neg` | Different persons check (negative) | running-example.xes | YES |
| `test_attr_value_repetition` | Attribute value repetition (Sara) | running-example.xes | YES |
| `test_filter_traces_attribute_in_timeframe` | Filter traces with attribute in timeframe | running-example.xes | NO |

### 13. filtering_pandas_test.py — 19 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_prefiltering_dataframe` | Pre-filter df: keep activities, filter cases, sort | running-example.csv | NO |
| `test_filtering_variants` | Variant filter on df | running-example.csv | NO |
| `test_filtering_attr_events` | Event attribute filter on df (positive+negative) | running-example.csv | NO |
| `test_filtering_attr_values` | Attribute value filter with NaN (positive+negative+keep) | running-example.csv | NO |
| `test_filtering_paths` | Path filter on df (positive+negative) | running-example.csv | NO |
| `test_filtering_timeframe` | Timestamp filter (events, intersecting, contained) | receipt.csv | NO |
| `test_filtering_timeframe_keep_nan` | Timestamp filter with NaN preservation | receipt.csv | NO |
| `test_filtering_traces_attribute_in_timeframe` | Filter traces with attribute in timeframe | receipt.csv | NO |
| `test_AeventuallyB_pos` | LTL eventually-follows (df, positive) | running-example.csv | NO |
| `test_AeventuallyB_neg` | LTL eventually-follows (df, negative) | running-example.csv | NO |
| `test_AeventuallyBeventuallyC_pos` | LTL 3-activity chain (df, positive) | running-example.csv | NO |
| `test_AeventuallyBeventuallyC_neg` | LTL 3-activity chain (df, negative) | running-example.csv | NO |
| `test_AnextBnextC_pos` | LTL next-activity chain (df, positive) | running-example.csv | NO |
| `test_AnextBnextC_neg` | LTL next-activity chain (df, negative) | running-example.csv | NO |
| `test_fourEeyesPrinciple_pos` | Four-eyes (df, positive) | running-example.csv | NO |
| `test_fourEeyesPrinciple_neg` | Four-eyes (df, negative) | running-example.csv | NO |
| `test_attrValueDifferentPersons_pos` | Different persons (df, positive) | running-example.csv | NO |
| `test_attrValueDifferentPersons_neg` | Different persons (df, negative) | running-example.csv | NO |
| `test_attr_value_repetition` | Attribute value repetition (df) | running-example.csv | NO |

### 14. geneticminer_test.py — 9 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_matrix2petrinet_or` | Genetic matrix→net OR-split/join | no data (hardcoded) | NO |
| `test_matrix2petrinet_or2` | Genetic matrix→net OR variant 2 | no data (hardcoded) | NO |
| `test_matrix2petrinet_andOrCross` | Genetic matrix→net AND/OR/cross | no data (hardcoded) | NO |
| `test_matrix2petrinet_orCross` | Genetic matrix→net OR/cross (Fig. 2) | no data (hardcoded) | NO |
| `test_matrix2petrinet_and` | Genetic matrix→net AND-split/join | no data (hardcoded) | NO |
| `test_matrix2petrinet_and2` | Genetic matrix→net AND variant 2 | no data (hardcoded) | NO |
| `test_matrix2petrinet_full` | Genetic matrix→net full (Fig. 3) | no data (hardcoded) | NO |
| `test_matrix2petrinet_mixed` | Genetic matrix→net BPI 2017 sample | no data (hardcoded) | NO |
| `test_matrix2petrinet_mixed2` | Genetic matrix→net BPI 2017 sample 2 | no data (hardcoded) | NO |

### 15. graphs_forming.py — 4 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_dfCasedurationPlotSemilogx` | KDE case duration plot (df) | receipt.csv | NO |
| `test_logCaseDurationPlotSemiLogx` | KDE case duration plot (log) | receipt.xes | NO |
| `test_dfNumericAttribute` | KDE numeric attribute plot (df) | roadtraffic100traces.csv | NO |
| `test_logNumericAttribute` | KDE numeric attribute plot (log) | roadtraffic100traces.xes | NO |

### 16. heuminer_test.py — 7 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_heunet_running_example` | Heuristics net discovery from XES | running-example.xes | YES |
| `test_petrinet_running_example` | Heuristics Petri net from XES | running-example.xes | YES |
| `test_petrinet_receipt_df` | Heuristics Petri net from CSV df | receipt.csv | NO |
| `test_heuplusplus_perf_df` | Heuristics++ performance from CSV | interval_event_log.csv | NO |
| `test_heuplusplus_perf_log` | Heuristics++ performance from XES | interval_event_log.xes | NO |
| `test_heuplusplus_petri_df` | Heuristics++ Petri net from CSV | interval_event_log.csv | NO |
| `test_heuplusplus_petri_log` | Heuristics++ Petri net from XES | interval_event_log.xes | NO |

### 17. inductive_test.py — 9 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_applyImdfToXES` | Inductive miner on XES, export/reimport, TBR | running-example.xes | YES |
| `test_applyImdfToCSV` | Inductive miner on CSV, export/reimport, TBR | running-example.csv | NO |
| `test_imdfVisualizationFromXES` | Inductive miner + Graphviz viz | running-example.xes | NO |
| `test_inductive_miner_new_log` | Simplified interface: process tree from log | running-example.xes | NO |
| `test_inductive_miner_new_df` | Simplified interface: process tree from df | running-example.xes | NO |
| `test_inductive_miner_new_log_dfg` | Process tree from log via DFG | running-example.xes | NO |
| `test_inductive_miner_new_df_dfg` | Process tree from df via typed DFG | running-example.xes | NO |
| `test_inductive_miner_new_log_variants` | Process tree from UVCL data structure | running-example.xes | NO |
| `test_inductive_miner_new_df_variants` | Process tree from UVCL (same as above) | running-example.xes | NO |

### 18. inductive_tree_test.py — 3 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_tree_running_example_log_plain_based` | Inductive tree from XES + log generation | running-example.xes | NO |
| `test_tree_receipt_log_plain_based` | Inductive tree from receipt XES | receipt.xes | NO |
| `test_tree_parsing` | Parse tree from string expression | no data file | NO |

### 19. llm_test.py — 13 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_abstract_case` | LLM abstraction of single case | running-example.xes | NO |
| `test_abstract_dfg` | LLM abstraction of DFG | running-example.xes | NO |
| `test_abstract_variants` | LLM abstraction of variants | running-example.xes | NO |
| `test_abstract_event_stream` | LLM abstraction of event stream | running-example.xes | NO |
| `test_abstract_log_attributes` | LLM abstraction of log attributes | running-example.xes | NO |
| `test_abstract_log_features` | LLM abstraction of log features | running-example.xes | NO |
| `test_abstract_temporal_profile` | LLM abstraction of temporal profile | receipt.xes | NO |
| `test_abstract_declare` | LLM abstraction of DECLARE model | running-example.xes | NO |
| `test_abstract_log_skeleton` | LLM abstraction of log skeleton | receipt.xes | NO |
| `test_abstract_ocel` | LLM abstraction of OCEL | ocel/example_log.jsonocel | NO |
| `test_abstract_ocel_ocdfg` | LLM abstraction of OCDFG | ocel/example_log.jsonocel | NO |
| `test_abstract_ocel_features` | LLM abstraction of OCEL features | ocel/example_log.jsonocel | NO |
| `test_abstract_petri_net` | LLM abstraction of Petri net | running-example.pnml | NO |

### 20. main_fac_test.py — 18 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_nonstandard_exporter` | XES export with line-by-line variant | running-example.xes | NO |
| `test_alphaminer_log` | Alpha miner + full conformance suite | running-example.xes | NO |
| `test_memory_efficient_iterparse` | Memory-efficient iterparse import | running-example.xes | NO |
| `test_alphaminer_stream` | Alpha miner from event stream (CSV) | running-example.csv | NO |
| `test_alphaminer_df` | Alpha miner from dataframe | running-example.csv | NO |
| `test_inductiveminer_log` | Inductive miner + full conformance | running-example.xes | NO |
| `test_inductiveminer_df` | Inductive miner from df + full conformance | running-example.csv | NO |
| `test_heu_log` | Heuristics miner + full conformance | running-example.xes | NO |
| `test_heu_stream` | Heuristics miner from stream + full conformance | running-example.csv | NO |
| `test_heu_df` | Heuristics miner from df + full conformance | running-example.csv | NO |
| `test_dfg_log` | DFG discovery from XES | running-example.xes | YES |
| `test_dfg_stream` | DFG discovery from event stream | running-example.csv | NO |
| `test_dfg_df` | DFG discovery from dataframe | running-example.csv | NO |
| `test_ts_log` | Transition system from XES | running-example.xes | NO |
| `test_ts_stream` | Transition system from stream | running-example.csv | NO |
| `test_ts_df` | Transition system from df | running-example.csv | NO |
| `test_csvimp_xesexp` | CSV import with log conversions + XES export | running-example.csv | NO |
| `test_xesimp_xesexp` | XES import with conversions + XES export | running-example.xes | NO |
| `test_pdimp_xesexp` | Pandas CSV import + conversions + XES export | running-example.csv | NO |

### 21. ocel_discovery_test.py — 15 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_discovery_ocfg_f1` | OCDFG freq annotation, events metric | ocel/example_log.jsonocel | NO |
| `test_discovery_ocfg_f2` | OCDFG freq annotation, unique_objects | ocel/example_log.jsonocel | NO |
| `test_discovery_ocfg_f3` | OCDFG freq annotation, total_objects | ocel/example_log.jsonocel | NO |
| `test_discovery_ocfg_f4` | OCDFG freq, unique_objects both | ocel/example_log.jsonocel | NO |
| `test_discovery_ocfg_f5` | OCDFG freq, unique_objs act, total_objs edges | ocel/example_log.jsonocel | NO |
| `test_discovery_ocfg_p1` | OCDFG perf annotation, events metric | ocel/example_log.jsonocel | NO |
| `test_discovery_ocfg_p2` | OCDFG perf, unique_objects activity | ocel/example_log.jsonocel | NO |
| `test_discovery_ocfg_p3` | OCDFG perf, total_objects activity | ocel/example_log.jsonocel | NO |
| `test_discovery_ocfg_p4` | OCDFG perf, unique_objects both | ocel/example_log.jsonocel | NO |
| `test_discovery_ocfg_p5` | OCDFG perf, unique_objs act, total_objs edges | ocel/example_log.jsonocel | NO |
| `test_discovery_ocfg_p6` | OCDFG perf, business hours + median | ocel/example_log.jsonocel | NO |
| `test_discovery_ocpn_im` | OC Petri net (IM variant) | ocel/example_log.jsonocel | NO |
| `test_discovery_ocpn_imd` | OC Petri net (IMD variant) | ocel/example_log.jsonocel | NO |
| `test_discovery_saw_nets_ocel` | SAW nets discovery | ocel/example_log.jsonocel | NO |

### 22. ocel_filtering_test.py — 19 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_ocel_import_csv_export_csv` | OCEL CSV round-trip | ocel/example_log.csv | NO |
| `test_ocel_import_csv_export_jsonocel` | OCEL CSV→JSONOCEL | ocel/example_log.csv | NO |
| `test_ocel_import_csv_export_xmlocel` | OCEL CSV→XMLOCEL | ocel/example_log.csv | NO |
| `test_ocel_import_jsonocel_export_csv` | OCEL JSONOCEL→CSV | ocel/example_log.jsonocel | NO |
| `test_ocel_import_jsonocel_export_jsonocel` | OCEL JSONOCEL round-trip | ocel/example_log.jsonocel | NO |
| `test_ocel_import_jsonocel_export_xmlocel` | OCEL JSONOCEL→XMLOCEL | ocel/example_log.jsonocel | NO |
| `test_ocel_import_xmlocel_export_csv` | OCEL XMLOCEL→CSV | ocel/example_log.xmlocel | NO |
| `test_ocel_import_xmlocel_export_jsonocel` | OCEL XMLOCEL→JSONOCEL | ocel/example_log.xmlocel | NO |
| `test_ocel_import_xmlocel_export_xmlocel` | OCEL XMLOCEL round-trip | ocel/example_log.xmlocel | NO |
| `test_ocel_statistic_object_type_activities` | OCEL object-type activities stats | ocel/example_log.jsonocel | NO |
| `test_ocel_objects_ot_count` | OCEL object count per type | ocel/example_log.jsonocel | NO |
| `test_ocel_filter_event_attribute` | OCEL event attribute filter | ocel/example_log.jsonocel | NO |
| `test_ocel_filter_object_attribute` | OCEL object attribute filter | ocel/example_log.jsonocel | NO |
| `test_ocel_filter_object_type_allowed_activities` | OCEL allowed activities filter | ocel/example_log.jsonocel | NO |
| `test_ocel_filter_start_events` | OCEL start events filter | ocel/example_log.jsonocel | NO |
| `test_ocel_filter_end_events` | OCEL end events filter | ocel/example_log.jsonocel | NO |
| `test_ocel_filter_object_per_type_count` | OCEL object count filter | ocel/example_log.jsonocel | NO |
| `test_ocel_filter_timestamp` | OCEL timestamp range filter | ocel/example_log.jsonocel | NO |

### 23. other_tests.py — 52 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_emd_1` | EMD between identical distributions | no data | NO |
| `test_emd_2` | EMD between model language and log language | running-example.xes | NO |
| `test_importing_dfg` | Import DFG from file | running-example.dfg | YES |
| `test_exporting_dfg` | Export DFG, re-import | running-example.xes | NO |
| `test_exporting_dfg_with_sa_ea` | Export DFG with start/end activities | running-example.xes | NO |
| `test_log_skeleton` | Log skeleton discovery + conformance | receipt.xes | YES |
| `test_performance_spectrum_log` | Performance spectrum (log) | receipt.xes | NO |
| `test_performance_spectrum_df` | Performance spectrum (df) | receipt.csv | NO |
| `test_alignment` | Alignment A* + Dijkstra variants | running-example.xes | YES |
| `test_import_export_ptml` | PTML import/export round-trip | running-example.ptml | YES |
| `test_footprints_net` | Footprints discovery + conformance (alpha net) | running-example.xes | YES |
| `test_footprints_tree` | Footprints + conformance (inductive tree) | running-example.xes | YES |
| `test_footprints_tree_df` | Footprints from df + conformance (tree) | running-example.csv | NO |
| `test_conversion_pn_to_pt` | Petri net to process tree | running-example.xes | NO |
| `test_playout_tree_basic` | Basic process tree playout | running-example.xes | NO |
| `test_playout_tree_extensive` | Extensive tree playout | running-example.xes | NO |
| `test_service_time_xes` | Service time from interval log (XES) | interval_event_log.xes | NO |
| `test_service_time_pandas` | Service time from interval log (CSV) | interval_event_log.csv | NO |
| `test_concurrent_activities_xes` | Concurrent activities (XES) | interval_event_log.xes | NO |
| `test_concurrent_activities_pandas` | Concurrent activities (CSV) | interval_event_log.csv | NO |
| `test_efg_xes` | Eventually-follows graph (XES) | interval_event_log.xes | NO |
| `test_efg_pandas` | Eventually-follows graph (CSV) | interval_event_log.csv | NO |
| `test_dfg_playout` | DFG-based playout | running-example.xes | NO |
| `test_dfg_align` | DFG filter + DFG-based alignment | running-example.xes | NO |
| `test_insert_idx_in_trace` | Insert event index in trace | running-example.csv | NO |
| `test_automatic_feature_extraction` | Automatic feature extraction (df) | receipt.csv | NO |
| `test_log_to_trie` | Log to trie data structure | running-example.xes | NO |
| `test_minimum_self_distance` | Minimum self-distance computation | running-example.xes | NO |
| `test_projection_univariate_log` | Univariate projection on concept:name (log) | receipt.xes | NO |
| `test_projection_univariate_df` | Univariate projection on concept:name (df) | receipt.csv | NO |
| `test_log_to_target_rem_time` | Log-to-target: remaining time | running-example.xes | NO |
| `test_log_to_target_next_time` | Log-to-target: next event time | running-example.xes | NO |
| `test_log_to_target_next_activity` | Log-to-target: next activity | running-example.xes | NO |
| `test_ocel_split_cc_non_simpl_interface` | OCEL split by connected components | ocel/example_log.jsonocel | NO |
| `test_ocel_split_ancestors_non_simpl_interface` | OCEL split by ancestors | ocel/example_log.jsonocel | NO |
| `test_ocel_object_features_non_simpl_interface` | OCEL object feature extraction | ocel/example_log.jsonocel | NO |
| `test_ocel_event_features_non_simpl_interface` | OCEL event feature extraction | ocel/example_log.jsonocel | NO |
| `test_ocel_event_object_features_non_simpl_interface` | OCEL event-object features | ocel/example_log.jsonocel | NO |
| `test_ocel_interaction_graph_non_simpl_interface` | OCEL interaction graph | ocel/example_log.jsonocel | NO |
| `test_ocel_descendants_graph_non_simpl_interface` | OCEL descendants graph | ocel/example_log.jsonocel | NO |
| `test_ocel_inheritance_graph_non_simpl_interface` | OCEL inheritance graph | ocel/example_log.jsonocel | NO |
| `test_ocel_cobirth_graph_non_simpl_interface` | OCEL cobirth graph | ocel/example_log.jsonocel | NO |
| `test_ocel_codeath_graph_non_simpl_interface` | OCEL codeath graph | ocel/example_log.jsonocel | NO |
| `test_ocel_description_non_simpl_interface` | OCEL description generation | ocel/example_log.jsonocel | NO |
| `test_murata1` | Murata implicit place reduction #1 | murata1.pnml | NO |
| `test_murata2` | Murata implicit place reduction #2 | murata2.pnml | NO |
| `test_murata3` | Murata implicit place reduction #3 | murata3.pnml | NO |

### 24. passed_time.py — 2 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_passedtime_prepost_log` | Pre/post passed time for "decide" (log) | running-example.xes | NO |
| `test_passedtime_prepost_df` | Pre/post passed time for "decide" (df) | running-example.csv | NO |

### 25. petri_imp_exp_test.py — 4 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_importingExportingPetri` | PNML import/export/re-import verification | running-example.pnml | YES |
| `test_importingPetriLogTokenReplay` | PNML import + TBR conformance | running-example.pnml, .xes | YES |
| `test_importingPetriLogAlignment` | PNML import + alignment conformance | running-example.pnml, .xes | YES |
| `test_s_components` | Inductive tree → Petri net → S-components | running-example.xes | NO |

### 26. polars_* tests — 67 tests (4 files)

| File | Tests | Ported? | Notes |
|------|-------|---------|-------|
| polars_cc_test.py | 16 | NO | N/A for Rust (no Polars) |
| polars_filters_simp_interface.py | 14 | NO | N/A for Rust (no Polars) |
| polars_filters_test.py | 20 | NO | N/A for Rust (no Polars) |
| polars_process_discovery_test.py | 23 | NO | N/A for Rust (no Polars) |
| polars_statistics_get.py | 15 | NO | N/A for Rust (no Polars) |
| polars_statistics_simp_interface.py | 13 | NO | N/A for Rust (no Polars) |

### 27. role_detection.py — 4 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_role_running_csv` | Role detection from CSV | running-example.csv | NO |
| `test_role_running_xes` | Role detection from XES | running-example.xes | NO |
| `test_role_receipt_csv` | Role detection from receipt CSV | receipt.csv | NO |
| `test_role_receipt_xes` | Role detection from receipt XES | receipt.xes | NO |

### 28. simplified_interface.py — 111 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| (See Part 3 output for full list) | Comprehensive simplified interface tests | running-example.*, receipt.*, ocel/*, murata* | ~24 YES, ~87 NO |

### 29. simplified_interface_2.py — 27 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| (See Part 3 output for full list) | Extended simplified interface tests | running-example.*, ocel/* | ~0 YES, ~27 NO |

### 30. simulation_test.py — 2 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_simulate_petrinet` | Petri net playout, verify traces | running-example.pnml | NO |
| `test_simulate_petrinet_start_params` | Playout with custom timestamp/case ID | running-example.pnml | NO |

### 31. sna_test.py — 8 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_1` | SNA (handover, together, subcontracting, joint) | running-example.xes | NO |
| `test_pandas` | SNA from dataframe | running-example.csv | NO |
| `test_log_orgmining_local_attr` | Org mining local diagnostics | receipt.xes | NO |
| `test_log_orgmining_local_clustering` | Org mining with clustering | receipt.xes | NO |
| `test_log_orgmining_local_roles` | Org mining from roles | receipt.xes | NO |
| `test_sna_clustering` | SNA with affinity propagation | running-example.xes | NO |
| `test_res_profiles_log` | Resource profiles (13 metrics) | running-example.xes | NO |
| `test_res_profiles_df` | Resource profiles (df) | running-example.csv | NO |

### 32. statistics_df_test.py — 11 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_end_activities` | End activities from df | roadtraffic100traces.csv | NO |
| `test_start_activities` | Start activities from df | roadtraffic100traces.csv | NO |
| `test_case_arrival` | Case arrival average from df | roadtraffic100traces.csv | NO |
| `test_case_statistics` | Case statistics (variants, KDE, durations) | roadtraffic100traces.csv | NO |
| `test_variants` | Variants set from df | roadtraffic100traces.csv | NO |
| `test_batch_detection` | Batch detection from df | receipt.csv | NO |
| `test_case_overlap` | Case overlap from df | roadtraffic100traces.csv | NO |
| `test_cycle_time` | Cycle time from df | roadtraffic100traces.csv | NO |
| `test_rework` | Rework statistics from df | roadtraffic100traces.csv | NO |
| `test_events_distribution` | Events distribution from df | roadtraffic100traces.csv | NO |
| `test_msd` | Minimum self-distance from df | roadtraffic100traces.csv | NO |

### 33. statistics_log_test.py — 13 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_select_attributes` | Select attributes for tree analysis | roadtraffic100traces.xes | NO |
| `test_end_activities` | End activities from log | roadtraffic100traces.xes | YES |
| `test_start_activities` | Start activities from log | roadtraffic100traces.xes | YES |
| `test_case_arrival` | Case arrival + dispersion | roadtraffic100traces.xes | YES |
| `test_case_statistics` | Full case statistics | roadtraffic100traces.xes | YES |
| `test_variants` | Variants + case durations | roadtraffic100traces.xes | YES |
| `test_batch_detection` | Batch detection | receipt.xes | YES |
| `test_case_overlap` | Case overlap | roadtraffic100traces.xes | YES |
| `test_cycle_time` | Cycle time | roadtraffic100traces.xes | YES |
| `test_rework` | Rework statistics | roadtraffic100traces.xes | YES |
| `test_events_distribution` | Events distribution | roadtraffic100traces.xes | YES |
| `test_msd` | Minimum self-distance | roadtraffic100traces.xes | YES |

### 34. trans_syst_tests.py — 1 test

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_transitionsystem1` | Transition system (seq view, window=3, forward) | running-example.xes | NO |

### 35. woflan_tests.py — 5 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_running_example_alpha` | Woflan soundness on alpha net | running-example.xes | NO |
| `test_running_example_inductive` | Woflan soundness on inductive net | running-example.xes | NO |
| `test_figure415` | Woflan on hand-crafted net (Fig 4.15) | no data file | NO |
| `test_figure42` | Woflan on non-sound net (Fig 4.2) | no data file | NO |
| `test_mcg` | Minimal coverability graph | no data file | NO |

### 36. xes_impexp_test.py — 6 tests

| Test | Description | Data | Ported? |
|------|-------------|------|---------|
| `test_importExportXEStoXES` | XES round-trip | running-example.xes | YES |
| `test_importExportProblematicLogs` | Round-trip for problematic XES files | problematic/ directory | NO |
| `test_importExportXESfromGZIP_imp1` | Gzipped XES import/export | 01_running-example.xes.gz | NO |
| `test_importXESfromGZIP_imp2` | Gzipped XES import | 01_running-example.xes.gz | NO |
| `test_rustxes_xes_import` | RustXES backend import | receipt.xes | NO |
| `test_rustxes_xesgz_import` | RustXES backend gzipped import | bpic2012.xes.gz | NO |

---

## Porting Priority Roadmap

### Phase 1: Unported Tests Using Available Data (HIGH — 89 tests)
These tests use data files already in pm4py-rust `test_data/`:

**From doc_tests.py (17 tests):**
- test_2, test_4, test_5, test_6, test_9, test_11, test_13, test_15, test_24, test_25, test_27, test_29, test_33, test_35, test_39, test_42, test_47, test_48, test_49, test_50, test_57, test_58, test_64, test_66, test_max_decomp, test_reach_graph, test_decomp, test_evaluation, test_playout, test_sna, test_ctmc

**From main_fac_test.py (8 tests):**
- test_alphaminer_log (full conformance suite), test_inductiveminer_log, test_inductiveminer_df, test_heu_log, test_heu_df, test_csvimp_xesexp, test_xesimp_xesexp, test_pdimp_xesexp

**From other_tests.py (20 tests):**
- test_emd_1, test_emd_2, test_exporting_dfg, test_exporting_dfg_with_sa_ea, test_performance_spectrum_log, test_performance_spectrum_df, test_playout_tree_basic, test_playout_tree_extensive, test_service_time_xes, test_service_time_pandas, test_concurrent_activities_xes, test_concurrent_activities_pandas, test_efg_xes, test_efg_pandas, test_dfg_playout, test_dfg_align, test_log_to_trie, test_minimum_self_distance, test_insert_idx_in_trace, test_automatic_feature_extraction, test_projection_univariate_log, test_projection_univariate_df, test_log_to_target_rem_time, test_log_to_target_next_time, test_log_to_target_next_activity, test_footprints_net, test_footprints_tree, test_footprints_tree_df, test_conversion_pn_to_pt

**From evaluation_tests.py (3 tests):**
- test_evaluation_pm1, test_evaluation_pm2, test_simplicity_arc_degree (or extended_cardoso/cyclomatic)

**From algorithm_test.py (4 tests):**
- test_decomp_alignment, test_evaluation, test_playout, test_tree_generation

**From woflan_tests.py (3 tests):**
- test_running_example_alpha, test_running_example_inductive, test_figure42 (or test_figure415)

**From other remaining (6 tests):**
- test_backwards_tbr, test_filter_traces_attribute_in_timeframe, test_filter_paths_performance, test_filter_four_eyes_neg, test_filter_attrValueDifferentPersons_neg, test_filter_attr_value_repetition, test_filter_casefilter_ncases, test_filter_AeventuallyBeventuallyC_*, test_AnextBnextC_*, test_concurrent_activities, test_performance_spectrum, test_business_hours, test_cycle_waiting_time, test_distr_*, test_passed_time, test_minimum_self_distance_2, test_batch_detection (df), test_case_overlap (log), test_events_distribution (log)

### Phase 2: Tests Requiring New Data Files (MEDIUM — 30 tests)
- interval_event_log.xes/csv (5 tests)
- receipt.csv (10 tests)
- roadtraffic100traces.csv (11 tests)
- roadtraffic50traces.xes (2 tests)
- reviewing.xes.gz (3 tests)
- compressed XES files (19 tests)
- OCEL data files (34 tests)

### Phase 3: Tests Requiring New Rust Features (LOW — 70 tests)
- Visualization tests (all — Rust has no viz)
- Polars tests (all — Rust has no Polars)
- LLM tests (all — Rust has no LLM)
- OCEL tests (all — Rust OCEL minimal)
- CLI tests (1)
- Murata reduction tests (3)
- Genetic miner tests (9 — programmatic only)
- OCPN semantics/simulation (20 — programmatic only)

---

## Porting Coverage by Data File

| Data File | Python Tests Using It | Rust Tests Using It | Gap |
|-----------|---------------------|-------------------|-----|
| running-example.xes | ~85 | ~20 | ~65 |
| running-example.csv | ~40 | ~5 | ~35 |
| running-example.pnml | ~10 | ~5 | ~5 |
| running-example.ptml | ~6 | ~5 | ~1 |
| running-example.bpmn | ~4 | ~1 | ~3 |
| running-example.dfg | ~4 | ~3 | ~1 |
| receipt.xes | ~12 | ~5 | ~7 |
| receipt.csv | ~10 | ~0 | ~10 |
| roadtraffic100traces.xes | ~12 | ~5 | ~7 |
| roadtraffic50traces.xes | ~2 | ~0 | ~2 |
| interval_event_log.xes | ~5 | ~0 | ~5 |
| interval_event_log.csv | ~5 | ~0 | ~5 |
| ocel/example_log.jsonocel | ~20 | ~0 | ~20 |
| ocel/example_log.csv | ~6 | ~0 | ~6 |
| murata*.pnml | ~3 | ~0 | ~3 |
| compressed_input_data/*.xes.gz | ~3 | ~0 | ~3 |
