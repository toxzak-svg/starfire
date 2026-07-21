//! Native IngExuity companion-state substrate for Starfire.
//!
//! The companion layer lives below HTTP, model, and presentation code. It owns
//! typed user claims, provenance, corrections, retention, sensitivity, replay,
//! and deletion semantics. Conflicting observations emit CHARGE but never
//! silently replace the user's active claim.

use crate::charge::{Charge, ChargeKind, ChargeScope};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

pub type ClaimId = u64;
pub type ObservationId = u64;

const RETENTION_EXPIRED_REASON: &str = "retention expired";
const CONTESTED_PARENT_DELETED_REASON: &str = "contested claim deleted";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimSource {
    UserStatement,
    UserCorrection { corrects: ClaimId },
    Inference { method: String },
    Import { system: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sensitivity {
    Public,
    Personal,
    Sensitive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Retention {
    Session,
    Durable,
    Until { expires_at_ms: u64 },
}

impl Retention {
    #[must_use]
    pub fn is_expired(&self, now_ms: u64) -> bool {
        matches!(self, Self::Until { expires_at_ms } if now_ms >= *expires_at_ms)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimStatus {
    Active,
    Contested { with: ClaimId },
    Superseded { by: ClaimId },
    Invalidated { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub id: ObservationId,
    pub claim_id: ClaimId,
    pub value: String,
    pub source: ClaimSource,
    pub observed_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanionClaim {
    pub id: ClaimId,
    pub key: String,
    pub value: String,
    pub confidence_bps: u16,
    pub source: ClaimSource,
    pub supporting_observation_ids: Vec<ObservationId>,
    pub status: ClaimStatus,
    pub sensitivity: Sensitivity,
    pub retention: Retention,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimInput {
    pub key: String,
    pub value: String,
    pub source: ClaimSource,
    pub confidence_bps: u16,
    pub sensitivity: Sensitivity,
    pub retention: Retention,
    pub observed_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrectionInput {
    pub value: String,
    pub confidence_bps: u16,
    pub sensitivity: Sensitivity,
    pub retention: Retention,
    pub observed_at_ms: u64,
}

/// Replayable mutation of companion state.
///
/// `retired_expired_claim_id` is part of the event rather than recomputed at
/// replay time. This makes retention-driven replacement deterministic and
/// auditable even if replay occurs at a different wall-clock time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompanionEvent {
    ClaimRecorded {
        claim: CompanionClaim,
        observation: Observation,
        retired_expired_claim_id: Option<ClaimId>,
    },
    ObservationAttached {
        claim_id: ClaimId,
        observation: Observation,
        confidence_bps: u16,
        updated_at_ms: u64,
    },
    ClaimCorrected {
        previous_claim_id: ClaimId,
        replacement: CompanionClaim,
        observation: Observation,
    },
    ClaimInvalidated {
        claim_id: ClaimId,
        reason: String,
        occurred_at_ms: u64,
    },
    ClaimDeleted {
        claim_id: ClaimId,
        occurred_at_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompanionTransition {
    pub version: u64,
    pub claim_id: Option<ClaimId>,
    pub event: CompanionEvent,
    pub emitted_charges: Vec<Charge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanionState {
    pub version: u64,
    claims: BTreeMap<ClaimId, CompanionClaim>,
    observations: BTreeMap<ObservationId, Observation>,
    active_by_key: BTreeMap<String, ClaimId>,
    next_claim_id: ClaimId,
    next_observation_id: ObservationId,
}

impl Default for CompanionState {
    fn default() -> Self {
        Self::new()
    }
}

impl CompanionState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: 0,
            claims: BTreeMap::new(),
            observations: BTreeMap::new(),
            active_by_key: BTreeMap::new(),
            next_claim_id: 1,
            next_observation_id: 1,
        }
    }

    #[must_use]
    pub fn claims(&self) -> &BTreeMap<ClaimId, CompanionClaim> {
        &self.claims
    }

    #[must_use]
    pub fn observations(&self) -> &BTreeMap<ObservationId, Observation> {
        &self.observations
    }

    #[must_use]
    pub fn claim(&self, claim_id: ClaimId) -> Option<&CompanionClaim> {
        self.claims.get(&claim_id)
    }

    #[must_use]
    pub fn active_claim(
        &self,
        key: &str,
        now_ms: u64,
        include_sensitive: bool,
    ) -> Option<&CompanionClaim> {
        let key = canonical_key(key)?;
        let claim_id = self.active_by_key.get(&key)?;
        let claim = self.claims.get(claim_id)?;
        if claim.retention.is_expired(now_ms)
            || (!include_sensitive && claim.sensitivity == Sensitivity::Sensitive)
        {
            return None;
        }
        Some(claim)
    }

    pub fn record_claim(
        &mut self,
        expected_version: u64,
        input: ClaimInput,
    ) -> Result<CompanionTransition, CompanionError> {
        self.require_version(expected_version)?;
        validate_confidence(input.confidence_bps)?;
        let key = canonical_key(&input.key).ok_or(CompanionError::EmptyKey)?;
        let value = normalized_value(&input.value).ok_or(CompanionError::EmptyValue)?;

        if let ClaimSource::UserCorrection { corrects } = &input.source {
            return self.correct_claim(
                expected_version,
                *corrects,
                CorrectionInput {
                    value,
                    confidence_bps: input.confidence_bps,
                    sensitivity: input.sensitivity,
                    retention: input.retention,
                    observed_at_ms: input.observed_at_ms,
                },
            );
        }

        let indexed_active_id = self.active_by_key.get(&key).copied();
        let retired_expired_claim_id = indexed_active_id
            .filter(|claim_id| {
                self.claims
                    .get(claim_id)
                    .is_some_and(|claim| claim.retention.is_expired(input.observed_at_ms))
            });
        let active_id = indexed_active_id.filter(|claim_id| {
            !retired_expired_claim_id.is_some_and(|expired_id| expired_id == *claim_id)
        });

        if let Some(active_id) = active_id {
            let active = self
                .claims
                .get(&active_id)
                .ok_or(CompanionError::BrokenActiveIndex(active_id))?;
            if equivalent_value(&active.value, &value) {
                let confidence_bps = active.confidence_bps.max(input.confidence_bps);
                let updated_at_ms = active.updated_at_ms.max(input.observed_at_ms);
                let observation = Observation {
                    id: self.next_observation_id,
                    claim_id: active_id,
                    value,
                    source: input.source,
                    observed_at_ms: input.observed_at_ms,
                };
                let event = CompanionEvent::ObservationAttached {
                    claim_id: active_id,
                    observation,
                    confidence_bps,
                    updated_at_ms,
                };
                let version = self.apply_event(expected_version, event.clone())?;
                return Ok(CompanionTransition {
                    version,
                    claim_id: Some(active_id),
                    event,
                    emitted_charges: Vec::new(),
                });
            }
        }

        let claim_id = self.next_claim_id;
        let observation_id = self.next_observation_id;
        let contested_with = active_id;
        let claim = CompanionClaim {
            id: claim_id,
            key: key.clone(),
            value: value.clone(),
            confidence_bps: input.confidence_bps,
            source: input.source.clone(),
            supporting_observation_ids: vec![observation_id],
            status: contested_with
                .map(|with| ClaimStatus::Contested { with })
                .unwrap_or(ClaimStatus::Active),
            sensitivity: input.sensitivity,
            retention: input.retention,
            created_at_ms: input.observed_at_ms,
            updated_at_ms: input.observed_at_ms,
        };
        let observation = Observation {
            id: observation_id,
            claim_id,
            value,
            source: input.source,
            observed_at_ms: input.observed_at_ms,
        };
        let event = CompanionEvent::ClaimRecorded {
            claim,
            observation,
            retired_expired_claim_id,
        };
        let emitted_charges = contested_with
            .and_then(|existing_id| self.claims.get(&existing_id))
            .map(|existing| {
                contradiction_charge(&key, existing.confidence_bps, input.confidence_bps)
            })
            .into_iter()
            .collect();
        let version = self.apply_event(expected_version, event.clone())?;

        Ok(CompanionTransition {
            version,
            claim_id: Some(claim_id),
            event,
            emitted_charges,
        })
    }

    pub fn correct_claim(
        &mut self,
        expected_version: u64,
        previous_claim_id: ClaimId,
        input: CorrectionInput,
    ) -> Result<CompanionTransition, CompanionError> {
        self.require_version(expected_version)?;
        validate_confidence(input.confidence_bps)?;
        let value = normalized_value(&input.value).ok_or(CompanionError::EmptyValue)?;
        let previous = self
            .claims
            .get(&previous_claim_id)
            .ok_or(CompanionError::ClaimNotFound(previous_claim_id))?;
        if self.active_by_key.get(&previous.key) != Some(&previous_claim_id) {
            return Err(CompanionError::ClaimNotActive(previous_claim_id));
        }

        let replacement_id = self.next_claim_id;
        let observation_id = self.next_observation_id;
        let source = ClaimSource::UserCorrection {
            corrects: previous_claim_id,
        };
        let replacement = CompanionClaim {
            id: replacement_id,
            key: previous.key.clone(),
            value: value.clone(),
            confidence_bps: input.confidence_bps,
            source: source.clone(),
            supporting_observation_ids: vec![observation_id],
            status: ClaimStatus::Active,
            sensitivity: input.sensitivity,
            retention: input.retention,
            created_at_ms: input.observed_at_ms,
            updated_at_ms: input.observed_at_ms,
        };
        let observation = Observation {
            id: observation_id,
            claim_id: replacement_id,
            value,
            source,
            observed_at_ms: input.observed_at_ms,
        };
        let event = CompanionEvent::ClaimCorrected {
            previous_claim_id,
            replacement,
            observation,
        };
        let version = self.apply_event(expected_version, event.clone())?;

        Ok(CompanionTransition {
            version,
            claim_id: Some(replacement_id),
            event,
            emitted_charges: Vec::new(),
        })
    }

    pub fn invalidate_claim(
        &mut self,
        expected_version: u64,
        claim_id: ClaimId,
        reason: impl Into<String>,
        occurred_at_ms: u64,
    ) -> Result<CompanionTransition, CompanionError> {
        let reason = reason.into().trim().to_owned();
        if reason.is_empty() {
            return Err(CompanionError::EmptyReason);
        }
        let event = CompanionEvent::ClaimInvalidated {
            claim_id,
            reason,
            occurred_at_ms,
        };
        let version = self.apply_event(expected_version, event.clone())?;
        Ok(CompanionTransition {
            version,
            claim_id: Some(claim_id),
            event,
            emitted_charges: Vec::new(),
        })
    }

    pub fn delete_claim(
        &mut self,
        expected_version: u64,
        claim_id: ClaimId,
        occurred_at_ms: u64,
    ) -> Result<CompanionTransition, CompanionError> {
        let event = CompanionEvent::ClaimDeleted {
            claim_id,
            occurred_at_ms,
        };
        let version = self.apply_event(expected_version, event.clone())?;
        Ok(CompanionTransition {
            version,
            claim_id: None,
            event,
            emitted_charges: Vec::new(),
        })
    }

    pub fn apply_event(
        &mut self,
        expected_version: u64,
        event: CompanionEvent,
    ) -> Result<u64, CompanionError> {
        self.require_version(expected_version)?;

        match &event {
            CompanionEvent::ClaimRecorded {
                claim,
                observation,
                retired_expired_claim_id,
            } => {
                self.validate_new_claim_event(claim, observation)?;
                if let Some(expired_id) = retired_expired_claim_id {
                    self.retire_expired_active(*expired_id, &claim.key, claim.created_at_ms)?;
                }
                match &claim.status {
                    ClaimStatus::Active => {
                        if self.active_by_key.contains_key(&claim.key) {
                            return Err(CompanionError::ActiveClaimAlreadyExists(
                                claim.key.clone(),
                            ));
                        }
                        self.active_by_key.insert(claim.key.clone(), claim.id);
                    }
                    ClaimStatus::Contested { with } => {
                        if self.active_by_key.get(&claim.key) != Some(with) {
                            return Err(CompanionError::InvalidContestation {
                                claim_id: claim.id,
                                active_id: *with,
                            });
                        }
                    }
                    _ => return Err(CompanionError::InvalidInitialStatus),
                }
                self.observations.insert(observation.id, observation.clone());
                self.claims.insert(claim.id, claim.clone());
                self.advance_ids(claim.id, observation.id)?;
            }
            CompanionEvent::ObservationAttached {
                claim_id,
                observation,
                confidence_bps,
                updated_at_ms,
            } => {
                validate_confidence(*confidence_bps)?;
                if observation.claim_id != *claim_id {
                    return Err(CompanionError::ObservationClaimMismatch {
                        observation_id: observation.id,
                        claim_id: *claim_id,
                    });
                }
                if self.observations.contains_key(&observation.id) {
                    return Err(CompanionError::DuplicateObservationId(observation.id));
                }
                let claim = self
                    .claims
                    .get_mut(claim_id)
                    .ok_or(CompanionError::ClaimNotFound(*claim_id))?;
                claim.supporting_observation_ids.push(observation.id);
                claim.confidence_bps = claim.confidence_bps.max(*confidence_bps);
                claim.updated_at_ms = claim.updated_at_ms.max(*updated_at_ms);
                self.observations.insert(observation.id, observation.clone());
                let next = observation
                    .id
                    .checked_add(1)
                    .ok_or(CompanionError::IdentifierOverflow)?;
                self.next_observation_id = self.next_observation_id.max(next);
            }
            CompanionEvent::ClaimCorrected {
                previous_claim_id,
                replacement,
                observation,
            } => {
                self.validate_new_claim_event(replacement, observation)?;
                let previous = self
                    .claims
                    .get_mut(previous_claim_id)
                    .ok_or(CompanionError::ClaimNotFound(*previous_claim_id))?;
                if previous.key != replacement.key
                    || self.active_by_key.get(&previous.key) != Some(previous_claim_id)
                {
                    return Err(CompanionError::ClaimNotActive(*previous_claim_id));
                }
                if replacement.status != ClaimStatus::Active {
                    return Err(CompanionError::InvalidReplacementStatus);
                }
                previous.status = ClaimStatus::Superseded { by: replacement.id };
                previous.updated_at_ms = previous.updated_at_ms.max(replacement.created_at_ms);
                self.active_by_key
                    .insert(replacement.key.clone(), replacement.id);
                self.observations.insert(observation.id, observation.clone());
                self.claims.insert(replacement.id, replacement.clone());
                self.advance_ids(replacement.id, observation.id)?;
            }
            CompanionEvent::ClaimInvalidated {
                claim_id,
                reason,
                occurred_at_ms,
            } => {
                if reason.trim().is_empty() {
                    return Err(CompanionError::EmptyReason);
                }
                let claim = self
                    .claims
                    .get_mut(claim_id)
                    .ok_or(CompanionError::ClaimNotFound(*claim_id))?;
                claim.status = ClaimStatus::Invalidated {
                    reason: reason.clone(),
                };
                claim.updated_at_ms = claim.updated_at_ms.max(*occurred_at_ms);
                if self.active_by_key.get(&claim.key) == Some(claim_id) {
                    self.active_by_key.remove(&claim.key);
                }
            }
            CompanionEvent::ClaimDeleted {
                claim_id,
                occurred_at_ms: _,
            } => {
                let claim = self
                    .claims
                    .remove(claim_id)
                    .ok_or(CompanionError::ClaimNotFound(*claim_id))?;
                if self.active_by_key.get(&claim.key) == Some(claim_id) {
                    self.active_by_key.remove(&claim.key);
                }
                self.observations
                    .retain(|_, observation| observation.claim_id != *claim_id);
                for other in self.claims.values_mut() {
                    let contested_deleted = matches!(
                        &other.status,
                        ClaimStatus::Contested { with } if *with == *claim_id
                    );
                    if contested_deleted {
                        other.status = ClaimStatus::Invalidated {
                            reason: CONTESTED_PARENT_DELETED_REASON.to_owned(),
                        };
                    }
                }
            }
        }

        self.version = self
            .version
            .checked_add(1)
            .ok_or(CompanionError::VersionOverflow)?;
        Ok(self.version)
    }

    fn retire_expired_active(
        &mut self,
        claim_id: ClaimId,
        replacement_key: &str,
        replacement_at_ms: u64,
    ) -> Result<(), CompanionError> {
        let claim = self
            .claims
            .get_mut(&claim_id)
            .ok_or(CompanionError::ClaimNotFound(claim_id))?;
        if claim.key != replacement_key
            || self.active_by_key.get(replacement_key) != Some(&claim_id)
            || !claim.retention.is_expired(replacement_at_ms)
        {
            return Err(CompanionError::InvalidExpiredRetirement(claim_id));
        }
        claim.status = ClaimStatus::Invalidated {
            reason: RETENTION_EXPIRED_REASON.to_owned(),
        };
        claim.updated_at_ms = claim.updated_at_ms.max(replacement_at_ms);
        self.active_by_key.remove(replacement_key);
        Ok(())
    }

    fn validate_new_claim_event(
        &self,
        claim: &CompanionClaim,
        observation: &Observation,
    ) -> Result<(), CompanionError> {
        validate_confidence(claim.confidence_bps)?;
        if canonical_key(&claim.key).as_deref() != Some(claim.key.as_str()) {
            return Err(CompanionError::NonCanonicalKey(claim.key.clone()));
        }
        if normalized_value(&claim.value).is_none() {
            return Err(CompanionError::EmptyValue);
        }
        if self.claims.contains_key(&claim.id) {
            return Err(CompanionError::DuplicateClaimId(claim.id));
        }
        if self.observations.contains_key(&observation.id) {
            return Err(CompanionError::DuplicateObservationId(observation.id));
        }
        if observation.claim_id != claim.id {
            return Err(CompanionError::ObservationClaimMismatch {
                observation_id: observation.id,
                claim_id: claim.id,
            });
        }
        if claim.supporting_observation_ids.as_slice() != [observation.id] {
            return Err(CompanionError::InvalidSupportingObservationSet(claim.id));
        }
        Ok(())
    }

    fn advance_ids(
        &mut self,
        claim_id: ClaimId,
        observation_id: ObservationId,
    ) -> Result<(), CompanionError> {
        let next_claim = claim_id
            .checked_add(1)
            .ok_or(CompanionError::IdentifierOverflow)?;
        let next_observation = observation_id
            .checked_add(1)
            .ok_or(CompanionError::IdentifierOverflow)?;
        self.next_claim_id = self.next_claim_id.max(next_claim);
        self.next_observation_id = self.next_observation_id.max(next_observation);
        Ok(())
    }

    fn require_version(&self, expected_version: u64) -> Result<(), CompanionError> {
        if self.version != expected_version {
            return Err(CompanionError::VersionConflict {
                expected: expected_version,
                actual: self.version,
            });
        }
        Ok(())
    }
}

fn validate_confidence(confidence_bps: u16) -> Result<(), CompanionError> {
    if confidence_bps > 10_000 {
        return Err(CompanionError::ConfidenceOutOfRange(confidence_bps));
    }
    Ok(())
}

fn canonical_key(value: &str) -> Option<String> {
    let normalized = value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

fn normalized_value(value: &str) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

fn equivalent_value(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn contradiction_charge(
    key: &str,
    existing_confidence_bps: u16,
    candidate_confidence_bps: u16,
) -> Charge {
    let existing = f32::from(existing_confidence_bps) / 10_000.0;
    let candidate = f32::from(candidate_confidence_bps) / 10_000.0;
    Charge::new(
        ChargeKind::Contradiction,
        vec![existing, candidate],
        ((existing + candidate) / 2.0).max(0.05),
        ChargeScope::Custom(format!("companion.claim:{key}")),
    )
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CompanionError {
    #[error("companion state version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: u64, actual: u64 },
    #[error("claim key cannot be empty")]
    EmptyKey,
    #[error("claim value cannot be empty")]
    EmptyValue,
    #[error("reason cannot be empty")]
    EmptyReason,
    #[error("confidence {0} exceeds 10000 basis points")]
    ConfidenceOutOfRange(u16),
    #[error("claim {0} was not found")]
    ClaimNotFound(ClaimId),
    #[error("claim {0} is not the active claim for its key")]
    ClaimNotActive(ClaimId),
    #[error("active index references missing claim {0}")]
    BrokenActiveIndex(ClaimId),
    #[error("claim id {0} already exists")]
    DuplicateClaimId(ClaimId),
    #[error("observation id {0} already exists")]
    DuplicateObservationId(ObservationId),
    #[error("observation {observation_id} does not belong to claim {claim_id}")]
    ObservationClaimMismatch {
        observation_id: ObservationId,
        claim_id: ClaimId,
    },
    #[error("claim {0} does not contain exactly its initial supporting observation")]
    InvalidSupportingObservationSet(ClaimId),
    #[error("claim key is not canonical: {0}")]
    NonCanonicalKey(String),
    #[error("an active claim already exists for key {0}")]
    ActiveClaimAlreadyExists(String),
    #[error("claim {claim_id} contests {active_id}, but that is not the active claim")]
    InvalidContestation {
        claim_id: ClaimId,
        active_id: ClaimId,
    },
    #[error("new claims must begin active or contested")]
    InvalidInitialStatus,
    #[error("replacement claim must be active")]
    InvalidReplacementStatus,
    #[error("claim {0} was not a matching expired active claim")]
    InvalidExpiredRetirement(ClaimId),
    #[error("companion state version overflow")]
    VersionOverflow,
    #[error("companion identifier overflow")]
    IdentifierOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user_claim(key: &str, value: &str, at: u64) -> ClaimInput {
        ClaimInput {
            key: key.to_owned(),
            value: value.to_owned(),
            source: ClaimSource::UserStatement,
            confidence_bps: 10_000,
            sensitivity: Sensitivity::Personal,
            retention: Retention::Durable,
            observed_at_ms: at,
        }
    }

    #[test]
    fn equivalent_observation_attaches_to_active_claim() {
        let mut state = CompanionState::new();
        let first = state
            .record_claim(0, user_claim("Preferred Language", "Rust", 10))
            .unwrap();
        let second = state
            .record_claim(
                first.version,
                ClaimInput {
                    key: "preferred   language".to_owned(),
                    value: "rust".to_owned(),
                    source: ClaimSource::Inference {
                        method: "conversation-pattern".to_owned(),
                    },
                    confidence_bps: 7_000,
                    sensitivity: Sensitivity::Personal,
                    retention: Retention::Durable,
                    observed_at_ms: 11,
                },
            )
            .unwrap();

        assert_eq!(first.claim_id, second.claim_id);
        assert!(second.emitted_charges.is_empty());
        let claim = state
            .active_claim("preferred language", 11, true)
            .unwrap();
        assert_eq!(claim.supporting_observation_ids.len(), 2);
        assert_eq!(state.observations().len(), 2);
    }

    #[test]
    fn conflicting_inference_is_contested_and_emits_charge() {
        let mut state = CompanionState::new();
        let first = state
            .record_claim(0, user_claim("response style", "direct", 10))
            .unwrap();
        let conflict = state
            .record_claim(
                first.version,
                ClaimInput {
                    key: "response style".to_owned(),
                    value: "verbose".to_owned(),
                    source: ClaimSource::Inference {
                        method: "style-classifier".to_owned(),
                    },
                    confidence_bps: 8_000,
                    sensitivity: Sensitivity::Personal,
                    retention: Retention::Durable,
                    observed_at_ms: 12,
                },
            )
            .unwrap();

        assert_eq!(conflict.emitted_charges.len(), 1);
        assert_eq!(conflict.emitted_charges[0].kind, ChargeKind::Contradiction);
        let candidate = state.claim(conflict.claim_id.unwrap()).unwrap();
        assert_eq!(
            &candidate.status,
            &ClaimStatus::Contested {
                with: first.claim_id.unwrap()
            }
        );
        assert_eq!(
            state.active_claim("response style", 12, true).unwrap().value,
            "direct"
        );
    }

    #[test]
    fn expired_active_claim_is_retired_before_new_claim_is_recorded() {
        let mut state = CompanionState::new();
        let first = state
            .record_claim(
                0,
                ClaimInput {
                    key: "temporary preference".to_owned(),
                    value: "old".to_owned(),
                    source: ClaimSource::UserStatement,
                    confidence_bps: 10_000,
                    sensitivity: Sensitivity::Personal,
                    retention: Retention::Until { expires_at_ms: 20 },
                    observed_at_ms: 10,
                },
            )
            .unwrap();
        let replacement = state
            .record_claim(
                first.version,
                user_claim("temporary preference", "new", 21),
            )
            .unwrap();

        assert!(replacement.emitted_charges.is_empty());
        assert_eq!(
            &state.claim(first.claim_id.unwrap()).unwrap().status,
            &ClaimStatus::Invalidated {
                reason: RETENTION_EXPIRED_REASON.to_owned()
            }
        );
        let active = state
            .active_claim("temporary preference", 21, true)
            .unwrap();
        assert_eq!(active.id, replacement.claim_id.unwrap());
        assert_eq!(active.value, "new");
    }

    #[test]
    fn explicit_user_correction_supersedes_active_claim() {
        let mut state = CompanionState::new();
        let first = state
            .record_claim(0, user_claim("coding language", "Python", 10))
            .unwrap();
        let corrected = state
            .correct_claim(
                first.version,
                first.claim_id.unwrap(),
                CorrectionInput {
                    value: "Rust".to_owned(),
                    confidence_bps: 10_000,
                    sensitivity: Sensitivity::Personal,
                    retention: Retention::Durable,
                    observed_at_ms: 20,
                },
            )
            .unwrap();

        assert_eq!(
            &state.claim(first.claim_id.unwrap()).unwrap().status,
            &ClaimStatus::Superseded {
                by: corrected.claim_id.unwrap()
            }
        );
        let active = state.active_claim("coding language", 20, true).unwrap();
        assert_eq!(active.id, corrected.claim_id.unwrap());
        assert_eq!(active.value, "Rust");
        assert!(matches!(
            &active.source,
            ClaimSource::UserCorrection { .. }
        ));
    }

    #[test]
    fn deletion_removes_claim_and_supporting_observations() {
        let mut state = CompanionState::new();
        let recorded = state
            .record_claim(0, user_claim("private note", "remove me", 10))
            .unwrap();
        let claim_id = recorded.claim_id.unwrap();
        let deleted = state.delete_claim(recorded.version, claim_id, 20).unwrap();

        assert_eq!(deleted.version, 2);
        assert!(state.claim(claim_id).is_none());
        assert!(state.observations().is_empty());
        assert!(state.active_claim("private note", 20, true).is_none());
    }

    #[test]
    fn optimistic_version_conflict_does_not_mutate_state() {
        let mut state = CompanionState::new();
        let recorded = state
            .record_claim(0, user_claim("topic", "value", 10))
            .unwrap();
        let before = state.clone();

        let error = state
            .invalidate_claim(0, recorded.claim_id.unwrap(), "stale write", 11)
            .unwrap_err();

        assert_eq!(
            error,
            CompanionError::VersionConflict {
                expected: 0,
                actual: 1
            }
        );
        assert_eq!(state, before);
    }

    #[test]
    fn event_replay_is_deterministic() {
        let mut source = CompanionState::new();
        let first = source
            .record_claim(
                0,
                ClaimInput {
                    key: "editor".to_owned(),
                    value: "vim".to_owned(),
                    source: ClaimSource::UserStatement,
                    confidence_bps: 10_000,
                    sensitivity: Sensitivity::Personal,
                    retention: Retention::Until { expires_at_ms: 12 },
                    observed_at_ms: 10,
                },
            )
            .unwrap();
        let second = source
            .record_claim(first.version, user_claim("editor", "helix", 15))
            .unwrap();

        let mut replay = CompanionState::new();
        replay.apply_event(0, first.event).unwrap();
        replay.apply_event(1, second.event).unwrap();

        assert_eq!(replay, source);
    }

    #[test]
    fn sensitive_and_expired_claims_are_hidden_by_query_policy() {
        let mut state = CompanionState::new();
        state
            .record_claim(
                0,
                ClaimInput {
                    key: "secret".to_owned(),
                    value: "value".to_owned(),
                    source: ClaimSource::UserStatement,
                    confidence_bps: 10_000,
                    sensitivity: Sensitivity::Sensitive,
                    retention: Retention::Until { expires_at_ms: 20 },
                    observed_at_ms: 10,
                },
            )
            .unwrap();

        assert!(state.active_claim("secret", 15, false).is_none());
        assert!(state.active_claim("secret", 15, true).is_some());
        assert!(state.active_claim("secret", 20, true).is_none());
    }
}
