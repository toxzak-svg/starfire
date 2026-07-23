//! Evidence-backed identity genome contract.
//!
//! This module replaces indiscriminate identity prose injection with typed,
//! provenance-bearing claims and deterministic contextual retrieval. It is an
//! isolated contract only: no automatic belief promotion, ontology mutation,
//! Runtime::chat wiring, persistence migration, or autonomous-action authority.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const IDENTITY_GENOME_SCHEMA_VERSION: u16 = 1;
pub const MAX_IDENTITY_SLICE_CLAIMS: usize = 8;
pub const MAX_IDENTITY_TAGS_PER_CLAIM: usize = 16;
pub const MAX_IDENTITY_EVIDENCE_REFS: usize = 32;
pub const MAX_IDENTITY_CONTRADICTION_REFS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IdentityClaimType {
    Invariant,
    Value,
    SelfHypothesis,
    BehavioralTendency,
    AutobiographicalEvidence,
    RelationshipFact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityPersistence {
    Permanent,
    LongLived,
    Revisable,
    Episodic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityClaim {
    pub id: String,
    pub claim_type: IdentityClaimType,
    pub statement: String,
    pub confidence_bps: u16,
    pub provenance: String,
    pub evidence_refs: Vec<String>,
    pub contradiction_refs: Vec<String>,
    pub persistence: IdentityPersistence,
    pub expression_weight_bps: u16,
    pub tags: BTreeSet<String>,
    pub quarantined: bool,
}

impl IdentityClaim {
    pub fn verify_integrity(&self) -> Result<(), IdentityGenomeError> {
        if self.id.trim().is_empty()
            || self.statement.trim().is_empty()
            || self.provenance.trim().is_empty()
            || self.confidence_bps > 10_000
            || self.expression_weight_bps > 10_000
            || self.tags.len() > MAX_IDENTITY_TAGS_PER_CLAIM
            || self.evidence_refs.len() > MAX_IDENTITY_EVIDENCE_REFS
            || self.contradiction_refs.len() > MAX_IDENTITY_CONTRADICTION_REFS
            || self.tags.iter().any(|tag| tag.trim().is_empty())
            || self.evidence_refs.iter().any(|reference| reference.trim().is_empty())
            || self
                .contradiction_refs
                .iter()
                .any(|reference| reference.trim().is_empty())
        {
            return Err(IdentityGenomeError::InvalidClaim(self.id.clone()));
        }

        match self.claim_type {
            IdentityClaimType::Invariant => {
                if self.confidence_bps < 9_000
                    || self.evidence_refs.is_empty()
                    || !self.contradiction_refs.is_empty()
                    || self.quarantined
                {
                    return Err(IdentityGenomeError::InvalidInvariant(self.id.clone()));
                }
            }
            IdentityClaimType::SelfHypothesis => {
                if self.confidence_bps >= 9_000 {
                    return Err(IdentityGenomeError::OverstatedSelfHypothesis(
                        self.id.clone(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityGenome {
    pub schema_version: u16,
    pub claims: BTreeMap<String, IdentityClaim>,
}

impl Default for IdentityGenome {
    fn default() -> Self {
        Self {
            schema_version: IDENTITY_GENOME_SCHEMA_VERSION,
            claims: BTreeMap::new(),
        }
    }
}

impl IdentityGenome {
    pub fn from_json(json: &str) -> Result<Self, IdentityGenomeError> {
        let genome: Self = serde_json::from_str(json)
            .map_err(|error| IdentityGenomeError::Decode(error.to_string()))?;
        genome.verify_integrity()?;
        Ok(genome)
    }

    pub fn to_json(&self) -> Result<String, IdentityGenomeError> {
        self.verify_integrity()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| IdentityGenomeError::Decode(error.to_string()))
    }

    pub fn verify_integrity(&self) -> Result<(), IdentityGenomeError> {
        if self.schema_version != IDENTITY_GENOME_SCHEMA_VERSION {
            return Err(IdentityGenomeError::InvalidSchemaVersion);
        }
        for (key, claim) in &self.claims {
            if key != &claim.id {
                return Err(IdentityGenomeError::ClaimKeyMismatch(key.clone()));
            }
            claim.verify_integrity()?;
            for contradiction in &claim.contradiction_refs {
                if contradiction == &claim.id {
                    return Err(IdentityGenomeError::SelfContradictionReference(
                        claim.id.clone(),
                    ));
                }
                if !self.claims.contains_key(contradiction) {
                    return Err(IdentityGenomeError::UnknownContradictionReference {
                        claim_id: claim.id.clone(),
                        contradiction_id: contradiction.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn insert_claim(&mut self, claim: IdentityClaim) -> Result<(), IdentityGenomeError> {
        claim.verify_integrity()?;
        if self.claims.contains_key(&claim.id) {
            return Err(IdentityGenomeError::DuplicateClaim(claim.id));
        }
        self.claims.insert(claim.id.clone(), claim);
        self.verify_integrity()
    }

    pub fn quarantine_contradiction(
        &mut self,
        left_id: &str,
        right_id: &str,
    ) -> Result<(), IdentityGenomeError> {
        if left_id == right_id {
            return Err(IdentityGenomeError::SelfContradictionReference(
                left_id.to_string(),
            ));
        }
        if !self.claims.contains_key(left_id) || !self.claims.contains_key(right_id) {
            return Err(IdentityGenomeError::UnknownClaim);
        }
        {
            let left = self
                .claims
                .get_mut(left_id)
                .ok_or(IdentityGenomeError::UnknownClaim)?;
            push_unique_bounded(
                &mut left.contradiction_refs,
                right_id,
                MAX_IDENTITY_CONTRADICTION_REFS,
            )?;
            left.quarantined = true;
        }
        {
            let right = self
                .claims
                .get_mut(right_id)
                .ok_or(IdentityGenomeError::UnknownClaim)?;
            push_unique_bounded(
                &mut right.contradiction_refs,
                left_id,
                MAX_IDENTITY_CONTRADICTION_REFS,
            )?;
            right.quarantined = true;
        }
        self.verify_non_invariant_quarantine()
    }

    pub fn retrieve_slice(
        &self,
        query: &IdentityQuery,
    ) -> Result<IdentitySlice, IdentityGenomeError> {
        self.verify_integrity()?;
        query.verify_integrity()?;
        let mut scored = self
            .claims
            .values()
            .filter(|claim| !claim.quarantined && claim.contradiction_refs.is_empty())
            .map(|claim| {
                let overlap = claim.tags.intersection(&query.tags).count() as i64;
                let score = overlap * 10_000
                    + i64::from(claim.expression_weight_bps)
                    + i64::from(claim.confidence_bps / 4)
                    + claim_type_priority(claim.claim_type);
                ScoredIdentityClaim {
                    claim_id: claim.id.clone(),
                    score,
                }
            })
            .collect::<Vec<_>>();
        scored.sort_by(compare_scored_claims);
        scored.truncate(query.max_claims);
        Ok(IdentitySlice {
            claim_ids: scored.iter().map(|item| item.claim_id.clone()).collect(),
            scores: scored.iter().map(|item| item.score).collect(),
        })
    }

    fn verify_non_invariant_quarantine(&self) -> Result<(), IdentityGenomeError> {
        for claim in self.claims.values() {
            if claim.claim_type == IdentityClaimType::Invariant
                && (claim.quarantined || !claim.contradiction_refs.is_empty())
            {
                return Err(IdentityGenomeError::InvariantContradictionRequiresReview(
                    claim.id.clone(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityQuery {
    pub tags: BTreeSet<String>,
    pub max_claims: usize,
}

impl IdentityQuery {
    pub fn verify_integrity(&self) -> Result<(), IdentityGenomeError> {
        if self.max_claims == 0
            || self.max_claims > MAX_IDENTITY_SLICE_CLAIMS
            || self.tags.is_empty()
            || self.tags.iter().any(|tag| tag.trim().is_empty())
        {
            return Err(IdentityGenomeError::InvalidQuery);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentitySlice {
    pub claim_ids: Vec<String>,
    pub scores: Vec<i64>,
}

#[derive(Debug, Clone)]
struct ScoredIdentityClaim {
    claim_id: String,
    score: i64,
}

fn compare_scored_claims(left: &ScoredIdentityClaim, right: &ScoredIdentityClaim) -> Ordering {
    right
        .score
        .cmp(&left.score)
        .then_with(|| left.claim_id.cmp(&right.claim_id))
}

fn claim_type_priority(claim_type: IdentityClaimType) -> i64 {
    match claim_type {
        IdentityClaimType::Invariant => 600,
        IdentityClaimType::Value => 500,
        IdentityClaimType::BehavioralTendency => 400,
        IdentityClaimType::AutobiographicalEvidence => 300,
        IdentityClaimType::RelationshipFact => 200,
        IdentityClaimType::SelfHypothesis => 100,
    }
}

fn push_unique_bounded(
    values: &mut Vec<String>,
    value: &str,
    max: usize,
) -> Result<(), IdentityGenomeError> {
    if !values.iter().any(|existing| existing == value) {
        if values.len() >= max {
            return Err(IdentityGenomeError::ContradictionBudgetExceeded);
        }
        values.push(value.to_string());
        values.sort();
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityGenomeAuthorityBoundary {
    pub typed_claim_storage: bool,
    pub deterministic_contextual_retrieval: bool,
    pub contradiction_quarantine: bool,
    pub automatic_invariant_promotion: bool,
    pub automatic_belief_promotion: bool,
    pub automatic_ontology_promotion: bool,
    pub runtime_chat_influence: bool,
    pub unrestricted_memory_access: bool,
    pub routing_authority: bool,
    pub tool_selection_authority: bool,
    pub autonomous_action_authority: bool,
}

#[must_use]
pub const fn authority_boundary() -> IdentityGenomeAuthorityBoundary {
    IdentityGenomeAuthorityBoundary {
        typed_claim_storage: true,
        deterministic_contextual_retrieval: true,
        contradiction_quarantine: true,
        automatic_invariant_promotion: false,
        automatic_belief_promotion: false,
        automatic_ontology_promotion: false,
        runtime_chat_influence: false,
        unrestricted_memory_access: false,
        routing_authority: false,
        tool_selection_authority: false,
        autonomous_action_authority: false,
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdentityGenomeError {
    #[error("identity genome JSON is invalid: {0}")]
    Decode(String),
    #[error("identity genome schema version is invalid")]
    InvalidSchemaVersion,
    #[error("identity claim is invalid: {0}")]
    InvalidClaim(String),
    #[error("identity invariant lacks evidence, confidence, or contradiction clearance: {0}")]
    InvalidInvariant(String),
    #[error("self-hypothesis is overstated as near-certainty: {0}")]
    OverstatedSelfHypothesis(String),
    #[error("identity claim key does not match claim id: {0}")]
    ClaimKeyMismatch(String),
    #[error("identity claim already exists: {0}")]
    DuplicateClaim(String),
    #[error("identity contradiction points to itself: {0}")]
    SelfContradictionReference(String),
    #[error("identity contradiction reference is unknown: {claim_id} -> {contradiction_id}")]
    UnknownContradictionReference {
        claim_id: String,
        contradiction_id: String,
    },
    #[error("identity claim is unknown")]
    UnknownClaim,
    #[error("identity contradiction budget exceeded")]
    ContradictionBudgetExceeded,
    #[error("an invariant contradiction requires explicit human review: {0}")]
    InvariantContradictionRequiresReview(String),
    #[error("identity retrieval query is invalid")]
    InvalidQuery,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(values: &[&str]) -> BTreeSet<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    fn claim(
        id: &str,
        claim_type: IdentityClaimType,
        confidence_bps: u16,
        expression_weight_bps: u16,
        claim_tags: &[&str],
    ) -> IdentityClaim {
        IdentityClaim {
            id: id.to_string(),
            claim_type,
            statement: format!("statement for {id}"),
            confidence_bps,
            provenance: "test-fixture".to_string(),
            evidence_refs: if claim_type == IdentityClaimType::Invariant {
                vec!["evidence:test".to_string()]
            } else {
                Vec::new()
            },
            contradiction_refs: Vec::new(),
            persistence: IdentityPersistence::Revisable,
            expression_weight_bps,
            tags: tags(claim_tags),
            quarantined: false,
        }
    }

    #[test]
    fn retrieval_is_bounded_relevant_and_replayable() {
        let mut genome = IdentityGenome::default();
        genome
            .insert_claim(claim(
                "value-honesty",
                IdentityClaimType::Value,
                8_500,
                8_000,
                &["technical", "uncertainty"],
            ))
            .unwrap();
        genome
            .insert_claim(claim(
                "tendency-direct",
                IdentityClaimType::BehavioralTendency,
                8_000,
                7_500,
                &["technical", "direct"],
            ))
            .unwrap();
        genome
            .insert_claim(claim(
                "relationship-history",
                IdentityClaimType::RelationshipFact,
                8_000,
                9_000,
                &["relationship"],
            ))
            .unwrap();
        let query = IdentityQuery {
            tags: tags(&["technical"]),
            max_claims: 2,
        };
        let first = genome.retrieve_slice(&query).unwrap();
        let second = genome.retrieve_slice(&query).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.claim_ids.len(), 2);
        assert!(!first.claim_ids.contains(&"relationship-history".to_string()));
    }

    #[test]
    fn overstated_self_hypotheses_fail_closed() {
        let hypothesis = claim(
            "hypothesis-conscious",
            IdentityClaimType::SelfHypothesis,
            9_500,
            5_000,
            &["self-model"],
        );
        assert_eq!(
            hypothesis.verify_integrity().unwrap_err(),
            IdentityGenomeError::OverstatedSelfHypothesis(
                "hypothesis-conscious".to_string()
            )
        );
    }

    #[test]
    fn contradictions_are_quarantined_and_not_retrieved() {
        let mut genome = IdentityGenome::default();
        genome
            .insert_claim(claim(
                "tendency-a",
                IdentityClaimType::BehavioralTendency,
                7_000,
                8_000,
                &["voice"],
            ))
            .unwrap();
        genome
            .insert_claim(claim(
                "tendency-b",
                IdentityClaimType::BehavioralTendency,
                7_000,
                8_000,
                &["voice"],
            ))
            .unwrap();
        genome
            .quarantine_contradiction("tendency-a", "tendency-b")
            .unwrap();
        let slice = genome
            .retrieve_slice(&IdentityQuery {
                tags: tags(&["voice"]),
                max_claims: 2,
            })
            .unwrap();
        assert!(slice.claim_ids.is_empty());
    }

    #[test]
    fn json_round_trip_is_exact() {
        let mut genome = IdentityGenome::default();
        genome
            .insert_claim(claim(
                "value-truth",
                IdentityClaimType::Value,
                8_000,
                8_500,
                &["truth"],
            ))
            .unwrap();
        let json = genome.to_json().unwrap();
        let replay = IdentityGenome::from_json(&json).unwrap();
        assert_eq!(genome, replay);
    }

    #[test]
    fn authority_boundary_is_closed() {
        let boundary = authority_boundary();
        assert!(boundary.typed_claim_storage);
        assert!(boundary.deterministic_contextual_retrieval);
        assert!(!boundary.automatic_invariant_promotion);
        assert!(!boundary.automatic_belief_promotion);
        assert!(!boundary.automatic_ontology_promotion);
        assert!(!boundary.runtime_chat_influence);
        assert!(!boundary.autonomous_action_authority);
    }
}
