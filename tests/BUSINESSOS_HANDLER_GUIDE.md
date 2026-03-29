# BusinessOS HTTP Handler Implementation Guide

This guide shows how to implement the required HTTP endpoints in BusinessOS to support pm4py-rust integration tests.

## Required Endpoints

The integration tests expect these endpoints in BusinessOS:

```
POST   /api/logs/upload                    — Upload event logs
POST   /api/discovery/{algorithm}          — Run discovery
GET    /api/discovery/results/{id}         — Retrieve results
GET    /api/logs/{id}                      — Retrieve uploaded log
POST   /api/conformance/check              — Check conformance
```

## Implementation in Go (BusinessOS Backend)

### 1. Add PM4PY Handler

**File:** `internal/handlers/pm4py.go`

```go
package handlers

import (
	"encoding/json"
	"io"
	"net/http"
	"path/filepath"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/rhl/businessos-backend/internal/database"
	"github.com/rhl/businessos-backend/internal/services"
)

type PM4PyHandler struct {
	pm4pyService *services.PM4PyService
	db           *database.Queries
}

// NewPM4PyHandler creates a new PM4PyHandler
func NewPM4PyHandler(service *services.PM4PyService, db *database.Queries) *PM4PyHandler {
	return &PM4PyHandler{
		pm4pyService: service,
		db:           db,
	}
}

// EventLog represents an event log JSON format
type EventLog struct {
	Events []Event `json:"events"`
	Format string  `json:"format"`
}

type Event struct {
	CaseID     string            `json:"case_id"`
	Activity   string            `json:"activity"`
	Timestamp  string            `json:"timestamp"`
	Resource   string            `json:"resource,omitempty"`
	Attributes map[string]interface{} `json:"attributes,omitempty"`
}

// UploadLogRequest is the upload endpoint request
type UploadLogRequest struct {
	Events []Event `json:"events"`
	Format string  `json:"format"`
}

// UploadLogResponse is the upload endpoint response
type UploadLogResponse struct {
	Status     string `json:"status"`
	LogID      string `json:"log_id"`
	EventCount int    `json:"event_count"`
	CaseCount  int    `json:"case_count"`
}

// DiscoveryRequest is the discovery endpoint request
type DiscoveryRequest struct {
	LogID      string                 `json:"log_id"`
	Filters    map[string]interface{} `json:"filters,omitempty"`
	Timeout    int                    `json:"timeout_seconds,omitempty"`
}

// DiscoveryResponse is the discovery endpoint response
type DiscoveryResponse struct {
	Status   string      `json:"status"`
	ResultID string      `json:"result_id"`
	Model    interface{} `json:"model"`
}

// HandleUploadLog handles POST /api/logs/upload
func (h *PM4PyHandler) HandleUploadLog(c *gin.Context) {
	var req UploadLogRequest

	// Try JSON body first
	if err := c.BindJSON(&req); err == nil && len(req.Events) > 0 {
		// Process JSON upload
		logID, caseCount, err := h.pm4pyService.StoreEventLog(c.Request.Context(), req.Events)
		if err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
			return
		}

		c.JSON(http.StatusOK, UploadLogResponse{
			Status:     "ok",
			LogID:      logID,
			EventCount: len(req.Events),
			CaseCount:  caseCount,
		})
		return
	}

	// Try file upload
	file, err := c.FormFile("file")
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Either JSON events or file upload required",
		})
		return
	}

	// Open uploaded file
	src, err := file.Open()
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Failed to open file"})
		return
	}
	defer src.Close()

	// Read file content
	content, err := io.ReadAll(src)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Failed to read file"})
		return
	}

	// Parse based on file extension
	ext := filepath.Ext(file.Filename)
	var events []Event

	switch ext {
	case ".csv":
		events, err = h.pm4pyService.ParseCSV(content)
	case ".xes":
		events, err = h.pm4pyService.ParseXES(content)
	default:
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Unsupported file format: " + ext,
		})
		return
	}

	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// Store events
	logID, caseCount, err := h.pm4pyService.StoreEventLog(c.Request.Context(), events)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, UploadLogResponse{
		Status:     "ok",
		LogID:      logID,
		EventCount: len(events),
		CaseCount:  caseCount,
	})
}

// HandleDiscover handles POST /api/discovery/{algorithm}
func (h *PM4PyHandler) HandleDiscover(c *gin.Context) {
	algorithm := c.Param("algorithm")
	var req DiscoveryRequest

	if err := c.BindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	// Run discovery
	resultID, model, err := h.pm4pyService.Discover(
		c.Request.Context(),
		req.LogID,
		algorithm,
		req.Filters,
	)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, DiscoveryResponse{
		Status:   "ok",
		ResultID: resultID,
		Model:    model,
	})
}

// HandleGetResults handles GET /api/discovery/results/{id}
func (h *PM4PyHandler) HandleGetResults(c *gin.Context) {
	resultID := c.Param("id")

	result, err := h.pm4pyService.GetResult(c.Request.Context(), resultID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "Result not found"})
		return
	}

	c.JSON(http.StatusOK, result)
}

// HandleGetLog handles GET /api/logs/{id}
func (h *PM4PyHandler) HandleGetLog(c *gin.Context) {
	logID := c.Param("id")

	events, err := h.pm4pyService.GetLog(c.Request.Context(), logID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "Log not found"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"log_id": logID,
		"events": events,
	})
}

// HandleConformance handles POST /api/conformance/check
func (h *PM4PyHandler) HandleConformance(c *gin.Context) {
	type ConformanceReq struct {
		LogID string      `json:"log_id"`
		Model interface{} `json:"model"`
	}

	var req ConformanceReq
	if err := c.BindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request"})
		return
	}

	fitness, precision, err := h.pm4pyService.CheckConformance(
		c.Request.Context(),
		req.LogID,
		req.Model,
	)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"status":    "ok",
		"fitness":   fitness,
		"precision": precision,
	})
}
```

