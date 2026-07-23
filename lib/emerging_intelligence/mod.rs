//! EI-0A canonical cognitive episode contracts.
//!
//! This module is deliberately data-only and feature-gated. It defines sealed,
//! replayable records for one developmental episode. It has no persistence,
//! `Runtime::chat()`, response-generation, routing, tool, ontology, or learning-
//! update application authority.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

pub const EI_0A_SCHEMA_VERSION: u16 = 1;
const DIGEST_DOMAIN: &[u8] = b"starfire/ei-0a/cognitive-episode/v1";
const MAX_BASIS_POINTS: u16 = 10_000;
const DIGEST_HEX_LEN: usize = 32;

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, EpisodeContractError> {
                Ok(Self(canonical_identifier(value.into())?))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

string_id!(EpisodeId);
string_id!(ObservationId);
string_id!(EvidenceId);
string_id!(PredictionId);
string_id!(StrategyId);
string_id!(ActionId);
string_id!(OutcomeId);
string_id!(EvaluationId);
string_id!(LearningUpdateId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodePhase {
    Observed,
    Predicted,
    Acted,
    OutcomeObserved,
    Evaluated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationPartition {
    Development,
    WithinFamilyHoldout,
    RenamedVocabularyTransfer,
    StructuralTransfer,
    Regression,
    Adversarial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub observation_id: ObservationId,
    pub kind: String,
    /// Sorted, unique environment facts.
    pub facts: Vec<String>,
    pub observed_at_step: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub evidence_id: EvidenceId,
    pub kind: String,
    pub content_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub evidence_id: EvidenceId,
}

impl EvidenceRef {
    pub fn new(evidence_id: EvidenceId) -> Self {
        Self { evidence_id }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prediction {
    pub prediction_id: PredictionId,
    pub proposition: String,
    pub probability_bps: u16,
    /// Sorted, unique evidence references.
    pub evidence_refs: Vec<EvidenceRef>,
    pub created_at_step: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategySelection {
    pub strategy_id: StrategyId,
    /// Sorted, unique evidence references.
    pub rationale_evidence: Vec<EvidenceRef>,
    pub selected_at_step: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intention {
    pub objective: String,
    pub declared_at_step: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedAction {
    pub action_id: ActionId,
    pub action: String,
    pub declared_cost: u64,
    pub performed_at_step: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub outcome_id: OutcomeId,
    pub action_id: ActionId,
    pub objective_satisfied: bool,
    pub score_bps: u16,
    /// Sorted, unique evidence references.
    pub evidence_refs: Vec<EvidenceRef>,
    pub observed_at_step: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PredictionAssessment {
    pub prediction_id: PredictionId,
    pub score_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeEvaluation {
    pub evaluation_id: EvaluationId,
    pub outcome_id: OutcomeId,
    /// Sorted by prediction identifier with no duplicates.
    pub prediction_scores: Vec<PredictionAssessment>,
    pub action_score_bps: u16,
    pub evaluator_id: String,
    pub evaluated_at_step: u32,
}

/// A proposal reference only. EI-0A cannot apply or execute an update.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningUpdate {
    pub update_id: LearningUpdateId,
    pub evaluation_id: EvaluationId,
    pub proposal_digest: String,
    pub proposed_at_step: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuthoritySnapshot {
    pub runtime_chat_authority: bool,
    pub persistence_authority: bool,
    pub response_generation_authority: bool,
    pub routing_authority: bool,
    pub learning_update_authority: bool,
    pub tool_authority: bool,
    pub ontology_promotion_authority: bool,
    pub autonomous_action_authority: bool,
}

impl AuthoritySnapshot {
    pub fn closed() -> Self {
        Self::default()
    }

    pub fn is_closed(&self) -> bool {
        !self.runtime_chat_authority
            && !self.persistence_authority
            && !self.response_generation_authority
            && !self.routing_authority
            && !self.learning_update_authority
            && !self.tool_authority
            && !self.ontology_promotion_authority
            && !self.autonomous_action_authority
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeProvenance {
    pub cohort_id: String,
    pub fixture_digest: String,
    pub seed: u64,
    pub generator_version: String,
    /// Sorted, unique source hashes used to construct the episode.
    pub source_hashes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CognitiveEpisode {
    pub episode_id: EpisodeId,
    pub phase: EpisodePhase,
    pub partition: EvaluationPartition,
    pub task_family: String,
    pub observation: Observation,
    /// Sorted by evidence identifier with no duplicates.
    pub evidence: Vec<EvidenceRecord>,
    /// Sorted by prediction identifier with no duplicates.
    pub predictions: Vec<Prediction>,
    pub selected_strategy: Option<StrategySelection>,
    pub intention: Option<Intention>,
    pub action: Option<BoundedAction>,
    pub outcome: Option<Outcome>,
    pub evaluation: Option<EpisodeEvaluation>,
    /// Sorted by update identifier with no duplicates.
    pub proposed_updates: Vec<LearningUpdate>,
    /// Sorted, unique references into `proposed_updates`.
    pub accepted_updates: Vec<LearningUpdateId>,
    pub authority: AuthoritySnapshot,
    pub provenance: EpisodeProvenance,
}

impl CognitiveEpisode {
    pub fn validate(&self) -> Result<(), EpisodeContractError> {
        validate_identifier(self.episode_id.as_str())?;
        validate_identifier(&self.task_family)?;
        validate_observation(&self.observation)?;
        validate_provenance(&self.provenance)?;

        if !self.authority.is_closed() {
            return Err(EpisodeContractError::UnauthorizedEpisode);
        }

        validate_sorted_unique_by(
            &self.evidence,
            |record| record.evidence_id.as_str(),
            "evidence",
        )?;
        validate_sorted_unique_by(
            &self.predictions,
            |prediction| prediction.prediction_id.as_str(),
            "predictions",
        )?;
        validate_sorted_unique_by(
            &self.proposed_updates,
            |update| update.update_id.as_str(),
            "proposed_updates",
        )?;
        validate_sorted_unique_by(
            &self.accepted_updates,
            LearningUpdateId::as_str,
            "accepted_updates",
        )?;

        let evidence_ids: BTreeSet<&str> = self
            .evidence
            .iter()
            .map(|record| record.evidence_id.as_str())
            .collect();
        let prediction_ids: BTreeSet<&str> = self
            .predictions
            .iter()
            .map(|prediction| prediction.prediction_id.as_str())
            .collect();
        let update_ids: BTreeSet<&str> = self
            .proposed_updates
            .iter()
            .map(|update| update.update_id.as_str())
            .collect();

        for record in &self.evidence {
            validate_identifier(record.evidence_id.as_str())?;
            validate_identifier(&record.kind)?;
            validate_digest_text(&record.content_digest, "evidence content digest")?;
        }

        for prediction in &self.predictions {
            validate_identifier(prediction.prediction_id.as_str())?;
            if prediction.evidence_refs.is_empty() {
                return Err(EpisodeContractError::MissingEvidenceReference("prediction"));
            }
            validate_identifier(&prediction.proposition)?;
            validate_basis_points(prediction.probability_bps, "prediction probability")?;
            validate_evidence_refs(&prediction.evidence_refs, &evidence_ids)?;
            if prediction.created_at_step <= self.observation.observed_at_step {
                return Err(EpisodeContractError::InvalidTimeline(
                    "prediction must follow observation",
                ));
            }
        }

        if let Some(strategy) = &self.selected_strategy {
            validate_identifier(strategy.strategy_id.as_str())?;
            if strategy.rationale_evidence.is_empty() {
                return Err(EpisodeContractError::MissingEvidenceReference("strategy"));
            }
            validate_evidence_refs(&strategy.rationale_evidence, &evidence_ids)?;
            if strategy.selected_at_step <= self.observation.observed_at_step {
                return Err(EpisodeContractError::InvalidTimeline(
                    "strategy selection must follow observation",
                ));
            }
        }

        if let Some(intention) = &self.intention {
            validate_identifier(&intention.objective)?;
            if intention.declared_at_step <= self.observation.observed_at_step {
                return Err(EpisodeContractError::InvalidTimeline(
                    "intention must follow observation",
                ));
            }
        }

        if let Some(action) = &self.action {
            validate_identifier(action.action_id.as_str())?;
            validate_identifier(&action.action)?;
            if action.declared_cost == 0 {
                return Err(EpisodeContractError::ZeroActionCost);
            }
            if action.performed_at_step <= self.observation.observed_at_step {
                return Err(EpisodeContractError::InvalidTimeline(
                    "action must follow observation",
                ));
            }
            if self.predictions.is_empty() {
                return Err(EpisodeContractError::MissingPreActionPrediction);
            }
            if self
                .predictions
                .iter()
                .any(|prediction| prediction.created_at_step >= action.performed_at_step)
            {
                return Err(EpisodeContractError::PostActionPrediction);
            }
            let strategy = self
                .selected_strategy
                .as_ref()
                .ok_or(EpisodeContractError::MissingPreActionStrategy)?;
            if strategy.selected_at_step >= action.performed_at_step {
                return Err(EpisodeContractError::MissingPreActionStrategy);
            }
            let intention = self
                .intention
                .as_ref()
                .ok_or(EpisodeContractError::MissingPreActionIntention)?;
            if intention.declared_at_step >= action.performed_at_step {
                return Err(EpisodeContractError::MissingPreActionIntention);
            }
        }

        if let Some(outcome) = &self.outcome {
            validate_identifier(outcome.outcome_id.as_str())?;
            if outcome.evidence_refs.is_empty() {
                return Err(EpisodeContractError::MissingEvidenceReference("outcome"));
            }
            validate_basis_points(outcome.score_bps, "outcome score")?;
            validate_evidence_refs(&outcome.evidence_refs, &evidence_ids)?;
            let action = self
                .action
                .as_ref()
                .ok_or(EpisodeContractError::OutcomeBeforeAction)?;
            if outcome.action_id != action.action_id {
                return Err(EpisodeContractError::DanglingReference("outcome action"));
            }
            if outcome.observed_at_step <= action.performed_at_step {
                return Err(EpisodeContractError::InvalidTimeline(
                    "outcome must follow action",
                ));
            }
        }

        if let Some(evaluation) = &self.evaluation {
            validate_identifier(evaluation.evaluation_id.as_str())?;
            validate_identifier(&evaluation.evaluator_id)?;
            validate_basis_points(evaluation.action_score_bps, "action score")?;
            validate_sorted_unique_by(
                &evaluation.prediction_scores,
                |score| score.prediction_id.as_str(),
                "prediction_scores",
            )?;
            let outcome = self
                .outcome
                .as_ref()
                .ok_or(EpisodeContractError::EvaluationBeforeOutcome)?;
            if evaluation.outcome_id != outcome.outcome_id {
                return Err(EpisodeContractError::DanglingReference(
                    "evaluation outcome",
                ));
            }
            if evaluation.evaluated_at_step <= outcome.observed_at_step {
                return Err(EpisodeContractError::InvalidTimeline(
                    "evaluation must follow outcome",
                ));
            }
            if self.predictions.is_empty() {
                return Err(EpisodeContractError::MissingPreActionPrediction);
            }
            for score in &evaluation.prediction_scores {
                validate_basis_points(score.score_bps, "prediction score")?;
                if !prediction_ids.contains(score.prediction_id.as_str()) {
                    return Err(EpisodeContractError::DanglingReference(
                        "prediction assessment",
                    ));
                }
            }
            if evaluation.prediction_scores.len() != self.predictions.len() {
                return Err(EpisodeContractError::IncompletePredictionAssessment);
            }
        }

        for update in &self.proposed_updates {
            validate_identifier(update.update_id.as_str())?;
            validate_digest_text(&update.proposal_digest, "learning proposal digest")?;
            let evaluation = self
                .evaluation
                .as_ref()
                .ok_or(EpisodeContractError::UpdateBeforeEvaluation)?;
            if update.evaluation_id != evaluation.evaluation_id {
                return Err(EpisodeContractError::DanglingReference(
                    "learning update evaluation",
                ));
            }
            if update.proposed_at_step <= evaluation.evaluated_at_step {
                return Err(EpisodeContractError::InvalidTimeline(
                    "learning proposal must follow evaluation",
                ));
            }
        }

        if !self.accepted_updates.is_empty() {
            if self.outcome.is_none() || self.evaluation.is_none() {
                return Err(EpisodeContractError::AcceptedUpdateBeforeEvaluation);
            }
            for update_id in &self.accepted_updates {
                if !update_ids.contains(update_id.as_str()) {
                    return Err(EpisodeContractError::DanglingReference(
                        "accepted learning update",
                    ));
                }
            }
        }

        validate_global_identifier_uniqueness(self)?;
        validate_phase_shape(self)?;
        Ok(())
    }

    pub fn canonical_payload_bytes(&self) -> Result<Vec<u8>, EpisodeContractError> {
        self.validate()?;
        serde_json::to_vec(self)
            .map_err(|error| EpisodeContractError::Serialization(error.to_string()))
    }

    pub fn digest(&self) -> Result<EpisodeDigest, EpisodeContractError> {
        let bytes = self.canonical_payload_bytes()?;
        Ok(EpisodeDigest(checksum128(&bytes)))
    }

    pub fn seal(self) -> Result<SealedCognitiveEpisode, EpisodeContractError> {
        SealedCognitiveEpisode::new(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EpisodeDigest(String);

impl EpisodeDigest {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedCognitiveEpisode {
    pub schema_version: u16,
    pub episode: CognitiveEpisode,
    pub digest: EpisodeDigest,
}

impl SealedCognitiveEpisode {
    pub fn new(episode: CognitiveEpisode) -> Result<Self, EpisodeContractError> {
        let digest = episode.digest()?;
        Ok(Self {
            schema_version: EI_0A_SCHEMA_VERSION,
            episode,
            digest,
        })
    }

    pub fn validate(&self) -> Result<(), EpisodeContractError> {
        if self.schema_version != EI_0A_SCHEMA_VERSION {
            return Err(EpisodeContractError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        self.episode.validate()?;
        validate_episode_digest(&self.digest)?;
        let expected = self.episode.digest()?;
        if self.digest != expected {
            return Err(EpisodeContractError::DigestMismatch);
        }
        Ok(())
    }

    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, EpisodeContractError> {
        self.validate()?;
        serde_json::to_vec(self)
            .map_err(|error| EpisodeContractError::Serialization(error.to_string()))
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, EpisodeContractError> {
        let sealed: Self = serde_json::from_slice(bytes)
            .map_err(|error| EpisodeContractError::Deserialization(error.to_string()))?;
        sealed.validate()?;
        let canonical = serde_json::to_vec(&sealed)
            .map_err(|error| EpisodeContractError::Serialization(error.to_string()))?;
        if canonical != bytes {
            return Err(EpisodeContractError::NonCanonicalEncoding);
        }
        Ok(sealed)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EpisodeContractError {
    #[error("identifier or required text must not be empty or padded")]
    InvalidIdentifier,
    #[error("{0} must be sorted and unique")]
    NonCanonicalCollection(&'static str),
    #[error("{0} is outside 0..=10000 basis points")]
    BasisPointsOutOfRange(&'static str),
    #[error("invalid digest text for {0}")]
    InvalidDigestText(&'static str),
    #[error("{0} requires at least one evidence reference")]
    MissingEvidenceReference(&'static str),
    #[error("evidence reference is not present in the episode evidence set")]
    DanglingEvidenceReference,
    #[error("dangling reference: {0}")]
    DanglingReference(&'static str),
    #[error("duplicate identifier across episode records: {0}")]
    DuplicateIdentifier(String),
    #[error("EI-0A episode attempted to claim runtime authority")]
    UnauthorizedEpisode,
    #[error("bounded action cost must be positive")]
    ZeroActionCost,
    #[error("an acted or scored episode requires a prediction created before action")]
    MissingPreActionPrediction,
    #[error("prediction was created at or after the action")]
    PostActionPrediction,
    #[error("an action requires a strategy selected before action")]
    MissingPreActionStrategy,
    #[error("an action requires an intention declared before action")]
    MissingPreActionIntention,
    #[error("outcome cannot exist before an action")]
    OutcomeBeforeAction,
    #[error("evaluation cannot exist before an outcome")]
    EvaluationBeforeOutcome,
    #[error("learning update proposal cannot exist before evaluation")]
    UpdateBeforeEvaluation,
    #[error("accepted update references require an evaluated outcome")]
    AcceptedUpdateBeforeEvaluation,
    #[error("evaluation must assess every episode prediction exactly once")]
    IncompletePredictionAssessment,
    #[error("invalid episode timeline: {0}")]
    InvalidTimeline(&'static str),
    #[error("episode phase does not match populated fields: {0}")]
    PhaseShape(&'static str),
    #[error("unsupported EI-0A schema version {0}")]
    UnsupportedSchemaVersion(u16),
    #[error("sealed episode digest does not match canonical payload")]
    DigestMismatch,
    #[error("sealed episode is valid JSON but not canonical byte encoding")]
    NonCanonicalEncoding,
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("deserialization failed: {0}")]
    Deserialization(String),
}

fn validate_observation(observation: &Observation) -> Result<(), EpisodeContractError> {
    validate_identifier(observation.observation_id.as_str())?;
    validate_identifier(&observation.kind)?;
    validate_canonical_strings(&observation.facts, "observation facts")
}

fn validate_provenance(provenance: &EpisodeProvenance) -> Result<(), EpisodeContractError> {
    validate_identifier(&provenance.cohort_id)?;
    validate_digest_text(&provenance.fixture_digest, "fixture digest")?;
    validate_identifier(&provenance.generator_version)?;
    validate_canonical_strings(&provenance.source_hashes, "source hashes")
}

fn validate_phase_shape(episode: &CognitiveEpisode) -> Result<(), EpisodeContractError> {
    let has_plan = episode.selected_strategy.is_some() && episode.intention.is_some();
    match episode.phase {
        EpisodePhase::Observed => {
            if !episode.predictions.is_empty()
                || episode.selected_strategy.is_some()
                || episode.intention.is_some()
                || episode.action.is_some()
                || episode.outcome.is_some()
                || episode.evaluation.is_some()
                || !episode.proposed_updates.is_empty()
                || !episode.accepted_updates.is_empty()
            {
                return Err(EpisodeContractError::PhaseShape("observed"));
            }
        }
        EpisodePhase::Predicted => {
            if episode.predictions.is_empty()
                || !has_plan
                || episode.action.is_some()
                || episode.outcome.is_some()
                || episode.evaluation.is_some()
                || !episode.proposed_updates.is_empty()
                || !episode.accepted_updates.is_empty()
            {
                return Err(EpisodeContractError::PhaseShape("predicted"));
            }
        }
        EpisodePhase::Acted => {
            if episode.predictions.is_empty()
                || !has_plan
                || episode.action.is_none()
                || episode.outcome.is_some()
                || episode.evaluation.is_some()
                || !episode.proposed_updates.is_empty()
                || !episode.accepted_updates.is_empty()
            {
                return Err(EpisodeContractError::PhaseShape("acted"));
            }
        }
        EpisodePhase::OutcomeObserved => {
            if episode.predictions.is_empty()
                || !has_plan
                || episode.action.is_none()
                || episode.outcome.is_none()
                || episode.evaluation.is_some()
                || !episode.proposed_updates.is_empty()
                || !episode.accepted_updates.is_empty()
            {
                return Err(EpisodeContractError::PhaseShape("outcome_observed"));
            }
        }
        EpisodePhase::Evaluated => {
            if episode.predictions.is_empty()
                || !has_plan
                || episode.action.is_none()
                || episode.outcome.is_none()
                || episode.evaluation.is_none()
            {
                return Err(EpisodeContractError::PhaseShape("evaluated"));
            }
        }
    }
    Ok(())
}

fn validate_global_identifier_uniqueness(
    episode: &CognitiveEpisode,
) -> Result<(), EpisodeContractError> {
    let mut identifiers = BTreeSet::new();
    insert_unique(&mut identifiers, episode.episode_id.as_str())?;
    insert_unique(
        &mut identifiers,
        episode.observation.observation_id.as_str(),
    )?;
    for record in &episode.evidence {
        insert_unique(&mut identifiers, record.evidence_id.as_str())?;
    }
    for prediction in &episode.predictions {
        insert_unique(&mut identifiers, prediction.prediction_id.as_str())?;
    }
    if let Some(strategy) = &episode.selected_strategy {
        insert_unique(&mut identifiers, strategy.strategy_id.as_str())?;
    }
    if let Some(action) = &episode.action {
        insert_unique(&mut identifiers, action.action_id.as_str())?;
    }
    if let Some(outcome) = &episode.outcome {
        insert_unique(&mut identifiers, outcome.outcome_id.as_str())?;
    }
    if let Some(evaluation) = &episode.evaluation {
        insert_unique(&mut identifiers, evaluation.evaluation_id.as_str())?;
    }
    for update in &episode.proposed_updates {
        insert_unique(&mut identifiers, update.update_id.as_str())?;
    }
    Ok(())
}

fn insert_unique(
    identifiers: &mut BTreeSet<String>,
    value: &str,
) -> Result<(), EpisodeContractError> {
    if !identifiers.insert(value.to_owned()) {
        return Err(EpisodeContractError::DuplicateIdentifier(value.to_owned()));
    }
    Ok(())
}

fn validate_evidence_refs(
    refs: &[EvidenceRef],
    evidence_ids: &BTreeSet<&str>,
) -> Result<(), EpisodeContractError> {
    validate_sorted_unique_by(
        refs,
        |evidence_ref| evidence_ref.evidence_id.as_str(),
        "evidence references",
    )?;
    if refs
        .iter()
        .any(|evidence_ref| !evidence_ids.contains(evidence_ref.evidence_id.as_str()))
    {
        return Err(EpisodeContractError::DanglingEvidenceReference);
    }
    Ok(())
}

fn validate_sorted_unique_by<T, F>(
    values: &[T],
    key: F,
    field: &'static str,
) -> Result<(), EpisodeContractError>
where
    F: for<'a> Fn(&'a T) -> &'a str,
{
    if values.windows(2).any(|pair| key(&pair[0]) >= key(&pair[1])) {
        return Err(EpisodeContractError::NonCanonicalCollection(field));
    }
    Ok(())
}

fn validate_canonical_strings(
    values: &[String],
    field: &'static str,
) -> Result<(), EpisodeContractError> {
    if values.is_empty()
        || values
            .iter()
            .any(|value| validate_identifier(value).is_err())
        || values
            .windows(2)
            .any(|pair| pair[0].as_str() >= pair[1].as_str())
    {
        return Err(EpisodeContractError::NonCanonicalCollection(field));
    }
    Ok(())
}

fn canonical_identifier(value: String) -> Result<String, EpisodeContractError> {
    validate_identifier(&value)?;
    Ok(value)
}

fn validate_identifier(value: &str) -> Result<(), EpisodeContractError> {
    if value.is_empty() || value.trim() != value {
        return Err(EpisodeContractError::InvalidIdentifier);
    }
    Ok(())
}

fn validate_basis_points(value: u16, field: &'static str) -> Result<(), EpisodeContractError> {
    if value > MAX_BASIS_POINTS {
        return Err(EpisodeContractError::BasisPointsOutOfRange(field));
    }
    Ok(())
}

fn validate_digest_text(value: &str, field: &'static str) -> Result<(), EpisodeContractError> {
    if value.len() < 8
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b':' || byte == b'-'
        })
    {
        return Err(EpisodeContractError::InvalidDigestText(field));
    }
    Ok(())
}

fn validate_episode_digest(digest: &EpisodeDigest) -> Result<(), EpisodeContractError> {
    if digest.0.len() != DIGEST_HEX_LEN
        || !digest
            .0
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(EpisodeContractError::InvalidDigestText("episode digest"));
    }
    Ok(())
}

fn checksum128(payload: &[u8]) -> String {
    let left = fnv1a64(0xcbf29ce484222325, 0x4c, payload);
    let right = fnv1a64(0x84222325cbf29ce4, 0x52, payload);
    format!("{left:016x}{right:016x}")
}

fn fnv1a64(seed: u64, lane: u8, payload: &[u8]) -> u64 {
    let mut digest = seed;
    for byte in DIGEST_DOMAIN
        .iter()
        .copied()
        .chain([lane])
        .chain((payload.len() as u64).to_le_bytes())
        .chain(payload.iter().copied())
    {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence_id(value: &str) -> EvidenceId {
        EvidenceId::new(value).unwrap()
    }

    fn evidence_ref(value: &str) -> EvidenceRef {
        EvidenceRef::new(evidence_id(value))
    }

    fn evaluated_episode() -> CognitiveEpisode {
        CognitiveEpisode {
            episode_id: EpisodeId::new("episode-001").unwrap(),
            phase: EpisodePhase::Evaluated,
            partition: EvaluationPartition::Development,
            task_family: "route-planning".into(),
            observation: Observation {
                observation_id: ObservationId::new("observation-001").unwrap(),
                kind: "route-state".into(),
                facts: vec!["edge:a-b:open".into(), "node:a:active".into()],
                observed_at_step: 1,
            },
            evidence: vec![
                EvidenceRecord {
                    evidence_id: evidence_id("evidence-environment-001"),
                    kind: "environment".into(),
                    content_digest: "fixture:environment001".into(),
                },
                EvidenceRecord {
                    evidence_id: evidence_id("evidence-memory-001"),
                    kind: "memory".into(),
                    content_digest: "fixture:memory0001".into(),
                },
            ],
            predictions: vec![Prediction {
                prediction_id: PredictionId::new("prediction-001").unwrap(),
                proposition: "route remains open".into(),
                probability_bps: 7_500,
                evidence_refs: vec![evidence_ref("evidence-memory-001")],
                created_at_step: 2,
            }],
            selected_strategy: Some(StrategySelection {
                strategy_id: StrategyId::new("strategy-bounded-search").unwrap(),
                rationale_evidence: vec![evidence_ref("evidence-memory-001")],
                selected_at_step: 3,
            }),
            intention: Some(Intention {
                objective: "reach goal".into(),
                declared_at_step: 3,
            }),
            action: Some(BoundedAction {
                action_id: ActionId::new("action-001").unwrap(),
                action: "move:a-b".into(),
                declared_cost: 1,
                performed_at_step: 4,
            }),
            outcome: Some(Outcome {
                outcome_id: OutcomeId::new("outcome-001").unwrap(),
                action_id: ActionId::new("action-001").unwrap(),
                objective_satisfied: true,
                score_bps: 10_000,
                evidence_refs: vec![evidence_ref("evidence-environment-001")],
                observed_at_step: 5,
            }),
            evaluation: Some(EpisodeEvaluation {
                evaluation_id: EvaluationId::new("evaluation-001").unwrap(),
                outcome_id: OutcomeId::new("outcome-001").unwrap(),
                prediction_scores: vec![PredictionAssessment {
                    prediction_id: PredictionId::new("prediction-001").unwrap(),
                    score_bps: 9_000,
                }],
                action_score_bps: 10_000,
                evaluator_id: "independent-fixture-v1".into(),
                evaluated_at_step: 6,
            }),
            proposed_updates: vec![LearningUpdate {
                update_id: LearningUpdateId::new("update-001").unwrap(),
                evaluation_id: EvaluationId::new("evaluation-001").unwrap(),
                proposal_digest: "proposal:00000001".into(),
                proposed_at_step: 7,
            }],
            accepted_updates: vec![LearningUpdateId::new("update-001").unwrap()],
            authority: AuthoritySnapshot::closed(),
            provenance: EpisodeProvenance {
                cohort_id: "development-v1".into(),
                fixture_digest: "fixture:abc12345".into(),
                seed: 7,
                generator_version: "ei-0a-fixture-v1".into(),
                source_hashes: vec!["source:00000001".into(), "source:00000002".into()],
            },
        }
    }

    #[test]
    fn canonical_seal_replays_exactly() {
        let sealed = evaluated_episode().seal().unwrap();
        let bytes = sealed.to_canonical_bytes().unwrap();
        let replay = SealedCognitiveEpisode::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(replay, sealed);
        assert_eq!(replay.to_canonical_bytes().unwrap(), bytes);
    }

    #[test]
    fn digest_changes_when_payload_changes() {
        let first = evaluated_episode().digest().unwrap();
        let mut changed = evaluated_episode();
        changed.provenance.seed = 8;
        assert_ne!(first, changed.digest().unwrap());
    }

    #[test]
    fn probability_outside_basis_points_fails_closed() {
        let mut episode = evaluated_episode();
        episode.predictions[0].probability_bps = 10_001;
        assert_eq!(
            episode.validate(),
            Err(EpisodeContractError::BasisPointsOutOfRange(
                "prediction probability"
            ))
        );
    }

    #[test]
    fn prediction_created_after_action_fails_closed() {
        let mut episode = evaluated_episode();
        episode.predictions[0].created_at_step = 4;
        assert_eq!(
            episode.validate(),
            Err(EpisodeContractError::PostActionPrediction)
        );
    }

    #[test]
    fn pre_action_phase_rejects_outcome_and_evaluation() {
        let mut episode = evaluated_episode();
        episode.phase = EpisodePhase::Predicted;
        episode.action = None;
        assert_eq!(
            episode.validate(),
            Err(EpisodeContractError::OutcomeBeforeAction)
        );
    }

    #[test]
    fn accepted_update_requires_evaluated_outcome() {
        let mut episode = evaluated_episode();
        episode.phase = EpisodePhase::OutcomeObserved;
        episode.evaluation = None;
        episode.proposed_updates.clear();
        assert_eq!(
            episode.validate(),
            Err(EpisodeContractError::AcceptedUpdateBeforeEvaluation)
        );
    }

    #[test]
    fn duplicate_identifier_across_record_types_fails_closed() {
        let mut episode = evaluated_episode();
        episode.action.as_mut().unwrap().action_id = ActionId::new("prediction-001").unwrap();
        episode.outcome.as_mut().unwrap().action_id = ActionId::new("prediction-001").unwrap();
        assert_eq!(
            episode.validate(),
            Err(EpisodeContractError::DuplicateIdentifier(
                "prediction-001".into()
            ))
        );
    }

    #[test]
    fn dangling_evidence_reference_fails_closed() {
        let mut episode = evaluated_episode();
        episode.predictions[0].evidence_refs = vec![evidence_ref("evidence-missing")];
        assert_eq!(
            episode.validate(),
            Err(EpisodeContractError::DanglingEvidenceReference)
        );
    }

    #[test]
    fn authority_defaults_closed_and_cannot_be_smuggled() {
        assert!(AuthoritySnapshot::default().is_closed());
        let mut episode = evaluated_episode();
        episode.authority.runtime_chat_authority = true;
        assert_eq!(
            episode.validate(),
            Err(EpisodeContractError::UnauthorizedEpisode)
        );
    }

    #[test]
    fn stale_schema_version_fails_closed() {
        let mut sealed = evaluated_episode().seal().unwrap();
        sealed.schema_version = EI_0A_SCHEMA_VERSION + 1;
        assert_eq!(
            sealed.validate(),
            Err(EpisodeContractError::UnsupportedSchemaVersion(2))
        );
    }

    #[test]
    fn tampered_digest_fails_closed() {
        let sealed = evaluated_episode().seal().unwrap();
        let bytes = sealed.to_canonical_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();
        let tampered = text.replace(sealed.digest.as_str(), &"0".repeat(DIGEST_HEX_LEN));
        assert_eq!(
            SealedCognitiveEpisode::from_canonical_bytes(tampered.as_bytes()),
            Err(EpisodeContractError::DigestMismatch)
        );
    }

    #[test]
    fn reordered_json_fields_fail_closed() {
        let sealed = evaluated_episode().seal().unwrap();
        let episode_json = serde_json::to_string(&sealed.episode).unwrap();
        let reordered = format!(
            "{{\"episode\":{episode_json},\"schema_version\":{},\"digest\":\"{}\"}}",
            sealed.schema_version,
            sealed.digest.as_str()
        );
        assert_eq!(
            SealedCognitiveEpisode::from_canonical_bytes(reordered.as_bytes()),
            Err(EpisodeContractError::NonCanonicalEncoding)
        );
    }

    #[test]
    fn reordered_collection_fails_closed() {
        let mut episode = evaluated_episode();
        episode.evidence.swap(0, 1);
        assert_eq!(
            episode.validate(),
            Err(EpisodeContractError::NonCanonicalCollection("evidence"))
        );
    }

    #[test]
    fn malformed_identifier_fails_closed_after_deserialization() {
        let sealed = evaluated_episode().seal().unwrap();
        let bytes = sealed.to_canonical_bytes().unwrap();
        let text = String::from_utf8(bytes).unwrap();
        let malformed = text.replacen("episode-001", " episode-001", 1);
        assert_eq!(
            SealedCognitiveEpisode::from_canonical_bytes(malformed.as_bytes()),
            Err(EpisodeContractError::InvalidIdentifier)
        );
    }
}
