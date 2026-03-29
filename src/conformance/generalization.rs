/// Generalization Metric for Conformance Checking
///
/// Generalization measures how well a model generalizes to unseen behavior.
/// Uses cross-validation: split log into k folds, train on k-1, test on 1.
///
/// Generalization = average fitness on test sets
use crate::conformance::TokenReplay;
use crate::log::EventLog;
use crate::models::PetriNet;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct Generalization;

impl Generalization {
    /// Calculate generalization metric using k-fold cross-validation
    pub fn calculate(log: &EventLog, net: &PetriNet, num_folds: usize) -> f64 {
        if log.len() < 2 {
            return 1.0;
        }

        let num_folds = num_folds.max(2).min(log.len());
        let folds = Self::create_folds(log, num_folds);

        let mut fold_fitness_scores = Vec::new();

        for fold in &folds {
            let mut test_log = EventLog::new();
            test_log.attributes = log.attributes.clone();
            for trace in fold {
                test_log.add_trace(trace.clone());
            }

            if !test_log.is_empty() {
                let checker = TokenReplay::new();
                let result = checker.check(&test_log, net);
                fold_fitness_scores.push(result.fitness);
            }
        }

        if fold_fitness_scores.is_empty() {
            return 1.0;
        }

        fold_fitness_scores.iter().sum::<f64>() / fold_fitness_scores.len() as f64
    }

    /// Split log into k folds
    fn create_folds(log: &EventLog, k: usize) -> Vec<Vec<crate::log::Trace>> {
        let mut rng = thread_rng();
        let mut traces = log.traces.clone();
        traces.shuffle(&mut rng);

        let fold_size = traces.len().div_ceil(k);
        let mut folds: Vec<Vec<crate::log::Trace>> = vec![Vec::new(); k];

        for (i, trace) in traces.into_iter().enumerate() {
            let fold_idx = i / fold_size;
            if fold_idx < k {
                folds[fold_idx].push(trace);
            }
        }

        folds
    }

    /// Train/test split generalization (80/20 by default)
    pub fn calculate_with_split(log: &EventLog, net: &PetriNet, train_ratio: f64) -> f64 {
        if log.len() < 2 {
            return 1.0;
        }

        let train_ratio = train_ratio.clamp(0.1, 0.9);

        let mut rng = thread_rng();
        let mut traces = log.traces.clone();
        traces.shuffle(&mut rng);

        let split_point = (traces.len() as f64 * train_ratio) as usize;

        let mut test_log = EventLog::new();
        test_log.attributes = log.attributes.clone();

        for trace in &traces[split_point..] {
            test_log.add_trace(trace.clone());
        }

        if test_log.is_empty() {
            return 1.0;
        }

        let checker = TokenReplay::new();
        let result = checker.check(&test_log, net);

        result.fitness
    }

    /// Multiple random splits
    pub fn calculate_with_multiple_splits(
        log: &EventLog,
        net: &PetriNet,
        num_splits: usize,
        train_ratio: f64,
    ) -> f64 {
        if log.len() < 2 {
            return 1.0;
        }

        let mut scores = Vec::new();

        for _ in 0..num_splits {
            let score = Self::calculate_with_split(log, net, train_ratio);
            scores.push(score);
        }

        if scores.is_empty() {
            return 1.0;
        }

        scores.iter().sum::<f64>() / scores.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::{Event, Trace};
    use crate::models::petri_net::{Arc, Place, Transition};
    use chrono::Utc;

    fn create_test_log() -> EventLog {
        let mut log = EventLog::new();
        let now = Utc::now();

        for i in 0..10 {
            let mut trace = Trace::new(format!("case_{}", i));
            trace.add_event(Event::new("a", now));
            trace.add_event(Event::new("b", now));
            trace.add_event(Event::new("c", now));
            log.add_trace(trace);
        }

        log
    }

    fn create_simple_net() -> PetriNet {
        let mut net = PetriNet::new();

        let p1 = Place::new("p1").with_initial_marking(1);
        let t1 = Transition::new("t1").with_label("a");
        let p2 = Place::new("p2");
        let t2 = Transition::new("t2").with_label("b");
        let p3 = Place::new("p3");
        let t3 = Transition::new("t3").with_label("c");
        let p4 = Place::new("p4").with_final_marking(1);

        let ids = (
            p1.id.clone(),
            t1.id.clone(),
            p2.id.clone(),
            t2.id.clone(),
            p3.id.clone(),
            t3.id.clone(),
            p4.id.clone(),
        );

        net.add_place(p1);
        net.add_transition(t1);
        net.add_place(p2);
        net.add_transition(t2);
        net.add_place(p3);
        net.add_transition(t3);
        net.add_place(p4);

        net.add_arc(Arc::new(&ids.0, &ids.1));
        net.add_arc(Arc::new(&ids.1, &ids.2));
        net.add_arc(Arc::new(&ids.2, &ids.3));
        net.add_arc(Arc::new(&ids.3, &ids.4));
        net.add_arc(Arc::new(&ids.4, &ids.5));
        net.add_arc(Arc::new(&ids.5, &ids.6));

        net.set_initial_place(ids.0);
        net.set_final_place(ids.6);

        net
    }

    #[test]
    fn test_generalization_calculation() {
        let log = create_test_log();
        let net = create_simple_net();

        let generalization = Generalization::calculate(&log, &net, 5);

        assert!(generalization >= 0.0 && generalization <= 1.0);
    }

    #[test]
    fn test_generalization_with_split() {
        let log = create_test_log();
        let net = create_simple_net();

        let generalization = Generalization::calculate_with_split(&log, &net, 0.8);

        assert!(generalization >= 0.0 && generalization <= 1.0);
    }

    #[test]
    fn test_generalization_with_multiple_splits() {
        let log = create_test_log();
        let net = create_simple_net();

        let generalization = Generalization::calculate_with_multiple_splits(&log, &net, 3, 0.8);

        assert!(generalization >= 0.0 && generalization <= 1.0);
    }

    #[test]
    fn test_generalization_with_empty_log() {
        let log = EventLog::new();
        let net = create_simple_net();

        let generalization = Generalization::calculate(&log, &net, 5);

        assert_eq!(generalization, 1.0);
    }

    #[test]
    fn test_fold_creation() {
        let log = create_test_log();
        let folds = Generalization::create_folds(&log, 5);

        assert_eq!(folds.len(), 5);
        assert!(!folds.is_empty());

        let total_traces: usize = folds.iter().map(|f| f.len()).sum();
        assert_eq!(total_traces, log.len());
    }
}
