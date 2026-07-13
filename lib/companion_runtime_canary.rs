//! S6-C session-scoped runtime canary for companion-derived response planning.
//!
//! The canary is deliberately not wired into `Runtime::chat()`. It accepts a
//! baseline `Response` and `RerankConfig`, prepares an S6 plan against a cloned
//! controller, and withholds the candidate plan until a matching S5-B trial has
//! been registered. Rejected commits leave the live controller unchanged.

use crate::companion_bounded_live_policy::{
    BoundedLivePolicyController, EvaluationEvidenceClass, LivePlanDisposition,
    LivePolicyActivationRequest, LivePolicyAuditRecord, LivePolicyControllerConfig,
    LivePolicyDecision, LivePolicyError, LivePolicyPlanningContext, LivePolicySummary,
    NeutralFallbackReason, ValidatedLivePolicyAuthorization,
};
use crate::companion_interaction_outcomes::{InteractionTrial, InteractionTrialId};
use crate::companion_interaction_policy::{
    PolicyVariant, ShadowPolicyBatch, ShadowPolicyProposal,
};
use crate::language_model::RerankConfig;
use crate::runtime::response_intent::Response;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

const MAX_CANARY_TURNS: u32 = 4;
const MAX_CANARY_DURATION_MS: u64 = 15 * 60 * 1_000;
const MIN_OUTPUT_CHARS: usize = 160;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeCanaryConfig {
    pub max_applied_turns: u32,
    pub max_duration_ms: u64,
    pub min_confidence_bps: u16,
    pub max_output_chars: usize,
}

impl Default for RuntimeCanaryConfig {
    fn default() -> Self {
        Self {
            max_applied_turns: MAX_CANARY_TURNS,
            max_duration_ms: MAX_CANARY_DURATION_MS,
            min_confidence_bps: 7_000,
            max_output_chars: 480,
        }
    }
}

impl RuntimeCanaryConfig {
    pub fn validate(self) -> Result<Self, RuntimeCanaryError> {
        if self.max_applied_turns == 0
            || self.max_applied_turns > MAX_CANARY_TURNS
            || self.max_duration_ms == 0
            || self.max_duration_ms > MAX_CANARY_DURATION_MS
            || self.min_confidence_bps > 10_000
            || self.max_output_chars < MIN_OUTPUT_CHARS
        {
            return Err(RuntimeCanaryError::InvalidConfig);
        }
        Ok(self)
    }

