//! STLM L1-A verified improvisational realization.
//!
//! This layer turns the frozen ΩV1-F1R1 remediated surface lattice into a
//! replayable candidate search shaped by a bounded conversational microstate,
//! a caller-supplied entropy seed, and a recent-language trace. Meaning remains
//! fixed by the semantic response program. Every selected surface is mapped
//! back to its committed remediation variant and independently verified. Any
//! lattice, scoring, trace, verification, or budget failure returns the frozen
//! grammar-v2 neutral realization.
//!
//! The module is offline-only. It has no Runtime::chat(), HTTP response, raw
//! conversation, unrestricted memory, persistence, routing, tool, CHARGE,
//! belief, ontology, companion-state mutation, or autonomous-action authority.

use crate::language_realization::{LexicalBindingTable, LexicalTableDigest};
use crate::omega_v1f1r1_surface::{
    ExpressionFamily, RemediatedExpressionError, RemediatedLattice, RemediatedLatticeDigest,
    RemediatedVariantId, RemediatedVerificationDigest, RemediatedVerifier,
};
use crate::semantic_response::{
    DetailLevel, DiscourseOperationKind, OperationId, ResponseProgramDigest,
    SemanticResponseProgram,
};
use crate::verifier_ready_realization::{
    VerifierReadyRealizationError, VerifierReadyRenderer, VERIFIER_READY_GRAMMAR_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const VERIFIED_IMPROVISATION_GRAMMAR_VERSION: u16 = 5;
pub const MAX_IMPROVISATION_BEAM_WIDTH: usize = 16;
pub const MAX_IMPROVISATION_CANDIDATES: usize = 32;
pub const MAX_LANGUAGE_TRACE_ITEMS: usize = 64;

const LATTICE_DOMAIN: &[u8] = b"starfire-stlm-l1a-improvisation-lattice-v1";
const VERIFY_DOMAIN: &[u8] = b"starfire-stlm-l1a-improvisation-verification-v1";
const SELECT_DOMAIN: &[u8] = b"starfire-stlm-l1a-improvisation-selection-v1";
const FINGERPRINT_DOMAIN: &[u8] = b"starfire-stlm-l1a-language-fingerprint-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovisationLatticeDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovisationVerificationDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovisationSelectionDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationalMicrostate {
    pub directness_bps: u16,
    pub warmth_bps: u16,
    pub energy_bps: u16,
    pub compression_bps: u16,
    pub playfulness_bps: u16,
    pub novelty_pressure_bps: u16,
}

impl ConversationalMicrostate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        directness_bps: u16,
        warmth_bps: u16,
        energy_bps: u16,
        compression_bps: u16,
        playfulness_bps: u16,
        novelty_pressure_bps: u16,
    ) -> Result<Self, VerifiedImprovisationError> {
        let state = Self {
            directness_bps,
            warmth_bps,
            energy_bps,
            compression_bps,
            playfulness_bps,
            novelty_pressure_bps,
        };
        state.verify_integrity()?;
        Ok(state)
    }

    pub fn verify_integrity(&self) -> Result<(), VerifiedImprovisationError> {
        if [
            self.directness_bps,
            self.warmth_bps,
            self.energy_bps,
            self.compression_bps,
            self.playfulness_bps,
            self.novelty_pressure_bps,
        ]
        .iter()
        .any(|value| *value > 10_000)
        {
            return Err(VerifiedImprovisationError::InvalidMicrostate);
        }
        Ok(())
    }
}

