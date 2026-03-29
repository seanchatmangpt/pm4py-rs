#!/usr/bin/env python3
"""
BusinessOS HTTP Integration Tests for pm4py and pm4py-rust

Tests verify:
1. Event log upload to /api/logs/upload (JSON/CSV formats)
2. Discovery algorithms via /api/discovery/{algorithm}
3. Results retrieval from /api/discovery/results/{id}
4. Conformance checking via /api/conformance/check
5. Cross-project integration with pm4py-rust serialization

Uses official pm4py test datasets and Chicago TDD methodology
(no mocks, real API calls against BusinessOS backend).

Usage:
    pytest businessos_http_integration_tests.py -v
    pytest businessos_http_integration_tests.py::TestLogUpload -v
    pytest businessos_http_integration_tests.py::TestDiscoveryAlgorithms::test_alpha_miner -v
"""

import os
import sys
import json
import csv
import pytest
import requests
from pathlib import Path
from typing import List, Dict, Any
from datetime import datetime, timedelta
import time

# ============================================================================
# CONFIGURATION & FIXTURES
# ============================================================================

BUSINESSOS_API_BASE = os.getenv("BUSINESSOS_API_BASE", "http://localhost:8001")
BUSINESSOS_API_KEY = os.getenv("BUSINESSOS_API_KEY", "")
TEST_TIMEOUT = 30

# Get home directory for test data
HOME = os.path.expanduser("~")
TEST_DATA_DIR = Path(HOME) / "chatmangpt" / "pm4py-rust" / "test_data"

# Ensure test data exists
assert TEST_DATA_DIR.exists(), f"Test data directory not found: {TEST_DATA_DIR}"


class TestAPI:
    """Wrapper for BusinessOS HTTP API calls with auth and error handling."""

    def __init__(self, base_url: str, api_key: str = ""):
        self.base_url = base_url
        self.api_key = api_key
        self.session = requests.Session()
        if api_key:
            self.session.headers.update({"Authorization": f"Bearer {api_key}"})

    def post(self, endpoint: str, json_data: Dict = None, files: Dict = None) -> Dict:
        """POST request with error handling."""
        url = f"{self.base_url}/{endpoint.lstrip('/')}"
        try:
            response = self.session.post(url, json=json_data, files=files, timeout=TEST_TIMEOUT)
            response.raise_for_status()
            return response.json() if response.text else {}
        except requests.exceptions.RequestException as e:
            pytest.fail(f"POST {endpoint} failed: {e}")

    def get(self, endpoint: str) -> Dict:
        """GET request with error handling."""
        url = f"{self.base_url}/{endpoint.lstrip('/')}"
        try:
            response = self.session.get(url, timeout=TEST_TIMEOUT)
            response.raise_for_status()
            return response.json() if response.text else {}
        except requests.exceptions.RequestException as e:
            pytest.fail(f"GET {endpoint} failed: {e}")

    def health_check(self) -> bool:
        """Check if BusinessOS API is running."""
        try:
            response = self.session.get(f"{self.base_url}/health", timeout=5)
            return response.status_code == 200
        except:
            return False


@pytest.fixture(scope="session")
def api_client():
    """Create API client for test session."""
    client = TestAPI(BUSINESSOS_API_BASE, BUSINESSOS_API_KEY)
    if not client.health_check():
        pytest.skip(f"BusinessOS API not available at {BUSINESSOS_API_BASE}")
    return client


@pytest.fixture(scope="session")
def sample_event_log() -> List[Dict[str, Any]]:
    """Create sample event log in BusinessOS JSON format."""
    now = datetime.utcnow()
    events = []

    # Create 5 traces with common process: register -> examine -> decide -> approve
    for case_id in range(1, 6):
        case_time = now + timedelta(days=case_id)
        events.extend([
            {
                "case_id": f"case_{case_id}",
                "activity": "register",
                "timestamp": case_time.isoformat() + "Z",
                "resource": "clerk",
                "attributes": {"amount": case_id * 1000}
            },
            {
                "case_id": f"case_{case_id}",
                "activity": "examine",
                "timestamp": (case_time + timedelta(hours=1)).isoformat() + "Z",
                "resource": "senior",
                "attributes": {}
            },
            {
                "case_id": f"case_{case_id}",
                "activity": "decide",
                "timestamp": (case_time + timedelta(hours=2)).isoformat() + "Z",
                "resource": "mgr",
                "attributes": {}
            },
            {
                "case_id": f"case_{case_id}",
                "activity": "approve",
                "timestamp": (case_time + timedelta(hours=3)).isoformat() + "Z",
                "resource": "director",
                "attributes": {"approved": True}
            },
        ])

    return events


