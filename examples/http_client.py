#!/usr/bin/env python3
"""
PM4Py REST API Client Examples (Python)

Demonstrates how to use the PM4Py REST API from Python applications.
This example covers:
- Authentication
- Process discovery
- Conformance checking
- Log statistics
- Error handling

Installation:
    pip install requests python-dateutil

Usage:
    export PM4PY_API_KEY="your-api-key"
    python examples/http_client.py
"""

import json
import os
import sys
from datetime import datetime, timedelta
from typing import Any, Dict, List, Optional

import requests


class PM4PyClient:
    """PM4Py REST API client with rate limit handling."""

    def __init__(self, api_key: str, base_url: str = "http://localhost:8080/api/v1"):
        """
        Initialize client.

        Args:
            api_key: API key for authentication
            base_url: Base URL of PM4Py API server
        """
        self.api_key = api_key
        self.base_url = base_url.rstrip("/")
        self.session = requests.Session()
        self.session.headers.update({"X-API-Key": api_key})
        self.rate_limit_remaining = 10000
        self.rate_limit_reset = None

    def _update_rate_limit_info(self, response: requests.Response) -> None:
        """Update rate limit information from response headers."""
        if "X-RateLimit-Remaining" in response.headers:
            self.rate_limit_remaining = int(response.headers["X-RateLimit-Remaining"])
        if "X-RateLimit-Reset" in response.headers:
            self.rate_limit_reset = response.headers["X-RateLimit-Reset"]

    def _make_request(self, method: str, path: str, data: Optional[Dict] = None) -> Dict[str, Any]:
        """
        Make HTTP request to API.

        Args:
            method: HTTP method (GET, POST, etc.)
            path: API path (without base URL)
            data: Request body data

        Returns:
            Response JSON

        Raises:
            requests.HTTPError: On HTTP errors
            ValueError: On JSON parsing errors
        """
        url = f"{self.base_url}{path}"

        try:
            if method == "GET":
                response = self.session.get(url)
            elif method == "POST":
                response = self.session.post(url, json=data)
            else:
                raise ValueError(f"Unsupported method: {method}")

            self._update_rate_limit_info(response)

            if response.status_code == 429:
                raise Exception(
                    f"Rate limit exceeded. Reset at {self.rate_limit_reset}"
                )

            response.raise_for_status()
            return response.json()

        except requests.exceptions.RequestException as e:
            print(f"HTTP Error: {e}")
            if hasattr(e.response, "text"):
                try:
                    error = e.response.json()
                    print(f"Error details: {error.get('message', 'Unknown error')}")
                except Exception:
                    print(f"Response: {e.response.text}")
            raise

    def health_check(self) -> Dict[str, Any]:
        """Check API health."""
        return self._make_request("GET", "/health")

    def discover(
        self,
        events: List[Dict],
        algorithm: str = "inductive",
        parameters: Optional[Dict] = None,
    ) -> Dict[str, Any]:
        """
        Discover process model from event log.

        Args:
            events: List of events (case_id, activity, timestamp, ...)
            algorithm: Discovery algorithm (alpha, inductive, heuristic, dfg, etc.)
            parameters: Algorithm-specific parameters

        Returns:
            Discovered model
        """
        payload = {
            "log": {
                "events": events,
                "format": "json",
            },
            "algorithm": algorithm,
        }
        if parameters:
            payload["parameters"] = parameters

        return self._make_request("POST", "/discover", payload)

    def conform(
        self,
        events: List[Dict],
        model: Dict[str, Any],
        variant: str = "token_replay",
    ) -> Dict[str, Any]:
        """
        Check log conformance against model.

        Args:
            events: List of events
            model: Petri Net model (places, transitions, arcs, etc.)
            variant: Conformance variant (token_replay, alignment)

        Returns:
            Conformance result
        """
        payload = {
            "log": {
                "events": events,
                "format": "json",
            },
            "model": model,
            "variant": variant,
        }

        return self._make_request("POST", "/conform", payload)

    def analyze(self, model: Dict[str, Any]) -> Dict[str, Any]:
        """
        Analyze model structure.

        Args:
            model: Petri Net model

        Returns:
            Analysis result
        """
        payload = {"model": model}
        return self._make_request("POST", "/analyze", payload)

    def stats(self, events: List[Dict]) -> Dict[str, Any]:
        """
        Extract log statistics.

        Args:
            events: List of events

        Returns:
            Log statistics
        """
        payload = {
            "log": {
                "events": events,
                "format": "json",
            }
        }

        return self._make_request("POST", "/stats", payload)


# ============================================================================
# EXAMPLE USAGE
# ============================================================================


def create_sample_events() -> List[Dict]:
    """Create sample event log for loan approval process."""
    events = []
    now = datetime.utcnow()

    # Create 5 sample cases
    for case_num in range(1, 6):
        case_id = f"loan_{case_num:03d}"
        case_time = now + timedelta(hours=case_num)

        # Event 1: Application
        events.append(
            {
                "case_id": case_id,
                "activity": "apply",
                "timestamp": case_time.isoformat() + "Z",
                "resource": "customer",
                "attributes": {"amount": 50000 * case_num},
            }
        )

        # Event 2: Registration
        events.append(
            {
                "case_id": case_id,
                "activity": "register",
                "timestamp": (case_time + timedelta(minutes=5)).isoformat() + "Z",
                "resource": "clerk",
            }
        )

        # Event 3: Document verification
        events.append(
            {
                "case_id": case_id,
                "activity": "verify_documents",
                "timestamp": (case_time + timedelta(hours=1)).isoformat() + "Z",
                "resource": "officer",
            }
        )

        # Event 4: Credit check
        events.append(
            {
                "case_id": case_id,
                "activity": "credit_check",
                "timestamp": (case_time + timedelta(hours=2)).isoformat() + "Z",
                "resource": "system",
            }
        )

        # Event 5: Approval
        events.append(
            {
                "case_id": case_id,
                "activity": "approve",
                "timestamp": (case_time + timedelta(hours=3)).isoformat() + "Z",
                "resource": "manager",
                "attributes": {"approved": True},
            }
        )

        # Event 6: Disbursement
        events.append(
            {
                "case_id": case_id,
                "activity": "disburse",
                "timestamp": (case_time + timedelta(hours=4)).isoformat() + "Z",
                "resource": "accountant",
                "attributes": {"amount_disbursed": 50000 * case_num},
            }
        )

    return events


