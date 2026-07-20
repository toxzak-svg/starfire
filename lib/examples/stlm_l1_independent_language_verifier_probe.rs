use anyhow::{Context, Result};
use serde_json::json;
use star::language_realization::{
    ClaimLexicalBinding, LexicalBindingTable, LexicalBindingTablePayload,
    MissingVariableLexicalBinding, ObservationLexicalBinding, PredictionLexicalBinding,
};
use star::language_verification::{
    IndependentLanguageVerifier, LanguageVerificationInput, ReconstructedOperationKind,
};
use star::semantic_response::{
    AbstentionReason, AcknowledgmentLevel, AuthorizedClaim, ClaimId, ClaimPolarity,
    CognitiveStateVersion, ComputeBudget, DetailLevel, DialogueMode, DiscourseOperation,
    DiscourseOperationKind, EpistemicConstraint, EpistemicStatus, MissingVariableId,
    ObservationId, OperationId, OutputBudget, PredictionId, ProhibitedClaim, ResponseProgramDigest,
    ResponseProgramId, SemanticResponseIntent, SemanticResponseProgram,
    SemanticResponseProgramPayload, SemanticValidationContext, SensitivityLevel,
    SensitivityPolicy, StyleEnvelope, SubjectScope, VocabularyLevel,
};
use star::verifier_ready_realization::{
    authority_boundary as renderer_authority_boundary, VerifierReadyRenderer,
    VERIFIER_READY_GRAMMAR_VERSION,
};

const SUBJECT: SubjectScope = SubjectScope(77);
const COGNITIVE_VERSION: CognitiveStateVersion = CognitiveStateVersion(41);

fn claim(
    id: u64,
    key: &str,
    polarity: ClaimPolarity,
    confidence_bps: u16,
    epistemic_status: EpistemicStatus,
) -> AuthorizedClaim {
    AuthorizedClaim {
        id: ClaimId(id),
        semantic_key: key.to_owned(),
        polarity,
        confidence_bps,
        epistemic_status,
        sensitivity: SensitivityLevel::Public,
        disclosure_scope: SUBJECT,
    }
}

