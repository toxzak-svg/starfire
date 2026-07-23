//! EI-0A canonical developmental episode contracts.
//!
//! This module is deliberately data-only. It has no persistence, runtime-chat,
//! tool, response, or learning-update authority. Later EI milestones may use
//! these sealed records as evidence, but cannot infer authority from their
//! existence.

use serde::{Deserialize, Serialize};
use thiserror::Error;

const DIGEST_DOMAIN: &[u8] = b"starfire/eI-0A/cognitive-episode/v1";
const MAX_PROBABILITY_BPS: u16 = 10_000;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EpisodeId(String);

impl EpisodeId {
    pub fn new(value: impl Into<String>) -> Result<Self, EpisodeContractError> {
        canonical_identifier(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TaskFamilyId(String);

impl TaskFamilyId {
    pub fn new(value: impl Into<String>) -> Result<Self, EpisodeContractError> {
        canonical_identifier(value.into()).map(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EvidenceRef(String);

impl EvidenceRef {
    pub fn new(value: impl Into<String>) -> Result<Self, EpisodeContractError> {
        canonical_identifier(value.into()).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpisodePhase {
    Development,
    WithinFamilyHoldout,
    RenamedVocabularyTransfer,
    StructuralTransfer,
    Regression,
    Adversarial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub kind: String,
    /// Canonically sorted, deduplicated facts exposed by the environment.
    pub facts: Vec<String>,
}

impl Observation {
    pub fn new(kind: impl Into<String>, facts: Vec<String>) -> Result<Self, EpisodeContractError> {
        let kind = canonical_identifier(kind.into())?;
        let facts = canonical_strings(facts)?;
        Ok(Self { kind, facts })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prediction {
    pub proposition: String,
    pub probability_bps: u16,
    pub evidence: Vec<EvidenceRef>,
    /// EI scoring rejects post-outcome predictions.
    pub created_before_action: bool,
}

impl Prediction {
    pub fn new(
        proposition: impl Into<String>,
        probability_bps: u16,
        evidence: Vec<EvidenceRef>,
    ) -> Result<Self, EpisodeContractError> {
        if probability_bps > MAX_PROBABILITY_BPS {
            return Err(EpisodeContractError::ProbabilityOutOfRange(probability_bps));
        }
        if evidence.is_empty() {
            return Err(EpisodeContractError::MissingPredictionEvidence);
        }
        Ok(Self {
            proposition: canonical_identifier(proposition.into())?,
            probability_bps,
            evidence,
            created_before_action: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategySelection {
    pub strategy_id: String,
    pub rationale_evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intention {
    pub objective: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedAction {
    pub action: String,
    pub declared_cost: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub objective_satisfied: bool,
    pub score_bps: u16,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeEvaluation {
    pub prediction_correct: bool,
    pub action_score_bps: u16,
    pub evaluator_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoritySnapshot {
    pub runtime_chat_authority: bool,
    pub persistence_authority: bool,
    pub learning_update_authority: bool,
    pub tool_authority: bool,
    pub ontology_promotion_authority: bool,
}

impl AuthoritySnapshot {
    pub fn shadow_only() -> Self {
        Self {
            runtime_chat_authority: false,
            persistence_authority: false,
            learning_update_authority: false,
            tool_authority: false,
            ontology_promotion_authority: false,
        }
    }

    fn has_any_authority(&self) -> bool {
        self.runtime_chat_authority
            || self.persistence_authority
            || self.learning_update_authority
            || self.tool_authority
            || self.ontology_promotion_authority
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeProvenance {
    pub cohort_id: String,
    pub fixture_digest: String,
    pub seed: u64,
    pub generator_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitiveEpisode {
    pub episode_id: EpisodeId,
    pub task_family: TaskFamilyId,
    pub phase: EpisodePhase,
    pub observation: Observation,
    pub retrieved_evidence: Vec<EvidenceRef>,
    pub predictions: Vec<Prediction>,
    pub selected_strategy: StrategySelection,
    pub intention: Intention,
    pub action: BoundedAction,
    pub outcome: Option<Outcome>,
    pub evaluation: Option<EpisodeEvaluation>,
    pub authority_snapshot: AuthoritySnapshot,
    pub provenance: EpisodeProvenance,
}

impl CognitiveEpisode {
    /// Validate EI-0A's pre-learning, shadow-only evidence boundary.
    pub fn validate(&self) -> Result<(), EpisodeContractError> {
        validate_identifier(self.episode_id.as_str())?;
        validate_identifier(&self.task_family.0)?;
        validate_observation(&self.observation)?;
        validate_identifier(&self.selected_strategy.strategy_id)?;
        validate_identifier(&self.intention.objective)?;
        validate_identifier(&self.action.action)?;
        validate_provenance(&self.provenance)?;
        if self.predictions.is_empty() {
            return Err(EpisodeContractError::MissingPrediction);
        }
        for prediction in &self.predictions {
            validate_identifier(&prediction.proposition)?;
            if prediction.probability_bps > MAX_PROBABILITY_BPS || prediction.evidence.is_empty() {
                return Err(EpisodeContractError::InvalidPrediction);
            }
        }
        if self
            .predictions
            .iter()
            .any(|prediction| !prediction.created_before_action)
        {
            return Err(EpisodeContractError::PostOutcomePrediction);
        }
        if self.authority_snapshot.has_any_authority() {
            return Err(EpisodeContractError::UnauthorizedEpisode);
        }
        if self.action.declared_cost == 0 {
            return Err(EpisodeContractError::ZeroActionCost);
        }
        if self.outcome.is_some() != self.evaluation.is_some() {
            return Err(EpisodeContractError::IncompleteOutcomeEvaluation);
        }
        Ok(())
    }

    /// Canonical bytes are stable for identical contract values and are the
    /// replay/ledger input for EI-0B and EI-0C.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, EpisodeContractError> {
        self.validate()?;
        serde_json::to_vec(self)
            .map_err(|error| EpisodeContractError::CanonicalSerialization(error.to_string()))
    }

    pub fn digest(&self) -> Result<EpisodeDigest, EpisodeContractError> {
        let bytes = self.canonical_bytes()?;
        let mut value = fnv1a64(DIGEST_DOMAIN);
        value = mix_u64(value, bytes.len() as u64);
        for byte in bytes {
            value ^= u64::from(byte);
            value = value.wrapping_mul(0x100000001b3);
        }
        Ok(EpisodeDigest(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeDigest(pub u64);

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EpisodeContractError {
    #[error("identifiers must not be empty")]
    EmptyIdentifier,
    #[error("observation facts must be non-empty, sorted, and unique")]
    NonCanonicalFacts,
    #[error("prediction probability {0} is outside 0..=10000 basis points")]
    ProbabilityOutOfRange(u16),
    #[error("predictions require evidence references")]
    MissingPredictionEvidence,
    #[error("prediction is malformed or lacks pre-action evidence")]
    InvalidPrediction,
    #[error("a scored episode requires at least one pre-action prediction")]
    MissingPrediction,
    #[error("post-outcome predictions are not valid EI evidence")]
    PostOutcomePrediction,
    #[error("EI-0A episode attempted to claim authority")]
    UnauthorizedEpisode,
    #[error("bounded actions must declare positive cost")]
    ZeroActionCost,
    #[error("outcome and independent evaluation must either both be present or both be absent")]
    IncompleteOutcomeEvaluation,
    #[error("canonical serialization failed: {0}")]
    CanonicalSerialization(String),
}

fn canonical_identifier(value: String) -> Result<String, EpisodeContractError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(EpisodeContractError::EmptyIdentifier);
    }
    Ok(trimmed.to_owned())
}

fn validate_identifier(value: &str) -> Result<(), EpisodeContractError> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(EpisodeContractError::EmptyIdentifier);
    }
    Ok(())
}

fn validate_observation(observation: &Observation) -> Result<(), EpisodeContractError> {
    validate_identifier(&observation.kind)?;
    canonical_strings(observation.facts.clone()).map(|_| ())
}

fn validate_provenance(provenance: &EpisodeProvenance) -> Result<(), EpisodeContractError> {
    validate_identifier(&provenance.cohort_id)?;
    validate_identifier(&provenance.fixture_digest)?;
    validate_identifier(&provenance.generator_version)
}

fn canonical_strings(values: Vec<String>) -> Result<Vec<String>, EpisodeContractError> {
    if values.is_empty() {
        return Err(EpisodeContractError::NonCanonicalFacts);
    }
    let canonical: Result<Vec<_>, _> = values.into_iter().map(canonical_identifier).collect();
    let canonical = canonical?;
    if canonical.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(EpisodeContractError::NonCanonicalFacts);
    }
    Ok(canonical)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut digest = 0xcbf29ce484222325_u64;
    for byte in bytes {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest
}

fn mix_u64(mut digest: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(value: &str) -> EvidenceRef {
        EvidenceRef::new(value).unwrap()
    }

    fn episode() -> CognitiveEpisode {
        CognitiveEpisode {
            episode_id: EpisodeId::new("episode-001").unwrap(),
            task_family: TaskFamilyId::new("route-planning").unwrap(),
            phase: EpisodePhase::Development,
            observation: Observation::new("route", vec!["edge:a-b".into(), "node:a".into()])
                .unwrap(),
            retrieved_evidence: vec![evidence("memory:route-0")],
            predictions: vec![Prediction::new(
                "route remains open",
                7_500,
                vec![evidence("memory:route-0")],
            )
            .unwrap()],
            selected_strategy: StrategySelection {
                strategy_id: "bounded-search".into(),
                rationale_evidence: vec![evidence("memory:route-0")],
            },
            intention: Intention {
                objective: "reach goal".into(),
            },
            action: BoundedAction {
                action: "move:a-b".into(),
                declared_cost: 1,
            },
            outcome: Some(Outcome {
                objective_satisfied: true,
                score_bps: 10_000,
                evidence: vec![evidence("environment:step-1")],
            }),
            evaluation: Some(EpisodeEvaluation {
                prediction_correct: true,
                action_score_bps: 10_000,
                evaluator_id: "independent-fixture-v1".into(),
            }),
            authority_snapshot: AuthoritySnapshot::shadow_only(),
            provenance: EpisodeProvenance {
                cohort_id: "development-v1".into(),
                fixture_digest: "fixture:abc123".into(),
                seed: 7,
                generator_version: "v1".into(),
            },
        }
    }

    #[test]
    fn canonical_bytes_and_digest_are_replay_stable() {
        let first = episode();
        let second = episode();
        assert_eq!(
            first.canonical_bytes().unwrap(),
            second.canonical_bytes().unwrap()
        );
        assert_eq!(first.digest().unwrap(), second.digest().unwrap());
    }

    #[test]
    fn changed_evidence_changes_the_sealed_digest() {
        let first = episode();
        let mut second = episode();
        second.provenance.seed = 8;
        assert_ne!(first.digest().unwrap(), second.digest().unwrap());
    }

    #[test]
    fn authority_cannot_be_smuggled_into_an_ei_0a_episode() {
        let mut record = episode();
        record.authority_snapshot.learning_update_authority = true;
        assert_eq!(
            record.validate(),
            Err(EpisodeContractError::UnauthorizedEpisode)
        );
    }

    #[test]
    fn scored_episode_rejects_post_outcome_predictions() {
        let mut record = episode();
        record.predictions[0].created_before_action = false;
        assert_eq!(
            record.validate(),
            Err(EpisodeContractError::PostOutcomePrediction)
        );
    }

    #[test]
    fn observation_facts_must_be_canonical() {
        assert_eq!(
            Observation::new("route", vec!["node:a".into(), "edge:a-b".into()]),
            Err(EpisodeContractError::NonCanonicalFacts)
        );
    }

    #[test]
    fn deserialized_malformed_fields_cannot_be_sealed() {
        let mut record = episode();
        record.provenance.cohort_id = " ".into();
        assert_eq!(record.digest(), Err(EpisodeContractError::EmptyIdentifier));
    }
}
