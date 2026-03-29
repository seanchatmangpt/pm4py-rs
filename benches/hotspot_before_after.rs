// Hotspot Optimization Benchmarks
//
// TDD approach: Benchmarks first (establish baselines), then optimize to meet targets.
// Measures improvement from each optimization independently and cumulatively.
//
// Performance targets:
// - Optimization 1 (node membership): 8-10% improvement
// - Optimization 2 (edge lookup): 6-8% improvement
// - Optimization 3 (parallel detection): 10-12% improvement
// - Optimization 4 (BFS early exit): 4-5% improvement
// - Optimization 5 (frequency scan): 3-4% improvement
// - Optimization 6 (memoization): 5-7% improvement
// Total: 30-45% expected

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pm4py::discovery::variants::Variant;
use pm4py::log::{Event, EventLog, Trace};
use pm4py::models::{Arc, PetriNet, Place, Transition};
use pm4py::optimization::cache_aware::{
    OptimizedPetriNet, OptimizedVariantAggregator, ParallelActivityDetector,
};
use pm4py::optimization::hotspot_elimination::{
    CalculationMemoizer, OptimizedReachabilityChecker, OptimizedVariantAnalyzer,
    SingleScanAggregator,
};
use std::collections::HashMap;

// ============================================================================
// Test Data Builders
// ============================================================================

fn create_large_petri_net(num_places: usize, num_transitions: usize) -> PetriNet {
    let mut net = PetriNet::new();

    // Add places
    for i in 0..num_places {
        let p = if i == 0 {
            Place::new(format!("p{}", i)).with_initial_marking(1)
        } else {
            Place::new(format!("p{}", i))
        };
        net.add_place(p);
    }

    // Add transitions
    for i in 0..num_transitions {
        let t = Transition::new(format!("t{}", i)).with_label(format!("a{}", i));
        net.add_transition(t);
    }

    // Add arcs to create a flow
    for i in 0..num_transitions.min(num_places - 1) {
        let from_place = &net.places[i].id.clone();
        let to_trans = &net.transitions[i].id.clone();
        let to_place = &net.places[(i + 1) % num_places].id.clone();

        net.add_arc(Arc::new(from_place, to_trans));
        net.add_arc(Arc::new(to_trans, to_place));
    }

    net
}

fn create_event_log(num_traces: usize, trace_length: usize) -> EventLog {
    let mut log = EventLog::new();
    let now = Utc::now();

    for trace_idx in 0..num_traces {
        let mut trace = Trace::new(format!("case_{}", trace_idx));

        for event_idx in 0..trace_length {
            let activity = format!("a{}", event_idx % 10);
            trace.add_event(Event::new(activity, now));
        }

        log.add_trace(trace);
    }

    log
}

fn create_variant_list(num_variants: usize, num_duplicate_sets: usize) -> Vec<Variant> {
    let mut variants = Vec::new();

    // Create base variants
    for i in 0..num_variants {
        let activities: Vec<String> = (0..5).map(|j| format!("a{}", (i + j) % 10)).collect();
        let variant = Variant::new(activities);

        // Duplicate each variant multiple times to simulate real frequency distributions
        for _ in 0..num_duplicate_sets {
            variants.push(variant.clone());
        }
    }

    variants
}

// ============================================================================
// OPTIMIZATION 1: Node Membership O(n)→O(1)
// ============================================================================

fn bench_node_membership_naive(c: &mut Criterion) {
    let net = black_box(create_large_petri_net(500, 400));
    let place_id = net.places[100].id.clone();

    c.bench_function("opt1_node_membership_naive", |b| {
        b.iter(|| {
            // Naive: linear search through places
            net.places.iter().find(|p| p.id == place_id).is_some()
        })
    });
}

fn bench_node_membership_optimized(c: &mut Criterion) {
    let net = create_large_petri_net(500, 400);
    let opt_net = black_box(OptimizedPetriNet::from_net(net));
    let place_id = opt_net.net.places[100].id.clone();

    c.bench_function("opt1_node_membership_optimized", |b| {
        b.iter(|| {
            // Optimized: O(1) HashSet lookup
            opt_net.contains_place(&place_id)
        })
    });
}