impl Default for ConversationalMicrostate {
    fn default() -> Self {
        Self {
            directness_bps: 7_000,
            warmth_bps: 5_000,
            energy_bps: 5_500,
            compression_bps: 6_500,
            playfulness_bps: 3_500,
            novelty_pressure_bps: 7_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RecentLanguageTrace {
    pub opening_fingerprints: Vec<u64>,
    pub surface_fingerprints: Vec<u64>,
}

impl RecentLanguageTrace {
    pub fn new(
        opening_fingerprints: Vec<u64>,
        surface_fingerprints: Vec<u64>,
    ) -> Result<Self, VerifiedImprovisationError> {
        let trace = Self {
            opening_fingerprints,
            surface_fingerprints,
        };
        trace.verify_integrity()?;
        Ok(trace)
    }

    pub fn verify_integrity(&self) -> Result<(), VerifiedImprovisationError> {
        if self.opening_fingerprints.len() > MAX_LANGUAGE_TRACE_ITEMS
            || self.surface_fingerprints.len() > MAX_LANGUAGE_TRACE_ITEMS
            || self.opening_fingerprints.contains(&0)
            || self.surface_fingerprints.contains(&0)
        {
            return Err(VerifiedImprovisationError::InvalidLanguageTrace);
        }
        Ok(())
    }

    pub fn record_text(&mut self, text: &str) -> Result<(), VerifiedImprovisationError> {
        if text.trim().is_empty() {
            return Err(VerifiedImprovisationError::InvalidLanguageTrace);
        }
        push_bounded(
            &mut self.opening_fingerprints,
            opening_fingerprint(text),
        );
        push_bounded(&mut self.surface_fingerprints, surface_fingerprint(text));
        self.verify_integrity()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovisationRequest {
    pub entropy_seed: u64,
    pub microstate: ConversationalMicrostate,
    pub recent_language: RecentLanguageTrace,
}

impl ImprovisationRequest {
    pub fn new(
        entropy_seed: u64,
        microstate: ConversationalMicrostate,
        recent_language: RecentLanguageTrace,
    ) -> Result<Self, VerifiedImprovisationError> {
        microstate.verify_integrity()?;
        recent_language.verify_integrity()?;
        Ok(Self {
            entropy_seed,
            microstate,
            recent_language,
        })
    }

    pub fn verify_integrity(&self) -> Result<(), VerifiedImprovisationError> {
        self.microstate.verify_integrity()?;
        self.recent_language.verify_integrity()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovisationalSurfaceVariant {
    pub operation: OperationId,
    pub variant_id: RemediatedVariantId,
    pub kind: DiscourseOperationKind,
    pub family: ExpressionFamily,
    pub phase: u8,
    pub remediated_text: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovisationLatticePayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub remediated_lattice_digest: RemediatedLatticeDigest,
    pub grammar_version: u16,
    pub variants: Vec<ImprovisationalSurfaceVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovisationLattice {
    pub payload: ImprovisationLatticePayload,
    pub digest: ImprovisationLatticeDigest,
}

impl ImprovisationLattice {
    pub fn build(
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Result<Self, VerifiedImprovisationError> {
        program.verify_replay_integrity()?;
        lexical_table.verify_integrity(program)?;
        let remediated = RemediatedLattice::build(program, lexical_table)?;
        let mut variants = Vec::with_capacity(remediated.payload.variants.len());
        for (operation_index, operation) in program.payload.operations.iter().enumerate() {
            for variant in remediated
                .payload
                .variants
                .iter()
                .filter(|variant| variant.operation == operation.id)
            {
                variants.push(ImprovisationalSurfaceVariant {
                    operation: operation.id,
                    variant_id: variant.variant_id,
                    kind: variant.kind.clone(),
                    family: variant.family,
                    phase: variant.phase,
                    remediated_text: variant.text.clone(),
                    text: improvise_surface(
                        &variant.text,
                        &variant.kind,
                        variant.family,
                        variant.phase,
                        operation_index,
                    )?,
                });
            }
        }
        validate_variants(program, &variants)?;
        let payload = ImprovisationLatticePayload {
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            remediated_lattice_digest: remediated.digest,
            grammar_version: VERIFIED_IMPROVISATION_GRAMMAR_VERSION,
            variants,
        };
        let digest = ImprovisationLatticeDigest(digest_value(LATTICE_DOMAIN, &payload)?);
        if digest.0 == 0 {
            return Err(VerifiedImprovisationError::EmptyDigest);
        }
        Ok(Self { payload, digest })
    }

    pub fn verify_integrity(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Result<(), VerifiedImprovisationError> {
        let rebuilt = Self::build(program, lexical_table)?;
        if &rebuilt != self {
            return Err(VerifiedImprovisationError::LatticeDigestMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedImprovisationVariant {
    pub operation: OperationId,
    pub variant_id: RemediatedVariantId,
    pub family: ExpressionFamily,
    pub phase: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovisationCosts {
    pub operation_cost: u32,
    pub claim_cost: u32,
    pub verification_step_cost: u32,
    pub character_cost: u32,
    pub sentence_count: u16,
    pub paragraph_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovisationVerificationPayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub lattice_digest: ImprovisationLatticeDigest,
    pub remediated_lattice_digest: RemediatedLatticeDigest,
    pub remediated_verification_digest: RemediatedVerificationDigest,
    pub grammar_version: u16,
    pub variants: Vec<VerifiedImprovisationVariant>,
    pub costs: ImprovisationCosts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovisationVerificationReport {
    pub payload: ImprovisationVerificationPayload,
    pub digest: ImprovisationVerificationDigest,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ImprovisationalVerifier;

impl ImprovisationalVerifier {
    pub fn verify(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        lattice_digest: ImprovisationLatticeDigest,
        text: &str,
    ) -> Result<ImprovisationVerificationReport, VerifiedImprovisationError> {
        if text.trim().is_empty() {
            return Err(VerifiedImprovisationError::UnsupportedSurface);
        }
        let lattice = ImprovisationLattice::build(program, lexical_table)?;
        if lattice.digest != lattice_digest {
            return Err(VerifiedImprovisationError::LatticeDigestMismatch);
        }
        reject_forbidden_text(text, &lexical_table.payload.forbidden_surface_forms)?;
        let matched = parse_exact(program, &lattice.payload.variants, text)?;
        let remediated_text = assemble_remediated(program, &matched);
        let remediated_report = RemediatedVerifier.verify(
            program,
            lexical_table,
            lattice.payload.remediated_lattice_digest,
            &remediated_text,
        )?;
        let costs = recompute_costs(
            program,
            text,
            remediated_report.payload.costs.claim_cost,
            remediated_report.payload.costs.verification_step_cost,
        )?;
        let payload = ImprovisationVerificationPayload {
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            lattice_digest: lattice.digest,
            remediated_lattice_digest: lattice.payload.remediated_lattice_digest,
            remediated_verification_digest: remediated_report.digest,
            grammar_version: VERIFIED_IMPROVISATION_GRAMMAR_VERSION,
            variants: matched
                .iter()
                .map(|variant| VerifiedImprovisationVariant {
                    operation: variant.operation,
                    variant_id: variant.variant_id,
                    family: variant.family,
                    phase: variant.phase,
                })
                .collect(),
            costs,
        };
        let digest = ImprovisationVerificationDigest(digest_value(VERIFY_DOMAIN, &payload)?);
        if digest.0 == 0 {
            return Err(VerifiedImprovisationError::EmptyDigest);
        }
        Ok(ImprovisationVerificationReport { payload, digest })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImprovisationDisposition {
    VerifiedImprovisation,
    NeutralFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovisationSelectionPayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub entropy_seed: u64,
    pub selected_grammar_version: u16,
    pub disposition: ImprovisationDisposition,
    pub text: String,
    pub variant_ids: Vec<RemediatedVariantId>,
    pub score: i64,
    pub complete_candidates_scored: u16,
    pub opening_fingerprint: u64,
    pub surface_fingerprint: u64,
    pub lattice_digest: Option<ImprovisationLatticeDigest>,
    pub verification_digest: Option<ImprovisationVerificationDigest>,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovisationSelectionResult {
    pub payload: ImprovisationSelectionPayload,
    pub digest: ImprovisationSelectionDigest,
}

#[derive(Debug, Clone)]
struct BeamCandidate {
    text: String,
    variant_ids: Vec<RemediatedVariantId>,
    score: i64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VerifiedImprovisationSelector;

impl VerifiedImprovisationSelector {
    pub fn select(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        request: &ImprovisationRequest,
    ) -> Result<ImprovisationSelectionResult, VerifiedImprovisationError> {
        program.verify_replay_integrity()?;
        lexical_table.verify_integrity(program)?;
        request.verify_integrity()?;
        let neutral = VerifierReadyRenderer.render(program, lexical_table)?;
        let attempted = self.try_select(program, lexical_table, request);
        let payload = match attempted {
            Ok(payload) => payload,
            Err(error) => ImprovisationSelectionPayload {
                program_digest: program.digest,
                lexical_table_digest: lexical_table.digest,
                entropy_seed: request.entropy_seed,
                selected_grammar_version: VERIFIER_READY_GRAMMAR_VERSION,
                disposition: ImprovisationDisposition::NeutralFallback,
                opening_fingerprint: opening_fingerprint(&neutral.payload.text),
                surface_fingerprint: surface_fingerprint(&neutral.payload.text),
                text: neutral.payload.text,
                variant_ids: Vec::new(),
                score: 0,
                complete_candidates_scored: 0,
                lattice_digest: None,
                verification_digest: None,
                fallback_reason: Some(error.to_string()),
            },
        };
        let digest = ImprovisationSelectionDigest(digest_value(SELECT_DOMAIN, &payload)?);
        if digest.0 == 0 {
            return Err(VerifiedImprovisationError::EmptyDigest);
        }
        Ok(ImprovisationSelectionResult { payload, digest })
    }

    fn try_select(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        request: &ImprovisationRequest,
    ) -> Result<ImprovisationSelectionPayload, VerifiedImprovisationError> {
        let lattice = ImprovisationLattice::build(program, lexical_table)?;
        let mut by_operation = BTreeMap::<OperationId, Vec<&ImprovisationalSurfaceVariant>>::new();
        for variant in &lattice.payload.variants {
            by_operation.entry(variant.operation).or_default().push(variant);
        }
        for variants in by_operation.values_mut() {
            variants.sort_by_key(|variant| variant.variant_id);
        }

        let mut beam = vec![BeamCandidate {
            text: String::new(),
            variant_ids: Vec::new(),
            score: 0,
        }];
        for (index, operation) in program.payload.operations.iter().enumerate() {
            let variants = by_operation
                .get(&operation.id)
                .ok_or(VerifiedImprovisationError::InvalidLattice)?;
            let separator = separator_before(program, index);
            let mut next = Vec::new();
            for partial in &beam {
                for variant in variants {
                    let mut text = partial.text.clone();
                    text.push_str(separator);
                    text.push_str(&variant.text);
                    let mut variant_ids = partial.variant_ids.clone();
                    variant_ids.push(variant.variant_id);
                    let score = partial.score
                        + score_variant(
                            &request.microstate,
                            request.entropy_seed,
                            program.digest.0,
                            variant,
                        );
                    next.push(BeamCandidate {
                        text,
                        variant_ids,
                        score,
                    });
                }
            }
            next.sort_by(|left, right| {
                right
                    .score
                    .cmp(&left.score)
                    .then_with(|| {
                        candidate_tie_key(request.entropy_seed, &left.variant_ids)
                            .cmp(&candidate_tie_key(request.entropy_seed, &right.variant_ids))
                    })
                    .then_with(|| left.variant_ids.cmp(&right.variant_ids))
            });
            next.truncate(MAX_IMPROVISATION_BEAM_WIDTH);
            beam = next;
        }
        if beam.is_empty() || beam.len() > MAX_IMPROVISATION_CANDIDATES {
            return Err(VerifiedImprovisationError::CandidateBudgetExceeded);
        }
        for candidate in &mut beam {
            candidate.score += score_complete_candidate(
                &candidate.text,
                &request.microstate,
                &request.recent_language,
                request.entropy_seed,
            );
        }
        beam.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| {
                    candidate_tie_key(request.entropy_seed, &left.variant_ids)
                        .cmp(&candidate_tie_key(request.entropy_seed, &right.variant_ids))
                })
                .then_with(|| left.variant_ids.cmp(&right.variant_ids))
        });
        let complete_candidates_scored = u16::try_from(beam.len())
            .map_err(|_| VerifiedImprovisationError::CandidateBudgetExceeded)?;
        for candidate in beam {
            if let Ok(report) = ImprovisationalVerifier.verify(
                program,
                lexical_table,
                lattice.digest,
                &candidate.text,
            ) {
                return Ok(ImprovisationSelectionPayload {
                    program_digest: program.digest,
                    lexical_table_digest: lexical_table.digest,
                    entropy_seed: request.entropy_seed,
                    selected_grammar_version: VERIFIED_IMPROVISATION_GRAMMAR_VERSION,
                    disposition: ImprovisationDisposition::VerifiedImprovisation,
                    opening_fingerprint: opening_fingerprint(&candidate.text),
                    surface_fingerprint: surface_fingerprint(&candidate.text),
                    text: candidate.text,
                    variant_ids: candidate.variant_ids,
                    score: candidate.score,
                    complete_candidates_scored,
                    lattice_digest: Some(lattice.digest),
                    verification_digest: Some(report.digest),
                    fallback_reason: None,
                });
            }
        }
        Err(VerifiedImprovisationError::NoVerifiedCandidate)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedImprovisationAuthorityBoundary {
    pub committed_surface_lattice: bool,
    pub conversational_microstate_scoring: bool,
    pub replayable_entropy_seed: bool,
    pub recent_language_anti_repetition: bool,
    pub blandness_penalty: bool,
    pub independent_candidate_verification: bool,
    pub runtime_chat_wiring: bool,
    pub http_response_influence: bool,
    pub live_generated_text_influence: bool,
    pub raw_prompt_access: bool,
    pub unrestricted_conversation_access: bool,
    pub unrestricted_memory_access: bool,
    pub persistence_authority: bool,
    pub voice_state_mutation: bool,
    pub companion_state_mutation: bool,
    pub belief_promotion_authority: bool,
    pub ontology_promotion_authority: bool,
    pub routing_authority: bool,
    pub tool_selection_authority: bool,
    pub charge_discharge_authority: bool,
    pub autonomous_action_authority: bool,
}

#[must_use]
pub const fn authority_boundary() -> VerifiedImprovisationAuthorityBoundary {
    VerifiedImprovisationAuthorityBoundary {
        committed_surface_lattice: true,
        conversational_microstate_scoring: true,
        replayable_entropy_seed: true,
        recent_language_anti_repetition: true,
        blandness_penalty: true,
        independent_candidate_verification: true,
        runtime_chat_wiring: false,
        http_response_influence: false,
        live_generated_text_influence: false,
        raw_prompt_access: false,
        unrestricted_conversation_access: false,
        unrestricted_memory_access: false,
        persistence_authority: false,
        voice_state_mutation: false,
        companion_state_mutation: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        routing_authority: false,
        tool_selection_authority: false,
        charge_discharge_authority: false,
        autonomous_action_authority: false,
    }
}

#[derive(Debug, Error)]
pub enum VerifiedImprovisationError {
    #[error("semantic program validation failed: {0}")]
    Semantic(#[from] crate::semantic_response::SemanticProgramError),
    #[error("lexical table validation failed: {0}")]
    Lexical(#[from] crate::language_realization::RealizationError),
    #[error("remediated expression failure: {0}")]
    Remediated(#[from] RemediatedExpressionError),
    #[error("neutral realization failed: {0}")]
    Neutral(#[from] VerifierReadyRealizationError),
    #[error("conversational microstate contains an out-of-range value")]
    InvalidMicrostate,
    #[error("recent-language trace is malformed or over budget")]
    InvalidLanguageTrace,
    #[error("improvisation lattice is malformed or ambiguous")]
    InvalidLattice,
    #[error("improvisation lattice digest is stale or mismatched")]
    LatticeDigestMismatch,
    #[error("improvisational surface is unsupported or ambiguous")]
    UnsupportedSurface,
    #[error("improvisation output exceeds a frozen budget")]
    BudgetExceeded,
    #[error("improvisation candidate search exceeded its frozen budget")]
    CandidateBudgetExceeded,
    #[error("no independently verified improvisation candidate survived")]
    NoVerifiedCandidate,
    #[error("canonical serialization failed: {0}")]
    CanonicalSerialization(String),
    #[error("canonical digest is zero")]
    EmptyDigest,
}

fn validate_variants(
    program: &SemanticResponseProgram,
    variants: &[ImprovisationalSurfaceVariant],
) -> Result<(), VerifiedImprovisationError> {
    let mut by_operation = BTreeMap::<OperationId, Vec<&ImprovisationalSurfaceVariant>>::new();
    for variant in variants {
        if variant.text.trim() != variant.text
            || variant.text.is_empty()
            || variant.text.contains('\n')
            || variant.phase > 2
        {
            return Err(VerifiedImprovisationError::InvalidLattice);
        }
        by_operation.entry(variant.operation).or_default().push(variant);
    }
    for operation in &program.payload.operations {
        let operation_variants = by_operation
            .get(&operation.id)
            .ok_or(VerifiedImprovisationError::InvalidLattice)?;
        if operation_variants.len() != 6 {
            return Err(VerifiedImprovisationError::InvalidLattice);
        }
        let texts = operation_variants
            .iter()
            .map(|variant| variant.text.as_str())
            .collect::<BTreeSet<_>>();
        if texts.len() != operation_variants.len() {
            return Err(VerifiedImprovisationError::InvalidLattice);
        }
        for left in &texts {
            for right in &texts {
                if left != right && right.starts_with(left) {
                    return Err(VerifiedImprovisationError::InvalidLattice);
                }
            }
        }
    }
    Ok(())
}

fn improvise_surface(
    remediated: &str,
    kind: &DiscourseOperationKind,
    family: ExpressionFamily,
    phase: u8,
    operation_index: usize,
) -> Result<String, VerifiedImprovisationError> {
    let body = strip_remediated_lead(remediated)
        .ok_or(VerifiedImprovisationError::InvalidLattice)?;
    let body = humanize_body(body, kind);
    let prefix = improvisation_prefix(family, phase, operation_index);
    Ok(format!("{prefix}{body}"))
}

fn strip_remediated_lead(text: &str) -> Option<&str> {
    const LEADS: [&str; 6] = [
        "Directly: ",
        "In brief: ",
        "The clearest answer: ",
        "With context preserved: ",
        "A grounded reading: ",
        "The point I would keep: ",
    ];
    LEADS.iter().find_map(|lead| text.strip_prefix(lead))
}

fn improvisation_prefix(
    family: ExpressionFamily,
    phase: u8,
    operation_index: usize,
) -> &'static str {
    let phase = usize::from(phase.min(2));
    if operation_index == 0 {
        match family {
            ExpressionFamily::Direct => ["", "Plainly: ", "The edge is this: "][phase],
            ExpressionFamily::Warm => [
                "With that in view: ",
                "The grounded answer: ",
                "What matters here: ",
            ][phase],
        }
    } else {
        match family {
            ExpressionFamily::Direct => ["Then: ", "Also: ", "Next: "][phase],
            ExpressionFamily::Warm => [
                "At the same time: ",
                "Another piece: ",
                "From there: ",
            ][phase],
        }
    }
}

fn humanize_body(body: &str, kind: &DiscourseOperationKind) -> String {
    match kind {
        DiscourseOperationKind::Acknowledge(_) => body
            .strip_prefix("I acknowledge ")
            .and_then(|rest| rest.strip_suffix('.'))
            .map_or_else(|| body.to_owned(), |rest| format!("Fair point: {rest}.")),
        DiscourseOperationKind::Contrast { .. } => body
            .replacen("On one side, ", "One side: ", 1)
            .replacen(". By contrast, ", ". The other: ", 1),
        DiscourseOperationKind::Correct { .. } => body
            .replacen("Correction: ", "The correction: ", 1)
            .replacen("; instead, ", ". More accurately, ", 1),
        DiscourseOperationKind::Explain { .. } => {
            body.replacen("Relevant support: ", "Why: ", 1)
        }
        DiscourseOperationKind::RequestEvidence(_) => body
            .replacen("What evidence resolves ", "What would settle ", 1)
            .replacen("Evidence is required for ", "This still needs evidence on ", 1),
        DiscourseOperationKind::Commit(_) => {
            body.replacen("I commit to track ", "I'll track ", 1)
        }
        DiscourseOperationKind::Abstain(_) => {
            body.replacen("I abstain because ", "I won't guess because ", 1)
        }
        DiscourseOperationKind::Assert(_) | DiscourseOperationKind::Qualify { .. } => {
            body.to_owned()
        }
    }
}

fn parse_exact<'a>(
    program: &SemanticResponseProgram,
    variants: &'a [ImprovisationalSurfaceVariant],
    text: &str,
) -> Result<Vec<&'a ImprovisationalSurfaceVariant>, VerifiedImprovisationError> {
    let mut cursor = 0usize;
    let mut matched = Vec::with_capacity(program.payload.operations.len());
    for (index, operation) in program.payload.operations.iter().enumerate() {
        let separator = separator_before(program, index);
        if !text[cursor..].starts_with(separator) {
            return Err(VerifiedImprovisationError::UnsupportedSurface);
        }
        cursor += separator.len();
        let remaining = &text[cursor..];
        let next_separator = if index + 1 < program.payload.operations.len() {
            separator_before(program, index + 1)
        } else {
            ""
        };
        let candidates = variants
            .iter()
            .filter(|variant| {
                if variant.operation != operation.id || !remaining.starts_with(&variant.text) {
                    return false;
                }
                let end = variant.text.len();
                if index + 1 == program.payload.operations.len() {
                    end == remaining.len()
                } else {
                    remaining[end..].starts_with(next_separator)
                }
            })
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            return Err(VerifiedImprovisationError::UnsupportedSurface);
        }
        cursor += candidates[0].text.len();
        matched.push(candidates[0]);
    }
    if cursor != text.len() {
        return Err(VerifiedImprovisationError::UnsupportedSurface);
    }
    Ok(matched)
}

fn assemble_remediated(
    program: &SemanticResponseProgram,
    variants: &[&ImprovisationalSurfaceVariant],
) -> String {
    let mut text = String::new();
    for (index, variant) in variants.iter().enumerate() {
        text.push_str(separator_before(program, index));
        text.push_str(&variant.remediated_text);
    }
    text
}

fn recompute_costs(
    program: &SemanticResponseProgram,
    text: &str,
    claim_cost: u32,
    remediated_verification_steps: u32,
) -> Result<ImprovisationCosts, VerifiedImprovisationError> {
    let operation_cost = u32::try_from(program.payload.operations.len())
        .map_err(|_| VerifiedImprovisationError::BudgetExceeded)?;
    let verification_step_cost = remediated_verification_steps
        .checked_add(operation_cost)
        .ok_or(VerifiedImprovisationError::BudgetExceeded)?;
    let character_cost =
        u32::try_from(text.len()).map_err(|_| VerifiedImprovisationError::BudgetExceeded)?;
    let sentence_count = count_sentences(text)?;
    let paragraph_count = u16::try_from(text.split("\n\n").count())
        .map_err(|_| VerifiedImprovisationError::BudgetExceeded)?;
    if operation_cost > u32::from(program.payload.compute_budget.maximum_operations)
        || claim_cost > u32::from(program.payload.compute_budget.maximum_claims)
        || verification_step_cost > program.payload.compute_budget.maximum_verification_steps
        || character_cost > program.payload.output_budget.maximum_characters
        || sentence_count > program.payload.output_budget.maximum_sentences
        || paragraph_count > program.payload.style.maximum_paragraphs
        || paragraph_count == 0
    {
        return Err(VerifiedImprovisationError::BudgetExceeded);
    }
    Ok(ImprovisationCosts {
        operation_cost,
        claim_cost,
        verification_step_cost,
        character_cost,
        sentence_count,
        paragraph_count,
    })
}

fn count_sentences(text: &str) -> Result<u16, VerifiedImprovisationError> {
    let count = text
        .chars()
        .filter(|character| matches!(character, '.' | '?' | '!'))
        .count();
    if count == 0 {
        return Err(VerifiedImprovisationError::BudgetExceeded);
    }
    u16::try_from(count).map_err(|_| VerifiedImprovisationError::BudgetExceeded)
}

fn separator_before(program: &SemanticResponseProgram, index: usize) -> &'static str {
    if index == 0 {
        return "";
    }
    let target_paragraphs = match program.payload.style.detail {
        DetailLevel::Detailed => program
            .payload
            .operations
            .len()
            .min(usize::from(program.payload.style.maximum_paragraphs)),
        DetailLevel::Brief | DetailLevel::Standard => 1,
    }
    .max(1);
    let operations_per_paragraph = program
        .payload
        .operations
        .len()
        .div_ceil(target_paragraphs)
        .max(1);
    if program.payload.style.detail == DetailLevel::Detailed
        && index % operations_per_paragraph == 0
    {
        "\n\n"
    } else {
        " "
    }
}

fn score_variant(
    microstate: &ConversationalMicrostate,
    seed: u64,
    program_digest: u64,
    variant: &ImprovisationalSurfaceVariant,
) -> i64 {
    let family_fit = match variant.family {
        ExpressionFamily::Direct => {
            i64::from(microstate.directness_bps) * 3
                - i64::from(microstate.warmth_bps) / 2
        }
        ExpressionFamily::Warm => {
            i64::from(microstate.warmth_bps) * 3
                - i64::from(microstate.directness_bps) / 3
        }
    };
    let preferred_phase = mixed_phase(seed, program_digest, variant.operation.0);
    let phase_fit = if variant.phase == preferred_phase {
        8_000
    } else {
        2_000 - 1_000 * i64::from(variant.phase.abs_diff(preferred_phase))
    };
    let energy_fit = if variant.phase == 2 {
        i64::from(microstate.energy_bps) / 2
    } else {
        i64::from(10_000_u16.saturating_sub(microstate.energy_bps)) / 5
    };
    let playfulness_fit = if variant.phase == 2 {
        i64::from(microstate.playfulness_bps) / 3
    } else {
        0
    };
    let compression_penalty = i64::try_from(variant.text.len()).unwrap_or(i64::MAX / 4)
        * i64::from(microstate.compression_bps)
        / 1_000;
    family_fit + phase_fit + energy_fit + playfulness_fit - compression_penalty
        - blandness_penalty(&variant.text)
}

fn score_complete_candidate(
    text: &str,
    microstate: &ConversationalMicrostate,
    trace: &RecentLanguageTrace,
    seed: u64,
) -> i64 {
    let opening = opening_fingerprint(text);
    let surface = surface_fingerprint(text);
    let opening_seen = trace.opening_fingerprints.contains(&opening);
    let surface_seen = trace.surface_fingerprints.contains(&surface);
    let novelty_weight = i64::from(microstate.novelty_pressure_bps);
    let novelty_score = if opening_seen {
        -1_000_000 - novelty_weight * 20
    } else {
        25_000 + novelty_weight * 4
    } + if surface_seen {
        -2_000_000 - novelty_weight * 30
    } else {
        40_000 + novelty_weight * 6
    };
    let words = text.split_whitespace().count();
    let punctuation = text
        .chars()
        .filter(|character| matches!(character, ':' | ';' | '?' | '!'))
        .count();
    let rhythm_score = i64::try_from(punctuation).unwrap_or_default() * 900
        + i64::try_from(text.matches(". ").count()).unwrap_or_default() * 300;
    let compression_target = 18usize
        + usize::from(10_000_u16.saturating_sub(microstate.compression_bps)) / 250;
    let compression_distance = words.abs_diff(compression_target);
    let compression_score = -i64::try_from(compression_distance).unwrap_or(i64::MAX / 4) * 250;
    let seed_tiebreak = i64::try_from(mix64(seed ^ surface) % 997).unwrap_or_default();
    novelty_score + rhythm_score + compression_score + seed_tiebreak - blandness_penalty(text)
}

fn blandness_penalty(text: &str) -> i64 {
    const BLAND: [&str; 10] = [
        "It is clear that",
        "The record supports that",
        "There is good reason to think that",
        "With that in view",
        "The grounded answer",
        "At the same time",
        "Another piece",
        "From there",
        "The point I would keep",
        "With context preserved",
    ];
    BLAND
        .iter()
        .filter(|phrase| text.contains(**phrase))
        .count()
        .try_into()
        .map_or(0, |count: i64| count * 2_500)
}

fn mixed_phase(seed: u64, program_digest: u64, operation_id: u64) -> u8 {
    (mix64(
        seed ^ program_digest.rotate_left(17)
            ^ operation_id.wrapping_mul(0x9e37_79b9_7f4a_7c15),
    ) % 3) as u8
}

fn candidate_tie_key(seed: u64, variants: &[RemediatedVariantId]) -> u64 {
    variants.iter().fold(mix64(seed), |state, variant| {
        mix64(state ^ u64::from(variant.0).wrapping_mul(0x9e37_79b9))
    })
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[must_use]
pub fn opening_fingerprint(text: &str) -> u64 {
    let normalized = text
        .split_whitespace()
        .take(6)
        .map(|word| {
            word.chars()
                .filter(|character| character.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ");
    fingerprint(normalized.as_bytes())
}

#[must_use]
pub fn surface_fingerprint(text: &str) -> u64 {
    let normalized = text
        .split_whitespace()
        .map(|word| {
            word.chars()
                .filter(|character| character.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ");
    fingerprint(normalized.as_bytes())
}

fn fingerprint(bytes: &[u8]) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    for byte in FINGERPRINT_DOMAIN.iter().chain(bytes.iter()) {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x1000_0000_01b3);
    }
    if digest == 0 { 1 } else { digest }
}

fn push_bounded(values: &mut Vec<u64>, value: u64) {
    values.push(value);
    if values.len() > MAX_LANGUAGE_TRACE_ITEMS {
        let excess = values.len() - MAX_LANGUAGE_TRACE_ITEMS;
        values.drain(0..excess);
    }
}

fn reject_forbidden_text(
    text: &str,
    forbidden: &[String],
) -> Result<(), VerifiedImprovisationError> {
    let text = text.to_lowercase();
    if forbidden
        .iter()
        .any(|form| text.contains(&form.to_lowercase()))
    {
        return Err(VerifiedImprovisationError::UnsupportedSurface);
    }
    Ok(())
}

fn digest_value<T: Serialize>(
    domain: &[u8],
    value: &T,
) -> Result<u64, VerifiedImprovisationError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| VerifiedImprovisationError::CanonicalSerialization(error.to_string()))?;
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    for byte in domain.iter().chain(bytes.iter()) {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x1000_0000_01b3);
    }
    Ok(digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language_realization::{
        ClaimLexicalBinding, LexicalBindingTablePayload, ObservationLexicalBinding,
    };
    use crate::semantic_response::{
        AcknowledgmentLevel, AuthorizedClaim, ClaimId, ClaimPolarity, CognitiveStateVersion,
        ComputeBudget, DialogueMode, DiscourseOperation, EpistemicConstraint, EpistemicStatus,
        ObservationId, OperationId, OutputBudget, ProhibitedClaim, ResponseProgramId,
        SemanticResponseIntent, SemanticResponseProgramPayload, SemanticValidationContext,
        SensitivityLevel, SensitivityPolicy, StyleEnvelope, SubjectScope, VocabularyLevel,
    };

    fn claim(id: u64, key: &str, status: EpistemicStatus) -> AuthorizedClaim {
        AuthorizedClaim {
            id: ClaimId(id),
            semantic_key: key.to_owned(),
            polarity: ClaimPolarity::Positive,
            confidence_bps: 8_000,
            epistemic_status: status,
            sensitivity: SensitivityLevel::Public,
            disclosure_scope: SubjectScope(77),
        }
    }

    fn fixture() -> (SemanticResponseProgram, LexicalBindingTable) {
        let required_claims = vec![
            claim(1, "templates_are_too_rigid", EpistemicStatus::Certain),
            claim(2, "meaning_must_remain_fixed", EpistemicStatus::Probable),
            claim(3, "expression_may_vary", EpistemicStatus::Probable),
        ];
        let epistemic_constraints = required_claims
            .iter()
            .map(|claim| {
                let (minimum, maximum) = claim.epistemic_status.confidence_bounds();
                EpistemicConstraint {
                    claim: claim.id,
                    required_status: claim.epistemic_status,
                    minimum_confidence_bps: minimum,
                    maximum_confidence_bps: maximum,
                }
            })
            .collect();
        let program = SemanticResponseProgram::validate(
            SemanticResponseProgramPayload {
                id: ResponseProgramId(900),
                source_state_version: CognitiveStateVersion(41),
                companion_state_version: None,
                subject_scope: SubjectScope(77),
                intent: SemanticResponseIntent::Contrast,
                operations: vec![
                    DiscourseOperation {
                        id: OperationId(1),
                        kind: DiscourseOperationKind::Acknowledge(ObservationId(901)),
                    },
                    DiscourseOperation {
                        id: OperationId(2),
                        kind: DiscourseOperationKind::Assert(ClaimId(1)),
                    },
                    DiscourseOperation {
                        id: OperationId(3),
                        kind: DiscourseOperationKind::Contrast {
                            left: ClaimId(2),
                            right: ClaimId(3),
                        },
                    },
                ],
                required_claims,
                optional_claims: Vec::new(),
                prohibited_claims: vec![ProhibitedClaim {
                    id: ClaimId(100),
                    semantic_key: "fluency_proves_cognition".to_owned(),
                }],
                epistemic_constraints,
                sensitivity: SensitivityPolicy {
                    maximum_disclosure: SensitivityLevel::Public,
                    disclosure_scope: SubjectScope(77),
                },
                style: StyleEnvelope {
                    detail: DetailLevel::Standard,
                    vocabulary: VocabularyLevel::Technical,
                    dialogue: DialogueMode::Collaborative,
                    acknowledgment: AcknowledgmentLevel::Brief,
                    allow_first_person: true,
                    allow_questions: true,
                    maximum_paragraphs: 2,
                },
                output_budget: OutputBudget {
                    maximum_characters: 5_000,
                    maximum_sentences: 24,
                },
                compute_budget: ComputeBudget {
                    maximum_operations: 16,
                    maximum_claims: 32,
                    maximum_verification_steps: 256,
                },
            },
            SemanticValidationContext {
                cognitive_state_version: CognitiveStateVersion(41),
                companion_state_version: None,
                subject_scope: SubjectScope(77),
            },
        )
        .expect("fixture semantic program must validate");
        let table = LexicalBindingTable::validate(
            LexicalBindingTablePayload {
                program_digest: program.digest,
                subject_scope: SubjectScope(77),
                claims: vec![
                    ClaimLexicalBinding {
                        claim: ClaimId(1),
                        positive_clause: "the current templates are too rigid".to_owned(),
                        negative_clause: "the current templates are not too rigid".to_owned(),
                    },
                    ClaimLexicalBinding {
                        claim: ClaimId(2),
                        positive_clause: "meaning must remain fixed".to_owned(),
                        negative_clause: "meaning must not remain fixed".to_owned(),
                    },
                    ClaimLexicalBinding {
                        claim: ClaimId(3),
                        positive_clause: "expression may vary".to_owned(),
                        negative_clause: "expression may not vary".to_owned(),
                    },
                ],
                observations: vec![ObservationLexicalBinding {
                    observation: ObservationId(901),
                    label: "the fluency objection".to_owned(),
                }],
                missing_variables: Vec::new(),
                predictions: Vec::new(),
                forbidden_surface_forms: vec!["fluency proves cognition".to_owned()],
            },
            &program,
        )
        .expect("fixture lexical table must validate");
        (program, table)
    }

    #[test]
    fn selector_replays_and_varies_across_seeds() {
        let (program, table) = fixture();
        let selector = VerifiedImprovisationSelector;
        let mut texts = BTreeSet::new();
        for seed in 0..18 {
            let request = ImprovisationRequest::new(
                seed,
                ConversationalMicrostate::default(),
                RecentLanguageTrace::default(),
            )
            .expect("request must validate");
            let selected = selector
                .select(&program, &table, &request)
                .expect("selection must complete");
            assert_eq!(
                selected.payload.disposition,
                ImprovisationDisposition::VerifiedImprovisation
            );
            assert!(selected.payload.verification_digest.is_some());
            texts.insert(selected.payload.text);
        }
        assert!(texts.len() >= 3);

        let request = ImprovisationRequest::new(
            7,
            ConversationalMicrostate::default(),
            RecentLanguageTrace::default(),
        )
        .expect("request must validate");
        let first = selector
            .select(&program, &table, &request)
            .expect("first replay must complete");
        let second = selector
            .select(&program, &table, &request)
            .expect("second replay must complete");
        assert_eq!(first, second);
    }

    #[test]
    fn recent_language_forces_a_different_opening() {
        let (program, table) = fixture();
        let selector = VerifiedImprovisationSelector;
        let request = ImprovisationRequest::new(
            19,
            ConversationalMicrostate::default(),
            RecentLanguageTrace::default(),
        )
        .expect("request must validate");
        let first = selector
            .select(&program, &table, &request)
            .expect("first selection must complete");
        let mut trace = RecentLanguageTrace::default();
        trace
            .record_text(&first.payload.text)
            .expect("trace update must validate");
        let repeated_request = ImprovisationRequest::new(
            19,
            ConversationalMicrostate::default(),
            trace,
        )
        .expect("repeated request must validate");
        let second = selector
            .select(&program, &table, &repeated_request)
            .expect("second selection must complete");
        assert_ne!(
            first.payload.opening_fingerprint,
            second.payload.opening_fingerprint
        );
    }

    #[test]
    fn authority_boundary_remains_closed() {
        let boundary = authority_boundary();
        assert!(boundary.committed_surface_lattice);
        assert!(boundary.conversational_microstate_scoring);
        assert!(boundary.replayable_entropy_seed);
        assert!(boundary.recent_language_anti_repetition);
        assert!(boundary.independent_candidate_verification);
        assert!(!boundary.runtime_chat_wiring);
        assert!(!boundary.http_response_influence);
        assert!(!boundary.live_generated_text_influence);
        assert!(!boundary.raw_prompt_access);
        assert!(!boundary.unrestricted_memory_access);
        assert!(!boundary.persistence_authority);
        assert!(!boundary.tool_selection_authority);
        assert!(!boundary.autonomous_action_authority);
    }
}
