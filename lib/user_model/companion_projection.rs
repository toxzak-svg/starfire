//! Read-only compatibility projection from authoritative companion state.
//!
//! This adapter lets legacy callers inspect a `UserCognitionModel` without
//! making that mutable legacy structure authoritative. Projection is
//! deterministic, policy-gated, provenance-preserving, and has no runtime or
//! response-routing authority.

use super::{
    ArgumentStyle, InferenceSource, PreferenceType, ResponsePattern, UserCognitionModel,
    UserMemoryModel, UserPreference,
};
use crate::companion_state::{
    ClaimId, ClaimSource, ClaimStatus, CompanionClaim, CompanionState, Retention, Sensitivity,
};
use std::collections::BTreeSet;
use thiserror::Error;

const STRONG_DOMAIN_KEY: &str = "knowledge.strong_domain";
const WEAK_DOMAIN_KEY: &str = "knowledge.weak_domain";
const DETAIL_PREFIX: &str = "preference.detail";
const BREVITY_PREFIX: &str = "preference.brevity";
const QUESTIONS_PREFIX: &str = "preference.questions";
const ARGUMENT_STYLE_PREFIX: &str = "preference.argument_style";
const RESPONSE_PATTERN_PREFIX: &str = "response_pattern";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanionProjectionPolicy {
    /// Minimum claim confidence required for compatibility projection.
    pub min_confidence_bps: u16,
    /// Include sensitive claims. Disabled by default.
    pub include_sensitive: bool,
    /// Include claims retained only for the current session. Disabled by default.
    pub include_session: bool,
}

