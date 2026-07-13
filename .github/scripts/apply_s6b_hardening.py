from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "lib/companion_bounded_live_policy.rs"
PROBE = ROOT / "lib/examples/s6a_bounded_live_policy_probe.rs"


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def replace_all_checked(text: str, old: str, new: str, minimum: int, label: str) -> str:
    count = text.count(old)
    if count < minimum:
        raise SystemExit(f"{label}: expected at least {minimum} matches, found {count}")
    return text.replace(old, new)


src = SRC.read_text()

# Add the exact-proposal authorization capability after the version-only S5-C gate.
marker = """}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct LivePolicyControllerConfig {"""
authorization = r''' }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedLivePolicyAuthorization {
    digest_fnv1a64: u64,
    promotion_gate_digest_fnv1a64: u64,
    evidence_class: EvaluationEvidenceClass,
    source_companion_version: u64,
    source_policy_digest_fnv1a64: u64,
    source_claim_ids: Vec<u64>,
    confidence_bps: u16,
    policy: InteractionPolicy,
}

impl ValidatedPromotionGate {
    pub fn authorize_proposal(
        &self,
        proposal: &ShadowPolicyProposal,
    ) -> Result<ValidatedLivePolicyAuthorization, LivePolicyError> {
        validate_proposal(proposal, 0)?;
        if proposal.source_companion_version != self.authorized_companion_version {
            return Err(LivePolicyError::PromotionSourceVersionMismatch {
                authorized: self.authorized_companion_version,
                actual: proposal.source_companion_version,
            });
        }
        let source_claim_ids = canonical_claim_ids(proposal)?;
        let mut digest = fnv1a64(b"s6b-live-policy-authorization-v1");
        digest = mix_u64(digest, self.digest_fnv1a64);
        digest = mix_u64(digest, proposal.source_companion_version);
        digest = mix_u64(digest, proposal.policy_digest_fnv1a64);
        digest = mix_u64(digest, u64::from(proposal.confidence_bps));
        digest = mix_u64(digest, proposal.policy.detail as u64);
        digest = mix_u64(digest, proposal.policy.explanation_style as u64);
        digest = mix_u64(digest, proposal.policy.dialogue as u64);
        digest = mix_u64(digest, proposal.policy.vocabulary as u64);
        digest = mix_u64(digest, proposal.policy.acknowledgment as u64);
        for claim_id in &source_claim_ids {
            digest = mix_u64(digest, *claim_id);
        }
        Ok(ValidatedLivePolicyAuthorization {
            digest_fnv1a64: digest,
            promotion_gate_digest_fnv1a64: self.digest_fnv1a64,
            evidence_class: self.evidence_class,
            source_companion_version: proposal.source_companion_version,
            source_policy_digest_fnv1a64: proposal.policy_digest_fnv1a64,
            source_claim_ids,
            confidence_bps: proposal.confidence_bps,
            policy: proposal.policy,
        })
    }
}

impl ValidatedLivePolicyAuthorization {
    #[must_use]
    pub const fn digest_fnv1a64(&self) -> u64 {
        self.digest_fnv1a64
    }

    #[must_use]
    pub const fn evidence_class(&self) -> EvaluationEvidenceClass {
        self.evidence_class
    }

    #[must_use]
    pub const fn source_companion_version(&self) -> u64 {
        self.source_companion_version
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LivePolicyControllerConfig {'''
src = replace_once(src, marker, authorization, "authorization insertion")

# Bind leases to the exact authorization, not only the S5-C gate.
src = replace_once(
    src,
    "    pub promotion_gate_digest_fnv1a64: u64,\n    pub operator_approval_digest: u64,",
    "    pub promotion_gate_digest_fnv1a64: u64,\n    pub authorization_digest_fnv1a64: u64,\n    pub operator_approval_digest: u64,",
    "lease authorization digest",
)