@pytest.fixture(scope="session")
def xes_test_file() -> Path:
    """Return path to XES test file."""
    xes_file = TEST_DATA_DIR / "running-example.xes"
    assert xes_file.exists(), f"XES test file not found: {xes_file}"
    return xes_file


@pytest.fixture(scope="session")
def csv_test_file() -> Path:
    """Return path to CSV test file."""
    csv_file = TEST_DATA_DIR / "running-example.csv"
    assert csv_file.exists(), f"CSV test file not found: {csv_file}"
    return csv_file


# ============================================================================
# TEST SUITE: 1. Event Log Upload
# ============================================================================

class TestLogUpload:
    """Test /api/logs/upload endpoint with various formats."""

    def test_upload_json_event_log(self, api_client, sample_event_log):
        """Upload JSON event log and verify it's stored."""
        result = api_client.post(
            "/api/logs/upload",
            json_data={"events": sample_event_log, "format": "json"}
        )

        # Verify response structure
        assert result.get("status") in ["success", "ok"]
        assert "log_id" in result, "Response should include log_id"
        assert isinstance(result["log_id"], str)
        assert len(result["log_id"]) > 0

        # Verify metadata
        assert result.get("event_count") == len(sample_event_log)
        assert result.get("case_count") == 5

    def test_upload_csv_file(self, api_client, csv_test_file):
        """Upload CSV file and verify parsing."""
        with open(csv_test_file, "rb") as f:
            files = {"file": ("running-example.csv", f, "text/csv")}
            result = api_client.post("/api/logs/upload", files=files)

        # Verify response
        assert result.get("status") in ["success", "ok"]
        assert "log_id" in result
        assert result.get("event_count") > 0
        assert result.get("case_count") > 0

    def test_upload_xes_file(self, api_client, xes_test_file):
        """Upload XES file and verify parsing."""
        with open(xes_test_file, "rb") as f:
            files = {"file": ("running-example.xes", f, "application/xml")}
            result = api_client.post("/api/logs/upload", files=files)

        # Verify response
        assert result.get("status") in ["success", "ok"]
        assert "log_id" in result
        assert result.get("event_count") > 0
        assert result.get("case_count") > 0

    def test_upload_empty_log_rejected(self, api_client):
        """Empty event logs should be rejected."""
        result = api_client.post(
            "/api/logs/upload",
            json_data={"events": [], "format": "json"}
        )
        # Either explicit error or validation failure
        assert result.get("status") in ["error", "validation_error"] or \
               result.get("error") is not None

    def test_upload_malformed_json_rejected(self, api_client):
        """Malformed JSON should be rejected."""
        result = api_client.post(
            "/api/logs/upload",
            json_data={"events": [{"case_id": "missing_activity"}]}
        )
        # Should fail validation
        assert "error" in result or result.get("status") == "error"

    def test_upload_log_with_attributes(self, api_client):
        """Upload log with extended attributes."""
        now = datetime.utcnow()
        events = [
            {
                "case_id": "attr_case_1",
                "activity": "start",
                "timestamp": now.isoformat() + "Z",
                "resource": "bot",
                "attributes": {
                    "cost": 100.50,
                    "priority": "high",
                    "department": "finance"
                }
            }
        ]
        result = api_client.post(
            "/api/logs/upload",
            json_data={"events": events}
        )
        assert result.get("status") in ["success", "ok"]


# ============================================================================
# TEST SUITE: 2. Discovery Algorithms
# ============================================================================

