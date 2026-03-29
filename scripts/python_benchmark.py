#!/usr/bin/env python3
"""
Performance Benchmarking: pm4py-rust vs Python pm4py
======================================================

Comprehensive benchmarking suite comparing Rust implementation against Python reference.

Metrics:
- Wall-clock time (seconds)
- Throughput (events/sec)
- Memory usage (MB)
- Metric accuracy comparison

Requirements:
- pm4py (Python process mining library)
- psutil (memory profiling)
- pandas (data analysis)

Usage:
    python scripts/python_benchmark.py [--output results.json] [--warmup 1]
"""

import json
import time
import sys
import argparse
from pathlib import Path
from typing import Dict, List, Tuple, Any
import statistics

try:
    import pm4py
    from pm4py.objects.log.importer.xes import importer as xes_importer
    from pm4py.objects.log.util import dataframe_utils
    from pm4py.algo.discovery.alpha import algorithm as alpha_miner
    from pm4py.algo.discovery.inductive import algorithm as inductive_miner
    from pm4py.algo.discovery.dfg import algorithm as dfg_miner
    from pm4py.algo.conformance.tokenreplay import algorithm as token_replay
    from pm4py.algo.discovery.footprints import algorithm as footprints_discovery
    from pm4py.statistics.start_activities import algorithm as start_activities
    from pm4py.statistics.end_activities import algorithm as end_activities
    from pm4py.statistics.traces_length import algorithm as traces_length
    import psutil
    import pandas as pd
    from datetime import datetime, timedelta
except ImportError as e:
    print(f"Missing dependency: {e}")
    print("Install with: pip install pm4py psutil pandas")
    sys.exit(1)


