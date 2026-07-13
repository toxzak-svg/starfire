//! S5-A shadow interaction-policy proposals.
//!
//! This module translates eligible companion claims into inert policy proposals,
//! generates matched controls, and enrolls pre-response outcome predictions in
//! the S4 ledger. It has no access to `Runtime::chat()`, response generation,
//! routing, belief promotion, persistence, or autonomous side effects.

use crate::companion_prediction_ledger::{
    AbstentionId, AbstentionInput, OutcomeProbability, PredictionId, PredictionInput,
    PredictionLedger, PredictionLedgerError, PredictionProducer, PredictionProducerKind,
    PredictionTransition,
};
use crate::companion_state::{
    ClaimId, ClaimStatus, CompanionClaim, CompanionState, Retention, Sensitivity,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

const DETAIL_PREFIX: &str = "preference.detail";
const BREVITY_PREFIX: &str = "preference.brevity";
const QUESTIONS_PREFIX: &str = "preference.questions";
const ARGUMENT_STYLE_PREFIX: &str = "preference.argument_style";
const STRONG_DOMAIN_PREFIX: &str = "knowledge.strong_domain";
const WEAK_DOMAIN_PREFIX: &str = "knowledge.weak_domain";
const SUCCESS_LABEL: &str = "interaction_satisfactory";
const FAILURE_LABEL: &str = "interaction_unsatisfactory";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetailLevel {
    Brief,
    Standard,
    Detailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplanationStyle {
    Concrete,
    Abstract,
    Adaptive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialogueMode {
    Direct,
    QuestionLed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VocabularyLevel {
    Plain,
    Standard,
    Technical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcknowledgmentLevel {
    Minimal,
    Standard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionPolicy {
    pub detail: DetailLevel,
    pub explanation_style: ExplanationStyle,
    pub dialogue: DialogueMode,
    pub vocabulary: VocabularyLevel,
    pub acknowledgment: AcknowledgmentLevel,
}

impl Default for InteractionPolicy {
    fn default() -> Self {
        Self {
            detail: DetailLevel::Standard,
            explanation_style: ExplanationStyle::Adaptive,
            dialogue: DialogueMode::Direct,
            vocabulary: VocabularyLevel::Standard,
            acknowledgment: AcknowledgmentLevel::Minimal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PolicyVariant {
    CompanionDerived,
    NeutralDefault,
    RecencyOnly,
    MajorityPrior,
    ContextOnly,
    ScrambledScope,
}

impl PolicyVariant {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::CompanionDerived => "companion-derived",
            Self::NeutralDefault => "neutral-default",
            Self::RecencyOnly => "recency-only",
            Self::MajorityPrior => "majority-prior",
            Self::ContextOnly => "context-only",
            Self::ScrambledScope => "scrambled-scope",
        }
    }

    #[must_use]
    pub const fn all() -> [Self; 6] {
        [
            Self::CompanionDerived,
            Self::NeutralDefault,
            Self::RecencyOnly,
            Self::MajorityPrior,
            Self::ContextOnly,
            Self::ScrambledScope,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEvidence {
    pub claim_id: ClaimId,
    pub key: String,
    pub confidence_bps: u16,
    pub updated_at_ms: u64,
    pub sensitivity: Sensitivity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowPolicyProposal {
    pub variant: PolicyVariant,
    pub source_companion_version: u64,
    pub policy: InteractionPolicy,
    pub evidence: Vec<PolicyEvidence>,
    pub confidence_bps: u16,
    pub context_digest: u64,
    pub policy_digest_fnv1a64: u64,
    pub predicted_outcomes: Vec<OutcomeProbability>,
    pub abstention_reason: Option<String>,
}

impl ShadowPolicyProposal {
    #[must_use]
    pub fn is_abstention(&self) -> bool {
        self.abstention_reason.is_some()
    }

    #[must_use]
    pub fn source_claim_ids(&self) -> Vec<ClaimId> {
        self.evidence.iter().map(|evidence| evidence.claim_id).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyContext {
    /// Digest of the pre-response context. Raw conversation text is excluded.
    pub context_digest: u64,
    /// Opaque subject digest used to form S4 scopes without exposing identity.
    pub subject_scope_digest: u64,
    /// Optional canonical domain such as `rust` or `public speaking`.
    pub domain: Option<String>,
    pub technical_context: bool,
    pub asks_for_explanation: bool,
    pub emotional_signal: bool,
    pub issued_at_ms: u64,
    pub not_before_ms: u64,
    pub expires_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowPolicyBatch {
    pub source_companion_version: u64,
    pub context: PolicyContext,
    pub proposals: Vec<ShadowPolicyProposal>,
}

impl ShadowPolicyBatch {
    #[must_use]
    pub fn proposal(&self, variant: PolicyVariant) -> Option<&ShadowPolicyProposal> {
        self.proposals
            .iter()
            .find(|proposal| proposal.variant == variant)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadowPolicyPlannerConfig {
    pub min_confidence_bps: u16,
    pub include_session_claims: bool,
    pub include_sensitive_claims: bool,
    pub max_source_claims: usize,
}

impl Default for ShadowPolicyPlannerConfig {
    fn default() -> Self {
        Self {
            min_confidence_bps: 7_000,
            include_session_claims: true,
            include_sensitive_claims: false,
            max_source_claims: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShadowPolicyPlanner {
    config: ShadowPolicyPlannerConfig,
}

impl Default for ShadowPolicyPlanner {
    fn default() -> Self {
        Self {
            config: ShadowPolicyPlannerConfig::default(),
        }
    }
}

impl ShadowPolicyPlanner {
    pub fn new(config: ShadowPolicyPlannerConfig) -> Result<Self, ShadowPolicyError> {
        if config.min_confidence_bps > 10_000 {
            return Err(ShadowPolicyError::ConfidenceOutOfRange(
                config.min_confidence_bps,
            ));
        }
        if config.max_source_claims == 0 {
            return Err(ShadowPolicyError::InvalidSourceClaimBudget);
        }
        Ok(Self { config })
    }

    #[must_use]
    pub fn config(&self) -> ShadowPolicyPlannerConfig {
        self.config
    }

    pub fn plan(
        &self,
        state: &CompanionState,
        context: PolicyContext,
    ) -> Result<ShadowPolicyBatch, ShadowPolicyError> {
        validate_context(&context)?;
        let domain = context.domain.as_deref().map(canonical_domain).transpose()?;
        let eligible = state
            .claims()
            .values()
            .filter(|claim| self.claim_allowed(claim, context.issued_at_ms))
            .filter(|claim| recognized_claim(claim, domain.as_deref()))
            .collect::<Vec<_>>();

        let candidate =
            self.companion_candidate(state.version, &context, domain.as_deref(), &eligible);
        let neutral = proposal(
            PolicyVariant::NeutralDefault,
            state.version,
            InteractionPolicy::default(),
            Vec::new(),
            5_000,
            &context,
            None,
        );
        let recency =
            self.recency_control(state.version, &context, domain.as_deref(), &eligible);
        let majority = proposal(
            PolicyVariant::MajorityPrior,
            state.version,
            InteractionPolicy {
                detail: DetailLevel::Detailed,
                explanation_style: ExplanationStyle::Concrete,
                dialogue: DialogueMode::Direct,
                vocabulary: VocabularyLevel::Standard,
                acknowledgment: AcknowledgmentLevel::Minimal,
            },
            Vec::new(),
            5_500,
            &context,
            None,
        );
        let context_only_policy = context_policy(&context);
        let context_only = proposal(
            PolicyVariant::ContextOnly,
            state.version,
            context_only_policy,
            Vec::new(),
            5_500,
            &context,
            None,
        );
        let scrambled_base = if candidate.is_abstention() {
            context_only_policy
        } else {
            candidate.policy
        };
        let scrambled = proposal(
            PolicyVariant::ScrambledScope,
            state.version,
            scramble_policy(scrambled_base, context.context_digest),
            candidate.evidence.clone(),
            candidate.confidence_bps.min(6_000),
            &context,
            None,
        );

        let proposals = vec![candidate, neutral, recency, majority, context_only, scrambled];
        debug_assert_eq!(
            proposals
                .iter()
                .map(|proposal| proposal.variant)
                .collect::<BTreeSet<_>>(),
            PolicyVariant::all().into_iter().collect()
        );

        Ok(ShadowPolicyBatch {
            source_companion_version: state.version,
            context,
            proposals,
        })
    }

    pub fn enroll(
        &self,
        state: &CompanionState,
        ledger: &mut PredictionLedger,
        expected_ledger_version: u64,
        context: PolicyContext,
    ) -> Result<ShadowPolicyEnrollment, ShadowPolicyError> {
        if ledger.version != expected_ledger_version {
            return Err(ShadowPolicyError::LedgerVersionConflict {
                expected: expected_ledger_version,
                actual: ledger.version,
            });
        }

        let batch = self.plan(state, context)?;
        let mut working = ledger.clone();
        let ledger_version_before = working.version;
        let mut transitions = Vec::with_capacity(batch.proposals.len());

        for proposal in &batch.proposals {
            let producer = PredictionProducer {
                id: format!("s5a-{}-v1", proposal.variant.id()),
                kind: PredictionProducerKind::CompanionPolicy,
            };
            let subject_scope = format!(
                "s5a/{:016x}/{}",
                batch.context.subject_scope_digest,
                proposal.variant.id()
            );
            let transition = if let Some(reason) = &proposal.abstention_reason {
                working.abstain(
                    working.version,
                    AbstentionInput {
                        subject_scope,
                        producer,
                        reason: reason.clone(),
                        occurred_at_ms: batch.context.issued_at_ms,
                        context_digest: batch.context.context_digest,
                    },
                )?
            } else {
                working.issue(
                    working.version,
                    PredictionInput {
                        subject_scope,
                        producer,
                        outcomes: proposal.predicted_outcomes.clone(),
                        issued_at_ms: batch.context.issued_at_ms,
                        not_before_ms: batch.context.not_before_ms,
                        expires_at_ms: batch.context.expires_at_ms,
                        context_digest: combined_context_digest(
                            batch.context.context_digest,
                            proposal.policy_digest_fnv1a64,
                        ),
                    },
                )?
            };
            transitions.push(transition);
        }

        let prediction_ids = transitions
            .iter()
            .filter_map(|transition| transition.prediction_id)
            .collect();
        let abstention_ids = transitions
            .iter()
            .filter_map(|transition| transition.abstention_id)
            .collect();
        let ledger_version_after = working.version;
        *ledger = working;

        Ok(ShadowPolicyEnrollment {
            batch,
            ledger_version_before,
            ledger_version_after,
            prediction_ids,
            abstention_ids,
            transitions,
        })
    }

    fn claim_allowed(&self, claim: &CompanionClaim, now_ms: u64) -> bool {
        matches!(&claim.status, ClaimStatus::Active)
            && claim.confidence_bps >= self.config.min_confidence_bps
            && !claim.retention.is_expired(now_ms)
            && (self.config.include_session_claims || claim.retention != Retention::Session)
            && (self.config.include_sensitive_claims || claim.sensitivity != Sensitivity::Sensitive)
    }

    fn companion_candidate(
        &self,
        source_version: u64,
        context: &PolicyContext,
        domain: Option<&str>,
        eligible: &[&CompanionClaim],
    ) -> ShadowPolicyProposal {
        let detail = best_claim(eligible, DETAIL_PREFIX, domain);
        let brevity = best_claim(eligible, BREVITY_PREFIX, domain);
        let questions = best_claim(eligible, QUESTIONS_PREFIX, domain);
        let style = best_claim(eligible, ARGUMENT_STYLE_PREFIX, domain);
        let strong = best_claim(eligible, STRONG_DOMAIN_PREFIX, domain);
        let weak = best_claim(eligible, WEAK_DOMAIN_PREFIX, domain);

        let mut evidence = [detail, brevity, questions, style, strong, weak]
            .into_iter()
            .flatten()
            .map(evidence_from_claim)
            .collect::<Vec<_>>();
        evidence.sort_by_key(|item| item.claim_id);
        evidence.dedup_by_key(|item| item.claim_id);
        evidence.truncate(self.config.max_source_claims);

        let conflict = if detail.is_some() && brevity.is_some() {
            Some("conflicting active detail and brevity claims")
        } else if strong.is_some() && weak.is_some() {
            Some("conflicting active strong-domain and weak-domain claims")
        } else {
            None
        };
        if let Some(reason) = conflict {
            return proposal(
                PolicyVariant::CompanionDerived,
                source_version,
                InteractionPolicy::default(),
                evidence,
                0,
                context,
                Some(reason.to_owned()),
            );
        }
        if evidence.is_empty() {
            return proposal(
                PolicyVariant::CompanionDerived,
                source_version,
                InteractionPolicy::default(),
                evidence,
                0,
                context,
                Some("insufficient eligible companion evidence".to_owned()),
            );
        }

        let mut policy = InteractionPolicy::default();
        if detail.is_some() {
            policy.detail = DetailLevel::Detailed;
        } else if brevity.is_some() {
            policy.detail = DetailLevel::Brief;
        }
        if questions.is_some() {
            policy.dialogue = DialogueMode::QuestionLed;
        }
        if let Some(claim) = style {
            if let Some(parsed) = parse_explanation_style(&claim.value) {
                policy.explanation_style = parsed;
            }
        }
        if strong.is_some() {
            policy.vocabulary = VocabularyLevel::Technical;
        } else if weak.is_some() {
            policy.vocabulary = VocabularyLevel::Plain;
        }
        if context.emotional_signal {
            policy.acknowledgment = AcknowledgmentLevel::Standard;
        }

        let confidence_bps = evidence
            .iter()
            .map(|item| item.confidence_bps)
            .min()
            .unwrap_or(5_000);
        proposal(
            PolicyVariant::CompanionDerived,
            source_version,
            policy,
            evidence,
            confidence_bps,
            context,
            None,
        )
    }

    fn recency_control(
        &self,
        source_version: u64,
        context: &PolicyContext,
        domain: Option<&str>,
        eligible: &[&CompanionClaim],
    ) -> ShadowPolicyProposal {
        let most_recent = eligible
            .iter()
            .copied()
            .filter(|claim| recognized_claim(claim, domain))
            .max_by_key(|claim| (claim.updated_at_ms, claim.id));
        let Some(claim) = most_recent else {
            return proposal(
                PolicyVariant::RecencyOnly,
                source_version,
                InteractionPolicy::default(),
                Vec::new(),
                5_000,
                context,
                None,
            );
        };
        let mut policy = InteractionPolicy::default();
        apply_single_claim(&mut policy, claim);
        proposal(
            PolicyVariant::RecencyOnly,
            source_version,
            policy,
            vec![evidence_from_claim(claim)],
            claim.confidence_bps.min(6_000),
            context,
            None,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowPolicyEnrollment {
    pub batch: ShadowPolicyBatch,
    pub ledger_version_before: u64,
    pub ledger_version_after: u64,
    pub prediction_ids: Vec<PredictionId>,
    pub abstention_ids: Vec<AbstentionId>,
    pub transitions: Vec<PredictionTransition>,
}

fn proposal(
    variant: PolicyVariant,
    source_version: u64,
    policy: InteractionPolicy,
    evidence: Vec<PolicyEvidence>,
    confidence_bps: u16,
    context: &PolicyContext,
    abstention_reason: Option<String>,
) -> ShadowPolicyProposal {
    let policy_digest_fnv1a64 = policy_digest(variant, policy, &evidence);
    let predicted_outcomes = if abstention_reason.is_some() {
        Vec::new()
    } else {
        outcomes_for_confidence(confidence_bps)
    };
    ShadowPolicyProposal {
        variant,
        source_companion_version: source_version,
        policy,
        evidence,
        confidence_bps,
        context_digest: context.context_digest,
        policy_digest_fnv1a64,
        predicted_outcomes,
        abstention_reason,
    }
}

fn outcomes_for_confidence(confidence_bps: u16) -> Vec<OutcomeProbability> {
    let success_bps = if confidence_bps <= 5_000 {
        5_000
    } else {
        5_000 + (confidence_bps - 5_000) / 2
    };
    vec![
        OutcomeProbability {
            label: SUCCESS_LABEL.to_owned(),
            probability_bps: success_bps,
        },
        OutcomeProbability {
            label: FAILURE_LABEL.to_owned(),
            probability_bps: 10_000 - success_bps,
        },
    ]
}

fn context_policy(context: &PolicyContext) -> InteractionPolicy {
    InteractionPolicy {
        detail: if context.asks_for_explanation {
            DetailLevel::Detailed
        } else {
            DetailLevel::Standard
        },
        explanation_style: if context.asks_for_explanation {
            ExplanationStyle::Concrete
        } else {
            ExplanationStyle::Adaptive
        },
        dialogue: DialogueMode::Direct,
        vocabulary: if context.technical_context {
            VocabularyLevel::Technical
        } else {
            VocabularyLevel::Standard
        },
        acknowledgment: if context.emotional_signal {
            AcknowledgmentLevel::Standard
        } else {
            AcknowledgmentLevel::Minimal
        },
    }
}

fn scramble_policy(mut policy: InteractionPolicy, context_digest: u64) -> InteractionPolicy {
    policy.detail = match policy.detail {
        DetailLevel::Brief => DetailLevel::Detailed,
        DetailLevel::Detailed => DetailLevel::Brief,
        DetailLevel::Standard => {
            if context_digest & 1 == 0 {
                DetailLevel::Brief
            } else {
                DetailLevel::Detailed
            }
        }
    };
    policy.explanation_style = match policy.explanation_style {
        ExplanationStyle::Concrete => ExplanationStyle::Abstract,
        ExplanationStyle::Abstract => ExplanationStyle::Concrete,
        ExplanationStyle::Adaptive => {
            if context_digest & 2 == 0 {
                ExplanationStyle::Concrete
            } else {
                ExplanationStyle::Abstract
            }
        }
    };
    policy.dialogue = match policy.dialogue {
        DialogueMode::Direct => DialogueMode::QuestionLed,
        DialogueMode::QuestionLed => DialogueMode::Direct,
    };
    policy.vocabulary = match policy.vocabulary {
        VocabularyLevel::Plain => VocabularyLevel::Technical,
        VocabularyLevel::Technical => VocabularyLevel::Plain,
        VocabularyLevel::Standard => {
            if context_digest & 4 == 0 {
                VocabularyLevel::Plain
            } else {
                VocabularyLevel::Technical
            }
        }
    };
    policy
}

fn apply_single_claim(policy: &mut InteractionPolicy, claim: &CompanionClaim) {
    if claim_key_has_prefix(&claim.key, DETAIL_PREFIX) {
        policy.detail = DetailLevel::Detailed;
    } else if claim_key_has_prefix(&claim.key, BREVITY_PREFIX) {
        policy.detail = DetailLevel::Brief;
    } else if claim_key_has_prefix(&claim.key, QUESTIONS_PREFIX) {
        policy.dialogue = DialogueMode::QuestionLed;
    } else if claim_key_has_prefix(&claim.key, ARGUMENT_STYLE_PREFIX) {
        if let Some(style) = parse_explanation_style(&claim.value) {
            policy.explanation_style = style;
        }
    } else if claim_key_has_prefix(&claim.key, STRONG_DOMAIN_PREFIX) {
        policy.vocabulary = VocabularyLevel::Technical;
    } else if claim_key_has_prefix(&claim.key, WEAK_DOMAIN_PREFIX) {
        policy.vocabulary = VocabularyLevel::Plain;
    }
}

fn recognized_claim(claim: &CompanionClaim, domain: Option<&str>) -> bool {
    if topic_matches(&claim.key, DETAIL_PREFIX, domain)
        || topic_matches(&claim.key, BREVITY_PREFIX, domain)
        || topic_matches(&claim.key, QUESTIONS_PREFIX, domain)
    {
        return truthy(&claim.value);
    }
    if topic_matches(&claim.key, ARGUMENT_STYLE_PREFIX, domain) {
        return parse_explanation_style(&claim.value).is_some();
    }
    topic_matches(&claim.key, STRONG_DOMAIN_PREFIX, domain)
        || topic_matches(&claim.key, WEAK_DOMAIN_PREFIX, domain)
}

fn best_claim<'a>(
    claims: &[&'a CompanionClaim],
    prefix: &str,
    domain: Option<&str>,
) -> Option<&'a CompanionClaim> {
    claims
        .iter()
        .copied()
        .filter(|claim| topic_matches(&claim.key, prefix, domain))
        .max_by_key(|claim| {
            let domain_specific = domain
                .is_some_and(|domain| claim.key == format!("{prefix}.{domain}"));
            (domain_specific, claim.confidence_bps, claim.updated_at_ms, claim.id)
        })
}

fn topic_matches(key: &str, prefix: &str, domain: Option<&str>) -> bool {
    key == prefix
        || key == format!("{prefix}.general")
        || domain.is_some_and(|domain| key == format!("{prefix}.{domain}"))
}

fn claim_key_has_prefix(key: &str, prefix: &str) -> bool {
    key == prefix || key.starts_with(&format!("{prefix}."))
}

fn truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "yes" | "1" | "on" | "prefer" | "preferred"
    )
}

fn parse_explanation_style(value: &str) -> Option<ExplanationStyle> {
    match value.trim().to_ascii_lowercase().as_str() {
        "concrete" | "examples" | "concrete examples" => Some(ExplanationStyle::Concrete),
        "abstract" | "theory" | "theoretical" => Some(ExplanationStyle::Abstract),
        "adaptive" | "mixed" | "contextual" => Some(ExplanationStyle::Adaptive),
        _ => None,
    }
}

fn evidence_from_claim(claim: &CompanionClaim) -> PolicyEvidence {
    PolicyEvidence {
        claim_id: claim.id,
        key: claim.key.clone(),
        confidence_bps: claim.confidence_bps,
        updated_at_ms: claim.updated_at_ms,
        sensitivity: claim.sensitivity,
    }
}

fn validate_context(context: &PolicyContext) -> Result<(), ShadowPolicyError> {
    if context.not_before_ms < context.issued_at_ms {
        return Err(ShadowPolicyError::OutcomeWindowBeforeIssue);
    }
    if context.expires_at_ms <= context.not_before_ms {
        return Err(ShadowPolicyError::InvalidExpirationWindow);
    }
    if let Some(domain) = &context.domain {
        canonical_domain(domain)?;
    }
    Ok(())
}

fn canonical_domain(raw: &str) -> Result<String, ShadowPolicyError> {
    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty()
        || normalized.len() > 80
        || !normalized
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || " +#._-".contains(character))
    {
        return Err(ShadowPolicyError::InvalidDomain(raw.to_owned()));
    }
    Ok(normalized.to_ascii_lowercase())
}

fn policy_digest(
    variant: PolicyVariant,
    policy: InteractionPolicy,
    evidence: &[PolicyEvidence],
) -> u64 {
    let mut canonical = format!(
        "{}|{:?}|{:?}|{:?}|{:?}|{:?}",
        variant.id(),
        policy.detail,
        policy.explanation_style,
        policy.dialogue,
        policy.vocabulary,
        policy.acknowledgment
    );
    for item in evidence {
        canonical.push_str(&format!(
            "|{}:{}:{}:{}",
            item.claim_id, item.key, item.confidence_bps, item.updated_at_ms
        ));
    }
    fnv1a64(canonical.as_bytes())
}

fn combined_context_digest(context_digest: u64, policy_digest: u64) -> u64 {
    fnv1a64(format!("{context_digest:016x}:{policy_digest:016x}").as_bytes())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ShadowPolicyError {
    #[error("minimum confidence exceeds 10000 basis points: {0}")]
    ConfidenceOutOfRange(u16),
    #[error("source claim budget must be greater than zero")]
    InvalidSourceClaimBudget,
    #[error("policy outcome window begins before issue time")]
    OutcomeWindowBeforeIssue,
    #[error("policy expiration must occur after the outcome window begins")]
    InvalidExpirationWindow,
    #[error("invalid policy domain: {0}")]
    InvalidDomain(String),
    #[error("prediction ledger version conflict: expected {expected}, actual {actual}")]
    LedgerVersionConflict { expected: u64, actual: u64 },
    #[error(transparent)]
    PredictionLedger(#[from] PredictionLedgerError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion_prediction_ledger::PredictionStatus;
    use crate::companion_state::{ClaimInput, ClaimSource};

    fn context() -> PolicyContext {
        PolicyContext {
            context_digest: 0x1234,
            subject_scope_digest: 0x9876,
            domain: Some("rust".to_owned()),
            technical_context: true,
            asks_for_explanation: true,
            emotional_signal: false,
            issued_at_ms: 1_000,
            not_before_ms: 1_100,
            expires_at_ms: 2_000,
        }
    }

    fn add_claim(
        state: &mut CompanionState,
        key: &str,
        value: &str,
        confidence_bps: u16,
        sensitivity: Sensitivity,
        at: u64,
    ) -> ClaimId {
        let transition = state
            .record_claim(
                state.version,
                ClaimInput {
                    key: key.to_owned(),
                    value: value.to_owned(),
                    source: ClaimSource::UserStatement,
                    confidence_bps,
                    sensitivity,
                    retention: Retention::Session,
                    observed_at_ms: at,
                },
            )
            .unwrap();
        transition.claim_id.unwrap()
    }

    fn populated_state() -> CompanionState {
        let mut state = CompanionState::new();
        add_claim(
            &mut state,
            "preference.detail.general",
            "yes",
            9_000,
            Sensitivity::Personal,
            10,
        );
        add_claim(
            &mut state,
            "preference.questions.general",
            "yes",
            8_500,
            Sensitivity::Personal,
            20,
        );
        add_claim(
            &mut state,
            "preference.argument_style.general",
            "concrete",
            8_000,
            Sensitivity::Personal,
            30,
        );
        add_claim(
            &mut state,
            "knowledge.strong_domain.rust",
            "rust",
            9_500,
            Sensitivity::Personal,
            40,
        );
        state
    }

    #[test]
    fn candidate_is_provenance_preserving_and_does_not_mutate_state() {
        let state = populated_state();
        let before = state.clone();
        let batch = ShadowPolicyPlanner::default().plan(&state, context()).unwrap();
        let candidate = batch.proposal(PolicyVariant::CompanionDerived).unwrap();

        assert_eq!(state, before);
        assert!(!candidate.is_abstention());
        assert_eq!(candidate.policy.detail, DetailLevel::Detailed);
        assert_eq!(candidate.policy.dialogue, DialogueMode::QuestionLed);
        assert_eq!(
            candidate.policy.explanation_style,
            ExplanationStyle::Concrete
        );
        assert_eq!(candidate.policy.vocabulary, VocabularyLevel::Technical);
        assert_eq!(candidate.source_claim_ids(), vec![1, 2, 3, 4]);
        assert!(candidate
            .evidence
            .iter()
            .all(|evidence| evidence.sensitivity != Sensitivity::Sensitive));
    }

    #[test]
    fn conflicting_preferences_abstain_instead_of_silently_selecting() {
        let mut state = populated_state();
        add_claim(
            &mut state,
            "preference.brevity.general",
            "yes",
            9_000,
            Sensitivity::Personal,
            50,
        );
        let batch = ShadowPolicyPlanner::default().plan(&state, context()).unwrap();
        let candidate = batch.proposal(PolicyVariant::CompanionDerived).unwrap();

        assert!(candidate.is_abstention());
        assert!(candidate
            .abstention_reason
            .as_deref()
            .unwrap()
            .contains("detail and brevity"));
        assert!(candidate.predicted_outcomes.is_empty());
        assert_eq!(batch.proposals.len(), PolicyVariant::all().len());
    }

    #[test]
    fn sensitive_claims_are_excluded_by_default() {
        let mut state = CompanionState::new();
        add_claim(
            &mut state,
            "preference.detail.general",
            "yes",
            10_000,
            Sensitivity::Sensitive,
            10,
        );
        let batch = ShadowPolicyPlanner::default().plan(&state, context()).unwrap();
        let candidate = batch.proposal(PolicyVariant::CompanionDerived).unwrap();

        assert!(candidate.is_abstention());
        assert!(candidate.evidence.is_empty());
    }

    #[test]
    fn controls_and_digests_are_deterministic() {
        let state = populated_state();
        let first = ShadowPolicyPlanner::default().plan(&state, context()).unwrap();
        let second = ShadowPolicyPlanner::default().plan(&state, context()).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.proposals.len(), 6);
        let variants = first
            .proposals
            .iter()
            .map(|proposal| proposal.variant)
            .collect::<BTreeSet<_>>();
        assert_eq!(variants, PolicyVariant::all().into_iter().collect());
        assert_eq!(
            first
                .proposal(PolicyVariant::RecencyOnly)
                .unwrap()
                .policy
                .vocabulary,
            VocabularyLevel::Technical
        );
        assert_ne!(
            first
                .proposal(PolicyVariant::CompanionDerived)
                .unwrap()
                .policy_digest_fnv1a64,
            first
                .proposal(PolicyVariant::ScrambledScope)
                .unwrap()
                .policy_digest_fnv1a64
        );
    }

    #[test]
    fn enrollment_is_atomic_and_leaves_all_predictions_pending() {
        let state = populated_state();
        let mut ledger = PredictionLedger::new();
        let enrollment = ShadowPolicyPlanner::default()
            .enroll(&state, &mut ledger, 0, context())
            .unwrap();

        assert_eq!(enrollment.prediction_ids.len(), 6);
        assert!(enrollment.abstention_ids.is_empty());
        assert_eq!(ledger.version, 6);
        assert_eq!(ledger.summary().pending, 6);
        assert_eq!(ledger.summary().resolved, 0);
        assert!(ledger.predictions().values().all(|prediction| {
            matches!(prediction.status, PredictionStatus::Pending)
                && prediction.subject_scope.starts_with("s5a/")
        }));

        let before = ledger.clone();
        let error = ShadowPolicyPlanner::default()
            .enroll(&state, &mut ledger, 0, context())
            .unwrap_err();
        assert_eq!(
            error,
            ShadowPolicyError::LedgerVersionConflict {
                expected: 0,
                actual: 6
            }
        );
        assert_eq!(ledger, before);
    }
}
