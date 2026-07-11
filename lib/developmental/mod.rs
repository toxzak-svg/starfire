//! Experimental Infant–Starfire developmental evidence boundary.
//!
//! This module is compiled only with the `developmental-evidence` feature and is
//! intentionally not wired into `Runtime::chat()`, cognitive routing, belief
//! promotion, ontology promotion, or autonomous action selection.

pub mod adapter;
pub mod evidence;
pub mod manifest;
pub mod replay;

pub use adapter::{
    AdapterError, DevelopmentalEvidenceSource, NoopDevelopmentalSource, OfflineReplaySource,
};
pub use evidence::{
    ConceptProposal, EvidenceValidationError, EvidenceValidationPolicy, LearnedEvidence,
    LearnedModality, LearnedObject, LearnedPayload, PredictedTransition, Provenance,
    DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION,
};
pub use manifest::{
    BaselineManifest, CheckpointRecord, ManifestValidationError, RepositoryRevision, RuntimeRecord,
    BASELINE_MANIFEST_SCHEMA_VERSION,
};
pub use replay::{
    EvidenceReplayLog, ReplayError, DEVELOPMENTAL_REPLAY_SCHEMA_VERSION,
};