### 2. Create PM4PyService

**File:** `internal/services/pm4py_service.go`

```go
package services

import (
	"context"
	"encoding/csv"
	"encoding/json"
	"fmt"
	"io"
	"strings"
	"time"
	"uuid"

	"github.com/rhl/businessos-backend/internal/database"
)

type PM4PyService struct {
	db *database.Queries
}

// NewPM4PyService creates a new PM4PyService
func NewPM4PyService(db *database.Queries) *PM4PyService {
	return &PM4PyService{db: db}
}

// StoreEventLog stores an event log and returns log ID
func (s *PM4PyService) StoreEventLog(
	ctx context.Context,
	events []interface{}, // []Event from handler
) (string, int, error) {
	if len(events) == 0 {
		return "", 0, fmt.Errorf("empty event log")
	}

	logID := "log_" + uuid.New().String()
	caseIDs := make(map[string]bool)

	// Count unique cases
	for _, e := range events {
		event := e.(map[string]interface{})
		if caseID, ok := event["case_id"].(string); ok {
			caseIDs[caseID] = true
		}
	}

	// TODO: Store in database (PostgreSQL)
	// For now, store in cache with TTL

	return logID, len(caseIDs), nil
}

// ParseCSV parses CSV content
func (s *PM4PyService) ParseCSV(content []byte) ([]interface{}, error) {
	reader := csv.NewReader(strings.NewReader(string(content)))
	records, err := reader.ReadAll()
	if err != nil {
		return nil, err
	}

	if len(records) < 2 {
		return nil, fmt.Errorf("CSV must have header row and data")
	}

	// Assume columns: case_id, activity, timestamp, resource
	var events []interface{}
	for i, record := range records[1:] {
		if len(record) < 3 {
			return nil, fmt.Errorf("invalid CSV row %d", i+2)
		}

		event := map[string]interface{}{
			"case_id":    record[0],
			"activity":   record[1],
			"timestamp":  record[2],
			"resource":   "",
			"attributes": map[string]interface{}{},
		}

		if len(record) > 3 {
			event["resource"] = record[3]
		}

		events = append(events, event)
	}

	return events, nil
}

// ParseXES parses XES content (simplified)
func (s *PM4PyService) ParseXES(content []byte) ([]interface{}, error) {
	// For now, return error - real implementation would parse XML
	return nil, fmt.Errorf("XES parsing not yet implemented")
}

// Discover runs discovery algorithm
func (s *PM4PyService) Discover(
	ctx context.Context,
	logID string,
	algorithm string,
	filters map[string]interface{},
) (string, interface{}, error) {
	// TODO: Call pm4py via Python wrapper
	// For now, return mock result
	resultID := "result_" + uuid.New().String()

	model := map[string]interface{}{
		"nodes": []string{"start", "end"},
		"edges": [][]string{{"start", "end"}},
		"metadata": map[string]interface{}{
			"algorithm":   algorithm,
			"discovered_at": time.Now().Format(time.RFC3339),
		},
	}

	return resultID, model, nil
}

// GetResult retrieves cached discovery result
func (s *PM4PyService) GetResult(ctx context.Context, resultID string) (interface{}, error) {
	// TODO: Retrieve from cache/database
	return nil, fmt.Errorf("result not found")
}

// GetLog retrieves stored event log
func (s *PM4PyService) GetLog(ctx context.Context, logID string) (interface{}, error) {
	// TODO: Retrieve from cache/database
	return nil, fmt.Errorf("log not found")
}

// CheckConformance checks log against model
func (s *PM4PyService) CheckConformance(
	ctx context.Context,
	logID string,
	model interface{},
) (float64, float64, error) {
	// TODO: Call conformance checker
	return 0.95, 0.92, nil
}
```