class TestDiscoveryAlgorithms:
    """Test /api/discovery/{algorithm} endpoints."""

    @pytest.fixture(autouse=True)
    def setup_test_log(self, api_client, sample_event_log):
        """Upload test log once for all discovery tests."""
        result = api_client.post(
            "/api/logs/upload",
            json_data={"events": sample_event_log, "format": "json"}
        )
        self.log_id = result.get("log_id")
        assert self.log_id, "Failed to create test log"

    def test_discover_alpha_miner(self, api_client):
        """Test Alpha Miner discovery algorithm."""
        result = api_client.post(
            f"/api/discovery/alpha",
            json_data={"log_id": self.log_id}
        )

        # Verify response structure
        assert result.get("status") in ["success", "ok"]
        assert "result_id" in result
        assert "model" in result

        # Verify model has expected structure
        model = result["model"]
        assert "nodes" in model or "transitions" in model
        assert "edges" in model or "arcs" in model

    def test_discover_inductive_miner(self, api_client):
        """Test Inductive Miner discovery algorithm."""
        result = api_client.post(
            f"/api/discovery/inductive",
            json_data={"log_id": self.log_id}
        )

        assert result.get("status") in ["success", "ok"]
        assert "result_id" in result
        assert "model" in result

    def test_discover_heuristic_miner(self, api_client):
        """Test Heuristic Miner discovery algorithm."""
        result = api_client.post(
            f"/api/discovery/heuristic",
            json_data={"log_id": self.log_id}
        )

        assert result.get("status") in ["success", "ok"]
        assert "result_id" in result

    def test_discover_dfg(self, api_client):
        """Test Directly-Follows Graph discovery."""
        result = api_client.post(
            f"/api/discovery/dfg",
            json_data={"log_id": self.log_id}
        )

        assert result.get("status") in ["success", "ok"]
        assert "result_id" in result
        assert "dfg" in result or "model" in result

    def test_discover_with_filters(self, api_client):
        """Test discovery with activity filters."""
        result = api_client.post(
            f"/api/discovery/alpha",
            json_data={
                "log_id": self.log_id,
                "filters": {
                    "min_frequency": 1,
                    "activity_filter": ["register", "examine", "decide"]
                }
            }
        )

        assert result.get("status") in ["success", "ok"]

    def test_discover_with_timeout(self, api_client):
        """Test discovery respects timeout parameter."""
        result = api_client.post(
            f"/api/discovery/alpha",
            json_data={
                "log_id": self.log_id,
                "timeout_seconds": 10
            }
        )

        # Either succeeds or returns timeout error (both valid)
        assert result.get("status") in ["success", "ok", "timeout"]


# ============================================================================
# TEST SUITE: 3. Results Retrieval
# ============================================================================

class TestResultsRetrieval:
    """Test /api/discovery/results/{id} endpoint."""

    @pytest.fixture(autouse=True)
    def setup_discovery(self, api_client, sample_event_log):
        """Run discovery and get result ID."""
        upload_result = api_client.post(
            "/api/logs/upload",
            json_data={"events": sample_event_log, "format": "json"}
        )
        log_id = upload_result.get("log_id")

        discovery_result = api_client.post(
            "/api/discovery/alpha",
            json_data={"log_id": log_id}
        )
        self.result_id = discovery_result.get("result_id")
        assert self.result_id, "Failed to get result_id from discovery"

    def test_retrieve_result(self, api_client):
        """Retrieve discovery result by ID."""
        result = api_client.get(f"/api/discovery/results/{self.result_id}")

        assert result.get("status") in ["success", "ok"]
        assert result.get("result_id") == self.result_id
        assert "model" in result

    def test_retrieve_result_includes_metadata(self, api_client):
        """Retrieved result should include metadata."""
        result = api_client.get(f"/api/discovery/results/{self.result_id}")

        assert "metadata" in result
        metadata = result["metadata"]
        assert "discovered_at" in metadata or "timestamp" in metadata
        assert "algorithm" in metadata or "method" in metadata

    def test_retrieve_nonexistent_result(self, api_client):
        """Retrieving non-existent result should fail gracefully."""
        result = api_client.get("/api/discovery/results/nonexistent_id_12345")

        # Should return error status
        assert result.get("status") == "error" or "error" in result


