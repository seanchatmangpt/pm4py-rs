# PM4Py REST API Reference

> Production-grade REST API for process mining on high-performance Rust backends.
>
> **Version:** 1.0.0
> **Base URL:** `http://localhost:8080/api/v1`

---

## Quick Reference

| Method | Endpoint | Purpose |
|--------|----------|---------|
| `GET` | `/health` | Health check |
| `POST` | `/discover` | Mine process model from log |
| `POST` | `/conform` | Check log conformance against model |
| `POST` | `/analyze` | Analyze model structure |
| `POST` | `/stats` | Extract log statistics |

---

## Authentication

All endpoints (except `/health`) require an API key in the request header:

```bash
curl -H "X-API-Key: your-api-key-here" \
  https://api.chatmangpt.com/api/v1/discover
```

**API Key Format:**
- 32+ character alphanumeric string
- Tenant-scoped (one key per organization)
- Rotate monthly for security

**Obtaining an API Key:**
1. Register at https://dashboard.chatmangpt.com
2. Create new application
3. Copy API key from credentials panel
4. Store securely (e.g., environment variable)

---

## Rate Limiting

**Limit:** 10,000 requests per hour per API key

**Headers returned with every request:**

```
X-RateLimit-Limit: 10000
X-RateLimit-Remaining: 9950
X-RateLimit-Reset: 2026-03-24T11:00:00Z
```

**When limit exceeded (HTTP 429):**
```json
{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "10000 requests/hour limit exceeded. Reset at 2026-03-24T11:00:00Z",
  "request_id": "req_abc123"
}
```

**Best practices:**
- Cache results when possible
- Batch requests efficiently
- Implement exponential backoff for retries
- Monitor `X-RateLimit-Remaining` header

---

## Error Handling

All errors follow a consistent structure:

```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable description",
  "details": {
    "field": "path.to.field",
    "constraint": "Detailed constraint info"
  },
  "timestamp": "2026-03-24T10:00:00Z",
  "request_id": "req_unique_id"
}
```

### Common HTTP Status Codes

| Code | Meaning | Recovery |
|------|---------|----------|
| 200 | Success | Operation completed |
| 400 | Bad Request | Fix request and retry |
| 401 | Unauthorized | Check API key |
| 429 | Rate Limited | Wait for reset time |
| 500 | Server Error | Retry with exponential backoff |
| 503 | Service Unavailable | Maintenance in progress |

### Common Error Codes

| Code | HTTP | Cause | Solution |
|------|------|-------|----------|
| `VALIDATION_ERROR` | 400 | Invalid input format | Check request schema against OpenAPI spec |
| `INVALID_LOG_FORMAT` | 400 | Malformed event log | Ensure events have case_id, activity, timestamp |
| `INVALID_MODEL` | 400 | Invalid Petri Net | Model must have valid places, transitions, arcs |
| `INVALID_API_KEY` | 401 | Missing/expired API key | Regenerate key in dashboard |
| `INSUFFICIENT_PERMISSIONS` | 403 | API key lacks endpoint permission | Request higher permission tier |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests | Wait until reset time |
| `INTERNAL_SERVER_ERROR` | 500 | Unexpected error | Retry or contact support |

---

## Endpoints

### Health Check

```
GET /health
```

Simple health probe. Requires no authentication.

**Response (200 OK):**
```json
{
  "status": "ok",
  "version": "0.3.0",
  "timestamp": "2026-03-24T10:00:00Z"
}
```

**Use cases:**
- Deployment health checks
- Monitoring/alerting
- Load balancer probes

---

### Process Discovery

```
POST /discover
```

**Description:**
Mines a process model (Petri Net) from an event log using specified algorithm.

**Available Algorithms:**

| Algorithm | Best For | Strengths | Limitations |
|-----------|----------|-----------|-------------|
| `alpha` | Simple, structured processes | Foundational, well-understood | Struggles with loops, noise |
| `inductive` | Complex, recursive processes | Excellent loop handling, robust | Slower on very large logs |
| `heuristic` | Noisy, real-world processes | Frequency-based filtering | May oversimplify |
| `dfg` | Quick overview, graph analysis | Fast, minimal | No formal semantics |
| `causal_net` | Causal dependency analysis | Shows cause-effect relationships | Complex output |
| `split_miner` | Advanced branching detection | State-of-the-art accuracy | Computationally intensive |