def main():
    """Run example workflows."""
    print("╔════════════════════════════════════════════════════════╗")
    print("║     PM4Py REST API Client Examples (Python)            ║")
    print("╚════════════════════════════════════════════════════════╝\n")

    # Get API key from environment
    api_key = os.getenv("PM4PY_API_KEY", "demo-key-for-testing")
    if api_key == "demo-key-for-testing":
        print("⚠️  Using demo API key. Set PM4PY_API_KEY for real API.\n")

    # Initialize client
    client = PM4PyClient(api_key)

    # Example 1: Health check
    print("1. HEALTH CHECK")
    print("─────────────────────────────────────────────────────────\n")
    try:
        health = client.health_check()
        print(f"✓ API Status: {health.get('status')}")
        print(f"  Version: {health.get('version')}")
        print(f"  Timestamp: {health.get('timestamp')}\n")
    except Exception as e:
        print(f"✗ Health check failed: {e}\n")
        return

    # Example 2: Process Discovery
    print("\n2. PROCESS DISCOVERY")
    print("─────────────────────────────────────────────────────────\n")

    try:
        events = create_sample_events()
        print(f"Created event log with {len(events)} events\n")

        discovery_result = client.discover(
            events=events,
            algorithm="inductive",
            parameters={"frequency_threshold": 0.1},
        )

        model = discovery_result.get("model", {})
        print(f"✓ Model discovered:")
        print(f"  Algorithm: {discovery_result.get('algorithm')}")
        print(f"  Places: {discovery_result.get('num_places')}")
        print(f"  Transitions: {discovery_result.get('num_transitions')}")
        print(f"  Processing time: {discovery_result.get('processing_time_ms')}ms")
        print(
            f"  Rate limit remaining: {client.rate_limit_remaining} requests/hour\n"
        )

        # Example 3: Conformance Checking
        print("\n3. CONFORMANCE CHECKING")
        print("─────────────────────────────────────────────────────────\n")

        if model:
            conformance_result = client.conform(
                events=events,
                model=model,
                variant="token_replay",
            )

            result = conformance_result.get("result", {})
            print(f"✓ Conformance checked:")
            print(f"  Fitness: {result.get('fitness', 0):.2%}")
            print(f"  Precision: {result.get('precision', 0):.2%}")
            print(f"  Generalization: {result.get('generalization', 0):.2%}")
            print(f"  Simplicity: {result.get('simplicity', 0):.2%}")
            print(f"  Deviant traces: {len(result.get('deviant_traces', []))}")
            print(
                f"  Processing time: {conformance_result.get('processing_time_ms')}ms\n"
            )

        # Example 4: Log Statistics
        print("\n4. LOG STATISTICS")
        print("─────────────────────────────────────────────────────────\n")

        stats_result = client.stats(events=events)

        stats = stats_result.get("stats", {})
        print(f"✓ Statistics extracted:")
        print(f"  Traces: {stats.get('num_traces')}")
        print(f"  Events: {stats.get('num_events')}")
        print(f"  Activities: {stats.get('num_activities')}")
        print(f"  Mean trace length: {stats.get('trace_length_mean', 0):.2f}")
        print(f"  Mean case duration: {stats.get('case_duration_mean')}")
        print(
            f"  Processing time: {stats_result.get('processing_time_ms')}ms\n"
        )

        # Show top activities
        activities = stats.get("activities", [])
        if activities:
            print("  Top activities:")
            for activity in activities[:3]:
                print(
                    f"    - {activity.get('name')}: {activity.get('frequency')} times"
                )

        # Example 5: Model Analysis
        if model:
            print("\n5. MODEL ANALYSIS")
            print("─────────────────────────────────────────────────────────\n")

            analysis_result = client.analyze(model=model)

            print(f"✓ Model analyzed:")
            print(f"  Is sound: {analysis_result.get('is_sound')}")

            profile = analysis_result.get("behavioral_profile", {})
            print(f"  Strongly connected components: {profile.get('strongly_connected_components')}")
            print(f"  Longest path: {profile.get('longest_path')}")

            deadlocks = analysis_result.get("deadlock_potential", [])
            if deadlocks:
                print(f"  ⚠️  Potential deadlock configurations: {len(deadlocks)}")
            else:
                print(f"  ✓ No deadlock potential")

            print(f"  Processing time: {analysis_result.get('processing_time_ms')}ms\n")

    except Exception as e:
        print(f"✗ Error during example: {e}\n")
        sys.exit(1)

    # Summary
    print("\n╔════════════════════════════════════════════════════════╗")
    print("║                Examples Complete                       ║")
    print("╚════════════════════════════════════════════════════════╝\n")

    print(f"Rate limit status: {client.rate_limit_remaining}/10000 remaining")
    if client.rate_limit_reset:
        print(f"Rate limit resets: {client.rate_limit_reset}")


if __name__ == "__main__":
    main()
