use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenPartition {
    pub partition: EvaluationPartition,
    /// Sorted and unique. Seeds are globally disjoint across partitions.
    pub seeds: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenEnvironmentManifest {
    pub schema_version: u16,
    pub generator_version: String,
    /// Canonical partition order: development, holdout, renamed, structural,
    /// regression, adversarial.
    pub partitions: Vec<FrozenPartition>,
    /// Sorted and unique source hashes.
    pub source_hashes: Vec<String>,
    pub action_budget: u32,
    pub evidence_budget: u32,
    pub authority: AuthoritySnapshot,
}

impl FrozenEnvironmentManifest {
    pub fn ei_0b_default() -> Self {
        Self {
            schema_version: EI_0B_SCHEMA_VERSION,
            generator_version: EI_0B_GENERATOR_VERSION.into(),
            partitions: vec![
                FrozenPartition {
                    partition: EvaluationPartition::Development,
                    seeds: vec![101, 102],
                },
                FrozenPartition {
                    partition: EvaluationPartition::WithinFamilyHoldout,
                    seeds: vec![201, 202],
                },
                FrozenPartition {
                    partition: EvaluationPartition::RenamedVocabularyTransfer,
                    seeds: vec![301, 302],
                },
                FrozenPartition {
                    partition: EvaluationPartition::StructuralTransfer,
                    seeds: vec![401, 402],
                },
                FrozenPartition {
                    partition: EvaluationPartition::Regression,
                    seeds: vec![501, 502],
                },
                FrozenPartition {
                    partition: EvaluationPartition::Adversarial,
                    seeds: vec![601, 602],
                },
            ],
            source_hashes: vec![
                "source:ei0b-attribute-rule-v1".into(),
                "source:ei0b-route-choice-v1".into(),
            ],
            action_budget: 1,
            evidence_budget: 2,
            authority: AuthoritySnapshot::closed(),
        }
    }

    pub fn validate(&self) -> Result<(), EnvironmentError> {
        if self.schema_version != EI_0B_SCHEMA_VERSION {
            return Err(EnvironmentError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        validate_text(&self.generator_version)?;
        if self.generator_version != EI_0B_GENERATOR_VERSION {
            return Err(EnvironmentError::UnsupportedGeneratorVersion(
                self.generator_version.clone(),
            ));
        }
        if self.action_budget == 0 || self.evidence_budget == 0 {
            return Err(EnvironmentError::ZeroBudget);
        }
        if !self.authority.is_closed() {
            return Err(EnvironmentError::UnauthorizedEnvironment);
        }
        validate_sorted_unique_strings(&self.source_hashes, "manifest source hashes")?;
        if self.partitions.len() != 6 {
            return Err(EnvironmentError::IncompletePartitionManifest);
        }

        let expected = [
            EvaluationPartition::Development,
            EvaluationPartition::WithinFamilyHoldout,
            EvaluationPartition::RenamedVocabularyTransfer,
            EvaluationPartition::StructuralTransfer,
            EvaluationPartition::Regression,
            EvaluationPartition::Adversarial,
        ];
        let mut all_seeds = BTreeSet::new();
        for (entry, expected_partition) in self.partitions.iter().zip(expected) {
            if entry.partition != expected_partition || entry.seeds.is_empty() {
                return Err(EnvironmentError::IncompletePartitionManifest);
            }
            if entry.seeds.windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err(EnvironmentError::NonCanonicalCollection("partition seeds"));
            }
            for seed in &entry.seeds {
                if !all_seeds.insert(*seed) {
                    return Err(EnvironmentError::CrossPartitionSeed(*seed));
                }
            }
        }
        Ok(())
    }

    pub fn contains(&self, partition: EvaluationPartition, seed: u64) -> bool {
        self.partitions
            .iter()
            .find(|entry| entry.partition == partition)
            .is_some_and(|entry| entry.seeds.binary_search(&seed).is_ok())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, EnvironmentError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(serialization_error)
    }

    pub fn digest(&self) -> Result<EnvironmentDigest, EnvironmentError> {
        Ok(EnvironmentDigest(checksum128(&self.canonical_bytes()?)))
    }
}
