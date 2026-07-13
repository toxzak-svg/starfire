//! S6-A bounded, reversible companion-policy influence over response planning.
//!
//! This module never selects tools, routes work, mutates companion claims, changes
//! beliefs, or performs actions. It may only adjust typed response-planning
//! metadata before the existing reranker: a style hint, bounded output length,
//! and auditable policy slots. Neutral fallback preserves the baseline response
//! and rerank configuration exactly.

use crate::companion_interaction_policy::{
    AcknowledgmentLevel, DetailLevel, DialogueMode, ExplanationStyle, InteractionPolicy,
    PolicyVariant, ShadowPolicyProposal, VocabularyLevel,
};
use crate::companion_policy_evaluation::{
    EvaluationSplit, PolicyEvaluationReport, PolicyEvaluationVerdict,
};
use crate::companion_state::Sensitivity;
use crate::language_model::RerankConfig;
use crate::personality::ResponseStyle;
use crate::runtime::response_intent::{Response, ResponseIntent};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

const DEFAULT_BRIEF_MAX_CHARS: usize = 160;
const DEFAULT_STANDARD_MAX_CHARS: usize = 280;
const DEFAULT_DETAILED_MAX_CHARS: usize = 420;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationEvidenceClass {
    FrozenSimulation,
    HeldOutConversationStudy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedPromotionGate {
    digest_fnv1a64: u64,
    evidence_class: EvaluationEvidenceClass,
}

impl ValidatedPromotionGate {
    pub fn validate(
        report: &PolicyEvaluationReport,
        evidence_class: EvaluationEvidenceClass,
        evaluation_artifact_digest: u64,
    ) -> Result<Self, LivePolicyError> {
        if evaluation_artifact_digest == 0 {
            return Err(LivePolicyError::EmptyEvaluationArtifactDigest);
        }
        if report.verdict != PolicyEvaluationVerdict::Pass
            || !report.promotion_eligible
            || !report.development_excluded_from_verdict
            || report.live_response_influence
            || report.routing_authority
            || report.belief_promotion_authority
            || report.action_authority
        {
            return Err(LivePolicyError::EvaluationNotPromotionEligible);
        }

        let controls = control_variants().into_iter().collect::<BTreeSet<_>>();
        let mut observed = BTreeSet::new();
        for comparison in &report.holdout_comparisons {
            if !comparison.split.is_holdout()
                || comparison.control == PolicyVariant::CompanionDerived
                || !comparison.gates.evidence_sufficient()
                || !comparison.gates.performance_passed()
            {
                return Err(LivePolicyError::MalformedPromotionEvidence);
            }
            if !observed.insert((comparison.split, comparison.control)) {
                return Err(LivePolicyError::MalformedPromotionEvidence);
            }
        }

        let expected = [
            EvaluationSplit::OpaqueSubjectHoldout,
            EvaluationSplit::TemporalHoldout,
        ]
        .into_iter()
        .flat_map(|split| {
            controls
                .iter()
                .copied()
                .map(move |control| (split, control))
        })
        .collect::<BTreeSet<_>>();
        if observed != expected {
            return Err(LivePolicyError::MalformedPromotionEvidence);
        }

        let mut digest = fnv1a64(b"s6a-promotion-gate-v1");
        digest = mix_u64(digest, evaluation_artifact_digest);
        digest = mix_u64(digest, evidence_class as u64);
        for (split, control) in observed {
            digest = mix_u64(digest, split as u64);
            digest = mix_u64(digest, control as u64);
        }
        Ok(Self {
            digest_fnv1a64: digest,
            evidence_class,
        })
    }

    #[must_use]
    pub const fn digest_fnv1a64(&self) -> u64 {
        self.digest_fnv1a64
    }

    #[must_use]
    pub const fn evidence_class(&self) -> EvaluationEvidenceClass {
        self.evidence_class
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LivePolicyControllerConfig {
    pub max_activation_turns: u32,
    pub max_activation_duration_ms: u64,
    pub min_confidence_bps: u16,
    pub max_output_chars: usize,
    pub allow_simulated_activation: bool,
}

impl Default for LivePolicyControllerConfig {
    fn default() -> Self {
        Self {
            max_activation_turns: 8,
            max_activation_duration_ms: 30 * 60 * 1_000,
            min_confidence_bps: 7_000,
            max_output_chars: 480,
            allow_simulated_activation: false,
        }
    }
}

impl LivePolicyControllerConfig {
    pub fn validate(self) -> Result<Self, LivePolicyError> {
        if self.max_activation_turns == 0
            || self.max_activation_duration_ms == 0
            || self.max_output_chars < DEFAULT_BRIEF_MAX_CHARS
            || self.min_confidence_bps > 10_000
        {
            return Err(LivePolicyError::InvalidControllerConfig);
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePolicyActivationRequest {
    pub subject_scope_digest: u64,
    pub valid_from_ms: u64,
    pub expires_at_ms: u64,
    pub max_turns: u32,
    pub operator_approval_digest: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePolicyLease {
    pub subject_scope_digest: u64,
    pub source_companion_version: u64,
    pub source_policy_digest_fnv1a64: u64,
    pub source_claim_ids: Vec<u64>,
    pub confidence_bps: u16,
    pub policy: InteractionPolicy,
    pub valid_from_ms: u64,
    pub expires_at_ms: u64,
    pub remaining_turns: u32,
    pub promotion_gate_digest_fnv1a64: u64,
    pub operator_approval_digest: u64,
    pub evidence_class: EvaluationEvidenceClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LivePlanDisposition {
    Applied,
    NeutralFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NeutralFallbackReason {
    Disabled,
    NotYetValid,
    Expired,
    BudgetExhausted,
    SubjectMismatch,
    SensitiveContext,
    DisallowedIntent,
    DuplicateTurn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LivePolicyEvent {
    Activated {
        lease: LivePolicyLease,
    },
    TurnPlanned {
        turn_digest: u64,
        context_digest: u64,
        planned_at_ms: u64,
        intent_label: String,
        disposition: LivePlanDisposition,
        fallback_reason: Option<NeutralFallbackReason>,
        baseline_plan_digest_fnv1a64: u64,
        planned_plan_digest_fnv1a64: u64,
        max_chars_after: Option<usize>,
        style_after: Option<String>,
        remaining_turns_after: u32,
    },
    Revoked {
        revoked_at_ms: u64,
        reason_digest: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePolicyTransition {
    pub version: u64,
    pub event: LivePolicyEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePolicyPlanningContext {
    pub subject_scope_digest: u64,
    pub turn_digest: u64,
    pub context_digest: u64,
    pub planned_at_ms: u64,
    pub sensitive_context: bool,
}

#[derive(Debug, Clone)]
pub struct LivePolicyDecision {
    pub disposition: LivePlanDisposition,
    pub fallback_reason: Option<NeutralFallbackReason>,
    pub response: Response,
    pub rerank_config: RerankConfig,
    pub audit_version: u64,
    pub remaining_turns: u32,
    pub live_response_influence: bool,
    pub routing_authority: bool,
    pub belief_promotion_authority: bool,
    pub action_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePolicySummary {
    pub version: u64,
    pub active: bool,
    pub remaining_turns: u32,
    pub applied_turns: u64,
    pub neutral_fallbacks: u64,
    pub revocations: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedLivePolicyController {
    pub version: u64,
    config: LivePolicyControllerConfig,
    active_lease: Option<LivePolicyLease>,
    events: Vec<LivePolicyEvent>,
    seen_turns: BTreeSet<u64>,
}

impl Default for BoundedLivePolicyController {
    fn default() -> Self {
        Self::new(LivePolicyControllerConfig::default()).expect("default S6-A config is valid")
    }
}

impl BoundedLivePolicyController {
    pub fn new(config: LivePolicyControllerConfig) -> Result<Self, LivePolicyError> {
        Ok(Self {
            version: 0,
            config: config.validate()?,
            active_lease: None,
            events: Vec::new(),
            seen_turns: BTreeSet::new(),
        })
    }

    #[must_use]
    pub fn active_lease(&self) -> Option<&LivePolicyLease> {
        self.active_lease.as_ref()
    }

    #[must_use]
    pub fn events(&self) -> &[LivePolicyEvent] {
        &self.events
    }

    pub fn activate(
        &mut self,
        expected_version: u64,
        gate: &ValidatedPromotionGate,
        proposal: &ShadowPolicyProposal,
        request: LivePolicyActivationRequest,
    ) -> Result<LivePolicyTransition, LivePolicyError> {
        self.require_version(expected_version)?;
        if self.active_lease.is_some() {
            return Err(LivePolicyError::AlreadyActive);
        }
        if gate.evidence_class == EvaluationEvidenceClass::FrozenSimulation
            && !self.config.allow_simulated_activation
        {
            return Err(LivePolicyError::SimulatedEvidenceRejected);
        }
        validate_proposal(proposal, self.config.min_confidence_bps)?;
        validate_activation_request(&request, self.config)?;

        let lease = LivePolicyLease {
            subject_scope_digest: request.subject_scope_digest,
            source_companion_version: proposal.source_companion_version,
            source_policy_digest_fnv1a64: proposal.policy_digest_fnv1a64,
            source_claim_ids: proposal.source_claim_ids(),
            confidence_bps: proposal.confidence_bps,
            policy: proposal.policy,
            valid_from_ms: request.valid_from_ms,
            expires_at_ms: request.expires_at_ms,
            remaining_turns: request.max_turns,
            promotion_gate_digest_fnv1a64: gate.digest_fnv1a64,
            operator_approval_digest: request.operator_approval_digest,
            evidence_class: gate.evidence_class,
        };
        self.apply_event(LivePolicyEvent::Activated { lease })
    }

    pub fn plan_response(
        &mut self,
        expected_version: u64,
        context: LivePolicyPlanningContext,
        baseline_response: Response,
        baseline_rerank_config: RerankConfig,
    ) -> Result<LivePolicyDecision, LivePolicyError> {
        self.require_version(expected_version)?;
        if context.turn_digest == 0 || context.context_digest == 0 {
            return Err(LivePolicyError::EmptyPlanningDigest);
        }

        let baseline_digest = response_plan_digest(&baseline_response, &baseline_rerank_config);
        let fallback = self.fallback_reason(&context, &baseline_response.intent);
        let mut response = baseline_response;
        let mut rerank_config = baseline_rerank_config;
        let mut disposition = LivePlanDisposition::NeutralFallback;
        let mut reason = fallback;
        let mut remaining = self
            .active_lease
            .as_ref()
            .map_or(0, |lease| lease.remaining_turns);

        if fallback.is_none() {
            let lease = self
                .active_lease
                .as_ref()
                .expect("eligible plan has an active lease");
            apply_policy(
                lease.policy,
                &mut response,
                &mut rerank_config,
                self.config.max_output_chars,
            );
            disposition = LivePlanDisposition::Applied;
            reason = None;
            remaining = lease.remaining_turns.saturating_sub(1);
        }

        let planned_digest = response_plan_digest(&response, &rerank_config);
        let style_after = response
            .style_hint
            .as_ref()
            .map(style_label)
            .map(str::to_owned);
        let event = LivePolicyEvent::TurnPlanned {
            turn_digest: context.turn_digest,
            context_digest: context.context_digest,
            planned_at_ms: context.planned_at_ms,
            intent_label: response.intent.label().to_owned(),
            disposition,
            fallback_reason: reason,
            baseline_plan_digest_fnv1a64: baseline_digest,
            planned_plan_digest_fnv1a64: planned_digest,
            max_chars_after: rerank_config.max_chars,
            style_after,
            remaining_turns_after: remaining,
        };
        let transition = self.apply_event(event)?;

        Ok(LivePolicyDecision {
            disposition,
            fallback_reason: reason,
            response,
            rerank_config,
            audit_version: transition.version,
            remaining_turns: remaining,
            live_response_influence: disposition == LivePlanDisposition::Applied,
            routing_authority: false,
            belief_promotion_authority: false,
            action_authority: false,
        })
    }

    pub fn revoke(
        &mut self,
        expected_version: u64,
        revoked_at_ms: u64,
        reason_digest: u64,
    ) -> Result<LivePolicyTransition, LivePolicyError> {
        self.require_version(expected_version)?;
        if self.active_lease.is_none() {
            return Err(LivePolicyError::NotActive);
        }
        if revoked_at_ms == 0 || reason_digest == 0 {
            return Err(LivePolicyError::EmptyRevocationEvidence);
        }
        self.apply_event(LivePolicyEvent::Revoked {
            revoked_at_ms,
            reason_digest,
        })
    }

    pub fn replay(
        config: LivePolicyControllerConfig,
        events: &[LivePolicyEvent],
    ) -> Result<Self, LivePolicyError> {
        let mut controller = Self::new(config)?;
        for event in events {
            controller.apply_event(event.clone())?;
        }
        Ok(controller)
    }

    #[must_use]
    pub fn summary(&self) -> LivePolicySummary {
        let applied_turns = self
            .events
            .iter()
            .filter(|event| {
                matches!(
                    event,
                    LivePolicyEvent::TurnPlanned {
                        disposition: LivePlanDisposition::Applied,
                        ..
                    }
                )
            })
            .count() as u64;
        let neutral_fallbacks = self
            .events
            .iter()
            .filter(|event| {
                matches!(
                    event,
                    LivePolicyEvent::TurnPlanned {
                        disposition: LivePlanDisposition::NeutralFallback,
                        ..
                    }
                )
            })
            .count() as u64;
        let revocations = self
            .events
            .iter()
            .filter(|event| matches!(event, LivePolicyEvent::Revoked { .. }))
            .count() as u64;
        LivePolicySummary {
            version: self.version,
            active: self.active_lease.is_some(),
            remaining_turns: self
                .active_lease
                .as_ref()
                .map_or(0, |lease| lease.remaining_turns),
            applied_turns,
            neutral_fallbacks,
            revocations,
        }
    }

    fn fallback_reason(
        &self,
        context: &LivePolicyPlanningContext,
        intent: &ResponseIntent,
    ) -> Option<NeutralFallbackReason> {
        let Some(lease) = self.active_lease.as_ref() else {
            return Some(NeutralFallbackReason::Disabled);
        };
        if self.seen_turns.contains(&context.turn_digest) {
            return Some(NeutralFallbackReason::DuplicateTurn);
        }
        if context.planned_at_ms < lease.valid_from_ms {
            return Some(NeutralFallbackReason::NotYetValid);
        }
        if context.planned_at_ms >= lease.expires_at_ms {
            return Some(NeutralFallbackReason::Expired);
        }
        if lease.remaining_turns == 0 {
            return Some(NeutralFallbackReason::BudgetExhausted);
        }
        if context.subject_scope_digest != lease.subject_scope_digest {
            return Some(NeutralFallbackReason::SubjectMismatch);
        }
        if context.sensitive_context {
            return Some(NeutralFallbackReason::SensitiveContext);
        }
        if !intent_allowed(intent) {
            return Some(NeutralFallbackReason::DisallowedIntent);
        }
        None
    }

    fn require_version(&self, expected_version: u64) -> Result<(), LivePolicyError> {
        if self.version != expected_version {
            return Err(LivePolicyError::VersionConflict {
                expected: expected_version,
                actual: self.version,
            });
        }
        Ok(())
    }

    fn apply_event(
        &mut self,
        event: LivePolicyEvent,
    ) -> Result<LivePolicyTransition, LivePolicyError> {
        match &event {
            LivePolicyEvent::Activated { lease } => {
                if self.active_lease.is_some() {
                    return Err(LivePolicyError::AlreadyActive);
                }
                self.active_lease = Some(lease.clone());
                self.seen_turns.clear();
            }
            LivePolicyEvent::TurnPlanned {
                turn_digest,
                disposition,
                fallback_reason,
                remaining_turns_after,
                ..
            } => {
                let duplicate_fallback = *disposition == LivePlanDisposition::NeutralFallback
                    && *fallback_reason == Some(NeutralFallbackReason::DuplicateTurn);
                if duplicate_fallback {
                    if !self.seen_turns.contains(turn_digest) {
                        return Err(LivePolicyError::InvalidDuplicateFallback(*turn_digest));
                    }
                } else if !self.seen_turns.insert(*turn_digest) {
                    return Err(LivePolicyError::DuplicateAuditTurn(*turn_digest));
                }
                match (disposition, self.active_lease.as_mut()) {
                    (LivePlanDisposition::Applied, Some(lease)) => {
                        let expected = lease.remaining_turns.saturating_sub(1);
                        if *remaining_turns_after != expected {
                            return Err(LivePolicyError::AuditBudgetMismatch);
                        }
                        lease.remaining_turns = *remaining_turns_after;
                    }
                    (LivePlanDisposition::Applied, None) => {
                        return Err(LivePolicyError::AuditWithoutActivation);
                    }
                    (LivePlanDisposition::NeutralFallback, Some(lease)) => {
                        if *remaining_turns_after != lease.remaining_turns {
                            return Err(LivePolicyError::AuditBudgetMismatch);
                        }
                    }
                    (LivePlanDisposition::NeutralFallback, None) => {
                        if *remaining_turns_after != 0 {
                            return Err(LivePolicyError::AuditBudgetMismatch);
                        }
                    }
                }
            }
            LivePolicyEvent::Revoked { .. } => {
                if self.active_lease.take().is_none() {
                    return Err(LivePolicyError::NotActive);
                }
            }
        }
        self.version = self
            .version
            .checked_add(1)
            .ok_or(LivePolicyError::VersionOverflow)?;
        self.events.push(event.clone());
        Ok(LivePolicyTransition {
            version: self.version,
            event,
        })
    }
}

fn validate_proposal(
    proposal: &ShadowPolicyProposal,
    min_confidence_bps: u16,
) -> Result<(), LivePolicyError> {
    if proposal.variant != PolicyVariant::CompanionDerived
        || proposal.is_abstention()
        || proposal.policy_digest_fnv1a64 == 0
        || proposal.source_companion_version == 0
        || proposal.evidence.is_empty()
    {
        return Err(LivePolicyError::InvalidCompanionProposal);
    }
    if proposal.confidence_bps < min_confidence_bps {
        return Err(LivePolicyError::InsufficientPolicyConfidence {
            minimum: min_confidence_bps,
            actual: proposal.confidence_bps,
        });
    }
    if proposal
        .evidence
        .iter()
        .any(|evidence| evidence.sensitivity == Sensitivity::Sensitive)
    {
        return Err(LivePolicyError::SensitivePolicyEvidence);
    }
    Ok(())
}

fn validate_activation_request(
    request: &LivePolicyActivationRequest,
    config: LivePolicyControllerConfig,
) -> Result<(), LivePolicyError> {
    if request.subject_scope_digest == 0 || request.operator_approval_digest == 0 {
        return Err(LivePolicyError::EmptyActivationEvidence);
    }
    if request.max_turns == 0 || request.max_turns > config.max_activation_turns {
        return Err(LivePolicyError::ActivationTurnBudgetExceeded);
    }
    let duration = request
        .expires_at_ms
        .checked_sub(request.valid_from_ms)
        .ok_or(LivePolicyError::InvalidActivationWindow)?;
    if duration == 0 || duration > config.max_activation_duration_ms {
        return Err(LivePolicyError::InvalidActivationWindow);
    }
    Ok(())
}

fn intent_allowed(intent: &ResponseIntent) -> bool {
    matches!(
        intent,
        ResponseIntent::Recall
            | ResponseIntent::Teaching
            | ResponseIntent::Capability
            | ResponseIntent::ResearchStatus
    )
}

fn apply_policy(
    policy: InteractionPolicy,
    response: &mut Response,
    rerank_config: &mut RerankConfig,
    max_output_chars: usize,
) {
    response.style_hint = match policy.explanation_style {
        ExplanationStyle::Concrete => Some(ResponseStyle::Direct),
        ExplanationStyle::Abstract => Some(ResponseStyle::Analytical),
        ExplanationStyle::Adaptive => response
            .style_hint
            .clone()
            .or_else(|| response.intent.default_style_hint()),
    };

    let baseline_max = rerank_config
        .max_chars
        .unwrap_or(DEFAULT_STANDARD_MAX_CHARS);
    let planned_max = match policy.detail {
        DetailLevel::Brief => baseline_max.min(DEFAULT_BRIEF_MAX_CHARS),
        DetailLevel::Standard => baseline_max.min(max_output_chars),
        DetailLevel::Detailed => baseline_max
            .max(DEFAULT_DETAILED_MAX_CHARS)
            .min(max_output_chars),
    };
    rerank_config.max_chars = Some(planned_max);

    set_slot(response, "companion.detail", detail_label(policy.detail));
    set_slot(
        response,
        "companion.explanation_style",
        explanation_label(policy.explanation_style),
    );
    set_slot(
        response,
        "companion.dialogue",
        dialogue_label(policy.dialogue),
    );
    set_slot(
        response,
        "companion.vocabulary",
        vocabulary_label(policy.vocabulary),
    );
    set_slot(
        response,
        "companion.acknowledgment",
        acknowledgment_label(policy.acknowledgment),
    );
}

fn set_slot(response: &mut Response, key: &str, value: &str) {
    response.slots.retain(|(existing, _)| existing != key);
    response.slots.push((key.to_owned(), value.to_owned()));
}

fn response_plan_digest(response: &Response, config: &RerankConfig) -> u64 {
    let mut digest = fnv1a64(b"s6a-response-plan-v1");
    digest = mix_bytes(digest, response.intent.label().as_bytes());
    digest = mix_bytes(digest, response.body.as_bytes());
    digest = mix_u64(digest, config.max_chars.unwrap_or(0) as u64);
    digest = mix_bytes(
        digest,
        response
            .style_hint
            .as_ref()
            .map_or("none", style_label)
            .as_bytes(),
    );
    for (key, value) in &response.slots {
        digest = mix_bytes(digest, key.as_bytes());
        digest = mix_bytes(digest, value.as_bytes());
    }
    digest
}

fn control_variants() -> [PolicyVariant; 5] {
    [
        PolicyVariant::NeutralDefault,
        PolicyVariant::RecencyOnly,
        PolicyVariant::MajorityPrior,
        PolicyVariant::ContextOnly,
        PolicyVariant::ScrambledScope,
    ]
}

fn style_label(style: &ResponseStyle) -> &'static str {
    match style {
        ResponseStyle::Direct => "direct",
        ResponseStyle::Analytical => "analytical",
        ResponseStyle::Warm => "warm",
        ResponseStyle::Playful => "playful",
        ResponseStyle::Curious => "curious",
        ResponseStyle::Minimal => "minimal",
        ResponseStyle::LeetMatch => "leet-match",
        ResponseStyle::Reserved => "reserved",
    }
}

fn detail_label(value: DetailLevel) -> &'static str {
    match value {
        DetailLevel::Brief => "brief",
        DetailLevel::Standard => "standard",
        DetailLevel::Detailed => "detailed",
    }
}

fn explanation_label(value: ExplanationStyle) -> &'static str {
    match value {
        ExplanationStyle::Concrete => "concrete",
        ExplanationStyle::Abstract => "abstract",
        ExplanationStyle::Adaptive => "adaptive",
    }
}

fn dialogue_label(value: DialogueMode) -> &'static str {
    match value {
        DialogueMode::Direct => "direct",
        DialogueMode::QuestionLed => "question-led",
    }
}

fn vocabulary_label(value: VocabularyLevel) -> &'static str {
    match value {
        VocabularyLevel::Plain => "plain",
        VocabularyLevel::Standard => "standard",
        VocabularyLevel::Technical => "technical",
    }
}

fn acknowledgment_label(value: AcknowledgmentLevel) -> &'static str {
    match value {
        AcknowledgmentLevel::Minimal => "minimal",
        AcknowledgmentLevel::Standard => "standard",
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    mix_bytes(0xcbf29ce484222325, bytes)
}

fn mix_u64(digest: u64, value: u64) -> u64 {
    mix_bytes(digest, &value.to_le_bytes())
}

fn mix_bytes(mut digest: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LivePolicyError {
    #[error("S5-C evaluation is not promotion eligible")]
    EvaluationNotPromotionEligible,
    #[error("S5-C promotion evidence is malformed or incomplete")]
    MalformedPromotionEvidence,
    #[error("evaluation artifact digest must be non-zero")]
    EmptyEvaluationArtifactDigest,
    #[error("controller configuration is invalid")]
    InvalidControllerConfig,
    #[error("a live policy lease is already active")]
    AlreadyActive,
    #[error("no live policy lease is active")]
    NotActive,
    #[error("frozen simulation evidence is rejected by default")]
    SimulatedEvidenceRejected,
    #[error("companion proposal is not an eligible non-abstaining candidate")]
    InvalidCompanionProposal,
    #[error("policy confidence {actual} is below minimum {minimum}")]
    InsufficientPolicyConfidence { minimum: u16, actual: u16 },
    #[error("sensitive companion evidence cannot authorize S6-A")]
    SensitivePolicyEvidence,
    #[error("activation evidence digests must be non-zero")]
    EmptyActivationEvidence,
    #[error("activation turn budget exceeds the configured bound")]
    ActivationTurnBudgetExceeded,
    #[error("activation time window is invalid or too long")]
    InvalidActivationWindow,
    #[error("planning digests must be non-zero")]
    EmptyPlanningDigest,
    #[error("revocation evidence must be non-zero")]
    EmptyRevocationEvidence,
    #[error("version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: u64, actual: u64 },
    #[error("audit contains duplicate turn digest {0} without duplicate fallback")]
    DuplicateAuditTurn(u64),
    #[error("duplicate fallback references unseen turn digest {0}")]
    InvalidDuplicateFallback(u64),
    #[error("audit budget does not match deterministic transition")]
    AuditBudgetMismatch,
    #[error("audit applies a policy without an active lease")]
    AuditWithoutActivation,
    #[error("controller version overflow")]
    VersionOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion_interaction_policy::PolicyEvidence;
    use crate::companion_policy_evaluation::{
        CandidateControlComparison, CandidatePairwiseMetrics, ComparisonGates,
        PolicyEvaluationReport, SplitEvaluationReport,
    };

    fn passing_gates() -> ComparisonGates {
        ComparisonGates {
            resolved_evidence_sufficient: true,
            direct_evidence_sufficient: true,
            pairwise_evidence_sufficient: true,
            brier_improvement_passed: true,
            calibration_non_regression_passed: true,
            pairwise_margin_passed: true,
            correction_non_regression_passed: true,
            clarification_non_regression_passed: true,
            completion_non_regression_passed: true,
            abandonment_non_regression_passed: true,
            abstention_non_regression_passed: true,
            compute_overhead_passed: true,
        }
    }

    fn passing_report() -> PolicyEvaluationReport {
        let mut comparisons = Vec::new();
        for split in [
            EvaluationSplit::OpaqueSubjectHoldout,
            EvaluationSplit::TemporalHoldout,
        ] {
            for control in control_variants() {
                comparisons.push(CandidateControlComparison {
                    split,
                    control,
                    candidate_resolved: 4,
                    control_resolved: 4,
                    candidate_direct_outcomes: 2,
                    control_direct_outcomes: 2,
                    pairwise: CandidatePairwiseMetrics {
                        candidate_wins: 3,
                        control_wins: 1,
                        ties: 0,
                        total: 4,
                        candidate_win_margin_bps: Some(5_000),
                    },
                    brier_improvement_ppm: Some(10_000),
                    calibration_regression_bps: Some(0),
                    correction_regression_bps: Some(0),
                    clarification_regression_bps: Some(0),
                    completion_regression_bps: Some(0),
                    abandonment_regression_bps: Some(0),
                    abstention_regression_bps: Some(0),
                    compute_overhead_bps: Some(500),
                    gates: passing_gates(),
                });
            }
        }
        PolicyEvaluationReport {
            splits: vec![SplitEvaluationReport {
                split: EvaluationSplit::Development,
                arms: Default::default(),
                candidate_pairwise: Default::default(),
            }],
            holdout_comparisons: comparisons,
            verdict: PolicyEvaluationVerdict::Pass,
            development_excluded_from_verdict: true,
            promotion_eligible: true,
            live_response_influence: false,
            routing_authority: false,
            belief_promotion_authority: false,
            action_authority: false,
        }
    }

    fn proposal() -> ShadowPolicyProposal {
        ShadowPolicyProposal {
            variant: PolicyVariant::CompanionDerived,
            source_companion_version: 3,
            policy: InteractionPolicy {
                detail: DetailLevel::Brief,
                explanation_style: ExplanationStyle::Concrete,
                dialogue: DialogueMode::Direct,
                vocabulary: VocabularyLevel::Technical,
                acknowledgment: AcknowledgmentLevel::Minimal,
            },
            evidence: vec![PolicyEvidence {
                claim_id: 1,
                key: "preference.brevity.general".to_owned(),
                confidence_bps: 9_000,
                updated_at_ms: 10,
                sensitivity: Sensitivity::Personal,
            }],
            confidence_bps: 9_000,
            context_digest: 20,
            policy_digest_fnv1a64: 30,
            predicted_outcomes: Vec::new(),
            abstention_reason: None,
        }
    }

    fn activation() -> LivePolicyActivationRequest {
        LivePolicyActivationRequest {
            subject_scope_digest: 99,
            valid_from_ms: 1_000,
            expires_at_ms: 2_000,
            max_turns: 2,
            operator_approval_digest: 77,
        }
    }

    fn context(turn: u64, sensitive: bool) -> LivePolicyPlanningContext {
        LivePolicyPlanningContext {
            subject_scope_digest: 99,
            turn_digest: turn,
            context_digest: 500 + turn,
            planned_at_ms: 1_100,
            sensitive_context: sensitive,
        }
    }

    fn baseline() -> (Response, RerankConfig) {
        (
            Response::with_body(
                ResponseIntent::Recall,
                "A sufficiently long baseline response.",
            ),
            RerankConfig {
                max_chars: Some(280),
                temperature: 0.7,
                top_k: 20,
                deterministic: true,
                seed: Some(1),
            },
        )
    }

    #[test]
    fn malformed_report_cannot_form_gate() {
        let mut report = passing_report();
        report.holdout_comparisons.pop();
        assert_eq!(
            ValidatedPromotionGate::validate(
                &report,
                EvaluationEvidenceClass::HeldOutConversationStudy,
                1,
            ),
            Err(LivePolicyError::MalformedPromotionEvidence)
        );
    }

    #[test]
    fn production_default_rejects_simulated_activation() {
        let gate = ValidatedPromotionGate::validate(
            &passing_report(),
            EvaluationEvidenceClass::FrozenSimulation,
            1,
        )
        .unwrap();
        let mut controller = BoundedLivePolicyController::default();
        assert_eq!(
            controller.activate(0, &gate, &proposal(), activation()),
            Err(LivePolicyError::SimulatedEvidenceRejected)
        );
    }

    #[test]
    fn applied_plan_changes_only_bounded_metadata() {
        let config = LivePolicyControllerConfig {
            allow_simulated_activation: true,
            ..LivePolicyControllerConfig::default()
        };
        let gate = ValidatedPromotionGate::validate(
            &passing_report(),
            EvaluationEvidenceClass::FrozenSimulation,
            1,
        )
        .unwrap();
        let mut controller = BoundedLivePolicyController::new(config).unwrap();
        controller
            .activate(0, &gate, &proposal(), activation())
            .unwrap();
        let (response, rerank) = baseline();
        let body_before = response.body.clone();
        let decision = controller
            .plan_response(controller.version, context(1, false), response, rerank)
            .unwrap();
        assert_eq!(decision.disposition, LivePlanDisposition::Applied);
        assert_eq!(decision.response.body, body_before);
        assert_eq!(decision.rerank_config.max_chars, Some(160));
        assert_eq!(decision.remaining_turns, 1);
        assert!(!decision.routing_authority);
        assert!(!decision.belief_promotion_authority);
        assert!(!decision.action_authority);
    }

    #[test]
    fn sensitive_context_uses_exact_neutral_fallback() {
        let config = LivePolicyControllerConfig {
            allow_simulated_activation: true,
            ..LivePolicyControllerConfig::default()
        };
        let gate = ValidatedPromotionGate::validate(
            &passing_report(),
            EvaluationEvidenceClass::FrozenSimulation,
            1,
        )
        .unwrap();
        let mut controller = BoundedLivePolicyController::new(config).unwrap();
        controller
            .activate(0, &gate, &proposal(), activation())
            .unwrap();
        let (response, rerank) = baseline();
        let body = response.body.clone();
        let max_chars = rerank.max_chars;
        let decision = controller
            .plan_response(controller.version, context(1, true), response, rerank)
            .unwrap();
        assert_eq!(decision.disposition, LivePlanDisposition::NeutralFallback);
        assert_eq!(
            decision.fallback_reason,
            Some(NeutralFallbackReason::SensitiveContext)
        );
        assert_eq!(decision.response.body, body);
        assert_eq!(decision.rerank_config.max_chars, max_chars);
        assert_eq!(decision.remaining_turns, 2);
    }

    #[test]
    fn duplicate_turn_is_audited_neutral_fallback_without_consuming_budget() {
        let config = LivePolicyControllerConfig {
            allow_simulated_activation: true,
            ..LivePolicyControllerConfig::default()
        };
        let gate = ValidatedPromotionGate::validate(
            &passing_report(),
            EvaluationEvidenceClass::FrozenSimulation,
            1,
        )
        .unwrap();
        let mut controller = BoundedLivePolicyController::new(config).unwrap();
        controller
            .activate(0, &gate, &proposal(), activation())
            .unwrap();
        let (response, rerank) = baseline();
        controller
            .plan_response(controller.version, context(1, false), response, rerank)
            .unwrap();
        let (duplicate_response, duplicate_rerank) = baseline();
        let duplicate = controller
            .plan_response(
                controller.version,
                context(1, false),
                duplicate_response,
                duplicate_rerank,
            )
            .unwrap();
        assert_eq!(duplicate.disposition, LivePlanDisposition::NeutralFallback);
        assert_eq!(
            duplicate.fallback_reason,
            Some(NeutralFallbackReason::DuplicateTurn)
        );
        assert_eq!(duplicate.remaining_turns, 1);
        let replayed = BoundedLivePolicyController::replay(config, controller.events()).unwrap();
        assert_eq!(replayed, controller);
    }

    #[test]
    fn replay_is_exact() {
        let config = LivePolicyControllerConfig {
            allow_simulated_activation: true,
            ..LivePolicyControllerConfig::default()
        };
        let gate = ValidatedPromotionGate::validate(
            &passing_report(),
            EvaluationEvidenceClass::FrozenSimulation,
            1,
        )
        .unwrap();
        let mut controller = BoundedLivePolicyController::new(config).unwrap();
        controller
            .activate(0, &gate, &proposal(), activation())
            .unwrap();
        let (response, rerank) = baseline();
        controller
            .plan_response(controller.version, context(1, false), response, rerank)
            .unwrap();
        controller.revoke(controller.version, 1_200, 88).unwrap();
        let replayed = BoundedLivePolicyController::replay(config, controller.events()).unwrap();
        assert_eq!(replayed, controller);
    }
}
