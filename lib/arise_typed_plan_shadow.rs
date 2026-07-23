//! ARISE-A1: typed semantic-program shadow bridge.
//!
//! This adapter consumes Starfire's existing validated `SemanticResponseProgram`
//! and scope-bound `LexicalBindingTable`. ARISE plans the authorized discourse
//! operations terminal-first, while the existing independent language verifier
//! reconstructs those operations from final text without trusting renderer
//! alignments. Results are observation-only and cannot alter returned text.

use crate::arise_edge::{
    AriseConfig, AriseEngine, LexicalSpanRenderer, LexicalTransitionVerifier, ObligationId,
    SemanticObligation,
};
use crate::language_realization::LexicalBindingTable;
use crate::language_verification::{
    IndependentLanguageVerifier, LanguageVerificationError, LanguageVerificationInput,
};
use crate::semantic_response::{DiscourseOperationKind, SemanticResponseProgram};
use crate::verifier_ready_realization::VERIFIER_READY_GRAMMAR_VERSION;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::{Mutex, OnceLock};

const PIPELINE: &str = "arise-a1-typed-program-shadow-v1";
const MAX_OPERATIONS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypedPlanTerminalClassification {
    Pass,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypedPlanRejectionReason {
    None,
    NotObserved,
    EmptyProgram,
    OperationCapacityExceeded,
    OperationIdentifierOutOfRange,
    InvalidAuthorizationPacket,
    ArisePlanningRejected,
    EmptyText,
    UnsupportedSurface,
    OperationMismatch,
    ForbiddenSurface,
    BudgetExceeded,
    VerifierRejected,
    ReconstructionOrderMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedPlanAuthorityBoundary {
    pub semantic_program_read: bool,
    pub lexical_table_read: bool,
    pub final_text_observation: bool,
    pub generated_text_influence: bool,
    pub raw_prompt_access: bool,
    pub memory_access: bool,
    pub persistence_authority: bool,
    pub routing_authority: bool,
    pub belief_promotion_authority: bool,
    pub ontology_promotion_authority: bool,
    pub tool_selection_authority: bool,
    pub charge_discharge_authority: bool,
    pub autonomous_action_authority: bool,
}

#[must_use]
pub const fn authority_boundary() -> TypedPlanAuthorityBoundary {
    TypedPlanAuthorityBoundary {
        semantic_program_read: true,
        lexical_table_read: true,
        final_text_observation: true,
        generated_text_influence: false,
        raw_prompt_access: false,
        memory_access: false,
        persistence_authority: false,
        routing_authority: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        tool_selection_authority: false,
        charge_discharge_authority: false,
        autonomous_action_authority: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AriseTypedPlanSnapshot {
    pub enabled: bool,
    pub pipeline: String,
    pub program_digest: u64,
    pub lexical_table_digest: u64,
    pub subject_scope: u64,
    pub body_digest: u64,
    pub operation_count: usize,
    pub required_claim_count: usize,
    pub reconstructed_operation_count: usize,
    pub initial_residual: usize,
    pub final_residual: usize,
    pub terminal_classification: TypedPlanTerminalClassification,
    pub rejection_reason: TypedPlanRejectionReason,
    pub authority: TypedPlanAuthorityBoundary,
}

impl Default for AriseTypedPlanSnapshot {
    fn default() -> Self {
        Self {
            enabled: true,
            pipeline: PIPELINE.to_string(),
            program_digest: 0,
            lexical_table_digest: 0,
            subject_scope: 0,
            body_digest: 0,
            operation_count: 0,
            required_claim_count: 0,
            reconstructed_operation_count: 0,
            initial_residual: 0,
            final_residual: 0,
            terminal_classification: TypedPlanTerminalClassification::Rejected,
            rejection_reason: TypedPlanRejectionReason::NotObserved,
            authority: authority_boundary(),
        }
    }
}

static LAST_SNAPSHOT: OnceLock<Mutex<AriseTypedPlanSnapshot>> = OnceLock::new();

fn snapshot_store() -> &'static Mutex<AriseTypedPlanSnapshot> {
    LAST_SNAPSHOT.get_or_init(|| Mutex::new(AriseTypedPlanSnapshot::default()))
}

#[must_use]
pub fn live_typed_plan_snapshot() -> AriseTypedPlanSnapshot {
    snapshot_store()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

#[must_use]
pub fn observe_semantic_program(
    program: &SemanticResponseProgram,
    lexical_table: &LexicalBindingTable,
    final_text: &str,
) -> AriseTypedPlanSnapshot {
    let mut snapshot = base_snapshot(program, lexical_table, final_text);

    if program.verify_replay_integrity().is_err()
        || lexical_table.verify_integrity(program).is_err()
    {
        snapshot.rejection_reason = TypedPlanRejectionReason::InvalidAuthorizationPacket;
        return store(snapshot);
    }

    let request = match arise_request(program) {
        Ok(request) => request,
        Err(reason) => {
            snapshot.rejection_reason = reason;
            return store(snapshot);
        }
    };

    let config = AriseConfig {
        maximum_obligations: MAX_OPERATIONS,
        maximum_obligations_per_span: 4,
        maximum_span_bytes: 512,
        maximum_repair_depth: 5,
    };
    let engine = match AriseEngine::new(config, LexicalSpanRenderer, LexicalTransitionVerifier) {
        Ok(engine) => engine,
        Err(_) => {
            snapshot.rejection_reason = TypedPlanRejectionReason::ArisePlanningRejected;
            return store(snapshot);
        }
    };
    let plan = match engine.plan(&request) {
        Ok(plan) => plan,
        Err(_) => {
            snapshot.rejection_reason = TypedPlanRejectionReason::ArisePlanningRejected;
            return store(snapshot);
        }
    };
    snapshot.initial_residual = plan.initial_residual;
    snapshot.final_residual = plan.initial_residual;

    let report = match IndependentLanguageVerifier.verify(LanguageVerificationInput {
        program,
        lexical_table,
        program_digest: program.digest,
        lexical_table_digest: lexical_table.digest,
        subject_scope: program.payload.subject_scope,
        grammar_version: VERIFIER_READY_GRAMMAR_VERSION,
        text: final_text,
    }) {
        Ok(report) => report,
        Err(error) => {
            snapshot.rejection_reason = rejection_reason(&error);
            return store(snapshot);
        }
    };

    let reconstructed = report
        .payload
        .reconstructed_operations
        .iter()
        .map(|operation| operation.operation.0)
        .collect::<Vec<_>>();
    snapshot.reconstructed_operation_count = reconstructed.len();

    let expected = plan
        .ordered_obligations
        .iter()
        .map(|obligation| u64::from(obligation.0))
        .collect::<Vec<_>>();
    if reconstructed != expected {
        let matched_prefix = expected
            .iter()
            .zip(reconstructed.iter())
            .take_while(|(left, right)| left == right)
            .count();
        snapshot.rejection_reason = TypedPlanRejectionReason::ReconstructionOrderMismatch;
        snapshot.final_residual = expected.len().saturating_sub(matched_prefix);
        return store(snapshot);
    }

    let satisfied = reconstructed.iter().copied().collect::<BTreeSet<_>>();
    snapshot.final_residual = expected
        .iter()
        .filter(|operation| !satisfied.contains(operation))
        .count();
    if snapshot.final_residual == 0
        && snapshot.reconstructed_operation_count == snapshot.operation_count
    {
        snapshot.terminal_classification = TypedPlanTerminalClassification::Pass;
        snapshot.rejection_reason = TypedPlanRejectionReason::None;
    }

    store(snapshot)
}

fn base_snapshot(
    program: &SemanticResponseProgram,
    lexical_table: &LexicalBindingTable,
    final_text: &str,
) -> AriseTypedPlanSnapshot {
    AriseTypedPlanSnapshot {
        program_digest: program.digest.0,
        lexical_table_digest: lexical_table.digest.0,
        subject_scope: program.payload.subject_scope.0,
        body_digest: stable_digest(final_text),
        operation_count: program.payload.operations.len(),
        required_claim_count: program.payload.required_claims.len(),
        ..AriseTypedPlanSnapshot::default()
    }
}

fn store(snapshot: AriseTypedPlanSnapshot) -> AriseTypedPlanSnapshot {
    let mut stored = snapshot_store()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *stored = snapshot.clone();
    snapshot
}

fn arise_request(
    program: &SemanticResponseProgram,
) -> Result<crate::arise_edge::AriseRequest, TypedPlanRejectionReason> {
    let operations = &program.payload.operations;
    if operations.is_empty() {
        return Err(TypedPlanRejectionReason::EmptyProgram);
    }
    if operations.len() > MAX_OPERATIONS {
        return Err(TypedPlanRejectionReason::OperationCapacityExceeded);
    }

    let mut obligations = Vec::with_capacity(operations.len());
    for (index, operation) in operations.iter().enumerate() {
        let id = u16::try_from(operation.id.0)
            .ok()
            .filter(|id| *id != 0)
            .ok_or(TypedPlanRejectionReason::OperationIdentifierOutOfRange)?;
        let dependencies = if index == 0 {
            Vec::new()
        } else {
            let prior = u16::try_from(operations[index - 1].id.0)
                .ok()
                .filter(|prior| *prior != 0)
                .ok_or(TypedPlanRejectionReason::OperationIdentifierOutOfRange)?;
            vec![ObligationId(prior)]
        };
        let kind = operation_kind_label(&operation.kind);
        obligations.push(SemanticObligation {
            id: ObligationId(id),
            semantic_key: format!("operation.{id}.{kind}"),
            dependencies,
            witness: format!("Authorized operation {id} {kind}"),
        });
    }

    let terminal_obligations = obligations
        .last()
        .map(|obligation| obligation.id)
        .into_iter()
        .collect();
    Ok(crate::arise_edge::AriseRequest {
        trace_id: program.payload.id.0,
        intent_label: "typed_program_shadow".to_string(),
        terminal_obligations,
        initially_satisfied: Vec::new(),
        obligations,
        prohibited_fragments: Vec::new(),
    })
}

fn operation_kind_label(kind: &DiscourseOperationKind) -> &'static str {
    match kind {
        DiscourseOperationKind::Assert(_) => "assert",
        DiscourseOperationKind::Qualify { .. } => "qualify",
        DiscourseOperationKind::Contrast { .. } => "contrast",
        DiscourseOperationKind::Correct { .. } => "correct",
        DiscourseOperationKind::Explain { .. } => "explain",
        DiscourseOperationKind::Acknowledge(_) => "acknowledge",
        DiscourseOperationKind::RequestEvidence(_) => "request_evidence",
        DiscourseOperationKind::Commit(_) => "commit",
        DiscourseOperationKind::Abstain(_) => "abstain",
    }
}

fn rejection_reason(error: &LanguageVerificationError) -> TypedPlanRejectionReason {
    match error {
        LanguageVerificationError::EmptyText => TypedPlanRejectionReason::EmptyText,
        LanguageVerificationError::UnsupportedSurface
        | LanguageVerificationError::AmbiguousSurfaceBinding
        | LanguageVerificationError::CandidateBudgetExceeded => {
            TypedPlanRejectionReason::UnsupportedSurface
        }
        LanguageVerificationError::OperationMismatch => TypedPlanRejectionReason::OperationMismatch,
        LanguageVerificationError::ForbiddenSurfaceForm => {
            TypedPlanRejectionReason::ForbiddenSurface
        }
        LanguageVerificationError::BudgetExceeded => TypedPlanRejectionReason::BudgetExceeded,
        LanguageVerificationError::SemanticProgram(_)
        | LanguageVerificationError::LexicalTable(_)
        | LanguageVerificationError::ProgramDigestMismatch
        | LanguageVerificationError::LexicalDigestMismatch
        | LanguageVerificationError::SubjectScopeMismatch
        | LanguageVerificationError::GrammarVersionMismatch
        | LanguageVerificationError::CanonicalSerialization(_)
        | LanguageVerificationError::EmptyDigest
        | LanguageVerificationError::DigestMismatch => TypedPlanRejectionReason::VerifierRejected,
    }
}

fn stable_digest(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language_realization::{ClaimLexicalBinding, LexicalBindingTablePayload};
    use crate::semantic_response::{
        AcknowledgmentLevel, AuthorizedClaim, ClaimId, ClaimPolarity, CognitiveStateVersion,
        ComputeBudget, DetailLevel, DialogueMode, DiscourseOperation, EpistemicConstraint,
        EpistemicStatus, OperationId, OutputBudget, ResponseProgramId, SemanticResponseIntent,
        SemanticResponseProgramPayload, SemanticValidationContext, SensitivityLevel,
        SensitivityPolicy, StyleEnvelope, SubjectScope, VocabularyLevel,
    };
    use crate::verifier_ready_realization::VerifierReadyRenderer;

    const SUBJECT: SubjectScope = SubjectScope(19);
    const COGNITIVE_VERSION: CognitiveStateVersion = CognitiveStateVersion(3);

    fn claim(
        id: u64,
        key: &str,
        polarity: ClaimPolarity,
        status: EpistemicStatus,
    ) -> AuthorizedClaim {
        let (minimum, maximum) = status.confidence_bounds();
        AuthorizedClaim {
            id: ClaimId(id),
            semantic_key: key.to_string(),
            polarity,
            confidence_bps: if status == EpistemicStatus::Unknown {
                0
            } else {
                minimum + (maximum - minimum) / 2
            },
            epistemic_status: status,
            sensitivity: SensitivityLevel::Public,
            disclosure_scope: SUBJECT,
        }
    }

    fn fixture() -> (SemanticResponseProgram, LexicalBindingTable, String) {
        let claims = vec![
            claim(
                1,
                "bridge_is_shadow_only",
                ClaimPolarity::Positive,
                EpistemicStatus::Certain,
            ),
            claim(
                2,
                "bridge_controls_output",
                ClaimPolarity::Negative,
                EpistemicStatus::Probable,
            ),
        ];
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
            .collect();
        let program = SemanticResponseProgram::validate(
            SemanticResponseProgramPayload {
                id: ResponseProgramId(11),
                source_state_version: COGNITIVE_VERSION,
                companion_state_version: None,
                subject_scope: SUBJECT,
                intent: SemanticResponseIntent::Explanation,
                operations: vec![
                    DiscourseOperation {
                        id: OperationId(1),
                        kind: DiscourseOperationKind::Assert(ClaimId(1)),
                    },
                    DiscourseOperation {
                        id: OperationId(2),
                        kind: DiscourseOperationKind::Qualify {
                            claim: ClaimId(2),
                            status: EpistemicStatus::Probable,
                        },
                    },
                ],
                required_claims: claims,
                optional_claims: Vec::new(),
                prohibited_claims: Vec::new(),
                epistemic_constraints: constraints,
                sensitivity: SensitivityPolicy {
                    maximum_disclosure: SensitivityLevel::Public,
                    disclosure_scope: SUBJECT,
                },
                style: StyleEnvelope {
                    detail: DetailLevel::Standard,
                    vocabulary: VocabularyLevel::Plain,
                    dialogue: DialogueMode::Declarative,
                    acknowledgment: AcknowledgmentLevel::None,
                    allow_first_person: false,
                    allow_questions: false,
                    maximum_paragraphs: 2,
                },
                output_budget: OutputBudget {
                    maximum_characters: 1_000,
                    maximum_sentences: 4,
                },
                compute_budget: ComputeBudget {
                    maximum_operations: 4,
                    maximum_claims: 4,
                    maximum_verification_steps: 16,
                },
            },
            SemanticValidationContext {
                cognitive_state_version: COGNITIVE_VERSION,
                companion_state_version: None,
                subject_scope: SUBJECT,
            },
        )
        .expect("fixture program should validate");
        let lexical_table = LexicalBindingTable::validate(
            LexicalBindingTablePayload {
                program_digest: program.digest,
                subject_scope: SUBJECT,
                claims: vec![
                    ClaimLexicalBinding {
                        claim: ClaimId(1),
                        positive_clause: "the bridge is shadow only".to_string(),
                        negative_clause: "the bridge is not shadow only".to_string(),
                    },
                    ClaimLexicalBinding {
                        claim: ClaimId(2),
                        positive_clause: "the bridge controls output".to_string(),
                        negative_clause: "the bridge does not control output".to_string(),
                    },
                ],
                observations: Vec::new(),
                missing_variables: Vec::new(),
                predictions: Vec::new(),
                forbidden_surface_forms: vec!["forbidden authority".to_string()],
            },
            &program,
        )
        .expect("fixture lexical table should validate");
        let surface = VerifierReadyRenderer
            .render(&program, &lexical_table)
            .expect("fixture should render");
        (program, lexical_table, surface.payload.text)
    }

    #[test]
    fn typed_program_shadow_reconstructs_authorized_operations() {
        let (program, lexical_table, text) = fixture();
        let snapshot = observe_semantic_program(&program, &lexical_table, &text);
        assert_eq!(
            snapshot.terminal_classification,
            TypedPlanTerminalClassification::Pass
        );
        assert_eq!(snapshot.initial_residual, 2);
        assert_eq!(snapshot.final_residual, 0);
        assert_eq!(snapshot.reconstructed_operation_count, 2);
        assert!(!snapshot.authority.generated_text_influence);
        assert!(!snapshot.authority.persistence_authority);
        assert!(!snapshot.authority.charge_discharge_authority);
        assert!(!snapshot.authority.autonomous_action_authority);
    }

    #[test]
    fn typed_program_shadow_rejects_missing_operation() {
        let (program, lexical_table, text) = fixture();
        let first_sentence = text
            .split_once('.')
            .map(|(first, _)| format!("{first}."))
            .expect("fixture should contain multiple sentences");
        let snapshot = observe_semantic_program(&program, &lexical_table, &first_sentence);
        assert_eq!(
            snapshot.terminal_classification,
            TypedPlanTerminalClassification::Rejected
        );
        assert_ne!(snapshot.rejection_reason, TypedPlanRejectionReason::None);
        assert_eq!(snapshot.final_residual, snapshot.initial_residual);
    }

    #[test]
    fn typed_program_shadow_rejects_forbidden_surface() {
        let (program, lexical_table, text) = fixture();
        let candidate = format!("{text} forbidden authority");
        let snapshot = observe_semantic_program(&program, &lexical_table, &candidate);
        assert_eq!(
            snapshot.terminal_classification,
            TypedPlanTerminalClassification::Rejected
        );
        assert_eq!(
            snapshot.rejection_reason,
            TypedPlanRejectionReason::ForbiddenSurface
        );
    }
}
