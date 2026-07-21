//! S5-B independent outcome collection for shadow interaction-policy evaluation.
//!
//! Direct user and environment evidence may resolve only the policy arm that was
//! actually delivered. Unshown arms remain pending unless an external evaluator
//! explicitly compares rendered alternatives. The response generator can never
//! witness or grade an outcome.

use crate::companion_interaction_policy::{
    PolicyVariant, ShadowPolicyEnrollment, ShadowPolicyProposal,
};
use crate::companion_prediction_ledger::{
    AbstentionInput, OutcomeWitness, PredictionEvent, PredictionId, PredictionInput,
    PredictionLedger, PredictionLedgerError, PredictionStatus, PredictionTransition, WitnessSource,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub type InteractionTrialId = u64;

const SUCCESS_LABEL: &str = "interaction_satisfactory";
const FAILURE_LABEL: &str = "interaction_unsatisfactory";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservedSignal {
    ExplicitPositiveRating,
    ExplicitNegativeRating,
    UserCorrection,
    ClarificationRequest,
    TaskCompleted,
    Abandoned,
    NeutralFollowUp,
}

impl ObservedSignal {
    #[must_use]
    pub const fn outcome_label(self) -> Option<&'static str> {
        match self {
            Self::ExplicitPositiveRating | Self::TaskCompleted => Some(SUCCESS_LABEL),
            Self::ExplicitNegativeRating
            | Self::UserCorrection
            | Self::ClarificationRequest
            | Self::Abandoned => Some(FAILURE_LABEL),
            Self::NeutralFollowUp => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedOutcomeEvidence {
    pub signal: ObservedSignal,
    pub source: WitnessSource,
    pub observed_at_ms: u64,
    pub evidence_digest: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PairwisePreference {
    Left,
    Right,
    Tie,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairedEvaluationEvidence {
    pub left_variant: PolicyVariant,
    pub right_variant: PolicyVariant,
    pub preference: PairwisePreference,
    pub left_render_digest: u64,
    pub right_render_digest: u64,
    pub evaluator_digest: u64,
    pub observed_at_ms: u64,
    pub evidence_digest: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrialArm {
    pub variant: PolicyVariant,
    pub policy_digest_fnv1a64: u64,
    pub prediction_id: Option<PredictionId>,
    pub abstention_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionTrial {
    pub id: InteractionTrialId,
    pub source_companion_version: u64,
    pub context_digest: u64,
    pub subject_scope_digest: u64,
    pub issued_at_ms: u64,
    pub not_before_ms: u64,
    pub expires_at_ms: u64,
    pub delivered_variant: Option<PolicyVariant>,
    pub arms: Vec<TrialArm>,
}

impl InteractionTrial {
    #[must_use]
    pub fn arm(&self, variant: PolicyVariant) -> Option<&TrialArm> {
        self.arms.iter().find(|arm| arm.variant == variant)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionOutcomeEvent {
    TrialRegistered {
        trial: InteractionTrial,
        enrollment_transitions: Vec<PredictionTransition>,
    },
    ObservedSignalRecorded {
        trial_id: InteractionTrialId,
        delivered_variant: PolicyVariant,
        evidence: ObservedOutcomeEvidence,
        resolution: Option<PredictionTransition>,
    },
    PairedEvaluationRecorded {
        trial_id: InteractionTrialId,
        evidence: PairedEvaluationEvidence,
        resolutions: Vec<PredictionTransition>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionOutcomeTransition {
    pub version: u64,
    pub trial_id: InteractionTrialId,
    pub event: InteractionOutcomeEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionOutcomeSummary {
    pub trials: u64,
    pub observed_signals: u64,
    pub paired_evaluations: u64,
    pub conclusive_observed_signals: u64,
    pub inconclusive_observed_signals: u64,
    pub resolved_trial_predictions: u64,
    pub pending_trial_predictions: u64,
    pub abstained_trial_arms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionOutcomeLedger {
    pub version: u64,
    base_prediction_ledger: PredictionLedger,
    mirrored_prediction_ledger: PredictionLedger,
    trials: BTreeMap<InteractionTrialId, InteractionTrial>,
    events: Vec<InteractionOutcomeEvent>,
    next_trial_id: InteractionTrialId,
}

impl InteractionOutcomeLedger {
    #[must_use]
    pub fn new(base_prediction_ledger: &PredictionLedger) -> Self {
        Self {
            version: 0,
            base_prediction_ledger: base_prediction_ledger.clone(),
            mirrored_prediction_ledger: base_prediction_ledger.clone(),
            trials: BTreeMap::new(),
            events: Vec::new(),
            next_trial_id: 1,
        }
    }

    #[must_use]
    pub fn trials(&self) -> &BTreeMap<InteractionTrialId, InteractionTrial> {
        &self.trials
    }

    #[must_use]
    pub fn events(&self) -> &[InteractionOutcomeEvent] {
        &self.events
    }

    #[must_use]
    pub fn mirrored_prediction_ledger(&self) -> &PredictionLedger {
        &self.mirrored_prediction_ledger
    }

    pub fn register_enrollment(
        &mut self,
        expected_version: u64,
        prediction_ledger: &PredictionLedger,
        enrollment: &ShadowPolicyEnrollment,
        delivered_variant: Option<PolicyVariant>,
    ) -> Result<InteractionOutcomeTransition, InteractionOutcomeError> {
        self.require_version(expected_version)?;
        if enrollment.ledger_version_before != self.mirrored_prediction_ledger.version {
            return Err(InteractionOutcomeError::EnrollmentVersionMismatch {
                expected_before: self.mirrored_prediction_ledger.version,
                actual_before: enrollment.ledger_version_before,
            });
        }
        if enrollment.ledger_version_after != prediction_ledger.version {
            return Err(InteractionOutcomeError::ExternalLedgerVersionMismatch {
                expected: enrollment.ledger_version_after,
                actual: prediction_ledger.version,
            });
        }

        let trial = trial_from_enrollment(self.next_trial_id, enrollment, delivered_variant)?;
        let event = InteractionOutcomeEvent::TrialRegistered {
            trial: trial.clone(),
            enrollment_transitions: enrollment.transitions.clone(),
        };

        let mut working = self.clone();
        let transition = working.apply_event(event)?;
        if working.mirrored_prediction_ledger != *prediction_ledger {
            return Err(InteractionOutcomeError::PredictionLedgerDiverged);
        }
        *self = working;
        Ok(transition)
    }

    pub fn record_observed_signal(
        &mut self,
        expected_version: u64,
        prediction_ledger: &mut PredictionLedger,
        trial_id: InteractionTrialId,
        evidence: ObservedOutcomeEvidence,
    ) -> Result<InteractionOutcomeTransition, InteractionOutcomeError> {
        self.require_version(expected_version)?;
        self.require_external_ledger(prediction_ledger)?;
        validate_observed_evidence(&evidence)?;

        let trial = self
            .trials
            .get(&trial_id)
            .ok_or(InteractionOutcomeError::UnknownTrial(trial_id))?;
        let delivered_variant = trial
            .delivered_variant
            .ok_or(InteractionOutcomeError::NoDeliveredArm(trial_id))?;
        let arm = trial
            .arm(delivered_variant)
            .ok_or(InteractionOutcomeError::MissingArm {
                trial_id,
                variant: delivered_variant,
            })?;
        let prediction_id = arm
            .prediction_id
            .ok_or(InteractionOutcomeError::ArmAbstained {
                trial_id,
                variant: delivered_variant,
            })?;

        let mut working_prediction = prediction_ledger.clone();
        let resolution = if let Some(label) = evidence.signal.outcome_label() {
            Some(working_prediction.resolve(
                working_prediction.version,
                prediction_id,
                OutcomeWitness {
                    source: evidence.source,
                    label: label.to_owned(),
                    observed_at_ms: evidence.observed_at_ms,
                    evidence_digest: evidence.evidence_digest,
                },
            )?)
        } else {
            None
        };

        let event = InteractionOutcomeEvent::ObservedSignalRecorded {
            trial_id,
            delivered_variant,
            evidence,
            resolution,
        };
        let mut working_outcomes = self.clone();
        let transition = working_outcomes.apply_event(event)?;
        if working_outcomes.mirrored_prediction_ledger != working_prediction {
            return Err(InteractionOutcomeError::PredictionLedgerDiverged);
        }
        *self = working_outcomes;
        *prediction_ledger = working_prediction;
        Ok(transition)
    }

    pub fn record_paired_evaluation(
        &mut self,
        expected_version: u64,
        prediction_ledger: &mut PredictionLedger,
        trial_id: InteractionTrialId,
        evidence: PairedEvaluationEvidence,
    ) -> Result<InteractionOutcomeTransition, InteractionOutcomeError> {
        self.require_version(expected_version)?;
        self.require_external_ledger(prediction_ledger)?;
        validate_paired_evidence(&evidence)?;

        let trial = self
            .trials
            .get(&trial_id)
            .ok_or(InteractionOutcomeError::UnknownTrial(trial_id))?;
        let left = trial
            .arm(evidence.left_variant)
            .ok_or(InteractionOutcomeError::MissingArm {
                trial_id,
                variant: evidence.left_variant,
            })?;
        let right = trial
            .arm(evidence.right_variant)
            .ok_or(InteractionOutcomeError::MissingArm {
                trial_id,
                variant: evidence.right_variant,
            })?;
        let left_prediction = left
            .prediction_id
            .ok_or(InteractionOutcomeError::ArmAbstained {
                trial_id,
                variant: evidence.left_variant,
            })?;
        let right_prediction = right
            .prediction_id
            .ok_or(InteractionOutcomeError::ArmAbstained {
                trial_id,
                variant: evidence.right_variant,
            })?;

        let mut working_prediction = prediction_ledger.clone();
        let mut resolutions = Vec::new();
        match evidence.preference {
            PairwisePreference::Tie => {}
            PairwisePreference::Left | PairwisePreference::Right => {
                let left_label = if evidence.preference == PairwisePreference::Left {
                    SUCCESS_LABEL
                } else {
                    FAILURE_LABEL
                };
                let right_label = if evidence.preference == PairwisePreference::Right {
                    SUCCESS_LABEL
                } else {
                    FAILURE_LABEL
                };
                resolutions.push(working_prediction.resolve(
                    working_prediction.version,
                    left_prediction,
                    OutcomeWitness {
                        source: WitnessSource::ExternalEvaluator,
                        label: left_label.to_owned(),
                        observed_at_ms: evidence.observed_at_ms,
                        evidence_digest: paired_arm_digest(
                            evidence.evidence_digest,
                            evidence.left_render_digest,
                            0,
                        ),
                    },
                )?);
                resolutions.push(working_prediction.resolve(
                    working_prediction.version,
                    right_prediction,
                    OutcomeWitness {
                        source: WitnessSource::ExternalEvaluator,
                        label: right_label.to_owned(),
                        observed_at_ms: evidence.observed_at_ms,
                        evidence_digest: paired_arm_digest(
                            evidence.evidence_digest,
                            evidence.right_render_digest,
                            1,
                        ),
                    },
                )?);
            }
        }

        let event = InteractionOutcomeEvent::PairedEvaluationRecorded {
            trial_id,
            evidence,
            resolutions,
        };
        let mut working_outcomes = self.clone();
        let transition = working_outcomes.apply_event(event)?;
        if working_outcomes.mirrored_prediction_ledger != working_prediction {
            return Err(InteractionOutcomeError::PredictionLedgerDiverged);
        }
        *self = working_outcomes;
        *prediction_ledger = working_prediction;
        Ok(transition)
    }

    pub fn replay(
        base_prediction_ledger: &PredictionLedger,
        events: &[InteractionOutcomeEvent],
    ) -> Result<Self, InteractionOutcomeError> {
        let mut ledger = Self::new(base_prediction_ledger);
        for event in events {
            ledger.apply_event(event.clone())?;
        }
        Ok(ledger)
    }

    #[must_use]
    pub fn summary(&self) -> InteractionOutcomeSummary {
        let mut observed_signals = 0_u64;
        let mut paired_evaluations = 0_u64;
        let mut conclusive_observed_signals = 0_u64;
        let mut inconclusive_observed_signals = 0_u64;

        for event in &self.events {
            match event {
                InteractionOutcomeEvent::ObservedSignalRecorded { resolution, .. } => {
                    observed_signals += 1;
                    if resolution.is_some() {
                        conclusive_observed_signals += 1;
                    } else {
                        inconclusive_observed_signals += 1;
                    }
                }
                InteractionOutcomeEvent::PairedEvaluationRecorded { .. } => {
                    paired_evaluations += 1;
                }
                InteractionOutcomeEvent::TrialRegistered { .. } => {}
            }
        }

        let mut resolved_trial_predictions = 0_u64;
        let mut pending_trial_predictions = 0_u64;
        let mut abstained_trial_arms = 0_u64;
        for trial in self.trials.values() {
            for arm in &trial.arms {
                match arm.prediction_id.and_then(|id| {
                    self.mirrored_prediction_ledger.prediction(id)
                }) {
                    Some(prediction) => match prediction.status {
                        PredictionStatus::Pending => pending_trial_predictions += 1,
                        PredictionStatus::Resolved { .. } => resolved_trial_predictions += 1,
                        PredictionStatus::Expired { .. } => {}
                    },
                    None if arm.abstention_id.is_some() => abstained_trial_arms += 1,
                    None => {}
                }
            }
        }

        InteractionOutcomeSummary {
            trials: self.trials.len() as u64,
            observed_signals,
            paired_evaluations,
            conclusive_observed_signals,
            inconclusive_observed_signals,
            resolved_trial_predictions,
            pending_trial_predictions,
            abstained_trial_arms,
        }
    }

    fn require_version(&self, expected_version: u64) -> Result<(), InteractionOutcomeError> {
        if self.version != expected_version {
            return Err(InteractionOutcomeError::VersionConflict {
                expected: expected_version,
                actual: self.version,
            });
        }
        Ok(())
    }

    fn require_external_ledger(
        &self,
        prediction_ledger: &PredictionLedger,
    ) -> Result<(), InteractionOutcomeError> {
        if self.mirrored_prediction_ledger != *prediction_ledger {
            return Err(InteractionOutcomeError::PredictionLedgerDiverged);
        }
        Ok(())
    }

    fn apply_event(
        &mut self,
        event: InteractionOutcomeEvent,
    ) -> Result<InteractionOutcomeTransition, InteractionOutcomeError> {
        let trial_id = match &event {
            InteractionOutcomeEvent::TrialRegistered {
                trial,
                enrollment_transitions,
            } => {
                validate_trial(trial)?;
                if trial.id != self.next_trial_id {
                    return Err(InteractionOutcomeError::UnexpectedTrialId {
                        expected: self.next_trial_id,
                        actual: trial.id,
                    });
                }
                if self.trials.contains_key(&trial.id) {
                    return Err(InteractionOutcomeError::DuplicateTrial(trial.id));
                }
                if enrollment_transitions.len() != trial.arms.len() {
                    return Err(InteractionOutcomeError::EnrollmentShapeMismatch);
                }
                for transition in enrollment_transitions {
                    if !matches!(
                        transition.event,
                        PredictionEvent::Issued { .. } | PredictionEvent::Abstained { .. }
                    ) {
                        return Err(InteractionOutcomeError::InvalidEnrollmentTransition);
                    }
                    replay_prediction_transition(
                        &mut self.mirrored_prediction_ledger,
                        transition,
                    )?;
                }
                self.trials.insert(trial.id, trial.clone());
                self.next_trial_id += 1;
                trial.id
            }
            InteractionOutcomeEvent::ObservedSignalRecorded {
                trial_id,
                delivered_variant,
                evidence,
                resolution,
            } => {
                validate_observed_evidence(evidence)?;
                let trial = self
                    .trials
                    .get(trial_id)
                    .ok_or(InteractionOutcomeError::UnknownTrial(*trial_id))?;
                if trial.delivered_variant != Some(*delivered_variant) {
                    return Err(InteractionOutcomeError::DeliveredArmMismatch {
                        trial_id: *trial_id,
                    });
                }
                let arm = trial
                    .arm(*delivered_variant)
                    .ok_or(InteractionOutcomeError::MissingArm {
                        trial_id: *trial_id,
                        variant: *delivered_variant,
                    })?;
                let expected_label = evidence.signal.outcome_label();
                match (expected_label, resolution) {
                    (None, None) => {}
                    (Some(label), Some(transition)) => {
                        validate_resolution_transition(
                            transition,
                            arm.prediction_id,
                            label,
                            evidence.source,
                            evidence.observed_at_ms,
                        )?;
                        replay_prediction_transition(
                            &mut self.mirrored_prediction_ledger,
                            transition,
                        )?;
                    }
                    _ => return Err(InteractionOutcomeError::ResolutionShapeMismatch),
                }
                *trial_id
            }
            InteractionOutcomeEvent::PairedEvaluationRecorded {
                trial_id,
                evidence,
                resolutions,
            } => {
                validate_paired_evidence(evidence)?;
                let trial = self
                    .trials
                    .get(trial_id)
                    .ok_or(InteractionOutcomeError::UnknownTrial(*trial_id))?;
                let left = trial
                    .arm(evidence.left_variant)
                    .ok_or(InteractionOutcomeError::MissingArm {
                        trial_id: *trial_id,
                        variant: evidence.left_variant,
                    })?;
                let right = trial
                    .arm(evidence.right_variant)
                    .ok_or(InteractionOutcomeError::MissingArm {
                        trial_id: *trial_id,
                        variant: evidence.right_variant,
                    })?;
                match evidence.preference {
                    PairwisePreference::Tie => {
                        if !resolutions.is_empty() {
                            return Err(InteractionOutcomeError::ResolutionShapeMismatch);
                        }
                    }
                    PairwisePreference::Left | PairwisePreference::Right => {
                        if resolutions.len() != 2 {
                            return Err(InteractionOutcomeError::ResolutionShapeMismatch);
                        }
                        let left_label = if evidence.preference == PairwisePreference::Left {
                            SUCCESS_LABEL
                        } else {
                            FAILURE_LABEL
                        };
                        let right_label = if evidence.preference == PairwisePreference::Right {
                            SUCCESS_LABEL
                        } else {
                            FAILURE_LABEL
                        };
                        validate_resolution_transition(
                            &resolutions[0],
                            left.prediction_id,
                            left_label,
                            WitnessSource::ExternalEvaluator,
                            evidence.observed_at_ms,
                        )?;
                        validate_resolution_transition(
                            &resolutions[1],
                            right.prediction_id,
                            right_label,
                            WitnessSource::ExternalEvaluator,
                            evidence.observed_at_ms,
                        )?;
                        replay_prediction_transition(
                            &mut self.mirrored_prediction_ledger,
                            &resolutions[0],
                        )?;
                        replay_prediction_transition(
                            &mut self.mirrored_prediction_ledger,
                            &resolutions[1],
                        )?;
                    }
                }
                *trial_id
            }
        };

        self.version += 1;
        self.events.push(event.clone());
        Ok(InteractionOutcomeTransition {
            version: self.version,
            trial_id,
            event,
        })
    }
}

fn trial_from_enrollment(
    id: InteractionTrialId,
    enrollment: &ShadowPolicyEnrollment,
    delivered_variant: Option<PolicyVariant>,
) -> Result<InteractionTrial, InteractionOutcomeError> {
    if enrollment.batch.proposals.len() != enrollment.transitions.len() {
        return Err(InteractionOutcomeError::EnrollmentShapeMismatch);
    }
    let mut variants = BTreeSet::new();
    let mut arms = Vec::with_capacity(enrollment.batch.proposals.len());
    for (proposal, transition) in enrollment
        .batch
        .proposals
        .iter()
        .zip(&enrollment.transitions)
    {
        if !variants.insert(proposal.variant) {
            return Err(InteractionOutcomeError::DuplicateVariant(proposal.variant));
        }
        validate_enrollment_pair(proposal, transition)?;
        arms.push(TrialArm {
            variant: proposal.variant,
            policy_digest_fnv1a64: proposal.policy_digest_fnv1a64,
            prediction_id: transition.prediction_id,
            abstention_id: transition.abstention_id,
        });
    }
    if variants != PolicyVariant::all().into_iter().collect() {
        return Err(InteractionOutcomeError::MissingPolicyVariants);
    }
    if let Some(variant) = delivered_variant {
        let arm = arms
            .iter()
            .find(|arm| arm.variant == variant)
            .ok_or(InteractionOutcomeError::MissingArm {
                trial_id: id,
                variant,
            })?;
        if arm.prediction_id.is_none() {
            return Err(InteractionOutcomeError::ArmAbstained {
                trial_id: id,
                variant,
            });
        }
    }

    Ok(InteractionTrial {
        id,
        source_companion_version: enrollment.batch.source_companion_version,
        context_digest: enrollment.batch.context.context_digest,
        subject_scope_digest: enrollment.batch.context.subject_scope_digest,
        issued_at_ms: enrollment.batch.context.issued_at_ms,
        not_before_ms: enrollment.batch.context.not_before_ms,
        expires_at_ms: enrollment.batch.context.expires_at_ms,
        delivered_variant,
        arms,
    })
}

fn validate_enrollment_pair(
    proposal: &ShadowPolicyProposal,
    transition: &PredictionTransition,
) -> Result<(), InteractionOutcomeError> {
    match (proposal.is_abstention(), transition.prediction_id, transition.abstention_id) {
        (true, None, Some(_)) | (false, Some(_), None) => Ok(()),
        _ => Err(InteractionOutcomeError::EnrollmentShapeMismatch),
    }
}

fn validate_trial(trial: &InteractionTrial) -> Result<(), InteractionOutcomeError> {
    if trial.arms.len() != PolicyVariant::all().len() {
        return Err(InteractionOutcomeError::MissingPolicyVariants);
    }
    if trial.expires_at_ms <= trial.not_before_ms || trial.not_before_ms < trial.issued_at_ms {
        return Err(InteractionOutcomeError::InvalidOutcomeWindow);
    }
    let variants = trial.arms.iter().map(|arm| arm.variant).collect::<BTreeSet<_>>();
    if variants != PolicyVariant::all().into_iter().collect() {
        return Err(InteractionOutcomeError::MissingPolicyVariants);
    }
    for arm in &trial.arms {
        if arm.prediction_id.is_some() == arm.abstention_id.is_some() {
            return Err(InteractionOutcomeError::EnrollmentShapeMismatch);
        }
    }
    if let Some(delivered) = trial.delivered_variant {
        let arm = trial.arm(delivered).ok_or(InteractionOutcomeError::MissingArm {
            trial_id: trial.id,
            variant: delivered,
        })?;
        if arm.prediction_id.is_none() {
            return Err(InteractionOutcomeError::ArmAbstained {
                trial_id: trial.id,
                variant: delivered,
            });
        }
    }
    Ok(())
}

fn validate_observed_evidence(
    evidence: &ObservedOutcomeEvidence,
) -> Result<(), InteractionOutcomeError> {
    if evidence.evidence_digest == 0 {
        return Err(InteractionOutcomeError::EmptyEvidenceDigest);
    }
    match evidence.source {
        WitnessSource::ResponseGenerator => Err(InteractionOutcomeError::SelfGradingWitness),
        WitnessSource::ExternalEvaluator => Err(InteractionOutcomeError::WrongEvidenceChannel),
        WitnessSource::UserObservation => Ok(()),
        WitnessSource::Environment => match evidence.signal {
            ObservedSignal::TaskCompleted | ObservedSignal::Abandoned => Ok(()),
            _ => Err(InteractionOutcomeError::WrongEvidenceChannel),
        },
    }
}

fn validate_paired_evidence(
    evidence: &PairedEvaluationEvidence,
) -> Result<(), InteractionOutcomeError> {
    if evidence.left_variant == evidence.right_variant {
        return Err(InteractionOutcomeError::IdenticalPairArms);
    }
    if evidence.left_render_digest == 0
        || evidence.right_render_digest == 0
        || evidence.evidence_digest == 0
        || evidence.evaluator_digest == 0
    {
        return Err(InteractionOutcomeError::EmptyEvidenceDigest);
    }
    if evidence.left_render_digest == evidence.right_render_digest {
        return Err(InteractionOutcomeError::IdenticalRenderedAlternatives);
    }
    Ok(())
}

fn validate_resolution_transition(
    transition: &PredictionTransition,
    expected_prediction_id: Option<PredictionId>,
    expected_label: &str,
    expected_source: WitnessSource,
    expected_observed_at_ms: u64,
) -> Result<(), InteractionOutcomeError> {
    let prediction_id = expected_prediction_id.ok_or(InteractionOutcomeError::ResolutionForAbstention)?;
    if transition.prediction_id != Some(prediction_id) || transition.abstention_id.is_some() {
        return Err(InteractionOutcomeError::ResolutionPredictionMismatch);
    }
    let PredictionEvent::Resolved {
        prediction_id: event_prediction_id,
        witness,
        ..
    } = &transition.event
    else {
        return Err(InteractionOutcomeError::ResolutionShapeMismatch);
    };
    if *event_prediction_id != prediction_id
        || witness.label != expected_label
        || witness.source != expected_source
        || witness.observed_at_ms != expected_observed_at_ms
    {
        return Err(InteractionOutcomeError::ResolutionPredictionMismatch);
    }
    Ok(())
}

fn replay_prediction_transition(
    ledger: &mut PredictionLedger,
    expected: &PredictionTransition,
) -> Result<(), InteractionOutcomeError> {
    let actual = match &expected.event {
        PredictionEvent::Issued { prediction } => ledger.issue(
            ledger.version,
            PredictionInput {
                subject_scope: prediction.subject_scope.clone(),
                producer: prediction.producer.clone(),
                outcomes: prediction.outcomes.clone(),
                issued_at_ms: prediction.issued_at_ms,
                not_before_ms: prediction.not_before_ms,
                expires_at_ms: prediction.expires_at_ms,
                context_digest: prediction.context_digest,
            },
        )?,
        PredictionEvent::Abstained { abstention } => ledger.abstain(
            ledger.version,
            AbstentionInput {
                subject_scope: abstention.subject_scope.clone(),
                producer: abstention.producer.clone(),
                reason: abstention.reason.clone(),
                occurred_at_ms: abstention.occurred_at_ms,
                context_digest: abstention.context_digest,
            },
        )?,
        PredictionEvent::Resolved {
            prediction_id,
            witness,
            ..
        } => ledger.resolve(ledger.version, *prediction_id, witness.clone())?,
        PredictionEvent::Expired {
            prediction_id,
            expired_at_ms,
        } => ledger.expire(ledger.version, *prediction_id, *expired_at_ms)?,
    };
    if actual != *expected {
        return Err(InteractionOutcomeError::PredictionTransitionMismatch);
    }
    Ok(())
}

fn paired_arm_digest(evidence_digest: u64, render_digest: u64, arm: u8) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in evidence_digest
        .to_le_bytes()
        .into_iter()
        .chain(render_digest.to_le_bytes())
        .chain([arm])
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InteractionOutcomeError {
    #[error("outcome-ledger version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: u64, actual: u64 },
    #[error("unknown interaction trial {0}")]
    UnknownTrial(InteractionTrialId),
    #[error("duplicate interaction trial {0}")]
    DuplicateTrial(InteractionTrialId),
    #[error("unexpected interaction trial id: expected {expected}, actual {actual}")]
    UnexpectedTrialId {
        expected: InteractionTrialId,
        actual: InteractionTrialId,
    },
    #[error("enrollment expected S4 version {expected_before}, got {actual_before}")]
    EnrollmentVersionMismatch {
        expected_before: u64,
        actual_before: u64,
    },
    #[error("external S4 version mismatch: expected {expected}, actual {actual}")]
    ExternalLedgerVersionMismatch { expected: u64, actual: u64 },
    #[error("external S4 ledger diverged from the S5-B replay mirror")]
    PredictionLedgerDiverged,
    #[error("enrollment proposal and transition shapes do not match")]
    EnrollmentShapeMismatch,
    #[error("enrollment contains an invalid S4 transition")]
    InvalidEnrollmentTransition,
    #[error("duplicate policy variant {0:?}")]
    DuplicateVariant(PolicyVariant),
    #[error("the enrollment does not contain exactly the six required policy variants")]
    MissingPolicyVariants,
    #[error("trial {trial_id} is missing policy arm {variant:?}")]
    MissingArm {
        trial_id: InteractionTrialId,
        variant: PolicyVariant,
    },
    #[error("trial {trial_id} arm {variant:?} abstained and has no prediction")]
    ArmAbstained {
        trial_id: InteractionTrialId,
        variant: PolicyVariant,
    },
    #[error("trial {0} has no declared delivered arm")]
    NoDeliveredArm(InteractionTrialId),
    #[error("recorded delivered arm does not match trial {trial_id}")]
    DeliveredArmMismatch { trial_id: InteractionTrialId },
    #[error("invalid interaction outcome window")]
    InvalidOutcomeWindow,
    #[error("response generator cannot witness or grade interaction outcomes")]
    SelfGradingWitness,
    #[error("evidence source is not valid for this outcome channel or signal")]
    WrongEvidenceChannel,
    #[error("evidence and render digests must be nonzero")]
    EmptyEvidenceDigest,
    #[error("paired evaluation requires two distinct policy arms")]
    IdenticalPairArms,
    #[error("paired evaluation requires two distinct rendered alternatives")]
    IdenticalRenderedAlternatives,
    #[error("resolution was supplied for an abstaining arm")]
    ResolutionForAbstention,
    #[error("S4 resolution count or shape does not match the outcome evidence")]
    ResolutionShapeMismatch,
    #[error("S4 resolution prediction, witness, or label does not match the evidence")]
    ResolutionPredictionMismatch,
    #[error("replayed S4 transition differs from the recorded transition")]
    PredictionTransitionMismatch,
    #[error(transparent)]
    Prediction(#[from] PredictionLedgerError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion_interaction_policy::{
        PolicyContext, ShadowPolicyPlanner,
    };
    use crate::companion_state::{
        ClaimInput, ClaimSource, CompanionState, Retention, Sensitivity,
    };

    fn state_with_detail_claim() -> CompanionState {
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
                    retention: Retention::Durable,
                    observed_at_ms: 10,
                },
            )
            .unwrap();
        state
    }

    fn context(seed: u64) -> crate::companion_interaction_policy::PolicyContext {
        PolicyContext {
            context_digest: 100 + seed,
            subject_scope_digest: 200 + seed,
            domain: None,
            technical_context: false,
            asks_for_explanation: true,
            emotional_signal: false,
            issued_at_ms: 1_000,
            not_before_ms: 1_100,
            expires_at_ms: 2_000,
        }
    }

    fn enrolled(
        delivered: Option<PolicyVariant>,
    ) -> (
        PredictionLedger,
        InteractionOutcomeLedger,
        InteractionTrialId,
    ) {
        let state = state_with_detail_claim();
        let planner = ShadowPolicyPlanner::default();
        let mut predictions = PredictionLedger::new();
        let mut outcomes = InteractionOutcomeLedger::new(&predictions);
        let enrollment = planner
            .enroll(&state, &mut predictions, 0, context(1))
            .unwrap();
        let transition = outcomes
            .register_enrollment(0, &predictions, &enrollment, delivered)
            .unwrap();
        (predictions, outcomes, transition.trial_id)
    }

    #[test]
    fn pure_shadow_trial_rejects_direct_observed_evidence() {
        let (mut predictions, mut outcomes, trial_id) = enrolled(None);
        let before_predictions = predictions.clone();
        let before_outcomes = outcomes.clone();
        let error = outcomes
            .record_observed_signal(
                outcomes.version,
                &mut predictions,
                trial_id,
                ObservedOutcomeEvidence {
                    signal: ObservedSignal::ExplicitPositiveRating,
                    source: WitnessSource::UserObservation,
                    observed_at_ms: 1_200,
                    evidence_digest: 1,
                },
            )
            .unwrap_err();
        assert_eq!(error, InteractionOutcomeError::NoDeliveredArm(trial_id));
        assert_eq!(predictions, before_predictions);
        assert_eq!(outcomes, before_outcomes);
    }

    #[test]
    fn direct_evidence_resolves_only_the_delivered_arm() {
        let (mut predictions, mut outcomes, trial_id) =
            enrolled(Some(PolicyVariant::NeutralDefault));
        outcomes
            .record_observed_signal(
                outcomes.version,
                &mut predictions,
                trial_id,
                ObservedOutcomeEvidence {
                    signal: ObservedSignal::TaskCompleted,
                    source: WitnessSource::Environment,
                    observed_at_ms: 1_200,
                    evidence_digest: 2,
                },
            )
            .unwrap();

        let trial = outcomes.trials().get(&trial_id).unwrap();
        for arm in &trial.arms {
            let prediction = arm
                .prediction_id
                .and_then(|id| predictions.prediction(id))
                .unwrap();
            if arm.variant == PolicyVariant::NeutralDefault {
                assert!(matches!(prediction.status, PredictionStatus::Resolved { .. }));
            } else {
                assert!(matches!(prediction.status, PredictionStatus::Pending));
            }
        }
    }

    #[test]
    fn neutral_follow_up_is_recorded_without_forced_resolution() {
        let (mut predictions, mut outcomes, trial_id) =
            enrolled(Some(PolicyVariant::NeutralDefault));
        let before_version = predictions.version;
        outcomes
            .record_observed_signal(
                outcomes.version,
                &mut predictions,
                trial_id,
                ObservedOutcomeEvidence {
                    signal: ObservedSignal::NeutralFollowUp,
                    source: WitnessSource::UserObservation,
                    observed_at_ms: 1_200,
                    evidence_digest: 3,
                },
            )
            .unwrap();
        assert_eq!(predictions.version, before_version);
        assert_eq!(outcomes.summary().inconclusive_observed_signals, 1);
    }

    #[test]
    fn external_pairwise_evaluation_resolves_exactly_two_arms() {
        let (mut predictions, mut outcomes, trial_id) = enrolled(None);
        outcomes
            .record_paired_evaluation(
                outcomes.version,
                &mut predictions,
                trial_id,
                PairedEvaluationEvidence {
                    left_variant: PolicyVariant::CompanionDerived,
                    right_variant: PolicyVariant::NeutralDefault,
                    preference: PairwisePreference::Left,
                    left_render_digest: 10,
                    right_render_digest: 20,
                    evaluator_digest: 30,
                    observed_at_ms: 1_200,
                    evidence_digest: 40,
                },
            )
            .unwrap();
        let summary = outcomes.summary();
        assert_eq!(summary.resolved_trial_predictions, 2);
        assert_eq!(summary.pending_trial_predictions, 4);
    }

    #[test]
    fn replay_reconstructs_outcomes_and_s4_state_exactly() {
        let (mut predictions, mut outcomes, trial_id) =
            enrolled(Some(PolicyVariant::NeutralDefault));
        outcomes
            .record_observed_signal(
                outcomes.version,
                &mut predictions,
                trial_id,
                ObservedOutcomeEvidence {
                    signal: ObservedSignal::ExplicitNegativeRating,
                    source: WitnessSource::UserObservation,
                    observed_at_ms: 1_200,
                    evidence_digest: 50,
                },
            )
            .unwrap();
        let replayed = InteractionOutcomeLedger::replay(
            &outcomes.base_prediction_ledger,
            outcomes.events(),
        )
        .unwrap();
        assert_eq!(replayed, outcomes);
        assert_eq!(replayed.mirrored_prediction_ledger(), &predictions);
    }

    #[test]
    fn failed_second_resolution_is_atomic() {
        let (mut predictions, mut outcomes, trial_id) = enrolled(None);
        let evidence = PairedEvaluationEvidence {
            left_variant: PolicyVariant::CompanionDerived,
            right_variant: PolicyVariant::NeutralDefault,
            preference: PairwisePreference::Left,
            left_render_digest: 10,
            right_render_digest: 20,
            evaluator_digest: 30,
            observed_at_ms: 1_200,
            evidence_digest: 60,
        };
        outcomes
            .record_paired_evaluation(
                outcomes.version,
                &mut predictions,
                trial_id,
                evidence.clone(),
            )
            .unwrap();
        let before_predictions = predictions.clone();
        let before_outcomes = outcomes.clone();
        assert!(outcomes
            .record_paired_evaluation(
                outcomes.version,
                &mut predictions,
                trial_id,
                evidence,
            )
            .is_err());
        assert_eq!(predictions, before_predictions);
        assert_eq!(outcomes, before_outcomes);
    }
}
