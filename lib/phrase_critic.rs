//! STLM L1-D offline recurrent phrase critic.
//!
//! The critic receives only candidates that already passed external semantic,
//! slot-preservation, and identity-conflict checks. Its learned score is a
//! bounded residual on top of the deterministic rule score, never the primary
//! authority. The module remains default-off and has no Runtime::chat, HTTP,
//! persistence, routing, tool, belief, ontology, CHARGE, or action authority.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

pub const PHRASE_CRITIC_SCHEMA_VERSION: u16 = 1;
pub const PHRASE_CRITIC_CONTEXT_SIZE: usize = 8;
pub const PHRASE_CRITIC_VOCABULARY_SIZE: usize = 128;
pub const MAX_PHRASE_CRITIC_HIDDEN_SIZE: usize = 128;
pub const MAX_PHRASE_CRITIC_CANDIDATES: usize = 32;
pub const MAX_PHRASE_CRITIC_TEXT_BYTES: usize = 1_024;
pub const PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS: i32 = 250;
pub const PHRASE_CRITIC_MAX_PAIRWISE_LEARNED_SWING_BPS: i32 =
    PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS * 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhraseCriticModel {
    pub schema_version: u16,
    pub vocabulary_size: usize,
    pub hidden_size: usize,
    pub context_size: usize,
    pub embeddings: Vec<Vec<f32>>,
    pub recurrent_weights: Vec<Vec<f32>>,
    pub context_weights: Vec<Vec<f32>>,
    pub hidden_bias: Vec<f32>,
    pub output_weights: Vec<f32>,
    pub output_bias: f32,
}

impl PhraseCriticModel {
    pub fn from_json(json: &str) -> Result<Self, PhraseCriticError> {
        let model: Self = serde_json::from_str(json)
            .map_err(|error| PhraseCriticError::ModelDecode(error.to_string()))?;
        model.verify_integrity()?;
        Ok(model)
    }

