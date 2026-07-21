//! STLM L1 independent inverse language verifier.
//!
//! Semantic acceptance is reconstructed from text. Renderer alignments are not
//! accepted as an input and the forward renderer is never called.

use crate::language_realization::{
    ClaimLexicalBinding, LexicalBindingTable, LexicalTableDigest, RealizationError, TextSpan,
};
use crate::semantic_response::{
    AbstentionReason, AuthorizedClaim, ClaimId, ClaimPolarity, DetailLevel, DiscourseOperationKind,
    EpistemicStatus, MissingVariableId, ObservationId, OperationId, PredictionId,
    ResponseProgramDigest, SemanticProgramError, SemanticResponseProgram, SubjectScope,
};
use crate::verifier_ready_realization::VERIFIER_READY_GRAMMAR_VERSION;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

const REPORT_DIGEST_DOMAIN: &[u8] = b"starfire-stlm-independent-language-verification-v1";
const MAX_CANDIDATE_SURFACES: usize = 4_096;

pub struct LanguageVerificationInput<'a> {
    pub program: &'a SemanticResponseProgram,
    pub lexical_table: &'a LexicalBindingTable,
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub subject_scope: SubjectScope,
    pub grammar_version: u16,
    pub text: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageVerificationReportDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationTerminalClassification {
    Pass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconstructedClaim {
    pub claim: ClaimId,
    pub polarity: ClaimPolarity,
    pub epistemic_status: EpistemicStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconstructedOperationKind {
    Assert(ReconstructedClaim),
    Qualify(ReconstructedClaim),
    Contrast {
        left: ReconstructedClaim,
        right: ReconstructedClaim,
    },
    Correct {
        prior: ReconstructedClaim,
        replacement: ReconstructedClaim,
    },
    Explain {
        claims: Vec<ReconstructedClaim>,
    },
    Acknowledge(ObservationId),
    RequestEvidence(MissingVariableId),
    Commit(PredictionId),
    Abstain(AbstentionReason),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconstructedOperation {
    pub operation: OperationId,
    pub span: TextSpan,
    pub kind: ReconstructedOperationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependentlyRecomputedCosts {
    pub operation_cost: u32,
    pub claim_cost: u32,
    pub verification_step_cost: u32,
    pub character_cost: u32,
    pub sentence_count: u16,
    pub paragraph_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageVerificationReportPayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub subject_scope: SubjectScope,
    pub grammar_version: u16,
    pub reconstructed_operations: Vec<ReconstructedOperation>,
    pub costs: IndependentlyRecomputedCosts,
    pub terminal_classification: VerificationTerminalClassification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageVerificationReport {
    pub payload: LanguageVerificationReportPayload,
    pub digest: LanguageVerificationReportDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageVerifierAuthorityBoundary {
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
pub const fn authority_boundary() -> LanguageVerifierAuthorityBoundary {
    LanguageVerifierAuthorityBoundary {
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
pub enum LanguageVerificationError {
    #[error("semantic program validation failed: {0}")]
    SemanticProgram(#[from] SemanticProgramError),
    #[error("lexical binding table validation failed: {0}")]
    LexicalTable(#[from] RealizationError),
    #[error("the supplied program digest is stale or mismatched")]
    ProgramDigestMismatch,
    #[error("the supplied lexical-table digest is stale or mismatched")]
    LexicalDigestMismatch,
    #[error("the supplied subject scope is stale or mismatched")]
    SubjectScopeMismatch,
    #[error("the verifier supports only verifier-ready grammar version 2")]
    GrammarVersionMismatch,
    #[error("the candidate text is empty")]
    EmptyText,
    #[error("the inverse lexicon contains an ambiguous surface binding")]
    AmbiguousSurfaceBinding,
    #[error("the inverse lexicon exceeds its bounded candidate budget")]
    CandidateBudgetExceeded,
    #[error("the candidate contains an unsupported, malformed, or unparsed surface")]
    UnsupportedSurface,
    #[error("the reconstructed operation sequence does not match the authorized program")]
    OperationMismatch,
    #[error("the candidate contains a prohibited or forbidden surface form")]
    ForbiddenSurfaceForm,
    #[error("the independently recomputed output or compute budget is exceeded")]
    BudgetExceeded,
    #[error("canonical serialization failed: {0}")]
    CanonicalSerialization(String),
    #[error("the report digest is zero")]
    EmptyDigest,
    #[error("the report digest does not match canonical bytes")]
    DigestMismatch,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IndependentLanguageVerifier;

impl IndependentLanguageVerifier {
    pub fn verify(
        &self,
        input: LanguageVerificationInput<'_>,
    ) -> Result<LanguageVerificationReport, LanguageVerificationError> {
        input.program.verify_replay_integrity()?;
        input.lexical_table.verify_integrity(input.program)?;

        if input.program_digest != input.program.digest {
            return Err(LanguageVerificationError::ProgramDigestMismatch);
        }
        if input.lexical_table_digest != input.lexical_table.digest {
            return Err(LanguageVerificationError::LexicalDigestMismatch);
        }
        if input.subject_scope != input.program.payload.subject_scope
            || input.subject_scope != input.lexical_table.payload.subject_scope
        {
            return Err(LanguageVerificationError::SubjectScopeMismatch);
        }
        if input.grammar_version != VERIFIER_READY_GRAMMAR_VERSION {
            return Err(LanguageVerificationError::GrammarVersionMismatch);
        }
        if input.text.is_empty() {
            return Err(LanguageVerificationError::EmptyText);
        }
        reject_forbidden_text(
            input.text,
            &input.lexical_table.payload.forbidden_surface_forms,
        )?;

        let candidates = build_candidates(input.program, input.lexical_table)?;
        let reconstructed = parse_text(input.program, input.text, &candidates)?;
        compare_with_program(input.program, &reconstructed)?;
        let costs = recompute_costs(input.program, input.text, &reconstructed)?;

        let payload = LanguageVerificationReportPayload {
            program_digest: input.program_digest,
            lexical_table_digest: input.lexical_table_digest,
            subject_scope: input.subject_scope,
            grammar_version: input.grammar_version,
            reconstructed_operations: reconstructed,
            costs,
            terminal_classification: VerificationTerminalClassification::Pass,
        };
        let digest = digest_payload(&payload)?;
        if digest.0 == 0 {
            return Err(LanguageVerificationError::EmptyDigest);
        }
        Ok(LanguageVerificationReport { payload, digest })
    }
}

impl LanguageVerificationReport {
    pub fn verify_digest(&self) -> Result<(), LanguageVerificationError> {
        let expected = digest_payload(&self.payload)?;
        if self.digest.0 == 0 {
            return Err(LanguageVerificationError::EmptyDigest);
        }
        if self.digest != expected {
            return Err(LanguageVerificationError::DigestMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, LanguageVerificationError> {
        canonical_bytes(&self.payload)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateSurface {
    text: String,
    kind: ReconstructedOperationKind,
}

fn build_candidates(
    program: &SemanticResponseProgram,
    lexical_table: &LexicalBindingTable,
) -> Result<Vec<CandidateSurface>, LanguageVerificationError> {
    let claims = program
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

    let mut by_surface = BTreeMap::<String, ReconstructedOperationKind>::new();

    for claim in claims.values().copied() {
        let witness = reconstructed_claim(claim);
        let claim_surface = claim_surface(claim, &lexical_claims)?;
        insert_candidate(
            &mut by_surface,
            format!("{}.", claim_surface),
            ReconstructedOperationKind::Assert(witness),
        )?;
        insert_candidate(
            &mut by_surface,
            format!("Qualification: {}.", claim_surface),
            ReconstructedOperationKind::Qualify(witness),
        )?;
    }

    for operation in &program.payload.operations {
        match &operation.kind {
            DiscourseOperationKind::Contrast { left, right } => {
                let left_claim = claims
                    .get(left)
                    .copied()
                    .ok_or(LanguageVerificationError::UnsupportedSurface)?;
                let right_claim = claims
                    .get(right)
                    .copied()
                    .ok_or(LanguageVerificationError::UnsupportedSurface)?;
                insert_candidate(
                    &mut by_surface,
                    format!(
                        "On one side, {}. By contrast, {}.",
                        claim_surface(left_claim, &lexical_claims)?,
                        claim_surface(right_claim, &lexical_claims)?
                    ),
                    ReconstructedOperationKind::Contrast {
                        left: reconstructed_claim(left_claim),
                        right: reconstructed_claim(right_claim),
                    },
                )?;
            }
            DiscourseOperationKind::Correct { prior, replacement } => {
                let prior_claim = claims
                    .get(prior)
                    .copied()
                    .ok_or(LanguageVerificationError::UnsupportedSurface)?;
                let replacement_claim = claims
                    .get(replacement)
                    .copied()
                    .ok_or(LanguageVerificationError::UnsupportedSurface)?;
                insert_candidate(
                    &mut by_surface,
                    format!(
                        "Correction: {}; instead, {}.",
                        claim_surface(prior_claim, &lexical_claims)?,
                        claim_surface(replacement_claim, &lexical_claims)?
                    ),
                    ReconstructedOperationKind::Correct {
                        prior: reconstructed_claim(prior_claim),
                        replacement: reconstructed_claim(replacement_claim),
                    },
                )?;
            }
            DiscourseOperationKind::Explain { claims: explained } => {
                let mut witnesses = Vec::with_capacity(explained.len());
                let mut surfaces = Vec::with_capacity(explained.len());
                for claim_id in explained {
                    let claim = claims
                        .get(claim_id)
                        .copied()
                        .ok_or(LanguageVerificationError::UnsupportedSurface)?;
                    witnesses.push(reconstructed_claim(claim));
                    surfaces.push(claim_surface(claim, &lexical_claims)?);
                }
                insert_candidate(
                    &mut by_surface,
                    format!("Relevant support: {}.", surfaces.join("; ")),
                    ReconstructedOperationKind::Explain { claims: witnesses },
                )?;
            }
            DiscourseOperationKind::Assert(_)
            | DiscourseOperationKind::Qualify { .. }
            | DiscourseOperationKind::Acknowledge(_)
            | DiscourseOperationKind::RequestEvidence(_)
            | DiscourseOperationKind::Commit(_)
            | DiscourseOperationKind::Abstain(_) => {}
        }
    }

    for binding in &lexical_table.payload.observations {
        insert_candidate(
            &mut by_surface,
            format!("I acknowledge {}.", binding.label),
            ReconstructedOperationKind::Acknowledge(binding.observation),
        )?;
    }
    for binding in &lexical_table.payload.missing_variables {
        let text = if program.payload.style.allow_questions {
            format!("What evidence resolves {}?", binding.label)
        } else {
            format!("Evidence is required for {}.", binding.label)
        };
        insert_candidate(
            &mut by_surface,
            text,
            ReconstructedOperationKind::RequestEvidence(binding.variable),
        )?;
    }
    for binding in &lexical_table.payload.predictions {
        insert_candidate(
            &mut by_surface,
            format!("I commit to track {}.", binding.label),
            ReconstructedOperationKind::Commit(binding.prediction),
        )?;
    }
    for reason in [
        AbstentionReason::InsufficientEvidence,
        AbstentionReason::ContradictoryEvidence,
        AbstentionReason::SensitiveContext,
        AbstentionReason::UnsupportedIntent,
        AbstentionReason::BudgetExhausted,
    ] {
        insert_candidate(
            &mut by_surface,
            abstention_text(reason).to_owned(),
            ReconstructedOperationKind::Abstain(reason),
        )?;
    }

    if by_surface.len() > MAX_CANDIDATE_SURFACES {
        return Err(LanguageVerificationError::CandidateBudgetExceeded);
    }

    Ok(by_surface
        .into_iter()
        .map(|(text, kind)| CandidateSurface { text, kind })
        .collect())
}

fn insert_candidate(
    by_surface: &mut BTreeMap<String, ReconstructedOperationKind>,
    text: String,
    kind: ReconstructedOperationKind,
) -> Result<(), LanguageVerificationError> {
    if let Some(existing) = by_surface.get(&text) {
        if existing != &kind {
            return Err(LanguageVerificationError::AmbiguousSurfaceBinding);
        }
        return Ok(());
    }
    by_surface.insert(text, kind);
    Ok(())
}

fn parse_text(
    program: &SemanticResponseProgram,
    text: &str,
    candidates: &[CandidateSurface],
) -> Result<Vec<ReconstructedOperation>, LanguageVerificationError> {
    let operation_count = program.payload.operations.len();
    let operations_per_paragraph = operations_per_paragraph(program, operation_count)?;
    let mut cursor = 0_usize;
    let mut reconstructed = Vec::with_capacity(operation_count);

    for index in 0..operation_count {
        let remaining = text
            .get(cursor..)
            .ok_or(LanguageVerificationError::UnsupportedSurface)?;
        let separator = if index + 1 == operation_count {
            None
        } else if program.payload.style.detail == DetailLevel::Detailed
            && (index + 1) % operations_per_paragraph == 0
        {
            Some("\n\n")
        } else {
            Some(" ")
        };

        let mut matches = candidates.iter().filter(|candidate| {
            if !remaining.starts_with(&candidate.text) {
                return false;
            }
            let after = &remaining[candidate.text.len()..];
            match separator {
                Some(separator) => after.starts_with(separator),
                None => after.is_empty(),
            }
        });

        let candidate = matches
            .next()
            .ok_or(LanguageVerificationError::UnsupportedSurface)?;
        if matches.next().is_some() {
            return Err(LanguageVerificationError::AmbiguousSurfaceBinding);
        }

        let start_byte = cursor;
        let end_byte = cursor
            .checked_add(candidate.text.len())
            .ok_or(LanguageVerificationError::UnsupportedSurface)?;
        reconstructed.push(ReconstructedOperation {
            operation: OperationId(index as u64 + 1),
            span: TextSpan {
                start_byte,
                end_byte,
            },
            kind: candidate.kind.clone(),
        });
        cursor = end_byte;
        if let Some(separator) = separator {
            cursor = cursor
                .checked_add(separator.len())
                .ok_or(LanguageVerificationError::UnsupportedSurface)?;
        }
    }

    if cursor != text.len() {
        return Err(LanguageVerificationError::UnsupportedSurface);
    }
    Ok(reconstructed)
}

fn operations_per_paragraph(
    program: &SemanticResponseProgram,
    operation_count: usize,
) -> Result<usize, LanguageVerificationError> {
    let maximum_paragraphs = usize::from(program.payload.style.maximum_paragraphs);
    if maximum_paragraphs == 0 || operation_count == 0 {
        return Err(LanguageVerificationError::BudgetExceeded);
    }
    let target_paragraphs = match program.payload.style.detail {
        DetailLevel::Detailed => operation_count.min(maximum_paragraphs),
        DetailLevel::Brief | DetailLevel::Standard => 1,
    }
    .max(1);
    Ok(operation_count.div_ceil(target_paragraphs).max(1))
}

fn compare_with_program(
    program: &SemanticResponseProgram,
    reconstructed: &[ReconstructedOperation],
) -> Result<(), LanguageVerificationError> {
    if reconstructed.len() != program.payload.operations.len() {
        return Err(LanguageVerificationError::OperationMismatch);
    }
    let claims = program
        .payload
        .required_claims
        .iter()
        .chain(program.payload.optional_claims.iter())
        .map(|claim| (claim.id, claim))
        .collect::<BTreeMap<_, _>>();

    for (expected, actual) in program.payload.operations.iter().zip(reconstructed) {
        if expected.id != actual.operation {
            return Err(LanguageVerificationError::OperationMismatch);
        }
        let expected_kind = reconstruct_expected_kind(&expected.kind, &claims)?;
        if expected_kind != actual.kind {
            return Err(LanguageVerificationError::OperationMismatch);
        }
    }
    Ok(())
}

fn reconstruct_expected_kind(
    kind: &DiscourseOperationKind,
    claims: &BTreeMap<ClaimId, &AuthorizedClaim>,
) -> Result<ReconstructedOperationKind, LanguageVerificationError> {
    Ok(match kind {
        DiscourseOperationKind::Assert(claim) => {
            ReconstructedOperationKind::Assert(witness_for(*claim, claims)?)
        }
        DiscourseOperationKind::Qualify { claim, status } => {
            let witness = witness_for(*claim, claims)?;
            if witness.epistemic_status != *status {
                return Err(LanguageVerificationError::OperationMismatch);
            }
            ReconstructedOperationKind::Qualify(witness)
        }
        DiscourseOperationKind::Contrast { left, right } => {
            ReconstructedOperationKind::Contrast {
                left: witness_for(*left, claims)?,
                right: witness_for(*right, claims)?,
            }
        }
        DiscourseOperationKind::Correct { prior, replacement } => {
            ReconstructedOperationKind::Correct {
                prior: witness_for(*prior, claims)?,
                replacement: witness_for(*replacement, claims)?,
            }
        }
        DiscourseOperationKind::Explain { claims: explained } => {
            ReconstructedOperationKind::Explain {
                claims: explained
                    .iter()
                    .map(|claim| witness_for(*claim, claims))
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        DiscourseOperationKind::Acknowledge(observation) => {
            ReconstructedOperationKind::Acknowledge(*observation)
        }
        DiscourseOperationKind::RequestEvidence(variable) => {
            ReconstructedOperationKind::RequestEvidence(*variable)
        }
        DiscourseOperationKind::Commit(prediction) => {
            ReconstructedOperationKind::Commit(*prediction)
        }
        DiscourseOperationKind::Abstain(reason) => {
            ReconstructedOperationKind::Abstain(*reason)
        }
    })
}

fn witness_for(
    claim: ClaimId,
    claims: &BTreeMap<ClaimId, &AuthorizedClaim>,
) -> Result<ReconstructedClaim, LanguageVerificationError> {
    claims
        .get(&claim)
        .copied()
        .map(reconstructed_claim)
        .ok_or(LanguageVerificationError::OperationMismatch)
}

fn reconstructed_claim(claim: &AuthorizedClaim) -> ReconstructedClaim {
    ReconstructedClaim {
        claim: claim.id,
        polarity: claim.polarity,
        epistemic_status: claim.epistemic_status,
    }
}

fn claim_surface(
    claim: &AuthorizedClaim,
    lexical_claims: &BTreeMap<ClaimId, &ClaimLexicalBinding>,
) -> Result<String, LanguageVerificationError> {
    let binding = lexical_claims
        .get(&claim.id)
        .copied()
        .ok_or(LanguageVerificationError::UnsupportedSurface)?;
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

fn recompute_costs(
    program: &SemanticResponseProgram,
    text: &str,
    operations: &[ReconstructedOperation],
) -> Result<IndependentlyRecomputedCosts, LanguageVerificationError> {
    let operation_cost =
        u32::try_from(operations.len()).map_err(|_| LanguageVerificationError::BudgetExceeded)?;
    let claim_cost = u32::try_from(
        operations
            .iter()
            .map(|operation| reconstructed_claim_count(&operation.kind))
            .sum::<usize>(),
    )
    .map_err(|_| LanguageVerificationError::BudgetExceeded)?;
    let verification_step_cost = operation_cost
        .checked_add(claim_cost)
        .and_then(|cost| cost.checked_add(operation_cost))
        .ok_or(LanguageVerificationError::BudgetExceeded)?;
    let character_cost =
        u32::try_from(text.len()).map_err(|_| LanguageVerificationError::BudgetExceeded)?;
    let sentence_count = count_sentences(text)?;
    let paragraph_count = u16::try_from(text.split("\n\n").count())
        .map_err(|_| LanguageVerificationError::BudgetExceeded)?;

    if operation_cost > u32::from(program.payload.compute_budget.maximum_operations)
        || claim_cost > u32::from(program.payload.compute_budget.maximum_claims)
        || verification_step_cost > program.payload.compute_budget.maximum_verification_steps
        || character_cost > program.payload.output_budget.maximum_characters
        || sentence_count > program.payload.output_budget.maximum_sentences
        || paragraph_count > program.payload.style.maximum_paragraphs
        || paragraph_count == 0
    {
        return Err(LanguageVerificationError::BudgetExceeded);
    }

    Ok(IndependentlyRecomputedCosts {
        operation_cost,
        claim_cost,
        verification_step_cost,
        character_cost,
        sentence_count,
        paragraph_count,
    })
}

fn reconstructed_claim_count(kind: &ReconstructedOperationKind) -> usize {
    match kind {
        ReconstructedOperationKind::Assert(_) | ReconstructedOperationKind::Qualify(_) => 1,
        ReconstructedOperationKind::Contrast { .. }
        | ReconstructedOperationKind::Correct { .. } => 2,
        ReconstructedOperationKind::Explain { claims } => claims.len(),
        ReconstructedOperationKind::Acknowledge(_)
        | ReconstructedOperationKind::RequestEvidence(_)
        | ReconstructedOperationKind::Commit(_)
        | ReconstructedOperationKind::Abstain(_) => 0,
    }
}

fn reject_forbidden_text(
    text: &str,
    forbidden_forms: &[String],
) -> Result<(), LanguageVerificationError> {
    let normalized = text.to_lowercase();
    if forbidden_forms
        .iter()
        .any(|form| normalized.contains(&form.to_lowercase()))
    {
        return Err(LanguageVerificationError::ForbiddenSurfaceForm);
    }
    Ok(())
}

fn count_sentences(text: &str) -> Result<u16, LanguageVerificationError> {
    let count = text
        .chars()
        .filter(|character| matches!(character, '.' | '?' | '!'))
        .count();
    if count == 0 {
        return Err(LanguageVerificationError::BudgetExceeded);
    }
    u16::try_from(count).map_err(|_| LanguageVerificationError::BudgetExceeded)
}

#[must_use]
const fn epistemic_marker(status: EpistemicStatus) -> &'static str {
    match status {
        EpistemicStatus::Certain => "I know that",
        EpistemicStatus::Probable => "It is probable that",
        EpistemicStatus::Possible => "It is possible that",
        EpistemicStatus::Uncertain => "I am uncertain whether",
        EpistemicStatus::Unknown => "I do not know whether",
    }
}

#[must_use]
const fn abstention_text(reason: AbstentionReason) -> &'static str {
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

fn digest_payload(
    payload: &LanguageVerificationReportPayload,
) -> Result<LanguageVerificationReportDigest, LanguageVerificationError> {
    let encoded = canonical_bytes(payload)?;
    Ok(LanguageVerificationReportDigest(domain_digest(
        REPORT_DIGEST_DOMAIN,
        &encoded,
    )))
}

fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, LanguageVerificationError> {
    serde_json::to_vec(value)
        .map_err(|error| LanguageVerificationError::CanonicalSerialization(error.to_string()))
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