# Persist enough non-raw metadata to validate replay semantics.
src = replace_once(
    src,
    """    TurnPlanned {
        turn_digest: u64,
        context_digest: u64,
        planned_at_ms: u64,
        current_companion_version: u64,
        intent_label: String,
        disposition: LivePlanDisposition,
        fallback_reason: Option<NeutralFallbackReason>,
        baseline_plan_digest_fnv1a64: u64,
        planned_plan_digest_fnv1a64: u64,
        max_chars_after: Option<usize>,
        style_after: Option<String>,
        remaining_turns_after: u32,
    },""",
    """    TurnPlanned {
        subject_scope_digest: u64,
        turn_digest: u64,
        context_digest: u64,
        planned_at_ms: u64,
        current_companion_version: u64,
        sensitive_context: bool,
        intent_label: String,
        disposition: LivePlanDisposition,
        fallback_reason: Option<NeutralFallbackReason>,
        baseline_plan_digest_fnv1a64: u64,
        planned_plan_digest_fnv1a64: u64,
        baseline_max_chars: Option<usize>,
        max_chars_after: Option<usize>,
        baseline_style: Option<String>,
        style_after: Option<String>,
        remaining_turns_after: u32,
    },""",
    "turn audit metadata",
)

# Add sequence-checked, hash-chained records.
transition_marker = """pub struct LivePolicyTransition {
    pub version: u64,
    pub event: LivePolicyEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePolicyPlanningContext {"""
audit_record = r'''pub struct LivePolicyTransition {
    pub version: u64,
    pub event: LivePolicyEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePolicyAuditRecord {
    pub sequence: u64,
    pub previous_chain_digest_fnv1a64: u64,
    pub event_digest_fnv1a64: u64,
    pub chain_digest_fnv1a64: u64,
    pub event: LivePolicyEvent,
}

impl LivePolicyAuditRecord {
    #[must_use]
    pub fn chain(events: &[LivePolicyEvent]) -> Vec<Self> {
        let mut previous = 0_u64;
        events
            .iter()
            .enumerate()
            .map(|(index, event)| {
                let sequence = index as u64 + 1;
                let event_digest = live_policy_event_digest(event);
                let chain_digest = live_policy_chain_digest(previous, sequence, event_digest);
                let record = Self {
                    sequence,
                    previous_chain_digest_fnv1a64: previous,
                    event_digest_fnv1a64: event_digest,
                    chain_digest_fnv1a64: chain_digest,
                    event: event.clone(),
                };
                previous = chain_digest;
                record
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePolicyPlanningContext {'''
src = replace_once(src, transition_marker, audit_record, "audit record insertion")

# Export records from the controller.
src = replace_once(
    src,
    """    pub fn events(&self) -> &[LivePolicyEvent] {
        &self.events
    }

    pub fn activate(""",
    """    pub fn events(&self) -> &[LivePolicyEvent] {
        &self.events
    }

    #[must_use]
    pub fn audit_records(&self) -> Vec<LivePolicyAuditRecord> {
        LivePolicyAuditRecord::chain(&self.events)
    }

    pub fn activate(""",
    "audit export",
)

# Activation now requires the exact-proposal authorization capability.
src = replace_once(
    src,
    """        gate: &ValidatedPromotionGate,
        proposal: &ShadowPolicyProposal,""",
    """        authorization: &ValidatedLivePolicyAuthorization,
        proposal: &ShadowPolicyProposal,""",
    "activate authorization parameter",
)
src = replace_once(
    src,
    """        if gate.evidence_class == EvaluationEvidenceClass::FrozenSimulation
            && !self.config.allow_simulated_activation
        {
            return Err(LivePolicyError::SimulatedEvidenceRejected);
        }
        if proposal.source_companion_version != gate.authorized_companion_version {
            return Err(LivePolicyError::PromotionSourceVersionMismatch {
                authorized: gate.authorized_companion_version,
                actual: proposal.source_companion_version,
            });
        }
        validate_proposal(proposal, self.config.min_confidence_bps)?;""",
    """        if authorization.evidence_class == EvaluationEvidenceClass::FrozenSimulation
            && !self.config.allow_simulated_activation
        {
            return Err(LivePolicyError::SimulatedEvidenceRejected);
        }
        validate_authorized_proposal(authorization, proposal)?;
        validate_proposal(proposal, self.config.min_confidence_bps)?;""",
    "activate exact proposal validation",
)
src = replace_once(
    src,
    """            promotion_gate_digest_fnv1a64: gate.digest_fnv1a64,
            operator_approval_digest: request.operator_approval_digest,
            evidence_class: gate.evidence_class,
        };
        self.apply_event(LivePolicyEvent::Activated { lease })""",
    """            promotion_gate_digest_fnv1a64: authorization.promotion_gate_digest_fnv1a64,
            authorization_digest_fnv1a64: authorization.digest_fnv1a64,
            operator_approval_digest: request.operator_approval_digest,
            evidence_class: authorization.evidence_class,
        };
        self.apply_event(LivePolicyEvent::Activated { lease }, Some(authorization))""",
    "activation lease provenance",
)

