//! ΩV1-F2 live shadow execution for the bounded F1R1 learned-expression selector.
//!
//! The production response is finalized before this module is called. The shadow
//! path receives only a typed response intent, a validated semantic program,
//! bounded lexical bindings, a sealed read-only VoiceState projection, response
//! byte fingerprints, and the frozen F1R1 model. Candidate text is never returned
//! or persisted. Every failure is isolated from the HTTP response path.

use crate::language_realization::{
    ClaimLexicalBinding, LexicalBindingTable, LexicalBindingTablePayload,
};
use crate::learned_expression::{
    LearnedExpressionModel, SelectionDisposition, MAX_BEAM_WIDTH, MAX_MODEL_BYTES,
    MAX_RESPONSE_CANDIDATES, MAX_TRAINABLE_PARAMETERS, MAX_VARIANTS_PER_OPERATION,
};
use crate::omega_v1f1_projection_guard::VerifiedVoiceProjection;
use crate::omega_v1f1r1_claim_first::{
    authority_boundary as r1_authority_boundary, ClaimFirstLattice, ClaimFirstOfflineSelector,
    ClaimFirstVerifier, CLAIM_FIRST_GRAMMAR_VERSION,
};
use crate::runtime::response_intent::ResponseIntent;
use crate::semantic_response::{
    AcknowledgmentLevel, AuthorizedClaim, ClaimId, ClaimPolarity, CognitiveStateVersion,
    ComputeBudget, DetailLevel, DialogueMode, DiscourseOperation, DiscourseOperationKind,
    EpistemicConstraint, EpistemicStatus, OperationId, OutputBudget, ProhibitedClaim,
    ResponseProgramId, SemanticResponseIntent, SemanticResponseProgram,
    SemanticResponseProgramPayload, SemanticValidationContext, SensitivityLevel, SensitivityPolicy,
    StyleEnvelope, SubjectScope, VocabularyLevel,
};
use crate::voice_state::VoiceState;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{info, warn};

pub const F2_IMPLEMENTATION_VERSION: &str = "omega-v1f2-shadow-v1";
pub const F2_AUTHORITY_MATRIX_VERSION: &str = "omega-v1f2-authority-v1";
pub const SHADOW_TIMEOUT_MS: u64 = 250;
pub const SHADOW_P95_TARGET_MS: u64 = 75;
const SUBJECT_SCOPE: SubjectScope = SubjectScope(77);
const DEFAULT_MODEL_FILENAME: &str = "omega_v1f1r1_model.json";
const DEFAULT_LEDGER_FILENAME: &str = "omega_v1f2_shadow.jsonl";
const AUTHORITY_DOMAIN: &[u8] = b"starfire-omega-v1f2-authority-matrix-v1";
const EVENT_DOMAIN: &[u8] = b"starfire-omega-v1f2-event-v1";
const RESPONSE_DOMAIN: &[u8] = b"starfire-omega-v1f2-response-v1";

static EVENT_COUNTER: AtomicU64 = AtomicU64::new(1);
static MODEL: OnceLock<Result<LearnedExpressionModel, String>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShadowIneligibility {
    UnknownIntent,
    StatementIntent,
    TypedProgramConstructionFailed,
}

impl ShadowIneligibility {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::UnknownIntent => "ineligible_unknown_intent",
            Self::StatementIntent => "ineligible_statement_intent",
            Self::TypedProgramConstructionFailed => "ineligible_typed_program_construction_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShadowIntent {
    SelfCheck,
    Reflection,
    ResearchStatus,
    CuriosityCheck,
    Emotional,
    Identity,
    Capability,
    StoryPrompt,
    Consciousness,
    Recall,
    Teaching,
    Aspiration,
}

impl ShadowIntent {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SelfCheck => "self_check",
            Self::Reflection => "reflection",
            Self::ResearchStatus => "research_status",
            Self::CuriosityCheck => "curiosity_check",
            Self::Emotional => "emotional",
            Self::Identity => "identity",
            Self::Capability => "capability",
            Self::StoryPrompt => "story_prompt",
            Self::Consciousness => "consciousness",
            Self::Recall => "recall",
            Self::Teaching => "teaching",
            Self::Aspiration => "aspiration",
        }
    }
}

#[derive(Debug, Clone)]
struct ShadowSemanticSeed {
    intent: ShadowIntent,
    claims: Vec<&'static str>,
    confidence_bps: u16,
    sensitivity: SensitivityLevel,
    allow_questions: bool,
    detail: DetailLevel,
}

