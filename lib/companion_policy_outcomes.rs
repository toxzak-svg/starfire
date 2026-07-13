//! S5-B independently witnessed interaction-policy outcomes.
//!
//! This module resolves S5-A policy predictions only from delayed independent
//! witnesses. Split assignment is derived from pre-outcome metadata, every
//! non-abstaining arm must be covered exactly once, and collection is atomic.
//! No result can alter generated text, routing, beliefs, persistence, or actions.

use crate::companion_interaction_policy::{PolicyVariant, ShadowPolicyEnrollment};
use crate::companion_prediction_ledger::{
    BrierScore, OutcomeWitness, PredictionEvent, PredictionId, PredictionLedger,
    PredictionLedgerError, PredictionRecord, PredictionStatus, PredictionTransition, WitnessSource,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvaluationSplit {
    Development,
    SubjectHoldout,
    TemporalHoldout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyOutcomeSplitPlan {
    pub temporal_holdout_start_ms: u64,
    pub subject_holdout_modulus: u64,
    pub subject_holdout_remainder: u64,
}

impl PolicyOutcomeSplitPlan {
    pub fn new(
        temporal_holdout_start_ms: u64,
        subject_holdout_modulus: u64,
        subject_holdout_remainder: u64,
    ) -> Result<Self, PolicyOutcomeError> {
        if temporal_holdout_start_ms == 0 {
            return Err(PolicyOutcomeError::InvalidTemporalBoundary);
        }
        if subject_holdout_modulus < 2 {
            return Err(PolicyOutcomeError::InvalidSubjectHoldoutModulus);
        }
        if subject_holdout_remainder >= subject_holdout_modulus {
            return Err(PolicyOutcomeError::InvalidSubjectHoldoutRemainder {
                remainder: subject_holdout_remainder,
                modulus: subject_holdout_modulus,
            });
        }
        Ok(Self {
            temporal_holdout_start_ms,
            subject_holdout_modulus,
            subject_holdout_remainder,
        })
    }

    #[must_use]
    pub fn classify(&self, enrollment: &ShadowPolicyEnrollment) -> EvaluationSplit {
        let context = &enrollment.batch.context;
        if context.issued_at_ms >= self.temporal_holdout_start_ms {
            EvaluationSplit::TemporalHoldout
        } else if context.subject_scope_digest % self.subject_holdout_modulus
            == self.subject_holdout_remainder
        {
            EvaluationSplit::SubjectHoldout
        } else {
            EvaluationSplit::Development
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyOutcomeCollectorConfig {
    pub max_corrections: u16,
    pub max_clarification_turns: u16,
    pub max_compute_micros: u64,
}

impl Default for PolicyOutcomeCollectorConfig {
    fn default() -> Self {
        Self {
            max_corrections: 32,
            max_clarification_turns: 32,
            max_compute_micros: 60_000_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyOutcomeMetrics {
    pub correction_count: u16,
    pub clarification_turns: u16,
    pub completion_bps: u16,
    pub compute_micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyOutcomeObservation {
    pub prediction_id: PredictionId,
    pub variant: PolicyVariant,
    pub source: WitnessSource,
    pub witness_id: String,
    pub label: String,
    pub observed_at_ms: u64,
    pub evidence_digest: u64,
    pub metrics: PolicyOutcomeMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectedPolicyOutcome {
    pub prediction_id: PredictionId,
    pub variant: PolicyVariant,
    pub split: EvaluationSplit,
    pub witness_id: String,
    pub witness: OutcomeWitness,
    pub score: BrierScore,
    pub metrics: PolicyOutcomeMetrics,
    pub subject_scope: String,
    pub producer_id: String,
    pub context_digest: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyOutcomeCollection {
    pub split: EvaluationSplit,
    pub ledger_version_before: u64,
    pub ledger_version_after: u64,
    pub outcomes: Vec<CollectedPolicyOutcome>,
    pub transitions: Vec<PredictionTransition>,
    pub abstention_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct PolicyOutcomeCollector {
    split_plan: PolicyOutcomeSplitPlan,
    config: PolicyOutcomeCollectorConfig,
}

impl PolicyOutcomeCollector {
    #[must_use]
    pub fn new(
        split_plan: PolicyOutcomeSplitPlan,
        config: PolicyOutcomeCollectorConfig,
    ) -> Self {
        Self { split_plan, config }
    }

    #[must_use]
    pub fn split_plan(&self) -> PolicyOutcomeSplitPlan {
        self.split_plan
    }

    #[must_use]
    pub fn config(&self) -> PolicyOutcomeCollectorConfig {
        self.config
    }

    pub fn collect(
        &self,
        ledger: &mut PredictionLedger,
        expected_ledger_version: u64,
        enrollment: &ShadowPolicyEnrollment,
        observations: Vec<PolicyOutcomeObservation>,
    ) -> Result<PolicyOutcomeCollection, PolicyOutcomeError> {
        if ledger.version != expected_ledger_version {
            return Err(PolicyOutcomeError::LedgerVersionConflict {
                expected: expected_ledger_version,
                actual: ledger.version,
            });
        }

        let expected = expected_predictions(enrollment)?;
        let expected_ids = expected
            .iter()
            .map(|(_, prediction_id, _)| *prediction_id)
            .collect::<BTreeSet<_>>();
        let mut by_prediction = BTreeMap::new();
        let mut evidence_digests = BTreeSet::new();

        for observation in observations {
            if !expected_ids.contains(&observation.prediction_id) {
                return Err(PolicyOutcomeError::UnknownObservationPrediction(
                    observation.prediction_id,
                ));
            }
            if by_prediction
                .insert(observation.prediction_id, observation.clone())
                .is_some()
            {
                return Err(PolicyOutcomeError::DuplicateObservation(
                    observation.prediction_id,
                ));
            }
            if observation.evidence_digest == 0 {
                return Err(PolicyOutcomeError::EmptyEvidenceDigest(
                    observation.prediction_id,
                ));
            }
            if !evidence_digests.insert(observation.evidence_digest) {
                return Err(PolicyOutcomeError::DuplicateEvidenceDigest(
                    observation.evidence_digest,
                ));
            }
        }

        for (_, prediction_id, _) in &expected {
            if !by_prediction.contains_key(prediction_id) {
                return Err(PolicyOutcomeError::MissingObservation(*prediction_id));
            }
        }

        let split = self.split_plan.classify(enrollment);
        let ledger_version_before = ledger.version;
        let mut working = ledger.clone();
        let mut outcomes = Vec::with_capacity(expected.len());
        let mut transitions = Vec::with_capacity(expected.len());

        for (variant, prediction_id, enrolled_prediction) in expected {
            let observation = by_prediction
                .remove(&prediction_id)
                .ok_or(PolicyOutcomeError::MissingObservation(prediction_id))?;
            if observation.variant != variant {
                return Err(PolicyOutcomeError::VariantMismatch {
                    prediction_id,
                    expected: variant,
                    actual: observation.variant,
                });
            }
            validate_observation(&self.config, &observation, &enrolled_prediction)?;

            let current = working
                .prediction(prediction_id)
                .ok_or(PolicyOutcomeError::MissingLedgerPrediction(prediction_id))?;
            if current != &enrolled_prediction {
                return Err(PolicyOutcomeError::LedgerRecordMismatch(prediction_id));
            }
            if !matches!(current.status, PredictionStatus::Pending) {
                return Err(PolicyOutcomeError::PredictionNotPending(prediction_id));
            }

            let transition = working.resolve(
                working.version,
                prediction_id,
                OutcomeWitness {
                    source: observation.source,
                    label: observation.label.clone(),
                    observed_at_ms: observation.observed_at_ms,
                    evidence_digest: observation.evidence_digest,
                },
            )?;
            let PredictionEvent::Resolved { witness, score, .. } = &transition.event else {
                return Err(PolicyOutcomeError::UnexpectedResolutionEvent(prediction_id));
            };
            outcomes.push(CollectedPolicyOutcome {
                prediction_id,
                variant,
                split,
                witness_id: normalize_identifier(&observation.witness_id)
                    .ok_or(PolicyOutcomeError::EmptyWitnessId(prediction_id))?,
                witness: witness.clone(),
                score: *score,
                metrics: observation.metrics,
                subject_scope: enrolled_prediction.subject_scope.clone(),
                producer_id: enrolled_prediction.producer.id.clone(),
                context_digest: enrolled_prediction.context_digest,
            });
            transitions.push(transition);
        }

        if let Some(prediction_id) = by_prediction.keys().next().copied() {
            return Err(PolicyOutcomeError::UnknownObservationPrediction(prediction_id));
        }

        let ledger_version_after = working.version;
        *ledger = working;
        Ok(PolicyOutcomeCollection {
            split,
            ledger_version_before,
            ledger_version_after,
            outcomes,
            transitions,
            abstention_ids: enrollment.abstention_ids.clone(),
        })
    }
}

fn expected_predictions(
    enrollment: &ShadowPolicyEnrollment,
) -> Result<Vec<(PolicyVariant, PredictionId, PredictionRecord)>, PolicyOutcomeError> {
    let variants = enrollment
        .batch
        .proposals
        .iter()
        .filter(|proposal| !proposal.is_abstention())
        .map(|proposal| proposal.variant)
        .collect::<Vec<_>>();
    if variants.len() != enrollment.prediction_ids.len() {
        return Err(PolicyOutcomeError::EnrollmentPredictionCountMismatch {
            proposals: variants.len(),
            prediction_ids: enrollment.prediction_ids.len(),
        });
    }

    variants
        .into_iter()
        .zip(enrollment.prediction_ids.iter().copied())
        .map(|(variant, prediction_id)| {
            let transition = enrollment
                .transitions
                .iter()
                .find(|transition| transition.prediction_id == Some(prediction_id))
                .ok_or(PolicyOutcomeError::EnrollmentTransitionMissing(prediction_id))?;
            let PredictionEvent::Issued { prediction } = &transition.event else {
                return Err(PolicyOutcomeError::EnrollmentTransitionNotIssued(
                    prediction_id,
                ));
            };
            let expected_producer = format!("s5a-{}-v1", variant.id());
            let expected_scope_suffix = format!("/{}", variant.id());
            if prediction.producer.id != expected_producer
                || !prediction.subject_scope.ends_with(&expected_scope_suffix)
            {
                return Err(PolicyOutcomeError::EnrollmentArmMismatch(prediction_id));
            }
            Ok((variant, prediction_id, prediction.clone()))
        })
        .collect()
}

fn validate_observation(
    config: &PolicyOutcomeCollectorConfig,
    observation: &PolicyOutcomeObservation,
    prediction: &PredictionRecord,
) -> Result<(), PolicyOutcomeError> {
    if observation.source == WitnessSource::ResponseGenerator {
        return Err(PolicyOutcomeError::SelfGradingWitness(
            observation.prediction_id,
        ));
    }
    let witness_id = normalize_identifier(&observation.witness_id)
        .ok_or(PolicyOutcomeError::EmptyWitnessId(observation.prediction_id))?;
    if witness_id == prediction.producer.id {
        return Err(PolicyOutcomeError::NonIndependentWitness(
            observation.prediction_id,
        ));
    }
    if observation.metrics.correction_count > config.max_corrections {
        return Err(PolicyOutcomeError::CorrectionCountOutOfRange {
            prediction_id: observation.prediction_id,
            actual: observation.metrics.correction_count,
            maximum: config.max_corrections,
        });
    }
    if observation.metrics.clarification_turns > config.max_clarification_turns {
        return Err(PolicyOutcomeError::ClarificationTurnsOutOfRange {
            prediction_id: observation.prediction_id,
            actual: observation.metrics.clarification_turns,
            maximum: config.max_clarification_turns,
        });
    }
    if observation.metrics.completion_bps > 10_000 {
        return Err(PolicyOutcomeError::CompletionOutOfRange {
            prediction_id: observation.prediction_id,
            actual: observation.metrics.completion_bps,
        });
    }
    if observation.metrics.compute_micros > config.max_compute_micros {
        return Err(PolicyOutcomeError::ComputeOutOfRange {
            prediction_id: observation.prediction_id,
            actual: observation.metrics.compute_micros,
            maximum: config.max_compute_micros,
        });
    }
    Ok(())
}

fn normalize_identifier(raw: &str) -> Option<String> {
    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty() && normalized.len() <= 160).then_some(normalized)
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PolicyOutcomeError {
    #[error("temporal holdout boundary must be greater than zero")]
    InvalidTemporalBoundary,
    #[error("subject holdout modulus must be at least two")]
    InvalidSubjectHoldoutModulus,
    #[error("subject holdout remainder {remainder} must be less than modulus {modulus}")]
    InvalidSubjectHoldoutRemainder { remainder: u64, modulus: u64 },
    #[error("prediction ledger version conflict: expected {expected}, actual {actual}")]
    LedgerVersionConflict { expected: u64, actual: u64 },
    #[error("enrollment contains {proposals} non-abstaining proposals but {prediction_ids} prediction ids")]
    EnrollmentPredictionCountMismatch {
        proposals: usize,
        prediction_ids: usize,
    },
    #[error("enrollment transition missing for prediction {0}")]
    EnrollmentTransitionMissing(PredictionId),
    #[error("enrollment transition for prediction {0} is not an issue event")]
    EnrollmentTransitionNotIssued(PredictionId),
    #[error("enrollment arm metadata does not match prediction {0}")]
    EnrollmentArmMismatch(PredictionId),
    #[error("observation references prediction outside the enrollment: {0}")]
    UnknownObservationPrediction(PredictionId),
    #[error("duplicate observation for prediction {0}")]
    DuplicateObservation(PredictionId),
    #[error("missing observation for prediction {0}")]
    MissingObservation(PredictionId),
    #[error("observation for prediction {prediction_id} names variant {actual:?}, expected {expected:?}")]
    VariantMismatch {
        prediction_id: PredictionId,
        expected: PolicyVariant,
        actual: PolicyVariant,
    },
    #[error("response generator cannot witness prediction {0}")]
    SelfGradingWitness(PredictionId),
    #[error("witness id must be non-empty and bounded for prediction {0}")]
    EmptyWitnessId(PredictionId),
    #[error("witness is not independent from the producer for prediction {0}")]
    NonIndependentWitness(PredictionId),
    #[error("evidence digest must be non-zero for prediction {0}")]
    EmptyEvidenceDigest(PredictionId),
    #[error("evidence digest is reused across policy arms: {0}")]
    DuplicateEvidenceDigest(u64),
    #[error("correction count {actual} exceeds maximum {maximum} for prediction {prediction_id}")]
    CorrectionCountOutOfRange {
        prediction_id: PredictionId,
        actual: u16,
        maximum: u16,
    },
    #[error("clarification turns {actual} exceeds maximum {maximum} for prediction {prediction_id}")]
    ClarificationTurnsOutOfRange {
        prediction_id: PredictionId,
        actual: u16,
        maximum: u16,
    },
    #[error("completion {actual} bps exceeds 10000 for prediction {prediction_id}")]
    CompletionOutOfRange {
        prediction_id: PredictionId,
        actual: u16,
    },
    #[error("compute {actual} us exceeds maximum {maximum} for prediction {prediction_id}")]
    ComputeOutOfRange {
        prediction_id: PredictionId,
        actual: u64,
        maximum: u64,
    },
    #[error("ledger is missing enrolled prediction {0}")]
    MissingLedgerPrediction(PredictionId),
    #[error("ledger record differs from the frozen enrollment for prediction {0}")]
    LedgerRecordMismatch(PredictionId),
    #[error("prediction is no longer pending: {0}")]
    PredictionNotPending(PredictionId),
    #[error("ledger returned a non-resolution event for prediction {0}")]
    UnexpectedResolutionEvent(PredictionId),
    #[error(transparent)]
    PredictionLedger(#[from] PredictionLedgerError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion_interaction_policy::{PolicyContext, ShadowPolicyPlanner};
    use crate::companion_state::{
        ClaimInput, ClaimSource, CompanionState, Retention, Sensitivity,
    };

    fn state() -> CompanionState {
        let mut state = CompanionState::new();
        state
            .record_claim(
                0,
                ClaimInput {
                    key: "preference.detail.general".to_owned(),
                    value: "yes".to_owned(),
                    source: ClaimSource::UserStatement,
                    confidence_bps: 9_000,
                    sensitivity: Sensitivity::Personal,
                    retention: Retention::Session,
                    observed_at_ms: 10,
                },
            )
            .unwrap();
        state
    }

    fn context(issued_at_ms: u64, subject_scope_digest: u64) -> PolicyContext {
        PolicyContext {
            context_digest: issued_at_ms ^ subject_scope_digest,
            subject_scope_digest,
            domain: Some("rust".to_owned()),
            technical_context: true,
            asks_for_explanation: true,
            emotional_signal: false,
            issued_at_ms,
            not_before_ms: issued_at_ms + 100,
            expires_at_ms: issued_at_ms + 1_000,
        }
    }

    fn enrollment(
        issued_at_ms: u64,
        subject_scope_digest: u64,
    ) -> (PredictionLedger, ShadowPolicyEnrollment) {
        let state = state();
        let mut ledger = PredictionLedger::new();
        let enrollment = ShadowPolicyPlanner::default()
            .enroll(
                &state,
                &mut ledger,
                0,
                context(issued_at_ms, subject_scope_digest),
            )
            .unwrap();
        (ledger, enrollment)
    }

    fn observations(
        enrollment: &ShadowPolicyEnrollment,
        source: WitnessSource,
    ) -> Vec<PolicyOutcomeObservation> {
        enrollment
            .batch
            .proposals
            .iter()
            .filter(|proposal| !proposal.is_abstention())
            .zip(enrollment.prediction_ids.iter().copied())
            .map(|(proposal, prediction_id)| PolicyOutcomeObservation {
                prediction_id,
                variant: proposal.variant,
                source,
                witness_id: "independent-evaluator-v1".to_owned(),
                label: if proposal.variant == PolicyVariant::ScrambledScope {
                    "interaction_unsatisfactory"
                } else {
                    "interaction_satisfactory"
                }
                .to_owned(),
                observed_at_ms: enrollment.batch.context.not_before_ms + 1,
                evidence_digest: 10_000 + prediction_id,
                metrics: PolicyOutcomeMetrics {
                    correction_count: 0,
                    clarification_turns: 1,
                    completion_bps: 10_000,
                    compute_micros: 2_000 + prediction_id,
                },
            })
            .collect()
    }

    fn collector() -> PolicyOutcomeCollector {
        PolicyOutcomeCollector::new(
            PolicyOutcomeSplitPlan::new(50_000, 4, 1).unwrap(),
            PolicyOutcomeCollectorConfig::default(),
        )
    }

    #[test]
    fn split_assignment_is_pre_outcome_and_deterministic() {
        let (_, development) = enrollment(10_000, 8);
        let (_, subject_holdout) = enrollment(10_000, 9);
        let (_, temporal_holdout) = enrollment(50_000, 8);
        let plan = collector().split_plan();

        assert_eq!(plan.classify(&development), EvaluationSplit::Development);
        assert_eq!(
            plan.classify(&subject_holdout),
            EvaluationSplit::SubjectHoldout
        );
        assert_eq!(
            plan.classify(&temporal_holdout),
            EvaluationSplit::TemporalHoldout
        );
    }

    #[test]
    fn complete_independent_batch_resolves_atomically_and_replays() {
        let (mut ledger, enrollment) = enrollment(10_000, 8);
        let issue_events = enrollment
            .transitions
            .iter()
            .map(|transition| transition.event.clone())
            .collect::<Vec<_>>();
        let collection = collector()
            .collect(
                &mut ledger,
                enrollment.ledger_version_after,
                &enrollment,
                observations(&enrollment, WitnessSource::ExternalEvaluator),
            )
            .unwrap();

        assert_eq!(collection.split, EvaluationSplit::Development);
        assert_eq!(collection.outcomes.len(), 6);
        assert_eq!(ledger.summary().resolved, 6);
        assert_eq!(ledger.summary().pending, 0);
        assert!(collection
            .outcomes
            .iter()
            .all(|outcome| outcome.witness.source == WitnessSource::ExternalEvaluator));

        let events = issue_events
            .into_iter()
            .chain(
                collection
                    .transitions
                    .iter()
                    .map(|transition| transition.event.clone()),
            )
            .collect::<Vec<_>>();
        assert_eq!(PredictionLedger::replay(&events).unwrap(), ledger);
    }

    #[test]
    fn self_grading_and_incomplete_batches_leave_ledger_unchanged() {
        let (mut ledger, enrollment) = enrollment(10_000, 8);
        let before = ledger.clone();
        let self_graded = observations(&enrollment, WitnessSource::ResponseGenerator);
        let error = collector()
            .collect(
                &mut ledger,
                enrollment.ledger_version_after,
                &enrollment,
                self_graded,
            )
            .unwrap_err();
        assert!(matches!(error, PolicyOutcomeError::SelfGradingWitness(_)));
        assert_eq!(ledger, before);

        let mut incomplete = observations(&enrollment, WitnessSource::UserObservation);
        incomplete.pop();
        let error = collector()
            .collect(
                &mut ledger,
                enrollment.ledger_version_after,
                &enrollment,
                incomplete,
            )
            .unwrap_err();
        assert!(matches!(error, PolicyOutcomeError::MissingObservation(_)));
        assert_eq!(ledger, before);
    }

    #[test]
    fn reused_evidence_and_unbounded_metrics_are_rejected_atomically() {
        let (mut ledger, enrollment) = enrollment(10_000, 8);
        let before = ledger.clone();
        let mut duplicated = observations(&enrollment, WitnessSource::Environment);
        duplicated[1].evidence_digest = duplicated[0].evidence_digest;
        let error = collector()
            .collect(
                &mut ledger,
                enrollment.ledger_version_after,
                &enrollment,
                duplicated,
            )
            .unwrap_err();
        assert!(matches!(
            error,
            PolicyOutcomeError::DuplicateEvidenceDigest(_)
        ));
        assert_eq!(ledger, before);

        let mut excessive = observations(&enrollment, WitnessSource::Environment);
        excessive[0].metrics.completion_bps = 10_001;
        let error = collector()
            .collect(
                &mut ledger,
                enrollment.ledger_version_after,
                &enrollment,
                excessive,
            )
            .unwrap_err();
        assert!(matches!(
            error,
            PolicyOutcomeError::CompletionOutOfRange { .. }
        ));
        assert_eq!(ledger, before);
    }
}
