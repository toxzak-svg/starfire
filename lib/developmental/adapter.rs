//! Developmental evidence sources.
//!
//! Adapters only supply typed evidence. They cannot write Starfire beliefs,
//! promote concepts, route operators, or choose actions.

use thiserror::Error;

use super::evidence::{EvidenceValidationPolicy, LearnedEvidence};
use super::replay::{EvidenceReplayLog, ReplayError};

pub trait DevelopmentalEvidenceSource {
    fn source_name(&self) -> &str;
    fn next_evidence(&mut self) -> Result<Option<LearnedEvidence>, AdapterError>;
}

#[derive(Debug, Default)]
pub struct NoopDevelopmentalSource;

impl DevelopmentalEvidenceSource for NoopDevelopmentalSource {
    fn source_name(&self) -> &str {
        "noop"
    }

    fn next_evidence(&mut self) -> Result<Option<LearnedEvidence>, AdapterError> {
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct OfflineReplaySource {
    source_name: String,
    records: Vec<LearnedEvidence>,
    cursor: usize,
}

impl OfflineReplaySource {
    pub fn from_log(
        source_name: impl Into<String>,
        log: EvidenceReplayLog,
        policy: &EvidenceValidationPolicy,
    ) -> Result<Self, AdapterError> {
        log.validate(policy)?;
        Ok(Self {
            source_name: source_name.into(),
            records: log.records,
            cursor: 0,
        })
    }

    pub fn remaining(&self) -> usize {
        self.records.len().saturating_sub(self.cursor)
    }
}

impl DevelopmentalEvidenceSource for OfflineReplaySource {
    fn source_name(&self) -> &str {
        &self.source_name
    }

    fn next_evidence(&mut self) -> Result<Option<LearnedEvidence>, AdapterError> {
        if self.cursor >= self.records.len() {
            return Ok(None);
        }

        let evidence = self.records[self.cursor].clone();
        self.cursor += 1;
        Ok(Some(evidence))
    }
}

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error(transparent)]
    Replay(#[from] ReplayError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::developmental::evidence::{
        LearnedModality, LearnedPayload, Provenance, DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION,
    };

    fn sample_record(observation_id: &str) -> LearnedEvidence {
        LearnedEvidence {
            schema_version: DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION,
            source_model: "infant".to_string(),
            source_version: "fixture-v1".to_string(),
            observation_id: observation_id.to_string(),
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
    fn noop_source_has_zero_behavioral_authority() {
        let mut source = NoopDevelopmentalSource;
        assert_eq!(source.source_name(), "noop");
        assert!(source.next_evidence().expect("noop adapter").is_none());
    }

    #[test]
    fn offline_replay_preserves_record_order() {
        let log = EvidenceReplayLog::new(vec![sample_record("obs-1"), sample_record("obs-2")]);
        let policy = EvidenceValidationPolicy::replay(2_000);
        let mut source = OfflineReplaySource::from_log("fixture", log, &policy).expect("adapter");

        assert_eq!(source.remaining(), 2);
        assert_eq!(
            source.next_evidence().expect("first").unwrap().observation_id,
            "obs-1"
        );
        assert_eq!(
            source.next_evidence().expect("second").unwrap().observation_id,
            "obs-2"
        );
        assert!(source.next_evidence().expect("exhausted").is_none());
    }
}