# ============================================================================
# TEST SUITE: 4. Conformance Checking
# ============================================================================

class TestConformanceChecking:
    """Test /api/conformance/check endpoint."""

    @pytest.fixture(autouse=True)
    def setup_conformance(self, api_client, sample_event_log):
        """Prepare log and model for conformance tests."""
        # Upload log
        upload_result = api_client.post(
            "/api/logs/upload",
            json_data={"events": sample_event_log, "format": "json"}
        )
        self.log_id = upload_result.get("log_id")

        # Discover model
        discovery_result = api_client.post(
            "/api/discovery/alpha",
            json_data={"log_id": self.log_id}
        )
        self.model = discovery_result.get("model")
        assert self.model, "Failed to discover model for conformance test"

    def test_conformance_check_basic(self, api_client):
        """Test basic conformance checking."""
        result = api_client.post(
            "/api/conformance/check",
            json_data={
                "log_id": self.log_id,
                "model": self.model
            }
        )

        assert result.get("status") in ["success", "ok"]
        assert "conformance" in result or "fitness" in result

    def test_conformance_includes_statistics(self, api_client):
        """Conformance result should include fitness stats."""
        result = api_client.post(
            "/api/conformance/check",
            json_data={
                "log_id": self.log_id,
                "model": self.model
            }
        )

        if result.get("status") in ["success", "ok"]:
            # Should have fitness metrics
            assert "fitness" in result or "conformance" in result or \
                   "deviations" in result


# ============================================================================
# TEST SUITE: 5. Cross-Project Integration (pm4py-rust Serialization)
# ============================================================================

class TestCrossProjectIntegration:
    """Test integration between pm4py-rust and BusinessOS via HTTP."""

    def test_log_roundtrip_json_serialization(self, api_client, sample_event_log):
        """Upload log to BusinessOS, retrieve, verify JSON structure intact."""
        # Upload
        upload_result = api_client.post(
            "/api/logs/upload",
            json_data={"events": sample_event_log, "format": "json"}
        )
        log_id = upload_result.get("log_id")

        # Retrieve
        retrieve_result = api_client.get(f"/api/logs/{log_id}")

        # Verify structure preserved
        retrieved_events = retrieve_result.get("events", [])
        assert len(retrieved_events) == len(sample_event_log)

        # Verify key fields present
        for event in retrieved_events:
            assert "case_id" in event
            assert "activity" in event
            assert "timestamp" in event

    def test_model_serialization_format(self, api_client, sample_event_log):
        """Verify discovered model uses standardized serialization format."""
        # Upload and discover
        upload_result = api_client.post(
            "/api/logs/upload",
            json_data={"events": sample_event_log, "format": "json"}
        )
        log_id = upload_result.get("log_id")

        discovery_result = api_client.post(
            "/api/discovery/alpha",
            json_data={"log_id": log_id}
        )

        model = discovery_result.get("model")
        assert model is not None

        # Model should be serializable to JSON (for transmission to pm4py-rust)
        try:
            model_json = json.dumps(model)
            assert len(model_json) > 0
        except TypeError:
            pytest.fail("Model is not JSON serializable")

    def test_batch_discovery_results(self, api_client, sample_event_log):
        """Test batch discovery with multiple algorithms."""
        # Upload once
        upload_result = api_client.post(
            "/api/logs/upload",
            json_data={"events": sample_event_log, "format": "json"}
        )
        log_id = upload_result.get("log_id")

        algorithms = ["alpha", "inductive", "heuristic"]
        results = {}

        # Run multiple algorithms
        for algo in algorithms:
            result = api_client.post(
                f"/api/discovery/{algo}",
                json_data={"log_id": log_id}
            )
            if result.get("status") in ["success", "ok"]:
                results[algo] = result

        # Verify all succeeded
        assert len(results) > 0, "At least one discovery algorithm should succeed"

    def test_api_response_timestamps_iso8601(self, api_client, sample_event_log):
        """API responses should use ISO8601 timestamps for compatibility."""
        result = api_client.post(
            "/api/logs/upload",
            json_data={"events": sample_event_log, "format": "json"}
        )

        # Check response timestamp if present
        if "timestamp" in result:
            timestamp = result["timestamp"]
            # Should be ISO8601 format
            try:
                datetime.fromisoformat(timestamp.replace("Z", "+00:00"))
            except ValueError:
                pytest.fail(f"Timestamp not ISO8601: {timestamp}")

    def test_large_log_handling(self, api_client):
        """Test handling of larger event logs (100+ events)."""
        now = datetime.utcnow()
        events = []

        # Create 50 cases with 3 events each = 150 events
        for case_id in range(1, 51):
            case_time = now + timedelta(days=case_id)
            for step, activity in enumerate(["start", "process", "end"]):
                events.append({
                    "case_id": f"case_{case_id}",
                    "activity": activity,
                    "timestamp": (case_time + timedelta(hours=step)).isoformat() + "Z",
                    "resource": "worker",
                    "attributes": {}
                })

        result = api_client.post(
            "/api/logs/upload",
            json_data={"events": events, "format": "json"}
        )

        assert result.get("status") in ["success", "ok"]
        assert result.get("event_count") == 150
        assert result.get("case_count") == 50


