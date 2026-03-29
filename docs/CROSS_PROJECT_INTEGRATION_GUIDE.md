# Cross-Project Integration Guide: pm4py-rust with BusinessOS, OSA, Canopy

**Status:** Complete (2026-03-24)
**Version:** 1.0.0
**Scope:** Integration patterns, HTTP API contracts, testing strategies

## Overview

This guide documents how pm4py-rust integrates with:

1. **BusinessOS** — Go backend + SvelteKit frontend (AI business operating system)
2. **OSA** — Elixir/OTP multi-agent system (Optimal System Architecture)
3. **Canopy** — Elixir/Phoenix workspace protocol (execution engine)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Application Layer                      │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────────┐  ┌──────────────┐  ┌──────────┐  │
│  │  BusinessOS      │  │     OSA      │  │  Canopy  │  │
│  │  Desktop App     │  │  Agents      │  │ Workspace│  │
│  │  (Go/Svelte)     │  │  (Elixir)    │  │(Elixir)  │  │
│  └────────┬─────────┘  └──────┬───────┘  └─────┬────┘  │
│           │                   │                 │       │
│           └───────────────────┼─────────────────┘       │
│                               │                         │
│                    HTTP JSON RPC / REST                 │
│                               │                         │
└───────────────────────────────┼─────────────────────────┘
                                │
                    ┌───────────▼────────────┐
                    │  pm4py-rust HTTP API  │
                    │   (Axum + Prometheus) │
                    └───────────┬────────────┘
                                │
                ┌───────────────┼───────────────┐
                │               │               │
         ┌──────▼────┐  ┌──────▼────┐  ┌──────▼────┐
         │ Discovery │  │Conformance│  │Statistics │
         │Algorithms │  │Checking   │  │Analytics  │
         └───────────┘  └───────────┘  └───────────┘
                │               │               │
         ┌──────▼───────────────▼───────────────▼───────┐
         │    Memory Optimization Layer                │
         │  (StringIntern, Arc, CacheAlign, Pooling)  │
         └─────────────────────────────────────────────┘
```

## BusinessOS Integration

### Overview

BusinessOS uses pm4py-rust for:
- **Process discovery** — Generate Petri nets from event logs
- **Compliance checking** — Validate execution against designed processes
- **Performance analytics** — Identify bottlenecks and anomalies

### HTTP Endpoints Used

**Base URL:** `http://localhost:8089`

| Endpoint | Purpose | Used By |
|----------|---------|---------|
| `POST /api/discovery/alpha` | Discover model from log | Desktop → discovery pipeline |
| `POST /api/conformance/token-replay` | Check fitness | Compliance module |
| `POST /api/statistics` | Analyze log | Analytics dashboard |
| `GET /api/health` | Readiness probe | K8s orchestration |
| `GET /metrics` | Prometheus scrape | Monitoring |

### Integration Points

#### 1. Discovery Pipeline

**Location:** `BusinessOS/desktop/backend-go/internal/handlers/discovery.go`

```go
// Pseudocode: Call pm4py-rust discovery
func (h *DiscoveryHandler) DiscoverProcess(ctx context.Context, log EventLog) {
    // POST to pm4py-rust
    req := DiscoveryRequest{
        EventLog: log,
        Variant: "alpha",
    }

    resp, err := httpClient.Post(
        "http://pm4py-rust:8089/api/discovery/alpha",
        "application/json",
        req,
    )

    // Response: PetriNet + metadata
    petriNet := resp.PetriNet
    executionTimeMs := resp.ExecutionTimeMs

    // Store in database for compliance checks
    h.db.StorePetriNet(petriNet, eventLogId)
}
```

**Code Path:** `BusinessOS/desktop/backend-go/internal/handlers/discovery.go`

#### 2. Compliance Module

**Location:** `BusinessOS/desktop/backend-go/internal/services/compliance_service.go`

```go
// Pseudocode: Conformance checking
func (s *ComplianceService) CheckConformance(
    ctx context.Context,
    eventLog EventLog,
    designedModel PetriNet,
) ConformanceMetrics {
    // POST to pm4py-rust
    req := ConformanceRequest{
        EventLog: eventLog,
        PetriNet: designedModel,
        Method: "token_replay",
    }

    resp, err := httpClient.Post(
        "http://pm4py-rust:8089/api/conformance/token-replay",
        "application/json",
        req,
    )

    return ConformanceMetrics{
        Fitness: resp.Fitness,
        Precision: resp.Precision,
        Generalization: resp.Generalization,
        IsCompliant: resp.IsConformant,
    }
}
```

