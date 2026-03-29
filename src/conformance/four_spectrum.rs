use crate::conformance::{
    generalization::Generalization, precision::Precision, simplicity::Simplicity,
    token_replay::TokenReplay,
};
/// 4-Spectrum Quality Metric - Unified model quality assessment
///
/// Quality = w_f * F + w_p * P + w_g * G + w_s * S
/// Where F=Fitness, P=Precision, G=Generalization, S=Simplicity
use crate::log::EventLog;
use crate::models::PetriNet;

#[derive(Debug, Clone)]
pub struct SpectrumResult {
    pub fitness: f64,
    pub precision: f64,
    pub generalization: f64,
    pub simplicity: f64,
    pub quality_score: f64,
    pub weights: SpectrumWeights,
    pub diagnosis: String,
}

#[derive(Debug, Clone)]
pub struct SpectrumWeights {
    pub fitness_weight: f64,
    pub precision_weight: f64,
    pub generalization_weight: f64,
    pub simplicity_weight: f64,
}

impl Default for SpectrumWeights {
    fn default() -> Self {
        Self {
            fitness_weight: 0.25,
            precision_weight: 0.25,
            generalization_weight: 0.25,
            simplicity_weight: 0.25,
        }
    }
}

impl SpectrumWeights {
    pub fn is_valid(&self) -> bool {
        let sum = self.fitness_weight
            + self.precision_weight
            + self.generalization_weight
            + self.simplicity_weight;
        (sum - 1.0).abs() < 0.001
    }

    pub fn normalize(&mut self) {
        let sum = self.fitness_weight
            + self.precision_weight
            + self.generalization_weight
            + self.simplicity_weight;

        if sum > 0.0 {
            self.fitness_weight /= sum;
            self.precision_weight /= sum;
            self.generalization_weight /= sum;
            self.simplicity_weight /= sum;
        }
    }
}

pub struct FourSpectrum;

impl FourSpectrum {
    pub fn calculate(log: &EventLog, net: &PetriNet) -> SpectrumResult {
        let weights = SpectrumWeights::default();
        Self::calculate_with_weights(log, net, weights)
    }

    pub fn calculate_with_weights(
        log: &EventLog,
        net: &PetriNet,
        mut weights: SpectrumWeights,
    ) -> SpectrumResult {
        weights.normalize();

        let checker = TokenReplay::new();
        let fitness_result = checker.check(log, net);
        let fitness = fitness_result.fitness;

        let precision = Precision::calculate(log, net);
        let generalization = Generalization::calculate(log, net, 5);
        let simplicity = Simplicity::calculate(net);

        let quality_score = weights.fitness_weight * fitness
            + weights.precision_weight * precision
            + weights.generalization_weight * generalization
            + weights.simplicity_weight * simplicity;

        let diagnosis = Self::generate_diagnosis(fitness, precision, generalization, simplicity);

        SpectrumResult {
            fitness,
            precision,
            generalization,
            simplicity,
            quality_score,
            weights,
            diagnosis,
        }
    }

    fn generate_diagnosis(
        fitness: f64,
        precision: f64,
        generalization: f64,
        simplicity: f64,
    ) -> String {
        let mut issues = Vec::new();

        if fitness < 0.7 {
            issues.push("Low Fitness: Model does not replay the log well".to_string());
        }

        if precision < 0.7 {
            issues.push("Low Precision: Model is too permissive".to_string());
        }

        if generalization < 0.7 {
            issues.push("Low Generalization: Model may be overfitted".to_string());
        }

        if simplicity < 0.5 {
            issues.push("Low Simplicity: Model is too complex".to_string());
        }

        if issues.is_empty() {
            "Model quality is good across all dimensions.".to_string()
        } else {
            format!("Issues found:\n- {}", issues.join("\n- "))
        }
    }

    pub fn quality_rating(quality_score: f64) -> &'static str {
        match quality_score {
            s if s >= 0.8 => "Excellent",
            s if s >= 0.6 => "Good",
            s if s >= 0.4 => "Acceptable",
            s if s >= 0.2 => "Poor",
            _ => "Very Poor",
        }
    }

    pub fn conformance_focused_weights() -> SpectrumWeights {
        let mut weights = SpectrumWeights {
            fitness_weight: 0.4,
            precision_weight: 0.4,
            generalization_weight: 0.1,
            simplicity_weight: 0.1,
        };
        weights.normalize();
        weights
    }

    pub fn modeling_focused_weights() -> SpectrumWeights {
        let mut weights = SpectrumWeights {
            fitness_weight: 0.2,
            precision_weight: 0.2,
            generalization_weight: 0.35,
            simplicity_weight: 0.25,
        };
        weights.normalize();
        weights
    }

    pub fn format_report(result: &SpectrumResult) -> String {
        format!(
            "=== 4-Spectrum Quality Report ===\n\n\
             Fitness:       {:.1}%\n\
             Precision:     {:.1}%\n\
             Generalization: {:.1}%\n\
             Simplicity:    {:.1}%\n\n\
             Overall Quality: {:.1}% ({})\n\n\
             Diagnosis:\n{}\n",
            result.fitness * 100.0,
            result.precision * 100.0,
            result.generalization * 100.0,
            result.simplicity * 100.0,
            result.quality_score * 100.0,
            Self::quality_rating(result.quality_score),
            result.diagnosis,
        )
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
    fn test_spectrum_calculation() {
        let log = create_test_log();
        let net = create_simple_net();

        let result = FourSpectrum::calculate(&log, &net);

        assert!(result.fitness >= 0.0 && result.fitness <= 1.0);
        assert!(result.precision >= 0.0 && result.precision <= 1.0);
        assert!(result.generalization >= 0.0 && result.generalization <= 1.0);
        assert!(result.simplicity >= 0.0 && result.simplicity <= 1.0);
        assert!(result.quality_score >= 0.0 && result.quality_score <= 1.0);
    }

    #[test]
    fn test_weights_normalization() {
        let mut weights = SpectrumWeights {
            fitness_weight: 1.0,
            precision_weight: 2.0,
            generalization_weight: 3.0,
            simplicity_weight: 4.0,
        };

        assert!(!weights.is_valid());
        weights.normalize();
        assert!(weights.is_valid());
    }

    #[test]
    fn test_quality_rating() {
        assert_eq!(FourSpectrum::quality_rating(0.9), "Excellent");
        assert_eq!(FourSpectrum::quality_rating(0.7), "Good");
        assert_eq!(FourSpectrum::quality_rating(0.5), "Acceptable");
        assert_eq!(FourSpectrum::quality_rating(0.3), "Poor");
        assert_eq!(FourSpectrum::quality_rating(0.1), "Very Poor");
    }

    #[test]
    fn test_format_report() {
        let log = create_test_log();
        let net = create_simple_net();
        let result = FourSpectrum::calculate(&log, &net);

        let report = FourSpectrum::format_report(&result);

        assert!(report.contains("4-Spectrum"));
        assert!(report.contains("Fitness"));
    }
}