// ============================================================================
// OPTIMIZATION 2: Edge Lookup O(m)→O(1)
// ============================================================================

fn bench_edge_lookup_naive(c: &mut Criterion) {
    let net = black_box(create_large_petri_net(500, 400));
    let trans_id = &net.transitions[50].id;

    c.bench_function("opt2_edge_lookup_naive", |b| {
        b.iter(|| {
            // Naive: filter all arcs for matching transition
            net.arcs.iter().filter(|a| a.to == *trans_id).count()
        })
    });
}

fn bench_edge_lookup_optimized(c: &mut Criterion) {
    let net = create_large_petri_net(500, 400);
    let opt_net = black_box(OptimizedPetriNet::from_net(net));
    let trans_id = opt_net.net.transitions[50].id.clone();

    c.bench_function("opt2_edge_lookup_optimized", |b| {
        b.iter(|| {
            // Optimized: O(1) cache lookup
            opt_net
                .get_input_places(&trans_id)
                .map(|v| v.len())
                .unwrap_or(0)
        })
    });
}

// ============================================================================
// OPTIMIZATION 3: Parallel Activity Detection O(n²)→O(n)
// ============================================================================

fn bench_parallel_detection_naive(c: &mut Criterion) {
    let net = black_box(create_large_petri_net(200, 150));

    c.bench_function("opt3_parallel_detection_naive", |b| {
        b.iter(|| {
            // Naive: O(n²) nested loop
            let mut parallel = HashMap::new();
            for t1 in &net.transitions {
                for t2 in &net.transitions {
                    if t1.id != t2.id {
                        let can_parallel =
                            net.arcs.iter().any(|a| a.from == t1.id && a.to == t2.id);
                        if can_parallel {
                            parallel
                                .entry(t1.id.clone())
                                .or_insert_with(Vec::new)
                                .push(t2.id.clone());
                        }
                    }
                }
            }
            parallel
        })
    });
}

fn bench_parallel_detection_optimized(c: &mut Criterion) {
    let net = create_large_petri_net(200, 150);
    let opt_net = black_box(OptimizedPetriNet::from_net(net));

    c.bench_function("opt3_parallel_detection_optimized", |b| {
        b.iter(|| {
            // Optimized: O(n) single-pass
            ParallelActivityDetector::detect_parallel(&opt_net)
        })
    });
}

// ============================================================================
// OPTIMIZATION 4: Reachability BFS with Early Termination
// ============================================================================

fn bench_reachability_exhaustive(c: &mut Criterion) {
    let net = black_box(create_large_petri_net(100, 80));
    let mut initial = HashMap::new();
    initial.insert(net.places[0].id.clone(), 1);

    let mut target = HashMap::new();
    target.insert(net.places[50].id.clone(), 1);

    c.bench_function("opt4_reachability_exhaustive", |b| {
        b.iter(|| {
            // Count all reachable states (exhaustive)
            net.count_reachable_states(&initial)
        })
    });
}

fn bench_reachability_early_termination(c: &mut Criterion) {
    let net = create_large_petri_net(100, 80);
    let opt_net = black_box(OptimizedPetriNet::from_net(net));
    let mut initial = HashMap::new();
    initial.insert(opt_net.net.places[0].id.clone(), 1);

    let mut target = HashMap::new();
    target.insert(opt_net.net.places[50].id.clone(), 1);

    c.bench_function("opt4_reachability_early_termination", |b| {
        b.iter(|| {
            // Early termination when target found
            OptimizedReachabilityChecker::is_reachable(&opt_net.net, &initial, &target, 50)
        })
    });
}

// ============================================================================
// OPTIMIZATION 5: Variant Frequency Aggregation Single-Scan
// ============================================================================

fn bench_variant_frequency_naive(c: &mut Criterion) {
    let variants = black_box(create_variant_list(50, 100));

    c.bench_function("opt5_variant_frequency_naive", |b| {
        b.iter(|| {
            // Naive: insert + immediate sort
            let mut map = HashMap::new();
            for v in &variants {
                *map.entry(v.clone()).or_insert(0) += 1;
            }
            let mut result: Vec<_> = map.into_iter().collect();
            result.sort_by(|a, b| b.1.cmp(&a.1));
            result
        })
    });
}