**Code Path:** `BusinessOS/desktop/backend-go/internal/services/compliance_service.go`

#### 3. Analytics Dashboard

**Location:** `BusinessOS/frontend/src/routes/analytics/+page.server.ts`

```typescript
// Pseudocode: Get event log statistics
export async function load({ params }) {
    const eventLogId = params.id;
    const eventLog = await db.getEventLog(eventLogId);

    // Call pm4py-rust statistics endpoint
    const stats = await fetch('http://pm4py-rust:8089/api/statistics', {
        method: 'POST',
        body: JSON.stringify({
            event_log: eventLog,
            include_variants: true,
            include_resource_metrics: true,
            include_bottlenecks: true,
        }),
    }).then(r => r.json());

    return {
        activities: stats.activity_frequencies,
        variants: stats.variant_frequencies,
        bottlenecks: stats.bottleneck_activities,
        resources: stats.resource_count,
        executionTimeMs: stats.execution_time_ms,
    };
}
```

**Code Path:** `BusinessOS/frontend/src/routes/analytics/+page.server.ts`

### Integration Testing

**Test Files:**
- `pm4py-rust/tests/businessos_rust_http_integration_tests.rs` — HTTP protocol
- `pm4py-rust/tests/businessos_compliance_integration_test.rs` — Compliance logic
- `pm4py-rust/tests/businessos_integration_test.rs` — Full stack

**Run BusinessOS Integration Tests:**
```bash
# Requires BusinessOS running on localhost:8001
BUSINESSOS_API_BASE=http://localhost:8001 cargo test --test businessos_integration_test
```

### Event Log Format

**JSON Schema:**
```json
{
  "traces": [
    {
      "id": "case_001",
      "events": [
        {
          "activity": "register",
          "timestamp": "2026-03-24T10:00:00Z",
          "resource": "clerk",
          "attributes": {
            "amount": 1000,
            "priority": "high"
          }
        }
      ],
      "attributes": {
        "case_type": "loan",
        "customer_id": "C123"
      }
    }
  ]
}
```

**Code Path for Serialization:** `pm4py-rust/tests/businessos_rust_http_integration_tests.rs:serialize_event_log_to_json()`

## OSA Integration

### Overview

OSA (Optimal System Architecture) uses pm4py-rust for:
- **Agent activity monitoring** — Trace agent execution
- **Audit trail analysis** — Reconstruct process from logs
- **Policy enforcement** — Model-based compliance checking

### Integration Pattern

**Location:** `OSA/lib/osa/adapters/pm4py_adapter.ex`

```elixir
# Pseudocode: OSA adapter for pm4py-rust
defmodule OSA.Adapters.PM4PyAdapter do
  @pm4py_base "http://pm4py-rust:8089/api"

  def discover_model(event_log) do
    # Call pm4py-rust discovery
    HTTPClient.post(
      "#{@pm4py_base}/discovery/alpha",
      Jason.encode!(event_log)
    )
    |> Jason.decode!()
  end

  def check_conformance(event_log, petri_net) do
    HTTPClient.post(
      "#{@pm4py_base}/conformance/token-replay",
      Jason.encode!(%{
        "event_log" => event_log,
        "petri_net" => petri_net,
        "method" => "token_replay"
      })
    )
    |> Jason.decode!()
  end

  def get_statistics(event_log) do
    HTTPClient.post(
      "#{@pm4py_base}/statistics",
      Jason.encode!(%{
        "event_log" => event_log,
        "include_variants" => true,
        "include_bottlenecks" => true
      })
    )
    |> Jason.decode!()
  end
end
```

**Code Path:** `OSA/lib/osa/adapters/pm4py_adapter.ex`

### Usage in Agent Loop

**Location:** `OSA/lib/osa/agent/loop.ex`

```elixir
# Pseudocode: Agent hooked execution calls pm4py-rust
defmodule OSA.Agent.Loop do
  def run_with_hooks(agent, task) do
    # Execute task
    result = execute(agent, task)

    # Collect audit trail
    audit_trail = collect_audit_trail(agent)

    # Call pm4py-rust to analyze
    stats = PM4PyAdapter.get_statistics(audit_trail)

    # Store statistics
    Agent.update(agent, fn state ->
      %{state | last_statistics: stats}
    end)

    result
  end
end
```

