//! S6-A reversible live interaction-policy canary.
//!
//! This module is the first boundary allowed to return a companion-derived
//! response-planning policy. It remains disabled by default, requires an
//! explicitly installed held-out S5-C authorization, applies only to a bounded
//! deterministic rollout, and falls back to the neutral policy on every gate
//! failure. It has no persistence, belief, ontology, routing, or action authority.

use crate::companion_interaction_policy::{
    InteractionPolicy, PolicyContext, PolicyVariant, ShadowPolicyError, ShadowPolicyPlanner,
    ShadowPolicyPlannerConfig,
};
use crate::companion_policy_evaluation::{
    EvaluationSplit, PolicyEvaluationReport, PolicyEvaluationVerdict,
};
use crate::companion_state::{ClaimId, CompanionState, Sensitivity};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanaryMode {
    Disabled,
    AuditOnly,
    LiveCanary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionEvidenceClass {
    SyntheticConformance,
    RealHeldOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePolicyCanaryConfig {
    pub mode: CanaryMode,
    pub rollout_modulus: u64,
    pub rollout_remainder: u64,
    pub min_confidence_bps: u16,
    pub include_session_claims: bool,
    pub max_source_claims: usize,
    pub max_planning_compute_micros: u64,
}

impl Default for LivePolicyCanaryConfig {
    fn default() -> Self {
        Self {
            mode: CanaryMode::Disabled,
            rollout_modulus: 100,
            rollout_remainder: 0,
            min_confidence_bps: 7_000,
            include_session_claims: true,
            max_source_claims: 8,
            max_planning_compute_micros: 5_000,
        }
    }
}

impl LivePolicyCanaryConfig {
    pub fn validate(self) -> Result<Self, LivePolicyCanaryError> {
        if self.rollout_modulus < 2 || self.rollout_remainder >= self.rollout_modulus {
            return Err(LivePolicyCanaryError::InvalidConfig(
                "rollout partition must have modulus >= 2 and remainder < modulus",
            ));
        }
        if self.min_confidence_bps > 10_000 {
            return Err(LivePolicyCanaryError::InvalidConfig(
                "minimum confidence must be within basis-point range",
            ));
        }
        if self.max_source_claims == 0 || self.max_source_claims > 32 {
            return Err(LivePolicyCanaryError::InvalidConfig(
                "source-claim budget must be between 1 and 32",
            ));
        }
        if self.max_planning_compute_micros == 0 {
            return Err(LivePolicyCanaryError::InvalidConfig(
                "planning compute budget must be positive",
            ));
        }
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionAuthorization {
    evidence_class: PromotionEvidenceClass,
    artifact_digest: String,
    report_fingerprint_fnv1a64: u64,
    authorized_companion_version: u64,
}

impl PromotionAuthorization {
    pub fn from_report(
        report: &PolicyEvaluationReport,
        evidence_class: PromotionEvidenceClass,
        artifact_digest: impl Into<String>,
        authorized_companion_version: u64,
    ) -> Result<Self, LivePolicyCanaryError> {
        validate_promotion_report(report)?;
        if authorized_companion_version == 0 {
            return Err(LivePolicyCanaryError::InvalidAuthorizedCompanionVersion);
        }
        let artifact_digest = artifact_digest.into();
        validate_sha256_digest(&artifact_digest)?;
        let report_fingerprint_fnv1a64 = fingerprint(report)?;
        Ok(Self {
            evidence_class,
            artifact_digest,
            report_fingerprint_fnv1a64,
            authorized_companion_version,
        })
    }

    #[must_use]
    pub const fn evidence_class(&self) -> PromotionEvidenceClass {
        self.evidence_class
    }

    #[must_use]
    pub fn artifact_digest(&self) -> &str {
        &self.artifact_digest
    }

    #[must_use]
    pub const fn report_fingerprint_fnv1a64(&self) -> u64 {
        self.report_fingerprint_fnv1a64
    }

    #[must_use]
    pub const fn authorized_companion_version(&self) -> u64 {
        self.authorized_companion_version
    }

    #[must_use]
    pub const fn permits_live_use(&self) -> bool {
        matches!(self.evidence_class, PromotionEvidenceClass::RealHeldOut)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanaryFallbackReason {
    Disabled,
    AuditOnly,
    RolloutExcluded,
    MissingAuthorization,
    SyntheticEvidenceOnly,
    SourceVersionMismatch,
    CandidateAbstained,
    SensitiveEvidence,
    SourceClaimBudgetExceeded,
    ComputeBudgetExceeded,
    RollbackLatched,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanaryRollbackReason {
    EvaluationFailed,
    EvaluationInconclusive,
    InvariantViolation,
    OperatorRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanaryAuditEventKind {
    AuthorizationInstalled {
        evidence_class: PromotionEvidenceClass,
        artifact_digest: String,
        report_fingerprint_fnv1a64: u64,
        authorized_companion_version: u64,
    },
    AuthorizationRemoved {
        verdict: PolicyEvaluationVerdict,
    },
    Decision {
        decision_id: u64,
        source_companion_version: u64,
        context_digest: u64,
        subject_scope_digest: u64,
        source_claim_ids: Vec<ClaimId>,
        candidate_policy_digest_fnv1a64: u64,
        effective_policy_digest_fnv1a64: u64,
        selected_variant: PolicyVariant,
        live_influence_applied: bool,
        fallback_reason: Option<CanaryFallbackReason>,
        planning_compute_micros: u64,
        rollback_generation: u64,
    },
    RollbackLatched {
        generation: u64,
        reason: CanaryRollbackReason,
    },
    RollbackCleared {
        generation: u64,
        operator_digest: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryAuditEvent {
    pub sequence: u64,
    pub previous_digest_fnv1a64: u64,
    pub digest_fnv1a64: u64,
    pub kind: CanaryAuditEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivePolicyDecision {
    pub decision_id: u64,
    pub source_companion_version: u64,
    pub source_claim_ids: Vec<ClaimId>,
    pub candidate_policy: InteractionPolicy,
    pub effective_policy: InteractionPolicy,
    pub selected_variant: PolicyVariant,
    pub candidate_policy_digest_fnv1a64: u64,
    pub effective_policy_digest_fnv1a64: u64,
    pub live_influence_applied: bool,
    pub fallback_reason: Option<CanaryFallbackReason>,
    pub planning_compute_micros: u64,
    pub rollback_generation: u64,
    pub audit_sequence: u64,
    pub audit_digest_fnv1a64: u64,
}

#[derive(Debug, Clone)]
pub struct LivePolicyCanary {
    config: LivePolicyCanaryConfig,
    planner: ShadowPolicyPlanner,
    authorization: Option<PromotionAuthorization>,
    rollback_reason: Option<CanaryRollbackReason>,
    rollback_generation: u64,
    next_decision_id: u64,
    audit: Vec<CanaryAuditEvent>,
}

impl LivePolicyCanary {
    pub fn new(config: LivePolicyCanaryConfig) -> Result<Self, LivePolicyCanaryError> {
        let config = config.validate()?;
        let planner = ShadowPolicyPlanner::new(ShadowPolicyPlannerConfig {
            min_confidence_bps: config.min_confidence_bps,
            include_session_claims: config.include_session_claims,
            include_sensitive_claims: false,
            max_source_claims: config.max_source_claims.saturating_add(1),
        })?;
        Ok(Self {
            config,
            planner,
            authorization: None,
            rollback_reason: None,
            rollback_generation: 0,
            next_decision_id: 1,
            audit: Vec::new(),
        })
    }

    #[must_use]
    pub const fn config(&self) -> LivePolicyCanaryConfig {
        self.config
    }

    #[must_use]
    pub fn authorization(&self) -> Option<&PromotionAuthorization> {
        self.authorization.as_ref()
    }

    #[must_use]
    pub const fn rollback_reason(&self) -> Option<CanaryRollbackReason> {
        self.rollback_reason
    }

    #[must_use]
    pub const fn rollback_generation(&self) -> u64 {
        self.rollback_generation
    }

    #[must_use]
    pub fn audit(&self) -> &[CanaryAuditEvent] {
        &self.audit
    }

    pub fn install_authorization(
        &mut self,
        authorization: PromotionAuthorization,
    ) -> Result<CanaryAuditEvent, LivePolicyCanaryError> {
        self.authorization = Some(authorization.clone());
        self.append_audit(CanaryAuditEventKind::AuthorizationInstalled {
            evidence_class: authorization.evidence_class,
            artifact_digest: authorization.artifact_digest,
            report_fingerprint_fnv1a64: authorization.report_fingerprint_fnv1a64,
            authorized_companion_version: authorization.authorized_companion_version,
        })
    }

    pub fn apply_evaluation_update(
        &mut self,
        report: &PolicyEvaluationReport,
        evidence_class: PromotionEvidenceClass,
        artifact_digest: impl Into<String>,
        authorized_companion_version: u64,
    ) -> Result<CanaryAuditEvent, LivePolicyCanaryError> {
        match report.verdict {
            PolicyEvaluationVerdict::Pass => {
                let authorization =
                    PromotionAuthorization::from_report(
                        report,
                        evidence_class,
                        artifact_digest,
                        authorized_companion_version,
                    )?;
                self.install_authorization(authorization)
            }
            PolicyEvaluationVerdict::Fail => {
                self.authorization = None;
                self.append_audit(CanaryAuditEventKind::AuthorizationRemoved {
                    verdict: report.verdict,
                })?;
                self.latch_rollback(CanaryRollbackReason::EvaluationFailed)
            }
            PolicyEvaluationVerdict::Inconclusive => {
                self.authorization = None;
                self.append_audit(CanaryAuditEventKind::AuthorizationRemoved {
                    verdict: report.verdict,
                })?;
                self.latch_rollback(CanaryRollbackReason::EvaluationInconclusive)
            }
        }
    }

    pub fn latch_rollback(
        &mut self,
        reason: CanaryRollbackReason,
    ) -> Result<CanaryAuditEvent, LivePolicyCanaryError> {
        self.rollback_generation = self.rollback_generation.saturating_add(1);
        self.rollback_reason = Some(reason);
        self.append_audit(CanaryAuditEventKind::RollbackLatched {
            generation: self.rollback_generation,
            reason,
        })
    }

    pub fn clear_rollback(
        &mut self,
        expected_generation: u64,
        operator_digest: u64,
    ) -> Result<CanaryAuditEvent, LivePolicyCanaryError> {
        if self.rollback_reason.is_none() {
            return Err(LivePolicyCanaryError::RollbackNotLatched);
        }
        if expected_generation != self.rollback_generation {
            return Err(LivePolicyCanaryError::RollbackGenerationConflict {
                expected: expected_generation,
                actual: self.rollback_generation,
            });
        }
        if operator_digest == 0 {
            return Err(LivePolicyCanaryError::InvalidOperatorDigest);
        }
        self.rollback_reason = None;
        self.append_audit(CanaryAuditEventKind::RollbackCleared {
            generation: self.rollback_generation,
            operator_digest,
        })
    }

    pub fn decide(
        &mut self,
        state: &CompanionState,
        context: PolicyContext,
        planning_compute_micros: u64,
    ) -> Result<LivePolicyDecision, LivePolicyCanaryError> {
        let batch = self.planner.plan(state, context.clone())?;
        let candidate = batch
            .proposal(PolicyVariant::CompanionDerived)
            .expect("S5-A planner always emits the companion-derived arm")
            .clone();
        let neutral = batch
            .proposal(PolicyVariant::NeutralDefault)
            .expect("S5-A planner always emits the neutral arm")
            .clone();

        let fallback_reason = self.fallback_reason(
            batch.source_companion_version,
            context.subject_scope_digest,
            &candidate,
            planning_compute_micros,
        );
        let live_influence_applied = fallback_reason.is_none();
        let selected = if live_influence_applied {
            &candidate
        } else {
            &neutral
        };
        let decision_id = self.next_decision_id;
        self.next_decision_id = self.next_decision_id.saturating_add(1);
        let source_claim_ids = candidate.source_claim_ids();
        let event = self.append_audit(CanaryAuditEventKind::Decision {
            decision_id,
            source_companion_version: batch.source_companion_version,
            context_digest: context.context_digest,
            subject_scope_digest: context.subject_scope_digest,
            source_claim_ids: source_claim_ids.clone(),
            candidate_policy_digest_fnv1a64: candidate.policy_digest_fnv1a64,
            effective_policy_digest_fnv1a64: selected.policy_digest_fnv1a64,
            selected_variant: selected.variant,
            live_influence_applied,
            fallback_reason,
            planning_compute_micros,
            rollback_generation: self.rollback_generation,
        })?;

        Ok(LivePolicyDecision {
            decision_id,
            source_companion_version: batch.source_companion_version,
            source_claim_ids,
            candidate_policy: candidate.policy,
            effective_policy: selected.policy,
            selected_variant: selected.variant,
            candidate_policy_digest_fnv1a64: candidate.policy_digest_fnv1a64,
            effective_policy_digest_fnv1a64: selected.policy_digest_fnv1a64,
            live_influence_applied,
            fallback_reason,
            planning_compute_micros,
            rollback_generation: self.rollback_generation,
            audit_sequence: event.sequence,
            audit_digest_fnv1a64: event.digest_fnv1a64,
        })
    }

    fn fallback_reason(
        &self,
        source_companion_version: u64,
        subject_scope_digest: u64,
        candidate: &crate::companion_interaction_policy::ShadowPolicyProposal,
        planning_compute_micros: u64,
    ) -> Option<CanaryFallbackReason> {
        if self.rollback_reason.is_some() {
            return Some(CanaryFallbackReason::RollbackLatched);
        }
        match self.config.mode {
            CanaryMode::Disabled => return Some(CanaryFallbackReason::Disabled),
            CanaryMode::AuditOnly => return Some(CanaryFallbackReason::AuditOnly),
            CanaryMode::LiveCanary => {}
        }
        if subject_scope_digest % self.config.rollout_modulus != self.config.rollout_remainder {
            return Some(CanaryFallbackReason::RolloutExcluded);
        }
        let Some(authorization) = &self.authorization else {
            return Some(CanaryFallbackReason::MissingAuthorization);
        };
        if !authorization.permits_live_use() {
            return Some(CanaryFallbackReason::SyntheticEvidenceOnly);
        }
        if authorization.authorized_companion_version != source_companion_version {
            return Some(CanaryFallbackReason::SourceVersionMismatch);
        }
        if candidate.is_abstention() {
            return Some(CanaryFallbackReason::CandidateAbstained);
        }
        if candidate
            .evidence
            .iter()
            .any(|evidence| evidence.sensitivity == Sensitivity::Sensitive)
        {
            return Some(CanaryFallbackReason::SensitiveEvidence);
        }
        if candidate.evidence.len() > self.config.max_source_claims {
            return Some(CanaryFallbackReason::SourceClaimBudgetExceeded);
        }
        if planning_compute_micros == 0
            || planning_compute_micros > self.config.max_planning_compute_micros
        {
            return Some(CanaryFallbackReason::ComputeBudgetExceeded);
        }
        None
    }

    fn append_audit(
        &mut self,
        kind: CanaryAuditEventKind,
    ) -> Result<CanaryAuditEvent, LivePolicyCanaryError> {
        let sequence = self.audit.len() as u64 + 1;
        let previous_digest_fnv1a64 = self
            .audit
            .last()
            .map_or(0, |event| event.digest_fnv1a64);
        let digest_fnv1a64 = audit_digest(sequence, previous_digest_fnv1a64, &kind)?;
        let event = CanaryAuditEvent {
            sequence,
            previous_digest_fnv1a64,
            digest_fnv1a64,
            kind,
        };
        self.audit.push(event.clone());
        Ok(event)
    }
}

#[must_use]
pub fn verify_audit_chain(events: &[CanaryAuditEvent]) -> bool {
    let mut previous = 0_u64;
    for (index, event) in events.iter().enumerate() {
        if event.sequence != index as u64 + 1 || event.previous_digest_fnv1a64 != previous {
            return false;
        }
        let Ok(expected) = audit_digest(event.sequence, previous, &event.kind) else {
            return false;
        };
        if expected != event.digest_fnv1a64 {
            return false;
        }
        previous = event.digest_fnv1a64;
    }
    true
}

fn validate_promotion_report(
    report: &PolicyEvaluationReport,
) -> Result<(), LivePolicyCanaryError> {
    if report.verdict != PolicyEvaluationVerdict::Pass || !report.promotion_eligible {
        return Err(LivePolicyCanaryError::PromotionReportNotEligible);
    }
    if !report.development_excluded_from_verdict {
        return Err(LivePolicyCanaryError::InvalidPromotionReport(
            "development evidence was not excluded from the verdict",
        ));
    }
    if report.live_response_influence
        || report.routing_authority
        || report.belief_promotion_authority
        || report.action_authority
    {
        return Err(LivePolicyCanaryError::InvalidPromotionReport(
            "S5-C report already claims authority it cannot grant",
        ));
    }

    let controls = [
        PolicyVariant::NeutralDefault,
        PolicyVariant::RecencyOnly,
        PolicyVariant::MajorityPrior,
        PolicyVariant::ContextOnly,
        PolicyVariant::ScrambledScope,
    ];
    for split in [
        EvaluationSplit::OpaqueSubjectHoldout,
        EvaluationSplit::TemporalHoldout,
    ] {
        for control in controls {
            let Some(comparison) = report
                .holdout_comparisons
                .iter()
                .find(|comparison| comparison.split == split && comparison.control == control)
            else {
                return Err(LivePolicyCanaryError::InvalidPromotionReport(
                    "a required held-out candidate-control comparison is missing",
                ));
            };
            if !comparison.gates.evidence_sufficient()
                || !comparison.gates.performance_passed()
            {
                return Err(LivePolicyCanaryError::InvalidPromotionReport(
                    "a held-out comparison did not pass all evidence and performance gates",
                ));
            }
        }
    }
    if report.holdout_comparisons.len() != 10 {
        return Err(LivePolicyCanaryError::InvalidPromotionReport(
            "the report must contain exactly ten held-out comparisons",
        ));
    }
    Ok(())
}

fn validate_sha256_digest(digest: &str) -> Result<(), LivePolicyCanaryError> {
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Err(LivePolicyCanaryError::InvalidArtifactDigest);
    };
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(LivePolicyCanaryError::InvalidArtifactDigest);
    }
    Ok(())
}

fn fingerprint<T: Serialize>(value: &T) -> Result<u64, LivePolicyCanaryError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| LivePolicyCanaryError::AuditSerialization(error.to_string()))?;
    Ok(fnv1a64(&bytes))
}

fn audit_digest(
    sequence: u64,
    previous_digest_fnv1a64: u64,
    kind: &CanaryAuditEventKind,
) -> Result<u64, LivePolicyCanaryError> {
    fingerprint(&(sequence, previous_digest_fnv1a64, kind))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[derive(Debug, Error)]
pub enum LivePolicyCanaryError {
    #[error("invalid S6-A configuration: {0}")]
    InvalidConfig(&'static str),
    #[error("S5-C promotion report is not eligible")]
    PromotionReportNotEligible,
    #[error("invalid S5-C promotion report: {0}")]
    InvalidPromotionReport(&'static str),
    #[error("artifact digest must be sha256 followed by exactly 64 hexadecimal characters")]
    InvalidArtifactDigest,
    #[error("authorized companion version must be positive")]
    InvalidAuthorizedCompanionVersion,
    #[error("rollback is not latched")]
    RollbackNotLatched,
    #[error("rollback generation conflict: expected {expected}, actual {actual}")]
    RollbackGenerationConflict { expected: u64, actual: u64 },
    #[error("operator digest must be non-zero")]
    InvalidOperatorDigest,
    #[error("audit serialization failed: {0}")]
    AuditSerialization(String),
    #[error(transparent)]
    ShadowPolicy(#[from] ShadowPolicyError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion_policy_evaluation::{
        CandidateControlComparison, CandidatePairwiseMetrics, ComparisonGates,
        PolicyEvaluationReport,
    };

    fn gates() -> ComparisonGates {
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
            for control in [
                PolicyVariant::NeutralDefault,
                PolicyVariant::RecencyOnly,
                PolicyVariant::MajorityPrior,
                PolicyVariant::ContextOnly,
                PolicyVariant::ScrambledScope,
            ] {
                comparisons.push(CandidateControlComparison {
                    split,
                    control,
                    candidate_resolved: 2,
                    control_resolved: 2,
                    candidate_direct_outcomes: 1,
                    control_direct_outcomes: 1,
                    pairwise: CandidatePairwiseMetrics {
                        candidate_wins: 1,
                        control_wins: 0,
                        ties: 0,
                        total: 1,
                        candidate_win_margin_bps: Some(10_000),
                    },
                    brier_improvement_ppm: Some(10_000),
                    calibration_regression_bps: Some(0),
                    correction_regression_bps: Some(0),
                    clarification_regression_bps: Some(0),
                    completion_regression_bps: Some(0),
                    abandonment_regression_bps: Some(0),
                    abstention_regression_bps: Some(0),
                    compute_overhead_bps: Some(0),
                    gates: gates(),
                });
            }
        }
        PolicyEvaluationReport {
            splits: Vec::new(),
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

    #[test]
    fn synthetic_authorization_never_permits_live_use() {
        let authorization = PromotionAuthorization::from_report(
            &passing_report(),
            PromotionEvidenceClass::SyntheticConformance,
            format!("sha256:{}", "a".repeat(64)),
            1,
        )
        .unwrap();
        assert!(!authorization.permits_live_use());
    }

    #[test]
    fn authorization_requires_all_ten_comparisons() {
        let mut report = passing_report();
        report.holdout_comparisons.pop();
        assert!(matches!(
            PromotionAuthorization::from_report(
                &report,
                PromotionEvidenceClass::RealHeldOut,
                format!("sha256:{}", "b".repeat(64)),
                1,
            ),
            Err(LivePolicyCanaryError::InvalidPromotionReport(_))
        ));
    }

    #[test]
    fn audit_chain_detects_tampering() {
        let mut canary = LivePolicyCanary::new(LivePolicyCanaryConfig::default()).unwrap();
        canary
            .latch_rollback(CanaryRollbackReason::OperatorRequested)
            .unwrap();
        assert!(verify_audit_chain(canary.audit()));
        let mut tampered = canary.audit().to_vec();
        tampered[0].digest_fnv1a64 ^= 1;
        assert!(!verify_audit_chain(&tampered));
    }
}
