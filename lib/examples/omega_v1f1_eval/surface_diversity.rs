#![allow(dead_code, clippy::type_complexity)]
//! ΩV1-F1R1 bounded surface-family remediation.
//!
//! This layer keeps the F1 learned direct-vs-warm ranker, but expands each
//! operation into six committed, independently verifiable surfaces: three
//! direct and three warm. A sealed VoiceState projection chooses the learned
//! family; a deterministic candidate-local phase chooses within that family.
//! No raw prompt, conversation, memory, mutation, routing, tool, CHARGE, HTTP,
//! or autonomous-action authority is introduced.

use crate::language_realization::{LexicalBindingTable, LexicalTableDigest};
use crate::learned_expression::{
    ExpressionLattice, ExpressionLatticeDigest, GrammarV3VerificationDigest, GrammarV3Verifier,
    LearnedExpressionError, LearnedExpressionModel, LearnedExpressionModelDigest,
    SelectionDisposition, SurfaceVariantId, VariantProfile, MAX_VARIANTS_PER_OPERATION,
};
use crate::omega_v1f1_projection_guard::VerifiedVoiceProjection;
use crate::semantic_response::{
    ClaimId, DetailLevel, DiscourseOperationKind, OperationId, ResponseProgramDigest,
    SemanticResponseProgram,
};
use crate::verifier_ready_realization::{VerifierReadyRenderer, VERIFIER_READY_GRAMMAR_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const REMEDIATED_GRAMMAR_VERSION: u16 = 4;
pub const REMEDIATED_VARIANTS_PER_OPERATION: usize = 6;
const LATTICE_DOMAIN: &[u8] = b"starfire-omega-v1f1r1-surface-lattice-v1";
const VERIFY_DOMAIN: &[u8] = b"starfire-omega-v1f1r1-verification-v1";
const SELECT_DOMAIN: &[u8] = b"starfire-omega-v1f1r1-selection-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RemediatedVariantId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedLatticeDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedVerificationDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedSelectionDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpressionFamily {
    Direct,
    Warm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedSurfaceVariant {
    pub operation: OperationId,
    pub variant_id: RemediatedVariantId,
    pub original_variant_id: SurfaceVariantId,
    pub kind: DiscourseOperationKind,
    pub claim_ids: Vec<ClaimId>,
    pub profile: VariantProfile,
    pub family: ExpressionFamily,
    pub phase: u8,
    pub canonical_text: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedLatticePayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub original_lattice_digest: ExpressionLatticeDigest,
    pub grammar_version: u16,
    pub variants: Vec<RemediatedSurfaceVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedLattice {
    pub payload: RemediatedLatticePayload,
    pub digest: RemediatedLatticeDigest,
}

impl RemediatedLattice {
    pub fn build(
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Result<Self, RemediatedExpressionError> {
        let original = ExpressionLattice::build(program, lexical_table)?;
        let mut variants = Vec::new();
        for operation in &program.payload.operations {
            let operation_variants = original
                .payload
                .variants
                .iter()
                .filter(|variant| variant.operation == operation.id)
                .collect::<Vec<_>>();
            let direct = unique_profile_variant(&operation_variants, VariantProfile::direct())?;
            let warm = unique_profile_variant(&operation_variants, VariantProfile::warm())?;
            for (family, base, offset) in [
                (ExpressionFamily::Direct, direct, 0_u16),
                (ExpressionFamily::Warm, warm, 3_u16),
            ] {
                for phase in 0_u8..3 {
                    let text = diversify_text(&base.text, family, phase);
                    variants.push(RemediatedSurfaceVariant {
                        operation: operation.id,
                        variant_id: RemediatedVariantId(offset + u16::from(phase)),
                        original_variant_id: base.variant_id,
                        kind: operation.kind.clone(),
                        claim_ids: base.claim_ids.clone(),
                        profile: base.profile,
                        family,
                        phase,
                        canonical_text: base.text.clone(),
                        text,
                    });
                }
            }
        }
        validate_variants(program, &variants)?;
        let payload = RemediatedLatticePayload {
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            original_lattice_digest: original.digest,
            grammar_version: REMEDIATED_GRAMMAR_VERSION,
            variants,
        };
        let digest = RemediatedLatticeDigest(digest_value(LATTICE_DOMAIN, &payload)?);
        if digest.0 == 0 {
            return Err(RemediatedExpressionError::EmptyDigest);
        }
        Ok(Self { payload, digest })
    }

    pub fn verify_integrity(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Result<(), RemediatedExpressionError> {
        let rebuilt = Self::build(program, lexical_table)?;
        if &rebuilt != self {
            return Err(RemediatedExpressionError::LatticeDigestMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedVerifiedVariant {
    pub operation: OperationId,
    pub variant_id: RemediatedVariantId,
    pub original_variant_id: SurfaceVariantId,
    pub kind: DiscourseOperationKind,
    pub family: ExpressionFamily,
    pub phase: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedCosts {
    pub operation_cost: u32,
    pub claim_cost: u32,
    pub verification_step_cost: u32,
    pub character_cost: u32,
    pub sentence_count: u16,
    pub paragraph_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedVerificationPayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub lattice_digest: RemediatedLatticeDigest,
    pub original_lattice_digest: ExpressionLatticeDigest,
    pub original_verification_digest: GrammarV3VerificationDigest,
    pub grammar_version: u16,
    pub variants: Vec<RemediatedVerifiedVariant>,
    pub costs: RemediatedCosts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedVerificationReport {
    pub payload: RemediatedVerificationPayload,
    pub digest: RemediatedVerificationDigest,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RemediatedVerifier;

impl RemediatedVerifier {
    pub fn verify(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        lattice_digest: RemediatedLatticeDigest,
        text: &str,
    ) -> Result<RemediatedVerificationReport, RemediatedExpressionError> {
        if text.is_empty() {
            return Err(RemediatedExpressionError::UnsupportedSurface);
        }
        let lattice = RemediatedLattice::build(program, lexical_table)?;
        if lattice.digest != lattice_digest {
            return Err(RemediatedExpressionError::LatticeDigestMismatch);
        }
        let matched = parse_exact(program, &lattice.payload.variants, text)?;
        let canonical = assemble_canonical(program, &matched);
        let original_report = GrammarV3Verifier.verify(
            program,
            lexical_table,
            lattice.payload.original_lattice_digest,
            &canonical,
        )?;
        let costs = recompute_costs(
            program,
            text,
            &matched,
            original_report.payload.costs.claim_cost,
        )?;
        let payload = RemediatedVerificationPayload {
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            lattice_digest: lattice.digest,
            original_lattice_digest: lattice.payload.original_lattice_digest,
            original_verification_digest: original_report.digest,
            grammar_version: REMEDIATED_GRAMMAR_VERSION,
            variants: matched
                .iter()
                .map(|variant| RemediatedVerifiedVariant {
                    operation: variant.operation,
                    variant_id: variant.variant_id,
                    original_variant_id: variant.original_variant_id,
                    kind: variant.kind.clone(),
                    family: variant.family,
                    phase: variant.phase,
                })
                .collect(),
            costs,
        };
        let digest = RemediatedVerificationDigest(digest_value(VERIFY_DOMAIN, &payload)?);
        if digest.0 == 0 {
            return Err(RemediatedExpressionError::EmptyDigest);
        }
        Ok(RemediatedVerificationReport { payload, digest })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedSelectionPayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub voice_projection_digest: String,
    pub model_digest: LearnedExpressionModelDigest,
    pub lattice_digest: Option<RemediatedLatticeDigest>,
    pub selected_grammar_version: u16,
    pub disposition: SelectionDisposition,
    pub text: String,
    pub variant_ids: Vec<RemediatedVariantId>,
    pub family: Option<ExpressionFamily>,
    pub score: i64,
    pub complete_candidates_scored: u16,
    pub verification_digest: Option<RemediatedVerificationDigest>,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedSelectionResult {
    pub payload: RemediatedSelectionPayload,
    pub digest: RemediatedSelectionDigest,
}

#[derive(Debug, Clone)]
pub struct RemediatedOfflineSelector {
    model: LearnedExpressionModel,
}

impl RemediatedOfflineSelector {
    #[must_use]
    pub fn new(model: LearnedExpressionModel) -> Self {
        Self { model }
    }

    pub fn select(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        projection: &VerifiedVoiceProjection,
    ) -> Result<RemediatedSelectionResult, RemediatedExpressionError> {
        program.verify_replay_integrity()?;
        lexical_table.verify_integrity(program)?;
        let neutral = VerifierReadyRenderer.render(program, lexical_table)?;
        let attempted = self.try_select(program, lexical_table, projection);
        let payload = match attempted {
            Ok(payload) => payload,
            Err(error) => RemediatedSelectionPayload {
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
        let digest = RemediatedSelectionDigest(digest_value(SELECT_DOMAIN, &payload)?);
        if digest.0 == 0 {
            return Err(RemediatedExpressionError::EmptyDigest);
        }
        Ok(RemediatedSelectionResult { payload, digest })
    }

    fn try_select(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
        projection: &VerifiedVoiceProjection,
    ) -> Result<RemediatedSelectionPayload, RemediatedExpressionError> {
        self.model.verify_integrity()?;
        projection.verify_integrity()?;
        let lattice = RemediatedLattice::build(program, lexical_table)?;
        let direct_score = model_score(&self.model, projection, VariantProfile::direct());
        let warm_score = model_score(&self.model, projection, VariantProfile::warm());
        let family = if direct_score >= warm_score {
            ExpressionFamily::Direct
        } else {
            ExpressionFamily::Warm
        };
        let mut text = String::new();
        let mut variant_ids = Vec::new();
        for (index, operation) in program.payload.operations.iter().enumerate() {
            text.push_str(separator_before(program, index));
            let phase = diversity_phase(projection.packet_digest, program.digest.0, operation.id.0);
            let variant_id = match family {
                ExpressionFamily::Direct => RemediatedVariantId(u16::from(phase)),
                ExpressionFamily::Warm => RemediatedVariantId(3 + u16::from(phase)),
            };
            let variant = lattice
                .payload
                .variants
                .iter()
                .find(|variant| {
                    variant.operation == operation.id && variant.variant_id == variant_id
                })
                .ok_or(RemediatedExpressionError::MissingVariant)?;
            text.push_str(&variant.text);
            variant_ids.push(variant_id);
        }
        let report = RemediatedVerifier.verify(program, lexical_table, lattice.digest, &text)?;
        Ok(RemediatedSelectionPayload {
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            voice_projection_digest: projection.source_digest.clone(),
            model_digest: self.model.digest,
            lattice_digest: Some(lattice.digest),
            selected_grammar_version: REMEDIATED_GRAMMAR_VERSION,
            disposition: SelectionDisposition::LearnedVerified,
            text,
            variant_ids,
            family: Some(family),
            score: match family {
                ExpressionFamily::Direct => direct_score,
                ExpressionFamily::Warm => warm_score,
            },
            complete_candidates_scored: 2,
            verification_digest: Some(report.digest),
            fallback_reason: None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemediatedAuthorityBoundary {
    pub candidate_lattice_construction: bool,
    pub learned_candidate_scoring: bool,
    pub deterministic_candidate_local_diversity: bool,
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
pub const fn authority_boundary() -> RemediatedAuthorityBoundary {
    RemediatedAuthorityBoundary {
        candidate_lattice_construction: true,
        learned_candidate_scoring: true,
        deterministic_candidate_local_diversity: true,
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
pub enum RemediatedExpressionError {
    #[error("base learned-expression failure: {0}")]
    Base(#[from] LearnedExpressionError),
    #[error("projection packet is stale or corrupt: {0}")]
    Projection(#[from] crate::omega_v1f1_projection_guard::ProjectionPacketError),
    #[error("semantic program validation failed: {0}")]
    Semantic(#[from] crate::semantic_response::SemanticProgramError),
    #[error("lexical table validation failed: {0}")]
    Lexical(#[from] crate::language_realization::RealizationError),
    #[error("neutral realization failed: {0}")]
    Neutral(#[from] crate::verifier_ready_realization::VerifierReadyRealizationError),
    #[error("remediated lattice is malformed or ambiguous")]
    InvalidLattice,
    #[error("remediated lattice digest is stale or mismatched")]
    LatticeDigestMismatch,
    #[error("remediated surface is unsupported or ambiguous")]
    UnsupportedSurface,
    #[error("required remediated candidate is missing")]
    MissingVariant,
    #[error("remediated output exceeds a frozen budget")]
    BudgetExceeded,
    #[error("canonical serialization failed: {0}")]
    CanonicalSerialization(String),
    #[error("canonical digest is zero")]
    EmptyDigest,
}

fn unique_profile_variant<'a>(
    variants: &[&'a crate::learned_expression::OperationSurfaceVariant],
    profile: VariantProfile,
) -> Result<&'a crate::learned_expression::OperationSurfaceVariant, RemediatedExpressionError> {
    let found = variants
        .iter()
        .copied()
        .filter(|variant| variant.profile == profile)
        .collect::<Vec<_>>();
    if found.len() != 1 {
        return Err(RemediatedExpressionError::InvalidLattice);
    }
    Ok(found[0])
}

fn validate_variants(
    program: &SemanticResponseProgram,
    variants: &[RemediatedSurfaceVariant],
) -> Result<(), RemediatedExpressionError> {
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
            return Err(RemediatedExpressionError::InvalidLattice);
        }
    }
    if program.payload.operations.iter().any(|operation| {
        ids.get(&operation.id)
            .map(BTreeSet::len)
            .unwrap_or_default()
            != REMEDIATED_VARIANTS_PER_OPERATION
    }) || REMEDIATED_VARIANTS_PER_OPERATION > MAX_VARIANTS_PER_OPERATION
    {
        return Err(RemediatedExpressionError::InvalidLattice);
    }
    Ok(())
}

fn diversify_text(canonical: &str, family: ExpressionFamily, phase: u8) -> String {
    let leads = match family {
        ExpressionFamily::Direct => ["Directly: ", "In brief: ", "The clearest answer: "],
        ExpressionFamily::Warm => [
            "With context preserved: ",
            "A grounded reading: ",
            "The point I would keep: ",
        ],
    };
    let mut text = canonical.to_owned();
    for (marker, direct, warm) in epistemic_alternatives() {
        let alternatives = match family {
            ExpressionFamily::Direct => direct,
            ExpressionFamily::Warm => warm,
        };
        text = text.replace(marker, alternatives[usize::from(phase)]);
    }
    format!("{}{}", leads[usize::from(phase)], text)
}

fn epistemic_alternatives() -> [(&'static str, [&'static str; 3], [&'static str; 3]); 5] {
    [
        (
            "I know that ",
            [
                "The evidence establishes that ",
                "The supported conclusion is that ",
                "It is established that ",
            ],
            [
                "It is clear that ",
                "The record supports that ",
                "We can be confident that ",
            ],
        ),
        (
            "It is probable that ",
            [
                "Most likely, ",
                "The evidence points to ",
                "A probable reading is that ",
            ],
            [
                "It likely follows that ",
                "The stronger likelihood is that ",
                "There is good reason to think that ",
            ],
        ),
        (
            "It is possible that ",
            [
                "Possibly, ",
                "A possible conclusion is that ",
                "The evidence permits that ",
            ],
            [
                "One possibility is that ",
                "It may be that ",
                "There is room to think that ",
            ],
        ),
        (
            "I am uncertain whether ",
            [
                "It remains uncertain whether ",
                "I cannot yet resolve whether ",
                "The evidence is unclear on whether ",
            ],
            [
                "Uncertainty remains over whether ",
                "I am not confident whether ",
                "There is not enough clarity to say whether ",
            ],
        ),
        (
            "I do not know whether ",
            [
                "It is unknown whether ",
                "The evidence does not show whether ",
                "I cannot determine whether ",
            ],
            [
                "There is no basis to decide whether ",
                "Whether this holds is unknown: ",
                "The answer remains unknown as to whether ",
            ],
        ),
    ]
}

fn parse_exact<'a>(
    program: &SemanticResponseProgram,
    variants: &'a [RemediatedSurfaceVariant],
    text: &str,
) -> Result<Vec<&'a RemediatedSurfaceVariant>, RemediatedExpressionError> {
    let mut cursor = 0usize;
    let mut matched = Vec::with_capacity(program.payload.operations.len());
    for (index, operation) in program.payload.operations.iter().enumerate() {
        let separator = separator_before(program, index);
        if !text[cursor..].starts_with(separator) {
            return Err(RemediatedExpressionError::UnsupportedSurface);
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
            return Err(RemediatedExpressionError::UnsupportedSurface);
        }
        cursor += candidates[0].text.len();
        matched.push(candidates[0]);
    }
    if cursor != text.len() {
        return Err(RemediatedExpressionError::UnsupportedSurface);
    }
    Ok(matched)
}

fn assemble_canonical(
    program: &SemanticResponseProgram,
    variants: &[&RemediatedSurfaceVariant],
) -> String {
    let mut text = String::new();
    for (index, variant) in variants.iter().enumerate() {
        text.push_str(separator_before(program, index));
        text.push_str(&variant.canonical_text);
    }
    text
}

fn recompute_costs(
    program: &SemanticResponseProgram,
    text: &str,
    variants: &[&RemediatedSurfaceVariant],
    claim_cost: u32,
) -> Result<RemediatedCosts, RemediatedExpressionError> {
    let operation_cost =
        u32::try_from(variants.len()).map_err(|_| RemediatedExpressionError::BudgetExceeded)?;
    let verification_step_cost = operation_cost
        .checked_add(claim_cost)
        .and_then(|cost| cost.checked_add(operation_cost))
        .and_then(|cost| cost.checked_add(operation_cost))
        .ok_or(RemediatedExpressionError::BudgetExceeded)?;
    let character_cost =
        u32::try_from(text.len()).map_err(|_| RemediatedExpressionError::BudgetExceeded)?;
    let sentence_count = count_sentences(text)?;
    let paragraph_count = u16::try_from(text.split("\n\n").count())
        .map_err(|_| RemediatedExpressionError::BudgetExceeded)?;
    if operation_cost > u32::from(program.payload.compute_budget.maximum_operations)
        || claim_cost > u32::from(program.payload.compute_budget.maximum_claims)
        || verification_step_cost > program.payload.compute_budget.maximum_verification_steps
        || character_cost > program.payload.output_budget.maximum_characters
        || sentence_count > program.payload.output_budget.maximum_sentences
        || paragraph_count > program.payload.style.maximum_paragraphs
        || paragraph_count == 0
    {
        return Err(RemediatedExpressionError::BudgetExceeded);
    }
    Ok(RemediatedCosts {
        operation_cost,
        claim_cost,
        verification_step_cost,
        character_cost,
        sentence_count,
        paragraph_count,
    })
}

fn count_sentences(text: &str) -> Result<u16, RemediatedExpressionError> {
    let count = text
        .chars()
        .filter(|character| matches!(character, '.' | '?' | '!'))
        .count();
    if count == 0 {
        return Err(RemediatedExpressionError::BudgetExceeded);
    }
    u16::try_from(count).map_err(|_| RemediatedExpressionError::BudgetExceeded)
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
        && index.is_multiple_of(operations_per_paragraph)
    {
        "\n\n"
    } else {
        " "
    }
}

fn model_score(
    model: &LearnedExpressionModel,
    projection: &VerifiedVoiceProjection,
    profile: VariantProfile,
) -> i64 {
    let projection = projection.as_array();
    let profile = profile.as_array();
    model
        .payload
        .weights
        .iter()
        .enumerate()
        .map(|(index, weight)| {
            let matched = 10_000_u16.saturating_sub(projection[index].abs_diff(profile[index]));
            i64::from(*weight) * i64::from(matched) / 10_000
        })
        .sum()
}

fn diversity_phase(packet_digest: u64, program_digest: u64, operation_id: u64) -> u8 {
    let mixed = packet_digest
        ^ program_digest.rotate_left(17)
        ^ operation_id.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    (mixed % 3) as u8
}

fn digest_value<T: Serialize>(domain: &[u8], value: &T) -> Result<u64, RemediatedExpressionError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| RemediatedExpressionError::CanonicalSerialization(error.to_string()))?;
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
    fn bounds_remain_stricter_than_frozen_caps() {
        assert_eq!(
            REMEDIATED_VARIANTS_PER_OPERATION,
            MAX_VARIANTS_PER_OPERATION
        );
        assert!(2 <= crate::learned_expression::MAX_RESPONSE_CANDIDATES);
        assert!(7 <= crate::learned_expression::MAX_TRAINABLE_PARAMETERS);
        assert!(crate::learned_expression::MAX_MODEL_BYTES >= 1024);
    }

    #[test]
    fn epistemic_surfaces_are_distinct() {
        let canonical = "Conclusion: It is possible that the bounded selector works.";
        let mut surfaces = BTreeSet::new();
        for family in [ExpressionFamily::Direct, ExpressionFamily::Warm] {
            for phase in 0..3 {
                surfaces.insert(diversify_text(canonical, family, phase));
            }
        }
        assert_eq!(surfaces.len(), REMEDIATED_VARIANTS_PER_OPERATION);
        assert!(surfaces
            .iter()
            .all(|surface| !surface.contains("It is possible that")));
    }
}
