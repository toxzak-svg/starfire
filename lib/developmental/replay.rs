use serde::{Deserialize, Serialize};
use thiserror::Error;
use super::evidence::{EvidenceValidationError, EvidenceValidationPolicy, LearnedEvidence, DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION};

pub const DEVELOPMENTAL_REPLAY_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceReplayLog {
    pub replay_schema_version: u16,
    pub evidence_schema_version: u16,
    pub records: Vec<LearnedEvidence>,
}

impl EvidenceReplayLog {
    pub fn new(records: Vec<LearnedEvidence>) -> Self {
        Self { replay_schema_version: DEVELOPMENTAL_REPLAY_SCHEMA_VERSION, evidence_schema_version: DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION, records }
    }
    pub fn validate(&self, policy: &EvidenceValidationPolicy) -> Result<(), ReplayError> {
        if self.replay_schema_version != DEVELOPMENTAL_REPLAY_SCHEMA_VERSION {
            return Err(ReplayError::UnsupportedReplaySchema { expected: DEVELOPMENTAL_REPLAY_SCHEMA_VERSION, actual: self.replay_schema_version });
        }
        if self.evidence_schema_version != policy.accepted_schema_version {
            return Err(ReplayError::EvidenceSchemaMismatch { declared: self.evidence_schema_version, policy: policy.accepted_schema_version });
        }
        for (index, record) in self.records.iter().enumerate() {
            record.validate(policy).map_err(|source| ReplayError::InvalidRecord { index, source })?;
        }
        Ok(())
    }
    pub fn to_json(&self) -> Result<String, ReplayError> { serde_json::to_string(self).map_err(ReplayError::Serialize) }
    pub fn from_json(json: &str) -> Result<Self, ReplayError> { serde_json::from_str(json).map_err(ReplayError::Deserialize) }
}

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("unsupported developmental replay schema: expected {expected}, got {actual}")]
    UnsupportedReplaySchema { expected: u16, actual: u16 },
    #[error("replay declares evidence schema {declared}, but policy accepts {policy}")]
    EvidenceSchemaMismatch { declared: u16, policy: u16 },
    #[error("invalid developmental evidence record at index {index}: {source}")]
    InvalidRecord { index: usize, #[source] source: EvidenceValidationError },
    #[error("failed to serialize developmental replay log: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to deserialize developmental replay log: {0}")]
    Deserialize(serde_json::Error),
}
