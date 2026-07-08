//! CHARGE emitters backed by real Starfire subsystem state.
//!
//! Emitters do not choose a resolver. They only translate unresolved state from
//! an existing subsystem into a typed [`Charge`]. Routing remains empirical.

use crate::metacog::KnowledgeGap;
use crate::prediction::{Prediction, PredictionOutcome};
use crate::quanot::QuanotResult;

use super::types::{Charge, ChargeKind, ChargeScope};

const TEXT_RESIDUAL_DIM: usize = 32;
const EPSILON: f64 = 1e-9;

/// Emits reservoir trajectory prediction residuals from consecutive Quanot states.
///
/// The emitter uses a constant-velocity local predictor:
///
/// `predicted_t = state_(t-1) + (state_(t-1) - state_(t-2))`
///
/// Any difference between the observed state and that prediction is unresolved
/// trajectory tension. The predictor is intentionally tiny and deterministic so
/// the experiment measures routing behavior rather than a learned forecaster.
#[derive(Debug, Clone, Default)]
pub struct QuanotTrajectoryEmitter {
    previous_state: Option<Vec<f64>>,
    previous_delta: Option<Vec<f64>>,
}

impl QuanotTrajectoryEmitter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observe(&mut self, result: &QuanotResult) -> Option<Charge> {
        self.observe_state(&result.reservoir_state)
    }

    pub fn observe_state(&mut self, state: &[f64]) -> Option<Charge> {
        if state.is_empty() || state.iter().any(|value| !value.is_finite()) {
            self.previous_state = None;
            self.previous_delta = None;
            return None;
        }

        let previous = match self.previous_state.as_ref() {
            Some(previous) if previous.len() == state.len() => previous.clone(),
            _ => {
                self.previous_state = Some(state.to_vec());
                self.previous_delta = None;
                return None;
            }
        };

        let current_delta: Vec<f64> = state
            .iter()
            .zip(previous.iter())
            .map(|(current, prior)| current - prior)
            .collect();

        let residual = self.previous_delta.as_ref().and_then(|previous_delta| {
            if previous_delta.len() != state.len() {
                return None;
            }

            let residual_f64: Vec<f64> = state
                .iter()
                .zip(previous.iter())
                .zip(previous_delta.iter())
                .map(|((current, prior), velocity)| current - (prior + velocity))
                .collect();

            let residual_rms = rms(&residual_f64);
            let state_rms = rms(state);
            let velocity_rms = rms(previous_delta);
            let magnitude = (residual_rms / (state_rms + velocity_rms + EPSILON)).clamp(0.0, 1.0);

            if magnitude <= f64::EPSILON {
                return None;
            }

            Some(Charge::new(
                ChargeKind::PredictionResidual,
                residual_f64.into_iter().map(|value| value as f32).collect(),
                magnitude as f32,
                ChargeScope::Reservoir,
            ))
        });

        self.previous_state = Some(state.to_vec());
        self.previous_delta = Some(current_delta);
        residual
    }

    pub fn reset(&mut self) {
        self.previous_state = None;
        self.previous_delta = None;
    }
}

/// Translate refuted or surprising Prediction Center evidence into contradiction charge.
pub fn prediction_contradiction_charge(
    prediction: &Prediction,
    outcome: PredictionOutcome,
    evidence_summary: &str,
) -> Option<Charge> {
    let outcome_weight = match outcome {
        PredictionOutcome::Refuted => 1.0,
        PredictionOutcome::Surprised => 0.75,
        PredictionOutcome::Confirmed | PredictionOutcome::Uncertain => return None,
    };

    let predicted = stable_text_vector(&prediction.description);
    let observed = stable_text_vector(evidence_summary);
    let residual: Vec<f32> = observed
        .iter()
        .zip(predicted.iter())
        .map(|(actual, expected)| actual - expected)
        .collect();
    let mismatch = rms_f32(&residual).clamp(0.25, 1.0);
    let magnitude = (prediction.confidence * outcome_weight * mismatch as f64).clamp(0.0, 1.0);

    if magnitude <= f64::EPSILON {
        return None;
    }

    Some(Charge::new(
        ChargeKind::Contradiction,
        residual,
        magnitude as f32,
        ChargeScope::Belief(format!("prediction:{:?}", prediction.id)),
    ))
}

