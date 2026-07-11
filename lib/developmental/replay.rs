//! Versioned replay container for developmental evidence.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::evidence::{
    EvidenceValidationError, EvidenceValidationPolicy, LearnedEvidence,
    DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION,
};

pub const DEVELOPMENTAL_REPLAY_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceReplayLog {
    pub replay_schema_version: u16,
    pub evidence_schema_version: u16,
    pub records: Vec<LearnedEvidence>,
}

impl EvidenceReplayLog {
    pub fn new(records: Vec<LearnedEvidence>) -> Self {
        Self {
            replay_schema_version: DEVELOPMENTAL_REPLAY_SCHEMA_VERSION,
            evidence_schema_version: DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION,
            records,
        }
    }

    pub fn validate(&self, policy: &EvidenceValidationPolicy) -> Result<(), ReplayError> {
        if self.replay_schema_version != DEVELOPMENTAL_REPLAY_SCHEMA_VERSION {
            return Err(ReplayError::UnsupportedReplaySchema {
                expected: DEVELOPMENTAL_REPLAY_SCHEMA_VERSION,
                actual: self.replay_schema_version,
            });
        }

        if self.evidence_schema_version != policy.accepted_schema_version {
            return Err(ReplayError::EvidenceSchemaMismatch {
                declared: self.evidence_schema_version,
                policy: policy.accepted_schema_version,
            });
        }

        for (index, record) in self.records.iter().enumerate() {
            record
                .validate(policy)
                .map_err(|source| ReplayError::InvalidRecord { index, source })?;
        }

        Ok(())
    }

    pub fn to_json(&self) -> Result<String, ReplayError> {
        serde_json::to_string(self).map_err(ReplayError::Serialize)
    }

    pub fn from_json(json: &str) -> Result<Self, ReplayError> {
        serde_json::from_str(json).map_err(ReplayError::Deserialize)
    }
}

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("unsupported developmental replay schema: expected {expected}, got {actual}")]
    UnsupportedReplaySchema { expected: u16, actual: u16 },
    #[error("replay declares evidence schema {declared}, but validation policy accepts {policy}")]
    EvidenceSchemaMismatch { declared: u16, policy: u16 },
    #[error("invalid developmental evidence record at index {index}: {source}")]
    InvalidRecord {
        index: usize,
        #[source]
        source: EvidenceValidationError,
    },
    #[error("failed to serialize developmental replay log: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to deserialize developmental replay log: {0}")]
    Deserialize(serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::developmental::evidence::{
        LearnedModality, LearnedPayload, Provenance,
    };

    fn sample_record() -> LearnedEvidence {
        LearnedEvidence {
            schema_version: DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION,
            source_model: "infant".to_string(),
            source_version: "fixture-v1".to_string(),
            observation_id: "obs-1".to_string(),
            modality: LearnedModality::Synthetic,
            payload: LearnedPayload::AnomalyScore(0.1),
            confidence: 0.9,
            uncertainty: 0.1,
            provenance: Provenance {
                producer: "fixture".to_string(),
                model_id: "infant".to_string(),
                model_version: "fixture-v1".to_string(),
                checkpoint_digest: "sha256:fixture".to_string(),
                source_episode_id: "episode-1".to_string(),
                transformation_trace: vec![],
            },
            timestamp: 1_000,
        }
    }

    #[test]
    fn json_round_trip_is_stable() {
        let log = EvidenceReplayLog::new(vec![sample_record()]);
        let json_a = log.to_json().expect("serialize");
        let decoded = EvidenceReplayLog::from_json(&json_a).expect("deserialize");
        let json_b = decoded.to_json().expect("serialize again");

        assert_eq!(log, decoded);
        assert_eq!(json_a, json_b);
    }

    #[test]
    fn malformed_json_is_rejected() {
        assert!(matches!(
            EvidenceReplayLog::from_json("{not-json}"),
            Err(ReplayError::Deserialize(_))
        ));
    }

    #[test]
    fn replay_validation_identifies_bad_record_index() {
        let mut bad = sample_record();
        bad.confidence = 2.0;
        let log = EvidenceReplayLog::new(vec![sample_record(), bad]);
        let policy = EvidenceValidationPolicy::replay(2_000);

        assert!(matches!(
            log.validate(&policy),
            Err(ReplayError::InvalidRecord { index: 1, .. })
        ));
    }
}