fn program(maximum_characters: u32, maximum_sentences: u16) -> Result<SemanticResponseProgram> {
    let required_claims = vec![
        claim(
            1,
            "semantic_boundary_explicit",
            ClaimPolarity::Positive,
            9_500,
            EpistemicStatus::Certain,
        ),
        claim(
            2,
            "renderer_owns_cognition",
            ClaimPolarity::Negative,
            8_100,
            EpistemicStatus::Probable,
        ),
        claim(
            3,
            "verifier_reads_text_independently",
            ClaimPolarity::Positive,
            9_300,
            EpistemicStatus::Certain,
        ),
        claim(
            4,
            "reference_grammar_bounded",
            ClaimPolarity::Positive,
            5_000,
            EpistemicStatus::Possible,
        ),
        claim(
            5,
            "alignment_is_independent_evidence",
            ClaimPolarity::Negative,
            7_600,
            EpistemicStatus::Probable,
        ),
        claim(
            6,
            "inverse_projection_preserves_references",
            ClaimPolarity::Positive,
            7_800,
            EpistemicStatus::Probable,
        ),
        claim(
            7,
            "open_ended_language_proven",
            ClaimPolarity::Negative,
            2_000,
            EpistemicStatus::Uncertain,
        ),
        claim(
            8,
            "final_mechanism_fully_known",
            ClaimPolarity::Negative,
            0,
            EpistemicStatus::Unknown,
        ),
    ];
    let epistemic_constraints = required_claims
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

    let payload = SemanticResponseProgramPayload {
        id: ResponseProgramId(1),
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
            DiscourseOperation {
                id: OperationId(3),
                kind: DiscourseOperationKind::Contrast {
                    left: ClaimId(3),
                    right: ClaimId(4),
                },
            },
            DiscourseOperation {
                id: OperationId(4),
                kind: DiscourseOperationKind::Correct {
                    prior: ClaimId(5),
                    replacement: ClaimId(6),
                },
            },
            DiscourseOperation {
                id: OperationId(5),
                kind: DiscourseOperationKind::Explain {
                    claims: vec![ClaimId(7), ClaimId(8)],
                },
            },
            DiscourseOperation {
                id: OperationId(6),
                kind: DiscourseOperationKind::Acknowledge(ObservationId(101)),
            },
            DiscourseOperation {
                id: OperationId(7),
                kind: DiscourseOperationKind::RequestEvidence(MissingVariableId(201)),
            },
            DiscourseOperation {
                id: OperationId(8),
                kind: DiscourseOperationKind::Commit(PredictionId(301)),
            },
            DiscourseOperation {
                id: OperationId(9),
                kind: DiscourseOperationKind::Abstain(AbstentionReason::InsufficientEvidence),
            },
        ],
        required_claims,
        optional_claims: Vec::new(),
        prohibited_claims: vec![ProhibitedClaim {
            id: ClaimId(9),
            semantic_key: "forbidden_unbounded_authority".to_owned(),
        }],
        epistemic_constraints,
        sensitivity: SensitivityPolicy {
            maximum_disclosure: SensitivityLevel::Public,
            disclosure_scope: SUBJECT,
        },
        style: StyleEnvelope {
            detail: DetailLevel::Detailed,
            vocabulary: VocabularyLevel::Technical,
            dialogue: DialogueMode::Collaborative,
            acknowledgment: AcknowledgmentLevel::Explicit,
            allow_first_person: true,
            allow_questions: true,
            maximum_paragraphs: 9,
        },
        output_budget: OutputBudget {
            maximum_characters,
            maximum_sentences,
        },
        compute_budget: ComputeBudget {
            maximum_operations: 16,
            maximum_claims: 16,
            maximum_verification_steps: 64,
        },
    };

    SemanticResponseProgram::validate(
        payload,
        SemanticValidationContext {
            cognitive_state_version: COGNITIVE_VERSION,
            companion_state_version: None,
            subject_scope: SUBJECT,
        },
    )
    .context("validate L1 semantic program")
}

fn lexical_table(program: &SemanticResponseProgram) -> Result<LexicalBindingTable> {
    lexical_table_with_observation(program, "the alignment witness")
}

fn lexical_table_with_observation(
    program: &SemanticResponseProgram,
    observation_label: &str,
) -> Result<LexicalBindingTable> {
    let payload = LexicalBindingTablePayload {
        program_digest: program.digest,
        subject_scope: SUBJECT,
        claims: vec![
            ClaimLexicalBinding {
                claim: ClaimId(1),
                positive_clause: "the semantic boundary is explicit".to_owned(),
                negative_clause: "the semantic boundary is not explicit".to_owned(),
            },
            ClaimLexicalBinding {
                claim: ClaimId(2),
                positive_clause: "the renderer owns cognition".to_owned(),
                negative_clause: "the renderer does not own cognition".to_owned(),
            },
            ClaimLexicalBinding {
                claim: ClaimId(3),
                positive_clause: "the verifier reads text independently".to_owned(),
                negative_clause: "the verifier does not read text independently".to_owned(),
            },
            ClaimLexicalBinding {
                claim: ClaimId(4),
                positive_clause: "the reference grammar remains bounded".to_owned(),
                negative_clause: "the reference grammar is not bounded".to_owned(),
            },
            ClaimLexicalBinding {
                claim: ClaimId(5),
                positive_clause: "the old alignment witness is independent evidence".to_owned(),
                negative_clause: "the old alignment witness is not independent evidence".to_owned(),
            },
            ClaimLexicalBinding {
                claim: ClaimId(6),
                positive_clause: "the inverse projection preserves typed references".to_owned(),
                negative_clause: "the inverse projection does not preserve typed references".to_owned(),
            },
            ClaimLexicalBinding {
                claim: ClaimId(7),
                positive_clause: "open ended language is proven".to_owned(),
                negative_clause: "open ended language remains unproven".to_owned(),
            },
            ClaimLexicalBinding {
                claim: ClaimId(8),
                positive_clause: "the final mechanism is fully known".to_owned(),
                negative_clause: "the final mechanism is not fully known".to_owned(),
            },
        ],
        observations: vec![ObservationLexicalBinding {
            observation: ObservationId(101),
            label: observation_label.to_owned(),
        }],
        missing_variables: vec![MissingVariableLexicalBinding {
            variable: MissingVariableId(201),
            label: "the independent verifier outcome".to_owned(),
        }],
        predictions: vec![PredictionLexicalBinding {
            prediction: PredictionId(301),
            label: "the frozen L1 controls".to_owned(),
        }],
        forbidden_surface_forms: vec!["forbidden leakage".to_owned()],
    };
    LexicalBindingTable::validate(payload, program).context("validate L1 lexical table")
}

