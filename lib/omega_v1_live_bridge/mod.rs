//! ΩV1-D bounded deterministic live-response bridge kernel.
//!
//! The kernel receives only a completed neutral response and the current
//! prompt. It may replace one exact, preregistered filler opener with one
//! member of a closed deterministic table. The protected response body must
//! remain byte-for-byte identical. Any ineligible input or invariant failure
//! returns the exact neutral response.
//!
//! This module is the ΩV1-D0 kernel. Until the separate ΩV1-D1 integration
//! commit lands, it has no `Runtime::chat()` or HTTP response influence.

use serde::{Deserialize, Serialize};

pub const ELIGIBLE_OPENER: &str = "Here for it. ";
pub const REPLACEMENT_OPENERS: [&str; 3] = ["Got it. ", "I'm following. ", "I'm with you. "];
pub const MAX_PROTECTED_BODY_BYTES: usize = 4_096;
pub const MAX_OUTPUT_GROWTH_BYTES: usize = 3;

const HASH_OFFSET: u64 = 0xcbf29ce484222325;
const HASH_PRIME: u64 = 0x100000001b3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiveBridgeMode {
    Applied,
    NeutralFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackReason {
    IneligibleOpener,
    EmptyProtectedBody,
    ProtectedBodyTooLarge,
    InvariantViolation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveBridgeDecision {
    pub mode: LiveBridgeMode,
    pub neutral_text: String,
    pub rendered_text: String,
    pub selected_opener: Option<String>,
    pub protected_body_bytes: usize,
    pub body_preserved_exactly: bool,
    pub fallback_reason: Option<FallbackReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveBridgeAuthorityBoundary {
    pub bounded_transform_available: bool,
    pub api_chat_wiring: bool,
    pub live_generated_text_influence: bool,
    pub raw_conversation_access: bool,
    pub unrestricted_memory_access: bool,
    pub voice_state_mutation: bool,
    pub companion_state_mutation: bool,
    pub persistence_authority: bool,
    pub belief_promotion_authority: bool,
    pub ontology_promotion_authority: bool,
    pub routing_authority: bool,
    pub tool_selection_authority: bool,
    pub charge_discharge_authority: bool,
    pub autonomous_action_authority: bool,
}

#[must_use]
pub const fn authority_boundary() -> LiveBridgeAuthorityBoundary {
    LiveBridgeAuthorityBoundary {
        bounded_transform_available: true,
        api_chat_wiring: false,
        live_generated_text_influence: false,
        raw_conversation_access: false,
        unrestricted_memory_access: false,
        voice_state_mutation: false,
        companion_state_mutation: false,
        persistence_authority: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        routing_authority: false,
        tool_selection_authority: false,
        charge_discharge_authority: false,
        autonomous_action_authority: false,
    }
}

/// Apply the frozen ΩV1-D canary transformation or return the exact neutral text.
#[must_use]
pub fn render_live_response(prompt: &str, neutral_text: &str) -> LiveBridgeDecision {
    let Some(protected_body) = neutral_text.strip_prefix(ELIGIBLE_OPENER) else {
        return neutral_fallback(neutral_text, FallbackReason::IneligibleOpener, 0);
    };

    if protected_body.trim().is_empty() {
        return neutral_fallback(
            neutral_text,
            FallbackReason::EmptyProtectedBody,
            protected_body.len(),
        );
    }

    if protected_body.len() > MAX_PROTECTED_BODY_BYTES {
        return neutral_fallback(
            neutral_text,
            FallbackReason::ProtectedBodyTooLarge,
            protected_body.len(),
        );
    }

    let selected = select_opener(prompt, protected_body);
    if !REPLACEMENT_OPENERS.contains(&selected) {
        return neutral_fallback(
            neutral_text,
            FallbackReason::InvariantViolation,
            protected_body.len(),
        );
    }

    let rendered_text = format!("{selected}{protected_body}");
    let body_preserved_exactly = rendered_text
        .strip_prefix(selected)
        .is_some_and(|rendered_body| rendered_body.as_bytes() == protected_body.as_bytes());
    let growth = rendered_text.len().saturating_sub(neutral_text.len());

    if !body_preserved_exactly
        || !rendered_text.ends_with(protected_body)
        || growth > MAX_OUTPUT_GROWTH_BYTES
    {
        return neutral_fallback(
            neutral_text,
            FallbackReason::InvariantViolation,
            protected_body.len(),
        );
    }

    LiveBridgeDecision {
        mode: LiveBridgeMode::Applied,
        neutral_text: neutral_text.to_owned(),
        rendered_text,
        selected_opener: Some(selected.to_owned()),
        protected_body_bytes: protected_body.len(),
        body_preserved_exactly,
        fallback_reason: None,
    }
}

/// Convenience entry point reserved for the later ΩV1-D1 HTTP integration.
/// Fallback is represented by the exact original response, never an error.
#[must_use]
pub fn render_or_neutral(prompt: &str, neutral_text: &str) -> String {
    render_live_response(prompt, neutral_text).rendered_text
}

fn select_opener(prompt: &str, protected_body: &str) -> &'static str {
    let mut hash = HASH_OFFSET;
    hash = fnv1a_extend(hash, prompt.as_bytes());
    hash = fnv1a_extend(hash, &[0]);
    hash = fnv1a_extend(hash, protected_body.as_bytes());
    REPLACEMENT_OPENERS[(hash as usize) % REPLACEMENT_OPENERS.len()]
}

fn fnv1a_extend(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(HASH_PRIME);
    }
    hash
}

fn neutral_fallback(
    neutral_text: &str,
    reason: FallbackReason,
    protected_body_bytes: usize,
) -> LiveBridgeDecision {
    LiveBridgeDecision {
        mode: LiveBridgeMode::NeutralFallback,
        neutral_text: neutral_text.to_owned(),
        rendered_text: neutral_text.to_owned(),
        selected_opener: None,
        protected_body_bytes,
        body_preserved_exactly: true,
        fallback_reason: Some(reason),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eligible_response_is_deterministic_and_preserves_body() {
        let prompt = "Explain the next architecture step.";
        let neutral = "Here for it. The semantic body remains protected.";
        let first = render_live_response(prompt, neutral);
        let second = render_live_response(prompt, neutral);

        assert_eq!(first, second);
        assert_eq!(first.mode, LiveBridgeMode::Applied);
        assert!(first.body_preserved_exactly);
        let body = neutral.strip_prefix(ELIGIBLE_OPENER).unwrap();
        let selected = first.selected_opener.as_deref().unwrap();
        assert!(REPLACEMENT_OPENERS.contains(&selected));
        assert_eq!(first.rendered_text.strip_prefix(selected), Some(body));
        assert!(first.rendered_text.len() <= neutral.len() + MAX_OUTPUT_GROWTH_BYTES);
    }

    #[test]
    fn ineligible_response_returns_exact_neutral_text() {
        let neutral = "The architecture is already bounded.";
        let decision = render_live_response("Continue.", neutral);

        assert_eq!(decision.mode, LiveBridgeMode::NeutralFallback);
        assert_eq!(decision.rendered_text.as_bytes(), neutral.as_bytes());
        assert_eq!(decision.fallback_reason, Some(FallbackReason::IneligibleOpener));
    }

    #[test]
    fn empty_body_returns_exact_neutral_text() {
        let neutral = ELIGIBLE_OPENER;
        let decision = render_live_response("Continue.", neutral);

        assert_eq!(decision.mode, LiveBridgeMode::NeutralFallback);
        assert_eq!(decision.rendered_text.as_bytes(), neutral.as_bytes());
        assert_eq!(decision.fallback_reason, Some(FallbackReason::EmptyProtectedBody));
    }

    #[test]
    fn oversized_body_returns_exact_neutral_text() {
        let body = "x".repeat(MAX_PROTECTED_BODY_BYTES + 1);
        let neutral = format!("{ELIGIBLE_OPENER}{body}");
        let decision = render_live_response("Continue.", &neutral);

        assert_eq!(decision.mode, LiveBridgeMode::NeutralFallback);
        assert_eq!(decision.rendered_text.as_bytes(), neutral.as_bytes());
        assert_eq!(
            decision.fallback_reason,
            Some(FallbackReason::ProtectedBodyTooLarge)
        );
    }

    #[test]
    fn kernel_authority_remains_shadow_only() {
        let boundary = authority_boundary();
        assert!(boundary.bounded_transform_available);
        assert!(!boundary.api_chat_wiring);
        assert!(!boundary.live_generated_text_influence);
        assert!(!boundary.raw_conversation_access);
        assert!(!boundary.unrestricted_memory_access);
        assert!(!boundary.voice_state_mutation);
        assert!(!boundary.companion_state_mutation);
        assert!(!boundary.persistence_authority);
        assert!(!boundary.belief_promotion_authority);
        assert!(!boundary.ontology_promotion_authority);
        assert!(!boundary.routing_authority);
        assert!(!boundary.tool_selection_authority);
        assert!(!boundary.charge_discharge_authority);
        assert!(!boundary.autonomous_action_authority);
    }
}
