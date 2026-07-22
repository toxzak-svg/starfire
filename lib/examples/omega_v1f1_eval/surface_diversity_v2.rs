//! ΩV1-F1R1 claim-first nested-verification layer.
//!
//! Final text reconstructs the bounded R1 surface, which reconstructs the
//! original grammar-v3 surface. The model chooses only direct versus warm.

use crate::language_realization::{LexicalBindingTable, LexicalTableDigest};
use crate::learned_expression::{
    LearnedExpressionError, LearnedExpressionModel, LearnedExpressionModelDigest,
    SelectionDisposition, VariantProfile,
};
use crate::omega_v1f1_projection_guard::VerifiedVoiceProjection;
use crate::semantic_response::{
    ClaimId, DetailLevel, DiscourseOperationKind, OperationId, ResponseProgramDigest,
    SemanticResponseProgram,
};
use crate::surface_diversity::{
    self, ExpressionFamily, RemediatedExpressionError, RemediatedLattice, RemediatedLatticeDigest,
    RemediatedOfflineSelector, RemediatedVariantId, RemediatedVerificationDigest,
    RemediatedVerifier,
};
use crate::verifier_ready_realization::{VerifierReadyRenderer, VERIFIER_READY_GRAMMAR_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const CLAIM_FIRST_GRAMMAR_VERSION: u16 = 5;
const LATTICE_DOMAIN: &[u8] = b"starfire-omega-v1f1r1-claim-first-lattice-v4";
const VERIFY_DOMAIN: &[u8] = b"starfire-omega-v1f1r1-claim-first-verification-v4";
const SELECT_DOMAIN: &[u8] = b"starfire-omega-v1f1r1-claim-first-selection-v4";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstLatticeDigest(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstVerificationDigest(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstSelectionDigest(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstSurfaceVariant {
    pub operation: OperationId,
    pub variant_id: RemediatedVariantId,
    pub kind: DiscourseOperationKind,
    pub claim_ids: Vec<ClaimId>,
    pub profile: VariantProfile,
    pub family: ExpressionFamily,
    pub phase: u8,
    pub base_text: String,
    pub text: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstLatticePayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub base_lattice_digest: RemediatedLatticeDigest,
    pub grammar_version: u16,
    pub variants: Vec<ClaimFirstSurfaceVariant>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstLattice {
    pub payload: ClaimFirstLatticePayload,
    pub digest: ClaimFirstLatticeDigest,
}

impl ClaimFirstLattice {
    pub fn build(
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Result<Self, ClaimFirstError> {
        let base = RemediatedLattice::build(program, lexical_table)?;
        let variants = base
            .payload
            .variants
            .iter()
            .map(|variant| ClaimFirstSurfaceVariant {
                operation: variant.operation,
                variant_id: variant.variant_id,
                kind: variant.kind.clone(),
                claim_ids: variant.claim_ids.clone(),
                profile: variant.profile,
                family: variant.family,
                phase: variant.phase,
                base_text: variant.text.clone(),
                text: claim_first_text(
                    &variant.canonical_text,
                    &variant.kind,
                    variant.family,
                    variant.phase,
                ),
            })
            .collect::<Vec<_>>();
        validate_variants(program, &variants)?;
        let payload = ClaimFirstLatticePayload {
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            base_lattice_digest: base.digest,
            grammar_version: CLAIM_FIRST_GRAMMAR_VERSION,
            variants,
        };
        let digest = ClaimFirstLatticeDigest(digest_value(LATTICE_DOMAIN, &payload)?);
        if digest.0 == 0 {
            return Err(ClaimFirstError::EmptyDigest);
        }
        Ok(Self { payload, digest })
    }

    pub fn verify_integrity(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Result<(), ClaimFirstError> {
        if self != &Self::build(program, lexical_table)? {
            return Err(ClaimFirstError::LatticeDigestMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstVerifiedVariant {
    pub operation: OperationId,
    pub variant_id: RemediatedVariantId,
    pub kind: DiscourseOperationKind,
    pub family: ExpressionFamily,
    pub phase: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstCosts {
    pub operation_cost: u32,
    pub claim_cost: u32,
    pub verification_step_cost: u32,
    pub character_cost: u32,
    pub sentence_count: u16,
    pub paragraph_count: u16,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstVerificationPayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub lattice_digest: ClaimFirstLatticeDigest,
    pub base_lattice_digest: RemediatedLatticeDigest,
    pub base_verification_digest: RemediatedVerificationDigest,
    pub grammar_version: u16,
    pub variants: Vec<ClaimFirstVerifiedVariant>,
    pub costs: ClaimFirstCosts,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstVerificationReport {
    pub payload: ClaimFirstVerificationPayload,
    pub digest: ClaimFirstVerificationDigest,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct ClaimFirstVerifier;

impl ClaimFirstVerifier {
    pub fn verify(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        lattice_digest: ClaimFirstLatticeDigest,
        text: &str,
    ) -> Result<ClaimFirstVerificationReport, ClaimFirstError> {
        if text.is_empty() {
            return Err(ClaimFirstError::UnsupportedSurface);
        }
        let lattice = ClaimFirstLattice::build(program, lexical_table)?;
        if lattice.digest != lattice_digest {
            return Err(ClaimFirstError::LatticeDigestMismatch);
        }
        let matched = parse_exact(program, &lattice.payload.variants, text)?;
        let base_report = RemediatedVerifier.verify(
            program,
            lexical_table,
            lattice.payload.base_lattice_digest,
            &assemble_base(program, &matched),
        )?;
        let payload = ClaimFirstVerificationPayload {
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            lattice_digest: lattice.digest,
            base_lattice_digest: lattice.payload.base_lattice_digest,
            base_verification_digest: base_report.digest,
            grammar_version: CLAIM_FIRST_GRAMMAR_VERSION,
            variants: matched
                .iter()
                .map(|variant| ClaimFirstVerifiedVariant {
                    operation: variant.operation,
                    variant_id: variant.variant_id,
                    kind: variant.kind.clone(),
                    family: variant.family,
                    phase: variant.phase,
                })
                .collect(),
            costs: recompute_costs(
                program,
                text,
                &matched,
                base_report.payload.costs.claim_cost,
            )?,
        };
        let digest = ClaimFirstVerificationDigest(digest_value(VERIFY_DOMAIN, &payload)?);
        if digest.0 == 0 {
            return Err(ClaimFirstError::EmptyDigest);
        }
        Ok(ClaimFirstVerificationReport { payload, digest })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstSelectionPayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub voice_projection_digest: String,
    pub model_digest: LearnedExpressionModelDigest,
    pub lattice_digest: Option<ClaimFirstLatticeDigest>,
    pub selected_grammar_version: u16,
    pub disposition: SelectionDisposition,
    pub text: String,
    pub variant_ids: Vec<RemediatedVariantId>,
    pub family: Option<ExpressionFamily>,
    pub score: i64,
    pub complete_candidates_scored: u16,
    pub verification_digest: Option<ClaimFirstVerificationDigest>,
    pub fallback_reason: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstSelectionResult {
    pub payload: ClaimFirstSelectionPayload,
    pub digest: ClaimFirstSelectionDigest,
}
#[derive(Debug, Clone)]
pub struct ClaimFirstOfflineSelector {
    model: LearnedExpressionModel,
}

impl ClaimFirstOfflineSelector {
    #[must_use]
    pub fn new(model: LearnedExpressionModel) -> Self {
        Self { model }
    }

    pub fn select(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        projection: &VerifiedVoiceProjection,
    ) -> Result<ClaimFirstSelectionResult, ClaimFirstError> {
        program.verify_replay_integrity()?;
        lexical_table.verify_integrity(program)?;
        let neutral = VerifierReadyRenderer.render(program, lexical_table)?;
        let payload = match self.try_select(program, lexical_table, projection) {
            Ok(payload) => payload,
            Err(error) => ClaimFirstSelectionPayload {
                program_digest: program.digest,
                lexical_table_digest: lexical_table.digest,
                voice_projection_digest: projection.source_digest.clone(),
                model_digest: self.model.digest,
                lattice_digest: None,
                selected_grammar_version: VERIFIER_READY_GRAMMAR_VERSION,
                disposition: SelectionDisposition::NeutralFallback,
                text: neutral.payload.text,
                variant_ids: Vec::new(),
                family: None,
                score: 0,
                complete_candidates_scored: 0,
                verification_digest: None,
                fallback_reason: Some(error.to_string()),
            },
        };
        let digest = ClaimFirstSelectionDigest(digest_value(SELECT_DOMAIN, &payload)?);
        if digest.0 == 0 {
            return Err(ClaimFirstError::EmptyDigest);
        }
        Ok(ClaimFirstSelectionResult { payload, digest })
    }

    fn try_select(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        projection: &VerifiedVoiceProjection,
    ) -> Result<ClaimFirstSelectionPayload, ClaimFirstError> {
        self.model.verify_integrity()?;
        projection.verify_integrity()?;
        let base = RemediatedOfflineSelector::new(self.model.clone()).select(
            program,
            lexical_table,
            projection,
        )?;
        if base.payload.disposition != SelectionDisposition::LearnedVerified
            || base.payload.variant_ids.len() != program.payload.operations.len()
        {
            return Err(ClaimFirstError::BaseSelectionFallback);
        }
        let lattice = ClaimFirstLattice::build(program, lexical_table)?;
        let mut text = String::new();
        for (index, (operation, variant_id)) in program
            .payload
            .operations
            .iter()
            .zip(&base.payload.variant_ids)
            .enumerate()
        {
            text.push_str(separator_before(program, index));
            let variant = lattice
                .payload
                .variants
                .iter()
                .find(|variant| {
                    variant.operation == operation.id && variant.variant_id == *variant_id
                })
                .ok_or(ClaimFirstError::MissingVariant)?;
            text.push_str(&variant.text);
        }
        let report = ClaimFirstVerifier.verify(program, lexical_table, lattice.digest, &text)?;
        Ok(ClaimFirstSelectionPayload {
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            voice_projection_digest: projection.source_digest.clone(),
            model_digest: self.model.digest,
            lattice_digest: Some(lattice.digest),
            selected_grammar_version: CLAIM_FIRST_GRAMMAR_VERSION,
            disposition: SelectionDisposition::LearnedVerified,
            text,
            variant_ids: base.payload.variant_ids,
            family: base.payload.family,
            score: base.payload.score,
            complete_candidates_scored: base.payload.complete_candidates_scored,
            verification_digest: Some(report.digest),
            fallback_reason: None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimFirstAuthorityBoundary {
    pub candidate_lattice_construction: bool,
    pub learned_candidate_scoring: bool,
    pub deterministic_candidate_local_diversity: bool,
    pub nested_independent_candidate_verification: bool,
    pub independent_candidate_verification: bool,
    pub runtime_chat_wiring: bool,
    pub http_response_influence: bool,
    pub live_generated_text_influence: bool,
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
pub const fn authority_boundary() -> ClaimFirstAuthorityBoundary {
    ClaimFirstAuthorityBoundary {
        candidate_lattice_construction: true,
        learned_candidate_scoring: true,
        deterministic_candidate_local_diversity: true,
        nested_independent_candidate_verification: true,
        independent_candidate_verification: true,
        runtime_chat_wiring: false,
        http_response_influence: false,
        live_generated_text_influence: false,
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

#[derive(Debug, Error)]
pub enum ClaimFirstError {
    #[error("base R1 surface failure: {0}")]
    BaseSurface(#[from] RemediatedExpressionError),
    #[error("base learned-expression failure: {0}")]
    BaseLearned(#[from] LearnedExpressionError),
    #[error("projection packet is stale or corrupt: {0}")]
    Projection(#[from] crate::omega_v1f1_projection_guard::ProjectionPacketError),
    #[error("semantic program validation failed: {0}")]
    Semantic(#[from] crate::semantic_response::SemanticProgramError),
    #[error("lexical table validation failed: {0}")]
    Lexical(#[from] crate::language_realization::RealizationError),
    #[error("neutral realization failed: {0}")]
    Neutral(#[from] crate::verifier_ready_realization::VerifierReadyRealizationError),
    #[error("base selector returned neutral fallback")]
    BaseSelectionFallback,
    #[error("claim-first lattice is malformed or ambiguous")]
    InvalidLattice,
    #[error("claim-first lattice digest is stale or mismatched")]
    LatticeDigestMismatch,
    #[error("claim-first surface is unsupported or ambiguous")]
    UnsupportedSurface,
    #[error("required claim-first candidate is missing")]
    MissingVariant,
    #[error("claim-first output exceeds a frozen budget")]
    BudgetExceeded,
    #[error("canonical serialization failed: {0}")]
    CanonicalSerialization(String),
    #[error("canonical digest is zero")]
    EmptyDigest,
}

fn validate_variants(
    program: &SemanticResponseProgram,
    variants: &[ClaimFirstSurfaceVariant],
) -> Result<(), ClaimFirstError> {
    let mut ids = BTreeMap::<OperationId, BTreeSet<RemediatedVariantId>>::new();
    let mut surfaces = BTreeSet::new();
    for variant in variants {
        if variant.text.is_empty()
            || variant.text.trim() != variant.text
            || variant.text.contains('\n')
            || !ids
                .entry(variant.operation)
                .or_default()
                .insert(variant.variant_id)
            || !surfaces.insert(variant.text.clone())
        {
            return Err(ClaimFirstError::InvalidLattice);
        }
    }
    if program.payload.operations.iter().any(|operation| {
        ids.get(&operation.id)
            .map(BTreeSet::len)
            .unwrap_or_default()
            != surface_diversity::REMEDIATED_VARIANTS_PER_OPERATION
    }) {
        return Err(ClaimFirstError::InvalidLattice);
    }
    let ordered = surfaces.iter().collect::<Vec<_>>();
    for (index, left) in ordered.iter().enumerate() {
        for right in ordered.iter().skip(index + 1) {
            if left.starts_with(right.as_str()) || right.starts_with(left.as_str()) {
                return Err(ClaimFirstError::InvalidLattice);
            }
        }
    }
    Ok(())
}

fn claim_first_text(
    canonical: &str,
    kind: &DiscourseOperationKind,
    family: ExpressionFamily,
    phase: u8,
) -> String {
    match kind {
        DiscourseOperationKind::Assert(_) => claim_first_epistemic(canonical, family, phase)
            .unwrap_or_else(|| generic_phase_text(canonical, family, phase)),
        DiscourseOperationKind::Qualify { .. } => {
            let base = claim_first_epistemic(canonical, family, phase)
                .unwrap_or_else(|| generic_phase_text(canonical, family, phase));
            format!("Qualified: {}", base)
        }
        DiscourseOperationKind::Acknowledge(_) => {
            let label = extract_between(
                canonical,
                &["I register ", "Acknowledged: ", "I acknowledge "],
                &['.'],
            )
            .unwrap_or(canonical);
            format!(
                "{}: {}",
                label,
                choose(
                    family,
                    phase,
                    [
                        "registered directly.",
                        "noted without ornament.",
                        "acknowledged plainly.",
                    ],
                    [
                        "is in view.",
                        "is acknowledged with context.",
                        "has my attention.",
                    ],
                )
            )
        }
        DiscourseOperationKind::RequestEvidence(_) => {
            let label = extract_between(
                canonical,
                &[
                    "Which evidence would resolve ",
                    "What would settle the evidence question around ",
                    "What evidence resolves ",
                    "Evidence is required for ",
                ],
                &['?', '.'],
            )
            .unwrap_or(canonical);
            format!(
                "{}: {}",
                label,
                choose(
                    family,
                    phase,
                    [
                        "what evidence resolves it?",
                        "which fact would settle it?",
                        "what observation closes the gap?",
                    ],
                    [
                        "what would make the answer clear?",
                        "which evidence would give us confidence?",
                        "what would let us resolve this carefully?",
                    ],
                )
            )
        }
        DiscourseOperationKind::Commit(_) => {
            let label = extract_between(
                canonical,
                &[
                    "I will track ",
                    "Tracking commitment: ",
                    "I commit to track ",
                ],
                &['.'],
            )
            .unwrap_or(canonical);
            format!(
                "{}: {}",
                label,
                choose(
                    family,
                    phase,
                    [
                        "I will track it.",
                        "it remains on the ledger.",
                        "the commitment is explicit.",
                    ],
                    [
                        "I will keep it in view.",
                        "I will carry that forward.",
                        "it stays with the next check.",
                    ],
                )
            )
        }
        _ => generic_phase_text(canonical, family, phase),
    }
}

fn generic_phase_text(canonical: &str, family: ExpressionFamily, phase: u8) -> String {
    format!(
        "{} {}",
        canonical,
        choose(
            family,
            phase,
            [
                "The relation is explicit.",
                "The operation remains bounded.",
                "The typed structure is preserved.",
            ],
            [
                "The relation stays visible.",
                "The boundary remains intact.",
                "The structure is carried through carefully.",
            ],
        )
    )
}

fn claim_first_epistemic(canonical: &str, family: ExpressionFamily, phase: u8) -> Option<String> {
    for (marker, band) in [
        ("I know that ", EpistemicBand::Certain),
        ("It is probable that ", EpistemicBand::Probable),
        ("It is possible that ", EpistemicBand::Possible),
        ("I am uncertain whether ", EpistemicBand::Uncertain),
        ("I do not know whether ", EpistemicBand::Unknown),
    ] {
        if let Some(position) = canonical.find(marker) {
            let claim = canonical[position + marker.len()..]
                .trim_end_matches(|character| matches!(character, '.' | '?' | '!'));
            return Some(format!(
                "{}. {}",
                capitalize_first(claim),
                epistemic_ending(band, family, phase)
            ));
        }
    }
    None
}

#[derive(Debug, Clone, Copy)]
enum EpistemicBand {
    Certain,
    Probable,
    Possible,
    Uncertain,
    Unknown,
}

fn epistemic_ending(band: EpistemicBand, family: ExpressionFamily, phase: u8) -> &'static str {
    match band {
        EpistemicBand::Certain => choose(
            family,
            phase,
            [
                "The evidence establishes it.",
                "That conclusion is supported.",
                "The authorized confidence is certain.",
            ],
            [
                "The record supports saying so clearly.",
                "We can hold that conclusion with confidence.",
                "That is established without embellishment.",
            ],
        ),
        EpistemicBand::Probable => choose(
            family,
            phase,
            [
                "That is the probable reading.",
                "The evidence points there most strongly.",
                "Probability favors that conclusion.",
            ],
            [
                "That is the likeliest reading.",
                "There is good reason to lean that way.",
                "I would keep it at probable, not certain.",
            ],
        ),
        EpistemicBand::Possible => choose(
            family,
            phase,
            [
                "That remains a possibility.",
                "The evidence permits it, but no more.",
                "Possible is the authorized limit.",
            ],
            [
                "One possibility, held carefully.",
                "There is room for that reading.",
                "It may be so, without overstating it.",
            ],
        ),
        EpistemicBand::Uncertain => choose(
            family,
            phase,
            [
                "That remains uncertain.",
                "The evidence does not yet resolve it.",
                "Uncertainty is the authorized status.",
            ],
            [
                "I would keep the uncertainty visible.",
                "There is not enough clarity to settle it.",
                "The open question should remain open.",
            ],
        ),
        EpistemicBand::Unknown => choose(
            family,
            phase,
            [
                "Whether it holds is unknown.",
                "The evidence cannot determine it.",
                "Unknown is the authorized status.",
            ],
            [
                "There is no sound basis to decide it yet.",
                "The answer should remain unknown for now.",
                "I would not pretend the record resolves it.",
            ],
        ),
    }
}

fn choose(
    family: ExpressionFamily,
    phase: u8,
    direct: [&'static str; 3],
    warm: [&'static str; 3],
) -> &'static str {
    match family {
        ExpressionFamily::Direct => direct[usize::from(phase)],
        ExpressionFamily::Warm => warm[usize::from(phase)],
    }
}

fn extract_between<'a>(text: &'a str, prefixes: &[&str], terminal: &[char]) -> Option<&'a str> {
    for prefix in prefixes {
        if let Some(position) = text.find(prefix) {
            return Some(
                text[position + prefix.len()..]
                    .trim_end_matches(|character| terminal.contains(&character)),
            );
        }
    }
    None
}

fn capitalize_first(text: &str) -> String {
    let mut characters = text.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => String::new(),
    }
}

fn parse_exact<'a>(
    program: &SemanticResponseProgram,
    variants: &'a [ClaimFirstSurfaceVariant],
    text: &str,
) -> Result<Vec<&'a ClaimFirstSurfaceVariant>, ClaimFirstError> {
    let mut cursor = 0usize;
    let mut matched = Vec::with_capacity(program.payload.operations.len());
    for (index, operation) in program.payload.operations.iter().enumerate() {
        let separator = separator_before(program, index);
        if !text[cursor..].starts_with(separator) {
            return Err(ClaimFirstError::UnsupportedSurface);
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
            return Err(ClaimFirstError::UnsupportedSurface);
        }
        cursor += candidates[0].text.len();
        matched.push(candidates[0]);
    }
    if cursor != text.len() {
        return Err(ClaimFirstError::UnsupportedSurface);
    }
    Ok(matched)
}

fn assemble_base(
    program: &SemanticResponseProgram,
    variants: &[&ClaimFirstSurfaceVariant],
) -> String {
    let mut text = String::new();
    for (index, variant) in variants.iter().enumerate() {
        text.push_str(separator_before(program, index));
        text.push_str(&variant.base_text);
    }
    text
}

fn recompute_costs(
    program: &SemanticResponseProgram,
    text: &str,
    variants: &[&ClaimFirstSurfaceVariant],
    claim_cost: u32,
) -> Result<ClaimFirstCosts, ClaimFirstError> {
    let operation_cost =
        u32::try_from(variants.len()).map_err(|_| ClaimFirstError::BudgetExceeded)?;
    let verification_step_cost = operation_cost
        .checked_add(claim_cost)
        .and_then(|cost| cost.checked_add(operation_cost))
        .and_then(|cost| cost.checked_add(operation_cost))
        .and_then(|cost| cost.checked_add(operation_cost))
        .ok_or(ClaimFirstError::BudgetExceeded)?;
    let character_cost = u32::try_from(text.len()).map_err(|_| ClaimFirstError::BudgetExceeded)?;
    let sentence_count = count_sentences(text)?;
    let paragraph_count =
        u16::try_from(text.split("\n\n").count()).map_err(|_| ClaimFirstError::BudgetExceeded)?;
    if operation_cost > u32::from(program.payload.compute_budget.maximum_operations)
        || claim_cost > u32::from(program.payload.compute_budget.maximum_claims)
        || verification_step_cost > program.payload.compute_budget.maximum_verification_steps
        || character_cost > program.payload.output_budget.maximum_characters
        || sentence_count > program.payload.output_budget.maximum_sentences
        || paragraph_count > program.payload.style.maximum_paragraphs
        || paragraph_count == 0
    {
        return Err(ClaimFirstError::BudgetExceeded);
    }
    Ok(ClaimFirstCosts {
        operation_cost,
        claim_cost,
        verification_step_cost,
        character_cost,
        sentence_count,
        paragraph_count,
    })
}

fn count_sentences(text: &str) -> Result<u16, ClaimFirstError> {
    let count = text
        .chars()
        .filter(|character| matches!(character, '.' | '?' | '!'))
        .count();
    if count == 0 {
        return Err(ClaimFirstError::BudgetExceeded);
    }
    u16::try_from(count).map_err(|_| ClaimFirstError::BudgetExceeded)
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

fn digest_value<T: Serialize>(domain: &[u8], value: &T) -> Result<u64, ClaimFirstError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| ClaimFirstError::CanonicalSerialization(error.to_string()))?;
    let mut digest = 0xcbf29ce484222325_u64;
    for byte in domain.iter().chain(bytes.iter()) {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    Ok(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_and_qualify_are_not_prefixes() {
        let canonical = "Conclusion: It is possible that the selector stays bounded.";
        let asserted = claim_first_text(
            canonical,
            &DiscourseOperationKind::Assert(ClaimId(1)),
            ExpressionFamily::Direct,
            0,
        );
        let qualified = claim_first_text(
            canonical,
            &DiscourseOperationKind::Qualify {
                claim: ClaimId(1),
                status: crate::semantic_response::EpistemicStatus::Possible,
            },
            ExpressionFamily::Direct,
            0,
        );
        assert_ne!(asserted, qualified);
        assert!(!asserted.starts_with(&qualified));
        assert!(!qualified.starts_with(&asserted));
    }
}