impl Default for CompanionProjectionPolicy {
    fn default() -> Self {
        Self {
            min_confidence_bps: 6_000,
            include_sensitive: false,
            include_session: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LegacyUserModelProjection {
    /// Companion version from which this compatibility view was produced.
    pub source_version: u64,
    /// Legacy view for existing callers. Mutating it cannot mutate companion state.
    pub model: UserCognitionModel,
    /// Exact active claim IDs that contributed to the view, sorted for replay.
    pub source_claim_ids: Vec<ClaimId>,
    /// Active claims examined but not recognized by the compatibility schema.
    pub unrecognized_claim_ids: Vec<ClaimId>,
}

impl LegacyUserModelProjection {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.source_claim_ids.is_empty()
    }
}

pub fn project_legacy_user_model(
    state: &CompanionState,
    now_ms: u64,
    policy: CompanionProjectionPolicy,
) -> Result<LegacyUserModelProjection, CompanionProjectionError> {
    if policy.min_confidence_bps > 10_000 {
        return Err(CompanionProjectionError::ConfidenceOutOfRange(
            policy.min_confidence_bps,
        ));
    }

    let mut model = UserCognitionModel {
        memory: UserMemoryModel::new(),
        preferences: Vec::new(),
        strong_domains: Vec::new(),
        weak_domains: Vec::new(),
        response_patterns: Vec::new(),
        last_updated: 0,
    };
    let mut source_claim_ids = BTreeSet::new();
    let mut unrecognized_claim_ids = BTreeSet::new();

    for claim in state.claims().values() {
        if !claim_allowed(claim, now_ms, policy) {
            continue;
        }

        if project_claim(claim, &mut model)? {
            source_claim_ids.insert(claim.id);
            let updated_at_seconds = i64::try_from(claim.updated_at_ms / 1_000)
                .map_err(|_| CompanionProjectionError::TimestampOverflow(claim.id))?;
            model.last_updated = model.last_updated.max(updated_at_seconds);
        } else {
            unrecognized_claim_ids.insert(claim.id);
        }
    }

    // The legacy delegation API returns the first preference for a topic. Keep
    // every compatible preference, but put explicit question preferences first
    // so an earlier detail/brevity claim cannot mask a later active question
    // preference for the same topic.
    model.preferences.sort_by(|left, right| {
        left.topic
            .to_ascii_lowercase()
            .cmp(&right.topic.to_ascii_lowercase())
            .then_with(|| {
                preference_priority(&left.preference).cmp(&preference_priority(&right.preference))
            })
            .then_with(|| left.learned_at.cmp(&right.learned_at))
    });

    Ok(LegacyUserModelProjection {
        source_version: state.version,
        model,
        source_claim_ids: source_claim_ids.into_iter().collect(),
        unrecognized_claim_ids: unrecognized_claim_ids.into_iter().collect(),
    })
}

fn claim_allowed(claim: &CompanionClaim, now_ms: u64, policy: CompanionProjectionPolicy) -> bool {
    matches!(&claim.status, ClaimStatus::Active)
        && claim.confidence_bps >= policy.min_confidence_bps
        && !claim.retention.is_expired(now_ms)
        && (policy.include_session || claim.retention != Retention::Session)
        && (policy.include_sensitive || claim.sensitivity != Sensitivity::Sensitive)
}

fn project_claim(
    claim: &CompanionClaim,
    model: &mut UserCognitionModel,
) -> Result<bool, CompanionProjectionError> {
    if let Some(domain) = claim_value_or_suffix(claim, STRONG_DOMAIN_KEY) {
        push_unique(&mut model.strong_domains, domain);
        return Ok(true);
    }
    if let Some(domain) = claim_value_or_suffix(claim, WEAK_DOMAIN_KEY) {
        push_unique(&mut model.weak_domains, domain);
        return Ok(true);
    }
    if let Some(topic) = enabled_topic(claim, DETAIL_PREFIX) {
        model.preferences.push(preference_from_claim(
            claim,
            topic,
            PreferenceType::PrefersDetail,
        )?);
        return Ok(true);
    }
    if let Some(topic) = enabled_topic(claim, BREVITY_PREFIX) {
        model.preferences.push(preference_from_claim(
            claim,
            topic,
            PreferenceType::PrefersBrevity,
        )?);
        return Ok(true);
    }
    if let Some(topic) = enabled_topic(claim, QUESTIONS_PREFIX) {
        model.preferences.push(preference_from_claim(
            claim,
            topic,
            PreferenceType::PrefersQuestions,
        )?);
        return Ok(true);
    }
    if let Some(topic) = key_topic(&claim.key, ARGUMENT_STYLE_PREFIX) {
        let Some(style) = parse_argument_style(&claim.value) else {
            return Ok(false);
        };
        model.preferences.push(preference_from_claim(
            claim,
            topic,
            PreferenceType::AgreesWithStyle(style),
        )?);
        return Ok(true);
    }
    if let Some(pattern_type) = key_topic(&claim.key, RESPONSE_PATTERN_PREFIX) {
        model.response_patterns.push(ResponsePattern {
            pattern_type,
            description: claim.value.clone(),
            reliability: f64::from(claim.confidence_bps) / 10_000.0,
            example_count: claim.supporting_observation_ids.len(),
        });
        return Ok(true);
    }

    Ok(false)
}

fn preference_from_claim(
    claim: &CompanionClaim,
    topic: String,
    preference: PreferenceType,
) -> Result<UserPreference, CompanionProjectionError> {
    let learned_at = i64::try_from(claim.created_at_ms / 1_000)
        .map_err(|_| CompanionProjectionError::TimestampOverflow(claim.id))?;
    Ok(UserPreference {
        topic,
        preference,
        confidence: f64::from(claim.confidence_bps) / 10_000.0,
        inferred_from: match &claim.source {
            ClaimSource::UserStatement | ClaimSource::UserCorrection { .. } => {
                InferenceSource::Teaching
            }
            ClaimSource::Inference { .. } | ClaimSource::Import { .. } => {
                InferenceSource::Observation
            }
        },
        learned_at,
    })
}

fn preference_priority(preference: &PreferenceType) -> u8 {
    match preference {
        PreferenceType::PrefersQuestions => 0,
        PreferenceType::PrefersDetail | PreferenceType::PrefersBrevity => 1,
        PreferenceType::AgreesWithStyle(_) => 2,
        PreferenceType::HasStrongPrior | PreferenceType::Unfamiliar => 3,
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn claim_value_or_suffix(claim: &CompanionClaim, prefix: &str) -> Option<String> {
    let topic = key_topic(&claim.key, prefix)?;
    let value = if topic == "general" {
        claim.value.trim().to_lowercase()
    } else {
        topic
    };
    (!value.is_empty()).then_some(value)
}

fn enabled_topic(claim: &CompanionClaim, prefix: &str) -> Option<String> {
    if !truthy(&claim.value) {
        return None;
    }
    key_topic(&claim.key, prefix)
}

fn key_topic(key: &str, prefix: &str) -> Option<String> {
    if key == prefix {
        return Some("general".to_owned());
    }
    key.strip_prefix(prefix)
        .and_then(|suffix| suffix.strip_prefix('.'))
        .map(str::trim)
        .filter(|suffix| !suffix.is_empty())
        .map(str::to_owned)
}

fn truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "yes" | "1" | "on" | "prefer" | "preferred"
    )
}

fn parse_argument_style(value: &str) -> Option<ArgumentStyle> {
    match value.trim().to_ascii_lowercase().as_str() {
        "concrete" | "concrete examples" | "examples" => Some(ArgumentStyle::Concrete),
        "abstract" | "abstraction" | "theory" => Some(ArgumentStyle::Abstract),
        "adaptive" | "mixed" | "contextual" => Some(ArgumentStyle::Adaptive),
        _ => None,
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CompanionProjectionError {
    #[error("projection confidence {0} exceeds 10000 basis points")]
    ConfidenceOutOfRange(u16),
    #[error("claim {0} timestamp cannot be represented by the legacy model")]
    TimestampOverflow(ClaimId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion_state::{ClaimInput, CorrectionInput};

    fn claim(
        key: &str,
        value: &str,
        confidence_bps: u16,
        sensitivity: Sensitivity,
        retention: Retention,
        at: u64,
    ) -> ClaimInput {
        ClaimInput {
            key: key.to_owned(),
            value: value.to_owned(),
            source: ClaimSource::UserStatement,
            confidence_bps,
            sensitivity,
            retention,
            observed_at_ms: at,
        }
    }

    #[test]
    fn projects_supported_active_claims_without_mutating_source() {
        let mut state = CompanionState::new();
        let strong = state
            .record_claim(
                0,
                claim(
                    "knowledge.strong_domain.rust",
                    "rust",
                    9_000,
                    Sensitivity::Personal,
                    Retention::Durable,
                    10_000,
                ),
            )
            .unwrap();
        let detail = state
            .record_claim(
                strong.version,
                claim(
                    "preference.detail.technical",
                    "yes",
                    8_000,
                    Sensitivity::Personal,
                    Retention::Durable,
                    12_000,
                ),
            )
            .unwrap();
        let before = state.clone();

        let projection =
            project_legacy_user_model(&state, 13_000, CompanionProjectionPolicy::default())
                .unwrap();

        assert_eq!(state, before);
        assert_eq!(projection.source_version, 2);
        assert_eq!(projection.source_claim_ids, vec![1, 2]);
        assert_eq!(projection.model.likely_knows("Rust ownership"), Some(true));
        assert!(projection
            .model
            .preferences
            .iter()
            .any(|preference| preference.topic == "technical"
                && matches!(preference.preference, PreferenceType::PrefersDetail)
                && preference.inferred_from == InferenceSource::Teaching));
        assert_eq!(projection.model.last_updated, 12);
        assert_eq!(detail.version, state.version);
    }

    #[test]
    fn default_policy_excludes_sensitive_session_expired_and_low_confidence_claims() {
        let mut state = CompanionState::new();
        let mut version = 0;
        for input in [
            claim(
                "knowledge.strong_domain.secret",
                "secret",
                9_000,
                Sensitivity::Sensitive,
                Retention::Durable,
                10,
            ),
            claim(
                "knowledge.strong_domain.session",
                "session",
                9_000,
                Sensitivity::Personal,
                Retention::Session,
                11,
            ),
            claim(
                "knowledge.strong_domain.expired",
                "expired",
                9_000,
                Sensitivity::Personal,
                Retention::Until { expires_at_ms: 20 },
                12,
            ),
            claim(
                "knowledge.strong_domain.weak-evidence",
                "weak-evidence",
                5_000,
                Sensitivity::Personal,
                Retention::Durable,
                13,
            ),
        ] {
            version = state.record_claim(version, input).unwrap().version;
        }

        let projection =
            project_legacy_user_model(&state, 20, CompanionProjectionPolicy::default()).unwrap();

        assert!(projection.is_empty());
        assert!(projection.model.strong_domains.is_empty());
    }

    #[test]
    fn correction_projects_only_the_replacement_claim() {
        let mut state = CompanionState::new();
        let original = state
            .record_claim(
                0,
                claim(
                    "preference.argument_style.general",
                    "abstract",
                    9_000,
                    Sensitivity::Personal,
                    Retention::Durable,
                    10_000,
                ),
            )
            .unwrap();
        let replacement = state
            .correct_claim(
                original.version,
                original.claim_id.unwrap(),
                CorrectionInput {
                    value: "concrete".to_owned(),
                    confidence_bps: 10_000,
                    sensitivity: Sensitivity::Personal,
                    retention: Retention::Durable,
                    observed_at_ms: 20_000,
                },
            )
            .unwrap();

        let projection =
            project_legacy_user_model(&state, 21_000, CompanionProjectionPolicy::default())
                .unwrap();

        assert_eq!(
            projection.source_claim_ids,
            vec![replacement.claim_id.unwrap()]
        );
        assert_eq!(
            projection.model.infer_argument_style(),
            ArgumentStyle::Concrete
        );
        assert_eq!(projection.model.preferences.len(), 1);
    }

    #[test]
    fn question_preference_precedes_same_topic_detail_for_legacy_delegation() {
        let mut state = CompanionState::new();
        let detail = state
            .record_claim(
                0,
                claim(
                    "preference.detail.technical",
                    "yes",
                    9_000,
                    Sensitivity::Personal,
                    Retention::Durable,
                    10_000,
                ),
            )
            .unwrap();
        state
            .record_claim(
                detail.version,
                claim(
                    "preference.questions.technical",
                    "yes",
                    9_000,
                    Sensitivity::Personal,
                    Retention::Durable,
                    11_000,
                ),
            )
            .unwrap();

        let projection =
            project_legacy_user_model(&state, 12_000, CompanionProjectionPolicy::default())
                .unwrap();

        assert_eq!(projection.model.preferences.len(), 2);
        assert!(matches!(
            projection.model.preferences[0].preference,
            PreferenceType::PrefersQuestions
        ));
        assert!(projection.model.should_delegate_to_user("technical"));
    }

    #[test]
    fn unsupported_argument_style_is_unrecognized_without_aborting_projection() {
        let mut state = CompanionState::new();
        let strong = state
            .record_claim(
                0,
                claim(
                    "knowledge.strong_domain.rust",
                    "rust",
                    9_000,
                    Sensitivity::Personal,
                    Retention::Durable,
                    10_000,
                ),
            )
            .unwrap();
        let unsupported = state
            .record_claim(
                strong.version,
                claim(
                    "preference.argument_style.general",
                    "direct",
                    9_000,
                    Sensitivity::Personal,
                    Retention::Durable,
                    11_000,
                ),
            )
            .unwrap();

        let projection =
            project_legacy_user_model(&state, 12_000, CompanionProjectionPolicy::default())
                .unwrap();

        assert_eq!(projection.source_claim_ids, vec![strong.claim_id.unwrap()]);
        assert_eq!(
            projection.unrecognized_claim_ids,
            vec![unsupported.claim_id.unwrap()]
        );
        assert_eq!(projection.model.likely_knows("Rust ownership"), Some(true));
    }

    #[test]
    fn contested_and_unrecognized_claims_do_not_enter_legacy_model() {
        let mut state = CompanionState::new();
        let active = state
            .record_claim(
                0,
                claim(
                    "custom.unknown",
                    "yes",
                    9_000,
                    Sensitivity::Personal,
                    Retention::Durable,
                    10,
                ),
            )
            .unwrap();
        let contested = state
            .record_claim(
                active.version,
                ClaimInput {
                    key: "custom.unknown".to_owned(),
                    value: "different".to_owned(),
                    source: ClaimSource::Inference {
                        method: "shadow".to_owned(),
                    },
                    confidence_bps: 9_000,
                    sensitivity: Sensitivity::Personal,
                    retention: Retention::Durable,
                    observed_at_ms: 11,
                },
            )
            .unwrap();

        let projection =
            project_legacy_user_model(&state, 12, CompanionProjectionPolicy::default()).unwrap();

        assert_eq!(
            projection.unrecognized_claim_ids,
            vec![active.claim_id.unwrap()]
        );
        assert!(!projection
            .unrecognized_claim_ids
            .contains(&contested.claim_id.unwrap()));
        assert!(projection.source_claim_ids.is_empty());
        assert!(projection.model.strong_domains.is_empty());
        assert!(projection.model.weak_domains.is_empty());
        assert!(projection.model.preferences.is_empty());
        assert!(projection.model.response_patterns.is_empty());
    }
}