    fn controller_config(self) -> LivePolicyControllerConfig {
        LivePolicyControllerConfig {
            max_activation_turns: self.max_applied_turns,
            max_activation_duration_ms: self.max_duration_ms,
            min_confidence_bps: self.min_confidence_bps,
            max_output_chars: self.max_output_chars,
            allow_simulated_activation: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCanaryActivation {
    pub session_scope_digest: u64,
    pub subject_scope_digest: u64,
    pub valid_from_ms: u64,
    pub expires_at_ms: u64,
    pub max_turns: u32,
    pub operator_approval_digest: u64,
    pub held_out_study_artifact_digest: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCanaryTurnContext {
    pub session_scope_digest: u64,
    pub subject_scope_digest: u64,
    pub turn_digest: u64,
    pub context_digest: u64,
    pub current_companion_version: u64,
    pub prepared_at_ms: u64,
    pub sensitive_context: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryRegistrationRequirement {
    pub subject_scope_digest: u64,
    pub context_digest: u64,
    pub source_companion_version: u64,
    pub required_delivered_variant: PolicyVariant,
    pub required_policy_digest_fnv1a64: u64,
    pub prepared_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeCanaryEvent {
    Activated {
        session_scope_digest: u64,
        subject_scope_digest: u64,
        valid_from_ms: u64,
        expires_at_ms: u64,
        max_turns: u32,
        operator_approval_digest: u64,
        held_out_study_artifact_digest: u64,
        authorization_digest_fnv1a64: u64,
    },
    TurnCommitted {
        session_scope_digest: u64,
        subject_scope_digest: u64,
        turn_digest: u64,
        context_digest: u64,
        trial_id: InteractionTrialId,
        delivered_variant: PolicyVariant,
        disposition: LivePlanDisposition,
        fallback_reason: Option<NeutralFallbackReason>,
        controller_version: u64,
        remaining_turns: u32,
    },
    Revoked {
        revoked_at_ms: u64,
        reason_digest: u64,
        controller_version: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCanarySummary {
    pub version: u64,
    pub session_scope_digest: u64,
    pub subject_scope_digest: u64,
    pub active: bool,
    pub committed_turns: u64,
    pub companion_applied_turns: u64,
    pub neutral_fallbacks: u64,
    pub revocations: u64,
    pub remaining_turns: u32,
}

#[derive(Clone, PartialEq, Eq)]
pub struct RuntimeCanarySession {
    pub version: u64,
    config: RuntimeCanaryConfig,
    controller_config: LivePolicyControllerConfig,
    session_scope_digest: u64,
    subject_scope_digest: u64,
    controller: BoundedLivePolicyController,
    events: Vec<RuntimeCanaryEvent>,
}

impl fmt::Debug for RuntimeCanarySession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuntimeCanarySession")
            .field("version", &self.version)
            .field("config", &self.config)
            .field("session_scope_digest", &self.session_scope_digest)
            .field("subject_scope_digest", &self.subject_scope_digest)
            .field("controller_summary", &self.controller.summary())
            .field("events", &self.events)
            .finish()
    }
}

pub struct PendingRuntimeCanaryTurn {
    base_canary_version: u64,
    session_scope_digest: u64,
    subject_scope_digest: u64,
    turn_digest: u64,
    context_digest: u64,
    source_companion_version: u64,
    required_delivered_variant: PolicyVariant,
    required_policy_digest_fnv1a64: u64,
    prepared_at_ms: u64,
    candidate_controller: BoundedLivePolicyController,
    decision: LivePolicyDecision,
}

impl fmt::Debug for PendingRuntimeCanaryTurn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingRuntimeCanaryTurn")
            .field("base_canary_version", &self.base_canary_version)
            .field("session_scope_digest", &self.session_scope_digest)
            .field("subject_scope_digest", &self.subject_scope_digest)
            .field("turn_digest", &self.turn_digest)
            .field("context_digest", &self.context_digest)
            .field("source_companion_version", &self.source_companion_version)
            .field("required_delivered_variant", &self.required_delivered_variant)
            .field(
                "required_policy_digest_fnv1a64",
                &self.required_policy_digest_fnv1a64,
            )
            .field("prepared_at_ms", &self.prepared_at_ms)
            .field("disposition", &self.decision.disposition)
            .field("fallback_reason", &self.decision.fallback_reason)
            .finish_non_exhaustive()
    }
}

impl PendingRuntimeCanaryTurn {
    #[must_use]
    pub const fn registration_requirement(&self) -> CanaryRegistrationRequirement {
        CanaryRegistrationRequirement {
            subject_scope_digest: self.subject_scope_digest,
            context_digest: self.context_digest,
            source_companion_version: self.source_companion_version,
            required_delivered_variant: self.required_delivered_variant,
            required_policy_digest_fnv1a64: self.required_policy_digest_fnv1a64,
            prepared_at_ms: self.prepared_at_ms,
        }
    }

    #[must_use]
    pub const fn disposition(&self) -> LivePlanDisposition {
        self.decision.disposition
    }

    #[must_use]
    pub const fn fallback_reason(&self) -> Option<NeutralFallbackReason> {
        self.decision.fallback_reason
    }
}

#[derive(Debug, Clone)]
pub struct CommittedRuntimeCanaryTurn {
    pub response: Response,
    pub rerank_config: RerankConfig,
    pub trial_id: InteractionTrialId,
    pub delivered_variant: PolicyVariant,
    pub disposition: LivePlanDisposition,
    pub fallback_reason: Option<NeutralFallbackReason>,
    pub canary_version: u64,
    pub controller_version: u64,
    pub remaining_turns: u32,
    pub live_response_influence: bool,
    pub routing_authority: bool,
    pub persistence_authority: bool,
    pub belief_promotion_authority: bool,
    pub ontology_promotion_authority: bool,
    pub tool_selection_authority: bool,
    pub action_authority: bool,
}

impl RuntimeCanarySession {
    pub fn activate(
        config: RuntimeCanaryConfig,
        authorization: &ValidatedLivePolicyAuthorization,
        proposal: &ShadowPolicyProposal,
        activation: RuntimeCanaryActivation,
    ) -> Result<Self, RuntimeCanaryError> {
        let config = config.validate()?;
        if authorization.evidence_class() != EvaluationEvidenceClass::HeldOutConversationStudy {
            return Err(RuntimeCanaryError::HeldOutConversationEvidenceRequired);
        }
        validate_activation(&activation, config)?;

        let controller_config = config.controller_config();
        let mut controller = BoundedLivePolicyController::new(controller_config)?;
        controller.activate(
            0,
            authorization,
            proposal,
            LivePolicyActivationRequest {
                subject_scope_digest: activation.subject_scope_digest,
                valid_from_ms: activation.valid_from_ms,
                expires_at_ms: activation.expires_at_ms,
                max_turns: activation.max_turns,
                operator_approval_digest: activation.operator_approval_digest,
            },
        )?;

        let event = RuntimeCanaryEvent::Activated {
            session_scope_digest: activation.session_scope_digest,
            subject_scope_digest: activation.subject_scope_digest,
            valid_from_ms: activation.valid_from_ms,
            expires_at_ms: activation.expires_at_ms,
            max_turns: activation.max_turns,
            operator_approval_digest: activation.operator_approval_digest,
            held_out_study_artifact_digest: activation.held_out_study_artifact_digest,
            authorization_digest_fnv1a64: authorization.digest_fnv1a64(),
        };

        Ok(Self {
            version: 1,
            config,
            controller_config,
            session_scope_digest: activation.session_scope_digest,
            subject_scope_digest: activation.subject_scope_digest,
            controller,
            events: vec![event],
        })
    }

    #[must_use]
    pub const fn config(&self) -> RuntimeCanaryConfig {
        self.config
    }

    #[must_use]
    pub const fn controller_config(&self) -> LivePolicyControllerConfig {
        self.controller_config
    }

    #[must_use]
    pub fn controller_summary(&self) -> LivePolicySummary {
        self.controller.summary()
    }

    #[must_use]
    pub fn controller_audit_records(&self) -> Vec<LivePolicyAuditRecord> {
        self.controller.audit_records()
    }

    #[must_use]
    pub fn events(&self) -> &[RuntimeCanaryEvent] {
        &self.events
    }

    pub fn prepare_turn(
        &self,
        expected_version: u64,
        context: RuntimeCanaryTurnContext,
        batch: &ShadowPolicyBatch,
        baseline_response: Response,
        baseline_rerank_config: RerankConfig,
    ) -> Result<PendingRuntimeCanaryTurn, RuntimeCanaryError> {
        self.require_version(expected_version)?;
        validate_turn_context(&context)?;
        if context.session_scope_digest != self.session_scope_digest {
            return Err(RuntimeCanaryError::SessionScopeMismatch);
        }
        if context.subject_scope_digest != self.subject_scope_digest {
            return Err(RuntimeCanaryError::SubjectScopeMismatch);
        }
        validate_batch(batch, &context)?;

        let companion = required_proposal(batch, PolicyVariant::CompanionDerived)?;
        let neutral = required_proposal(batch, PolicyVariant::NeutralDefault)?;

        let mut candidate_controller = self.controller.clone();
        let decision = candidate_controller.plan_response(
            candidate_controller.version,
            LivePolicyPlanningContext {
                subject_scope_digest: context.subject_scope_digest,
                turn_digest: context.turn_digest,
                context_digest: context.context_digest,
                current_companion_version: context.current_companion_version,
                planned_at_ms: context.prepared_at_ms,
                sensitive_context: context.sensitive_context,
            },
            baseline_response,
            baseline_rerank_config,
        )?;

        let (required_delivered_variant, required_policy_digest_fnv1a64) =
            match decision.disposition {
                LivePlanDisposition::Applied => {
                    let lease = self
                        .controller
                        .active_lease()
                        .ok_or(RuntimeCanaryError::AppliedWithoutActiveLease)?;
                    if companion.source_companion_version != lease.source_companion_version
                        || companion.policy_digest_fnv1a64 != lease.source_policy_digest_fnv1a64
                    {
                        return Err(RuntimeCanaryError::AppliedProposalMismatch);
                    }
                    (
                        PolicyVariant::CompanionDerived,
                        companion.policy_digest_fnv1a64,
                    )
                }
                LivePlanDisposition::NeutralFallback => (
                    PolicyVariant::NeutralDefault,
                    neutral.policy_digest_fnv1a64,
                ),
            };

        Ok(PendingRuntimeCanaryTurn {
            base_canary_version: self.version,
            session_scope_digest: context.session_scope_digest,
            subject_scope_digest: context.subject_scope_digest,
            turn_digest: context.turn_digest,
            context_digest: context.context_digest,
            source_companion_version: batch.source_companion_version,
            required_delivered_variant,
            required_policy_digest_fnv1a64,
            prepared_at_ms: context.prepared_at_ms,
            candidate_controller,
            decision,
        })
    }

    pub fn commit_turn(
        &mut self,
        expected_version: u64,
        pending: PendingRuntimeCanaryTurn,
        trial: &InteractionTrial,
    ) -> Result<CommittedRuntimeCanaryTurn, RuntimeCanaryError> {
        self.require_version(expected_version)?;
        if pending.base_canary_version != self.version {
            return Err(RuntimeCanaryError::VersionConflict {
                expected: pending.base_canary_version,
                actual: self.version,
            });
        }
        if pending.session_scope_digest != self.session_scope_digest {
            return Err(RuntimeCanaryError::SessionScopeMismatch);
        }
        if pending.subject_scope_digest != self.subject_scope_digest {
            return Err(RuntimeCanaryError::SubjectScopeMismatch);
        }
        validate_trial_binding(&pending, trial)?;
        if pending.decision.routing_authority
            || pending.decision.belief_promotion_authority
            || pending.decision.action_authority
        {
            return Err(RuntimeCanaryError::UnexpectedAuthority);
        }

        let PendingRuntimeCanaryTurn {
            turn_digest,
            context_digest,
            required_delivered_variant,
            candidate_controller,
            decision,
            ..
        } = pending;

        self.controller = candidate_controller;
        self.version = self
            .version
            .checked_add(1)
            .ok_or(RuntimeCanaryError::VersionOverflow)?;
        self.events.push(RuntimeCanaryEvent::TurnCommitted {
            session_scope_digest: self.session_scope_digest,
            subject_scope_digest: self.subject_scope_digest,
            turn_digest,
            context_digest,
            trial_id: trial.id,
            delivered_variant: required_delivered_variant,
            disposition: decision.disposition,
            fallback_reason: decision.fallback_reason,
            controller_version: decision.audit_version,
            remaining_turns: decision.remaining_turns,
        });

        Ok(CommittedRuntimeCanaryTurn {
            response: decision.response,
            rerank_config: decision.rerank_config,
            trial_id: trial.id,
            delivered_variant: required_delivered_variant,
            disposition: decision.disposition,
            fallback_reason: decision.fallback_reason,
            canary_version: self.version,
            controller_version: decision.audit_version,
            remaining_turns: decision.remaining_turns,
            live_response_influence: decision.live_response_influence,
            routing_authority: false,
            persistence_authority: false,
            belief_promotion_authority: false,
            ontology_promotion_authority: false,
            tool_selection_authority: false,
            action_authority: false,
        })
    }

    pub fn revoke(
        &mut self,
        expected_version: u64,
        revoked_at_ms: u64,
        reason_digest: u64,
    ) -> Result<(), RuntimeCanaryError> {
        self.require_version(expected_version)?;
        let mut candidate = self.controller.clone();
        let transition = candidate.revoke(candidate.version, revoked_at_ms, reason_digest)?;
        self.controller = candidate;
        self.version = self
            .version
            .checked_add(1)
            .ok_or(RuntimeCanaryError::VersionOverflow)?;
        self.events.push(RuntimeCanaryEvent::Revoked {
            revoked_at_ms,
            reason_digest,
            controller_version: transition.version,
        });
        Ok(())
    }

    #[must_use]
    pub fn summary(&self) -> RuntimeCanarySummary {
        let controller = self.controller.summary();
        let committed_turns = self
            .events
            .iter()
            .filter(|event| matches!(event, RuntimeCanaryEvent::TurnCommitted { .. }))
            .count() as u64;
        let companion_applied_turns = self
            .events
            .iter()
            .filter(|event| {
                matches!(
                    event,
                    RuntimeCanaryEvent::TurnCommitted {
                        disposition: LivePlanDisposition::Applied,
                        ..
                    }
                )
            })
            .count() as u64;
        let neutral_fallbacks = committed_turns.saturating_sub(companion_applied_turns);
        let revocations = self
            .events
            .iter()
            .filter(|event| matches!(event, RuntimeCanaryEvent::Revoked { .. }))
            .count() as u64;
        RuntimeCanarySummary {
            version: self.version,
            session_scope_digest: self.session_scope_digest,
            subject_scope_digest: self.subject_scope_digest,
            active: controller.active,
            committed_turns,
            companion_applied_turns,
            neutral_fallbacks,
            revocations,
            remaining_turns: controller.remaining_turns,
        }
    }

    fn require_version(&self, expected_version: u64) -> Result<(), RuntimeCanaryError> {
        if self.version != expected_version {
            return Err(RuntimeCanaryError::VersionConflict {
                expected: expected_version,
                actual: self.version,
            });
        }
        Ok(())
    }
}

fn validate_activation(
    activation: &RuntimeCanaryActivation,
    config: RuntimeCanaryConfig,
) -> Result<(), RuntimeCanaryError> {
    if activation.session_scope_digest == 0
        || activation.subject_scope_digest == 0
        || activation.operator_approval_digest == 0
        || activation.held_out_study_artifact_digest == 0
        || activation.max_turns == 0
        || activation.max_turns > config.max_applied_turns
    {
        return Err(RuntimeCanaryError::InvalidActivation);
    }
    let duration = activation
        .expires_at_ms
        .checked_sub(activation.valid_from_ms)
        .ok_or(RuntimeCanaryError::InvalidActivation)?;
    if duration == 0 || duration > config.max_duration_ms {
        return Err(RuntimeCanaryError::InvalidActivation);
    }
    Ok(())
}

fn validate_turn_context(context: &RuntimeCanaryTurnContext) -> Result<(), RuntimeCanaryError> {
    if context.session_scope_digest == 0
        || context.subject_scope_digest == 0
        || context.turn_digest == 0
        || context.context_digest == 0
        || context.current_companion_version == 0
        || context.prepared_at_ms == 0
    {
        return Err(RuntimeCanaryError::InvalidTurnContext);
    }
    Ok(())
}

fn validate_batch(
    batch: &ShadowPolicyBatch,
    context: &RuntimeCanaryTurnContext,
) -> Result<(), RuntimeCanaryError> {
    if batch.context.context_digest != context.context_digest {
        return Err(RuntimeCanaryError::BatchContextMismatch);
    }
    if batch.context.subject_scope_digest != context.subject_scope_digest {
        return Err(RuntimeCanaryError::BatchSubjectMismatch);
    }
    if batch.source_companion_version != context.current_companion_version {
        return Err(RuntimeCanaryError::BatchVersionMismatch);
    }
    Ok(())
}

fn required_proposal(
    batch: &ShadowPolicyBatch,
    variant: PolicyVariant,
) -> Result<&ShadowPolicyProposal, RuntimeCanaryError> {
    let proposal = batch
        .proposal(variant)
        .ok_or(RuntimeCanaryError::MissingPolicyVariant(variant))?;
    if proposal.variant != variant
        || proposal.is_abstention()
        || proposal.policy_digest_fnv1a64 == 0
        || proposal.source_companion_version != batch.source_companion_version
        || proposal.context_digest != batch.context.context_digest
    {
        return Err(RuntimeCanaryError::InvalidPolicyProposal(variant));
    }
    Ok(proposal)
}

fn validate_trial_binding(
    pending: &PendingRuntimeCanaryTurn,
    trial: &InteractionTrial,
) -> Result<(), RuntimeCanaryError> {
    if trial.id == 0 {
        return Err(RuntimeCanaryError::InvalidTrialId);
    }
    if trial.subject_scope_digest != pending.subject_scope_digest {
        return Err(RuntimeCanaryError::TrialSubjectMismatch);
    }
    if trial.context_digest != pending.context_digest {
        return Err(RuntimeCanaryError::TrialContextMismatch);
    }
    if trial.source_companion_version != pending.source_companion_version {
        return Err(RuntimeCanaryError::TrialVersionMismatch);
    }
    if trial.issued_at_ms > pending.prepared_at_ms
        || trial.expires_at_ms <= pending.prepared_at_ms
    {
        return Err(RuntimeCanaryError::TrialTimingMismatch);
    }
    if trial.delivered_variant != Some(pending.required_delivered_variant) {
        return Err(RuntimeCanaryError::DeliveredVariantMismatch {
            expected: pending.required_delivered_variant,
            actual: trial.delivered_variant,
        });
    }
    let arm = trial
        .arm(pending.required_delivered_variant)
        .ok_or(RuntimeCanaryError::MissingTrialArm(
            pending.required_delivered_variant,
        ))?;
    if arm.policy_digest_fnv1a64 != pending.required_policy_digest_fnv1a64 {
        return Err(RuntimeCanaryError::TrialPolicyDigestMismatch);
    }
    if arm.prediction_id.is_none() || arm.abstention_id.is_some() {
        return Err(RuntimeCanaryError::DeliveredArmNotPredictive);
    }
    Ok(())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RuntimeCanaryError {
    #[error("runtime canary configuration is invalid")]
    InvalidConfig,
    #[error("S6-C requires held-out conversation evidence")]
    HeldOutConversationEvidenceRequired,
    #[error("runtime canary activation metadata or bounds are invalid")]
    InvalidActivation,
    #[error("runtime canary turn context is invalid")]
    InvalidTurnContext,
    #[error("version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: u64, actual: u64 },
    #[error("session scope does not match the activated canary")]
    SessionScopeMismatch,
    #[error("subject scope does not match the activated canary")]
    SubjectScopeMismatch,
    #[error("policy batch context does not match the prepared turn")]
    BatchContextMismatch,
    #[error("policy batch subject does not match the prepared turn")]
    BatchSubjectMismatch,
    #[error("policy batch companion version does not match the prepared turn")]
    BatchVersionMismatch,
    #[error("policy batch is missing required variant {0:?}")]
    MissingPolicyVariant(PolicyVariant),
    #[error("policy proposal for variant {0:?} is invalid")]
    InvalidPolicyProposal(PolicyVariant),
    #[error("an applied canary turn has no active S6 lease")]
    AppliedWithoutActiveLease,
    #[error("the applied companion proposal does not match the active S6 lease")]
    AppliedProposalMismatch,
    #[error("interaction trial id must be non-zero")]
    InvalidTrialId,
    #[error("interaction trial subject does not match the prepared turn")]
    TrialSubjectMismatch,
    #[error("interaction trial context does not match the prepared turn")]
    TrialContextMismatch,
    #[error("interaction trial companion version does not match the prepared turn")]
    TrialVersionMismatch,
    #[error("interaction trial timing does not cover response delivery")]
    TrialTimingMismatch,
    #[error("delivered variant mismatch: expected {expected:?}, actual {actual:?}")]
    DeliveredVariantMismatch {
        expected: PolicyVariant,
        actual: Option<PolicyVariant>,
    },
    #[error("interaction trial is missing arm {0:?}")]
    MissingTrialArm(PolicyVariant),
    #[error("interaction trial policy digest does not match the prepared plan")]
    TrialPolicyDigestMismatch,
    #[error("the delivered interaction arm is abstaining or lacks a prediction")]
    DeliveredArmNotPredictive,
    #[error("the underlying S6 decision unexpectedly claims broader authority")]
    UnexpectedAuthority,
    #[error("runtime canary version overflow")]
    VersionOverflow,
    #[error(transparent)]
    LivePolicy(#[from] LivePolicyError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid_and_frozen_to_canary_bounds() {
        let config = RuntimeCanaryConfig::default().validate().unwrap();
        assert_eq!(config.max_applied_turns, MAX_CANARY_TURNS);
        assert_eq!(config.max_duration_ms, MAX_CANARY_DURATION_MS);
    }

    #[test]
    fn oversized_canary_budget_is_rejected() {
        let config = RuntimeCanaryConfig {
            max_applied_turns: MAX_CANARY_TURNS + 1,
            ..RuntimeCanaryConfig::default()
        };
        assert_eq!(config.validate(), Err(RuntimeCanaryError::InvalidConfig));
    }

    #[test]
    fn zero_canary_duration_is_rejected() {
        let config = RuntimeCanaryConfig {
            max_duration_ms: 0,
            ..RuntimeCanaryConfig::default()
        };
        assert_eq!(config.validate(), Err(RuntimeCanaryError::InvalidConfig));
    }
}
