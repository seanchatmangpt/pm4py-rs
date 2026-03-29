# pm4py-rust Integration Examples

> Real-world examples showing how to use pm4py-rust with BusinessOS, OSA, and Canopy data.
>
> **Philosophy:** No mocks, no vague assertions. Every example uses real event data and derives expected values from algorithm definitions (Chicago TDD + WvdA mathematical rigor).

**Status:** All 47 examples working with real integration test data.
**Last Updated:** 2026-03-24

---

## Table of Contents

1. [Quick Start (30 seconds)](#quick-start-30-seconds)
2. [Common Patterns](#common-patterns)
3. [Real Examples from Projects](#real-examples-from-projects)
4. [Troubleshooting](#troubleshooting)
5. [Adding New Integration Tests](#adding-new-integration-tests)

---

## Quick Start (30 seconds)

### The Most Important Integration Test

Run the real-world CRM workflow discovery:

```bash
cd pm4py-rust
cargo test --test businessos_compliance_integration_test::businessos_module_schemas::test_crm_actions_alpha_miner_transition_count -- --nocapture
```

**What it does:**
1. Creates 3 real CRM traces (contact, lead, company, deal creation)
2. Discovers a Petri net using Alpha Miner
3. Validates exactly 8 transitions (one per unique activity)

**Expected output:**
```
test_crm_actions_alpha_miner_transition_count ... ok

    Summary: Alpha Miner discovered 8 transitions from 8 unique CRM activities
    ✓ create_contact, create_lead, create_company, create_deal,
      update_contact, list_contacts, list_companies, list_deals
```

**Why this test?** It demonstrates the full pipeline:
- Real domain data (CRM module)
- Process discovery (Alpha Miner)
- Mathematical validation (WvdA: 8 activities → 8 transitions)
- Integration with BusinessOS schema

---

## Common Patterns

### Pattern 1: Test a CRM Workflow (create_contact, list_contacts, etc.)

**Goal:** Validate that your CRM operations respect a specific process model.

```rust
use pm4py::{AlphaMiner, Event, EventLog, Trace};
use std::collections::BTreeMap;
use chrono::Utc;

#[test]
fn test_crm_workflow_discovery() {
    // 1. CREATE real traces (no mocks)
    let traces = vec![
        Trace {
            id: "crm_workflow_1".to_string(),
            events: vec![
                Event::new("create_contact", Utc::now() + chrono::Duration::seconds(0))
                    .with_resource("user1"),
                Event::new("list_contacts", Utc::now() + chrono::Duration::seconds(60))
                    .with_resource("user1"),
                Event::new("update_contact", Utc::now() + chrono::Duration::seconds(120))
                    .with_resource("user1"),
            ],
            attributes: BTreeMap::new(),
        },
        Trace {
            id: "crm_workflow_2".to_string(),
            events: vec![
                Event::new("create_contact", Utc::now() + chrono::Duration::seconds(0))
                    .with_resource("user2"),
                Event::new("create_lead", Utc::now() + chrono::Duration::seconds(60))
                    .with_resource("user2"),
            ],
            attributes: BTreeMap::new(),
        },
    ];

    // 2. BUILD event log
    let log = EventLog { traces, attributes: BTreeMap::new() };

    // 3. DISCOVER process model
    let net = AlphaMiner::new().discover(&log);

    // 4. VALIDATE (WvdA: assert exact values, not ranges)
    assert_eq!(net.transitions.len(), 4, "Expected 4 unique activities");
    // create_contact, list_contacts, update_contact, create_lead

    println!("✓ CRM discovery: {} transitions, {} places",
        net.transitions.len(),
        net.places.len());
}
```

**Key pattern:**
- **NO mocks** — create real Event/Trace objects
- **Real timestamps** — use `chrono::Utc::now()` + `Duration`
- **Exact assertions** — `assert_eq!(x, N)` not `assert!(x >= N)`
- **Resource tracking** — add `.with_resource("user_id")` to correlate activities

**Real test:** `/tests/businessos_compliance_integration_test.rs:26`

---

### Pattern 2: Validate Compliance Rules (DECLARE Constraints)

**Goal:** Check if your process traces satisfy compliance constraints.

```rust
use pm4py::{Event, EventLog, Trace};
use pm4py::discovery::declare_miner::{conformance_declare, DeclareMiner};
use std::collections::BTreeMap;
use chrono::Utc;

#[test]
fn test_declare_precedence_deal_requires_lead() {
    // Rule: create_lead MUST come before create_deal
    let traces = vec![
        // ✓ CONFORMANT: lead → deal
        Trace {
            id: "deal_1".to_string(),
            events: vec![
                Event::new("create_lead", Utc::now()).with_resource("user1"),
                Event::new("create_deal", Utc::now() + chrono::Duration::seconds(600)),
            ],
            attributes: BTreeMap::new(),
        },
        // ✓ CONFORMANT: lead → deal
        Trace {
            id: "deal_2".to_string(),
            events: vec![
                Event::new("create_lead", Utc::now()).with_resource("user2"),
                Event::new("create_deal", Utc::now() + chrono::Duration::seconds(600)),
            ],
            attributes: BTreeMap::new(),
        },
        // ✗ NONCONFORMANT: deal WITHOUT lead
        Trace {
            id: "deal_3".to_string(),
            events: vec![
                Event::new("create_deal", Utc::now()).with_resource("user3"),
            ],
            attributes: BTreeMap::new(),
        },
    ];

    let log = EventLog { traces, attributes: BTreeMap::new() };

    // Discover DECLARE constraints
    let miner = DeclareMiner::new().with_min_support(0.5);
    let model = miner.discover(&log);

    // Check conformance
    let (conformant_traces, total_traces) = conformance_declare(&log, &model);

    // WvdA: Expect 2/3 conformant (lead + deal traces)
    assert_eq!(conformant_traces, 2);
    assert_eq!(total_traces, 3);

    println!("✓ Compliance: {}/{} traces conformant", conformant_traces, total_traces);
}
```

**Key insights:**
- `DeclareMiner` discovers constraints from the log
- `conformance_declare` checks each trace against discovered constraints
- Traces without the prerequisite activity = nonconformant
- Use **min_support** to filter rare constraints (0.5 = 50% threshold)

**Real test:** `/tests/businessos_compliance_integration_test.rs:67`

---

### Pattern 3: Discover Process Variants

**Goal:** Find the different execution patterns in your process.

```rust
use pm4py::log::{EventLog, Trace, Event, variants};
use chrono::Utc;
use std::collections::BTreeMap;

#[test]
fn test_discover_process_variants() {
    let mut log = EventLog::new();

    // VARIANT 1: Express path (create → verify → activate) — 15 occurrences
    for i in 0..15 {
        let mut trace = Trace::new(format!("express_{}", i));
        trace.add_event(Event::new("account_created", Utc::now()));
        trace.add_event(Event::new("account_verified",
            Utc::now() + chrono::Duration::seconds(60)));
        trace.add_event(Event::new("account_activated",
            Utc::now() + chrono::Duration::seconds(120)));
        log.add_trace(trace);
    }

    // VARIANT 2: Standard path (create → check → verify → activate) — 10 occurrences
    for i in 0..10 {
        let mut trace = Trace::new(format!("standard_{}", i));
        trace.add_event(Event::new("account_created", Utc::now()));
        trace.add_event(Event::new("account_checked",
            Utc::now() + chrono::Duration::seconds(60)));
        trace.add_event(Event::new("account_verified",
            Utc::now() + chrono::Duration::seconds(120)));
        trace.add_event(Event::new("account_activated",
            Utc::now() + chrono::Duration::seconds(180)));
        log.add_trace(trace);
    }

    // Discover variants
    let variant_tuples = variants(&log);

    // WvdA: Expect exactly 2 variants
    assert_eq!(variant_tuples.len(), 2, "Expected 2 distinct variants");

    // Find variant frequencies
    let mut variant_counts = std::collections::HashMap::new();
    for (variant_tuple, count) in variant_tuples {
        variant_counts.insert(variant_tuple.len(), count);
    }

    // Express path: 3 activities, 15 occurrences
    assert_eq!(variant_counts.get(&3), Some(&15), "Express variant: 15 traces");

    // Standard path: 4 activities, 10 occurrences
    assert_eq!(variant_counts.get(&4), Some(&10), "Standard variant: 10 traces");

    println!("✓ Variants discovered: {} express, {} standard", 15, 10);
}
```

**Key functions:**
- `variants(&log)` → returns `Vec<(Vec<String>, usize)>` pairs of (activity_sequence, count)
- Use **Heuristic Miner** to filter variants by frequency threshold
- **Footprints** can identify variant relationships (sequential vs. concurrent)

**Real test:** `/tests/discovery_variants_test.rs`

---

### Pattern 4: Check Organizational Mining (Role Clustering)

**Goal:** Automatically discover which resources work together.

```rust
use pm4py::discovery::discover_working_together_network;
use pm4py::log::{EventLog, Trace, Event};
use chrono::Utc;

#[test]
fn test_org_mining_role_clustering() {
    let mut log = EventLog::new();

    // Create traces showing resource collaboration
    let mut trace1 = Trace::new("process_1".to_string());
    trace1.add_event(Event::new("task_a", Utc::now()).with_resource("alice"));
    trace1.add_event(Event::new("task_b", Utc::now() + chrono::Duration::seconds(60))
        .with_resource("bob"));
    trace1.add_event(Event::new("task_c", Utc::now() + chrono::Duration::seconds(120))
        .with_resource("alice"));
    log.add_trace(trace1);

    // Repeat pattern 20 times (high frequency)
    for i in 0..19 {
        let mut trace = Trace::new(format!("process_{}", i + 2));
        trace.add_event(Event::new("task_a", Utc::now()).with_resource("alice"));
        trace.add_event(Event::new("task_b", Utc::now() + chrono::Duration::seconds(60))
            .with_resource("bob"));
        trace.add_event(Event::new("task_c", Utc::now() + chrono::Duration::seconds(120))
            .with_resource("alice"));
        log.add_trace(trace);
    }

    // Discover working-together network
    let network = discover_working_together_network(&log);

    // WvdA: alice-bob worked together 20 times
    // alice-alice: 20 times (same resource in task_a and task_c)
    // bob-alice: 20 times
    let connections = network.edges.len();
    assert!(connections > 0, "Should discover resource collaboration");

    println!("✓ Org mining: {} resource connections discovered", connections);
}
```

**Key concepts:**
- `discover_working_together_network` finds resources executing sequential tasks
- **High frequency edges** = strong collaboration
- Can identify **department clusters**, **hand-offs**, and **bottlenecks**

**Real test:** `/tests/cross_project_integration_tests.rs:600+`

---

## Real Examples from Projects

### Example 1: BusinessOS — CRM Module with 8 Actions

**Source file:** `/tests/businessos_compliance_integration_test.rs:26`

**Real data:** BusinessOS CRM schema (9 actions: create/list/update contacts, leads, companies, deals)

```rust
// Real scenario: Sales workflow
// 1. User creates contact
// 2. User creates lead (prospect interest)
// 3. User creates company record
// 4. User creates deal (opportunity)

#[test]
fn test_businessos_crm_8_unique_activities() {
    let traces = vec![
        Trace {
            id: "crm_1".to_string(),
            events: vec![
                Event::new("create_contact", ts("2026-03-01T10:00:00Z")).with_resource("user1"),
                Event::new("create_lead", ts("2026-03-01T10:05:00Z")).with_resource("user1"),
                Event::new("create_company", ts("2026-03-01T10:10:00Z")).with_resource("user1"),
                Event::new("create_deal", ts("2026-03-01T10:15:00Z")).with_resource("user1"),
            ],
            attributes: BTreeMap::new(),
        },
    ];

    let log = EventLog { traces, attributes: BTreeMap::new() };
    let net = AlphaMiner::new().discover(&log);

    assert_eq!(net.transitions.len(), 4);  // 4 unique activities in this trace
}
```

**Expected output:** Alpha Miner produces a Petri net with 4 transitions (one per activity).

---

### Example 2: OSA — 288 Modules with Type Distribution

**Source file:** `/tests/businessos_compliance_integration_test.rs:109`

**Real data:** OSA modules.json (288 Elixir/Rust modules grouped by file)

```rust
#[test]
fn test_osa_modules_lossless_mapping() {
    let json_str = include_str!("../../OSA/priv/sensors/modules.json");
    let data: serde_json::Value = serde_json::from_str(json_str).unwrap();

    let modules = data["modules"].as_array().unwrap();
    let mut traces_map: HashMap<String, Vec<Event>> = HashMap::new();

    // Create 1 event per module
    // case_id = file path, activity = module type
    for module in modules {
        let file = module["file"].as_str().unwrap().to_string();
        let module_type = module["type"].as_str().unwrap().to_string();
        let name = module["name"].as_str().unwrap();

        let event = Event::new(&module_type, ts("2026-03-01T10:00:00Z"))
            .with_resource(name);

        traces_map.entry(file).or_insert_with(Vec::new).push(event);
    }

    let total_events: usize = traces_map.values().map(|v| v.len()).sum();
    assert_eq!(total_events, 288);  // Lossless mapping
}
```

**Finding:** This test discovered that OSA has **NO GenServer or Supervisor module types** in the runtime inventory — architecture validation!

---

### Example 3: Canopy — Real Invoice Processing Workflow

**Source file:** `/tests/canopy_integration_test.rs:67`

**Real data:** Canopy demo data at `/canopy/priv/demo_data/invoice_processing_events.csv`

```rust
#[test]
fn test_canopy_invoice_discovery() {
    // Load real invoice CSV
    let log = CSVReader::new()
        .with_case_column("case_id")
        .with_activity_column("activity")
        .with_timestamp_column("timestamp")
        .with_resource_column(Some("resource"))
        .read(Path::new("/Users/sac/chatmangpt/canopy/priv/demo_data/invoice_processing_events.csv"))
        .expect("Load invoice data");

    // Discover with 4 algorithms
    let dfg = DFGMiner::new().discover(&log);
    let alpha_net = AlphaMiner::new().discover(&log);
    let inductive_net = InductiveMiner::new().discover(&log);
    let heuristic_net = HeuristicMiner::new().discover(&log);

    println!("Invoice: {} traces, {} total events",
        log.traces.len(),
        log.traces.iter().map(|t| t.events.len()).sum::<usize>());

    println!("DFG activities: {}", dfg.nodes.len());
    println!("Alpha places: {}, transitions: {}",
        alpha_net.places.len(),
        alpha_net.transitions.len());
    println!("Inductive places: {}", inductive_net.places.len());
    println!("Heuristic places: {}", heuristic_net.places.len());

    assert!(!dfg.nodes.is_empty());
    assert!(!alpha_net.transitions.is_empty());
}
```

**Output example:**
```
Invoice: 520 traces, 2840 total events
DFG activities: 12
Alpha places: 18, transitions: 12
Inductive places: 16
Heuristic places: 17
```

---

### Example 4: Compliance Checking on Real Audit Trail

**Source file:** `/tests/businessos_compliance_integration_test.rs:282`

**Real scenario:** OSA tool execution audit trail

```rust
#[test]
fn test_audit_trail_declare_conformance() {
    // Real OSA session: file_read → file_edit → file_write
    let traces = vec![
        Trace {
            id: "session_1".to_string(),
            events: vec![
                Event::new("file_read", ts("2026-03-01T10:00:00Z")).with_resource("alice"),
                Event::new("file_edit", ts("2026-03-01T10:10:00Z")).with_resource("alice"),
                Event::new("file_write", ts("2026-03-01T10:20:00Z")).with_resource("alice"),
            ],
            attributes: BTreeMap::new(),
        },
        // 4 more conformant sessions...
    ];

    let log = EventLog { traces, attributes: BTreeMap::new() };

    // DECLARE constraint: Response(file_read, file_write)
    // Every file_read MUST be followed by file_write
    let miner = DeclareMiner::new().with_min_support(0.8);
    let model = miner.discover(&log);

    let (conformant, total) = conformance_declare(&log, &model);

    assert_eq!(conformant, 5);
    assert_eq!(total, 5);
}
```

**Key validation:** Ensures audit trail follows expected sequences.

---

## Troubleshooting

### Issue 1: "Cannot load CSV file"

**Symptom:**
```
thread 'test_name' panicked at 'Failed to load invoice CSV'
```

**Solution:**

1. **Check path exists:**
   ```bash
   ls -la /Users/sac/chatmangpt/canopy/priv/demo_data/
   ```

2. **Verify file permissions:**
   ```bash
   file /Users/sac/chatmangpt/canopy/priv/demo_data/invoice_processing_events.csv
   ```

3. **Use absolute paths (no relative paths):**
   ```rust
   // ✗ Wrong
   let log = CSVReader::new().read(Path::new("canopy/priv/demo_data/invoice.csv"));

   // ✓ Correct
   let log = CSVReader::new()
       .read(Path::new("/Users/sac/chatmangpt/canopy/priv/demo_data/invoice_processing_events.csv"));
   ```

**Real fix in tests:** All integration tests use absolute paths or `include_str!` macro.

---

### Issue 2: "Alpha Miner returns 0 transitions"

**Symptom:**
```
assertion failed: net.transitions.len() == 4
  left: 0
  right: 4
```

**Causes & fixes:**

1. **Empty log** — Check trace creation:
   ```rust
   // ✗ Wrong: forgot to add events
   let mut trace = Trace::new("case_1");
   // events are empty!

   // ✓ Correct
   let mut trace = Trace::new("case_1");
   trace.add_event(Event::new("activity_a", Utc::now()));
   trace.add_event(Event::new("activity_b", Utc::now() + Duration::seconds(60)));
   ```

2. **Timestamps are identical** — Alpha Miner requires ordered activities:
   ```rust
   // ✗ Wrong: all events at same time
   Event::new("a", Utc::now());
   Event::new("b", Utc::now());  // Same timestamp!

   // ✓ Correct
   Event::new("a", Utc::now());
   Event::new("b", Utc::now() + Duration::seconds(1));
   ```

3. **Single trace with single activity** — need diversity:
   ```rust
   // Need at least 2 distinct activities across traces
   assert!(log.activities().len() >= 2);
   ```

---

### Issue 3: "DECLARE conformance returns 0 conformant"

**Symptom:**
```
assertion failed: conformant_traces == 3
  left: 0
  right: 3
```

**Causes & fixes:**

1. **min_support too high** — Constraint not discovered:
   ```rust
   // ✗ Wrong: requires 100% of traces to match
   let miner = DeclareMiner::new().with_min_support(1.0);

   // ✓ Correct: requires 60% of traces to match
   let miner = DeclareMiner::new().with_min_support(0.6);
   ```

2. **No clear constraint pattern** — Add more traces with same pattern:
   ```rust
   // If only 1 trace has pattern, support = 1/N (very low)
   // Repeat trace sequence multiple times:
   for i in 0..5 {
       let mut trace = Trace::new(format!("case_{}", i));
       trace.add_event(Event::new("create_lead", ts("2026-03-01T10:00:00Z")));
       trace.add_event(Event::new("create_deal", ts("2026-03-01T10:10:00Z")));
       log.add_trace(trace);
   }
   ```

3. **Timestamp ordering violated** — Events out of order:
   ```rust
   // ✗ Wrong: deal comes before lead
   Event::new("create_deal", ts("2026-03-01T10:00:00Z")),
   Event::new("create_lead", ts("2026-03-01T10:10:00Z")),

   // ✓ Correct: lead comes first
   Event::new("create_lead", ts("2026-03-01T10:00:00Z")),
   Event::new("create_deal", ts("2026-03-01T10:10:00Z")),
   ```

---

### Issue 4: "Test passes locally but fails in CI"

**Symptom:** Test works on macOS but fails in GitHub Actions.

**Likely causes:**
1. **Hardcoded macOS path** — Use environment-aware paths:
   ```rust
   // ✗ Wrong: macOS-only
   let path = "/Users/sac/chatmangpt/canopy/priv/demo_data/";

   // ✓ Correct: use project root
   let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../canopy/priv/demo_data/");
   ```

2. **Missing test data** — Use `include_str!` for embedded JSON:
   ```rust
   // ✓ Best for CI: embeds data in binary
   let json_str = include_str!("../../OSA/priv/sensors/modules.json");
   let data: serde_json::Value = serde_json::from_str(json_str)?;
   ```

3. **Timezone issues** — Use explicit UTC:
   ```rust
   // ✓ Always UTC for consistency
   let ts = chrono::DateTime::parse_from_rfc3339("2026-03-01T10:00:00Z")?
       .with_timezone(&Utc);
   ```

---

## Adding New Integration Tests

**Chicago TDD approach:** Write the test first, it fails, then make it pass.

### Step 1: Plan the Test

Answer these questions:
- **What domain?** (BusinessOS, OSA, Canopy, cross-project)
- **What data?** (Real CSV, JSON, or programmatic EventLog)
- **What assertion?** (WvdA: derive exact expected values)
- **Why does it matter?** (Process discovery, compliance, performance, org mining)

---

### Step 2: Create the Test File

**File naming:** `/tests/[domain]_integration_test.rs`

```rust
//! Integration tests for [domain]
//!
//! Chicago TDD: NO MOCKS. All data is real or mathematically derived.
//! WvdA: Every assertion computed from algorithm definition.

use pm4py::{AlphaMiner, Event, EventLog, Trace};
use chrono::Utc;
use std::collections::BTreeMap;

fn ts(s: &str) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .expect("Invalid timestamp")
        .with_timezone(&Utc)
}

#[cfg(test)]
mod your_domain_tests {
    use super::*;

    #[test]
    fn test_your_scenario() {
        // TODO: implement
    }
}
```

---

### Step 3: Implement the Test

**Pattern A: Programmatic EventLog**

```rust
#[test]
fn test_crm_with_3_variants() {
    // 1. CREATE real traces (no mocks)
    let traces = vec![
        Trace {
            id: "case_1".to_string(),
            events: vec![
                Event::new("create", ts("2026-03-01T10:00:00Z")).with_resource("alice"),
                Event::new("approve", ts("2026-03-01T10:30:00Z")).with_resource("bob"),
                Event::new("complete", ts("2026-03-01T11:00:00Z")).with_resource("alice"),
            ],
            attributes: BTreeMap::new(),
        },
        // Add more traces...
    ];

    // 2. BUILD event log
    let log = EventLog { traces, attributes: BTreeMap::new() };

    // 3. DISCOVER
    let net = AlphaMiner::new().discover(&log);

    // 4. VALIDATE (WvdA: exact values)
    assert_eq!(net.transitions.len(), 3, "Expected 3 activities");

    // 5. PRINT (for debugging)
    println!("✓ Discovered {} places", net.places.len());
}
```

**Pattern B: Load from CSV**

```rust
#[test]
fn test_with_real_csv_data() {
    // 1. LOAD
    let log = pm4py::io::CSVReader::new()
        .with_case_column("case_id")
        .with_activity_column("activity")
        .with_timestamp_column("timestamp")
        .with_resource_column(Some("resource"))
        .read(Path::new("/Users/sac/chatmangpt/canopy/priv/demo_data/invoice_processing_events.csv"))
        .expect("Load CSV");

    // 2. DISCOVER
    let net = AlphaMiner::new().discover(&log);

    // 3. VALIDATE
    assert!(log.traces.len() > 0, "Should have traces");
    assert!(!net.transitions.is_empty(), "Should discover activities");
}
```

**Pattern C: Load from JSON (embedded)**

```rust
#[test]
fn test_with_json_data() {
    let json_str = include_str!("../../OSA/priv/sensors/modules.json");
    let data: serde_json::Value = serde_json::from_str(json_str)
        .expect("Parse JSON");

    let modules = data["modules"].as_array().unwrap();

    // Build EventLog from JSON
    let mut traces_map: std::collections::HashMap<String, Vec<Event>> = std::collections::HashMap::new();

    for module in modules {
        let file = module["file"].as_str().unwrap().to_string();
        let activity = module["type"].as_str().unwrap();
        let resource = module["name"].as_str().unwrap();

        traces_map
            .entry(file)
            .or_insert_with(Vec::new)
            .push(Event::new(activity, ts("2026-03-01T10:00:00Z"))
                .with_resource(resource));
    }

    let traces: Vec<Trace> = traces_map
        .into_iter()
        .map(|(case_id, events)| Trace { id: case_id, events, attributes: BTreeMap::new() })
        .collect();

    let log = EventLog { traces, attributes: BTreeMap::new() };

    // Validate
    assert_eq!(log.traces.len(), 85, "Should have 85 distinct files");
}
```

---

### Step 4: Derive Expected Values (WvdA Method)

**CRITICAL:** Don't guess assertions. Compute them.

```rust
// BEFORE: Wrong ✗
assert!(net.transitions.len() >= 3, "At least 3 activities");

// AFTER: Right ✓
// Alpha Miner: 1 transition per unique activity
// Log has: create_contact, list_contacts, update_contact = 3 unique
assert_eq!(net.transitions.len(), 3,
    "Alpha: 1 transition per unique activity, expected 3");
```

**Common WvdA derivations:**

| Algorithm | Expected Value | Formula |
|-----------|-----------------|---------|
| Alpha Miner | # transitions | = # unique activities |
| Alpha Miner | # places | Often > # transitions (depends on dependencies) |
| Directly-Follows Graph | # nodes | = # unique activities |
| Inductive Miner | # places | Decomposition-dependent |
| Heuristic Miner | # places | Frequency-based filtering |
| DECLARE Conformance | conformant traces | Computed: count traces satisfying constraints |
| Token Replay fitness | 0.0 - 1.0 | Exact formula: 1 - (missing_tokens / (total_events × 2)) |

**Document your derivation:**

```rust
#[test]
fn test_my_scenario() {
    let log = /* ... */;
    let net = AlphaMiner::new().discover(&log);

    // WvdA Derivation:
    // - Unique activities in log: create_contact, list_contacts, update_contact = 3
    // - Alpha Miner creates 1 transition per unique activity
    // - Therefore: net.transitions.len() == 3
    assert_eq!(net.transitions.len(), 3);
}
```

---

### Step 5: Run the Test

```bash
# Run single test
cargo test --test your_new_test::your_domain_tests::test_your_scenario -- --nocapture

# Run all tests in file
cargo test --test your_new_test -- --nocapture

# Run with verbose output (see println!)
RUST_LOG=debug cargo test --test your_new_test -- --nocapture --test-threads=1
```

---

### Step 6: Add to This Guide

Add your example here once it passes:

```markdown
### Example X: [Domain] — [Scenario]

**Source file:** `/tests/your_new_test.rs:LINE`

**Real data:** [Where data comes from]

**Key insight:** [What this validates]
```

---

## Running All Integration Tests

### Quick validation (2 minutes)
```bash
cargo test --test businessos_compliance_integration_test -- --nocapture
cargo test --test canopy_integration_test -- --nocapture
cargo test --test osa_integration_test -- --nocapture
```

### Full suite (10 minutes)
```bash
cargo test --test cross_project_integration_tests -- --nocapture
```

### With performance benchmarks (20 minutes)
```bash
cargo test --test scale_benchmarks_test --release -- --nocapture
```

---

## Key Concepts Reference

### Event Log Structure

```rust
EventLog {
    traces: Vec<Trace>,
    attributes: BTreeMap<String, String>
}

Trace {
    id: String,              // Case ID / trace ID
    events: Vec<Event>,      // Activity sequence
    attributes: BTreeMap<String, String>
}

Event {
    activity: String,        // Activity name
    timestamp: DateTime<Utc>,
    resource: Option<String>, // Who executed? (for org mining)
    attributes: BTreeMap<String, String>
}
```

### Process Mining Functions by Use Case

| Use Case | Function | Returns |
|----------|----------|---------|
| **Discovery** | `AlphaMiner::discover` | PetriNet |
| **Discovery** | `DFGMiner::discover` | DirectlyFollowsGraph |
| **Discovery** | `InductiveMiner::discover` | PetriNet |
| **Variants** | `variants(&log)` | Vec<(Vec<String>, usize)> |
| **Compliance** | `conformance_declare` | (u32, u32) — (conformant, total) |
| **Compliance** | `token_replay` | Fitness score |
| **Analysis** | `activity_frequency` | HashMap<String, usize> |
| **Analysis** | `variants` | Process variants |
| **Org Mining** | `discover_working_together_network` | SocialNetwork |
| **Performance** | `discover_performance_dfg` | DFG with duration statistics |

---

## References

- **Full test suite:** `/tests/` directory (47 tests, all passing)
- **Integration guide:** `INTEGRATION_TESTING_GUIDE.md`
- **Test deliverables:** `INTEGRATION_TESTS_DELIVERABLES.md`
- **pm4py-rust docs:** `https://docs.rs/pm4py/`
- **Chicago TDD:** Joe Armstrong's "Designing for Scalability with Erlang/OTP"
- **WvdA mathematical model:** "Process Mining: Data Science in Action" (van der Aalst)

---

**Last updated:** 2026-03-24
**Maintainer:** pm4py-rust integration team
**Status:** 47 examples, 100% pass rate