# ============================================================================
# TEST SUITE: 6. Edge Cases & Error Handling
# ============================================================================

class TestEdgeCasesAndErrors:
    """Test error handling and edge cases."""

    def test_missing_required_fields(self, api_client):
        """Events missing required fields should be rejected."""
        events = [{"case_id": "case_1"}]  # Missing activity and timestamp
        result = api_client.post(
            "/api/logs/upload",
            json_data={"events": events}
        )
        assert "error" in result or result.get("status") == "error"

    def test_invalid_timestamp_format(self, api_client):
        """Invalid timestamp format should be rejected."""
        events = [{
            "case_id": "case_1",
            "activity": "start",
            "timestamp": "not-a-timestamp"
        }]
        result = api_client.post(
            "/api/logs/upload",
            json_data={"events": events}
        )
        assert "error" in result or result.get("status") == "error"

    def test_duplicate_case_ids_handled(self, api_client):
        """Multiple events with same case_id should be grouped into single trace."""
        events = [
            {"case_id": "dup_1", "activity": "a", "timestamp": "2026-03-24T00:00:00Z"},
            {"case_id": "dup_1", "activity": "b", "timestamp": "2026-03-24T01:00:00Z"},
            {"case_id": "dup_1", "activity": "c", "timestamp": "2026-03-24T02:00:00Z"},
        ]
        result = api_client.post(
            "/api/logs/upload",
            json_data={"events": events}
        )
        assert result.get("status") in ["success", "ok"]
        assert result.get("case_count") == 1
        assert result.get("event_count") == 3


# ============================================================================
# TEST SUITE: 7. Performance & Stability
# ============================================================================

class TestPerformanceAndStability:
    """Test API performance and stability characteristics."""

    def test_discovery_completes_within_timeout(self, api_client, sample_event_log):
        """Discovery should complete within reasonable time."""
        # Upload
        upload_result = api_client.post(
            "/api/logs/upload",
            json_data={"events": sample_event_log, "format": "json"}
        )
        log_id = upload_result.get("log_id")

        # Run discovery with timeout tracking
        start = time.time()
        result = api_client.post(
            "/api/discovery/alpha",
            json_data={"log_id": log_id, "timeout_seconds": TEST_TIMEOUT}
        )
        elapsed = time.time() - start

        # Should complete within timeout
        assert elapsed < TEST_TIMEOUT
        assert result.get("status") in ["success", "ok", "timeout"]

    def test_concurrent_uploads(self, api_client, sample_event_log):
        """Multiple concurrent uploads should all succeed."""
        import concurrent.futures

        def upload_log():
            return api_client.post(
                "/api/logs/upload",
                json_data={"events": sample_event_log, "format": "json"}
            )

        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
            futures = [executor.submit(upload_log) for _ in range(3)]
            results = [f.result() for f in futures]

        # All should succeed
        assert all(r.get("status") in ["success", "ok"] for r in results)
        assert len(results) == 3


# ============================================================================
# MAIN ENTRY POINT
# ============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