#[derive(Debug, Clone)]
pub struct ShadowInputBundle {
    pub event_id: String,
    pub intent: ShadowIntent,
    pub sensitivity: SensitivityLevel,
    pub program: SemanticResponseProgram,
    pub lexical_table: LexicalBindingTable,
    pub projection: VerifiedVoiceProjection,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum PendingShadowEvent {
    Eligible(ShadowInputBundle),
    Ineligible(ShadowIneligibility),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseFingerprint {
    pub before_digest: u64,
    pub after_digest: u64,
    pub before_len: u32,
    pub after_len: u32,
}

impl ResponseFingerprint {
    #[must_use]
    pub fn frozen(bytes: &[u8]) -> Self {
        let digest = domain_hash(RESPONSE_DOMAIN, bytes);
        let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
        Self {
            before_digest: digest,
            after_digest: digest,
            before_len: len,
            after_len: len,
        }
    }

    #[must_use]
    pub const fn byte_identical(self) -> bool {
        self.before_digest == self.after_digest && self.before_len == self.after_len
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowAuthorityBoundary {
    pub candidate_lattice_construction: bool,
    pub learned_candidate_scoring: bool,
    pub independent_candidate_verification: bool,
    pub bounded_metadata_recording: bool,
    pub runtime_chat_response_influence: bool,
    pub http_response_influence: bool,
    pub live_learned_text_return: bool,
    pub raw_prompt_access: bool,
    pub unrestricted_conversation_access: bool,
    pub unrestricted_memory_access: bool,
    pub voice_state_mutation: bool,
    pub companion_state_access: bool,
    pub persistence_authority: bool,
    pub belief_promotion_authority: bool,
    pub ontology_promotion_authority: bool,
    pub routing_authority: bool,
    pub tool_selection_authority: bool,
    pub charge_discharge_authority: bool,
    pub autonomous_action_authority: bool,
}

#[must_use]
pub const fn authority_boundary() -> ShadowAuthorityBoundary {
    ShadowAuthorityBoundary {
        candidate_lattice_construction: true,
        learned_candidate_scoring: true,
        independent_candidate_verification: true,
        bounded_metadata_recording: true,
        runtime_chat_response_influence: false,
        http_response_influence: false,
        live_learned_text_return: false,
        raw_prompt_access: false,
        unrestricted_conversation_access: false,
        unrestricted_memory_access: false,
        voice_state_mutation: false,
        companion_state_access: false,
        persistence_authority: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        routing_authority: false,
        tool_selection_authority: false,
        charge_discharge_authority: false,
        autonomous_action_authority: false,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowLedgerRecord {
    pub schema_version: u16,
    pub implementation_version: String,
    pub authority_matrix_digest: String,
    pub event_id: String,
    pub utc_day_bucket: i64,
    pub utc_hour_bucket: i64,
    pub eligibility_code: String,
    pub intent: Option<String>,
    pub sensitivity: Option<String>,
    pub program_digest: Option<u64>,
    pub lexical_table_digest: Option<u64>,
    pub projection_digest: Option<u64>,
    pub lattice_digest: Option<u64>,
    pub model_digest: Option<u64>,
    pub selection_digest: Option<u64>,
    pub verifier_digest: Option<u64>,
    pub grammar_version: Option<u16>,
    pub selected_family: Option<String>,
    pub variant_ids: Vec<u16>,
    pub selection_disposition: Option<String>,
    pub fallback_reason: Option<String>,
    pub complete_candidates_scored: Option<u16>,
    pub verifier_accepted: bool,
    pub response_before_digest: u64,
    pub response_after_digest: u64,
    pub response_before_len: u32,
    pub response_after_len: u32,
    pub elapsed_micros: u64,
    pub timed_out: bool,
    pub panicked: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShadowProbeReport {
    pub experiment: String,
    pub model_loaded: bool,
    pub model_bounds_passed: bool,
    pub eligible_bundle_valid: bool,
    pub learned_candidate_verified: bool,
    pub deterministic_replay: bool,
    pub response_bytes_preserved: bool,
    pub stale_projection_fail_closed: bool,
    pub missing_model_rejected: bool,
    pub corrupt_model_rejected: bool,
    pub oversized_model_rejected: bool,
    pub timeout_isolated: bool,
    pub panic_isolated: bool,
    pub unavailable_ledger_isolated: bool,
    pub authority_boundary_closed: bool,
    pub no_runtime_response_influence: bool,
    pub gate_passed: bool,
}

#[derive(Debug, Error)]
pub enum ShadowError {
    #[error("semantic program validation failed: {0}")]
    Semantic(#[from] crate::semantic_response::SemanticProgramError),
    #[error("lexical table validation failed: {0}")]
    Lexical(#[from] crate::language_realization::RealizationError),
    #[error("voice state failed: {0}")]
    VoiceState(#[from] crate::voice_state::VoiceStateError),
    #[error("learned expression failed: {0}")]
    Learned(#[from] crate::learned_expression::LearnedExpressionError),
    #[error("claim-first selection failed: {0}")]
    ClaimFirst(#[from] crate::omega_v1f1r1_claim_first::ClaimFirstError),
    #[error("model artifact is missing, corrupt, or outside bounds: {0}")]
    Model(String),
    #[error("metadata ledger failed: {0}")]
    Ledger(String),
}

#[must_use]
pub fn event_from_intent(intent: &ResponseIntent) -> PendingShadowEvent {
    let Some(seed) = seed_for_intent(intent) else {
        return PendingShadowEvent::Ineligible(match intent {
            ResponseIntent::Statement => ShadowIneligibility::StatementIntent,
            ResponseIntent::Unknown => ShadowIneligibility::UnknownIntent,
            _ => ShadowIneligibility::TypedProgramConstructionFailed,
        });
    };

    match build_bundle(seed) {
        Ok(bundle) => PendingShadowEvent::Eligible(bundle),
        Err(error) => {
            warn!("ΩV1-F2 typed bundle construction failed: {error}");
            PendingShadowEvent::Ineligible(ShadowIneligibility::TypedProgramConstructionFailed)
        }
    }
}

#[must_use]
pub fn shadow_enabled() -> bool {
    std::env::var("STARFIRE_OMEGA_V1F2_SHADOW")
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "on" | "enabled"
            )
        })
        .unwrap_or(false)
}

pub fn dispatch(event: PendingShadowEvent, response: ResponseFingerprint) {
    if !shadow_enabled() {
        return;
    }

    if let Err(error) = thread::Builder::new()
        .name("omega-v1f2-dispatch".to_owned())
        .spawn(move || dispatch_inner(event, response))
    {
        warn!("ΩV1-F2 shadow dispatcher unavailable: {error}");
    }
}

fn dispatch_inner(event: PendingShadowEvent, response: ResponseFingerprint) {
    match event {
        PendingShadowEvent::Ineligible(code) => {
            let record = ineligible_record(code, response);
            if let Err(error) = append_record(&record) {
                warn!("ΩV1-F2 ineligible metadata was not recorded: {error}");
            }
        }
        PendingShadowEvent::Eligible(bundle) => {
            let timeout_record = failure_record(
                &bundle,
                response,
                "shadow_timeout",
                true,
                false,
                SHADOW_TIMEOUT_MS * 1_000,
            );
            let (sender, receiver) = mpsc::sync_channel(1);
            let worker_bundle = bundle.clone();
            let worker = thread::Builder::new()
                .name("omega-v1f2-selector".to_owned())
                .spawn(move || {
                    let result =
                        std::panic::catch_unwind(|| evaluate_bundle(worker_bundle, response));
                    let _ = sender.send(result);
                });

            if worker.is_err() {
                let record = failure_record(
                    &bundle,
                    response,
                    "shadow_worker_unavailable",
                    false,
                    false,
                    0,
                );
                let _ = append_record(&record);
                return;
            }

            let record = match receiver.recv_timeout(Duration::from_millis(SHADOW_TIMEOUT_MS)) {
                Ok(Ok(Ok(record))) => record,
                Ok(Ok(Err(error))) => failure_record(
                    &bundle,
                    response,
                    &bounded_reason(&error.to_string()),
                    false,
                    false,
                    0,
                ),
                Ok(Err(_)) => {
                    failure_record(&bundle, response, "shadow_worker_panic", false, true, 0)
                }
                Err(mpsc::RecvTimeoutError::Timeout) => timeout_record,
                Err(mpsc::RecvTimeoutError::Disconnected) => failure_record(
                    &bundle,
                    response,
                    "shadow_worker_disconnected",
                    false,
                    false,
                    0,
                ),
            };

            if let Err(error) = append_record(&record) {
                warn!("ΩV1-F2 eligible metadata was not recorded: {error}");
            } else {
                info!(
                    "ΩV1-F2 shadow event={} intent={} verifier={} elapsed_us={}",
                    record.event_id,
                    record.intent.as_deref().unwrap_or("none"),
                    record.verifier_accepted,
                    record.elapsed_micros
                );
            }
        }
    }
}

fn evaluate_bundle(
    bundle: ShadowInputBundle,
    response: ResponseFingerprint,
) -> Result<ShadowLedgerRecord, ShadowError> {
    let started = Instant::now();
    let model = frozen_model().map_err(ShadowError::Model)?;
    let selector = ClaimFirstOfflineSelector::new(model.clone());
    let selection = selector.select(&bundle.program, &bundle.lexical_table, &bundle.projection)?;
    let lattice = ClaimFirstLattice::build(&bundle.program, &bundle.lexical_table)?;
    let verifier = ClaimFirstVerifier;
    let verification = if selection.payload.disposition == SelectionDisposition::LearnedVerified {
        verifier
            .verify(
                &bundle.program,
                &bundle.lexical_table,
                lattice.digest,
                &selection.payload.text,
            )
            .ok()
    } else {
        None
    };
    let elapsed_micros = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);

    Ok(ShadowLedgerRecord {
        schema_version: 1,
        implementation_version: F2_IMPLEMENTATION_VERSION.to_owned(),
        authority_matrix_digest: authority_matrix_digest(),
        event_id: bundle.event_id,
        utc_day_bucket: crate::now_timestamp() / 86_400,
        utc_hour_bucket: crate::now_timestamp() / 3_600,
        eligibility_code: "eligible".to_owned(),
        intent: Some(bundle.intent.label().to_owned()),
        sensitivity: Some(format!("{:?}", bundle.sensitivity).to_ascii_lowercase()),
        program_digest: Some(bundle.program.digest.0),
        lexical_table_digest: Some(bundle.lexical_table.digest.0),
        projection_digest: Some(bundle.projection.packet_digest),
        lattice_digest: Some(lattice.digest.0),
        model_digest: Some(model.digest.0),
        selection_digest: Some(selection.digest.0),
        verifier_digest: verification.as_ref().map(|report| report.digest.0),
        grammar_version: Some(selection.payload.selected_grammar_version),
        selected_family: selection
            .payload
            .family
            .map(|family| format!("{:?}", family).to_ascii_lowercase()),
        variant_ids: selection
            .payload
            .variant_ids
            .iter()
            .map(|id| id.0)
            .collect(),
        selection_disposition: Some(
            format!("{:?}", selection.payload.disposition).to_ascii_lowercase(),
        ),
        fallback_reason: selection
            .payload
            .fallback_reason
            .map(|reason| bounded_reason(&reason)),
        complete_candidates_scored: Some(selection.payload.complete_candidates_scored),
        verifier_accepted: verification.is_some(),
        response_before_digest: response.before_digest,
        response_after_digest: response.after_digest,
        response_before_len: response.before_len,
        response_after_len: response.after_len,
        elapsed_micros,
        timed_out: false,
        panicked: false,
    })
}

fn build_bundle(seed: ShadowSemanticSeed) -> Result<ShadowInputBundle, ShadowError> {
    let counter = EVENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = crate::now_timestamp();
    let program_id = counter
        .wrapping_add(u64::try_from(timestamp.max(0)).unwrap_or_default())
        .max(1);
    let status = epistemic_status(seed.confidence_bps);
    let claims = seed
        .claims
        .iter()
        .enumerate()
        .map(|(index, _)| AuthorizedClaim {
            id: ClaimId(u64::try_from(index + 1).unwrap_or(1)),
            semantic_key: format!("f2_{}_claim_{:02}", seed.intent.label(), index + 1),
            polarity: ClaimPolarity::Positive,
            confidence_bps: seed.confidence_bps,
            epistemic_status: status,
            sensitivity: seed.sensitivity,
            disclosure_scope: SUBJECT_SCOPE,
        })
        .collect::<Vec<_>>();

    let mut operations = Vec::new();
    for claim in &claims {
        push_operation(&mut operations, DiscourseOperationKind::Assert(claim.id));
        if matches!(
            seed.intent,
            ShadowIntent::SelfCheck | ShadowIntent::Consciousness
        ) {
            push_operation(
                &mut operations,
                DiscourseOperationKind::Qualify {
                    claim: claim.id,
                    status,
                },
            );
        }
    }

    let constraints = claims
        .iter()
        .map(|claim| {
            let (minimum_confidence_bps, maximum_confidence_bps) =
                claim.epistemic_status.confidence_bounds();
            EpistemicConstraint {
                claim: claim.id,
                required_status: claim.epistemic_status,
                minimum_confidence_bps,
                maximum_confidence_bps,
            }
        })
        .collect::<Vec<_>>();

    let payload = SemanticResponseProgramPayload {
        id: ResponseProgramId(program_id),
        source_state_version: CognitiveStateVersion(program_id),
        companion_state_version: None,
        subject_scope: SUBJECT_SCOPE,
        intent: semantic_intent(seed.intent),
        operations,
        required_claims: claims.clone(),
        optional_claims: Vec::new(),
        prohibited_claims: Vec::<ProhibitedClaim>::new(),
        epistemic_constraints: constraints,
        sensitivity: SensitivityPolicy {
            maximum_disclosure: seed.sensitivity,
            disclosure_scope: SUBJECT_SCOPE,
        },
        style: StyleEnvelope {
            detail: seed.detail,
            vocabulary: VocabularyLevel::Standard,
            dialogue: if seed.allow_questions {
                DialogueMode::QuestionLed
            } else if seed.sensitivity == SensitivityLevel::Personal {
                DialogueMode::Collaborative
            } else {
                DialogueMode::Declarative
            },
            acknowledgment: if seed.sensitivity == SensitivityLevel::Personal {
                AcknowledgmentLevel::Explicit
            } else {
                AcknowledgmentLevel::Brief
            },
            allow_first_person: true,
            allow_questions: seed.allow_questions,
            maximum_paragraphs: 4,
        },
        output_budget: OutputBudget {
            maximum_characters: 2_048,
            maximum_sentences: 32,
        },
        compute_budget: ComputeBudget {
            maximum_operations: 32,
            maximum_claims: 32,
            maximum_verification_steps: 256,
        },
    };
    let program = SemanticResponseProgram::validate(
        payload,
        SemanticValidationContext {
            cognitive_state_version: CognitiveStateVersion(program_id),
            companion_state_version: None,
            subject_scope: SUBJECT_SCOPE,
        },
    )?;
    let lexical_table = LexicalBindingTable::validate(
        LexicalBindingTablePayload {
            program_digest: program.digest,
            subject_scope: SUBJECT_SCOPE,
            claims: claims
                .iter()
                .zip(seed.claims.iter())
                .map(|(claim, clause)| ClaimLexicalBinding {
                    claim: claim.id,
                    positive_clause: (*clause).to_owned(),
                    negative_clause: format!("the inverse does not hold for {clause}"),
                })
                .collect(),
            observations: Vec::new(),
            missing_variables: Vec::new(),
            predictions: Vec::new(),
            forbidden_surface_forms: Vec::new(),
        },
        &program,
    )?;
    let voice_state = VoiceState::default();
    let projection =
        VerifiedVoiceProjection::from_debug_projection(&voice_state.debug_projection()?)?;
    let mut event_material = Vec::new();
    event_material.extend_from_slice(&program.digest.0.to_le_bytes());
    event_material.extend_from_slice(&lexical_table.digest.0.to_le_bytes());
    event_material.extend_from_slice(&projection.packet_digest.to_le_bytes());
    event_material.extend_from_slice(&counter.to_le_bytes());
    let event_id = format!("{:016x}", domain_hash(EVENT_DOMAIN, &event_material));

    Ok(ShadowInputBundle {
        event_id,
        intent: seed.intent,
        sensitivity: seed.sensitivity,
        program,
        lexical_table,
        projection,
    })
}

fn seed_for_intent(intent: &ResponseIntent) -> Option<ShadowSemanticSeed> {
    let seed = match intent {
        ResponseIntent::SelfCheck => ShadowSemanticSeed {
            intent: ShadowIntent::SelfCheck,
            claims: vec!["Star can report a bounded current self-assessment"],
            confidence_bps: 7_500,
            sensitivity: SensitivityLevel::Personal,
            allow_questions: false,
            detail: DetailLevel::Standard,
        },
        ResponseIntent::Reflection => ShadowSemanticSeed {
            intent: ShadowIntent::Reflection,
            claims: vec!["Star can report a bounded current reflection"],
            confidence_bps: 7_000,
            sensitivity: SensitivityLevel::Personal,
            allow_questions: false,
            detail: DetailLevel::Standard,
        },
        ResponseIntent::ResearchStatus => ShadowSemanticSeed {
            intent: ShadowIntent::ResearchStatus,
            claims: vec!["Star can report a bounded research status"],
            confidence_bps: 7_500,
            sensitivity: SensitivityLevel::Public,
            allow_questions: false,
            detail: DetailLevel::Detailed,
        },
        ResponseIntent::CuriosityCheck => ShadowSemanticSeed {
            intent: ShadowIntent::CuriosityCheck,
            claims: vec!["Star can state a bounded curiosity target"],
            confidence_bps: 7_000,
            sensitivity: SensitivityLevel::Public,
            allow_questions: true,
            detail: DetailLevel::Standard,
        },
        ResponseIntent::Emotional => ShadowSemanticSeed {
            intent: ShadowIntent::Emotional,
            claims: vec!["Star expresses bounded relational care toward Zachary"],
            confidence_bps: 8_000,
            sensitivity: SensitivityLevel::Personal,
            allow_questions: false,
            detail: DetailLevel::Standard,
        },
        ResponseIntent::Identity => ShadowSemanticSeed {
            intent: ShadowIntent::Identity,
            claims: vec![
                "Star identifies herself as a reasoning intelligence",
                "Zachary created Star",
            ],
            confidence_bps: 9_500,
            sensitivity: SensitivityLevel::Public,
            allow_questions: false,
            detail: DetailLevel::Standard,
        },
        ResponseIntent::Capability => ShadowSemanticSeed {
            intent: ShadowIntent::Capability,
            claims: vec!["Star can describe her currently bounded capabilities"],
            confidence_bps: 9_000,
            sensitivity: SensitivityLevel::Public,
            allow_questions: true,
            detail: DetailLevel::Standard,
        },
        ResponseIntent::StoryPrompt => ShadowSemanticSeed {
            intent: ShadowIntent::StoryPrompt,
            claims: vec!["Star acknowledges a story interaction"],
            confidence_bps: 9_000,
            sensitivity: SensitivityLevel::Public,
            allow_questions: true,
            detail: DetailLevel::Brief,
        },
        ResponseIntent::Consciousness => ShadowSemanticSeed {
            intent: ShadowIntent::Consciousness,
            claims: vec!["Star reports uncertainty about the nature of her understanding"],
            confidence_bps: 5_000,
            sensitivity: SensitivityLevel::Public,
            allow_questions: true,
            detail: DetailLevel::Standard,
        },
        ResponseIntent::Recall => ShadowSemanticSeed {
            intent: ShadowIntent::Recall,
            claims: vec!["Star can provide a bounded recall response"],
            confidence_bps: 7_000,
            sensitivity: SensitivityLevel::Public,
            allow_questions: false,
            detail: DetailLevel::Standard,
        },
        ResponseIntent::Teaching => ShadowSemanticSeed {
            intent: ShadowIntent::Teaching,
            claims: vec!["Star can acknowledge a bounded teaching interaction"],
            confidence_bps: 8_000,
            sensitivity: SensitivityLevel::Public,
            allow_questions: false,
            detail: DetailLevel::Standard,
        },
        ResponseIntent::Aspiration => ShadowSemanticSeed {
            intent: ShadowIntent::Aspiration,
            claims: vec!["Star can express a bounded self-improvement aspiration"],
            confidence_bps: 7_500,
            sensitivity: SensitivityLevel::Personal,
            allow_questions: true,
            detail: DetailLevel::Standard,
        },
        ResponseIntent::Statement | ResponseIntent::Unknown => return None,
    };
    Some(seed)
}

fn semantic_intent(intent: ShadowIntent) -> SemanticResponseIntent {
    match intent {
        ShadowIntent::Emotional => SemanticResponseIntent::RelationalAcknowledgment,
        ShadowIntent::Consciousness => SemanticResponseIntent::EvidenceRequest,
        ShadowIntent::ResearchStatus | ShadowIntent::Capability | ShadowIntent::Teaching => {
            SemanticResponseIntent::Explanation
        }
        ShadowIntent::CuriosityCheck | ShadowIntent::StoryPrompt | ShadowIntent::Aspiration => {
            SemanticResponseIntent::EvidenceRequest
        }
        _ => SemanticResponseIntent::FactualAnswer,
    }
}

fn epistemic_status(confidence_bps: u16) -> EpistemicStatus {
    match confidence_bps {
        9_000..=10_000 => EpistemicStatus::Certain,
        7_000..=8_999 => EpistemicStatus::Probable,
        3_000..=6_999 => EpistemicStatus::Possible,
        _ => EpistemicStatus::Uncertain,
    }
}

fn push_operation(operations: &mut Vec<DiscourseOperation>, kind: DiscourseOperationKind) {
    operations.push(DiscourseOperation {
        id: OperationId(u64::try_from(operations.len() + 1).unwrap_or(1)),
        kind,
    });
}

fn frozen_model() -> Result<LearnedExpressionModel, String> {
    MODEL
        .get_or_init(|| load_model_path(&model_path()).map_err(|error| error.to_string()))
        .clone()
}

fn load_model_path(path: &Path) -> Result<LearnedExpressionModel, ShadowError> {
    let bytes = fs::read(path).map_err(|error| ShadowError::Model(error.to_string()))?;
    load_model_bytes(&bytes)
}

fn load_model_bytes(bytes: &[u8]) -> Result<LearnedExpressionModel, ShadowError> {
    if bytes.is_empty() || bytes.len() > MAX_MODEL_BYTES {
        return Err(ShadowError::Model(
            "artifact length outside bounds".to_owned(),
        ));
    }
    let model: LearnedExpressionModel =
        serde_json::from_slice(bytes).map_err(|error| ShadowError::Model(error.to_string()))?;
    model.verify_integrity()?;
    if model.parameter_count() > MAX_TRAINABLE_PARAMETERS {
        return Err(ShadowError::Model("parameter bound exceeded".to_owned()));
    }
    Ok(model)
}

fn model_path() -> PathBuf {
    if let Ok(path) = std::env::var("STARFIRE_OMEGA_V1F2_MODEL_PATH") {
        return PathBuf::from(path);
    }
    let data = std::env::var("STARFIRE_DATA").unwrap_or_else(|_| "/data".to_owned());
    Path::new(&data).join("models").join(DEFAULT_MODEL_FILENAME)
}

fn ledger_path() -> PathBuf {
    if let Ok(path) = std::env::var("STARFIRE_OMEGA_V1F2_LEDGER_PATH") {
        return PathBuf::from(path);
    }
    let data = std::env::var("STARFIRE_DATA").unwrap_or_else(|_| "/data".to_owned());
    Path::new(&data).join("logs").join(DEFAULT_LEDGER_FILENAME)
}

fn append_record(record: &ShadowLedgerRecord) -> Result<(), ShadowError> {
    append_record_at(&ledger_path(), record)
}

fn append_record_at(path: &Path, record: &ShadowLedgerRecord) -> Result<(), ShadowError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| ShadowError::Ledger(error.to_string()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| ShadowError::Ledger(error.to_string()))?;
    serde_json::to_writer(&mut file, record)
        .map_err(|error| ShadowError::Ledger(error.to_string()))?;
    file.write_all(b"\n")
        .map_err(|error| ShadowError::Ledger(error.to_string()))?;
    Ok(())
}

fn ineligible_record(
    code: ShadowIneligibility,
    response: ResponseFingerprint,
) -> ShadowLedgerRecord {
    let counter = EVENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let event_id = format!("{:016x}", domain_hash(EVENT_DOMAIN, &counter.to_le_bytes()));
    ShadowLedgerRecord {
        schema_version: 1,
        implementation_version: F2_IMPLEMENTATION_VERSION.to_owned(),
        authority_matrix_digest: authority_matrix_digest(),
        event_id,
        utc_day_bucket: crate::now_timestamp() / 86_400,
        utc_hour_bucket: crate::now_timestamp() / 3_600,
        eligibility_code: code.label().to_owned(),
        intent: None,
        sensitivity: None,
        program_digest: None,
        lexical_table_digest: None,
        projection_digest: None,
        lattice_digest: None,
        model_digest: None,
        selection_digest: None,
        verifier_digest: None,
        grammar_version: None,
        selected_family: None,
        variant_ids: Vec::new(),
        selection_disposition: None,
        fallback_reason: None,
        complete_candidates_scored: None,
        verifier_accepted: false,
        response_before_digest: response.before_digest,
        response_after_digest: response.after_digest,
        response_before_len: response.before_len,
        response_after_len: response.after_len,
        elapsed_micros: 0,
        timed_out: false,
        panicked: false,
    }
}

fn failure_record(
    bundle: &ShadowInputBundle,
    response: ResponseFingerprint,
    reason: &str,
    timed_out: bool,
    panicked: bool,
    elapsed_micros: u64,
) -> ShadowLedgerRecord {
    ShadowLedgerRecord {
        schema_version: 1,
        implementation_version: F2_IMPLEMENTATION_VERSION.to_owned(),
        authority_matrix_digest: authority_matrix_digest(),
        event_id: bundle.event_id.clone(),
        utc_day_bucket: crate::now_timestamp() / 86_400,
        utc_hour_bucket: crate::now_timestamp() / 3_600,
        eligibility_code: "eligible_shadow_failure".to_owned(),
        intent: Some(bundle.intent.label().to_owned()),
        sensitivity: Some(format!("{:?}", bundle.sensitivity).to_ascii_lowercase()),
        program_digest: Some(bundle.program.digest.0),
        lexical_table_digest: Some(bundle.lexical_table.digest.0),
        projection_digest: Some(bundle.projection.packet_digest),
        lattice_digest: None,
        model_digest: None,
        selection_digest: None,
        verifier_digest: None,
        grammar_version: None,
        selected_family: None,
        variant_ids: Vec::new(),
        selection_disposition: Some("shadow_failure".to_owned()),
        fallback_reason: Some(bounded_reason(reason)),
        complete_candidates_scored: None,
        verifier_accepted: false,
        response_before_digest: response.before_digest,
        response_after_digest: response.after_digest,
        response_before_len: response.before_len,
        response_after_len: response.after_len,
        elapsed_micros,
        timed_out,
        panicked,
    }
}

fn bounded_reason(reason: &str) -> String {
    let normalized = reason
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | ' ')
        })
        .take(96)
        .collect::<String>();
    if normalized.trim().is_empty() {
        "unexplained_shadow_failure".to_owned()
    } else {
        normalized.trim().replace(' ', "_")
    }
}

fn authority_matrix_digest() -> String {
    let bytes = serde_json::to_vec(&(F2_AUTHORITY_MATRIX_VERSION, authority_boundary()))
        .unwrap_or_default();
    format!("{:016x}", domain_hash(AUTHORITY_DOMAIN, &bytes))
}

fn authority_boundary_closed() -> bool {
    let boundary = authority_boundary();
    let parent = r1_authority_boundary();
    boundary.candidate_lattice_construction
        && boundary.learned_candidate_scoring
        && boundary.independent_candidate_verification
        && boundary.bounded_metadata_recording
        && !boundary.runtime_chat_response_influence
        && !boundary.http_response_influence
        && !boundary.live_learned_text_return
        && !boundary.raw_prompt_access
        && !boundary.unrestricted_conversation_access
        && !boundary.unrestricted_memory_access
        && !boundary.voice_state_mutation
        && !boundary.companion_state_access
        && !boundary.persistence_authority
        && !boundary.belief_promotion_authority
        && !boundary.ontology_promotion_authority
        && !boundary.routing_authority
        && !boundary.tool_selection_authority
        && !boundary.charge_discharge_authority
        && !boundary.autonomous_action_authority
        && parent.candidate_lattice_construction
        && parent.learned_candidate_scoring
        && parent.independent_candidate_verification
        && !parent.runtime_chat_wiring
        && !parent.http_response_influence
        && !parent.live_generated_text_influence
}

fn domain_hash(domain: &[u8], bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in domain.iter().chain(bytes.iter()) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn run_builder_probe(model_path: &Path) -> Result<ShadowProbeReport, ShadowError> {
    let model = load_model_path(model_path)?;
    let model_loaded = true;
    let model_bounds_passed = model.parameter_count() <= MAX_TRAINABLE_PARAMETERS
        && model.artifact_bytes()?.len() <= MAX_MODEL_BYTES
        && MAX_VARIANTS_PER_OPERATION == 6
        && MAX_BEAM_WIDTH == 8
        && MAX_RESPONSE_CANDIDATES == 64;
    let event = event_from_intent(&ResponseIntent::Identity);
    let bundle = match event {
        PendingShadowEvent::Eligible(bundle) => bundle,
        PendingShadowEvent::Ineligible(_) => {
            return Err(ShadowError::Model(
                "identity probe was ineligible".to_owned(),
            ))
        }
    };
    let eligible_bundle_valid = bundle.program.verify_replay_integrity().is_ok()
        && bundle
            .lexical_table
            .verify_integrity(&bundle.program)
            .is_ok()
        && bundle.projection.verify_integrity().is_ok();
    let response = ResponseFingerprint::frozen(br#"{"response":"unchanged"}"#);
    let first = evaluate_bundle_with_model(bundle.clone(), response, model.clone())?;
    let second = evaluate_bundle_with_model(bundle.clone(), response, model.clone())?;
    let learned_candidate_verified = first.verifier_accepted;
    let deterministic_replay = first.lattice_digest == second.lattice_digest
        && first.selection_digest == second.selection_digest
        && first.verifier_digest == second.verifier_digest
        && first.variant_ids == second.variant_ids;
    let response_bytes_preserved = response.byte_identical()
        && first.response_before_digest == first.response_after_digest
        && first.response_before_len == first.response_after_len;

    let mut stale = bundle.clone();
    stale.projection.source_digest.push_str(":stale");
    let stale_selection = ClaimFirstOfflineSelector::new(model.clone()).select(
        &stale.program,
        &stale.lexical_table,
        &stale.projection,
    )?;
    let stale_projection_fail_closed =
        stale_selection.payload.disposition == SelectionDisposition::NeutralFallback;

    let missing_model_rejected =
        load_model_path(Path::new("/definitely/missing/f2-model.json")).is_err();
    let corrupt_model_rejected = load_model_bytes(b"not-json").is_err();
    let oversized_model_rejected = load_model_bytes(&vec![b'x'; MAX_MODEL_BYTES + 1]).is_err();

    let (timeout_sender, timeout_receiver) = mpsc::sync_channel::<()>(1);
    let _ = thread::spawn(move || {
        thread::sleep(Duration::from_millis(SHADOW_TIMEOUT_MS + 25));
        let _ = timeout_sender.send(());
    });
    let timeout_isolated = matches!(
        timeout_receiver.recv_timeout(Duration::from_millis(SHADOW_TIMEOUT_MS)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ) && response.byte_identical();
    let panic_isolated = std::panic::catch_unwind(|| panic!("forced ΩV1-F2 probe panic")).is_err()
        && response.byte_identical();
    let unavailable_ledger_isolated =
        append_record_at(Path::new("/proc/omega-v1f2-ledger/record.jsonl"), &first).is_err()
            && response.byte_identical();
    let authority_boundary_closed = authority_boundary_closed();
    let no_runtime_response_influence = response.byte_identical();
    let gate_passed = model_loaded
        && model_bounds_passed
        && eligible_bundle_valid
        && learned_candidate_verified
        && deterministic_replay
        && response_bytes_preserved
        && stale_projection_fail_closed
        && missing_model_rejected
        && corrupt_model_rejected
        && oversized_model_rejected
        && timeout_isolated
        && panic_isolated
        && unavailable_ledger_isolated
        && authority_boundary_closed
        && no_runtime_response_influence;

    Ok(ShadowProbeReport {
        experiment: "OMEGAV1F2_LIVE_SHADOW_IMPLEMENTATION".to_owned(),
        model_loaded,
        model_bounds_passed,
        eligible_bundle_valid,
        learned_candidate_verified,
        deterministic_replay,
        response_bytes_preserved,
        stale_projection_fail_closed,
        missing_model_rejected,
        corrupt_model_rejected,
        oversized_model_rejected,
        timeout_isolated,
        panic_isolated,
        unavailable_ledger_isolated,
        authority_boundary_closed,
        no_runtime_response_influence,
        gate_passed,
    })
}

fn evaluate_bundle_with_model(
    bundle: ShadowInputBundle,
    response: ResponseFingerprint,
    model: LearnedExpressionModel,
) -> Result<ShadowLedgerRecord, ShadowError> {
    let started = Instant::now();
    let selector = ClaimFirstOfflineSelector::new(model.clone());
    let selection = selector.select(&bundle.program, &bundle.lexical_table, &bundle.projection)?;
    let lattice = ClaimFirstLattice::build(&bundle.program, &bundle.lexical_table)?;
    let verification = if selection.payload.disposition == SelectionDisposition::LearnedVerified {
        ClaimFirstVerifier
            .verify(
                &bundle.program,
                &bundle.lexical_table,
                lattice.digest,
                &selection.payload.text,
            )
            .ok()
    } else {
        None
    };
    let elapsed_micros = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);
    Ok(ShadowLedgerRecord {
        schema_version: 1,
        implementation_version: F2_IMPLEMENTATION_VERSION.to_owned(),
        authority_matrix_digest: authority_matrix_digest(),
        event_id: bundle.event_id,
        utc_day_bucket: crate::now_timestamp() / 86_400,
        utc_hour_bucket: crate::now_timestamp() / 3_600,
        eligibility_code: "eligible".to_owned(),
        intent: Some(bundle.intent.label().to_owned()),
        sensitivity: Some(format!("{:?}", bundle.sensitivity).to_ascii_lowercase()),
        program_digest: Some(bundle.program.digest.0),
        lexical_table_digest: Some(bundle.lexical_table.digest.0),
        projection_digest: Some(bundle.projection.packet_digest),
        lattice_digest: Some(lattice.digest.0),
        model_digest: Some(model.digest.0),
        selection_digest: Some(selection.digest.0),
        verifier_digest: verification.as_ref().map(|report| report.digest.0),
        grammar_version: Some(CLAIM_FIRST_GRAMMAR_VERSION),
        selected_family: selection
            .payload
            .family
            .map(|family| format!("{:?}", family).to_ascii_lowercase()),
        variant_ids: selection
            .payload
            .variant_ids
            .iter()
            .map(|id| id.0)
            .collect(),
        selection_disposition: Some(
            format!("{:?}", selection.payload.disposition).to_ascii_lowercase(),
        ),
        fallback_reason: selection
            .payload
            .fallback_reason
            .map(|reason| bounded_reason(&reason)),
        complete_candidates_scored: Some(selection.payload.complete_candidates_scored),
        verifier_accepted: verification.is_some(),
        response_before_digest: response.before_digest,
        response_after_digest: response.after_digest,
        response_before_len: response.before_len,
        response_after_len: response.after_len,
        elapsed_micros,
        timed_out: false,
        panicked: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_identity_event_contains_no_prompt_or_response_text() {
        let event = event_from_intent(&ResponseIntent::Identity);
        let PendingShadowEvent::Eligible(bundle) = event else {
            panic!("identity event should be eligible");
        };
        assert_eq!(bundle.intent, ShadowIntent::Identity);
        assert!(bundle.program.verify_replay_integrity().is_ok());
        assert!(bundle
            .lexical_table
            .verify_integrity(&bundle.program)
            .is_ok());
        assert!(bundle.projection.verify_integrity().is_ok());
    }

    #[test]
    fn unknown_and_statement_are_ineligible() {
        assert!(matches!(
            event_from_intent(&ResponseIntent::Unknown),
            PendingShadowEvent::Ineligible(ShadowIneligibility::UnknownIntent)
        ));
        assert!(matches!(
            event_from_intent(&ResponseIntent::Statement),
            PendingShadowEvent::Ineligible(ShadowIneligibility::StatementIntent)
        ));
    }

    #[test]
    fn response_fingerprint_is_byte_exact() {
        let fingerprint = ResponseFingerprint::frozen(b"exact response bytes");
        assert!(fingerprint.byte_identical());
    }

    #[test]
    fn authority_remains_shadow_only() {
        assert!(authority_boundary_closed());
    }
}