# Capture baseline metadata and include the complete semantic envelope in turn events.
src = replace_once(
    src,
    """        let baseline_digest = response_plan_digest(&baseline_response, &baseline_rerank_config);
        let fallback = self.fallback_reason(&context, &baseline_response.intent);""",
    """        let baseline_digest = response_plan_digest(&baseline_response, &baseline_rerank_config);
        let baseline_max_chars = baseline_rerank_config.max_chars;
        let baseline_style = baseline_response
            .style_hint
            .as_ref()
            .map(style_label)
            .map(str::to_owned);
        let fallback = self.fallback_reason(&context, &baseline_response.intent);""",
    "baseline metadata capture",
)
src = replace_once(
    src,
    """        let event = LivePolicyEvent::TurnPlanned {
            turn_digest: context.turn_digest,
            context_digest: context.context_digest,
            planned_at_ms: context.planned_at_ms,
            current_companion_version: context.current_companion_version,
            intent_label: response.intent.label().to_owned(),
            disposition,
            fallback_reason: reason,
            baseline_plan_digest_fnv1a64: baseline_digest,
            planned_plan_digest_fnv1a64: planned_digest,
            max_chars_after: rerank_config.max_chars,
            style_after,
            remaining_turns_after: remaining,
        };
        let transition = self.apply_event(event)?;""",
    """        let event = LivePolicyEvent::TurnPlanned {
            subject_scope_digest: context.subject_scope_digest,
            turn_digest: context.turn_digest,
            context_digest: context.context_digest,
            planned_at_ms: context.planned_at_ms,
            current_companion_version: context.current_companion_version,
            sensitive_context: context.sensitive_context,
            intent_label: response.intent.label().to_owned(),
            disposition,
            fallback_reason: reason,
            baseline_plan_digest_fnv1a64: baseline_digest,
            planned_plan_digest_fnv1a64: planned_digest,
            baseline_max_chars,
            max_chars_after: rerank_config.max_chars,
            baseline_style,
            style_after,
            remaining_turns_after: remaining,
        };
        let transition = self.apply_event(event, None)?;""",
    "turn event envelope",
)
src = replace_once(
    src,
    """        self.apply_event(LivePolicyEvent::Revoked {
            revoked_at_ms,
            reason_digest,
        })""",
    """        self.apply_event(
            LivePolicyEvent::Revoked {
                revoked_at_ms,
                reason_digest,
            },
            None,
        )""",
    "revoke apply signature",
)

# Replace unsafe raw-event replay with trusted authorization plus audit-chain replay.
old_replay = """    pub fn replay(
        config: LivePolicyControllerConfig,
        events: &[LivePolicyEvent],
    ) -> Result<Self, LivePolicyError> {
        let mut controller = Self::new(config)?;
        for event in events {
            controller.apply_event(event.clone())?;
        }
        Ok(controller)
    }"""
