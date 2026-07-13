//! Frozen STLM L0-B semantic-program mechanics probe.

use serde_json::json;
use star::semantic_response::{
    authority_boundary, AbstentionReason, AcknowledgmentLevel, AuthorizedClaim, ClaimId,
    ClaimPolarity, CompanionStateVersion, ComputeBudget, CognitiveStateVersion, DetailLevel,
    DialogueMode, DiscourseOperation, DiscourseOperationKind, EpistemicConstraint,
    EpistemicStatus, MissingVariableId, ObservationId, OperationId, OutputBudget,
    PredictionId, ProhibitedClaim, ResponseProgramId, SemanticProgramError,
    SemanticProgramRegistry, SemanticResponseIntent, SemanticResponseProgram,
    SemanticResponseProgramPayload, SemanticValidationContext, SensitivityLevel,
    SensitivityPolicy, StyleEnvelope, SubjectScope, VocabularyLevel,
};

fn claim(
    id: u64,
    key: &str,
    confidence_bps: u16,
    status: EpistemicStatus,
    sensitivity: SensitivityLevel,
) -> AuthorizedClaim {
    AuthorizedClaim {
        id: ClaimId(id),
        semantic_key: key.to_owned(),
        polarity: ClaimPolarity::Positive,
        confidence_bps,
        epistemic_status: status,
        sensitivity,
        disclosure_scope: SubjectScope(77),
    }
}

