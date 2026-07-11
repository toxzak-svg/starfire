//! Experimental Infant–Starfire developmental evidence boundary.
//!
//! Compiled only with the `developmental-evidence` feature and intentionally not
//! wired into Runtime::chat(), routing, belief promotion, ontology promotion, or
//! autonomous action selection.

pub mod adapter;
pub mod calibration;
pub mod evidence;
pub mod manifest;
pub mod replay;
pub mod residual;

pub use adapter::{
    AdapterError, DevelopmentalEvidenceSource, NoopDevelopmentalSource,
    OfflineReplaySource,
};
pub use calibration::{
    CalibrationError, QuantileMethod, ResidualAssessment,
    ResidualCalibrationProfile, ResidualCalibrationScope, ResidualMetric,
    RESIDUAL_CALIBRATION_SCHEMA_VERSION,
};
pub use evidence::{
    ConceptProposal, EvidenceValidationError, EvidenceValidationPolicy,
    LearnedEvidence, LearnedModality, LearnedObject, LearnedPayload, NamedVector,
    NumericStateObservation, NumericTransitionPrediction, PredictedTransition,
    Provenance, DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION,
};
pub use manifest::{
    BaselineManifest, CheckpointRecord, ManifestValidationError,
    RepositoryRevision, RuntimeRecord, BASELINE_MANIFEST_SCHEMA_VERSION,
};
pub use replay::{
    EvidenceReplayLog, ReplayError, DEVELOPMENTAL_REPLAY_SCHEMA_VERSION,
};
pub use residual::{
    compare_numeric_transition, NumericPredictionResidual, ResidualError,
};

#[cfg(test)]
mod tests;