/// Translate MetaCognition's unresolved knowledge-gap state into epistemic charge.
pub fn knowledge_gap_charge(gap: &KnowledgeGap) -> Option<Charge> {
    if gap.investigated {
        return None;
    }

    let remaining = (gap.importance * (1.0 - gap.progress.clamp(0.0, 1.0))).clamp(0.0, 1.0);
    if remaining <= f64::EPSILON {
        return None;
    }

    let residual = stable_text_vector(&gap.topic)
        .into_iter()
        .map(|value| value * remaining as f32)
        .collect();

    Some(Charge::new(
        ChargeKind::EpistemicGap,
        residual,
        remaining as f32,
        ChargeScope::Topic(gap.topic.clone()),
    ))
}

fn rms(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    (values.iter().map(|value| value * value).sum::<f64>() / values.len() as f64).sqrt()
}

fn rms_f32(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    (values
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        / values.len() as f32)
        .sqrt()
}

fn stable_text_vector(text: &str) -> Vec<f32> {
    let mut vector = vec![0.0f32; TEXT_RESIDUAL_DIM];

    for token in text
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
    {
        let hash = fnv1a(token.to_ascii_lowercase().as_bytes());
        let index = hash as usize % TEXT_RESIDUAL_DIM;
        let sign = if hash & (1 << 63) == 0 { 1.0 } else { -1.0 };
        vector[index] += sign;
    }

    let norm = vector
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt();
    if norm > f32::EPSILON {
        for value in &mut vector {
            *value /= norm;
        }
    }

    vector
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use crate::metacog::KnowledgeGap;
    use crate::prediction::{
        PredictedCore, Prediction, PredictionEngine, PredictionKind, PredictionOutcome,
    };

    use super::*;

    #[test]
    fn trajectory_emitter_waits_for_three_states_and_emits_on_mismatch() {
        let mut emitter = QuanotTrajectoryEmitter::new();
        assert!(emitter.observe_state(&[0.0, 0.0]).is_none());
        assert!(emitter.observe_state(&[1.0, 1.0]).is_none());

        let charge = emitter.observe_state(&[3.0, 1.0]).unwrap();
        assert_eq!(charge.kind, ChargeKind::PredictionResidual);
        assert_eq!(charge.scope, ChargeScope::Reservoir);
        assert!(charge.magnitude > 0.0);
    }

    #[test]
    fn prediction_emitter_only_charges_refuted_or_surprising_outcomes() {
        let prediction = Prediction::new(
            PredictionEngine::BeliefRevision,
            PredictionKind::Conclusion,
            PredictedCore::Conclusion {
                topic: "cache".into(),
                predicate: "misses are free".into(),
                confidence: 0.9,
            },
            "cache misses are free".into(),
            0.9,
            1,
            vec![],
        );

        assert!(prediction_contradiction_charge(
            &prediction,
            PredictionOutcome::Confirmed,
            "cache misses are free"
        )
        .is_none());

        let charge = prediction_contradiction_charge(
            &prediction,
            PredictionOutcome::Refuted,
            "cache misses fetch slower memory and add latency",
        )
        .unwrap();
        assert_eq!(charge.kind, ChargeKind::Contradiction);
        assert!(charge.magnitude > 0.0);
    }

    #[test]
    fn gap_emitter_tracks_unresolved_progress() {
        let mut gap = KnowledgeGap::new("dns", 0.8);
        let initial = knowledge_gap_charge(&gap).unwrap();
        gap.progress = 0.5;
        let partial = knowledge_gap_charge(&gap).unwrap();

        assert!(partial.magnitude < initial.magnitude);
        gap.investigated = true;
        assert!(knowledge_gap_charge(&gap).is_none());
    }
}