new_replay = """    pub fn replay(
        config: LivePolicyControllerConfig,
        authorizations: &[ValidatedLivePolicyAuthorization],
        records: &[LivePolicyAuditRecord],
    ) -> Result<Self, LivePolicyError> {
        validate_audit_chain(records)?;
        let mut controller = Self::new(config)?;
        for record in records {
            let authorization = match &record.event {
                LivePolicyEvent::Activated { lease } => Some(
                    authorizations
                        .iter()
                        .find(|authorization| {
                            authorization.digest_fnv1a64 == lease.authorization_digest_fnv1a64
                        })
                        .ok_or(LivePolicyError::UnknownReplayAuthorization(
                            lease.authorization_digest_fnv1a64,
                        ))?,
                ),
                _ => None,
            };
            controller.apply_event(record.event.clone(), authorization)?;
        }
        Ok(controller)
    }"""
src = replace_once(src, old_replay, new_replay, "trusted replay")

# Add metadata-based fallback and semantic replay validation before version checking.
require_marker = """    fn require_version(&self, expected_version: u64) -> Result<(), LivePolicyError> {"""
validation_methods = r'''    fn fallback_reason_from_metadata(
        &self,
        subject_scope_digest: u64,
        turn_digest: u64,
        current_companion_version: u64,
        planned_at_ms: u64,
        sensitive_context: bool,
        intent_label: &str,
    ) -> Option<NeutralFallbackReason> {
        let Some(lease) = self.active_lease.as_ref() else {
            return Some(NeutralFallbackReason::Disabled);
        };
        if self.seen_turns.contains(&turn_digest) {
            return Some(NeutralFallbackReason::DuplicateTurn);
        }
        if current_companion_version != lease.source_companion_version {
            return Some(NeutralFallbackReason::SourceVersionMismatch);
        }
        if planned_at_ms < lease.valid_from_ms {
            return Some(NeutralFallbackReason::NotYetValid);
        }
        if planned_at_ms >= lease.expires_at_ms {
            return Some(NeutralFallbackReason::Expired);
        }
        if lease.remaining_turns == 0 {
            return Some(NeutralFallbackReason::BudgetExhausted);
        }
        if subject_scope_digest != lease.subject_scope_digest {
            return Some(NeutralFallbackReason::SubjectMismatch);
        }
        if sensitive_context {
            return Some(NeutralFallbackReason::SensitiveContext);
        }
        if !intent_label_allowed(intent_label) {
            return Some(NeutralFallbackReason::DisallowedIntent);
        }
        None
    }

    fn validate_turn_event(&self, event: &LivePolicyEvent) -> Result<(), LivePolicyError> {
        let LivePolicyEvent::TurnPlanned {
            subject_scope_digest,
            turn_digest,
            context_digest,
            planned_at_ms,
            current_companion_version,
            sensitive_context,
            intent_label,
            disposition,
            fallback_reason,
            baseline_plan_digest_fnv1a64,
            planned_plan_digest_fnv1a64,
            baseline_max_chars,
            max_chars_after,
            baseline_style,
            style_after,
            remaining_turns_after,
        } = event
        else {
            return Ok(());
        };
        if *subject_scope_digest == 0
            || *turn_digest == 0
            || *context_digest == 0
            || *current_companion_version == 0
            || *baseline_plan_digest_fnv1a64 == 0
            || *planned_plan_digest_fnv1a64 == 0
        {
            return Err(LivePolicyError::MalformedReplayTurn);
        }
        let expected_reason = self.fallback_reason_from_metadata(
            *subject_scope_digest,
            *turn_digest,
            *current_companion_version,
            *planned_at_ms,
            *sensitive_context,
            intent_label,
        );
        match (expected_reason, disposition) {
            (None, LivePlanDisposition::Applied) => {
                if fallback_reason.is_some()
                    || baseline_plan_digest_fnv1a64 == planned_plan_digest_fnv1a64
                {
                    return Err(LivePolicyError::ReplayAppliedPlanMismatch);
                }
                let lease = self
                    .active_lease
                    .as_ref()
                    .ok_or(LivePolicyError::AuditWithoutActivation)?;
                let expected_max = planned_max_chars(
                    lease.policy.detail,
                    *baseline_max_chars,
                    self.config.max_output_chars,
                );
                let expected_style = planned_style_label(
                    lease.policy.explanation_style,
                    baseline_style.as_deref(),
                    intent_label,
                );
                if *max_chars_after != Some(expected_max)
                    || *style_after != expected_style
                    || *remaining_turns_after != lease.remaining_turns.saturating_sub(1)
                {
                    return Err(LivePolicyError::ReplayAppliedPlanMismatch);
                }
            }
            (Some(expected), LivePlanDisposition::NeutralFallback) => {
                let remaining = self
                    .active_lease
                    .as_ref()
                    .map_or(0, |lease| lease.remaining_turns);
                if *fallback_reason != Some(expected)
                    || baseline_plan_digest_fnv1a64 != planned_plan_digest_fnv1a64
                    || baseline_max_chars != max_chars_after
                    || baseline_style != style_after
                    || *remaining_turns_after != remaining
                {
                    return Err(LivePolicyError::ReplayNeutralPlanMismatch);
                }
            }
            _ => return Err(LivePolicyError::ReplaySemanticMismatch),
        }
        Ok(())
    }

    fn require_version(&self, expected_version: u64) -> Result<(), LivePolicyError> {'''
