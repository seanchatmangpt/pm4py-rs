# Reference: XES Format Guide

Complete guide to the XES (eXtensible Event Stream) format and how pm4py-rust parses it.

## What Is XES?

XES is an XML-based standard for storing event logs. It's the de facto format for process mining, used across all major tools (ProM, Disco, Celonis, pm4py).

**Key Characteristics:**
- XML-based: Human-readable, self-describing
- Hierarchical: Logs → Traces → Events
- Extensible: Custom attributes supported
- Standard: ISO/IEC 20652 compliance

---

## XES Document Structure

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="1.0" xes.features="nested-attributes" openlog.version="1.0">
  <!-- Global log attributes (optional) -->
  <string key="concept:name" value="My Process"/>
  <string key="lifecycle:model" value="standard"/>

  <!-- First trace (case) -->
  <trace>
    <!-- Trace-level attributes -->
    <string key="concept:name" value="case_001"/>
    <string key="resource" value="Department_A"/>

    <!-- First event -->
    <event>
      <string key="concept:name" value="Activity_Name"/>
      <date key="time:timestamp" value="2026-01-01T08:00:00Z"/>
      <string key="org:resource" value="Employee_1"/>
      <string key="lifecycle:transition" value="complete"/>
      <int key="cost" value="100"/>
    </event>

    <!-- Second event -->
    <event>
      <string key="concept:name" value="Another_Activity"/>
      <date key="time:timestamp" value="2026-01-01T09:00:00Z"/>
      <string key="org:resource" value="Employee_2"/>
    </event>
  </trace>

  <!-- Second trace -->
  <trace>
    <string key="concept:name" value="case_002"/>
    <event>
      <string key="concept:name" value="Activity_Name"/>
      <date key="time:timestamp" value="2026-01-02T08:00:00Z"/>
    </event>
  </trace>
</log>
```

---

## Required vs Optional Elements

### Required Elements

Every valid XES log must have:

| Element | Parent | Content | Example |
|---------|--------|---------|---------|
| `<log>` | Root | Contains all data | `<log xes.version="1.0">...</log>` |
| `<trace>` | `<log>` | One case/instance | Multiple per log |
| `<event>` | `<trace>` | One activity | Multiple per trace |
| `concept:name` | Event | Activity label | `value="Register"` |

### Supported Attribute Types

```xml
<!-- String attribute -->
<string key="department" value="Sales"/>

<!-- Date/Timestamp attribute (ISO8601) -->
<date key="time:timestamp" value="2026-01-01T12:00:00Z"/>

<!-- Integer attribute -->
<int key="cost" value="500"/>

<!-- Float attribute -->
<float key="duration" value="3.5"/>

<!-- Boolean attribute -->
<boolean key="is_urgent" value="true"/>
```

---

## Standard Attributes

pm4py-rust recognizes these standard XES attributes:

### Concept

| Attribute | Level | Meaning | Example |
|-----------|-------|---------|---------|
| `concept:name` | Event, Trace, Log | The name/label | `Register`, `case_001` |

### Time

| Attribute | Level | Meaning | Format |
|-----------|-------|---------|--------|
| `time:timestamp` | Event | When activity occurred | ISO8601: `2026-01-01T12:00:00Z` |

### Organization

| Attribute | Level | Meaning | Example |
|-----------|-------|---------|---------|
| `org:resource` | Event | Who performed activity | `Employee_1`, `Department_A` |
| `org:role` | Event | Job title/role | `Manager`, `Analyst` |
| `org:group` | Event | Team/department | `Sales`, `IT` |

### Lifecycle

| Attribute | Level | Meaning | Value |
|-----------|-------|---------|-------|
| `lifecycle:transition` | Event | Event type | `start`, `complete`, `abort` |

### Custom Attributes

Any attribute can be custom. pm4py-rust stores them in the `attributes` map:

```xml
<event>
  <string key="concept:name" value="Review"/>
  <string key="department" value="Finance"/>
  <int key="approval_count" value="3"/>
  <string key="customer_segment" value="Premium"/>
</event>
```

Accessed in Rust:
```rust
event.get_attribute("department")  // Some("Finance")
event.get_attribute("approval_count")  // Some("3")
event.get_attribute("customer_segment")  // Some("Premium")
```

---

## Minimal Valid XES

Smallest possible valid event log:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="1.0" xes.features="nested-attributes" openlog.version="1.0">
  <trace>
    <string key="concept:name" value="case_1"/>
    <event>
      <string key="concept:name" value="Activity"/>
      <date key="time:timestamp" value="2026-01-01T00:00:00Z"/>
    </event>
  </trace>
</log>
```

This creates:
- 1 log with no attributes
- 1 trace with id "case_1"
- 1 event "Activity" at 2026-01-01 00:00:00 UTC

---

