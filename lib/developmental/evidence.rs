//! Typed, provenance-carrying evidence emitted by an external developmental substrate.
//!
//! These types deliberately carry no authority to mutate Starfire beliefs, ontology,
//! routing, or runtime behavior. They are an evidence boundary only.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearnedModality {
    Vision,
    Audio,
    Proprioception,
    Text,
    Multimodal,
    Synthetic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearnedObject {
    pub object_id: String,
    pub label: Option<String>,
    pub features: Vec<f32>,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictedTransition {
    pub action: String,
    pub predicted_state: String,
    pub probability: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptProposal {
    pub candidate_id: String,
    pub descriptor: String,
    pub supporting_observation_ids: Vec<String>,
    pub counterexample_observation_ids: Vec<String>,
    pub stability: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum LearnedPayload {
    ObjectSet(Vec<LearnedObject>),
    StateEmbedding(Vec<f32>),
    PredictedTransition(PredictedTransition),
    ConceptProposal(ConceptProposal),
    AnomalyScore(f64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub producer: String,
    pub model_id: String,
    pub model_version: String,
    pub checkpoint_digest: String,
    pub source_episode_id: String,
    pub transformation_trace: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearnedEvidence {
    pub schema_version: u16,
    pub source_model: String,
    pub source_version: String,
    pub observation_id: String,
    pub modality: LearnedModality,
    pub payload: LearnedPayload,
    pub confidence: f64,
    pub uncertainty: f64,
    pub provenance: Provenance,
    pub timestamp: i64,
}

impl LearnedEvidence {
    pub fn validate(&self, policy: &EvidenceValidationPolicy) -> Result<(), EvidenceValidationError> {
        if self.schema_version != policy.accepted_schema_version {
            return Err(EvidenceValidationError::UnsupportedSchema {
                expected: policy.accepted_schema_version,
                actual: self.schema_version,
            });
        }

        require_non_empty("source_model", &self.source_model)?;
        require_non_empty("source_version", &self.source_version)?;
        require_non_empty("observation_id", &self.observation_id)?;
        require_non_empty("provenance.producer", &self.provenance.producer)?;
        require_non_empty("provenance.model_id", &self.provenance.model_id)?;
        require_non_empty("provenance.model_version", &self.provenance.model_version)?;
        require_non_empty("provenance.checkpoint_digest", &self.provenance.checkpoint_digest)?;
        require_non_empty("provenance.source_episode_id", &self.provenance.source_episode_id)?;

        require_unit_interval("confidence", self.confidence)?;
        require_unit_interval("uncertainty", self.uncertainty)?;

        if self.source_model != self.provenance.model_id {
            return Err(EvidenceValidationError::ProvenanceMismatch {
                field: "model_id",
                declared: self.source_model.clone(),
                provenance: self.provenance.model_id.clone(),
            });
        }
        if self.source_version != self.provenance.model_version {
            return Err(EvidenceValidationError::ProvenanceMismatch {
                field: "model_version",
                declared: self.source_version.clone(),
                provenance: self.provenance.model_version.clone(),
            });
        }

        validate_payload(&self.payload)?;

        if self.timestamp <= 0 {
            return Err(EvidenceValidationError::InvalidTimestamp(self.timestamp));
        }
        if self.timestamp > policy.now_timestamp + policy.max_future_skew_secs {
            return Err(EvidenceValidationError::FutureEvidence {
                timestamp: self.timestamp,
                now: policy.now_timestamp,
                max_future_skew_secs: policy.max_future_skew_secs,
            });
        }
        if policy.max_age_secs >= 0 {
            let age = policy.now_timestamp.saturating_sub(self.timestamp);
            if age > policy.max_age_secs {
                return Err(EvidenceValidationError::StaleEvidence {
                    age_secs: age,
                    max_age_secs: policy.max_age_secs,
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvidenceValidationPolicy {
    pub accepted_schema_version: u16,
    pub now_timestamp: i64,
    pub max_age_secs: i64,
    pub max_future_skew_secs: i64,
}

impl EvidenceValidationPolicy {
    pub fn strict(now_timestamp: i64, max_age_secs: i64) -> Self {
        Self {
            accepted_schema_version: DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION,
            now_timestamp,
            max_age_secs,
            max_future_skew_secs: 60,
        }
    }

    pub fn replay(now_timestamp: i64) -> Self {
        Self {
            accepted_schema_version: DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION,
            now_timestamp,
            max_age_secs: -1,
            max_future_skew_secs: 60,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum EvidenceValidationError {
    #[error("unsupported developmental evidence schema: expected {expected}, got {actual}")]
    UnsupportedSchema { expected: u16, actual: u16 },
    #[error("required field is empty: {0}")]
    EmptyField(&'static str),
    #[error("{field} must be finite and in [0, 1], got {value}")]
    InvalidUnitInterval { field: &'static str, value: f64 },
    #[error("provenance mismatch for {field}: declared {declared:?}, provenance {provenance:?}")]
    ProvenanceMismatch {
        field: &'static str,
        declared: String,
        provenance: String,
    },
    #[error("invalid evidence timestamp: {0}")]
    InvalidTimestamp(i64),
    #[error("evidence is stale by {age_secs}s; maximum accepted age is {max_age_secs}s")]
    StaleEvidence { age_secs: i64, max_age_secs: i64 },
    #[error("evidence timestamp {timestamp} is too far in the future relative to {now} (max skew {max_future_skew_secs}s)")]
    FutureEvidence {
        timestamp: i64,
        now: i64,
        max_future_skew_secs: i64,
    },
    #[error("payload validation failed: {0}")]
    InvalidPayload(String),
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), EvidenceValidationError> {
    if value.trim().is_empty() {
        return Err(EvidenceValidationError::EmptyField(field));
    }
    Ok(())
}

fn require_unit_interval(field: &'static str, value: f64) -> Result<(), EvidenceValidationError> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(EvidenceValidationError::InvalidUnitInterval { field, value });
    }
    Ok(())
}

fn validate_payload(payload: &LearnedPayload) -> Result<(), EvidenceValidationError> {
    match payload {
        LearnedPayload::ObjectSet(objects) => {
            for object in objects {
                require_non_empty("payload.object_id", &object.object_id)?;
                require_unit_interval("payload.object.confidence", object.confidence)?;
                if object.features.iter().any(|value| !value.is_finite()) {
                    return Err(EvidenceValidationError::InvalidPayload("object features must be finite".to_string()));
                }
            }
        }
        LearnedPayload::StateEmbedding(values) => {
            if values.is_empty() {
                return Err(EvidenceValidationError::InvalidPayload("state embedding cannot be empty".to_string()));
            }
            if values.iter().any(|value| !value.is_finite()) {
                return Err(EvidenceValidationError::InvalidPayload("state embedding values must be finite".to_string()));
            }
        }
        LearnedPayload::PredictedTransition(prediction) => {
            require_non_empty("payload.predicted_transition.action", &prediction.action)?;
            require_non_empty("payload.predicted_transition.predicted_state", &prediction.predicted_state)?;
            require_unit_interval("payload.predicted_transition.probability", prediction.probability)?;
        }
        LearnedPayload::ConceptProposal(proposal) => {
            require_non_empty("payload.concept.candidate_id", &proposal.candidate_id)?;
            require_non_empty("payload.concept.descriptor", &proposal.descriptor)?;
            require_unit_interval("payload.concept.stability", proposal.stability)?;
        }
        LearnedPayload::AnomalyScore(score) => {
            require_unit_interval("payload.anomaly_score", *score)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_evidence(timestamp: i64) -> LearnedEvidence {
        LearnedEvidence {
            schema_version: DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION,
            source_model: "infant".to_string(),
            source_version: "test-v1".to_string(),
            observation_id: "obs-1".to_string(),
            modality: LearnedModality::Synthetic,
            payload: LearnedPayload::AnomalyScore(0.25),
            confidence: 0.8,
            uncertainty: 0.2,
            provenance: Provenance {
                producer: "infant-adapter".to_string(),
                model_id: "infant".to_string(),
                model_version: "test-v1".to_string(),
                checkpoint_digest: "sha256:abc123".to_string(),
                source_episode_id: "episode-7".to_string(),
                transformation_trace: vec!["normalize-v1".to_string()],
            },
            timestamp,
        }
    }

    #[test]
    fn valid_evidence_is_accepted() {
        let evidence = sample_evidence(1_000);
        let policy = EvidenceValidationPolicy::strict(1_010, 60);
        assert_eq!(evidence.validate(&policy), Ok(()));
    }

    #[test]
    fn stale_evidence_is_rejected() {
        let evidence = sample_evidence(900);
        let policy = EvidenceValidationPolicy::strict(1_000, 30);
        assert!(matches!(evidence.validate(&policy), Err(EvidenceValidationError::StaleEvidence { .. })));
    }

    #[test]
    fn provenance_mismatch_is_rejected() {
        let mut evidence = sample_evidence(1_000);
        evidence.provenance.model_version = "wrong-version".to_string();
        let policy = EvidenceValidationPolicy::strict(1_010, 60);
        assert!(matches!(evidence.validate(&policy), Err(EvidenceValidationError::ProvenanceMismatch { .. })));
    }

    #[test]
    fn non_finite_embedding_is_rejected() {
        let mut evidence = sample_evidence(1_000);
        evidence.payload = LearnedPayload::StateEmbedding(vec![0.0, f32::NAN]);
        let policy = EvidenceValidationPolicy::strict(1_010, 60);
        assert!(matches!(evidence.validate(&policy), Err(EvidenceValidationError::InvalidPayload(_))));
    }
}