src = replace_once(src, require_marker, validation_methods, "semantic validation methods")

# Delegate ordinary fallback classification to the same replay-checkable metadata path.
old_fallback = """    fn fallback_reason(
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
        if context.current_companion_version != lease.source_companion_version {
            return Some(NeutralFallbackReason::SourceVersionMismatch);
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
    }"""
new_fallback = """    fn fallback_reason(
        &self,
        context: &LivePolicyPlanningContext,
        intent: &ResponseIntent,
    ) -> Option<NeutralFallbackReason> {
        self.fallback_reason_from_metadata(
            context.subject_scope_digest,
            context.turn_digest,
            context.current_companion_version,
            context.planned_at_ms,
            context.sensitive_context,
            intent.label(),
        )
    }"""
src = replace_once(src, old_fallback, new_fallback, "fallback delegation")

# Apply events only after trusted activation and deterministic semantic validation.
src = replace_once(
    src,
    """    fn apply_event(
        &mut self,
        event: LivePolicyEvent,
    ) -> Result<LivePolicyTransition, LivePolicyError> {
        match &event {
            LivePolicyEvent::Activated { lease } => {
                if self.active_lease.is_some() {
                    return Err(LivePolicyError::AlreadyActive);
                }
                self.active_lease = Some(lease.clone());""",
    """    fn apply_event(
        &mut self,
        event: LivePolicyEvent,
        authorization: Option<&ValidatedLivePolicyAuthorization>,
    ) -> Result<LivePolicyTransition, LivePolicyError> {
        match &event {
            LivePolicyEvent::Activated { lease } => {
                if self.active_lease.is_some() {
                    return Err(LivePolicyError::AlreadyActive);
                }
                let authorization = authorization.ok_or(
                    LivePolicyError::UnknownReplayAuthorization(
                        lease.authorization_digest_fnv1a64,
                    ),
                )?;
                validate_replay_lease(lease, authorization, self.config)?;
                self.active_lease = Some(lease.clone());""",
    "apply event trusted activation",
)
src = replace_once(
    src,
    """            LivePolicyEvent::TurnPlanned {
                turn_digest,
                disposition,""",
    """            LivePolicyEvent::TurnPlanned {
                turn_digest,
                disposition,""",
    "turn arm anchor",
)
# Inject semantic validation immediately inside the turn arm.
src = replace_once(
    src,
    """                ..
            } => {
                let duplicate_fallback = *disposition == LivePlanDisposition::NeutralFallback""",
    """                ..
            } => {
                self.validate_turn_event(&event)?;
                let duplicate_fallback = *disposition == LivePlanDisposition::NeutralFallback""",
    "turn semantic validation call",
)
src = replace_once(
    src,
    """            LivePolicyEvent::Revoked { .. } => {
                if self.active_lease.take().is_none() {""",
    """            LivePolicyEvent::Revoked {
                revoked_at_ms,
                reason_digest,
            } => {
                if *revoked_at_ms == 0 || *reason_digest == 0 {
                    return Err(LivePolicyError::EmptyRevocationEvidence);
                }
                if self.active_lease.take().is_none() {""",
    "revocation replay validation",
)

