//! Shadow-only companion observation.
//!
//! The observer extracts a small, explicit vocabulary of first-person user
//! statements into typed `ClaimInput` proposals. It has no access to
//! `CompanionState`, persistence, response routing, or action authority. A
//! caller must separately review and commit any proposal.

use crate::companion_state::{ClaimInput, ClaimSource, Retention, Sensitivity};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::OnceLock;
use thiserror::Error;

const DEFAULT_MAX_MESSAGE_BYTES: usize = 16 * 1024;
const DEFAULT_MAX_PROPOSALS: usize = 4;
const EXPLICIT_STATEMENT_CONFIDENCE_BPS: u16 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowObservationRule {
    PrefersDetail,
    PrefersBrevity,
    PrefersQuestions,
    PrefersConcreteExamples,
    PrefersAbstractExplanations,
    PrefersAdaptiveStyle,
    StrongDomain,
    WeakDomain,
}

impl ShadowObservationRule {
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::PrefersDetail => "explicit.preference.detail.v1",
            Self::PrefersBrevity => "explicit.preference.brevity.v1",
            Self::PrefersQuestions => "explicit.preference.questions.v1",
            Self::PrefersConcreteExamples => "explicit.preference.concrete.v1",
            Self::PrefersAbstractExplanations => "explicit.preference.abstract.v1",
            Self::PrefersAdaptiveStyle => "explicit.preference.adaptive.v1",
            Self::StrongDomain => "explicit.knowledge.strong-domain.v1",
            Self::WeakDomain => "explicit.knowledge.weak-domain.v1",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceSpan {
    pub start_byte: usize,
    pub end_byte: usize,
}

impl EvidenceSpan {
    #[must_use]
    pub fn slice<'a>(&self, message: &'a str) -> Option<&'a str> {
        message.get(self.start_byte..self.end_byte)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowClaimProposal {
    pub rule: ShadowObservationRule,
    pub evidence: EvidenceSpan,
    pub claim: ClaimInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowObservationBatch {
    pub observed_at_ms: u64,
    pub message_digest_fnv1a64: u64,
    pub message_bytes: usize,
    pub proposals: Vec<ShadowClaimProposal>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShadowObserverConfig {
    pub max_message_bytes: usize,
    pub max_proposals: usize,
    pub proposed_retention: Retention,
    pub proposed_sensitivity: Sensitivity,
}

impl Default for ShadowObserverConfig {
    fn default() -> Self {
        Self {
            max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
            max_proposals: DEFAULT_MAX_PROPOSALS,
            proposed_retention: Retention::Session,
            proposed_sensitivity: Sensitivity::Personal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShadowCompanionObserver {
    config: ShadowObserverConfig,
}

impl Default for ShadowCompanionObserver {
    fn default() -> Self {
        Self {
            config: ShadowObserverConfig::default(),
        }
    }
}

impl ShadowCompanionObserver {
    pub fn new(config: ShadowObserverConfig) -> Result<Self, ShadowObserverError> {
        if config.max_message_bytes == 0 {
            return Err(ShadowObserverError::InvalidMaxMessageBytes);
        }
        if config.max_proposals == 0 {
            return Err(ShadowObserverError::InvalidMaxProposals);
        }
        Ok(Self { config })
    }

    #[must_use]
    pub fn config(&self) -> &ShadowObserverConfig {
        &self.config
    }

    /// Observe one user message and return inert claim proposals.
    ///
    /// No state is read or mutated. Rules are evaluated in a fixed order and
    /// duplicate key/value pairs are removed before the proposal budget is
    /// applied.
    pub fn observe(
        &self,
        message: &str,
        observed_at_ms: u64,
    ) -> Result<ShadowObservationBatch, ShadowObserverError> {
        if message.len() > self.config.max_message_bytes {
            return Err(ShadowObserverError::MessageTooLarge {
                actual: message.len(),
                maximum: self.config.max_message_bytes,
            });
        }

        let mut candidates = Vec::new();
        self.observe_fixed_rules(message, observed_at_ms, &mut candidates);
        self.observe_domain_rules(message, observed_at_ms, &mut candidates);

        let mut seen = BTreeSet::new();
        candidates.retain(|proposal| {
            seen.insert((proposal.claim.key.clone(), proposal.claim.value.clone()))
        });
        let truncated = candidates.len() > self.config.max_proposals;
        candidates.truncate(self.config.max_proposals);

        Ok(ShadowObservationBatch {
            observed_at_ms,
            message_digest_fnv1a64: fnv1a64(message.as_bytes()),
            message_bytes: message.len(),
            proposals: candidates,
            truncated,
        })
    }

    fn observe_fixed_rules(
        &self,
        message: &str,
        observed_at_ms: u64,
        proposals: &mut Vec<ShadowClaimProposal>,
    ) {
        let rules = [
            (
                ShadowObservationRule::PrefersDetail,
                detail_regex(),
                "preference.detail.general",
                "yes",
            ),
            (
                ShadowObservationRule::PrefersBrevity,
                brevity_regex(),
                "preference.brevity.general",
                "yes",
            ),
            (
                ShadowObservationRule::PrefersQuestions,
                questions_regex(),
                "preference.questions.general",
                "yes",
            ),
            (
                ShadowObservationRule::PrefersConcreteExamples,
                concrete_regex(),
                "preference.argument_style.general",
                "concrete",
            ),
            (
                ShadowObservationRule::PrefersAbstractExplanations,
                abstract_regex(),
                "preference.argument_style.general",
                "abstract",
            ),
            (
                ShadowObservationRule::PrefersAdaptiveStyle,
                adaptive_regex(),
                "preference.argument_style.general",
                "adaptive",
            ),
        ];

        for (rule, regex, key, value) in rules {
            if let Some(found) = regex.find(message) {
                proposals.push(self.proposal(
                    rule,
                    found.start(),
                    found.end(),
                    key.to_owned(),
                    value.to_owned(),
                    observed_at_ms,
                ));
            }
        }
    }

    fn observe_domain_rules(
        &self,
        message: &str,
        observed_at_ms: u64,
        proposals: &mut Vec<ShadowClaimProposal>,
    ) {
        for captures in strong_domain_regex().captures_iter(message) {
            if let Some(proposal) = self.domain_proposal(
                ShadowObservationRule::StrongDomain,
                "knowledge.strong_domain",
                captures,
                observed_at_ms,
            ) {
                proposals.push(proposal);
            }
        }
        for captures in strong_domain_well_regex().captures_iter(message) {
            if let Some(proposal) = self.domain_proposal(
                ShadowObservationRule::StrongDomain,
                "knowledge.strong_domain",
                captures,
                observed_at_ms,
            ) {
                proposals.push(proposal);
            }
        }
        for captures in weak_domain_regex().captures_iter(message) {
            if let Some(proposal) = self.domain_proposal(
                ShadowObservationRule::WeakDomain,
                "knowledge.weak_domain",
                captures,
                observed_at_ms,
            ) {
                proposals.push(proposal);
            }
        }
    }

    fn domain_proposal(
        &self,
        rule: ShadowObservationRule,
        key_prefix: &str,
        captures: Captures<'_>,
        observed_at_ms: u64,
    ) -> Option<ShadowClaimProposal> {
        let whole = captures.get(0)?;
        let domain = normalize_domain(captures.get(1)?.as_str())?;
        Some(self.proposal(
            rule,
            whole.start(),
            whole.end(),
            format!("{key_prefix}.{domain}"),
            domain,
            observed_at_ms,
        ))
    }

    fn proposal(
        &self,
        rule: ShadowObservationRule,
        start_byte: usize,
        end_byte: usize,
        key: String,
        value: String,
        observed_at_ms: u64,
    ) -> ShadowClaimProposal {
        ShadowClaimProposal {
            rule,
            evidence: EvidenceSpan {
                start_byte,
                end_byte,
            },
            claim: ClaimInput {
                key,
                value,
                source: ClaimSource::UserStatement,
                confidence_bps: EXPLICIT_STATEMENT_CONFIDENCE_BPS,
                sensitivity: self.config.proposed_sensitivity,
                retention: self.config.proposed_retention.clone(),
                observed_at_ms,
            },
        }
    }
}

fn normalize_domain(raw: &str) -> Option<String> {
    let normalized = raw
        .trim_matches(|character: char| character.is_whitespace() || ",;:.!?".contains(character))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn regex(pattern: &'static str, cell: &'static OnceLock<Regex>) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("companion observer regex must compile"))
}

fn detail_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    regex(
        r"(?i)\b(?:i prefer|i want|please give me|give me)\s+(?:more\s+)?(?:detailed|in-depth)\s+(?:answers?|responses?|explanations?)\b",
        &CELL,
    )
}

fn brevity_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    regex(
        r"(?i)\b(?:(?:i prefer|i want)\s+(?:short|brief|concise)\s+(?:answers?|responses?)|(?:please\s+)?keep\s+(?:it|answers?|responses?)\s+(?:short|brief|concise))\b",
        &CELL,
    )
}

fn questions_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    regex(
        r"(?i)\b(?:i prefer\s+(?:being asked|answering)\s+questions?|i want you to\s+ask\s+me\s+questions?|please\s+ask\s+me\s+questions?)\b",
        &CELL,
    )
}

fn concrete_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    regex(
        r"(?i)\b(?:i prefer|please use|use)\s+(?:concrete\s+)?examples?\b",
        &CELL,
    )
}

fn abstract_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    regex(
        r"(?i)\b(?:i prefer|please use|use)\s+(?:abstract explanations?|theory|theoretical explanations?)\b",
        &CELL,
    )
}