**Code Path:** `OSA/lib/osa/agent/loop.ex`

### Integration Testing

**Test Files:**
- `pm4py-rust/tests/osa_integration_test.rs` — Agent coordination
- `pm4py-rust/tests/cross_project_integration_tests.rs` — Multi-project

**Run OSA Integration Tests:**
```bash
# Requires OSA running on localhost:8089
cargo test --test osa_integration_test
```

### Audit Trail Format

OSA sends audit trails as EventLog JSON:
```json
{
  "traces": [
    {
      "id": "agent_session_abc123",
      "events": [
        {
          "activity": "task_dispatch",
          "timestamp": "2026-03-24T10:00:00Z",
          "resource": "agent_1",
          "attributes": {
            "task_id": "task_xyz",
            "priority": 5
          }
        },
        {
          "activity": "task_execute",
          "timestamp": "2026-03-24T10:00:01Z",
          "resource": "agent_1",
          "attributes": {
            "task_id": "task_xyz",
            "status": "success"
          }
        }
      ]
    }
  ]
}
```

## Canopy Integration

### Overview

Canopy (workspace protocol) uses pm4py-rust for:
- **Workflow monitoring** — Track executed processes
- **Decision support** — Suggest optimizations
- **Performance analytics** — SLA tracking

### Integration Pattern

**Location:** `canopy/backend/lib/canopy/adapters/pm4py.ex`

```elixir
# Pseudocode: Canopy adapter for pm4py-rust
defmodule Canopy.Adapters.PM4Py do
  @pm4py_base "http://pm4py-rust:8089/api"

  def analyze_workflow(workflow_execution) do
    # Convert Canopy workflow to event log format
    event_log = convert_to_event_log(workflow_execution)

    # Get statistics
    stats = HTTPClient.post(
      "#{@pm4py_base}/statistics",
      Jason.encode!(event_log)
    )
    |> Jason.decode!()

    # Extract metrics for dashboard
    %{
      "activity_frequencies" => activities,
      "variant_frequencies" => variants,
      "bottleneck_activities" => bottlenecks,
      "execution_time_ms" => duration
    } = stats

    {activities, variants, bottlenecks}
  end

  defp convert_to_event_log(workflow_execution) do
    # Convert Canopy heartbeat structure to pm4py EventLog
    %{
      "traces" => [
        %{
          "id" => workflow_execution.id,
          "events" => Enum.map(workflow_execution.steps, fn step ->
            %{
              "activity" => step.name,
              "timestamp" => DateTime.to_iso8601(step.timestamp),
              "resource" => step.executor,
              "attributes" => step.context
            }
          end)
        }
      ]
    }
  end
end
```

**Code Path:** `canopy/backend/lib/canopy/adapters/pm4py.ex`

### Heartbeat Integration

**Location:** `canopy/backend/lib/canopy/heartbeat.ex`

```elixir
# Pseudocode: Canopy heartbeat triggers pm4py analysis
defmodule Canopy.Heartbeat do
  def emit_heartbeat(workspace) do
    # Standard heartbeat
    heartbeat = %{
      "timestamp" => DateTime.utc_now(),
      "workspace_id" => workspace.id,
      "agents" => list_agents(workspace),
      # ... heartbeat fields
    }

    # OPTIONAL: Include pm4py statistics
    if should_analyze_workflow?(workspace) do
      stats = PM4Py.analyze_workflow(workspace.execution)
      heartbeat = Map.put(heartbeat, "workflow_stats", stats)
    end

    broadcast_heartbeat(heartbeat)
  end
end
```

**Code Path:** `canopy/backend/lib/canopy/heartbeat.ex`

### Integration Testing

**Test Files:**
- `pm4py-rust/tests/canopy_integration_test.rs` — Workspace protocol
- `pm4py-rust/tests/extended_discovery_integration_tests.rs` — Discovery patterns

**Run Canopy Integration Tests:**
```bash
# Requires Canopy running on localhost:9089
cargo test --test canopy_integration_test
```

## Shared Data Contracts

### EventLog JSON Schema

All projects use this EventLog format:

