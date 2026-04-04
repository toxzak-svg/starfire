//! Causal Validation — Test hypotheses against observations
//!
//! Provides tools for validating causal hypotheses against
//! held-out observations.

use super::{CausalEngine, CausalHypothesis, DiscoveredCausalEdge};

/// Validation result for a hypothesis
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub hypothesis: String,
    pub holds: bool,
    pub support_ratio: f64,
    pub contradiction_count: usize,
    pub prediction_accuracy: f64,
}

impl ValidationResult {
    pub fn from_hypothesis(h: &CausalHypothesis, predictions: &[bool]) -> Self {
        let support_ratio = if h.supporting_observations + h.contradicting_observations > 0 {
            h.supporting_observations as f64
                / (h.supporting_observations + h.contradicting_observations) as f64
        } else {
            0.0
        };

        let accuracy = if !predictions.is_empty() {
            predictions.iter().filter(|&&p| p).count() as f64 / predictions.len() as f64
        } else {
            0.0
        };

        Self {
            hypothesis: format!("{} → {}", h.candidate.cause, h.candidate.effect),
            holds: support_ratio > 0.5,
            support_ratio,
            contradiction_count: h.contradicting_observations,
            prediction_accuracy: accuracy,
        }
    }
}

/// Causal validator
pub struct CausalValidator {
    /// Fraction of observations to use for testing (vs training)
    test_fraction: f64,
}

impl Default for CausalValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CausalValidator {
    pub fn new() -> Self {
        Self {
            test_fraction: 0.2,
        }
    }

    /// Validate a causal hypothesis by temporal split
    ///
    /// Train on first (1 - test_fraction) of observations,
    /// test on remaining test_fraction.
    pub fn validate_by_split(
        &self,
        hypothesis: &CausalHypothesis,
        observations: &[(i64, f64, f64)], // (timestamp, cause_value, effect_value)
    ) -> ValidationResult {
        if observations.len() < 5 {
            return ValidationResult {
                hypothesis: format!("{} → {}", hypothesis.candidate.cause, hypothesis.candidate.effect),
                holds: false,
                support_ratio: 0.0,
                contradiction_count: 0,
                prediction_accuracy: 0.0,
            };
        }

        let split_idx = ((observations.len() as f64) * (1.0 - self.test_fraction)) as usize;
        let train = &observations[..split_idx];
        let test = &observations[split_idx..];

        if train.is_empty() || test.is_empty() {
            return ValidationResult {
                hypothesis: format!("{} → {}", hypothesis.candidate.cause, hypothesis.candidate.effect),
                holds: false,
                support_ratio: 0.0,
                contradiction_count: 0,
                prediction_accuracy: 0.0,
            };
        }

        // Train: compute average effect given cause
        let train_cause_avg: f64 = train.iter().map(|(_t, c, _e)| c).sum::<f64>() / train.len() as f64;
        let train_effect_given_cause: f64 = train
            .iter()
            .filter(|(_t, c, _e)| *c > train_cause_avg * 0.9) // cause active
            .map(|(_t, _c, e)| e)
            .sum::<f64>()
            / train.len() as f64;

        // Test: predict and measure accuracy
        let threshold = train_effect_given_cause * 0.8; // Some tolerance
        let predictions: Vec<bool> = test
            .iter()
            .map(|(_t, c, e)| {
                let predicted = if *c > train_cause_avg * 0.9 { threshold } else { 0.0 };
                *e > predicted
            })
            .collect();

        ValidationResult::from_hypothesis(hypothesis, &predictions)
    }

    /// Validate multiple hypotheses
    pub fn validate_all(
        &self,
        engine: &CausalEngine,
        observations: &[(i64, f64, f64)],
    ) -> Vec<ValidationResult> {
        engine
            .hypotheses
            .iter()
            .map(|h| self.validate_by_split(h, observations))
            .collect()
    }

    /// Test counterfactual: would changing cause change effect?
    pub fn test_counterfactual(
        &self,
        cause_active: &[f64],
        cause_inactive: &[f64],
        effect_active: &[f64],
        effect_inactive: &[f64],
    ) -> bool {
        if cause_active.len() != effect_active.len()
            || cause_inactive.len() != effect_inactive.len()
            || cause_active.is_empty()
        {
            return false;
        }

        // Average effect when cause is active vs inactive
        let avg_active = effect_active.iter().sum::<f64>() / effect_active.len() as f64;
        let avg_inactive = effect_inactive.iter().sum::<f64>() / effect_inactive.len() as f64;

        // If effect is consistently higher when cause is active, hypothesis supported
        avg_active > avg_inactive * 1.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_from_hypothesis() {
        let edge = super::super::CausalEdge {
            id: super::super::CausalEdgeId::new(1),
            cause: "A".to_string(),
            effect: "B".to_string(),
            confidence: 0.7,
            evidence_count: 10,
            temporal_lag: None,
            mechanism: None,
        };
        let mut hypothesis = CausalHypothesis::new(edge);
        hypothesis.supporting_observations = 7;
        hypothesis.contradicting_observations = 3;

        let result = ValidationResult::from_hypothesis(&hypothesis, &[true, true, false, true, true]);

        assert!(result.holds);
        assert!((result.support_ratio - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_counterfactual() {
        let validator = CausalValidator::new();
        let result = validator.test_counterfactual(
            &[1.0, 1.0, 1.0],
            &[0.0, 0.0, 0.0],
            &[10.0, 11.0, 10.5],
            &[1.0, 1.5, 1.0],
        );
        assert!(result);
    }
}