# Canonical proposal/lease checks and audit-chain validation.
validate_marker = """fn validate_proposal(
    proposal: &ShadowPolicyProposal,"""
helpers = r'''fn canonical_claim_ids(
    proposal: &ShadowPolicyProposal,
) -> Result<Vec<u64>, LivePolicyError> {
    let mut claim_ids = proposal.source_claim_ids();
    if claim_ids.is_empty() || claim_ids.iter().any(|claim_id| *claim_id == 0) {
        return Err(LivePolicyError::InvalidAuthorizationClaims);
    }
    claim_ids.sort_unstable();
    if claim_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(LivePolicyError::InvalidAuthorizationClaims);
    }
    Ok(claim_ids)
}

fn validate_authorized_proposal(
    authorization: &ValidatedLivePolicyAuthorization,
    proposal: &ShadowPolicyProposal,
) -> Result<(), LivePolicyError> {
    let claim_ids = canonical_claim_ids(proposal)?;
    if authorization.source_companion_version != proposal.source_companion_version
        || authorization.source_policy_digest_fnv1a64 != proposal.policy_digest_fnv1a64
        || authorization.source_claim_ids != claim_ids
        || authorization.confidence_bps != proposal.confidence_bps
        || authorization.policy != proposal.policy
    {
        return Err(LivePolicyError::AuthorizationProposalMismatch);
    }
    Ok(())
}

fn validate_replay_lease(
    lease: &LivePolicyLease,
    authorization: &ValidatedLivePolicyAuthorization,
    config: LivePolicyControllerConfig,
) -> Result<(), LivePolicyError> {
    if authorization.digest_fnv1a64 != lease.authorization_digest_fnv1a64
        || authorization.promotion_gate_digest_fnv1a64
            != lease.promotion_gate_digest_fnv1a64
        || authorization.evidence_class != lease.evidence_class
        || authorization.source_companion_version != lease.source_companion_version
        || authorization.source_policy_digest_fnv1a64
            != lease.source_policy_digest_fnv1a64
        || authorization.source_claim_ids != lease.source_claim_ids
        || authorization.confidence_bps != lease.confidence_bps
        || authorization.policy != lease.policy
    {
        return Err(LivePolicyError::ReplayAuthorizationMismatch);
    }
    if lease.source_policy_digest_fnv1a64 == 0
        || lease.source_claim_ids.is_empty()
        || lease.source_claim_ids.iter().any(|claim_id| *claim_id == 0)
        || lease
            .source_claim_ids
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        || lease.confidence_bps < config.min_confidence_bps
    {
        return Err(LivePolicyError::MalformedReplayLease);
    }
    if lease.evidence_class == EvaluationEvidenceClass::FrozenSimulation
        && !config.allow_simulated_activation
    {
        return Err(LivePolicyError::SimulatedEvidenceRejected);
    }
    validate_activation_request(
        &LivePolicyActivationRequest {
            subject_scope_digest: lease.subject_scope_digest,
            valid_from_ms: lease.valid_from_ms,
            expires_at_ms: lease.expires_at_ms,
            max_turns: lease.remaining_turns,
            operator_approval_digest: lease.operator_approval_digest,
        },
        config,
    )
    .map_err(|_| LivePolicyError::MalformedReplayLease)
}

fn validate_audit_chain(records: &[LivePolicyAuditRecord]) -> Result<(), LivePolicyError> {
    let mut previous = 0_u64;
    for (index, record) in records.iter().enumerate() {
        let expected_sequence = index as u64 + 1;
        if record.sequence != expected_sequence {
            return Err(LivePolicyError::AuditSequenceMismatch);
        }
        if record.previous_chain_digest_fnv1a64 != previous {
            return Err(LivePolicyError::AuditPreviousDigestMismatch);
        }
        let event_digest = live_policy_event_digest(&record.event);
        if record.event_digest_fnv1a64 != event_digest {
            return Err(LivePolicyError::AuditEventDigestMismatch);
        }
        let chain_digest = live_policy_chain_digest(previous, expected_sequence, event_digest);
        if record.chain_digest_fnv1a64 != chain_digest {
            return Err(LivePolicyError::AuditChainDigestMismatch);
        }
        previous = chain_digest;
    }
    Ok(())
}

fn live_policy_event_digest(event: &LivePolicyEvent) -> u64 {
    let encoded = serde_json::to_vec(event).expect("serializing typed live-policy event cannot fail");
    fnv1a64(&encoded)
}

fn live_policy_chain_digest(previous: u64, sequence: u64, event_digest: u64) -> u64 {
    let mut digest = fnv1a64(b"s6b-live-policy-audit-chain-v1");
    digest = mix_u64(digest, previous);
    digest = mix_u64(digest, sequence);
    mix_u64(digest, event_digest)
}

fn validate_proposal(
    proposal: &ShadowPolicyProposal,'''