**Request:**

```json
{
  "log": {
    "events": [
      {
        "case_id": "order_001",
        "activity": "receive_order",
        "timestamp": "2026-03-24T10:00:00Z",
        "resource": "clerk",
        "attributes": {
          "amount": 1000,
          "priority": "high"
        }
      },
      {
        "case_id": "order_001",
        "activity": "process_order",
        "timestamp": "2026-03-24T10:30:00Z",
        "resource": "processor"
      }
    ],
    "format": "json"
  },
  "algorithm": "inductive",
  "parameters": {
    "frequency_threshold": 0.1,
    "min_edge_frequency": 1
  }
}
```

**Response (200 OK):**

```json
{
  "model": {
    "places": ["p0", "p1", "p2", "p3"],
    "transitions": ["receive_order", "process_order", "ship_order"],
    "arcs": [
      {
        "source": "p0",
        "target": "receive_order",
        "type": "place_to_transition"
      },
      {
        "source": "receive_order",
        "target": "p1",
        "type": "transition_to_place"
      }
    ],
    "initial_marking": {"p0": 1},
    "final_marking": {"p3": 1}
  },
  "algorithm": "inductive",
  "num_places": 4,
  "num_transitions": 3,
  "processing_time_ms": 234
}
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `frequency_threshold` | float | 0.1 | Min edge frequency (0-1). Only edges with freq >= threshold included |
| `min_edge_frequency` | int | 1 | Min count of edge occurrences |
| `max_activity_distance` | int | 10 | Max distance for activity relationships |

**Error Examples:**

```json
// Missing required field
{
  "error": "VALIDATION_ERROR",
  "message": "Missing required field: log",
  "details": {
    "field": "log",
    "constraint": "required"
  },
  "request_id": "req_abc123"
}
```

```json
// Empty event log
{
  "error": "INVALID_LOG_FORMAT",
  "message": "Event log must contain at least 1 event",
  "details": {
    "field": "log.events",
    "constraint": "minItems: 1"
  },
  "request_id": "req_def456"
}
```

---

### Conformance Checking

```
POST /conform
```

**Description:**
Evaluates fitness, precision, generalization, and simplicity of an event log against a Petri Net model.

**Metrics Explained:**

| Metric | Range | Interpretation |
|--------|-------|-----------------|
| **Fitness** | 0-1 | Proportion of log traces perfectly replayed by model. 1.0 = all traces fit |
| **Precision** | 0-1 | Model specificity. How many allowed behaviors aren't in log. 1.0 = no overfitting |
| **Generalization** | 0-1 | Model coverage. How diverse behaviors model captures. High = more flexibility |
| **Simplicity** | 0-1 | Model complexity. Higher = simpler model (prefer simpler when possible) |

**Request:**

```json
{
  "log": {
    "events": [
      {
        "case_id": "order_001",
        "activity": "receive_order",
        "timestamp": "2026-03-24T10:00:00Z"
      },
      {
        "case_id": "order_001",
        "activity": "process_order",
        "timestamp": "2026-03-24T10:30:00Z"
      }
    ]
  },
  "model": {
    "places": ["p0", "p1", "p2"],
    "transitions": ["receive_order", "process_order"],
    "arcs": [
      {"source": "p0", "target": "receive_order", "type": "place_to_transition"},
      {"source": "receive_order", "target": "p1", "type": "transition_to_place"}
    ],
    "initial_marking": {"p0": 1},
    "final_marking": {"p2": 1}
  },
  "variant": "token_replay"
}
```

**Response (200 OK):**

```json
{
  "result": {
    "fitness": 0.95,
    "precision": 0.92,
    "generalization": 0.88,
    "simplicity": 0.85,
    "deviant_traces": ["order_002", "order_005"],
    "conformance_details": {
      "order_001": {
        "conforms": true,
        "remaining_tokens": 0,
        "enabled_transitions": []
      },
      "order_002": {
        "conforms": false,
        "remaining_tokens": 2,
        "enabled_transitions": ["receive_order"]
      }
    }
  },
  "model_summary": {
    "num_places": 3,
    "num_transitions": 2,
    "num_arcs": 4
  },
  "processing_time_ms": 156
}
```

**Variants:**

| Variant | Method | Best For |
|---------|--------|----------|
| `token_replay` | Token replay | Balance of speed and accuracy |
| `alignment` | Optimal alignment | Maximum accuracy, slower |

---

### Model Analysis

```
POST /analyze
```

**Description:**
Analyzes Petri Net structure for soundness, behavioral profiles, and deadlock potential.

**Request:**

```json
{
  "model": {
    "places": ["p0", "p1", "p2"],
    "transitions": ["A", "B", "C"],
    "arcs": [
      {"source": "p0", "target": "A", "type": "place_to_transition"},
      {"source": "A", "target": "p1", "type": "transition_to_place"},
      {"source": "p1", "target": "B", "type": "place_to_transition"},
      {"source": "B", "target": "p2", "type": "transition_to_place"}
    ],
    "initial_marking": {"p0": 1},
    "final_marking": {"p2": 1}
  }
}
```

**Response (200 OK):**

```json
{
  "is_sound": true,
  "behavioral_profile": {
    "causality": [
      {
        "activity_a": "A",
        "activity_b": "B",
        "relationship": "sequential"
      },
      {
        "activity_a": "B",
        "activity_b": "C",
        "relationship": "sequential"
      }
    ],
    "strongly_connected_components": 1,
    "longest_path": 3
  },
  "deadlock_potential": [],
  "processing_time_ms": 89
}
```

**Definitions:**

- **Sound:** Model has proper initial/final markings, no deadlocks, proper termination
- **Behavioral Profile:** Activity relationships (sequential, concurrent, inverse sequential)
- **Strongly Connected Components:** Number of irreducible process subgraphs
- **Longest Path:** Maximum activity sequence length
- **Deadlock Potential:** Configurations that could cause deadlock

---

### Log Statistics

```
POST /stats
```

**Description:**
Extracts comprehensive statistical metrics from event log.

**Request:**

```json
{
  "log": {
    "events": [
      {
        "case_id": "order_001",
        "activity": "receive",
        "timestamp": "2026-03-24T10:00:00Z"
      },
      {
        "case_id": "order_001",
        "activity": "process",
        "timestamp": "2026-03-24T10:30:00Z"
      }
    ]
  }
}
```

**Response (200 OK):**

```json
{
  "stats": {
    "num_traces": 1000,
    "num_events": 5420,
    "num_activities": 23,
    "trace_length_mean": 5.42,
    "trace_length_std": 2.15,
    "trace_length_min": 2,
    "trace_length_max": 18,
    "case_duration_mean": "PT2H30M",
    "case_duration_std": "PT45M",
    "activities": [
      {
        "name": "receive",
        "frequency": 1000,
        "mean_duration": "PT5M"
      },
      {
        "name": "process",
        "frequency": 998,
        "mean_duration": "PT25M"
      }
    ],
    "variants": [
      {
        "trace_pattern": ["receive", "process", "complete"],
        "frequency": 850,
        "percentage": 85.0
      },
      {
        "trace_pattern": ["receive", "process", "rework", "complete"],
        "frequency": 150,
        "percentage": 15.0
      }
    ]
  },
  "processing_time_ms": 342
}
```

---

## Data Formats

### Event Log Format (JSON)

```json
{
  "events": [
    {
      "case_id": "string",          // Required: unique trace ID
      "activity": "string",         // Required: activity/event name
      "timestamp": "2026-03-24T10:00:00Z", // Required: ISO 8601 timestamp
      "resource": "string",         // Optional: who performed activity
      "cost": 50.00,                // Optional: activity cost
      "attributes": {               // Optional: custom fields
        "priority": "high",
        "amount": 1000
      }
    }
  ],
  "format": "json"                  // Optional: json|csv|xes
}
```

### Event Log Format (CSV)

When submitting CSV logs, convert to JSON first with:
```json
{
  "events": [
    {"case_id": "...", "activity": "...", "timestamp": "..."},
    ...
  ],
  "format": "csv"
}
```

### Petri Net Format

```json
{
  "places": ["p0", "p1", "p2"],
  "transitions": ["A", "B", "C"],
  "arcs": [
    {
      "source": "p0",
      "target": "A",
      "type": "place_to_transition"
    },
    {
      "source": "A",
      "target": "p1",
      "type": "transition_to_place"
    }
  ],
  "initial_marking": {"p0": 1},
  "final_marking": {"p2": 1}
}
```

---

## Best Practices

### 1. Request Validation

Always validate your event log before sending:

```json
// DO validate timestamps
{
  "case_id": "order_001",
  "activity": "receive",
  "timestamp": "2026-03-24T10:00:00Z"  // ISO 8601 required
}