fn adaptive_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    regex(
        r"(?i)\b(?:i prefer|please use|use)\s+(?:an?\s+)?(?:adaptive|mixed|contextual)\s+(?:style|approach|explanations?)\b",
        &CELL,
    )
}

fn strong_domain_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    regex(
        r"(?i)\b(?:i am|i'm)\s+(?:good|strong|experienced|skilled)\s+(?:at|in|with)\s+([a-z0-9][a-z0-9+#._ -]{1,39})(?:[.!?]|$)",
        &CELL,
    )
}

fn strong_domain_well_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    regex(
        r"(?i)\bi\s+know\s+([a-z0-9][a-z0-9+#._ -]{1,39})\s+well(?:[.!?]|$)",
        &CELL,
    )
}

fn weak_domain_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    regex(
        r"(?i)\b(?:i struggle with|i am weak (?:at|in)|i'm weak (?:at|in))\s+([a-z0-9][a-z0-9+#._ -]{1,39})(?:[.!?]|$)",
        &CELL,
    )
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ShadowObserverError {
    #[error("max_message_bytes must be greater than zero")]
    InvalidMaxMessageBytes,
    #[error("max_proposals must be greater than zero")]
    InvalidMaxProposals,
    #[error("message contains {actual} bytes, exceeding configured maximum {maximum}")]
    MessageTooLarge { actual: usize, maximum: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_first_person_preferences_produce_bounded_typed_proposals() {
        let observer = ShadowCompanionObserver::default();
        let message = "I prefer detailed answers. I want you to ask me questions. Please use concrete examples.";
        let batch = observer.observe(message, 42).unwrap();

        assert_eq!(batch.proposals.len(), 3);
        assert!(!batch.truncated);
        assert_eq!(batch.observed_at_ms, 42);
        assert_eq!(batch.message_bytes, message.len());
        assert_eq!(
            batch
                .proposals
                .iter()
                .map(|proposal| proposal.claim.key.as_str())
                .collect::<Vec<_>>(),
            vec![
                "preference.detail.general",
                "preference.questions.general",
                "preference.argument_style.general"
            ]
        );
        for proposal in &batch.proposals {
            assert_eq!(proposal.claim.source, ClaimSource::UserStatement);
            assert_eq!(proposal.claim.retention, Retention::Session);
            assert_eq!(proposal.claim.sensitivity, Sensitivity::Personal);
            assert_eq!(proposal.claim.confidence_bps, 10_000);
            assert!(proposal.evidence.slice(message).is_some());
        }
    }

    #[test]
    fn negated_and_third_person_statements_do_not_produce_claims() {
        let observer = ShadowCompanionObserver::default();
        let message = "I don't prefer detailed answers. She prefers brief answers. I am not good at Rust.";
        let batch = observer.observe(message, 10).unwrap();

        assert!(batch.proposals.is_empty());
    }

    #[test]
    fn explicit_domain_statements_preserve_evidence_and_normalize_values() {
        let observer = ShadowCompanionObserver::default();
        let message = "I'm good at Rust. I struggle with public speaking.";
        let first = observer.observe(message, 10).unwrap();
        let second = observer.observe(message, 10).unwrap();

        assert_eq!(first.message_digest_fnv1a64, second.message_digest_fnv1a64);
        assert_eq!(first.proposals.len(), 2);
        assert_eq!(first.proposals[0].claim.key, "knowledge.strong_domain.rust");
        assert_eq!(first.proposals[0].claim.value, "rust");
        assert_eq!(first.proposals[1].claim.key, "knowledge.weak_domain.public speaking");
        assert_eq!(first.proposals[1].claim.value, "public speaking");
        assert!(first
            .proposals
            .iter()
            .all(|proposal| proposal.evidence.slice(message).is_some()));
    }

    #[test]
    fn proposal_budget_is_deterministic_and_reports_truncation() {
        let observer = ShadowCompanionObserver::new(ShadowObserverConfig {
            max_proposals: 1,
            ..ShadowObserverConfig::default()
        })
        .unwrap();
        let batch = observer
            .observe(
                "I prefer detailed answers. I prefer brief answers. Please use concrete examples.",
                10,
            )
            .unwrap();

        assert_eq!(batch.proposals.len(), 1);
        assert!(batch.truncated);
        assert_eq!(batch.proposals[0].rule, ShadowObservationRule::PrefersDetail);
    }

    #[test]
    fn observer_rejects_oversized_messages_without_partial_output() {
        let observer = ShadowCompanionObserver::new(ShadowObserverConfig {
            max_message_bytes: 8,
            ..ShadowObserverConfig::default()
        })
        .unwrap();
        let error = observer.observe("I prefer detailed answers", 10).unwrap_err();

        assert_eq!(
            error,
            ShadowObserverError::MessageTooLarge {
                actual: 25,
                maximum: 8
            }
        );
    }

    #[test]
    fn rule_identifiers_are_stable_and_nonempty() {
        for rule in [
            ShadowObservationRule::PrefersDetail,
            ShadowObservationRule::PrefersBrevity,
            ShadowObservationRule::PrefersQuestions,
            ShadowObservationRule::PrefersConcreteExamples,
            ShadowObservationRule::PrefersAbstractExplanations,
            ShadowObservationRule::PrefersAdaptiveStyle,
            ShadowObservationRule::StrongDomain,
            ShadowObservationRule::WeakDomain,
        ] {
            assert!(!rule.id().is_empty());
        }
    }
}
