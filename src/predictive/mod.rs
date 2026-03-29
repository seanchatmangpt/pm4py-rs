//! Predictive Process Analytics
//!
//! This module provides predictive capabilities for process mining, including:
//! - Remaining time prediction for ongoing cases
//! - Next activity prediction using Markov chains
//! - Outcome prediction and risk assessment

pub mod next_activity;
pub mod outcome_prediction;
pub mod remaining_time;

pub use next_activity::{ActivityPrediction, NextActivityPredictor};
pub use outcome_prediction::{CaseOutcome, OutcomePredictor, RiskAssessment};
pub use remaining_time::{
    predict_remaining_time_from_log, RemainingTimePrediction, RemainingTimePredictionResponse,
    RemainingTimePredictor,
};