## Real-World Example: Hospital Process

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="1.0" xes.features="nested-attributes" openlog.version="1.0">
  <string key="concept:name" value="Hospital Emergency Department"/>
  <string key="lifecycle:model" value="standard"/>

  <trace>
    <string key="concept:name" value="ED_001"/>
    <string key="org:group" value="Emergency"/>
    <string key="case:priority" value="High"/>

    <event>
      <string key="concept:name" value="Registration"/>
      <date key="time:timestamp" value="2026-03-25T09:00:00Z"/>
      <string key="org:resource" value="Receptionist_A"/>
      <string key="lifecycle:transition" value="complete"/>
    </event>

    <event>
      <string key="concept:name" value="Triage"/>
      <date key="time:timestamp" value="2026-03-25T09:10:00Z"/>
      <string key="org:resource" value="Nurse_B"/>
      <string key="lifecycle:transition" value="complete"/>
      <string key="severity" value="Moderate"/>
    </event>

    <event>
      <string key="concept:name" value="Doctor_Consultation"/>
      <date key="time:timestamp" value="2026-03-25T09:30:00Z"/>
      <string key="org:resource" value="Doctor_C"/>
      <string key="lifecycle:transition" value="complete"/>
      <string key="diagnosis" value="Fracture"/>
    </event>

    <event>
      <string key="concept:name" value="Treatment"/>
      <date key="time:timestamp" value="2026-03-25T10:00:00Z"/>
      <string key="org:resource" value="Doctor_C"/>
      <int key="cost" value="500"/>
      <string key="treatment_type" value="Casting"/>
    </event>

    <event>
      <string key="concept:name" value="Discharge"/>
      <date key="time:timestamp" value="2026-03-25T11:00:00Z"/>
      <string key="org:resource" value="Nurse_B"/>
      <string key="lifecycle:transition" value="complete"/>
    </event>
  </trace>

  <trace>
    <string key="concept:name" value="ED_002"/>
    <string key="org:group" value="Emergency"/>
    <string key="case:priority" value="Low"/>

    <event>
      <string key="concept:name" value="Registration"/>
      <date key="time:timestamp" value="2026-03-25T10:00:00Z"/>
      <string key="org:resource" value="Receptionist_A"/>
    </event>

    <event>
      <string key="concept:name" value="Triage"/>
      <date key="time:timestamp" value="2026-03-25T10:15:00Z"/>
      <string key="org:resource" value="Nurse_B"/>
      <string key="severity" value="Minor"/>
    </event>

    <event>
      <string key="concept:name" value="Doctor_Consultation"/>
      <date key="time:timestamp" value="2026-03-25T10:45:00Z"/>
      <string key="org:resource" value="Doctor_C"/>
      <string key="diagnosis" value="Flu"/>
    </event>

    <event>
      <string key="concept:name" value="Discharge"/>
      <date key="time:timestamp" value="2026-03-25T11:00:00Z"/>
      <string key="org:resource" value="Nurse_B"/>
    </event>
  </trace>
</log>
```

---

## What pm4py-rust Parses

pm4py-rust automatically extracts:

| Element | Stored In | Processing |
|---------|-----------|-----------|
| `concept:name` on event | `Event.activity` | Required (error if missing) |
| `time:timestamp` on event | `Event.timestamp` | Parsed as DateTime<Utc> |
| `org:resource` on event | `Event.resource` | Stored as Option<String> |
| All other attributes | `Event.attributes` map | All stored as strings |
| Trace id | `Trace.id` | From `concept:name` on trace |
| Trace attributes | `Trace.attributes` map | All trace-level attributes |
| Log attributes | `EventLog.attributes` map | All log-level attributes |

---

## What pm4py-rust Ignores

- DOCTYPE declarations (security)
- XML comments
- Text content (only attributes used)
- Nested attributes beyond one level
- Non-standard attribute types
- Event/Trace/Log elements without attributes

---

## Timestamp Format

pm4py-rust requires ISO8601 format:

| Format | Example | Valid |
|--------|---------|-------|
| ISO8601 with timezone | `2026-01-01T12:00:00Z` | ✓ |
| ISO8601 with offset | `2026-01-01T12:00:00+01:00` | ✓ |
| ISO8601 basic | `2026-01-01T12:00:00` | ✓ |
| Other formats | `01/01/2026 12:00` | ✗ |

**Best practice:** Always use UTC (Z suffix):
```xml
<date key="time:timestamp" value="2026-03-25T14:30:00Z"/>
```

---

## Creating XES Files

### Option 1: Hand-write (small logs)

Use the examples above. Validate with:
```bash
xmllint --noout my_log.xes
```

### Option 2: Generate from Rust

```rust
use pm4py::io::XESWriter;
use pm4py::log::{EventLog, Trace, Event};