src = replace_once(src, validate_marker, helpers, "proposal and replay helpers")

# Ensure valid live activations store canonical claim IDs.
src = replace_once(
    src,
    """            source_claim_ids: proposal.source_claim_ids(),""",
    """            source_claim_ids: canonical_claim_ids(proposal)?,""",
    "canonical lease claims",
)

# Centralize deterministic planning metadata calculations.
src = replace_once(
    src,
    """    let baseline_max = rerank_config
        .max_chars
        .unwrap_or(DEFAULT_STANDARD_MAX_CHARS);
    let planned_max = match policy.detail {
        DetailLevel::Brief => baseline_max.min(DEFAULT_BRIEF_MAX_CHARS),
        DetailLevel::Standard => baseline_max.min(max_output_chars),
        DetailLevel::Detailed => baseline_max
            .max(DEFAULT_DETAILED_MAX_CHARS)
            .min(max_output_chars),
    };
    rerank_config.max_chars = Some(planned_max);""",
    """    rerank_config.max_chars = Some(planned_max_chars(
        policy.detail,
        rerank_config.max_chars,
        max_output_chars,
    ));""",
    "planned max helper use",
)
intent_marker = """fn intent_allowed(intent: &ResponseIntent) -> bool {
    matches!(
        intent,
        ResponseIntent::Recall
            | ResponseIntent::Teaching
            | ResponseIntent::Capability
            | ResponseIntent::ResearchStatus
    )
}

fn apply_policy("""
intent_helpers = r'''fn intent_allowed(intent: &ResponseIntent) -> bool {
    intent_label_allowed(intent.label())
}

fn intent_label_allowed(intent_label: &str) -> bool {
    matches!(
        intent_label,
        "recall" | "teaching" | "capability" | "research_status"
    )
}

fn default_style_label_for_intent(intent_label: &str) -> Option<String> {
    match intent_label {
        "recall" | "research_status" => Some("analytical".to_owned()),
        "teaching" | "capability" => Some("direct".to_owned()),
        _ => None,
    }
}

fn planned_style_label(
    explanation_style: ExplanationStyle,
    baseline_style: Option<&str>,
    intent_label: &str,
) -> Option<String> {
    match explanation_style {
        ExplanationStyle::Concrete => Some("direct".to_owned()),
        ExplanationStyle::Abstract => Some("analytical".to_owned()),
        ExplanationStyle::Adaptive => baseline_style
            .map(str::to_owned)
            .or_else(|| default_style_label_for_intent(intent_label)),
    }
}

fn planned_max_chars(
    detail: DetailLevel,
    baseline_max_chars: Option<usize>,
    max_output_chars: usize,
) -> usize {
    let baseline_max = baseline_max_chars.unwrap_or(DEFAULT_STANDARD_MAX_CHARS);
    match detail {
        DetailLevel::Brief => baseline_max.min(DEFAULT_BRIEF_MAX_CHARS),
        DetailLevel::Standard => baseline_max.min(max_output_chars),
        DetailLevel::Detailed => baseline_max
            .max(DEFAULT_DETAILED_MAX_CHARS)
            .min(max_output_chars),
    }
}

fn apply_policy('''
src = replace_once(src, intent_marker, intent_helpers, "planning helpers")