fn bench_variant_frequency_optimized(c: &mut Criterion) {
    let variants = create_variant_list(50, 100);
    let variants = black_box(variants);

    c.bench_function("opt5_variant_frequency_optimized", |b| {
        b.iter(|| {
            // Optimized: single-pass aggregation, threshold filtering
            SingleScanAggregator::aggregate_threshold(variants.clone(), 10)
        })
    });
}

// ============================================================================
// OPTIMIZATION 6: Memoized Calculations
// ============================================================================

fn bench_memoization_no_cache(c: &mut Criterion) {
    let net = black_box(create_large_petri_net(100, 80));
    let mut initial = HashMap::new();
    initial.insert(net.places[0].id.clone(), 1);

    c.bench_function("opt6_memoization_no_cache", |b| {
        b.iter(|| {
            // No caching: recompute each time
            for i in 0..20 {
                OptimizedReachabilityChecker::is_reachable(
                    &net,
                    &initial,
                    &{
                        let mut m = HashMap::new();
                        m.insert(format!("p{}", i % net.places.len()), 1);
                        m
                    },
                    10,
                );
            }
        })
    });
}

fn bench_memoization_with_cache(c: &mut Criterion) {
    let net = create_large_petri_net(100, 80);
    let opt_net = black_box(OptimizedPetriNet::from_net(net));
    let mut initial = HashMap::new();
    initial.insert(opt_net.net.places[0].id.clone(), 1);

    c.bench_function("opt6_memoization_with_cache", |b| {
        b.iter(|| {
            // With caching: compute once, reuse
            let mut memoizer = CalculationMemoizer::new();
            for i in 0..20 {
                let _ = memoizer.is_reachable_memoized(
                    "p0",
                    &format!("p{}", i % opt_net.net.places.len()),
                    &opt_net.net,
                    &initial,
                );
            }
        })
    });
}

// ============================================================================
// CUMULATIVE OPTIMIZATION BENCHMARK
// ============================================================================

fn bench_cumulative_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("cumulative");

    for size in [100, 200, 500].iter() {
        let net = create_large_petri_net(*size, (*size as f64 * 0.8) as usize);
        let variants = create_variant_list(50, 100);

        // Naive: all operations without optimization
        group.bench_with_input(BenchmarkId::new("naive", size), size, |b, _| {
            b.iter(|| {
                let mut _results = 0;

                // Node lookup naive
                for place in &net.places {
                    let _ = net.places.iter().find(|p| p.id == place.id);
                    _results += 1;
                }

                // Variant frequency naive
                let mut map = HashMap::new();
                for v in &variants {
                    *map.entry(v.clone()).or_insert(0) += 1;
                }
                _results += map.len();

                _results
            })
        });

        // Optimized: all optimizations enabled
        let opt_net = OptimizedPetriNet::from_net(net);
        group.bench_with_input(BenchmarkId::new("optimized", size), size, |b, _| {
            b.iter(|| {
                let mut _results = 0;

                // Node lookup optimized
                for place in &opt_net.net.places {
                    let _ = opt_net.contains_place(&place.id);
                    _results += 1;
                }

                // Variant frequency optimized
                let freqs = SingleScanAggregator::aggregate_threshold(variants.clone(), 5);
                _results += freqs.len();

                _results
            })
        });
    }

    group.finish();
}

// ============================================================================
// Criterion Setup
// ============================================================================

criterion_group!(
    benches,
    bench_node_membership_naive,
    bench_node_membership_optimized,
    bench_edge_lookup_naive,
    bench_edge_lookup_optimized,
    bench_parallel_detection_naive,
    bench_parallel_detection_optimized,
    bench_reachability_exhaustive,
    bench_reachability_early_termination,
    bench_variant_frequency_naive,
    bench_variant_frequency_optimized,
    bench_memoization_no_cache,
    bench_memoization_with_cache,
    bench_cumulative_optimization,
);

criterion_main!(benches);