// DON'T use ambiguous dates
{
  "case_id": "order_001",
  "activity": "receive",
  "timestamp": "03/24/2026"  // Invalid
}
```

### 2. Batch Processing

For large logs, batch requests:

```bash
# Split 1M events into 10 batches of 100K each
# Process in parallel (respecting rate limits)
for batch in {1..10}; do
  curl -X POST https://api.chatmangpt.com/api/v1/discover \
    -H "X-API-Key: $API_KEY" \
    -d @batch_${batch}.json
done
```

### 3. Caching Results

Cache expensive operations:

```python
import hashlib
import json

def get_model(log, algorithm):
    # Create cache key from log hash
    log_hash = hashlib.sha256(
        json.dumps(log, sort_keys=True).encode()
    ).hexdigest()

    cache_key = f"model_{algorithm}_{log_hash}"

    # Check cache first
    cached = cache.get(cache_key)
    if cached:
        return cached

    # Fetch from API
    response = api.discover(log=log, algorithm=algorithm)

    # Cache for 7 days
    cache.set(cache_key, response, ttl=604800)
    return response
```

### 4. Error Handling

Implement robust error handling:

```python
import time
from typing import Optional

def discover_with_retry(log: dict, max_retries: int = 3) -> Optional[dict]:
    for attempt in range(max_retries):
        try:
            response = api.discover(log)
            return response
        except RateLimitError as e:
            # Wait until reset time
            wait_time = (e.reset_time - time.time()) + 1
            time.sleep(wait_time)
        except (ConnectionError, TimeoutError) as e:
            # Exponential backoff
            wait_time = 2 ** attempt
            time.sleep(wait_time)
        except ValidationError as e:
            # Fix validation errors
            log = fix_validation_issues(log, e.details)
            # Don't retry, continue loop

    return None