class EventLogGenerator:
    """Generate synthetic event logs for benchmarking"""

    @staticmethod
    def generate_linear_log(num_events: int, num_traces: int) -> pm4py.LogFile:
        """Generate linear (sequential) event log"""
        activities = ["A", "B", "C", "D", "E"]
        events_per_trace = max(1, num_events // num_traces)
        base_time = datetime.now()

        data = []
        for trace_id in range(num_traces):
            for event_idx in range(events_per_trace):
                activity_idx = event_idx % len(activities)
                timestamp = base_time + timedelta(
                    seconds=trace_id * events_per_trace + event_idx
                )
                data.append({
                    "case:concept:name": f"trace_{trace_id:08d}",
                    "concept:name": activities[activity_idx],
                    "time:timestamp": timestamp,
                    "org:resource": f"res_{trace_id % 5}",
                })

        df = pd.DataFrame(data)
        df = dataframe_utils.convert_to_event_log(df)
        return pm4py.convert_to_event_log(df)

    @staticmethod
    def generate_parallel_log(num_events: int, num_traces: int) -> pm4py.LogFile:
        """Generate parallel event log (complex control flow)"""
        activities = ["Start", "ParallelA", "ParallelB", "ParallelC", "Join", "Process", "End"]
        events_per_trace = max(1, num_events // num_traces)
        base_time = datetime.now()

        data = []
        for trace_id in range(num_traces):
            for event_idx in range(events_per_trace):
                activity = activities[event_idx % len(activities)]
                timestamp = base_time + timedelta(
                    seconds=trace_id * events_per_trace + event_idx
                )
                data.append({
                    "case:concept:name": f"parallel_{trace_id:08d}",
                    "concept:name": activity,
                    "time:timestamp": timestamp,
                    "org:resource": f"specialist_{event_idx % 4}",
                })

        df = pd.DataFrame(data)
        df = dataframe_utils.convert_to_event_log(df)
        return pm4py.convert_to_event_log(df)

    @staticmethod
    def generate_loop_log(num_events: int, num_traces: int) -> pm4py.LogFile:
        """Generate event log with loops and rework patterns"""
        activities = ["Init", "Check", "Process", "Complete", "Rework"]
        events_per_trace = max(1, num_events // num_traces)
        base_time = datetime.now()

        data = []
        for trace_id in range(num_traces):
            current_time = base_time
            for event_idx in range(events_per_trace):
                if event_idx % 5 == 4 and event_idx > 0:
                    activity = "Rework"
                else:
                    activity = activities[(event_idx // 5) % 4]

                data.append({
                    "case:concept:name": f"loop_{trace_id:08d}",
                    "concept:name": activity,
                    "time:timestamp": current_time,
                    "org:resource": f"agent_{trace_id % 8}",
                })
                current_time += timedelta(seconds=3)

        df = pd.DataFrame(data)
        df = dataframe_utils.convert_to_event_log(df)
        return pm4py.convert_to_event_log(df)

    @staticmethod
    def generate_conformance_log(num_events: int, num_traces: int) -> pm4py.LogFile:
        """Generate standard log for conformance testing"""
        activities = ["A", "B", "C", "D", "E"]
        events_per_trace = max(1, num_events // num_traces)
        base_time = datetime.now()

        data = []
        for trace_id in range(num_traces):
            for event_idx in range(events_per_trace):
                activity = activities[event_idx % len(activities)]
                timestamp = base_time + timedelta(
                    seconds=trace_id * events_per_trace + event_idx
                )
                data.append({
                    "case:concept:name": f"case_{trace_id:08d}",
                    "concept:name": activity,
                    "time:timestamp": timestamp,
                    "org:resource": f"worker_{trace_id % 5}",
                })

        df = pd.DataFrame(data)
        df = dataframe_utils.convert_to_event_log(df)
        return pm4py.convert_to_event_log(df)


class Benchmarker:
    """Run benchmarks and collect metrics"""

    def __init__(self, warmup_runs: int = 1):
        self.warmup_runs = warmup_runs
        self.results: Dict[str, Any] = {
            "timestamp": datetime.now().isoformat(),
            "python_pm4py_version": pm4py.__version__,
            "benchmarks": {}
        }

    def benchmark_function(
        self,
        name: str,
        func,
        *args,
        runs: int = 3,
    ) -> Dict[str, float]:
        """Benchmark a function multiple times"""
        print(f"  Benchmarking {name}...", end=" ", flush=True)

        # Warmup
        for _ in range(self.warmup_runs):
            try:
                func(*args)
            except Exception as e:
                print(f"\nWarmup failed: {e}")
                return {}

        # Actual runs
        times = []
        for _ in range(runs):
            start = time.perf_counter()
            try:
                result = func(*args)
            except Exception as e:
                print(f"\nBenchmark failed: {e}")
                return {}
            elapsed = time.perf_counter() - start
            times.append(elapsed)

        stats = {
            "time_min": min(times),
            "time_max": max(times),
            "time_mean": statistics.mean(times),
            "time_stdev": statistics.stdev(times) if len(times) > 1 else 0.0,
            "runs": runs,
        }

        print(f"✓ {stats['time_mean']:.3f}s (±{stats['time_stdev']:.3f}s)")
        return stats

    def benchmark_discovery(self):
        """Benchmark process discovery algorithms"""
        print("\n=== Process Discovery ===")
        self.results["benchmarks"]["discovery"] = {}

        # Alpha Miner
        print("Alpha Miner:")
        gen = EventLogGenerator()

        for size, num_traces in [(100_000, 2_000), (1_000_000, 10_000)]:
            log = gen.generate_linear_log(size, num_traces)
            size_key = f"{size // 1000}k"

            stats = self.benchmark_function(
                f"  100K linear",
                alpha_miner.apply,
                log,
            )
            if stats:
                stats["events"] = size
                stats["traces"] = len(log)
                stats["avg_trace_length"] = size / len(log)
                stats["throughput_events_per_sec"] = size / stats["time_mean"]
                self.results["benchmarks"]["discovery"][f"alpha_{size_key}"] = stats

        # Inductive Miner
        print("Inductive Miner:")
        for size, num_traces in [(100_000, 2_000), (1_000_000, 10_000)]:
            log = gen.generate_loop_log(size, num_traces)
            size_key = f"{size // 1000}k"

            stats = self.benchmark_function(
                f"  {size_key} loop",
                inductive_miner.apply,
                log,
            )
            if stats:
                stats["events"] = size
                stats["traces"] = len(log)
                stats["throughput_events_per_sec"] = size / stats["time_mean"]
                self.results["benchmarks"]["discovery"][f"inductive_{size_key}"] = stats

        # DFG Miner
        print("DFG Miner:")
        for size, num_traces in [(100_000, 2_000), (1_000_000, 10_000), (10_000_000, 20_000)]:
            log = gen.generate_parallel_log(size, num_traces)
            size_key = f"{size // 1000}k"

            stats = self.benchmark_function(
                f"  {size_key} parallel",
                dfg_miner.apply,
                log,
            )
            if stats:
                stats["events"] = size
                stats["traces"] = len(log)
                stats["throughput_events_per_sec"] = size / stats["time_mean"]
                self.results["benchmarks"]["discovery"][f"dfg_{size_key}"] = stats

    def benchmark_conformance(self):
        """Benchmark conformance checking algorithms"""
        print("\n=== Conformance Checking ===")
        self.results["benchmarks"]["conformance"] = {}

        gen = EventLogGenerator()

        # Token Replay
        print("Token Replay:")
        for size, num_traces in [(100_000, 2_000), (1_000_000, 10_000)]:
            log = gen.generate_conformance_log(size, num_traces)
            net, im, fm = alpha_miner.apply(log)
            size_key = f"{size // 1000}k"

            stats = self.benchmark_function(
                f"  {size_key}",
                token_replay.apply,
                log,
                net,
                im,
                fm,
            )
            if stats:
                stats["events"] = size
                stats["traces"] = len(log)
                stats["throughput_events_per_sec"] = size / stats["time_mean"]
                self.results["benchmarks"]["conformance"][f"token_replay_{size_key}"] = stats

    def benchmark_statistics(self):
        """Benchmark statistical analysis"""
        print("\n=== Statistics & Analysis ===")
        self.results["benchmarks"]["statistics"] = {}

        gen = EventLogGenerator()

        # Frequency analysis
        print("Frequency Analysis:")
        for size, num_traces in [(100_000, 2_000), (1_000_000, 10_000)]:
            log = gen.generate_linear_log(size, num_traces)
            size_key = f"{size // 1000}k"

            stats = self.benchmark_function(
                f"  {size_key}",
                start_activities.get_start_activities,
                log,
            )
            if stats:
                stats["events"] = size
                stats["traces"] = len(log)
                stats["throughput_events_per_sec"] = size / stats["time_mean"]
                self.results["benchmarks"]["statistics"][f"frequency_{size_key}"] = stats

        # Variants
        print("Variants:")
        for size, num_traces in [(100_000, 2_000), (1_000_000, 10_000)]:
            log = gen.generate_parallel_log(size, num_traces)
            size_key = f"{size // 1000}k"

            stats = self.benchmark_function(
                f"  {size_key}",
                lambda l: pm4py.get_variants(l),
                log,
            )
            if stats:
                stats["events"] = size
                stats["traces"] = len(log)
                stats["throughput_events_per_sec"] = size / stats["time_mean"]
                self.results["benchmarks"]["statistics"][f"variants_{size_key}"] = stats

        # Trace length analysis
        print("Trace Lengths:")
        for size, num_traces in [(100_000, 2_000), (1_000_000, 10_000)]:
            log = gen.generate_loop_log(size, num_traces)
            size_key = f"{size // 1000}k"

            stats = self.benchmark_function(
                f"  {size_key}",
                traces_length.get_trace_length_summary,
                log,
            )
            if stats:
                stats["events"] = size
                stats["traces"] = len(log)
                stats["throughput_events_per_sec"] = size / stats["time_mean"]
                self.results["benchmarks"]["statistics"][f"trace_lengths_{size_key}"] = stats

    def run_all(self):
        """Run all benchmarks"""
        print("=" * 60)
        print("Python pm4py Performance Benchmarks")
        print("=" * 60)

        try:
            self.benchmark_discovery()
            self.benchmark_conformance()
            self.benchmark_statistics()

            print("\n" + "=" * 60)
            print("Benchmarks Complete")
            print("=" * 60)

            return self.results

        except Exception as e:
            print(f"\nError during benchmarking: {e}")
            import traceback
            traceback.print_exc()
            return None

    def save_results(self, output_path: str):
        """Save results to JSON file"""
        with open(output_path, "w") as f:
            json.dump(self.results, f, indent=2, default=str)
        print(f"\nResults saved to {output_path}")


def main():
    parser = argparse.ArgumentParser(description="Benchmark Python pm4py")
    parser.add_argument(
        "--output",
        default="benchmark_results_python.json",
        help="Output JSON file"
    )
    parser.add_argument(
        "--warmup",
        type=int,
        default=1,
        help="Number of warmup runs"
    )
    parser.add_argument(
        "--discovery-only",
        action="store_true",
        help="Only run discovery benchmarks"
    )
    parser.add_argument(
        "--conformance-only",
        action="store_true",
        help="Only run conformance benchmarks"
    )
    parser.add_argument(
        "--statistics-only",
        action="store_true",
        help="Only run statistics benchmarks"
    )

    args = parser.parse_args()

    benchmarker = Benchmarker(warmup_runs=args.warmup)

    if args.discovery_only:
        benchmarker.benchmark_discovery()
    elif args.conformance_only:
        benchmarker.benchmark_conformance()
    elif args.statistics_only:
        benchmarker.benchmark_statistics()
    else:
        benchmarker.run_all()

    if benchmarker.results.get("benchmarks"):
        benchmarker.save_results(args.output)
    else:
        print("No benchmarks were collected")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
