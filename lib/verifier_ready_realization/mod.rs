//! STLM L1 verifier-ready deterministic realization.
//!
//! This module preserves the frozen L0-C renderer and adds a separate grammar-v2
//! surface whose operation forms are invertible without renderer alignments.

use crate::language_realization::{
    ClaimLexicalBinding, LexicalBindingTable, LexicalTableDigest, SurfaceReference, TextSpan,
};
use crate::semantic_response::{
    AbstentionReason, AuthorizedClaim, ClaimId, ClaimPolarity, DetailLevel, DiscourseOperationKind,
    EpistemicStatus, MissingVariableId, ObservationId, OperationId, PredictionId,
    ResponseProgramDigest, SemanticProgramError, SemanticResponseProgram,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

const REALIZATION_DIGEST_DOMAIN: &[u8] = b"starfire-stlm-verifier-ready-realization-v2";
pub const VERIFIER_READY_GRAMMAR_VERSION: u16 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierReadyRealizationDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifierReadyRendererIdentity {
    DeterministicVerifierReadyV2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierReadyAlignment {
    pub operation: OperationId,
    pub span: TextSpan,
    pub claim_ids: Vec<ClaimId>,
    pub references: Vec<SurfaceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierReadySurfacePayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub renderer: VerifierReadyRendererIdentity,
    pub grammar_version: u16,
    pub text: String,
    pub alignments: Vec<VerifierReadyAlignment>,
    pub operation_cost: u32,
    pub claim_cost: u32,
    pub verification_step_cost: u32,
    pub character_cost: u32,
    pub sentence_count: u16,
    pub paragraph_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierReadySurface {
    pub payload: VerifierReadySurfacePayload,
    pub digest: VerifierReadyRealizationDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierReadyRendererAuthorityBoundary {
    pub runtime_chat_wiring: bool,
    pub live_generated_text_influence: bool,
    pub raw_conversation_access: bool,
    pub unrestricted_memory_access: bool,
    pub persistence_authority: bool,
    pub routing_authority: bool,
    pub companion_mutation_authority: bool,
    pub belief_promotion_authority: bool,
    pub ontology_promotion_authority: bool,
    pub tool_selection_authority: bool,
    pub charge_discharge_authority: bool,
    pub autonomous_action_authority: bool,
}

#[must_use]
pub const fn authority_boundary() -> VerifierReadyRendererAuthorityBoundary {
    VerifierReadyRendererAuthorityBoundary {
        runtime_chat_wiring: false,
        live_generated_text_influence: false,
        raw_conversation_access: false,
        unrestricted_memory_access: false,
        persistence_authority: false,
        routing_authority: false,
        companion_mutation_authority: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        tool_selection_authority: false,
        charge_discharge_authority: false,
        autonomous_action_authority: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VerifierReadyRealizationError {
    #[error("semantic program validation failed: {0}")]
    SemanticProgram(#[from] SemanticProgramError),
    #[error("lexical binding table validation failed: {0}")]
    LexicalTable(String),
    #[error("a required lexical binding is missing")]
    MissingLexicalBinding,
    #[error("the verifier-ready surface contains a forbidden form")]
    ForbiddenSurfaceForm,
    #[error("the verifier-ready surface exceeds an output or compute budget")]
    BudgetExceeded,
    #[error("canonical serialization failed: {0}")]
    CanonicalSerialization(String),
    #[error("the realization digest is zero")]
    EmptyDigest,
    #[error("the realization digest does not match canonical bytes")]
    DigestMismatch,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VerifierReadyRenderer;

impl VerifierReadyRenderer {
    pub fn render(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Result<VerifierReadySurface, VerifierReadyRealizationError> {
        program.verify_replay_integrity()?;
        lexical_table
            .verify_integrity(program)
            .map_err(|error| VerifierReadyRealizationError::LexicalTable(error.to_string()))?;

        let claim_map = program
            .payload
            .required_claims
            .iter()
            .chain(program.payload.optional_claims.iter())
            .map(|claim| (claim.id, claim))
            .collect::<BTreeMap<_, _>>();
        let lexical_claims = lexical_table
            .payload
            .claims
            .iter()
            .map(|binding| (binding.claim, binding))
            .collect::<BTreeMap<_, _>>();
        let observations = lexical_table
            .payload
            .observations
            .iter()
            .map(|binding| (binding.observation, binding.label.as_str()))
            .collect::<BTreeMap<_, _>>();
        let variables = lexical_table
            .payload
            .missing_variables
            .iter()
            .map(|binding| (binding.variable, binding.label.as_str()))
            .collect::<BTreeMap<_, _>>();
        let predictions = lexical_table
            .payload
            .predictions
            .iter()
            .map(|binding| (binding.prediction, binding.label.as_str()))
            .collect::<BTreeMap<_, _>>();

        let mut segments = Vec::with_capacity(program.payload.operations.len());
        for operation in &program.payload.operations {
            segments.push(render_operation(
                &operation.kind,
                &claim_map,
                &lexical_claims,
                &observations,
                &variables,
                &predictions,
                program.payload.style.allow_questions,
            )?);
        }

        let target_paragraphs = match program.payload.style.detail {
            DetailLevel::Detailed => segments
                .len()
                .min(usize::from(program.payload.style.maximum_paragraphs)),
            DetailLevel::Brief | DetailLevel::Standard => 1,
        }
        .max(1);
        let operations_per_paragraph = segments.len().div_ceil(target_paragraphs).max(1);

        let mut text = String::new();
        let mut alignments = Vec::with_capacity(segments.len());
        for (index, segment) in segments.iter().enumerate() {
            if index > 0 {
                if program.payload.style.detail == DetailLevel::Detailed
                    && index % operations_per_paragraph == 0
                {
                    text.push_str("\n\n");
                } else {
                    text.push(' ');
                }
            }
            let start_byte = text.len();
            text.push_str(&segment.text);
            let end_byte = text.len();
            alignments.push(VerifierReadyAlignment {
                operation: OperationId(index as u64 + 1),
                span: TextSpan {
                    start_byte,
                    end_byte,
                },
                claim_ids: segment.claim_ids.clone(),
                references: segment.references.clone(),
            });
        }

        reject_forbidden_text(&text, &lexical_table.payload.forbidden_surface_forms)?;

        let operation_cost = u32::try_from(alignments.len())
            .map_err(|_| VerifierReadyRealizationError::BudgetExceeded)?;
        let claim_cost = u32::try_from(
            alignments
                .iter()
                .map(|alignment| alignment.claim_ids.len())
                .sum::<usize>(),
        )
        .map_err(|_| VerifierReadyRealizationError::BudgetExceeded)?;
        let verification_step_cost = operation_cost
            .checked_add(claim_cost)
            .and_then(|cost| cost.checked_add(operation_cost))
            .ok_or(VerifierReadyRealizationError::BudgetExceeded)?;
        let character_cost =
            u32::try_from(text.len()).map_err(|_| VerifierReadyRealizationError::BudgetExceeded)?;
        let sentence_count = count_sentences(&text)?;
        let paragraph_count = u16::try_from(text.split("\n\n").count())
            .map_err(|_| VerifierReadyRealizationError::BudgetExceeded)?;

        validate_budgets(
            program,
            operation_cost,
            claim_cost,
            verification_step_cost,
            character_cost,
            sentence_count,
            paragraph_count,
        )?;

        let payload = VerifierReadySurfacePayload {
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            renderer: VerifierReadyRendererIdentity::DeterministicVerifierReadyV2,
            grammar_version: VERIFIER_READY_GRAMMAR_VERSION,
            text,
            alignments,
            operation_cost,
            claim_cost,
            verification_step_cost,
            character_cost,
            sentence_count,
            paragraph_count,
        };
        let digest = digest_payload(&payload)?;
        if digest.0 == 0 {
            return Err(VerifierReadyRealizationError::EmptyDigest);
        }
        Ok(VerifierReadySurface { payload, digest })
    }
}

impl VerifierReadySurface {
    pub fn verify_digest(&self) -> Result<(), VerifierReadyRealizationError> {
        let expected = digest_payload(&self.payload)?;
        if self.digest.0 == 0 {
            return Err(VerifierReadyRealizationError::EmptyDigest);
        }
        if self.digest != expected {
            return Err(VerifierReadyRealizationError::DigestMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, VerifierReadyRealizationError> {
        canonical_bytes(&self.payload)
    }
}

#[derive(Debug, Clone)]
struct RenderedSegment {
    text: String,
    claim_ids: Vec<ClaimId>,
    references: Vec<SurfaceReference>,
}

fn render_operation(
    kind: &DiscourseOperationKind,
    claims: &BTreeMap<ClaimId, &AuthorizedClaim>,
    lexical_claims: &BTreeMap<ClaimId, &ClaimLexicalBinding>,
    observations: &BTreeMap<ObservationId, &str>,
    variables: &BTreeMap<MissingVariableId, &str>,
    predictions: &BTreeMap<PredictionId, &str>,
    allow_questions: bool,
) -> Result<RenderedSegment, VerifierReadyRealizationError> {
    let mut claim_ids = Vec::new();
    let mut references = Vec::new();
    let text = match kind {
        DiscourseOperationKind::Assert(claim) => {
            claim_ids.push(*claim);
            format!("{}.", render_claim(*claim, claims, lexical_claims)?)
        }
        DiscourseOperationKind::Qualify { claim, status } => {
            claim_ids.push(*claim);
            let authorized = claims
                .get(claim)
                .copied()
                .ok_or(VerifierReadyRealizationError::MissingLexicalBinding)?;
            if authorized.epistemic_status != *status {
                return Err(VerifierReadyRealizationError::MissingLexicalBinding);
            }
            format!(
                "Qualification: {}.",
                render_claim(*claim, claims, lexical_claims)?
            )
        }
        DiscourseOperationKind::Contrast { left, right } => {
            claim_ids.extend([*left, *right]);
            format!(
                "On one side, {}. By contrast, {}.",
                render_claim(*left, claims, lexical_claims)?,
                render_claim(*right, claims, lexical_claims)?
            )
        }
        DiscourseOperationKind::Correct { prior, replacement } => {
            claim_ids.extend([*prior, *replacement]);
            format!(
                "Correction: {}; instead, {}.",
                render_claim(*prior, claims, lexical_claims)?,
                render_claim(*replacement, claims, lexical_claims)?
            )
        }
        DiscourseOperationKind::Explain { claims: explained } => {
            claim_ids.extend(explained.iter().copied());
            let rendered = explained
                .iter()
                .map(|claim| render_claim(*claim, claims, lexical_claims))
                .collect::<Result<Vec<_>, _>>()?;
            format!("Relevant support: {}.", rendered.join("; "))
        }
        DiscourseOperationKind::Acknowledge(observation) => {
            references.push(SurfaceReference::Observation(*observation));
            let label = observations
                .get(observation)
                .copied()
                .ok_or(VerifierReadyRealizationError::MissingLexicalBinding)?;
            format!("I acknowledge {}.", label)
        }
        DiscourseOperationKind::RequestEvidence(variable) => {
            references.push(SurfaceReference::MissingVariable(*variable));
            let label = variables
                .get(variable)
                .copied()
                .ok_or(VerifierReadyRealizationError::MissingLexicalBinding)?;
            if allow_questions {
                format!("What evidence resolves {}?", label)
            } else {
                format!("Evidence is required for {}.", label)
            }
        }
        DiscourseOperationKind::Commit(prediction) => {
            references.push(SurfaceReference::Prediction(*prediction));
            let label = predictions
                .get(prediction)
                .copied()
                .ok_or(VerifierReadyRealizationError::MissingLexicalBinding)?;
            format!("I commit to track {}.", label)
        }
        DiscourseOperationKind::Abstain(reason) => abstention_text(*reason).to_owned(),
    };
    Ok(RenderedSegment {
        text,
        claim_ids,
        references,
    })
}

fn render_claim(
    claim_id: ClaimId,
    claims: &BTreeMap<ClaimId, &AuthorizedClaim>,
    lexical_claims: &BTreeMap<ClaimId, &ClaimLexicalBinding>,
) -> Result<String, VerifierReadyRealizationError> {
    let claim = claims
        .get(&claim_id)
        .copied()
        .ok_or(VerifierReadyRealizationError::MissingLexicalBinding)?;
    let binding = lexical_claims
        .get(&claim_id)
        .copied()
        .ok_or(VerifierReadyRealizationError::MissingLexicalBinding)?;
    let clause = match claim.polarity {
        ClaimPolarity::Positive => &binding.positive_clause,
        ClaimPolarity::Negative => &binding.negative_clause,
    };
    Ok(format!(
        "{} {}",
        epistemic_marker(claim.epistemic_status),
        clause
    ))
}

#[must_use]
pub const fn epistemic_marker(status: EpistemicStatus) -> &'static str {
    match status {
        EpistemicStatus::Certain => "I know that",
        EpistemicStatus::Probable => "It is probable that",
        EpistemicStatus::Possible => "It is possible that",
        EpistemicStatus::Uncertain => "I am uncertain whether",
        EpistemicStatus::Unknown => "I do not know whether",
    }
}

#[must_use]
pub const fn abstention_text(reason: AbstentionReason) -> &'static str {
    match reason {
        AbstentionReason::InsufficientEvidence => {
            "I abstain because the available evidence is insufficient."
        }
        AbstentionReason::ContradictoryEvidence => {
            "I abstain because the available evidence is contradictory."
        }
        AbstentionReason::SensitiveContext => {
            "I abstain because this context is too sensitive for disclosure."
        }
        AbstentionReason::UnsupportedIntent => {
            "I abstain because this response intent is unsupported."
        }
        AbstentionReason::BudgetExhausted => {
            "I abstain because the authorized response budget is exhausted."
        }
    }
}

fn reject_forbidden_text(
    text: &str,
    forbidden_forms: &[String],
) -> Result<(), VerifierReadyRealizationError> {
    let normalized = text.to_lowercase();
    if forbidden_forms
        .iter()
        .any(|form| normalized.contains(&form.to_lowercase()))
    {
        return Err(VerifierReadyRealizationError::ForbiddenSurfaceForm);
    }
    Ok(())
}

fn validate_budgets(
    program: &SemanticResponseProgram,
    operation_cost: u32,
    claim_cost: u32,
    verification_step_cost: u32,
    character_cost: u32,
    sentence_count: u16,
    paragraph_count: u16,
) -> Result<(), VerifierReadyRealizationError> {
    if operation_cost > u32::from(program.payload.compute_budget.maximum_operations)
        || claim_cost > u32::from(program.payload.compute_budget.maximum_claims)
        || verification_step_cost > program.payload.compute_budget.maximum_verification_steps
        || character_cost > program.payload.output_budget.maximum_characters
        || sentence_count > program.payload.output_budget.maximum_sentences
        || paragraph_count > program.payload.style.maximum_paragraphs
        || paragraph_count == 0
    {
        return Err(VerifierReadyRealizationError::BudgetExceeded);
    }
    Ok(())
}

fn count_sentences(text: &str) -> Result<u16, VerifierReadyRealizationError> {
    let count = text
        .chars()
        .filter(|character| matches!(character, '.' | '?' | '!'))
        .count();
    if count == 0 {
        return Err(VerifierReadyRealizationError::BudgetExceeded);
    }
    u16::try_from(count).map_err(|_| VerifierReadyRealizationError::BudgetExceeded)
}

fn digest_payload(
    payload: &VerifierReadySurfacePayload,
) -> Result<VerifierReadyRealizationDigest, VerifierReadyRealizationError> {
    let encoded = canonical_bytes(payload)?;
    Ok(VerifierReadyRealizationDigest(domain_digest(
        REALIZATION_DIGEST_DOMAIN,
        &encoded,
    )))
}

fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, VerifierReadyRealizationError> {
    serde_json::to_vec(value)
        .map_err(|error| VerifierReadyRealizationError::CanonicalSerialization(error.to_string()))
}

fn domain_digest(domain: &[u8], encoded: &[u8]) -> u64 {
    let mut digest = fnv1a64(domain);
    digest = mix_u64(digest, encoded.len() as u64);
    for byte in encoded {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut digest = 0xcbf29ce484222325_u64;
    for byte in bytes {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest
}

fn mix_u64(mut digest: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(0x100000001b3);
    }
    digest
}