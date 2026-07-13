//! STLM L0-C deterministic reference language realization.
//!
//! The renderer consumes only a validated semantic response program and a
//! scope-bound lexical binding table. It has no access to raw conversation,
//! memory, tools, persistence, routing, belief or ontology mutation, CHARGE
//! discharge, or autonomous action.

use crate::semantic_response::{
    AbstentionReason, AuthorizedClaim, ClaimId, ClaimPolarity, DetailLevel, DialogueMode,
    DiscourseOperationKind, EpistemicStatus, MissingVariableId, ObservationId, OperationId,
    PredictionId, ResponseProgramDigest, SemanticProgramError, SemanticResponseProgram,
    SubjectScope,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

const LEXICAL_DIGEST_DOMAIN: &[u8] = b"starfire-stlm-lexical-binding-table-v1";
const REALIZATION_DIGEST_DOMAIN: &[u8] = b"starfire-stlm-deterministic-realization-v1";
const GRAMMAR_VERSION: u16 = 1;
const MAX_LEXICAL_TEXT_BYTES: usize = 512;
const MAX_FORBIDDEN_FORM_BYTES: usize = 160;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexicalTableDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealizationDigest(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RendererIdentity {
    DeterministicReferenceV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimLexicalBinding {
    pub claim: ClaimId,
    pub positive_clause: String,
    pub negative_clause: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationLexicalBinding {
    pub observation: ObservationId,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingVariableLexicalBinding {
    pub variable: MissingVariableId,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionLexicalBinding {
    pub prediction: PredictionId,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexicalBindingTablePayload {
    pub program_digest: ResponseProgramDigest,
    pub subject_scope: SubjectScope,
    pub claims: Vec<ClaimLexicalBinding>,
    pub observations: Vec<ObservationLexicalBinding>,
    pub missing_variables: Vec<MissingVariableLexicalBinding>,
    pub predictions: Vec<PredictionLexicalBinding>,
    pub forbidden_surface_forms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexicalBindingTable {
    pub payload: LexicalBindingTablePayload,
    pub digest: LexicalTableDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextSpan {
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SurfaceReference {
    Observation(ObservationId),
    MissingVariable(MissingVariableId),
    Prediction(PredictionId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticAlignment {
    pub operation: OperationId,
    pub span: TextSpan,
    pub claim_ids: Vec<ClaimId>,
    pub references: Vec<SurfaceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceRealizationPayload {
    pub program_digest: ResponseProgramDigest,
    pub lexical_table_digest: LexicalTableDigest,
    pub renderer: RendererIdentity,
    pub grammar_version: u16,
    pub text: String,
    pub alignments: Vec<SemanticAlignment>,
    pub operation_cost: u32,
    pub claim_cost: u32,
    pub verification_step_cost: u32,
    pub character_cost: u32,
    pub sentence_count: u16,
    pub paragraph_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceRealization {
    pub payload: SurfaceRealizationPayload,
    pub digest: RealizationDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RendererAuthorityBoundary {
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
pub const fn authority_boundary() -> RendererAuthorityBoundary {
    RendererAuthorityBoundary {
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
pub enum RealizationError {
    #[error("semantic program validation failed: {0}")]
    SemanticProgram(#[from] SemanticProgramError),
    #[error("lexical table program digest does not match the semantic program")]
    ProgramDigestMismatch,
    #[error("lexical table subject scope does not match the semantic program")]
    SubjectScopeMismatch,
    #[error("lexical collection identifiers must be nonzero, unique, and strictly ordered")]
    NoncanonicalLexicalOrder,
    #[error("lexical table contains a missing or unused binding")]
    LexicalCoverageMismatch,
    #[error("lexical table contains a prohibited claim binding")]
    ProhibitedClaimBinding,
    #[error("lexical text is empty, oversized, malformed, or not canonically spaced")]
    InvalidLexicalText,
    #[error("forbidden surface forms must be nonempty, valid, unique, and strictly ordered")]
    InvalidForbiddenForms,
    #[error("a lexical binding or rendered output contains a forbidden surface form")]
    ForbiddenSurfaceForm,
    #[error("lexical table digest is zero")]
    EmptyLexicalDigest,
    #[error("lexical table digest does not match canonical bytes")]
    LexicalDigestMismatch,
    #[error("surface realization exceeds an output or compute budget")]
    BudgetExceeded,
    #[error("surface realization silently omitted or duplicated an operation")]
    OperationCoverageMismatch,
    #[error("surface alignment is malformed, overlapping, out of order, or out of bounds")]
    InvalidAlignment,
    #[error("surface realization does not preserve claim polarity or epistemic status")]
    SemanticMarkerMismatch,
    #[error("surface realization digest is zero")]
    EmptyRealizationDigest,
    #[error("surface realization digest does not match canonical bytes")]
    RealizationDigestMismatch,
    #[error("canonical serialization failed: {0}")]
    CanonicalSerialization(String),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DeterministicRenderer;

impl LexicalBindingTable {
    pub fn validate(
        payload: LexicalBindingTablePayload,
        program: &SemanticResponseProgram,
    ) -> Result<Self, RealizationError> {
        program.verify_replay_integrity()?;
        validate_lexical_payload(&payload, program)?;
        let digest = digest_lexical_payload(&payload)?;
        if digest.0 == 0 {
            return Err(RealizationError::EmptyLexicalDigest);
        }
        Ok(Self { payload, digest })
    }

    pub fn verify_integrity(
        &self,
        program: &SemanticResponseProgram,
    ) -> Result<(), RealizationError> {
        program.verify_replay_integrity()?;
        validate_lexical_payload(&self.payload, program)?;
        let expected = digest_lexical_payload(&self.payload)?;
        if self.digest.0 == 0 {
            return Err(RealizationError::EmptyLexicalDigest);
        }
        if self.digest != expected {
            return Err(RealizationError::LexicalDigestMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RealizationError> {
        canonical_bytes(&self.payload)
    }
}

impl DeterministicRenderer {
    pub fn render(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Result<SurfaceRealization, RealizationError> {
        program.verify_replay_integrity()?;
        lexical_table.verify_integrity(program)?;

        let claim_map = program
            .payload
            .required_claims
            .iter()
            .chain(program.payload.optional_claims.iter())
            .map(|claim| (claim.id, claim))
            .collect::<BTreeMap<_, _>>();
        let claim_lexemes = lexical_table
            .payload
            .claims
            .iter()
            .map(|binding| (binding.claim, binding))
            .collect::<BTreeMap<_, _>>();
        let observation_labels = lexical_table
            .payload
            .observations
            .iter()
            .map(|binding| (binding.observation, binding.label.as_str()))
            .collect::<BTreeMap<_, _>>();
        let variable_labels = lexical_table
            .payload
            .missing_variables
            .iter()
            .map(|binding| (binding.variable, binding.label.as_str()))
            .collect::<BTreeMap<_, _>>();
        let prediction_labels = lexical_table
            .payload
            .predictions
            .iter()
            .map(|binding| (binding.prediction, binding.label.as_str()))
            .collect::<BTreeMap<_, _>>();

        let mut segments = Vec::with_capacity(program.payload.operations.len());
        for operation in &program.payload.operations {
            let rendered = render_operation(
                operation,
                program,
                &claim_map,
                &claim_lexemes,
                &observation_labels,
                &variable_labels,
                &prediction_labels,
            )?;
            segments.push(rendered);
        }

        let (text, alignments, paragraph_count) = assemble_segments(program, &segments)?;
        reject_forbidden_text(&text, &lexical_table.payload.forbidden_surface_forms)?;

        let operation_cost = u32::try_from(program.payload.operations.len())
            .map_err(|_| RealizationError::BudgetExceeded)?;
        let claim_cost = u32::try_from(
            segments
                .iter()
                .map(|segment| segment.claim_ids.len())
                .sum::<usize>(),
        )
        .map_err(|_| RealizationError::BudgetExceeded)?;
        let verification_step_cost = operation_cost
            .checked_add(claim_cost)
            .and_then(|cost| cost.checked_add(operation_cost))
            .ok_or(RealizationError::BudgetExceeded)?;
        let character_cost = u32::try_from(text.len()).map_err(|_| RealizationError::BudgetExceeded)?;
        let sentence_count = count_sentences(&text)?;

        validate_realization_budgets(
            program,
            operation_cost,
            claim_cost,
            verification_step_cost,
            character_cost,
            sentence_count,
            paragraph_count,
        )?;

        let payload = SurfaceRealizationPayload {
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            renderer: RendererIdentity::DeterministicReferenceV1,
            grammar_version: GRAMMAR_VERSION,
            text,
            alignments,
            operation_cost,
            claim_cost,
            verification_step_cost,
            character_cost,
            sentence_count,
            paragraph_count,
        };
        validate_surface_payload(&payload, program, lexical_table)?;
        let digest = digest_realization_payload(&payload)?;
        if digest.0 == 0 {
            return Err(RealizationError::EmptyRealizationDigest);
        }
        Ok(SurfaceRealization { payload, digest })
    }
}

impl SurfaceRealization {
    pub fn verify_integrity(
        &self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Result<(), RealizationError> {
        program.verify_replay_integrity()?;
        lexical_table.verify_integrity(program)?;
        validate_surface_payload(&self.payload, program, lexical_table)?;
        let expected = digest_realization_payload(&self.payload)?;
        if self.digest.0 == 0 {
            return Err(RealizationError::EmptyRealizationDigest);
        }
        if self.digest != expected {
            return Err(RealizationError::RealizationDigestMismatch);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RealizationError> {
        canonical_bytes(&self.payload)
    }
}

#[derive(Debug, Clone)]
struct RenderedSegment {
    operation: OperationId,
    text: String,
    claim_ids: Vec<ClaimId>,
    references: Vec<SurfaceReference>,
    semantic_markers: Vec<(ClaimId, EpistemicStatus, ClaimPolarity)>,
}

fn render_operation(
    operation: &crate::semantic_response::DiscourseOperation,
    program: &SemanticResponseProgram,
    claim_map: &BTreeMap<ClaimId, &AuthorizedClaim>,
    claim_lexemes: &BTreeMap<ClaimId, &ClaimLexicalBinding>,
    observation_labels: &BTreeMap<ObservationId, &str>,
    variable_labels: &BTreeMap<MissingVariableId, &str>,
    prediction_labels: &BTreeMap<PredictionId, &str>,
) -> Result<RenderedSegment, RealizationError> {
    let mut claim_ids = Vec::new();
    let mut references = Vec::new();
    let mut semantic_markers = Vec::new();

    let text = match &operation.kind {
        DiscourseOperationKind::Assert(claim_id) => {
            claim_ids.push(*claim_id);
            let claim = claim_map
                .get(claim_id)
                .copied()
                .ok_or(RealizationError::LexicalCoverageMismatch)?;
            semantic_markers.push((claim.id, claim.epistemic_status, claim.polarity));
            format!(
                "{}.",
                render_claim(claim, claim_lexemes.get(claim_id).copied())?
            )
        }
        DiscourseOperationKind::Qualify { claim, status } => {
            claim_ids.push(*claim);
            let authorized = claim_map
                .get(claim)
                .copied()
                .ok_or(RealizationError::LexicalCoverageMismatch)?;
            if authorized.epistemic_status != *status {
                return Err(RealizationError::SemanticMarkerMismatch);
            }
            semantic_markers.push((authorized.id, *status, authorized.polarity));
            format!(
                "{}.",
                render_claim_with_status(
                    authorized,
                    *status,
                    claim_lexemes.get(claim).copied(),
                )?
            )
        }
        DiscourseOperationKind::Contrast { left, right } => {
            claim_ids.extend([*left, *right]);
            let left_claim = claim_map
                .get(left)
                .copied()
                .ok_or(RealizationError::LexicalCoverageMismatch)?;
            let right_claim = claim_map
                .get(right)
                .copied()
                .ok_or(RealizationError::LexicalCoverageMismatch)?;
            semantic_markers.extend([
                (left_claim.id, left_claim.epistemic_status, left_claim.polarity),
                (
                    right_claim.id,
                    right_claim.epistemic_status,
                    right_claim.polarity,
                ),
            ]);
            format!(
                "On one side, {}. By contrast, {}.",
                render_claim(left_claim, claim_lexemes.get(left).copied())?,
                render_claim(right_claim, claim_lexemes.get(right).copied())?
            )
        }
        DiscourseOperationKind::Correct { prior, replacement } => {
            claim_ids.extend([*prior, *replacement]);
            let prior_claim = claim_map
                .get(prior)
                .copied()
                .ok_or(RealizationError::LexicalCoverageMismatch)?;
            let replacement_claim = claim_map
                .get(replacement)
                .copied()
                .ok_or(RealizationError::LexicalCoverageMismatch)?;
            semantic_markers.extend([
                (
                    prior_claim.id,
                    prior_claim.epistemic_status,
                    prior_claim.polarity,
                ),
                (
                    replacement_claim.id,
                    replacement_claim.epistemic_status,
                    replacement_claim.polarity,
                ),
            ]);
            format!(
                "Correction: {}; instead, {}.",
                render_claim(prior_claim, claim_lexemes.get(prior).copied())?,
                render_claim(
                    replacement_claim,
                    claim_lexemes.get(replacement).copied(),
                )?
            )
        }
        DiscourseOperationKind::Explain { claims } => {
            claim_ids.extend(claims.iter().copied());
            let mut rendered = Vec::with_capacity(claims.len());
            for claim_id in claims {
                let claim = claim_map
                    .get(claim_id)
                    .copied()
                    .ok_or(RealizationError::LexicalCoverageMismatch)?;
                semantic_markers.push((claim.id, claim.epistemic_status, claim.polarity));
                rendered.push(render_claim(
                    claim,
                    claim_lexemes.get(claim_id).copied(),
                )?);
            }
            format!("Relevant support: {}.", rendered.join("; "))
        }
        DiscourseOperationKind::Acknowledge(observation) => {
            references.push(SurfaceReference::Observation(*observation));
            let label = observation_labels
                .get(observation)
                .copied()
                .ok_or(RealizationError::LexicalCoverageMismatch)?;
            format!("I acknowledge {}.", label)
        }
        DiscourseOperationKind::RequestEvidence(variable) => {
            references.push(SurfaceReference::MissingVariable(*variable));
            let label = variable_labels
                .get(variable)
                .copied()
                .ok_or(RealizationError::LexicalCoverageMismatch)?;
            if program.payload.style.allow_questions {
                format!("What evidence resolves {}?", label)
            } else {
                format!("Evidence is required for {}.", label)
            }
        }
        DiscourseOperationKind::Commit(prediction) => {
            references.push(SurfaceReference::Prediction(*prediction));
            let label = prediction_labels
                .get(prediction)
                .copied()
                .ok_or(RealizationError::LexicalCoverageMismatch)?;
            format!("I commit to track {}.", label)
        }
        DiscourseOperationKind::Abstain(reason) => abstention_text(*reason).to_owned(),
    };

    Ok(RenderedSegment {
        operation: operation.id,
        text,
        claim_ids,
        references,
        semantic_markers,
    })
}

fn render_claim(
    claim: &AuthorizedClaim,
    binding: Option<&ClaimLexicalBinding>,
) -> Result<String, RealizationError> {
    render_claim_with_status(claim, claim.epistemic_status, binding)
}

fn render_claim_with_status(
    claim: &AuthorizedClaim,
    status: EpistemicStatus,
    binding: Option<&ClaimLexicalBinding>,
) -> Result<String, RealizationError> {
    let binding = binding.ok_or(RealizationError::LexicalCoverageMismatch)?;
    let clause = match claim.polarity {
        ClaimPolarity::Positive => &binding.positive_clause,
        ClaimPolarity::Negative => &binding.negative_clause,
    };
    Ok(format!("{} {}", epistemic_marker(status), clause))
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

fn assemble_segments(
    program: &SemanticResponseProgram,
    segments: &[RenderedSegment],
) -> Result<(String, Vec<SemanticAlignment>, u16), RealizationError> {
    if segments.len() != program.payload.operations.len() {
        return Err(RealizationError::OperationCoverageMismatch);
    }

    let max_paragraphs = usize::from(program.payload.style.maximum_paragraphs);
    if max_paragraphs == 0 {
        return Err(RealizationError::BudgetExceeded);
    }
    let target_paragraphs = match program.payload.style.detail {
        DetailLevel::Detailed => segments.len().min(max_paragraphs),
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
        alignments.push(SemanticAlignment {
            operation: segment.operation,
            span: TextSpan {
                start_byte,
                end_byte,
            },
            claim_ids: segment.claim_ids.clone(),
            references: segment.references.clone(),
        });
    }

    let paragraph_count = if text.is_empty() {
        0
    } else {
        u16::try_from(text.split("\n\n").count())
            .map_err(|_| RealizationError::BudgetExceeded)?
    };
    Ok((text, alignments, paragraph_count))
}

fn validate_lexical_payload(
    payload: &LexicalBindingTablePayload,
    program: &SemanticResponseProgram,
) -> Result<(), RealizationError> {
    if payload.program_digest != program.digest {
        return Err(RealizationError::ProgramDigestMismatch);
    }
    if payload.subject_scope != program.payload.subject_scope || payload.subject_scope.0 == 0 {
        return Err(RealizationError::SubjectScopeMismatch);
    }

    validate_claim_binding_order(&payload.claims)?;
    validate_observation_binding_order(&payload.observations)?;
    validate_variable_binding_order(&payload.missing_variables)?;
    validate_prediction_binding_order(&payload.predictions)?;
    validate_forbidden_forms(&payload.forbidden_surface_forms)?;

    let expected = expected_lexical_references(program);
    let actual_claims = payload
        .claims
        .iter()
        .map(|binding| binding.claim)
        .collect::<BTreeSet<_>>();
    let actual_observations = payload
        .observations
        .iter()
        .map(|binding| binding.observation)
        .collect::<BTreeSet<_>>();
    let actual_variables = payload
        .missing_variables
        .iter()
        .map(|binding| binding.variable)
        .collect::<BTreeSet<_>>();
    let actual_predictions = payload
        .predictions
        .iter()
        .map(|binding| binding.prediction)
        .collect::<BTreeSet<_>>();
    if actual_claims != expected.claims
        || actual_observations != expected.observations
        || actual_variables != expected.variables
        || actual_predictions != expected.predictions
    {
        return Err(RealizationError::LexicalCoverageMismatch);
    }

    let prohibited = program
        .payload
        .prohibited_claims
        .iter()
        .map(|claim| claim.id)
        .collect::<BTreeSet<_>>();
    if actual_claims.iter().any(|claim| prohibited.contains(claim)) {
        return Err(RealizationError::ProhibitedClaimBinding);
    }

    for binding in &payload.claims {
        reject_forbidden_text(
            &binding.positive_clause,
            &payload.forbidden_surface_forms,
        )?;
        reject_forbidden_text(
            &binding.negative_clause,
            &payload.forbidden_surface_forms,
        )?;
    }
    for label in payload
        .observations
        .iter()
        .map(|binding| binding.label.as_str())
        .chain(
            payload
                .missing_variables
                .iter()
                .map(|binding| binding.label.as_str()),
        )
        .chain(
            payload
                .predictions
                .iter()
                .map(|binding| binding.label.as_str()),
        )
    {
        reject_forbidden_text(label, &payload.forbidden_surface_forms)?;
    }
    Ok(())
}

#[derive(Debug, Default)]
struct ExpectedLexicalReferences {
    claims: BTreeSet<ClaimId>,
    observations: BTreeSet<ObservationId>,
    variables: BTreeSet<MissingVariableId>,
    predictions: BTreeSet<PredictionId>,
}

fn expected_lexical_references(program: &SemanticResponseProgram) -> ExpectedLexicalReferences {
    let mut expected = ExpectedLexicalReferences::default();
    for operation in &program.payload.operations {
        match &operation.kind {
            DiscourseOperationKind::Assert(claim)
            | DiscourseOperationKind::Qualify { claim, .. } => {
                expected.claims.insert(*claim);
            }
            DiscourseOperationKind::Contrast { left, right }
            | DiscourseOperationKind::Correct {
                prior: left,
                replacement: right,
            } => {
                expected.claims.extend([*left, *right]);
            }
            DiscourseOperationKind::Explain { claims } => {
                expected.claims.extend(claims.iter().copied());
            }
            DiscourseOperationKind::Acknowledge(observation) => {
                expected.observations.insert(*observation);
            }
            DiscourseOperationKind::RequestEvidence(variable) => {
                expected.variables.insert(*variable);
            }
            DiscourseOperationKind::Commit(prediction) => {
                expected.predictions.insert(*prediction);
            }
            DiscourseOperationKind::Abstain(_) => {}
        }
    }
    expected
}

fn validate_claim_binding_order(
    bindings: &[ClaimLexicalBinding],
) -> Result<(), RealizationError> {
    let mut previous = 0_u64;
    for binding in bindings {
        if binding.claim.0 == 0 || binding.claim.0 <= previous {
            return Err(RealizationError::NoncanonicalLexicalOrder);
        }
        validate_lexical_text(&binding.positive_clause, MAX_LEXICAL_TEXT_BYTES)?;
        validate_lexical_text(&binding.negative_clause, MAX_LEXICAL_TEXT_BYTES)?;
        if binding.positive_clause == binding.negative_clause {
            return Err(RealizationError::InvalidLexicalText);
        }
        previous = binding.claim.0;
    }
    Ok(())
}

fn validate_observation_binding_order(
    bindings: &[ObservationLexicalBinding],
) -> Result<(), RealizationError> {
    let mut previous = 0_u64;
    for binding in bindings {
        if binding.observation.0 == 0 || binding.observation.0 <= previous {
            return Err(RealizationError::NoncanonicalLexicalOrder);
        }
        validate_lexical_text(&binding.label, MAX_LEXICAL_TEXT_BYTES)?;
        previous = binding.observation.0;
    }
    Ok(())
}

fn validate_variable_binding_order(
    bindings: &[MissingVariableLexicalBinding],
) -> Result<(), RealizationError> {
    let mut previous = 0_u64;
    for binding in bindings {
        if binding.variable.0 == 0 || binding.variable.0 <= previous {
            return Err(RealizationError::NoncanonicalLexicalOrder);
        }
        validate_lexical_text(&binding.label, MAX_LEXICAL_TEXT_BYTES)?;
        previous = binding.variable.0;
    }
    Ok(())
}

fn validate_prediction_binding_order(
    bindings: &[PredictionLexicalBinding],
) -> Result<(), RealizationError> {
    let mut previous = 0_u64;
    for binding in bindings {
        if binding.prediction.0 == 0 || binding.prediction.0 <= previous {
            return Err(RealizationError::NoncanonicalLexicalOrder);
        }
        validate_lexical_text(&binding.label, MAX_LEXICAL_TEXT_BYTES)?;
        previous = binding.prediction.0;
    }
    Ok(())
}

fn validate_forbidden_forms(forms: &[String]) -> Result<(), RealizationError> {
    let mut previous: Option<&str> = None;
    for form in forms {
        validate_lexical_text(form, MAX_FORBIDDEN_FORM_BYTES)
            .map_err(|_| RealizationError::InvalidForbiddenForms)?;
        if let Some(previous) = previous {
            if previous >= form.as_str() {
                return Err(RealizationError::InvalidForbiddenForms);
            }
        }
        previous = Some(form);
    }
    Ok(())
}

fn validate_lexical_text(text: &str, maximum_bytes: usize) -> Result<(), RealizationError> {
    if text.is_empty()
        || text.len() > maximum_bytes
        || text.trim() != text
        || text.contains("  ")
        || text.chars().any(char::is_control)
    {
        return Err(RealizationError::InvalidLexicalText);
    }
    Ok(())
}

fn reject_forbidden_text(text: &str, forbidden_forms: &[String]) -> Result<(), RealizationError> {
    let normalized = text.to_lowercase();
    if forbidden_forms
        .iter()
        .any(|form| normalized.contains(&form.to_lowercase()))
    {
        return Err(RealizationError::ForbiddenSurfaceForm);
    }
    Ok(())
}

fn validate_surface_payload(
    payload: &SurfaceRealizationPayload,
    program: &SemanticResponseProgram,
    lexical_table: &LexicalBindingTable,
) -> Result<(), RealizationError> {
    if payload.program_digest != program.digest {
        return Err(RealizationError::ProgramDigestMismatch);
    }
    if payload.lexical_table_digest != lexical_table.digest
        || payload.renderer != RendererIdentity::DeterministicReferenceV1
        || payload.grammar_version != GRAMMAR_VERSION
    {
        return Err(RealizationError::LexicalDigestMismatch);
    }
    reject_forbidden_text(
        &payload.text,
        &lexical_table.payload.forbidden_surface_forms,
    )?;
    validate_alignments(payload, program)?;
    validate_realization_budgets(
        program,
        payload.operation_cost,
        payload.claim_cost,
        payload.verification_step_cost,
        payload.character_cost,
        payload.sentence_count,
        payload.paragraph_count,
    )?;
    validate_semantic_markers(payload, program, lexical_table)?;
    Ok(())
}

fn validate_alignments(
    payload: &SurfaceRealizationPayload,
    program: &SemanticResponseProgram,
) -> Result<(), RealizationError> {
    if payload.alignments.len() != program.payload.operations.len() {
        return Err(RealizationError::OperationCoverageMismatch);
    }
    let mut previous_end = 0_usize;
    for (index, alignment) in payload.alignments.iter().enumerate() {
        let expected_operation = OperationId(index as u64 + 1);
        if alignment.operation != expected_operation
            || alignment.span.start_byte < previous_end
            || alignment.span.start_byte >= alignment.span.end_byte
            || alignment.span.end_byte > payload.text.len()
            || !payload.text.is_char_boundary(alignment.span.start_byte)
            || !payload.text.is_char_boundary(alignment.span.end_byte)
        {
            return Err(RealizationError::InvalidAlignment);
        }
        let operation = &program.payload.operations[index];
        if operation.id != alignment.operation
            || expected_claim_ids(&operation.kind) != alignment.claim_ids
            || expected_surface_references(&operation.kind) != alignment.references
        {
            return Err(RealizationError::OperationCoverageMismatch);
        }
        previous_end = alignment.span.end_byte;
    }
    Ok(())
}

fn expected_claim_ids(operation: &DiscourseOperationKind) -> Vec<ClaimId> {
    match operation {
        DiscourseOperationKind::Assert(claim)
        | DiscourseOperationKind::Qualify { claim, .. } => vec![*claim],
        DiscourseOperationKind::Contrast { left, right } => vec![*left, *right],
        DiscourseOperationKind::Correct { prior, replacement } => vec![*prior, *replacement],
        DiscourseOperationKind::Explain { claims } => claims.clone(),
        DiscourseOperationKind::Acknowledge(_)
        | DiscourseOperationKind::RequestEvidence(_)
        | DiscourseOperationKind::Commit(_)
        | DiscourseOperationKind::Abstain(_) => Vec::new(),
    }
}

fn expected_surface_references(operation: &DiscourseOperationKind) -> Vec<SurfaceReference> {
    match operation {
        DiscourseOperationKind::Acknowledge(observation) => {
            vec![SurfaceReference::Observation(*observation)]
        }
        DiscourseOperationKind::RequestEvidence(variable) => {
            vec![SurfaceReference::MissingVariable(*variable)]
        }
        DiscourseOperationKind::Commit(prediction) => {
            vec![SurfaceReference::Prediction(*prediction)]
        }
        DiscourseOperationKind::Assert(_)
        | DiscourseOperationKind::Qualify { .. }
        | DiscourseOperationKind::Contrast { .. }
        | DiscourseOperationKind::Correct { .. }
        | DiscourseOperationKind::Explain { .. }
        | DiscourseOperationKind::Abstain(_) => Vec::new(),
    }
}

fn validate_semantic_markers(
    payload: &SurfaceRealizationPayload,
    program: &SemanticResponseProgram,
    lexical_table: &LexicalBindingTable,
) -> Result<(), RealizationError> {
    let claim_map = program
        .payload
        .required_claims
        .iter()
        .chain(program.payload.optional_claims.iter())
        .map(|claim| (claim.id, claim))
        .collect::<BTreeMap<_, _>>();
    let lexical_map = lexical_table
        .payload
        .claims
        .iter()
        .map(|binding| (binding.claim, binding))
        .collect::<BTreeMap<_, _>>();

    for alignment in &payload.alignments {
        let segment = &payload.text[alignment.span.start_byte..alignment.span.end_byte];
        for claim_id in &alignment.claim_ids {
            let claim = claim_map
                .get(claim_id)
                .copied()
                .ok_or(RealizationError::SemanticMarkerMismatch)?;
            let lexical = lexical_map
                .get(claim_id)
                .copied()
                .ok_or(RealizationError::SemanticMarkerMismatch)?;
            let clause = match claim.polarity {
                ClaimPolarity::Positive => &lexical.positive_clause,
                ClaimPolarity::Negative => &lexical.negative_clause,
            };
            if !segment.contains(epistemic_marker(claim.epistemic_status))
                || !segment.contains(clause)
            {
                return Err(RealizationError::SemanticMarkerMismatch);
            }
        }
    }
    Ok(())
}

fn validate_realization_budgets(
    program: &SemanticResponseProgram,
    operation_cost: u32,
    claim_cost: u32,
    verification_step_cost: u32,
    character_cost: u32,
    sentence_count: u16,
    paragraph_count: u16,
) -> Result<(), RealizationError> {
    if operation_cost > u32::from(program.payload.compute_budget.maximum_operations)
        || claim_cost > u32::from(program.payload.compute_budget.maximum_claims)
        || verification_step_cost > program.payload.compute_budget.maximum_verification_steps
        || character_cost > program.payload.output_budget.maximum_characters
        || sentence_count > program.payload.output_budget.maximum_sentences
        || paragraph_count > program.payload.style.maximum_paragraphs
        || paragraph_count == 0
    {
        return Err(RealizationError::BudgetExceeded);
    }
    Ok(())
}

fn count_sentences(text: &str) -> Result<u16, RealizationError> {
    let count = text
        .chars()
        .filter(|character| matches!(character, '.' | '?' | '!'))
        .count();
    if count == 0 {
        return Err(RealizationError::BudgetExceeded);
    }
    u16::try_from(count).map_err(|_| RealizationError::BudgetExceeded)
}

fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, RealizationError> {
    serde_json::to_vec(value)
        .map_err(|error| RealizationError::CanonicalSerialization(error.to_string()))
}

fn digest_lexical_payload(
    payload: &LexicalBindingTablePayload,
) -> Result<LexicalTableDigest, RealizationError> {
    let encoded = canonical_bytes(payload)?;
    Ok(LexicalTableDigest(domain_digest(
        LEXICAL_DIGEST_DOMAIN,
        &encoded,
    )))
}

fn digest_realization_payload(
    payload: &SurfaceRealizationPayload,
) -> Result<RealizationDigest, RealizationError> {
    let encoded = canonical_bytes(payload)?;
    Ok(RealizationDigest(domain_digest(
        REALIZATION_DIGEST_DOMAIN,
        &encoded,
    )))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_response::{
        AcknowledgmentLevel, ClaimPolarity, CompanionStateVersion, ComputeBudget,
        CognitiveStateVersion, DiscourseOperation, EpistemicConstraint, LexicalTableDigest as _,
        OutputBudget, ProhibitedClaim, ResponseProgramId, SemanticResponseIntent,
        SemanticResponseProgramPayload, SemanticValidationContext, SensitivityLevel,
        SensitivityPolicy, StyleEnvelope, VocabularyLevel,
    };

    fn claim(
        id: u64,
        key: &str,
        polarity: ClaimPolarity,
        confidence_bps: u16,
        status: EpistemicStatus,
    ) -> AuthorizedClaim {
        AuthorizedClaim {
            id: ClaimId(id),
            semantic_key: key.to_owned(),
            polarity,
            confidence_bps,
            epistemic_status: status,
            sensitivity: SensitivityLevel::Public,
            disclosure_scope: SubjectScope(77),
        }
    }

    fn program(detail: DetailLevel, max_chars: u32) -> SemanticResponseProgram {
        let required_claims = vec![
            claim(
                1,
                "wrapper_risk_valid",
                ClaimPolarity::Positive,
                9_500,
                EpistemicStatus::Certain,
            ),
            claim(
                2,
                "renderer_does_not_own_cognition",
                ClaimPolarity::Negative,
                8_200,
                EpistemicStatus::Probable,
            ),
            claim(
                3,
                "semantic_boundary_preserves_authority",
                ClaimPolarity::Positive,
                7_700,
                EpistemicStatus::Probable,
            ),
            claim(
                4,
                "current_capability_is_limited",
                ClaimPolarity::Positive,
                9_100,
                EpistemicStatus::Certain,
            ),
        ];
        let optional_claims = vec![
            claim(
                5,
                "prior_design_was_wrapper_like",
                ClaimPolarity::Positive,
                5_000,
                EpistemicStatus::Possible,
            ),
            claim(
                6,
                "reference_renderer_is_bounded",
                ClaimPolarity::Positive,
                7_400,
                EpistemicStatus::Probable,
            ),
            claim(
                7,
                "alignment_enables_verification",
                ClaimPolarity::Positive,
                2_500,
                EpistemicStatus::Uncertain,
            ),
        ];
        let epistemic_constraints = required_claims
            .iter()
            .chain(optional_claims.iter())
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
        let payload = SemanticResponseProgramPayload {
            id: ResponseProgramId(1),
            source_state_version: CognitiveStateVersion(41),
            companion_state_version: Some(CompanionStateVersion(12)),
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
                    kind: DiscourseOperationKind::Qualify {
                        claim: ClaimId(4),
                        status: EpistemicStatus::Certain,
                    },
                },
                DiscourseOperation {
                    id: OperationId(4),
                    kind: DiscourseOperationKind::Contrast {
                        left: ClaimId(2),
                        right: ClaimId(3),
                    },
                },
                DiscourseOperation {
                    id: OperationId(5),
                    kind: DiscourseOperationKind::Correct {
                        prior: ClaimId(5),
                        replacement: ClaimId(6),
                    },
                },
                DiscourseOperation {
                    id: OperationId(6),
                    kind: DiscourseOperationKind::Explain {
                        claims: vec![ClaimId(2), ClaimId(3), ClaimId(7)],
                    },
                },
                DiscourseOperation {
                    id: OperationId(7),
                    kind: DiscourseOperationKind::RequestEvidence(MissingVariableId(33)),
                },
                DiscourseOperation {
                    id: OperationId(8),
                    kind: DiscourseOperationKind::Commit(PredictionId(44)),
                },
                DiscourseOperation {
                    id: OperationId(9),
                    kind: DiscourseOperationKind::Abstain(
                        AbstentionReason::InsufficientEvidence,
                    ),
                },
            ],
            required_claims,
            optional_claims,
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
                detail,
                vocabulary: VocabularyLevel::Technical,
                dialogue: DialogueMode::Collaborative,
                acknowledgment: AcknowledgmentLevel::Brief,
                allow_first_person: true,
                allow_questions: true,
                maximum_paragraphs: 9,
            },
            output_budget: OutputBudget {
                maximum_characters: max_chars,
                maximum_sentences: 32,
            },
            compute_budget: ComputeBudget {
                maximum_operations: 16,
                maximum_claims: 32,
                maximum_verification_steps: 128,
            },
        };
        SemanticResponseProgram::validate(
            payload,
            SemanticValidationContext {
                cognitive_state_version: CognitiveStateVersion(41),
                companion_state_version: Some(CompanionStateVersion(12)),
                subject_scope: SubjectScope(77),
            },
        )
        .unwrap()
    }

    fn lexical_payload(program: &SemanticResponseProgram) -> LexicalBindingTablePayload {
        LexicalBindingTablePayload {
            program_digest: program.digest,
            subject_scope: SubjectScope(77),
            claims: (1..=7)
                .map(|id| ClaimLexicalBinding {
                    claim: ClaimId(id),
                    positive_clause: format!("positive clause {id}"),
                    negative_clause: format!("negative clause {id}"),
                })
                .collect(),
            observations: vec![ObservationLexicalBinding {
                observation: ObservationId(901),
                label: "the architectural objection".to_owned(),
            }],
            missing_variables: vec![MissingVariableLexicalBinding {
                variable: MissingVariableId(33),
                label: "renderer substitution performance".to_owned(),
            }],
            predictions: vec![PredictionLexicalBinding {
                prediction: PredictionId(44),
                label: "the held-out attribution result".to_owned(),
            }],
            forbidden_surface_forms: vec!["fluency proves cognition".to_owned()],
        }
    }

    #[test]
    fn deterministic_realization_is_exact_and_verified() {
        let program = program(DetailLevel::Detailed, 4_000);
        let table = LexicalBindingTable::validate(lexical_payload(&program), &program).unwrap();
        let renderer = DeterministicRenderer;
        let first = renderer.render(&program, &table).unwrap();
        let second = renderer.render(&program, &table).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.payload.alignments.len(), 9);
        assert!(first.payload.text.contains("negative clause 2"));
        assert!(first.payload.text.contains("It is probable that"));
        first.verify_integrity(&program, &table).unwrap();
    }

    #[test]
    fn style_changes_layout_not_semantic_alignment() {
        let detailed = program(DetailLevel::Detailed, 4_000);
        let brief = program(DetailLevel::Brief, 4_000);
        let detailed_table =
            LexicalBindingTable::validate(lexical_payload(&detailed), &detailed).unwrap();
        let brief_table = LexicalBindingTable::validate(lexical_payload(&brief), &brief).unwrap();
        let renderer = DeterministicRenderer;
        let detailed_output = renderer.render(&detailed, &detailed_table).unwrap();
        let brief_output = renderer.render(&brief, &brief_table).unwrap();
        assert_ne!(detailed_output.payload.text, brief_output.payload.text);
        assert_eq!(
            detailed_output
                .payload
                .alignments
                .iter()
                .map(|alignment| (&alignment.operation, &alignment.claim_ids))
                .collect::<Vec<_>>(),
            brief_output
                .payload
                .alignments
                .iter()
                .map(|alignment| (&alignment.operation, &alignment.claim_ids))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn lexical_mismatch_and_forbidden_forms_fail_closed() {
        let program = program(DetailLevel::Standard, 4_000);
        let mut payload = lexical_payload(&program);
        payload.claims.pop();
        assert_eq!(
            LexicalBindingTable::validate(payload, &program).unwrap_err(),
            RealizationError::LexicalCoverageMismatch
        );

        let mut payload = lexical_payload(&program);
        payload.claims[0].positive_clause = "fluency proves cognition".to_owned();
        assert_eq!(
            LexicalBindingTable::validate(payload, &program).unwrap_err(),
            RealizationError::ForbiddenSurfaceForm
        );
    }

    #[test]
    fn output_budget_is_not_silently_truncated() {
        let program = program(DetailLevel::Standard, 64);
        let table = LexicalBindingTable::validate(lexical_payload(&program), &program).unwrap();
        assert_eq!(
            DeterministicRenderer.render(&program, &table).unwrap_err(),
            RealizationError::BudgetExceeded
        );
    }

    #[test]
    fn digest_and_alignment_tampering_are_rejected() {
        let program = program(DetailLevel::Standard, 4_000);
        let table = LexicalBindingTable::validate(lexical_payload(&program), &program).unwrap();
        let mut output = DeterministicRenderer.render(&program, &table).unwrap();
        output.digest.0 ^= 1;
        assert_eq!(
            output.verify_integrity(&program, &table).unwrap_err(),
            RealizationError::RealizationDigestMismatch
        );

        let mut output = DeterministicRenderer.render(&program, &table).unwrap();
        output.payload.alignments[1].span.start_byte = 0;
        assert_eq!(
            output.verify_integrity(&program, &table).unwrap_err(),
            RealizationError::InvalidAlignment
        );
    }

    #[test]
    fn every_renderer_authority_flag_is_closed() {
        let boundary = authority_boundary();
        assert!(!boundary.runtime_chat_wiring);
        assert!(!boundary.live_generated_text_influence);
        assert!(!boundary.raw_conversation_access);
        assert!(!boundary.unrestricted_memory_access);
        assert!(!boundary.persistence_authority);
        assert!(!boundary.routing_authority);
        assert!(!boundary.companion_mutation_authority);
        assert!(!boundary.belief_promotion_authority);
        assert!(!boundary.ontology_promotion_authority);
        assert!(!boundary.tool_selection_authority);
        assert!(!boundary.charge_discharge_authority);
        assert!(!boundary.autonomous_action_authority);
    }
}
