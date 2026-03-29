#!/usr/bin/env python3
"""
Integration tests for pm4py-rust Python bindings.

These tests verify that Python code can successfully import and use
the Rust implementations through PyO3 bindings.

Usage:
    # Build the extension first
    maturin develop

    # Run tests
    pytest tests/test_python_bindings.py -v
"""

import pytest
from datetime import datetime, timezone

# Import the Python bindings (available after maturin build)
try:
    from pm4py_rust import (
        EventLog, Event, Trace,
        AlphaMiner, InductiveMiner, HeuristicMiner,
        FootprintsConformanceChecker,
        LogStatistics,
        PetriNet
    )
    BINDINGS_AVAILABLE = True
except ImportError:
    BINDINGS_AVAILABLE = False


@pytest.mark.skipif(not BINDINGS_AVAILABLE, reason="pm4py_rust bindings not available")
class TestEventLogCreation:
    """Test EventLog creation and manipulation through Python bindings."""

    def test_create_empty_log(self):
        """Create an empty event log."""
        log = EventLog()
        assert len(log) == 0
        assert log.to_json() is not None

    def test_create_event(self):
        """Create an event with proper timestamp format."""
        event = Event(
            "activity_A",
            "2024-01-01T00:00:00Z"
        )
        assert event.activity == "activity_A"
        assert "2024-01-01" in event.timestamp

    def test_create_trace(self):
        """Create a trace and add events."""
        trace = Trace("case_1")
        assert trace.case_id == "case_1"
        assert trace.len() == 0

        # Add events
        trace.add_event("A", "2024-01-01T00:00:00Z")
        trace.add_event("B", "2024-01-01T01:00:00Z")
        trace.add_event("C", "2024-01-01T02:00:00Z")

        assert trace.len() == 3

    def test_trace_with_resource(self):
        """Add events to trace with resource information."""
        trace = Trace("case_1")
        trace.add_event_with_resource(
            "A",
            "2024-01-01T00:00:00Z",
            "resource_1"
        )
        assert trace.len() == 1

    def test_add_trace_to_log(self):
        """Add traces to event log."""
        log = EventLog()

        # Add first trace
        trace1 = Trace("case_1")
        trace1.add_event("A", "2024-01-01T00:00:00Z")
        trace1.add_event("B", "2024-01-01T01:00:00Z")
        log.add_trace_obj(trace1)

        # Add second trace
        trace2 = Trace("case_2")
        trace2.add_event("A", "2024-01-02T00:00:00Z")
        trace2.add_event("B", "2024-01-02T01:00:00Z")
        log.add_trace_obj(trace2)

        assert len(log) == 2


@pytest.mark.skipif(not BINDINGS_AVAILABLE, reason="pm4py_rust bindings not available")
class TestDiscoveryAlgorithms:
    """Test discovery algorithms through Python bindings."""

    @staticmethod
    def create_sample_log():
        """Create a sample event log for testing."""
        log = EventLog()

        # Create 10 traces: A -> B -> C
        for i in range(10):
            trace = Trace(f"case_{i}")
            trace.add_event("A", f"2024-01-01T{i:02d}:00:00Z")
            trace.add_event("B", f"2024-01-01T{i:02d}:01:00Z")
            trace.add_event("C", f"2024-01-01T{i:02d}:02:00Z")
            log.add_trace_obj(trace)

        return log

    def test_alpha_miner(self):
        """Test Alpha Miner discovery."""
        log = self.create_sample_log()
        miner = AlphaMiner()
        net = miner.apply(log)

        # Verify result is a Petri net
        assert net is not None
        assert net.places_count() > 0
        assert net.transitions_count() > 0

    def test_heuristic_miner(self):
        """Test Heuristic Miner discovery."""
        log = self.create_sample_log()
        miner = HeuristicMiner()
        net = miner.apply(log)

        # Verify result is a Petri net
        assert net is not None
        assert net.places_count() > 0
        assert net.transitions_count() > 0

    def test_inductive_miner(self):
        """Test Inductive Miner discovery."""
        log = self.create_sample_log()
        miner = InductiveMiner()
        result = miner.apply(log)

        # Verify result
        assert result is not None


@pytest.mark.skipif(not BINDINGS_AVAILABLE, reason="pm4py_rust bindings not available")
class TestConformanceChecking:
    """Test conformance checking through Python bindings."""

    @staticmethod
    def create_sample_log():
        """Create a sample event log for testing."""
        log = EventLog()

        # Create 10 traces: A -> B -> C
        for i in range(10):
            trace = Trace(f"case_{i}")
            trace.add_event("A", f"2024-01-01T{i:02d}:00:00Z")
            trace.add_event("B", f"2024-01-01T{i:02d}:01:00Z")
            trace.add_event("C", f"2024-01-01T{i:02d}:02:00Z")
            log.add_trace_obj(trace)

        return log

    def test_footprints_conformance(self):
        """Test Footprints Conformance Checker."""
        log = self.create_sample_log()

        # Discover a model
        miner = AlphaMiner()
        net = miner.apply(log)

        # Check conformance
        checker = FootprintsConformanceChecker()
        result = checker.apply(net, log)

        # Verify result properties
        assert result is not None
        assert result.is_conformant is not None
        assert result.traces_fit >= 0
        assert result.traces_total > 0
        assert 0.0 <= result.fitness <= 1.0