# Add explicit S6-B replay errors.
error_marker = """    #[error("controller version overflow")]
    VersionOverflow,
}"""
errors = r'''    #[error("controller version overflow")]
    VersionOverflow,
    #[error("proposal does not match the exact live-policy authorization")]
    AuthorizationProposalMismatch,
    #[error("authorization claim IDs must be unique, non-zero, and non-empty")]
    InvalidAuthorizationClaims,
    #[error("no trusted replay authorization matches digest {0}")]
    UnknownReplayAuthorization(u64),
    #[error("replayed activation does not match its trusted authorization")]
    ReplayAuthorizationMismatch,
    #[error("replayed activation lease is malformed")]
    MalformedReplayLease,
    #[error("replayed turn record is malformed")]
    MalformedReplayTurn,
    #[error("audit record sequence is invalid")]
    AuditSequenceMismatch,
    #[error("audit record previous digest is invalid")]
    AuditPreviousDigestMismatch,
    #[error("audit record event digest is invalid")]
    AuditEventDigestMismatch,
    #[error("audit record chain digest is invalid")]
    AuditChainDigestMismatch,
    #[error("replayed event disposition does not match controller state")]
    ReplaySemanticMismatch,
    #[error("replayed neutral fallback is not exactly baseline-preserving")]
    ReplayNeutralPlanMismatch,
    #[error("replayed applied plan does not match the authorized bounded policy")]
    ReplayAppliedPlanMismatch,
}'''
src = replace_once(src, error_marker, errors, "S6-B errors")

# Update all source tests to create and use an exact authorization.
pattern = re.compile(
    r"(        let gate = ValidatedPromotionGate::validate\([\s\S]*?\n        \)\n        \.unwrap\(\);)"
)

def add_auth(match: re.Match[str]) -> str:
    block = match.group(1)
    return block + "\n        let authorization = gate.authorize_proposal(&proposal()).unwrap();"

src, auth_count = pattern.subn(add_auth, src)
if auth_count < 5:
    raise SystemExit(f"source test authorization insertion: expected >=5, got {auth_count}")
src = src.replace("            .activate(0, &gate, &proposal(), activation())", "            .activate(0, &authorization, &proposal(), activation())")
src = src.replace(
    "BoundedLivePolicyController::replay(config, controller.events()).unwrap()",
    "BoundedLivePolicyController::replay(\n            config,\n            std::slice::from_ref(&authorization),\n            &controller.audit_records(),\n        )\n        .unwrap()",
)

SRC.write_text(src)

# Update the frozen S6-A probe to the exact authorization and trusted replay API.
probe = PROBE.read_text()
probe = replace_once(
    probe,
    """    let promotion_gate_validated = gate.digest_fnv1a64() != 0;

    let mut production""",
    """    let promotion_gate_validated = gate.digest_fnv1a64() != 0;
    let authorization = gate.authorize_proposal(&proposal()).unwrap();

    let mut production""",
    "probe authorization creation",
)
probe = replace_all_checked(probe, "            &gate,", "            &authorization,", 3, "probe activation authorization")
probe = replace_once(
    probe,
    "BoundedLivePolicyController::replay(config, controller.events()).unwrap()",
    "BoundedLivePolicyController::replay(\n        config,\n        std::slice::from_ref(&authorization),\n        &controller.audit_records(),\n    )\n    .unwrap()",
    "probe trusted replay",
)
PROBE.write_text(probe)