```typescript
interface EventLog {
  traces: Trace[];
  attributes?: Record<string, string | number | boolean>;
}

interface Trace {
  id: string;                           // Case ID
  events: Event[];
  attributes?: Record<string, any>;     // Case-level metadata
}

interface Event {
  activity: string;                     // Activity name
  timestamp: string;                    // ISO 8601 format
  resource?: string;                    // Who performed activity
  attributes?: Record<string, any>;     // Event-level metadata
}
```

### PetriNet JSON Schema

```typescript
interface PetriNet {
  places: Place[];
  transitions: Transition[];
  arcs: Arc[];
  initial_place?: string;               // Place ID
  final_place?: string;                 // Place ID
}

interface Place {
  id: string;
  name: string;
  initial_marking: number;
}

interface Transition {
  id: string;
  name: string;
  label?: string;                       // User-facing label
}

interface Arc {
  from: string;                         // Place or transition ID
  to: string;                           // Place or transition ID
  weight: number;                       // Default 1
}
```

### ConformanceResult JSON Schema

```typescript
interface ConformanceResult {
  is_conformant: boolean;               // fitness ≥ 0.8
  fitness: number;                      // 0-1: trace coverage
  precision: number;                    // 0-1: model specificity
  generalization: number;               // 0-1: generalization ability
  method: string;                       // "token_replay", "alignment"
  execution_time_ms: number;
}
```

## Error Handling

### HTTP Status Codes

| Status | Meaning | Example |
|--------|---------|---------|
| 200 | Success | ✓ Discovery returned model |
| 400 | Bad Request | Invalid event log format |
| 500 | Server Error | Algorithm failure |

### Error Response Format

```json
{
  "error": "EventLog parsing failed",
  "details": "Expected 'traces' field in JSON",
  "status": 400
}
```

**Code Path:** `pm4py-rust/src/http/businessos_api.rs::ApiError`

### Retry Strategy

**Recommended for clients:**
```
Attempt 1: Immediate
Attempt 2: 100ms wait
Attempt 3: 1s wait
Max retries: 3
Timeout: 30s per request
```

## Monitoring & Observability

### Prometheus Metrics

All projects can scrape metrics from pm4py-rust:

**Endpoint:** `GET http://pm4py-rust:8089/metrics`

**Key Metrics:**
```
pm4py_discovery_duration_seconds_sum    — Total discovery time
pm4py_conformance_duration_seconds_sum  — Total conformance time
pm4py_active_requests                   — Current request count
pm4py_event_log_size_bytes              — Memory used by logs
pm4py_errors_total{error_type="..."}    — Error counts
```

### Logging

pm4py-rust logs to stdout in JSON format:
```json
{
  "timestamp": "2026-03-24T10:00:01Z",
  "level": "info",
  "message": "Discovery request received",
  "request_id": "req_abc123",
  "algorithm": "alpha_miner",
  "event_count": 1250,
  "duration_ms": 145
}
```

### Health Checks

All projects should periodically call health endpoint:

```
GET http://pm4py-rust:8089/api/health

Response (200):
{
  "status": "healthy",
  "version": "0.1.0",
  "timestamp": "2026-03-24T14:35:22.123Z"
}
```

## Deployment Topology

### Local Development

```
┌─────────────────────────────┐
│ Localhost                   │
├─────────────────────────────┤
│ :8001 — BusinessOS (Go)     │
│ :8089 — pm4py-rust          │
│ :9089 — OSA (Elixir)        │
│ :9089 — Canopy (Elixir)     │
└─────────────────────────────┘
```

**Docker Compose Example:**
```yaml
version: "3.8"

services:
  pm4py-rust:
    image: pm4py-rust:latest
    ports:
      - "8089:8089"
    environment:
      PM4PY_HTTP_HOST: 0.0.0.0
      PM4PY_HTTP_PORT: 8089

  businessos:
    image: businessos:latest
    ports:
      - "8001:8001"
    environment:
      PM4PY_API_BASE: http://pm4py-rust:8089

  osa:
    image: osa:latest
    ports:
      - "8089:8089"
    environment:
      PM4PY_API_BASE: http://pm4py-rust:8089
```

### Production (Kubernetes)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pm4py-rust
spec:
  replicas: 3
  selector:
    matchLabels:
      app: pm4py-rust
  template:
    metadata:
      labels:
        app: pm4py-rust
    spec:
      containers:
      - name: pm4py-rust
        image: pm4py-rust:latest
        ports:
        - containerPort: 8089
        livenessProbe:
          httpGet:
            path: /api/health
            port: 8089
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/health
            port: 8089
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: pm4py-rust
spec:
  type: ClusterIP
  ports:
  - port: 8089
    targetPort: 8089
  selector:
    app: pm4py-rust
