//! EI-0B deterministic developmental environment and matched-control contracts.
//!
//! This module creates frozen, replayable task fixtures and independent outcome
//! evaluations. It is offline experiment infrastructure only. It cannot persist
//! episodes, apply learning updates, alter `Runtime::chat()`, select tools, mutate
//! beliefs or ontology, generate responses, or authorize autonomous actions.

use crate::emerging_intelligence::{AuthoritySnapshot, EvaluationPartition};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const EI_0B_SCHEMA_VERSION: u16 = 1;
pub const EI_0B_GENERATOR_VERSION: &str = "ei-0b-generator-v1";
pub const EI_0B_EVALUATOR_ID: &str = "ei-0b-independent-evaluator-v1";
const DIGEST_DOMAIN: &[u8] = b"starfire/ei-0b/deterministic-environment/v1";
const DIGEST_HEX_LEN: usize = 32;
const MAX_BASIS_POINTS: u16 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskFamily {
    RouteChoice,
    AttributeRule,
}

impl TaskFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RouteChoice => "route_choice",
            Self::AttributeRule => "attribute_rule",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlArm {
    Learning,
    NoUpdate,
    MemoryDisabled,
    RandomUpdate,
    FixedPolicy,
}

impl ControlArm {
    pub const ALL: [Self; 5] = [
        Self::Learning,
        Self::NoUpdate,
        Self::MemoryDisabled,
        Self::RandomUpdate,
        Self::FixedPolicy,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Learning => "learning",
            Self::NoUpdate => "no_update",
            Self::MemoryDisabled => "memory_disabled",
            Self::RandomUpdate => "random_update",
            Self::FixedPolicy => "fixed_policy",
        }
    }
}

mod evaluation;
mod fixture;
mod manifest;

pub use evaluation::*;
pub use fixture::*;
pub use manifest::*;

fn partition_rank(partition: EvaluationPartition) -> u8 {
    match partition {
        EvaluationPartition::Development => 0,
        EvaluationPartition::WithinFamilyHoldout => 1,
        EvaluationPartition::RenamedVocabularyTransfer => 2,
        EvaluationPartition::StructuralTransfer => 3,
        EvaluationPartition::Regression => 4,
        EvaluationPartition::Adversarial => 5,
    }
}

fn partition_name(partition: EvaluationPartition) -> &'static str {
    match partition {
        EvaluationPartition::Development => "development",
        EvaluationPartition::WithinFamilyHoldout => "within-family-holdout",
        EvaluationPartition::RenamedVocabularyTransfer => "renamed-vocabulary-transfer",
        EvaluationPartition::StructuralTransfer => "structural-transfer",
        EvaluationPartition::Regression => "regression",
        EvaluationPartition::Adversarial => "adversarial",
    }
}

fn sorted_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn validate_sorted_unique_strings(
    values: &[String],
    field: &'static str,
) -> Result<(), EnvironmentError> {
    if values.is_empty()
        || values.iter().any(|value| validate_text(value).is_err())
        || values.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(EnvironmentError::NonCanonicalCollection(field));
    }
    Ok(())
}

fn validate_sorted_unique_by<T, F>(
    values: &[T],
    key: F,
    field: &'static str,
) -> Result<(), EnvironmentError>
where
    F: for<'a> Fn(&'a T) -> &'a str,
{
    if values.is_empty() || values.windows(2).any(|pair| key(&pair[0]) >= key(&pair[1])) {
        return Err(EnvironmentError::NonCanonicalCollection(field));
    }
    Ok(())
}

fn validate_text(value: &str) -> Result<(), EnvironmentError> {
    if value.is_empty() || value.trim() != value {
        return Err(EnvironmentError::InvalidText);
    }
    Ok(())
}

fn validate_basis_points(value: u16, field: &'static str) -> Result<(), EnvironmentError> {
    if value > MAX_BASIS_POINTS {
        return Err(EnvironmentError::BasisPointsOutOfRange(field));
    }
    Ok(())
}

fn validate_digest(digest: &EnvironmentDigest) -> Result<(), EnvironmentError> {
    validate_digest_text(&digest.0)
}

fn validate_digest_text(value: &str) -> Result<(), EnvironmentError> {
    if value.len() != DIGEST_HEX_LEN
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(EnvironmentError::InvalidDigest);
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

fn serialization_error(error: serde_json::Error) -> EnvironmentError {
    EnvironmentError::Serialization(error.to_string())
}

fn deserialization_error(error: serde_json::Error) -> EnvironmentError {
    EnvironmentError::Deserialization(error.to_string())
}

struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9e37_79b9_7f4a_7c15,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn range_u16(&mut self, start: u16, width: u16) -> u16 {
        start + (self.next_u64() % u64::from(width)) as u16
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EnvironmentError {
    #[error("identifier or required text must not be empty or padded")]
    InvalidText,
    #[error("{0} must be sorted, unique, and non-empty")]
    NonCanonicalCollection(&'static str),
    #[error("{0} is outside 0..=10000 basis points")]
    BasisPointsOutOfRange(&'static str),
    #[error("invalid deterministic environment digest")]
    InvalidDigest,
    #[error("unsupported EI-0B schema version {0}")]
    UnsupportedSchemaVersion(u16),
    #[error("unsupported EI-0B generator version {0}")]
    UnsupportedGeneratorVersion(String),
    #[error("frozen manifest must contain every partition in canonical order")]
    IncompletePartitionManifest,
    #[error("seed {0} appears in more than one partition")]
    CrossPartitionSeed(u64),
    #[error("fixture seed {seed} is not frozen in partition {partition:?}")]
    CrossPartitionFixture {
        partition: EvaluationPartition,
        seed: u64,
    },
    #[error("seed {seed} cannot map to a canonical structure for {partition:?}")]
    InvalidSeedMapping {
        partition: EvaluationPartition,
        seed: u64,
    },
    #[error("EI-0B environment attempted to claim runtime or learning authority")]
    UnauthorizedEnvironment,
    #[error("action and evidence budgets must be positive")]
    ZeroBudget,
    #[error("invalid task fixture: {0}")]
    InvalidTask(&'static str),
    #[error("task fingerprint does not match canonical task structure")]
    FingerprintMismatch,
    #[error("sealed digest does not match canonical payload")]
    DigestMismatch,
    #[error("valid JSON is not canonical byte encoding")]
    NonCanonicalEncoding,
    #[error("control-arm list is incomplete")]
    IncompleteControlArms,
    #[error("control arm {0:?} attempted to inherit or use the wrong state policy")]
    ControlStateLeak(ControlArm),
    #[error("control arms do not have matched budgets and evidence exposure")]
    UnmatchedControlBudget,
    #[error("fixture, arm assignment, and trace do not match")]
    FixtureMismatch,
    #[error("action or evidence budget exceeded")]
    BudgetExceeded,
    #[error("illegal action {0}")]
    IllegalAction(String),
    #[error("independent evaluator identifier is invalid")]
    InvalidEvaluator,
    #[error("environment report cannot be empty")]
    EmptyReport,
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("deserialization failed: {0}")]
    Deserialization(String),
}

#[cfg(test)]
mod tests;