    pub fn to_json(&self) -> Result<String, PhraseCriticError> {
        self.verify_integrity()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| PhraseCriticError::ModelDecode(error.to_string()))
    }

    pub fn verify_integrity(&self) -> Result<(), PhraseCriticError> {
        if self.schema_version != PHRASE_CRITIC_SCHEMA_VERSION
            || self.vocabulary_size != PHRASE_CRITIC_VOCABULARY_SIZE
            || self.context_size != PHRASE_CRITIC_CONTEXT_SIZE
            || self.hidden_size == 0
            || self.hidden_size > MAX_PHRASE_CRITIC_HIDDEN_SIZE
        {
            return Err(PhraseCriticError::InvalidModelShape);
        }
        if self.embeddings.len() != self.vocabulary_size
            || self
                .embeddings
                .iter()
                .any(|row| row.len() != self.hidden_size)
            || self.recurrent_weights.len() != self.hidden_size
            || self
                .recurrent_weights
                .iter()
                .any(|row| row.len() != self.hidden_size)
            || self.context_weights.len() != self.context_size
            || self
                .context_weights
                .iter()
                .any(|row| row.len() != self.hidden_size)
            || self.hidden_bias.len() != self.hidden_size
            || self.output_weights.len() != self.hidden_size
        {
            return Err(PhraseCriticError::InvalidModelShape);
        }
        let all_finite = self
            .embeddings
            .iter()
            .flatten()
            .chain(self.recurrent_weights.iter().flatten())
            .chain(self.context_weights.iter().flatten())
            .chain(self.hidden_bias.iter())
            .chain(self.output_weights.iter())
            .chain(std::iter::once(&self.output_bias))
            .all(|value| value.is_finite());
        if !all_finite {
            return Err(PhraseCriticError::NonFiniteModelWeight);
        }
        Ok(())
    }

    fn score_text(
        &self,
        context: &PhraseCriticContext,
        text: &str,
    ) -> Result<i32, PhraseCriticError> {
        self.verify_integrity()?;
        context.verify_integrity()?;
        if text.trim().is_empty() || text.len() > MAX_PHRASE_CRITIC_TEXT_BYTES {
            return Err(PhraseCriticError::InvalidCandidateText);
        }

        let context_vector = context.as_normalized_vector();
        let mut context_projection = vec![0.0_f32; self.hidden_size];
        for (context_index, context_value) in context_vector.iter().enumerate() {
            for (hidden_index, projected) in context_projection.iter_mut().enumerate() {
                *projected += context_value * self.context_weights[context_index][hidden_index];
            }
        }

        let mut hidden = vec![0.0_f32; self.hidden_size];
        for byte in text.bytes() {
            let token = usize::from(if byte < 128 { byte } else { 127 });
            let previous = hidden.clone();
            for hidden_index in 0..self.hidden_size {
                let mut activation = self.embeddings[token][hidden_index]
                    + self.hidden_bias[hidden_index]
                    + context_projection[hidden_index];
                for (previous_index, previous_value) in previous.iter().enumerate() {
                    activation +=
                        previous_value * self.recurrent_weights[hidden_index][previous_index];
                }
                hidden[hidden_index] = activation.tanh();
            }
        }

        let logit = self
            .output_weights
            .iter()
            .zip(hidden.iter())
            .fold(self.output_bias, |sum, (weight, value)| {
                sum + weight * value
            });
        if !logit.is_finite() {
            return Err(PhraseCriticError::NonFiniteInference);
        }
        let probability = 1.0_f32 / (1.0_f32 + (-logit.clamp(-20.0, 20.0)).exp());
        Ok((probability * 10_000.0).round() as i32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhraseCriticContext {
    pub directness_bps: u16,
    pub warmth_bps: u16,
    pub energy_bps: u16,
    pub compression_bps: u16,
    pub playfulness_bps: u16,
    pub novelty_pressure_bps: u16,
    pub identity_relevance_bps: u16,
    pub semantic_specificity_bps: u16,
}

impl PhraseCriticContext {
    pub fn verify_integrity(&self) -> Result<(), PhraseCriticError> {
        if self.as_basis_points().iter().any(|value| *value > 10_000) {
            return Err(PhraseCriticError::InvalidContext);
        }
        Ok(())
    }

    fn as_basis_points(&self) -> [u16; PHRASE_CRITIC_CONTEXT_SIZE] {
        [
            self.directness_bps,
            self.warmth_bps,
            self.energy_bps,
            self.compression_bps,
            self.playfulness_bps,
            self.novelty_pressure_bps,
            self.identity_relevance_bps,
            self.semantic_specificity_bps,
        ]
    }

    fn as_normalized_vector(&self) -> [f32; PHRASE_CRITIC_CONTEXT_SIZE] {
        self.as_basis_points()
            .map(|value| f32::from(value) / 5_000.0 - 1.0)
    }
}

impl Default for PhraseCriticContext {
    fn default() -> Self {
        Self {
            directness_bps: 7_000,
            warmth_bps: 5_000,
            energy_bps: 5_500,
            compression_bps: 6_500,
            playfulness_bps: 3_500,
            novelty_pressure_bps: 7_000,
            identity_relevance_bps: 5_000,
            semantic_specificity_bps: 8_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhraseCriticCandidate {
    pub candidate_id: u16,
    pub text: String,
    pub semantic_verified: bool,
    pub slots_preserved: bool,
    pub identity_conflicts: u16,
    pub rule_score: i64,
}

impl PhraseCriticCandidate {
    fn hard_gate_passed(&self) -> bool {
        self.semantic_verified && self.slots_preserved && self.identity_conflicts == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhraseCriticSelection {
    pub selected_candidate_id: u16,
    pub learned_score_bps: i32,
    pub learned_residual_bps: i32,
    pub rule_score: i64,
    pub combined_score: i64,
    pub complete_candidates_scored: u16,
    pub candidates_rejected_by_hard_gate: u16,
}

#[derive(Debug, Clone)]
struct ScoredCandidate {
    candidate_id: u16,
    learned_score_bps: i32,
    learned_residual_bps: i32,
    rule_score: i64,
    combined_score: i64,
}

#[derive(Debug, Clone)]
pub struct PhraseCritic {
    model: PhraseCriticModel,
}

impl PhraseCritic {
    pub fn new(model: PhraseCriticModel) -> Result<Self, PhraseCriticError> {
        model.verify_integrity()?;
        Ok(Self { model })
    }

    pub fn select(
        &self,
        context: &PhraseCriticContext,
        candidates: &[PhraseCriticCandidate],
    ) -> Result<PhraseCriticSelection, PhraseCriticError> {
        context.verify_integrity()?;
        if candidates.is_empty() || candidates.len() > MAX_PHRASE_CRITIC_CANDIDATES {
            return Err(PhraseCriticError::CandidateBudgetExceeded);
        }

        let mut scored = Vec::with_capacity(candidates.len());
        let mut rejected = 0_u16;
        for candidate in candidates {
            if !candidate.hard_gate_passed() {
                rejected = rejected.saturating_add(1);
                continue;
            }
            let learned_score_bps = self.model.score_text(context, &candidate.text)?;
            let learned_residual_bps = bounded_learned_residual(learned_score_bps);
            let combined_score = candidate
                .rule_score
                .saturating_add(i64::from(learned_residual_bps));
            scored.push(ScoredCandidate {
                candidate_id: candidate.candidate_id,
                learned_score_bps,
                learned_residual_bps,
                rule_score: candidate.rule_score,
                combined_score,
            });
        }
        if scored.is_empty() {
            return Err(PhraseCriticError::NoEligibleCandidate);
        }
        scored.sort_by(compare_scored_candidates);
        let selected = &scored[0];
        Ok(PhraseCriticSelection {
            selected_candidate_id: selected.candidate_id,
            learned_score_bps: selected.learned_score_bps,
            learned_residual_bps: selected.learned_residual_bps,
            rule_score: selected.rule_score,
            combined_score: selected.combined_score,
            complete_candidates_scored: u16::try_from(scored.len())
                .map_err(|_| PhraseCriticError::CandidateBudgetExceeded)?,
            candidates_rejected_by_hard_gate: rejected,
        })
    }
}

#[must_use]
pub const fn bounded_learned_residual(learned_score_bps: i32) -> i32 {
    let centered = learned_score_bps - 5_000;
    if centered < -PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS {
        -PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS
    } else if centered > PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS {
        PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS
    } else {
        centered
    }
}

fn compare_scored_candidates(left: &ScoredCandidate, right: &ScoredCandidate) -> Ordering {
    right
        .combined_score
        .cmp(&left.combined_score)
        .then_with(|| right.rule_score.cmp(&left.rule_score))
        .then_with(|| left.candidate_id.cmp(&right.candidate_id))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhraseCriticAuthorityBoundary {
    pub verified_candidate_text_access: bool,
    pub bounded_context_access: bool,
    pub learned_wording_rank: bool,
    pub learned_primary_rank: bool,
    pub learned_residual_limit_bps: u16,
    pub hard_semantic_gate_override: bool,
    pub identity_conflict_override: bool,
    pub selected_text_return: bool,
    pub candidate_text_persistence: bool,
    pub runtime_chat_influence: bool,
    pub http_response_influence: bool,
    pub unrestricted_memory_access: bool,
    pub belief_promotion_authority: bool,
    pub ontology_promotion_authority: bool,
    pub routing_authority: bool,
    pub tool_selection_authority: bool,
    pub autonomous_action_authority: bool,
}

#[must_use]
pub const fn authority_boundary() -> PhraseCriticAuthorityBoundary {
    PhraseCriticAuthorityBoundary {
        verified_candidate_text_access: true,
        bounded_context_access: true,
        learned_wording_rank: true,
        learned_primary_rank: false,
        learned_residual_limit_bps: PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS as u16,
        hard_semantic_gate_override: false,
        identity_conflict_override: false,
        selected_text_return: false,
        candidate_text_persistence: false,
        runtime_chat_influence: false,
        http_response_influence: false,
        unrestricted_memory_access: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        routing_authority: false,
        tool_selection_authority: false,
        autonomous_action_authority: false,
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PhraseCriticError {
    #[error("phrase critic model JSON is invalid: {0}")]
    ModelDecode(String),
    #[error("phrase critic model dimensions or schema are invalid")]
    InvalidModelShape,
    #[error("phrase critic model contains a non-finite weight")]
    NonFiniteModelWeight,
    #[error("phrase critic context contains an out-of-range value")]
    InvalidContext,
    #[error("phrase critic candidate text is empty or over budget")]
    InvalidCandidateText,
    #[error("phrase critic candidate budget is empty or exceeded")]
    CandidateBudgetExceeded,
    #[error("no candidate passed semantic, slot, and identity hard gates")]
    NoEligibleCandidate,
    #[error("phrase critic inference produced a non-finite value")]
    NonFiniteInference,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toy_model() -> PhraseCriticModel {
        let hidden_size = 2;
        let mut embeddings = vec![vec![0.0; hidden_size]; PHRASE_CRITIC_VOCABULARY_SIZE];
        embeddings[usize::from(b'!')][0] = 2.0;
        embeddings[usize::from(b'.')][0] = -2.0;
        PhraseCriticModel {
            schema_version: PHRASE_CRITIC_SCHEMA_VERSION,
            vocabulary_size: PHRASE_CRITIC_VOCABULARY_SIZE,
            hidden_size,
            context_size: PHRASE_CRITIC_CONTEXT_SIZE,
            embeddings,
            recurrent_weights: vec![vec![0.0; hidden_size]; hidden_size],
            context_weights: vec![vec![0.0; hidden_size]; PHRASE_CRITIC_CONTEXT_SIZE],
            hidden_bias: vec![0.0; hidden_size],
            output_weights: vec![2.0, 0.0],
            output_bias: 0.0,
        }
    }

    fn candidate(id: u16, text: &str, rule_score: i64) -> PhraseCriticCandidate {
        PhraseCriticCandidate {
            candidate_id: id,
            text: text.to_string(),
            semantic_verified: true,
            slots_preserved: true,
            identity_conflicts: 0,
            rule_score,
        }
    }

    #[test]
    fn exact_replay_is_identical() {
        let critic = PhraseCritic::new(toy_model()).unwrap();
        let candidates = vec![
            candidate(1, "That follows.", 20),
            candidate(2, "That follows!", 10),
        ];
        let first = critic
            .select(&PhraseCriticContext::default(), &candidates)
            .unwrap();
        let second = critic
            .select(&PhraseCriticContext::default(), &candidates)
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.selected_candidate_id, 2);
        assert!(first.learned_residual_bps.abs() <= PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS);
    }

    #[test]
    fn hard_gates_override_any_rule_or_learned_score() {
        let critic = PhraseCritic::new(toy_model()).unwrap();
        let mut drifted = candidate(1, "Different claim!", i64::MAX);
        drifted.semantic_verified = false;
        let valid = candidate(2, "Same claim.", 0);
        let selected = critic
            .select(&PhraseCriticContext::default(), &[drifted, valid])
            .unwrap();
        assert_eq!(selected.selected_candidate_id, 2);
        assert_eq!(selected.candidates_rejected_by_hard_gate, 1);
    }

    #[test]
    fn identity_conflicts_are_not_rankable() {
        let critic = PhraseCritic::new(toy_model()).unwrap();
        let mut conflict = candidate(1, "Identity contradiction!", i64::MAX);
        conflict.identity_conflicts = 1;
        let valid = candidate(2, "Identity-consistent wording.", 0);
        let selected = critic
            .select(&PhraseCriticContext::default(), &[conflict, valid])
            .unwrap();
        assert_eq!(selected.selected_candidate_id, 2);
    }

    #[test]
    fn rule_score_remains_primary_outside_residual_band() {
        let critic = PhraseCritic::new(toy_model()).unwrap();
        let rule_winner = candidate(1, "Lower learned score.", 1_000);
        let learned_winner = candidate(2, "Higher learned score!", 499);
        let selected = critic
            .select(
                &PhraseCriticContext::default(),
                &[rule_winner, learned_winner],
            )
            .unwrap();
        assert_eq!(selected.selected_candidate_id, 1);
    }

    #[test]
    fn learned_residual_can_resolve_a_near_tie() {
        let critic = PhraseCritic::new(toy_model()).unwrap();
        let rule_winner = candidate(1, "Lower learned score.", 1_000);
        let learned_winner = candidate(2, "Higher learned score!", 501);
        let selected = critic
            .select(
                &PhraseCriticContext::default(),
                &[rule_winner, learned_winner],
            )
            .unwrap();
        assert_eq!(selected.selected_candidate_id, 2);
    }

    #[test]
    fn malformed_models_fail_closed() {
        let mut model = toy_model();
        model.embeddings.pop();
        assert_eq!(
            PhraseCritic::new(model).unwrap_err(),
            PhraseCriticError::InvalidModelShape
        );
    }

    #[test]
    fn authority_boundary_is_closed_and_residual_only() {
        let boundary = authority_boundary();
        assert!(boundary.learned_wording_rank);
        assert!(!boundary.learned_primary_rank);
        assert_eq!(
            boundary.learned_residual_limit_bps,
            PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS as u16
        );
        assert!(!boundary.hard_semantic_gate_override);
        assert!(!boundary.identity_conflict_override);
        assert!(!boundary.selected_text_return);
        assert!(!boundary.runtime_chat_influence);
        assert!(!boundary.http_response_influence);
        assert!(!boundary.belief_promotion_authority);
        assert!(!boundary.ontology_promotion_authority);
        assert!(!boundary.autonomous_action_authority);
    }
}
