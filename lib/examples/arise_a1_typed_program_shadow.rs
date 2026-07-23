use anyhow::{ensure, Context, Result};
use star::arise_edge::{
    live_typed_plan_snapshot, ResponseSemanticShadowExt, TypedPlanTerminalClassification,
};
use star::language_realization::{
    ClaimLexicalBinding, LexicalBindingTable, LexicalBindingTablePayload,
};
use star::runtime::response_intent::{Response, ResponseIntent};
use star::semantic_response::{
    AcknowledgmentLevel, AuthorizedClaim, ClaimId, ClaimPolarity, CognitiveStateVersion,
    ComputeBudget, DetailLevel, DialogueMode, DiscourseOperation, DiscourseOperationKind,
    EpistemicConstraint, EpistemicStatus, OperationId, OutputBudget, ResponseProgramId,
    SemanticResponseIntent, SemanticResponseProgram, SemanticResponseProgramPayload,
    SemanticValidationContext, SensitivityLevel, SensitivityPolicy, StyleEnvelope, SubjectScope,
    VocabularyLevel,
};
use star::verifier_ready_realization::VerifierReadyRenderer;

const SUBJECT: SubjectScope = SubjectScope(23);
const COGNITIVE_VERSION: CognitiveStateVersion = CognitiveStateVersion(5);

fn claim(
    id: u64,
    semantic_key: &str,
    polarity: ClaimPolarity,
    status: EpistemicStatus,
) -> AuthorizedClaim {
    let (minimum, maximum) = status.confidence_bounds();
    AuthorizedClaim {
        id: ClaimId(id),
        semantic_key: semantic_key.to_string(),
        polarity,
        confidence_bps: minimum + (maximum - minimum) / 2,
        epistemic_status: status,
        sensitivity: SensitivityLevel::Public,
        disclosure_scope: SUBJECT,
    }
}

fn main() -> Result<()> {
    let claims = vec![
        claim(
            1,
            "arise_reads_authorized_operations",
            ClaimPolarity::Positive,
            EpistemicStatus::Certain,
        ),
        claim(
            2,
            "arise_changes_returned_text",
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
            id: ResponseProgramId(17),
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
    .context("validate A1 semantic response program")?;

    let lexical_table = LexicalBindingTable::validate(
        LexicalBindingTablePayload {
            program_digest: program.digest,
            subject_scope: SUBJECT,
            claims: vec![
                ClaimLexicalBinding {
                    claim: ClaimId(1),
                    positive_clause: "ARISE reads authorized operations".to_string(),
                    negative_clause: "ARISE does not read authorized operations".to_string(),
                },
                ClaimLexicalBinding {
                    claim: ClaimId(2),
                    positive_clause: "ARISE changes returned text".to_string(),
                    negative_clause: "ARISE does not change returned text".to_string(),
                },
            ],
            observations: Vec::new(),
            missing_variables: Vec::new(),
            predictions: Vec::new(),
            forbidden_surface_forms: vec!["unbounded authority".to_string()],
        },
        &program,
    )
    .context("validate A1 lexical table")?;

    let surface = VerifierReadyRenderer
        .render(&program, &lexical_table)
        .context("render verifier-ready A1 surface")?;
    let original_body = surface.payload.text;
    let original_response = Response {
        intent: ResponseIntent::Teaching,
        style_hint: None,
        body: original_body.clone(),
        slots: Vec::new(),
    };
    let intent_before = original_response.intent.clone();
    let style_hint_before = original_response.style_hint.clone();
    let body_before = original_response.body.clone();
    let slots_before = original_response.slots.clone();
    let response = original_response.observe_semantic_shadow(&program, &lexical_table);
    let snapshot = live_typed_plan_snapshot();

    ensure!(
        response.intent == intent_before,
        "A1 altered the response intent"
    );
    ensure!(
        response.style_hint == style_hint_before,
        "A1 altered the response style hint"
    );
    ensure!(
        response.body == body_before,
        "A1 altered the returned response body"
    );
    ensure!(
        response.slots == slots_before,
        "A1 altered response metadata slots"
    );
    ensure!(
        response.body == original_body,
        "A1 altered verifier-ready text"
    );
    ensure!(
        snapshot.terminal_classification == TypedPlanTerminalClassification::Pass,
        "A1 did not reconstruct the authorized semantic program"
    );
    ensure!(
        snapshot.initial_residual == 2,
        "unexpected initial residual"
    );
    ensure!(
        snapshot.final_residual == 0,
        "semantic residual was not discharged"
    );
    ensure!(
        !snapshot.authority.generated_text_influence
            && !snapshot.authority.persistence_authority
            && !snapshot.authority.routing_authority
            && !snapshot.authority.tool_selection_authority
            && !snapshot.authority.charge_discharge_authority
            && !snapshot.authority.autonomous_action_authority,
        "A1 authority boundary opened unexpectedly"
    );

    println!("{}", serde_json::to_string_pretty(&snapshot)?);
    Ok(())
}