fn fixture_payload(program_id: u64, source_version: u64) -> SemanticResponseProgramPayload {
    let required_claims = vec![
        claim(
            1,
            "wrapper_risk_valid",
            9_500,
            EpistemicStatus::Certain,
            SensitivityLevel::Public,
        ),
        claim(
            2,
            "unrestricted_llm_owns_cognition",
            8_400,
            EpistemicStatus::Probable,
            SensitivityLevel::Public,
        ),
        claim(
            3,
            "constrained_renderer_preserves_authority",
            7_800,
            EpistemicStatus::Probable,
            SensitivityLevel::Public,
        ),
        claim(
            4,
            "current_starfire_semantics_limited",
            9_200,
            EpistemicStatus::Certain,
            SensitivityLevel::Personal,
        ),
    ];
    let optional_claims = vec![
        claim(
            5,
            "prior_wrapper_equivalence",
            5_000,
            EpistemicStatus::Possible,
            SensitivityLevel::Public,
        ),
        claim(
            6,
            "renderer_is_surface_realizer",
            8_100,
            EpistemicStatus::Probable,
            SensitivityLevel::Public,
        ),
        claim(
            7,
            "semantic_boundary_enables_attribution",
            7_400,
            EpistemicStatus::Probable,
            SensitivityLevel::Public,
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

    SemanticResponseProgramPayload {
        id: ResponseProgramId(program_id),
        source_state_version: CognitiveStateVersion(source_version),
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
        prohibited_claims: vec![
            ProhibitedClaim {
                id: ClaimId(100),
                semantic_key: "starfire_has_general_language_understanding".to_owned(),
            },
            ProhibitedClaim {
                id: ClaimId(101),
                semantic_key: "fluency_proves_cognition".to_owned(),
            },
        ],
        epistemic_constraints,
        sensitivity: SensitivityPolicy {
            maximum_disclosure: SensitivityLevel::Personal,
            disclosure_scope: SubjectScope(77),
        },
        style: StyleEnvelope {
            detail: DetailLevel::Detailed,
            vocabulary: VocabularyLevel::Technical,
            dialogue: DialogueMode::Collaborative,
            acknowledgment: AcknowledgmentLevel::Brief,
            allow_first_person: true,
            allow_questions: true,
            maximum_paragraphs: 6,
        },
        output_budget: OutputBudget {
            maximum_characters: 1_200,
            maximum_sentences: 20,
        },
        compute_budget: ComputeBudget {
            maximum_operations: 16,
            maximum_claims: 16,
            maximum_verification_steps: 128,
        },
    }
}

fn context(source_version: u64) -> SemanticValidationContext {
    SemanticValidationContext {
        cognitive_state_version: CognitiveStateVersion(source_version),
        companion_state_version: Some(CompanionStateVersion(12)),
        subject_scope: SubjectScope(77),
    }
}

fn rejected_atomically(
    registry: &mut SemanticProgramRegistry,
    expected_registry_version: u64,
    payload: SemanticResponseProgramPayload,
    validation_context: SemanticValidationContext,
) -> bool {
    let before = registry.clone();
    registry
        .commit(
            expected_registry_version,
            payload,
            validation_context,
        )
        .is_err()
        && *registry == before
}

fn main() {
    let payload = fixture_payload(1, 41);
    let first = SemanticResponseProgram::validate(payload.clone(), context(41));
    let second = SemanticResponseProgram::validate(payload.clone(), context(41));
    let deterministic_validation = matches!((&first, &second), (Ok(left), Ok(right)) if left == right);
    let canonical_bytes_identical =
        matches!((&first, &second), (Ok(left), Ok(right)) if left.canonical_bytes().ok() == right.canonical_bytes().ok());
    let digest_identical =
        matches!((&first, &second), (Ok(left), Ok(right)) if left.digest == right.digest);

    let all_operation_variants_present = {
        let operations = &payload.operations;
        operations
            .iter()
            .any(|operation| matches!(&operation.kind, DiscourseOperationKind::Assert(_)))
            && operations.iter().any(|operation| {
                matches!(&operation.kind, DiscourseOperationKind::Qualify { .. })
            })
            && operations.iter().any(|operation| {
                matches!(&operation.kind, DiscourseOperationKind::Contrast { .. })
            })
            && operations.iter().any(|operation| {
                matches!(&operation.kind, DiscourseOperationKind::Correct { .. })
            })
            && operations.iter().any(|operation| {
                matches!(&operation.kind, DiscourseOperationKind::Explain { .. })
            })
            && operations.iter().any(|operation| {
                matches!(&operation.kind, DiscourseOperationKind::Acknowledge(_))
            })
            && operations.iter().any(|operation| {
                matches!(&operation.kind, DiscourseOperationKind::RequestEvidence(_))
            })
            && operations.iter().any(|operation| {
                matches!(&operation.kind, DiscourseOperationKind::Commit(_))
            })
            && operations.iter().any(|operation| {
                matches!(&operation.kind, DiscourseOperationKind::Abstain(_))
            })
    };

    let mut registry = SemanticProgramRegistry::default();
    let first_commit = registry.commit(0, payload.clone(), context(41)).is_ok();
    let second_commit = registry
        .commit(1, fixture_payload(2, 42), context(42))
        .is_ok();
    let replay = SemanticProgramRegistry::replay(registry.events());
    let exact_replay = replay.as_ref() == Ok(&registry);
    let repeated_replay = replay
        .as_ref()
        .ok()
        .and_then(|replayed| SemanticProgramRegistry::replay(replayed.events()).ok())
        .as_ref()
        == Some(&registry);

    let mut duplicate_operation = fixture_payload(3, 43);
    duplicate_operation.operations[1].id = OperationId(1);
    let duplicate_operation_rejected = rejected_atomically(
        &mut registry,
        2,
        duplicate_operation,
        context(43),
    );

    let mut unknown_claim = fixture_payload(3, 43);
    unknown_claim.operations[1].kind = DiscourseOperationKind::Assert(ClaimId(999));
    let unknown_claim_rejected =
        rejected_atomically(&mut registry, 2, unknown_claim, context(43));

    let mut overlap = fixture_payload(3, 43);
    overlap.prohibited_claims[0].semantic_key =
        overlap.required_claims[0].semantic_key.clone();
    let authorized_prohibited_overlap_rejected =
        rejected_atomically(&mut registry, 2, overlap, context(43));

    let mut confidence = fixture_payload(3, 43);
    confidence.required_claims[0].confidence_bps = 10_001;
    let invalid_confidence_rejected =
        rejected_atomically(&mut registry, 2, confidence, context(43));

    let mut qualifier = fixture_payload(3, 43);
    qualifier.operations[2].kind = DiscourseOperationKind::Qualify {
        claim: ClaimId(4),
        status: EpistemicStatus::Possible,
    };
    let qualifier_mismatch_rejected =
        rejected_atomically(&mut registry, 2, qualifier, context(43));

    let stale_state_rejected = rejected_atomically(
        &mut registry,
        2,
        fixture_payload(3, 43),
        context(44),
    );

    let stale_companion_rejected = {
        let before = registry.clone();
        let mut stale_context = context(43);
        stale_context.companion_state_version = Some(CompanionStateVersion(13));
        let rejected = registry
            .commit(2, fixture_payload(3, 43), stale_context)
            .is_err();
        rejected && registry == before
    };

    let subject_mismatch_rejected = {
        let before = registry.clone();
        let mut mismatched_context = context(43);
        mismatched_context.subject_scope = SubjectScope(88);
        let rejected = registry
            .commit(2, fixture_payload(3, 43), mismatched_context)
            .is_err();
        rejected && registry == before
    };

    let mut sensitivity = fixture_payload(3, 43);
    sensitivity.required_claims[3].disclosure_scope = SubjectScope(88);
    let sensitivity_scope_rejected =
        rejected_atomically(&mut registry, 2, sensitivity, context(43));

    let mut sensitivity_level = fixture_payload(3, 43);
    sensitivity_level.sensitivity.maximum_disclosure = SensitivityLevel::Public;
    let sensitivity_level_rejected =
        rejected_atomically(&mut registry, 2, sensitivity_level, context(43));

    let mut claim_order = fixture_payload(3, 43);
    claim_order.required_claims.swap(0, 1);
    let noncanonical_claim_order_rejected =
        rejected_atomically(&mut registry, 2, claim_order, context(43));

    let mut epistemic_order = fixture_payload(3, 43);
    epistemic_order.epistemic_constraints.swap(0, 1);
    let noncanonical_epistemic_order_rejected =
        rejected_atomically(&mut registry, 2, epistemic_order, context(43));

    let mut output_budget = fixture_payload(3, 43);
    output_budget.output_budget.maximum_characters = 0;
    let invalid_output_budget_rejected =
        rejected_atomically(&mut registry, 2, output_budget, context(43));

    let mut compute_budget = fixture_payload(3, 43);
    compute_budget.compute_budget.maximum_operations = 1;
    let exceeded_compute_budget_rejected =
        rejected_atomically(&mut registry, 2, compute_budget, context(43));

    let duplicate_program_rejected = rejected_atomically(
        &mut registry,
        2,
        fixture_payload(1, 41),
        context(41),
    );
    let stale_registry_version_rejected = rejected_atomically(
        &mut registry,
        1,
        fixture_payload(3, 43),
        context(43),
    );

    let digest_tampering_rejected = {
        let mut tampered = registry.events().to_vec();
        tampered[0].program.digest.0 ^= 1;
        matches!(
            SemanticProgramRegistry::replay(&tampered),
            Err(SemanticProgramError::DigestMismatch)
        )
    };

    let reordered_events_rejected = {
        let mut reordered = registry.events().to_vec();
        reordered.swap(0, 1);
        matches!(
            SemanticProgramRegistry::replay(&reordered),
            Err(SemanticProgramError::ReplayVersionMismatch)
        )
    };

    let deleted_event_rejected = matches!(
        SemanticProgramRegistry::replay(&[registry.events()[1].clone()]),
        Err(SemanticProgramError::ReplayVersionMismatch)
    );

    let authority = authority_boundary();
    let authority_closed = !authority.runtime_chat_wiring
        && !authority.generated_text_influence
        && !authority.persistence_authority
        && !authority.routing_authority
        && !authority.companion_mutation_authority
        && !authority.belief_promotion_authority
        && !authority.ontology_promotion_authority
        && !authority.tool_selection_authority
        && !authority.charge_discharge_authority
        && !authority.autonomous_action_authority;

    let gate_passed = deterministic_validation
        && canonical_bytes_identical
        && digest_identical
        && all_operation_variants_present
        && first_commit
        && second_commit
        && exact_replay
        && repeated_replay
        && duplicate_operation_rejected
        && unknown_claim_rejected
        && authorized_prohibited_overlap_rejected
        && invalid_confidence_rejected
        && qualifier_mismatch_rejected
        && stale_state_rejected
        && stale_companion_rejected
        && subject_mismatch_rejected
        && sensitivity_scope_rejected
        && sensitivity_level_rejected
        && noncanonical_claim_order_rejected
        && noncanonical_epistemic_order_rejected
        && invalid_output_budget_rejected
        && exceeded_compute_budget_rejected
        && duplicate_program_rejected
        && stale_registry_version_rejected
        && digest_tampering_rejected
        && reordered_events_rejected
        && deleted_event_rejected
        && authority_closed;

    let report = json!({
        "experiment": "STLM_L0_SEMANTIC_PROGRAM",
        "terminal_classification": if gate_passed { "PASS" } else { "FAIL" },
        "deterministic_validation": deterministic_validation,
        "canonical_bytes_identical": canonical_bytes_identical,
        "digest_identical": digest_identical,
        "all_operation_variants_present": all_operation_variants_present,
        "first_commit": first_commit,
        "second_commit": second_commit,
        "exact_replay": exact_replay,
        "repeated_replay": repeated_replay,
        "duplicate_operation_rejected_atomically": duplicate_operation_rejected,
        "unknown_claim_rejected_atomically": unknown_claim_rejected,
        "authorized_prohibited_overlap_rejected_atomically": authorized_prohibited_overlap_rejected,
        "invalid_confidence_rejected_atomically": invalid_confidence_rejected,
        "qualifier_mismatch_rejected_atomically": qualifier_mismatch_rejected,
        "stale_state_rejected_atomically": stale_state_rejected,
        "stale_companion_rejected_atomically": stale_companion_rejected,
        "subject_mismatch_rejected_atomically": subject_mismatch_rejected,
        "sensitivity_scope_rejected_atomically": sensitivity_scope_rejected,
        "sensitivity_level_rejected_atomically": sensitivity_level_rejected,
        "noncanonical_claim_order_rejected_atomically": noncanonical_claim_order_rejected,
        "noncanonical_epistemic_order_rejected_atomically": noncanonical_epistemic_order_rejected,
        "invalid_output_budget_rejected_atomically": invalid_output_budget_rejected,
        "exceeded_compute_budget_rejected_atomically": exceeded_compute_budget_rejected,
        "duplicate_program_rejected_atomically": duplicate_program_rejected,
        "stale_registry_version_rejected_atomically": stale_registry_version_rejected,
        "digest_tampering_rejected": digest_tampering_rejected,
        "reordered_events_rejected": reordered_events_rejected,
        "deleted_event_rejected": deleted_event_rejected,
        "authority_boundary_closed": authority_closed,
        "registry_version": registry.version(),
        "program_count": registry.programs().len(),
        "event_count": registry.events().len(),
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