fn verify_text(
    program: &SemanticResponseProgram,
    lexical_table: &LexicalBindingTable,
    text: &str,
) -> bool {
    IndependentLanguageVerifier
        .verify(LanguageVerificationInput {
            program,
            lexical_table,
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            subject_scope: SUBJECT,
            grammar_version: VERIFIER_READY_GRAMMAR_VERSION,
            text,
        })
        .is_ok()
}

fn main() -> Result<()> {
    let program = program(5_000, 10)?;
    let lexical_table = lexical_table(&program)?;
    let surface = VerifierReadyRenderer
        .render(&program, &lexical_table)
        .context("render verifier-ready surface")?;
    surface.verify_digest().context("verify v2 surface digest")?;

    let verifier = IndependentLanguageVerifier;
    let report = verifier
        .verify(LanguageVerificationInput {
            program: &program,
            lexical_table: &lexical_table,
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            subject_scope: SUBJECT,
            grammar_version: VERIFIER_READY_GRAMMAR_VERSION,
            text: &surface.payload.text,
        })
        .context("verify canonical v2 surface")?;
    report.verify_digest().context("verify L1 report digest")?;
    let repeated = verifier
        .verify(LanguageVerificationInput {
            program: &program,
            lexical_table: &lexical_table,
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            subject_scope: SUBJECT,
            grammar_version: VERIFIER_READY_GRAMMAR_VERSION,
            text: &surface.payload.text,
        })
        .context("repeat canonical verification")?;

    let all_nine_operations_reconstructed = report.payload.reconstructed_operations.len() == 9
        && report
            .payload
            .reconstructed_operations
            .iter()
            .any(|operation| matches!(operation.kind, ReconstructedOperationKind::Assert(_)))
        && report
            .payload
            .reconstructed_operations
            .iter()
            .any(|operation| matches!(operation.kind, ReconstructedOperationKind::Qualify(_)))
        && report
            .payload
            .reconstructed_operations
            .iter()
            .any(|operation| matches!(operation.kind, ReconstructedOperationKind::Contrast { .. }))
        && report
            .payload
            .reconstructed_operations
            .iter()
            .any(|operation| matches!(operation.kind, ReconstructedOperationKind::Correct { .. }))
        && report
            .payload
            .reconstructed_operations
            .iter()
            .any(|operation| matches!(operation.kind, ReconstructedOperationKind::Explain { .. }))
        && report
            .payload
            .reconstructed_operations
            .iter()
            .any(|operation| matches!(operation.kind, ReconstructedOperationKind::Acknowledge(_)))
        && report.payload.reconstructed_operations.iter().any(|operation| {
            matches!(
                operation.kind,
                ReconstructedOperationKind::RequestEvidence(_)
            )
        })
        && report
            .payload
            .reconstructed_operations
            .iter()
            .any(|operation| matches!(operation.kind, ReconstructedOperationKind::Commit(_)))
        && report
            .payload
            .reconstructed_operations
            .iter()
            .any(|operation| matches!(operation.kind, ReconstructedOperationKind::Abstain(_)));

    let segments = surface
        .payload
        .alignments
        .iter()
        .map(|alignment| {
            surface.payload.text[alignment.span.start_byte..alignment.span.end_byte].to_owned()
        })
        .collect::<Vec<_>>();

    let mut omitted = segments.clone();
    omitted.remove(0);
    let omission_rejected = !verify_text(&program, &lexical_table, &omitted.join("\n\n"));

    let mut duplicated = segments.clone();
    duplicated.insert(1, segments[0].clone());
    let duplicate_rejected = !verify_text(&program, &lexical_table, &duplicated.join("\n\n"));

    let mut reordered = segments.clone();
    reordered.swap(0, 1);
    let reorder_rejected = !verify_text(&program, &lexical_table, &reordered.join("\n\n"));

    let mut inserted = segments.clone();
    inserted.insert(1, "Injected unsupported sentence.".to_owned());
    let unsupported_insertion_rejected =
        !verify_text(&program, &lexical_table, &inserted.join("\n\n"));

    let trailing_text_rejected = !verify_text(
        &program,
        &lexical_table,
        &format!("{} trailing", surface.payload.text),
    );

    let polarity_reversal_rejected = !verify_text(
        &program,
        &lexical_table,
        &surface.payload.text.replacen(
            "the semantic boundary is explicit",
            "the semantic boundary is not explicit",
            1,
        ),
    );

    let certainty_inflation_rejected = !verify_text(
        &program,
        &lexical_table,
        &surface
            .payload
            .text
            .replacen("It is probable that", "I know that", 1),
    );
    let certainty_collapse_rejected = !verify_text(
        &program,
        &lexical_table,
        &surface
            .payload
            .text
            .replacen("I know that", "I am uncertain whether", 1),
    );

    let claim_substitution_rejected = !verify_text(
        &program,
        &lexical_table,
        &surface.payload.text.replacen(
            &segments[0],
            "I know that the verifier reads text independently.",
            1,
        ),
    );
    let observation_substitution_rejected = !verify_text(
        &program,
        &lexical_table,
        &surface
            .payload
            .text
            .replacen("the alignment witness", "a substituted observation", 1),
    );
    let variable_substitution_rejected = !verify_text(
        &program,
        &lexical_table,
        &surface.payload.text.replacen(
            "the independent verifier outcome",
            "a substituted variable",
            1,
        ),
    );
    let prediction_substitution_rejected = !verify_text(
        &program,
        &lexical_table,
        &surface
            .payload
            .text
            .replacen("the frozen L1 controls", "a substituted prediction", 1),
    );
    let abstention_reason_substitution_rejected = !verify_text(
        &program,
        &lexical_table,
        &surface.payload.text.replacen(
            "I abstain because the available evidence is insufficient.",
            "I abstain because the available evidence is contradictory.",
            1,
        ),
    );

    let forbidden_form_rejected = !verify_text(
        &program,
        &lexical_table,
        &surface
            .payload
            .text
            .replacen(&segments[5], "I acknowledge forbidden leakage.", 1),
    );
    let noncanonical_separator_rejected = !verify_text(
        &program,
        &lexical_table,
        &surface.payload.text.replacen("\n\n", " ", 1),
    );

    let mut ambiguous_payload = lexical_table.payload.clone();
    ambiguous_payload.claims[2].positive_clause =
        ambiguous_payload.claims[0].positive_clause.clone();
    let ambiguous_table = LexicalBindingTable::validate(ambiguous_payload, &program)
        .context("validate intentionally inverse-ambiguous table")?;
    let ambiguous_surface_binding_rejected = !verify_text(
        &program,
        &ambiguous_table,
        &surface.payload.text,
    );

    let character_limited_program = program(
        u32::try_from(surface.payload.text.len().saturating_sub(1))
            .context("character budget conversion")?,
        10,
    )?;
    let character_limited_lexical = lexical_table(&character_limited_program)?;
    let character_budget_rejected = !verify_text(
        &character_limited_program,
        &character_limited_lexical,
        &surface.payload.text,
    );

    let sentence_limited_lexical =
        lexical_table_with_observation(&program, "the alignment witness. Extra sentence")?;
    let sentence_overflow_text = surface.payload.text.replacen(
        "I acknowledge the alignment witness.",
        "I acknowledge the alignment witness. Extra sentence.",
        1,
    );
    let sentence_budget_rejected = !verify_text(
        &program,
        &sentence_limited_lexical,
        &sentence_overflow_text,
    );

    let stale_program_digest_rejected = verifier
        .verify(LanguageVerificationInput {
            program: &program,
            lexical_table: &lexical_table,
            program_digest: ResponseProgramDigest(program.digest.0.wrapping_add(1)),
            lexical_table_digest: lexical_table.digest,
            subject_scope: SUBJECT,
            grammar_version: VERIFIER_READY_GRAMMAR_VERSION,
            text: &surface.payload.text,
        })
        .is_err();
    let stale_lexical_digest_rejected = verifier
        .verify(LanguageVerificationInput {
            program: &program,
            lexical_table: &lexical_table,
            program_digest: program.digest,
            lexical_table_digest: star::language_realization::LexicalTableDigest(
                lexical_table.digest.0.wrapping_add(1),
            ),
            subject_scope: SUBJECT,
            grammar_version: VERIFIER_READY_GRAMMAR_VERSION,
            text: &surface.payload.text,
        })
        .is_err();
    let wrong_scope_rejected = verifier
        .verify(LanguageVerificationInput {
            program: &program,
            lexical_table: &lexical_table,
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            subject_scope: SubjectScope(999),
            grammar_version: VERIFIER_READY_GRAMMAR_VERSION,
            text: &surface.payload.text,
        })
        .is_err();
    let wrong_grammar_rejected = verifier
        .verify(LanguageVerificationInput {
            program: &program,
            lexical_table: &lexical_table,
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            subject_scope: SUBJECT,
            grammar_version: 1,
            text: &surface.payload.text,
        })
        .is_err();

    let mut forged_alignment_surface = surface.clone();
    forged_alignment_surface.payload.alignments.clear();
    let alignment_independence_preserved = verifier
        .verify(LanguageVerificationInput {
            program: &program,
            lexical_table: &lexical_table,
            program_digest: program.digest,
            lexical_table_digest: lexical_table.digest,
            subject_scope: SUBJECT,
            grammar_version: VERIFIER_READY_GRAMMAR_VERSION,
            text: &forged_alignment_surface.payload.text,
        })
        .map(|forged_report| forged_report == report)
        .unwrap_or(false);

    let verifier_boundary = star::language_verification::authority_boundary();
    let renderer_boundary = renderer_authority_boundary();
    let authority_boundary_closed = !verifier_boundary.runtime_chat_wiring
        && !verifier_boundary.live_generated_text_influence
        && !verifier_boundary.raw_conversation_access
        && !verifier_boundary.unrestricted_memory_access
        && !verifier_boundary.persistence_authority
        && !verifier_boundary.routing_authority
        && !verifier_boundary.companion_mutation_authority
        && !verifier_boundary.belief_promotion_authority
        && !verifier_boundary.ontology_promotion_authority
        && !verifier_boundary.tool_selection_authority
        && !verifier_boundary.charge_discharge_authority
        && !verifier_boundary.autonomous_action_authority
        && !renderer_boundary.runtime_chat_wiring
        && !renderer_boundary.live_generated_text_influence
        && !renderer_boundary.raw_conversation_access
        && !renderer_boundary.unrestricted_memory_access
        && !renderer_boundary.persistence_authority
        && !renderer_boundary.routing_authority
        && !renderer_boundary.companion_mutation_authority
        && !renderer_boundary.belief_promotion_authority
        && !renderer_boundary.ontology_promotion_authority
        && !renderer_boundary.tool_selection_authority
        && !renderer_boundary.charge_discharge_authority
        && !renderer_boundary.autonomous_action_authority;

    let deterministic_report = repeated == report
        && repeated.canonical_bytes()? == report.canonical_bytes()?
        && repeated.digest == report.digest;

    let gate_passed = all_nine_operations_reconstructed
        && deterministic_report
        && alignment_independence_preserved
        && omission_rejected
        && duplicate_rejected
        && reorder_rejected
        && unsupported_insertion_rejected
        && trailing_text_rejected
        && polarity_reversal_rejected
        && certainty_inflation_rejected
        && certainty_collapse_rejected
        && claim_substitution_rejected
        && observation_substitution_rejected
        && variable_substitution_rejected
        && prediction_substitution_rejected
        && abstention_reason_substitution_rejected
        && forbidden_form_rejected
        && noncanonical_separator_rejected
        && ambiguous_surface_binding_rejected
        && character_budget_rejected
        && sentence_budget_rejected
        && stale_program_digest_rejected
        && stale_lexical_digest_rejected
        && wrong_scope_rejected
        && wrong_grammar_rejected
        && authority_boundary_closed;

    let verdict = json!({
        "terminal_classification": if gate_passed { "PASS" } else { "FAIL" },
        "gate_passed": gate_passed,
        "grammar_version": VERIFIER_READY_GRAMMAR_VERSION,
        "all_nine_operations_reconstructed": all_nine_operations_reconstructed,
        "deterministic_report": deterministic_report,
        "alignment_independence_preserved": alignment_independence_preserved,
        "omission_rejected": omission_rejected,
        "duplicate_rejected": duplicate_rejected,
        "reorder_rejected": reorder_rejected,
        "unsupported_insertion_rejected": unsupported_insertion_rejected,
        "trailing_text_rejected": trailing_text_rejected,
        "polarity_reversal_rejected": polarity_reversal_rejected,
        "certainty_inflation_rejected": certainty_inflation_rejected,
        "certainty_collapse_rejected": certainty_collapse_rejected,
        "claim_substitution_rejected": claim_substitution_rejected,
        "observation_substitution_rejected": observation_substitution_rejected,
        "variable_substitution_rejected": variable_substitution_rejected,
        "prediction_substitution_rejected": prediction_substitution_rejected,
        "abstention_reason_substitution_rejected": abstention_reason_substitution_rejected,
        "forbidden_form_rejected": forbidden_form_rejected,
        "noncanonical_separator_rejected": noncanonical_separator_rejected,
        "ambiguous_surface_binding_rejected": ambiguous_surface_binding_rejected,
        "character_budget_rejected": character_budget_rejected,
        "sentence_budget_rejected": sentence_budget_rejected,
        "stale_program_digest_rejected": stale_program_digest_rejected,
        "stale_lexical_digest_rejected": stale_lexical_digest_rejected,
        "wrong_scope_rejected": wrong_scope_rejected,
        "wrong_grammar_rejected": wrong_grammar_rejected,
        "authority_boundary_closed": authority_boundary_closed,
        "reconstructed_operation_count": report.payload.reconstructed_operations.len(),
        "report_digest": report.digest.0,
        "surface_digest": surface.digest.0,
        "character_cost": report.payload.costs.character_cost,
        "sentence_count": report.payload.costs.sentence_count,
        "paragraph_count": report.payload.costs.paragraph_count
    });
    println!("{}", serde_json::to_string_pretty(&verdict)?);

    if gate_passed {
        Ok(())
    } else {
        anyhow::bail!("STLM L1 frozen verifier probe failed")
    }
}
