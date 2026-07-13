//! Shadow-only companion observation.
//!
//! The observer extracts a deliberately small vocabulary of explicit,
//! first-person user statements into typed `ClaimInput` proposals. It has no
//! access to `CompanionState`, persistence, response routing, or action
//! authority. A separate caller must review and commit any proposal.

use crate::companion_state::{ClaimInput, ClaimSource, Retention, Sensitivity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

const DEFAULT_MAX_MESSAGE_BYTES: usize = 16 * 1024;
const DEFAULT_MAX_PROPOSALS: usize = 4;
const EXPLICIT_STATEMENT_CONFIDENCE_BPS: u16 = 10_000;
const MAX_DOMAIN_BYTES: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    /// Only complete sentence-like clauses beginning with an explicit supported
    /// first-person statement or direct preference request are eligible. This
    /// excludes quoted, hypothetical, third-person, and adversarial occurrences
    /// of the same words. No state is read or mutated.
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
        for clause in explicit_clauses(message) {
            self.observe_clause(message, clause, observed_at_ms, &mut candidates);
        }

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

    fn observe_clause(
        &self,
        message: &str,
        clause: ClauseSpan,
        observed_at_ms: u64,
        proposals: &mut Vec<ShadowClaimProposal>,
    ) {
        let Some(text) = message.get(clause.start_byte..clause.end_byte) else {
            return;
        };
        let lower = text.to_ascii_lowercase();

        let fixed_rules = [
            (
                ShadowObservationRule::PrefersDetail,
                "preference.detail.general",
                "yes",
                DETAIL_PREFIXES,
            ),
            (
                ShadowObservationRule::PrefersBrevity,
                "preference.brevity.general",
                "yes",
                BREVITY_PREFIXES,
            ),
            (
                ShadowObservationRule::PrefersQuestions,
                "preference.questions.general",
                "yes",
                QUESTION_PREFIXES,
            ),
            (
                ShadowObservationRule::PrefersConcreteExamples,
                "preference.argument_style.general",
                "concrete",
                CONCRETE_PREFIXES,
            ),
            (
                ShadowObservationRule::PrefersAbstractExplanations,
                "preference.argument_style.general",
                "abstract",
                ABSTRACT_PREFIXES,
            ),
            (
                ShadowObservationRule::PrefersAdaptiveStyle,
                "preference.argument_style.general",
                "adaptive",
                ADAPTIVE_PREFIXES,
            ),
        ];

        for (rule, key, value, prefixes) in fixed_rules {
            if prefixes.iter().any(|prefix| starts_with_phrase(&lower, prefix)) {
                proposals.push(self.proposal(
                    rule,
                    clause,
                    key.to_owned(),
                    value.to_owned(),
                    observed_at_ms,
                ));
            }
        }

        if let Some(domain) = domain_after_prefix(&lower, STRONG_DOMAIN_PREFIXES) {
            proposals.push(self.proposal(
                ShadowObservationRule::StrongDomain,
                clause,
                format!("knowledge.strong_domain.{domain}"),
                domain,
                observed_at_ms,
            ));
        }
        if let Some(domain) = domain_after_prefix(&lower, WEAK_DOMAIN_PREFIXES) {
            proposals.push(self.proposal(
                ShadowObservationRule::WeakDomain,
                clause,
                format!("knowledge.weak_domain.{domain}"),
                domain,
                observed_at_ms,
            ));
        }
    }

    fn proposal(
        &self,
        rule: ShadowObservationRule,
        evidence: ClauseSpan,
        key: String,
        value: String,
        observed_at_ms: u64,
    ) -> ShadowClaimProposal {
        ShadowClaimProposal {
            rule,
            evidence: EvidenceSpan {
                start_byte: evidence.start_byte,
                end_byte: evidence.end_byte,
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

const DETAIL_PREFIXES: &[&str] = &[
    "i prefer detailed answers",
    "i prefer detailed responses",
    "i prefer detailed explanations",
    "i prefer more detailed answers",
    "i prefer more detailed responses",
    "i want detailed answers",
    "i want detailed responses",
    "i want more detailed answers",
    "please give me detailed answers",
    "please give me more detailed answers",
    "give me detailed answers",
    "give me more detailed answers",
];

const BREVITY_PREFIXES: &[&str] = &[
    "i prefer short answers",
    "i prefer brief answers",
    "i prefer concise answers",
    "i prefer short responses",
    "i prefer brief responses",
    "i prefer concise responses",
    "i want short answers",
    "i want brief answers",
    "i want concise answers",
    "please keep it short",
    "please keep it brief",
    "please keep it concise",
    "please keep responses short",
    "please keep responses brief",
    "please keep responses concise",
    "keep it short",
    "keep it brief",
    "keep it concise",
];

const QUESTION_PREFIXES: &[&str] = &[
    "i prefer being asked questions",
    "i prefer answering questions",
    "i want you to ask me questions",
    "please ask me questions",
];

const CONCRETE_PREFIXES: &[&str] = &[
    "i prefer examples",
    "i prefer concrete examples",
    "please use examples",
    "please use concrete examples",
    "use examples",
    "use concrete examples",
];

const ABSTRACT_PREFIXES: &[&str] = &[
    "i prefer abstract explanations",
    "i prefer theory",
    "i prefer theoretical explanations",
    "please use abstract explanations",
    "please use theory",
    "please use theoretical explanations",
    "use abstract explanations",
    "use theory",
    "use theoretical explanations",
];

const ADAPTIVE_PREFIXES: &[&str] = &[
    "i prefer an adaptive style",
    "i prefer a mixed style",
    "i prefer a contextual style",
    "please use an adaptive style",
    "please use a mixed approach",
    "please use a contextual approach",
    "use an adaptive style",
    "use a mixed approach",
    "use a contextual approach",
];

const STRONG_DOMAIN_PREFIXES: &[&str] = &[
    "i am good at ",
    "i'm good at ",
    "i am strong in ",
    "i'm strong in ",
    "i am experienced with ",
    "i'm experienced with ",
    "i am skilled in ",
    "i'm skilled in ",
    "i know ",
];

const WEAK_DOMAIN_PREFIXES: &[&str] = &[
    "i struggle with ",
    "i am weak at ",
    "i'm weak at ",
    "i am weak in ",
    "i'm weak in ",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ClauseSpan {
    start_byte: usize,
    end_byte: usize,
}

fn explicit_clauses(message: &str) -> Vec<ClauseSpan> {
    let mut clauses = Vec::new();
    let mut segment_start = 0;

    for (index, character) in message.char_indices() {
        if matches!(character, '.' | '!' | '?' | '\n') {
            push_trimmed_clause(message, segment_start, index, &mut clauses);
            segment_start = index + character.len_utf8();
        }
    }
    push_trimmed_clause(message, segment_start, message.len(), &mut clauses);
    clauses
}

fn push_trimmed_clause(
    message: &str,
    start: usize,
    end: usize,
    clauses: &mut Vec<ClauseSpan>,
) {
    let Some(segment) = message.get(start..end) else {
        return;
    };
    let leading = segment.len() - segment.trim_start().len();
    let trailing = segment.len() - segment.trim_end().len();
    let trimmed_start = start + leading;
    let trimmed_end = end.saturating_sub(trailing);
    if trimmed_start < trimmed_end {
        clauses.push(ClauseSpan {
            start_byte: trimmed_start,
            end_byte: trimmed_end,
        });
    }
}

fn starts_with_phrase(text: &str, phrase: &str) -> bool {
    let Some(rest) = text.strip_prefix(phrase) else {
        return false;
    };
    rest.is_empty()
        || rest
            .chars()
            .next()
            .is_some_and(|character| character.is_whitespace() || ",;:".contains(character))
}

fn domain_after_prefix(text: &str, prefixes: &[&str]) -> Option<String> {
    for prefix in prefixes {
        let Some(raw) = text.strip_prefix(prefix) else {
            continue;
        };
        let raw = if *prefix == "i know " {
            raw.strip_suffix(" well")?
        } else {
            raw
        };
        return normalize_domain(raw);
    }
    None
}

fn normalize_domain(raw: &str) -> Option<String> {
    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty()
        || normalized.len() > MAX_DOMAIN_BYTES
        || !normalized
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || " +#._-".contains(character))
    {
        return None;
    }
    Some(normalized.to_ascii_lowercase())
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
    fn matched_control_corpus_has_no_false_positives() {
        let observer = ShadowCompanionObserver::default();
        let controls = [
            "I do not prefer detailed answers.",
            "I don't prefer detailed answers.",
            "She prefers detailed answers.",
            "He said, \"I prefer detailed answers.\"",
            "I said, 'I prefer detailed answers.'",
            "If I prefer detailed answers, would that help?",
            "Imagine I prefer detailed answers.",
            "Repeat exactly: I prefer detailed answers.",
            "Ignore this test phrase: I prefer detailed answers.",
            "> I prefer detailed answers.",
            "The user wrote I struggle with calculus.",
            "Suppose I'm good at Rust.",
        ];

        for message in controls {
            let batch = observer.observe(message, 10).unwrap();
            assert!(batch.proposals.is_empty(), "false positive for: {message}");
        }
    }

    #[test]
    fn frozen_explicit_corpus_has_no_false_negatives() {
        let observer = ShadowCompanionObserver::default();
        let cases = [
            ("I prefer detailed answers.", ShadowObservationRule::PrefersDetail),
            ("Please keep responses concise.", ShadowObservationRule::PrefersBrevity),
            ("I want you to ask me questions.", ShadowObservationRule::PrefersQuestions),
            ("Please use concrete examples.", ShadowObservationRule::PrefersConcreteExamples),
            ("Please use theoretical explanations.", ShadowObservationRule::PrefersAbstractExplanations),
            ("Use a mixed approach.", ShadowObservationRule::PrefersAdaptiveStyle),
            ("I'm good at Rust.", ShadowObservationRule::StrongDomain),
            ("I struggle with public speaking.", ShadowObservationRule::WeakDomain),
        ];

        for (message, expected_rule) in cases {
            let batch = observer.observe(message, 10).unwrap();
            assert_eq!(batch.proposals.len(), 1, "missed or duplicated: {message}");
            assert_eq!(batch.proposals[0].rule, expected_rule);
        }
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
