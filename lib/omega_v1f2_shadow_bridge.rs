//! Public ΩV1-F2 facade with optional STLM L1-C shadow fan-out.
//!
//! The HTTP layer continues to call the established F2 API. This facade forwards
//! the same sealed typed event and frozen response-byte fingerprint to F2 and,
//! when explicitly enabled, to L1-C. Neither observer receives response text or
//! gains authority over the finalized response.

pub use crate::omega_v1f2_shadow_inner::{
    authority_boundary, event_from_intent, run_builder_probe, PendingShadowEvent,
    ResponseFingerprint, ShadowAuthorityBoundary, ShadowError, ShadowIneligibility,
    ShadowInputBundle, ShadowIntent, ShadowLedgerRecord, ShadowProbeReport,
    F2_AUTHORITY_MATRIX_VERSION, F2_IMPLEMENTATION_VERSION, SHADOW_P95_TARGET_MS,
    SHADOW_TIMEOUT_MS,
};

#[must_use]
pub fn shadow_enabled() -> bool {
    let f2_enabled = crate::omega_v1f2_shadow_inner::shadow_enabled();
    #[cfg(feature = "stlm-l1c-shadow")]
    {
        f2_enabled || crate::stlm_l1c_shadow::shadow_enabled()
    }
    #[cfg(not(feature = "stlm-l1c-shadow"))]
    f2_enabled
}

pub fn dispatch(event: PendingShadowEvent, response: ResponseFingerprint) {
    if crate::omega_v1f2_shadow_inner::shadow_enabled() {
        crate::omega_v1f2_shadow_inner::dispatch(event.clone(), response);
    }

    #[cfg(feature = "stlm-l1c-shadow")]
    if crate::stlm_l1c_shadow::shadow_enabled() {
        crate::stlm_l1c_shadow::dispatch(event, response);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_preserves_closed_f2_authority() {
        let boundary = authority_boundary();
        assert!(!boundary.runtime_chat_response_influence);
        assert!(!boundary.http_response_influence);
        assert!(!boundary.live_learned_text_return);
        assert!(!boundary.raw_prompt_access);
        assert!(!boundary.autonomous_action_authority);
    }
}