```

### 5. Rate Limit Management

Monitor and respect rate limits:

```python
class RateLimitAwareClient:
    def __init__(self, api_key: str):
        self.api_key = api_key
        self.remaining = 10000
        self.reset_time = None

    def request(self, method: str, path: str, data: dict) -> dict:
        if self.remaining < 100:  # Warn when low
            print(f"Warning: {self.remaining} requests remaining")

        response = self._make_request(method, path, data)

        # Update rate limit info
        self.remaining = int(response.headers.get('X-RateLimit-Remaining', 0))
        self.reset_time = response.headers.get('X-RateLimit-Reset')

        return response.json()
```

---

## Versioning

Current API version: **v1**

Future versions will use `/api/v2`, etc.

**Deprecation policy:**
- API versions supported for 12+ months
- Deprecation notices 6 months in advance
- Breaking changes only in major versions

---

## Support

**Documentation:** https://github.com/seanchatmangpt/pm4py-rust/docs
**Issues:** https://github.com/seanchatmangpt/pm4py-rust/issues
**Email:** info@chatmangpt.com
**Response time:** 24 hours (business days)

---

## API Examples by Language

See the following for complete working examples:
- **Python:** `examples/http_client.py`
- **JavaScript/Node:** `examples/http_client.js`
- **Rust:** `examples/discover_example.rs`, `examples/conform_example.rs`
- **cURL:** `examples/http_examples.sh`