### 3. Register Routes

**File:** `internal/handlers/routes.go` (add to `registerRoutes`)

```go
// In the RegisterRoutes method, add:
func (h *Handlers) registerPM4PyRoutes(api *gin.RouterGroup, auth gin.HandlerFunc) {
	pm4pyHandler := NewPM4PyHandler(h.pm4pyService, h.queries)

	logs := api.Group("/logs")
	{
		logs.POST("/upload", pm4pyHandler.HandleUploadLog)
		logs.GET("/:id", pm4pyHandler.HandleGetLog)
	}

	discovery := api.Group("/discovery")
	{
		discovery.POST("/:algorithm", pm4pyHandler.HandleDiscover)
		discovery.GET("/results/:id", pm4pyHandler.HandleGetResults)
	}

	conformance := api.Group("/conformance")
	{
		conformance.POST("/check", pm4pyHandler.HandleConformance)
	}
}

// And call in RegisterRoutes:
h.registerPM4PyRoutes(api, auth)
```

### 4. Add to Handlers Struct

**File:** `internal/handlers/handlers.go`

```go
type Handlers struct {
	// ... existing fields ...
	pm4pyService *services.PM4PyService
}

// In NewHandlers:
handlers.pm4pyService = services.NewPM4PyService(queries)
```

## Database Schema (PostgreSQL)

```sql
-- Event logs storage
CREATE TABLE event_logs (
    log_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    event_count INT NOT NULL,
    case_count INT NOT NULL,
    raw_events JSONB NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Discovery results cache
CREATE TABLE discovery_results (
    result_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    log_id UUID NOT NULL REFERENCES event_logs(log_id),
    algorithm VARCHAR(50) NOT NULL,
    discovered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    model JSONB NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    ttl INT DEFAULT 86400  -- 24 hour TTL
);

-- Indexes
CREATE INDEX idx_event_logs_created ON event_logs(created_at DESC);
CREATE INDEX idx_discovery_results_log_id ON discovery_results(log_id);
CREATE INDEX idx_discovery_results_algorithm ON discovery_results(algorithm);
```

## Python Wrapper (Optional but Recommended)

For complex algorithms, call pm4py from Go:

**File:** `internal/services/pm4py_wrapper.go`

```go
package services

import (
	"encoding/json"
	"fmt"
	"os/exec"
)

func (s *PM4PyService) CallPM4Py(algorithm string, logPath string) (interface{}, error) {
	// Call external Python script
	cmd := exec.Command("python3", "scripts/pm4py_bridge.py", algorithm, logPath)

	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("pm4py call failed: %w", err)
	}

	var result interface{}
	if err := json.Unmarshal(output, &result); err != nil {
		return nil, fmt.Errorf("failed to parse result: %w", err)
	}

	return result, nil
}
```

## Testing the Endpoints

Once implemented, verify with curl:

```bash
# Upload log
curl -X POST http://localhost:8001/api/logs/upload \
  -H "Content-Type: application/json" \
  -d '{
    "events": [
      {"case_id": "case_1", "activity": "start", "timestamp": "2026-03-24T00:00:00Z"},
      {"case_id": "case_1", "activity": "end", "timestamp": "2026-03-24T01:00:00Z"}
    ],
    "format": "json"
  }'

# Run discovery
curl -X POST http://localhost:8001/api/discovery/alpha \
  -H "Content-Type: application/json" \
  -d '{"log_id": "log_xxx"}'

# Get results
curl http://localhost:8001/api/discovery/results/result_xxx

# Check conformance
curl -X POST http://localhost:8001/api/conformance/check \
  -H "Content-Type: application/json" \
  -d '{
    "log_id": "log_xxx",
    "model": {"nodes": ["start", "end"]}
  }'
```

## Integration Checklist

- [ ] Create `pm4py.go` handler file
- [ ] Create `pm4py_service.go` service file
- [ ] Create database schema (migrations)
- [ ] Register routes in `routes.go`
- [ ] Add service to `Handlers` struct
- [ ] Test with curl commands
- [ ] Run integration test suite: `pytest businessos_http_integration_tests.py`
- [ ] Verify all 27 Python tests pass
- [ ] Commit code

## References

- BusinessOS Code: `/Users/sac/chatmangpt/BusinessOS/internal/handlers/`
- Handler Pattern: `internal/handlers/compliance.go`
- Service Pattern: `internal/services/`
- Integration Tests: `pm4py-rust/tests/businessos_http_integration_tests.py`
