use crate::omega_v1f2_shadow::{
    PendingShadowEvent, ResponseFingerprint, ShadowIneligibility, ShadowInputBundle, ShadowIntent,
};
use crate::verified_improvisation::{
    ConversationalMicrostate, ImprovisationDisposition, ImprovisationLattice, ImprovisationRequest,
    ImprovisationalVerifier, RecentLanguageTrace, VerifiedImprovisationError,
    VerifiedImprovisationSelector,
};
use crate::verifier_ready_realization::{VerifierReadyRealizationError, VerifierReadyRenderer};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{info, warn};

pub const L1C_IMPLEMENTATION_VERSION: &str = "stlm-l1c-shadow-v1";
pub const L1C_AUTHORITY_MATRIX_VERSION: &str = "stlm-l1c-authority-v1";
pub const L1C_SHADOW_TIMEOUT_MS: u64 = 250;
pub const L1C_SHADOW_P95_TARGET_MS: u64 = 75;
const DEFAULT_LEDGER_FILENAME: &str = "stlm_l1c_shadow.jsonl";
const MAX_TRACE_KEYS: usize = 16;
const AUTHORITY_DOMAIN: &[u8] = b"starfire-stlm-l1c-authority-v1";
const COMPARISON_DOMAIN: &[u8] = b"starfire-stlm-l1c-comparison-v1";
const SEED_DOMAIN: &[u8] = b"starfire-stlm-l1c-seed-v1";
const TEXT_DOMAIN: &[u8] = b"starfire-stlm-l1c-text-fingerprint-v1";
const OMEGA_F2_RESPONSE_DOMAIN: &[u8] = b"starfire-omega-v1f2-response-v1";

static TRACE_STORE: OnceLock<Mutex<BTreeMap<String, RecentLanguageTrace>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct L1CShadowAuthorityBoundary {
    pub typed_shadow_bundle_access: bool,
    pub response_fingerprint_only: bool,
    pub verified_improvisation_execution: bool,
    pub neutral_control_comparison: bool,
    pub independent_candidate_verification: bool,
    pub ephemeral_fingerprint_trace: bool,
    pub bounded_metadata_recording: bool,
    pub candidate_text_return: bool,
    pub candidate_text_persistence: bool,
    pub raw_prompt_access: bool,
    pub raw_live_response_access: bool,
    pub unrestricted_conversation_access: bool,
    pub unrestricted_memory_access: bool,
    pub runtime_chat_response_influence: bool,
    pub http_response_influence: bool,
    pub voice_state_mutation: bool,
    pub companion_state_mutation: bool,
    pub general_persistence_authority: bool,
    pub belief_promotion_authority: bool,
    pub ontology_promotion_authority: bool,
    pub routing_authority: bool,
    pub tool_selection_authority: bool,
    pub charge_discharge_authority: bool,
    pub autonomous_action_authority: bool,
}

#[must_use]
pub const fn authority_boundary() -> L1CShadowAuthorityBoundary {
    L1CShadowAuthorityBoundary {
        typed_shadow_bundle_access: true,
        response_fingerprint_only: true,
        verified_improvisation_execution: true,
        neutral_control_comparison: true,
        independent_candidate_verification: true,
        ephemeral_fingerprint_trace: true,
        bounded_metadata_recording: true,
        candidate_text_return: false,
        candidate_text_persistence: false,
        raw_prompt_access: false,
        raw_live_response_access: false,
        unrestricted_conversation_access: false,
        unrestricted_memory_access: false,
        runtime_chat_response_influence: false,
        http_response_influence: false,
        voice_state_mutation: false,
        companion_state_mutation: false,
        general_persistence_authority: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        routing_authority: false,
        tool_selection_authority: false,
        charge_discharge_authority: false,
        autonomous_action_authority: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct L1CComparisonPayload {
    pub authority_matrix_digest: String,
    pub event_id: String,
    pub intent: String,
    pub sensitivity: String,
    pub program_digest: u64,
    pub lexical_table_digest: u64,
    pub response_before_digest: u64,
    pub response_after_digest: u64,
    pub response_before_len: u32,
    pub response_after_len: u32,
    pub response_bytes_preserved: bool,
    pub entropy_seed: u64,
    pub microstate: ConversationalMicrostate,
    pub trace_openings_before: u16,
    pub trace_surfaces_before: u16,
    pub trace_openings_after: u16,
    pub trace_surfaces_after: u16,
    pub trace_changed: bool,
    pub neutral_control_fingerprint: u64,
    pub candidate_opening_fingerprint: u64,
    pub candidate_surface_fingerprint: u64,
    pub selection_digest: u64,
    pub lattice_digest: Option<u64>,
    pub verification_digest: Option<u64>,
    pub selected_grammar_version: u16,
    pub disposition: String,
    pub variant_ids: Vec<u16>,
    pub score: i64,
    pub complete_candidates_scored: u16,
    pub fallback_reason: Option<String>,
    pub independent_verifier_accepted: bool,
    pub selection_verification_matches: bool,
    pub neutral_control_diverged: bool,
    pub candidate_matches_returned_response: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct L1CShadowLedgerRecord {
    pub schema_version: u16,
    pub implementation_version: String,
    pub utc_day_bucket: i64,
    pub utc_hour_bucket: i64,
    pub eligibility_code: String,
    pub response_before_digest: u64,
    pub response_after_digest: u64,
    pub response_before_len: u32,
    pub response_after_len: u32,
    pub response_bytes_preserved: bool,
    pub comparison_digest: Option<u64>,
    pub comparison: Option<L1CComparisonPayload>,
    pub failure_reason: Option<String>,
    pub trace_update_committed: bool,
    pub elapsed_micros: u64,
    pub timed_out: bool,
    pub panicked: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct L1CShadowProbeReport {
    pub experiment: String,
    pub implementation_version: String,
    pub eligible_observation_created: bool,
    pub independent_candidate_verified: bool,
    pub exact_replay: bool,
    pub neutral_control_diverged: bool,
    pub recent_language_changed_selection: bool,
    pub response_bytes_preserved: bool,
    pub candidate_text_absent_from_ledger: bool,
    pub ineligible_event_isolated: bool,
    pub authority_boundary_closed: bool,
    pub no_runtime_response_influence: bool,
    pub gate_passed: bool,
}

#[derive(Debug, Error)]
pub enum L1CShadowError {
    #[error("verified improvisation failed: {0}")]
    Improvisation(#[from] VerifiedImprovisationError),
    #[error("neutral control rendering failed: {0}")]
    Neutral(#[from] VerifierReadyRealizationError),
    #[error("shadow trace failed: {0}")]
    Trace(String),
    #[error("shadow metadata serialization failed: {0}")]
    Serialization(String),
    #[error("shadow metadata ledger failed: {0}")]
    Ledger(String),
}