@pytest.mark.skipif(not BINDINGS_AVAILABLE, reason="pm4py_rust bindings not available")
class TestStatistics:
    """Test statistics functions through Python bindings."""

    @staticmethod
    def create_sample_log():
        """Create a sample event log for testing."""
        log = EventLog()

        # Create 10 traces with different patterns
        for i in range(10):
            trace = Trace(f"case_{i}")
            if i < 5:
                # First 5: A -> B -> C
                trace.add_event("A", f"2024-01-01T{i:02d}:00:00Z")
                trace.add_event("B", f"2024-01-01T{i:02d}:01:00Z")
                trace.add_event("C", f"2024-01-01T{i:02d}:02:00Z")
            else:
                # Last 5: A -> C -> B
                trace.add_event("A", f"2024-01-01T{i:02d}:00:00Z")
                trace.add_event("C", f"2024-01-01T{i:02d}:01:00Z")
                trace.add_event("B", f"2024-01-01T{i:02d}:02:00Z")
            log.add_trace_obj(trace)

        return log

    def test_basic_stats(self):
        """Test basic log statistics."""
        log = self.create_sample_log()
        stats = LogStatistics()
        result = stats.basic_stats(log)

        # Verify statistics
        assert result["num_traces"] == 10
        assert result["num_events"] == 30
        assert result["num_variants"] == 2
        assert result["avg_trace_length"] == 3.0
        assert result["min_trace_length"] == 3
        assert result["max_trace_length"] == 3

    def test_activities(self):
        """Test getting activities from log."""
        log = self.create_sample_log()
        stats = LogStatistics()
        activities = stats.get_activities(log)

        # Verify activities
        assert set(activities) == {"A", "B", "C"}

    def test_activity_frequencies(self):
        """Test activity frequency calculation."""
        log = self.create_sample_log()
        stats = LogStatistics()
        frequencies = stats.get_activity_frequencies(log)

        # Each activity appears 10 times
        assert frequencies["A"] == 10
        assert frequencies["B"] == 10
        assert frequencies["C"] == 10

    def test_variants(self):
        """Test variant extraction."""
        log = self.create_sample_log()
        stats = LogStatistics()
        variants = stats.get_variants(log)

        # Should have 2 variants
        assert len(variants) == 2
        assert variants["A,B,C"] == 5
        assert variants["A,C,B"] == 5


@pytest.mark.skipif(not BINDINGS_AVAILABLE, reason="pm4py_rust bindings not available")
class TestPetriNetModels:
    """Test Petri Net model representations."""

    def test_petri_net_structure(self):
        """Test Petri Net structure representation."""
        # Create a simple log and discover model
        log = EventLog()
        trace = Trace("case_1")
        trace.add_event("A", "2024-01-01T00:00:00Z")
        trace.add_event("B", "2024-01-01T01:00:00Z")
        log.add_trace_obj(trace)

        miner = AlphaMiner()
        net = miner.apply(log)

        # Verify structure
        assert net.places_count() > 0
        assert net.transitions_count() > 0
        assert net.arcs_count() > 0

    def test_petri_net_serialization(self):
        """Test Petri Net JSON serialization."""
        log = EventLog()
        trace = Trace("case_1")
        trace.add_event("A", "2024-01-01T00:00:00Z")
        trace.add_event("B", "2024-01-01T01:00:00Z")
        log.add_trace_obj(trace)

        miner = AlphaMiner()
        net = miner.apply(log)

        # Serialize to JSON
        json_str = net.to_json()
        assert json_str is not None
        assert len(json_str) > 0


@pytest.mark.skipif(not BINDINGS_AVAILABLE, reason="pm4py_rust bindings not available")
class TestPerformanceComparison:
    """Compare performance characteristics (informal tests)."""

    @staticmethod
    def create_large_log(num_traces=100):
        """Create a larger event log for performance testing."""
        log = EventLog()

        for i in range(num_traces):
            trace = Trace(f"case_{i}")
            # Create varying trace patterns
            for j in range((i % 5) + 3):
                activity = chr(ord("A") + (j % 26))
                trace.add_event(activity, f"2024-01-01T{j:02d}:00:00Z")
            log.add_trace_obj(trace)

        return log

    def test_large_log_processing(self):
        """Test processing of larger event logs."""
        log = self.create_large_log(num_traces=100)
        assert len(log) == 100

        # Test statistics calculation
        stats = LogStatistics()
        result = stats.basic_stats(log)
        assert result["num_traces"] == 100
        assert result["num_events"] > 300

    def test_discovery_on_large_log(self):
        """Test discovery algorithm on larger log."""
        log = self.create_large_log(num_traces=50)
        miner = AlphaMiner()

        # This should complete successfully
        net = miner.apply(log)
        assert net.places_count() > 0
        assert net.transitions_count() > 0


if __name__ == "__main__":
    # Allow running tests directly if pytest is not available
    pytest.main([__file__, "-v"])
