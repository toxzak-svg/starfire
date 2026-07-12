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
    fn source_name(&self) -> &str { "noop" }
    fn next_evidence(&mut self) -> Result<Option<LearnedEvidence>, AdapterError> { Ok(None) }
}

#[derive(Debug, Clone)]
pub struct OfflineReplaySource {
    source_name: String,
    records: Vec<LearnedEvidence>,
    cursor: usize,
}

impl OfflineReplaySource {
    pub fn from_log(source_name: impl Into<String>, log: EvidenceReplayLog, policy: &EvidenceValidationPolicy) -> Result<Self, AdapterError> {
        log.validate(policy)?;
        Ok(Self { source_name: source_name.into(), records: log.records, cursor: 0 })
    }
    pub fn remaining(&self) -> usize { self.records.len().saturating_sub(self.cursor) }
}

impl DevelopmentalEvidenceSource for OfflineReplaySource {
    fn source_name(&self) -> &str { &self.source_name }
    fn next_evidence(&mut self) -> Result<Option<LearnedEvidence>, AdapterError> {
        if self.cursor >= self.records.len() { return Ok(None); }
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