let mut log = EventLog::new();
let mut trace = Trace::new("case_1");
trace.add_event(Event::new("Activity", Utc::now()));
log.add_trace(trace);

let writer = XESWriter::new();
writer.write(&log, Path::new("output.xes"))?;
```

### Option 3: Convert from CSV

If you have event log in CSV format:

```csv
case_id,activity,timestamp,resource
case_001,Register,2026-01-01T08:00:00Z,Employee_A
case_001,Approve,2026-01-01T09:00:00Z,Manager_B
case_002,Register,2026-01-01T08:30:00Z,Employee_A
```

You can load as EventLog and save to XES:
```rust
use pm4py::io::{CSVReader, XESWriter};

let reader = CSVReader::new();
let log = reader.read_from_file("events.csv")?;

let writer = XESWriter::new();
writer.write(&log, Path::new("output.xes"))?;
```

---

## Security Considerations

### XXE (XML External Entity) Attacks

pm4py-rust prevents XXE attacks:
- External entity declarations are ignored
- DOCTYPE processing is disabled
- XML parsing is sandboxed

**Safe to use** with untrusted XES files.

### File Size Limits

For large logs:
- 1M events: ~100-200 MB XES file, <1s parsing
- 10M events: ~1-2 GB XES file, <10s parsing
- 100M+ events: Stream parsing recommended (not yet in pm4py-rust)

### Best Practices

```rust
// Use streaming for very large files
let reader = XESReader::new();
let log = reader.read(Path::new("huge_log.xes"))?;

// Filter after loading
let filtered = filter_case_size(&log, 1, 1000)?;  // 1-1000 events per case
```

---

## Troubleshooting

### Error: "No traces found"

**Cause:** Log structure incorrect
**Fix:** Ensure format is:
```xml
<log>
  <trace>
    <string key="concept:name" value="case_id"/>
    <event>...</event>
  </trace>
</log>
```

### Error: "Invalid timestamp format"

**Cause:** Timestamp not ISO8601
**Fix:** Use format: `2026-01-01T12:00:00Z`

### Error: "File not found"

**Cause:** Path incorrect or file missing
**Fix:** Check file exists: `ls -la mylog.xes`

### Error: "XML parsing error"

**Cause:** Malformed XML
**Fix:** Validate: `xmllint --noout mylog.xes`

### Slow loading (>10s for 1M events)

**Cause:** Large file
**Fix:**
1. Check disk (SSD vs HDD)
2. Profile parsing: `cargo build --release`
3. Pre-filter: keep only recent traces

---

## Example: Multi-Language Process

Event log for an order processing system with multiple languages:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<log xes.version="1.0">
  <string key="concept:name" value="International Order Process"/>

  <trace>
    <string key="concept:name" value="Order_DE_001"/>
    <string key="customer_country" value="Germany"/>
    <string key="currency" value="EUR"/>

    <event>
      <string key="concept:name" value="Bestellung"/>
      <date key="time:timestamp" value="2026-01-01T10:00:00Z"/>
      <string key="org:resource" value="Shop_DE"/>
      <float key="amount" value="150.50"/>
    </event>

    <event>
      <string key="concept:name" value="Zahlung"/>
      <date key="time:timestamp" value="2026-01-01T10:05:00Z"/>
      <string key="payment_method" value="Kreditkarte"/>
    </event>

    <event>
      <string key="concept:name" value="Versand"/>
      <date key="time:timestamp" value="2026-01-02T08:00:00Z"/>
      <string key="carrier" value="DHL"/>
      <string key="tracking" value="DE123456789"/>
    </event>
  </trace>

  <trace>
    <string key="concept:name" value="Order_US_001"/>
    <string key="customer_country" value="United States"/>
    <string key="currency" value="USD"/>

    <event>
      <string key="concept:name" value="Order"/>
      <date key="time:timestamp" value="2026-01-01T15:00:00Z"/>
      <string key="org:resource" value="Shop_US"/>
      <float key="amount" value="200.00"/>
    </event>

    <event>
      <string key="concept:name" value="Payment"/>
      <date key="time:timestamp" value="2026-01-01T15:02:00Z"/>
      <string key="payment_method" value="Credit Card"/>
    </event>

    <event>
      <string key="concept:name" value="Shipment"/>
      <date key="time:timestamp" value="2026-01-02T09:00:00Z"/>
      <string key="carrier" value="FedEx"/>
      <string key="tracking" value="US987654321"/>
    </event>
  </trace>
</log>
```

---

## See Also

- **Tutorial:** First Process Mining Analysis
- **Reference:** Rust API Complete
- **How-To:** Extract Process Patterns

---

**XES Standard:** ISO/IEC 20652:2013
**Last Updated:** 2026-03-25
**pm4py-rust Support:** Full (read), Full (write)
