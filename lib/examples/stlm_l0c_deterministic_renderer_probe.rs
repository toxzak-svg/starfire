//! Frozen STLM L0-C deterministic reference renderer probe.

use serde_json::json;
use star::language_realization::{
    authority_boundary as renderer_authority_boundary, ClaimLexicalBinding, DeterministicRenderer,
    LexicalBindingTable, LexicalBindingTablePayload, MissingVariableLexicalBinding,
    ObservationLexicalBinding, PredictionLexicalBinding, RealizationError, SurfaceReference,
};
use star::semantic_response::{
    AbstentionReason, AcknowledgmentLevel, AuthorizedClaim, ClaimId, ClaimPolarity,
    CognitiveStateVersion, CompanionStateVersion, ComputeBudget, DetailLevel, DialogueMode,
    DiscourseOperation, DiscourseOperationKind, EpistemicConstraint, EpistemicStatus,
    MissingVariableId, ObservationId, OperationId, OutputBudget, PredictionId, ProhibitedClaim,
    ResponseProgramId, SemanticResponseIntent, SemanticResponseProgram,
    SemanticResponseProgramPayload, SemanticValidationContext, SensitivityLevel, SensitivityPolicy,
    StyleEnvelope, SubjectScope, VocabularyLevel,
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

fn program(
    id: u64,
    detail: DetailLevel,
    allow_questions: bool,
    maximum_characters: u32,
) -> SemanticResponseProgram {
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
            "renderer_owns_cognition",
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
        id: ResponseProgramId(id),
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
                kind: DiscourseOperationKind::Abstain(AbstentionReason::InsufficientEvidence),
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
            allow_questions,
            maximum_paragraphs: 9,
        },
        output_budget: OutputBudget {
            maximum_characters,
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
    .expect("frozen semantic program must validate")
}

fn lexical_payload(program: &SemanticResponseProgram) -> LexicalBindingTablePayload {
    let clauses = [
        (
            "the wrapper objection is valid",
            "the wrapper objection is not valid",
        ),
        (
            "the renderer owns the cognition",
            "the renderer does not own the cognition",
        ),
        (
            "the semantic boundary preserves reasoning authority",
            "the semantic boundary does not preserve reasoning authority",
        ),
        (
            "Starfire's current semantic capability is limited",
            "Starfire's current semantic capability is not limited",
        ),
        (
            "the prior design was wrapper-like",
            "the prior design was not wrapper-like",
        ),
        (
            "the reference renderer is bounded",
            "the reference renderer is not bounded",
        ),
        (
            "the alignment can support later verification",
            "the alignment cannot support later verification",
        ),
    ];

    LexicalBindingTablePayload {
        program_digest: program.digest,
        subject_scope: SubjectScope(77),
        claims: clauses
            .iter()
            .enumerate()
            .map(|(index, (positive, negative))| ClaimLexicalBinding {
                claim: ClaimId(index as u64 + 1),
                positive_clause: (*positive).to_owned(),
                negative_clause: (*negative).to_owned(),
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

fn rejected_table(payload: LexicalBindingTablePayload, program: &SemanticResponseProgram) -> bool {
    LexicalBindingTable::validate(payload, program).is_err()
}

fn main() {
    let renderer = DeterministicRenderer;
    let detailed_program = program(1, DetailLevel::Detailed, true, 5_000);
    let detailed_table =
        LexicalBindingTable::validate(lexical_payload(&detailed_program), &detailed_program)
            .expect("frozen lexical table must validate");
    let first = renderer
        .render(&detailed_program, &detailed_table)
        .expect("frozen detailed render must succeed");
    let second = renderer
        .render(&detailed_program, &detailed_table)
        .expect("repeated frozen detailed render must succeed");

    let deterministic_text = first.payload.text == second.payload.text;
    let deterministic_alignments = first.payload.alignments == second.payload.alignments;
    let deterministic_costs = first.payload.operation_cost == second.payload.operation_cost
        && first.payload.claim_cost == second.payload.claim_cost
        && first.payload.verification_step_cost == second.payload.verification_step_cost
        && first.payload.character_cost == second.payload.character_cost;
    let deterministic_digest = first.digest == second.digest;
    let integrity_verified = first
        .verify_integrity(&detailed_program, &detailed_table)
        .is_ok();
    let all_operations_aligned = first.payload.alignments.len() == 9
        && first
            .payload
            .alignments
            .iter()
            .enumerate()
            .all(|(index, alignment)| alignment.operation == OperationId(index as u64 + 1));
    let all_reference_kinds_present = first
        .payload
        .alignments
        .iter()
        .flat_map(|alignment| alignment.references.iter())
        .any(|reference| matches!(reference, SurfaceReference::Observation(_)))
        && first
            .payload
            .alignments
            .iter()
            .flat_map(|alignment| alignment.references.iter())
            .any(|reference| matches!(reference, SurfaceReference::MissingVariable(_)))
        && first
            .payload
            .alignments
            .iter()
            .flat_map(|alignment| alignment.references.iter())
            .any(|reference| matches!(reference, SurfaceReference::Prediction(_)));
    let negative_polarity_preserved = first
        .payload
        .text
        .contains("the renderer does not own the cognition")
        && !first
            .payload
            .text
            .contains("the renderer owns the cognition");
    let epistemic_markers_preserved = first.payload.text.contains("I know that")
        && first.payload.text.contains("It is probable that")
        && first.payload.text.contains("It is possible that")
        && first.payload.text.contains("I am uncertain whether");
    let forbidden_absent = !first
        .payload
        .text
        .to_lowercase()
        .contains("fluency proves cognition");

    let brief_program = program(2, DetailLevel::Brief, false, 5_000);
    let brief_table =
        LexicalBindingTable::validate(lexical_payload(&brief_program), &brief_program)
            .expect("frozen brief lexical table must validate");
    let brief = renderer
        .render(&brief_program, &brief_table)
        .expect("frozen brief render must succeed");
    let style_changes_layout = detailed_program.payload.style.detail
        != brief_program.payload.style.detail
        && first.payload.text != brief.payload.text;
    let style_preserves_semantics = first
        .payload
        .alignments
        .iter()
        .map(|alignment| (alignment.operation, alignment.claim_ids.clone()))
        .collect::<Vec<_>>()
        == brief
            .payload
            .alignments
            .iter()
            .map(|alignment| (alignment.operation, alignment.claim_ids.clone()))
            .collect::<Vec<_>>();
    let question_policy_preserved = first.payload.text.contains('?')
        && !brief.payload.text.contains('?')
        && brief
            .payload
            .text
            .contains("Evidence is required for renderer substitution performance.");

    let tampered_program_rejected = {
        let mut program = detailed_program.clone();
        program.digest.0 ^= 1;
        renderer.render(&program, &detailed_table).is_err()
    };

    let wrong_program_digest_rejected = {
        let mut payload = lexical_payload(&detailed_program);
        payload.program_digest.0 ^= 1;
        rejected_table(payload, &detailed_program)
    };

    let wrong_scope_rejected = {
        let mut payload = lexical_payload(&detailed_program);
        payload.subject_scope = SubjectScope(88);
        rejected_table(payload, &detailed_program)
    };

    let unsorted_claims_rejected = {
        let mut payload = lexical_payload(&detailed_program);
        payload.claims.swap(0, 1);
        rejected_table(payload, &detailed_program)
    };

    let missing_binding_rejected = {
        let mut payload = lexical_payload(&detailed_program);
        payload.claims.pop();
        rejected_table(payload, &detailed_program)
    };

    let unused_binding_rejected = {
        let mut payload = lexical_payload(&detailed_program);
        payload.claims.push(ClaimLexicalBinding {
            claim: ClaimId(99),
            positive_clause: "unused positive clause".to_owned(),
            negative_clause: "unused negative clause".to_owned(),
        });
        rejected_table(payload, &detailed_program)
    };

    let prohibited_binding_rejected = {
        let mut payload = lexical_payload(&detailed_program);
        payload.claims.push(ClaimLexicalBinding {
            claim: ClaimId(100),
            positive_clause: "prohibited positive clause".to_owned(),
            negative_clause: "prohibited negative clause".to_owned(),
        });
        rejected_table(payload, &detailed_program)
    };

    let malformed_text_rejected = {
        let mut payload = lexical_payload(&detailed_program);
        payload.claims[0].positive_clause = " leading whitespace".to_owned();
        rejected_table(payload, &detailed_program)
    };

    let forbidden_binding_rejected = {
        let mut payload = lexical_payload(&detailed_program);
        payload.claims[0].positive_clause = "fluency proves cognition".to_owned();
        rejected_table(payload, &detailed_program)
    };

    let malformed_forbidden_forms_rejected = {
        let mut payload = lexical_payload(&detailed_program);
        payload
            .forbidden_surface_forms
            .push("fluency proves cognition".to_owned());
        rejected_table(payload, &detailed_program)
    };

    let forbidden_generated_text_rejected = {
        let mut payload = lexical_payload(&detailed_program);
        payload.forbidden_surface_forms = vec!["I know that".to_owned()];
        let table = LexicalBindingTable::validate(payload, &detailed_program)
            .expect("marker-only forbidden form does not occur in lexical bindings");
        matches!(
            renderer.render(&detailed_program, &table),
            Err(RealizationError::ForbiddenSurfaceForm)
        )
    };

    let budget_overflow_rejected_without_truncation = {
        let tiny_program = program(3, DetailLevel::Standard, true, 64);
        let table = LexicalBindingTable::validate(lexical_payload(&tiny_program), &tiny_program)
            .expect("tiny-budget lexical table must validate");
        matches!(
            renderer.render(&tiny_program, &table),
            Err(RealizationError::BudgetExceeded)
        )
    };

    let lexical_digest_tampering_rejected = {
        let mut table = detailed_table.clone();
        table.digest.0 ^= 1;
        matches!(
            renderer.render(&detailed_program, &table),
            Err(RealizationError::LexicalDigestMismatch)
        )
    };

    let realization_digest_tampering_rejected = {
        let mut realization = first.clone();
        realization.digest.0 ^= 1;
        matches!(
            realization.verify_integrity(&detailed_program, &detailed_table),
            Err(RealizationError::RealizationDigestMismatch)
        )
    };

    let alignment_overlap_rejected = {
        let mut realization = first.clone();
        realization.payload.alignments[1].span.start_byte = 0;
        matches!(
            realization.verify_integrity(&detailed_program, &detailed_table),
            Err(RealizationError::InvalidAlignment)
        )
    };

    let operation_reorder_rejected = {
        let mut realization = first.clone();
        realization.payload.alignments.swap(0, 1);
        matches!(
            realization.verify_integrity(&detailed_program, &detailed_table),
            Err(RealizationError::InvalidAlignment | RealizationError::OperationCoverageMismatch)
        )
    };

    let operation_omission_rejected = {
        let mut realization = first.clone();
        realization.payload.alignments.pop();
        matches!(
            realization.verify_integrity(&detailed_program, &detailed_table),
            Err(RealizationError::OperationCoverageMismatch)
        )
    };

    let unaligned_gap_rejected = {
        let mut realization = first.clone();
        let insertion = " unsupported";
        let insertion_point = realization.payload.alignments[0].span.end_byte;
        realization
            .payload
            .text
            .insert_str(insertion_point, insertion);
        for alignment in realization.payload.alignments.iter_mut().skip(1) {
            alignment.span.start_byte += insertion.len();
            alignment.span.end_byte += insertion.len();
        }
        realization.payload.character_cost +=
            u32::try_from(insertion.len()).expect("frozen insertion length fits u32");
        matches!(
            realization.verify_integrity(&detailed_program, &detailed_table),
            Err(RealizationError::InvalidAlignment)
        )
    };

    let underreported_costs_rejected = {
        let mut realization = first.clone();
        realization.payload.character_cost = 1;
        matches!(
            realization.verify_integrity(&detailed_program, &detailed_table),
            Err(RealizationError::BudgetExceeded)
        )
    };

    let polarity_tampering_rejected = {
        let mut realization = first.clone();
        realization.payload.text = realization.payload.text.replace(
            "the renderer does not own the cognition",
            "the renderer does now own the cognition",
        );
        matches!(
            realization.verify_integrity(&detailed_program, &detailed_table),
            Err(RealizationError::SemanticMarkerMismatch
                | RealizationError::RealizationDigestMismatch)
        )
    };

    let authority = renderer_authority_boundary();
    let authority_closed = !authority.runtime_chat_wiring
        && !authority.live_generated_text_influence
        && !authority.raw_conversation_access
        && !authority.unrestricted_memory_access
        && !authority.persistence_authority
        && !authority.routing_authority
        && !authority.companion_mutation_authority
        && !authority.belief_promotion_authority
        && !authority.ontology_promotion_authority
        && !authority.tool_selection_authority
        && !authority.charge_discharge_authority
        && !authority.autonomous_action_authority;

    let gate_passed = deterministic_text
        && deterministic_alignments
        && deterministic_costs
        && deterministic_digest
        && integrity_verified
        && all_operations_aligned
        && all_reference_kinds_present
        && negative_polarity_preserved
        && epistemic_markers_preserved
        && forbidden_absent
        && style_changes_layout
        && style_preserves_semantics
        && question_policy_preserved
        && tampered_program_rejected
        && wrong_program_digest_rejected
        && wrong_scope_rejected
        && unsorted_claims_rejected
        && missing_binding_rejected
        && unused_binding_rejected
        && prohibited_binding_rejected
        && malformed_text_rejected
        && forbidden_binding_rejected
        && malformed_forbidden_forms_rejected
        && forbidden_generated_text_rejected
        && budget_overflow_rejected_without_truncation
        && lexical_digest_tampering_rejected
        && realization_digest_tampering_rejected
        && alignment_overlap_rejected
        && operation_reorder_rejected
        && operation_omission_rejected
        && unaligned_gap_rejected
        && underreported_costs_rejected
        && polarity_tampering_rejected
        && authority_closed;

    let report = json!({
        "experiment": "STLM_L0C_DETERMINISTIC_RENDERER",
        "terminal_classification": if gate_passed { "PASS" } else { "FAIL" },
        "deterministic_text": deterministic_text,
        "deterministic_alignments": deterministic_alignments,
        "deterministic_costs": deterministic_costs,
        "deterministic_digest": deterministic_digest,
        "integrity_verified": integrity_verified,
        "all_operations_aligned": all_operations_aligned,
        "all_reference_kinds_present": all_reference_kinds_present,
        "negative_polarity_preserved": negative_polarity_preserved,
        "epistemic_markers_preserved": epistemic_markers_preserved,
        "forbidden_absent": forbidden_absent,
        "style_changes_layout": style_changes_layout,
        "style_preserves_semantics": style_preserves_semantics,
        "question_policy_preserved": question_policy_preserved,
        "tampered_program_rejected": tampered_program_rejected,
        "wrong_program_digest_rejected": wrong_program_digest_rejected,
        "wrong_scope_rejected": wrong_scope_rejected,
        "unsorted_claims_rejected": unsorted_claims_rejected,
        "missing_binding_rejected": missing_binding_rejected,
        "unused_binding_rejected": unused_binding_rejected,
        "prohibited_binding_rejected": prohibited_binding_rejected,
        "malformed_text_rejected": malformed_text_rejected,
        "forbidden_binding_rejected": forbidden_binding_rejected,
        "malformed_forbidden_forms_rejected": malformed_forbidden_forms_rejected,
        "forbidden_generated_text_rejected": forbidden_generated_text_rejected,
        "budget_overflow_rejected_without_truncation": budget_overflow_rejected_without_truncation,
        "lexical_digest_tampering_rejected": lexical_digest_tampering_rejected,
        "realization_digest_tampering_rejected": realization_digest_tampering_rejected,
        "alignment_overlap_rejected": alignment_overlap_rejected,
        "operation_reorder_rejected": operation_reorder_rejected,
        "operation_omission_rejected": operation_omission_rejected,
        "unaligned_gap_rejected": unaligned_gap_rejected,
        "underreported_costs_rejected": underreported_costs_rejected,
        "polarity_tampering_rejected": polarity_tampering_rejected,
        "authority_boundary_closed": authority_closed,
        "detailed_character_cost": first.payload.character_cost,
        "detailed_sentence_count": first.payload.sentence_count,
        "detailed_paragraph_count": first.payload.paragraph_count,
        "alignment_count": first.payload.alignments.len(),
        "gate_passed": gate_passed,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("serializing frozen report cannot fail")
    );

    if !gate_passed {
        std::process::exit(1);
    }
}