```

## Testing Strategy

### 1. Unit Tests (pm4py-rust)

Test individual algorithms, memory optimization layers.

```bash
cargo test --lib
```

### 2. Integration Tests

Test HTTP API contracts without external services.

```bash
cargo test --test '*_integration_test'
```

### 3. Cross-Project Tests

Test communication between pm4py-rust and BusinessOS/OSA/Canopy.

```bash
# Requires all services running
cargo test --test cross_project_integration_tests
```

### 4. End-to-End Tests

Full workflow from data ingestion to metrics export.

```bash
# Requires full stack running
bash tests/run-integration-tests.sh
```

### 5. Load Tests

Python script for concurrent testing.

```bash
pip install -r tests/requirements-integration.txt
python tests/businessos_http_integration_tests.py --concurrency=10 --requests=1000
```

## Configuration Reference

### Environment Variables

```bash
# HTTP Server
PM4PY_HTTP_HOST=0.0.0.0
PM4PY_HTTP_PORT=8089

# Metrics
PM4PY_METRICS_ENABLED=true
PM4PY_METRICS_PORT=9090

# Memory
PM4PY_MAX_EVENT_LOG_SIZE=1073741824     # 1GB
PM4PY_STRING_INTERN_THRESHOLD=10        # Min occurrences to intern

# Logging
PM4PY_LOG_LEVEL=info
PM4PY_LOG_FORMAT=json
```

### BusinessOS Configuration

```go
// businessos/desktop/backend-go/internal/config/pm4py.go
type PM4PyConfig struct {
    BaseURL           string        // http://pm4py-rust:8089
    Timeout           time.Duration // 30s
    MaxEventLogSize   int64         // 1GB
    RetryAttempts     int           // 3
    RetryBackoff      time.Duration // 100ms
}
```

### OSA Configuration

```elixir
# osa/config/config.exs
config :osa, :pm4py,
  base_url: System.get_env("PM4PY_API_BASE", "http://localhost:8089"),
  timeout: :timer.seconds(30),
  pool_size: 10,
  pool_overflow: 5
```

### Canopy Configuration

```elixir
# canopy/config/config.exs
config :canopy, :pm4py,
  enabled: true,
  base_url: System.get_env("PM4PY_API_BASE", "http://localhost:8089"),
  timeout: :timer.seconds(30)
```

## Best Practices

1. **Always call health endpoint** before running large operations
2. **Cache discovered models** to avoid redundant discovery calls
3. **Use connection pooling** for high-throughput scenarios
4. **Monitor execution times** and alert if >5s
5. **Batch requests** when processing multiple logs
6. **Handle rate limiting** gracefully (503 responses)
7. **Use Prometheus metrics** for performance analysis
8. **Test error paths** (400, 500 responses)

## Troubleshooting

### pm4py-rust not responding

```bash
# Check if service is running
curl http://localhost:8089/api/health

# Check logs
docker logs pm4py-rust

# Check metrics for errors
curl http://localhost:8089/metrics | grep errors_total
```

### Event log parsing fails

```bash
# Validate JSON format
cat event_log.json | jq . > /dev/null

# Check for required fields
jq '.traces[0].events[0]' event_log.json
```

### Slow discovery

```bash
# Monitor metrics
curl http://localhost:8089/metrics | grep discovery_duration

# Check memory usage
curl http://localhost:8089/metrics | grep memory_usage

# Try Inductive Miner (faster for complex logs)
# Use variant: "inductive" instead of "alpha"
```

## See Also

- **HTTP API Reference:** `VISION_2030_PHASE2_HTTP_INTEGRATION.md`
- **Memory Optimization:** `MEMORY_OPTIMIZATION_ARCHITECTURE.md`
- **Performance Tuning:** `PERFORMANCE_TUNING_GUIDE.md`
- **BusinessOS Architecture:** `BusinessOS/docs/ARCHITECTURE.md`
- **OSA Architecture:** `OSA/docs/ARCHITECTURE.md`
- **Canopy Architecture:** `canopy/docs/ARCHITECTURE.md`

---

**Last Updated:** 2026-03-24
**Version:** 1.0.0
**Maintainer:** ChatmanGPT Vision 2030 Team
