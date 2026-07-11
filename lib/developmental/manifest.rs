//! Machine-readable experiment baseline manifests for Infant–Starfire fusion work.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const BASELINE_MANIFEST_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineManifest {
    pub schema_version: u16,
    pub experiment_id: String,
    pub created_at: i64,
    pub repositories: BTreeMap<String, RepositoryRevision>,
    pub checkpoints: Vec<CheckpointRecord>,
    pub task_versions: BTreeMap<String, String>,
    pub random_seeds: Vec<u64>,
    pub runtime: RuntimeRecord,
    pub metrics: BTreeMap<String, f64>,
    pub known_failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryRevision {
    pub repository: String,
    pub commit_sha: String,
    pub dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointRecord {
    pub model_id: String,
    pub model_version: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeRecord {
    pub operating_system: String,
    pub architecture: String,
    pub hardware_summary: String,
    pub runtime_versions: BTreeMap<String, String>,
}

impl BaselineManifest {
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        if self.schema_version != BASELINE_MANIFEST_SCHEMA_VERSION {
            return Err(ManifestValidationError::UnsupportedSchema {
                expected: BASELINE_MANIFEST_SCHEMA_VERSION,
                actual: self.schema_version,
            });
        }
        if self.experiment_id.trim().is_empty() {
            return Err(ManifestValidationError::EmptyExperimentId);
        }
        if self.created_at <= 0 {
            return Err(ManifestValidationError::InvalidCreatedAt(self.created_at));
        }
        if self.repositories.is_empty() {
            return Err(ManifestValidationError::MissingRepositories);
        }
        for (name, revision) in &self.repositories {
            if name.trim().is_empty()
                || revision.repository.trim().is_empty()
                || revision.commit_sha.trim().is_empty()
            {
                return Err(ManifestValidationError::InvalidRepositoryRecord(name.clone()));
            }
        }
        for checkpoint in &self.checkpoints {
            if checkpoint.model_id.trim().is_empty()
                || checkpoint.model_version.trim().is_empty()
                || checkpoint.digest.trim().is_empty()
            {
                return Err(ManifestValidationError::InvalidCheckpointRecord(
                    checkpoint.model_id.clone(),
                ));
            }
        }
        for (metric, value) in &self.metrics {
            if metric.trim().is_empty() || !value.is_finite() {
                return Err(ManifestValidationError::InvalidMetric(metric.clone()));
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ManifestValidationError {
    #[error("unsupported baseline manifest schema: expected {expected}, got {actual}")]
    UnsupportedSchema { expected: u16, actual: u16 },
    #[error("experiment_id cannot be empty")]
    EmptyExperimentId,
    #[error("created_at must be positive, got {0}")]
    InvalidCreatedAt(i64),
    #[error("baseline manifest must record at least one repository revision")]
    MissingRepositories,
    #[error("invalid repository record: {0}")]
    InvalidRepositoryRecord(String),
    #[error("invalid checkpoint record for model: {0}")]
    InvalidCheckpointRecord(String),
    #[error("invalid metric: {0}")]
    InvalidMetric(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> BaselineManifest {
        let mut repositories = BTreeMap::new();
        repositories.insert(
            "starfire".to_string(),
            RepositoryRevision {
                repository: "toxzak-svg/starfire".to_string(),
                commit_sha: "abc123".to_string(),
                dirty: false,
            },
        );

        BaselineManifest {
            schema_version: BASELINE_MANIFEST_SCHEMA_VERSION,
            experiment_id: "h-infant-0-fixture".to_string(),
            created_at: 1_000,
            repositories,
            checkpoints: vec![],
            task_versions: BTreeMap::new(),
            random_seeds: vec![1, 2, 3],
            runtime: RuntimeRecord {
                operating_system: "test-os".to_string(),
                architecture: "test-arch".to_string(),
                hardware_summary: "fixture".to_string(),
                runtime_versions: BTreeMap::new(),
            },
            metrics: BTreeMap::new(),
            known_failures: vec![],
        }
    }

    #[test]
    fn baseline_manifest_round_trips() {
        let manifest = sample_manifest();
        manifest.validate().expect("valid manifest");
        let json = manifest.to_json().expect("serialize");
        let decoded = BaselineManifest::from_json(&json).expect("deserialize");
        assert_eq!(manifest, decoded);
    }

    #[test]
    fn non_finite_metrics_are_rejected() {
        let mut manifest = sample_manifest();
        manifest.metrics.insert("bad".to_string(), f64::NAN);
        assert!(matches!(
            manifest.validate(),
            Err(ManifestValidationError::InvalidMetric(_))
        ));
    }
}
