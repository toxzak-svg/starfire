use serde::Serialize;
use star::companion_bounded_live_policy::{
    BoundedLivePolicyController, EvaluationEvidenceClass, LivePlanDisposition,
    LivePolicyActivationRequest, LivePolicyControllerConfig, LivePolicyError,
    LivePolicyPlanningContext, NeutralFallbackReason, ValidatedPromotionGate,
};
use star::companion_interaction_policy::{
    AcknowledgmentLevel, DetailLevel, DialogueMode, ExplanationStyle, InteractionPolicy,
    PolicyEvidence, PolicyVariant, ShadowPolicyProposal, VocabularyLevel,
};
use star::companion_policy_evaluation::{
    CandidateControlComparison, CandidatePairwiseMetrics, ComparisonGates, EvaluationSplit,
    PolicyEvaluationReport, PolicyEvaluationVerdict, SplitEvaluationReport,
};
use star::companion_state::Sensitivity;
use star::language_model::{IntentReranker, MockReranker, RerankConfig};
use star::runtime::response_intent::{Response, ResponseIntent};
use star::voice::InternalState;

#[derive(Debug, Serialize)]
struct S6AReport {
    promotion_gate_validated: bool,
    production_default_rejected_simulation: bool,
    explicit_simulation_override_required: bool,
    activation_version_mismatch_rejected: bool,
    stale_companion_version_used_exact_neutral_fallback: bool,
    applied_plan_changed_metadata_only: bool,
    reranked_output_respected_brief_budget: bool,
    sensitive_context_used_exact_neutral_fallback: bool,
    duplicate_turn_used_neutral_fallback: bool,
    disallowed_intent_used_neutral_fallback: bool,
    turn_budget_enforced: bool,
    revocation_immediate: bool,
    replay_equal: bool,
    applied_turns: u64,
    neutral_fallbacks: u64,
    remaining_turns: u32,
    gate_passed: bool,
    default_runtime_chat_wiring: bool,
    routing_authority: bool,
    belief_promotion_authority: bool,
    persistence_authority: bool,
    action_authority: bool,
}

fn controls() -> [PolicyVariant; 5] {
    [
        PolicyVariant::NeutralDefault,
        PolicyVariant::RecencyOnly,
        PolicyVariant::MajorityPrior,
        PolicyVariant::ContextOnly,
        PolicyVariant::ScrambledScope,
    ]
}

fn passing_gates() -> ComparisonGates {
    ComparisonGates {
        resolved_evidence_sufficient: true,
        direct_evidence_sufficient: true,
        pairwise_evidence_sufficient: true,
        brier_improvement_passed: true,
        calibration_non_regression_passed: true,
        pairwise_margin_passed: true,
        correction_non_regression_passed: true,
        clarification_non_regression_passed: true,
        completion_non_regression_passed: true,
        abandonment_non_regression_passed: true,
        abstention_non_regression_passed: true,
        compute_overhead_passed: true,
    }
}

fn passing_report() -> PolicyEvaluationReport {
    let mut holdout_comparisons = Vec::new();
    for split in [
        EvaluationSplit::OpaqueSubjectHoldout,
        EvaluationSplit::TemporalHoldout,
    ] {
        for control in controls() {
            holdout_comparisons.push(CandidateControlComparison {
                split,
                control,
                candidate_resolved: 8,
                control_resolved: 8,
                candidate_direct_outcomes: 4,
                control_direct_outcomes: 4,
                pairwise: CandidatePairwiseMetrics {
                    candidate_wins: 6,
                    control_wins: 2,
                    ties: 0,
                    total: 8,
                    candidate_win_margin_bps: Some(5_000),
                },
                brier_improvement_ppm: Some(20_000),
                calibration_regression_bps: Some(0),
                correction_regression_bps: Some(0),
                clarification_regression_bps: Some(0),
                completion_regression_bps: Some(0),
                abandonment_regression_bps: Some(0),
                abstention_regression_bps: Some(0),
                compute_overhead_bps: Some(500),
                gates: passing_gates(),
            });
        }
    }
    PolicyEvaluationReport {
        splits: vec![SplitEvaluationReport {
            split: EvaluationSplit::Development,
            arms: Default::default(),
            candidate_pairwise: Default::default(),
        }],
        holdout_comparisons,
        verdict: PolicyEvaluationVerdict::Pass,
        development_excluded_from_verdict: true,
        promotion_eligible: true,
        live_response_influence: false,
        routing_authority: false,
        belief_promotion_authority: false,
        action_authority: false,
    }
}

fn proposal() -> ShadowPolicyProposal {
    ShadowPolicyProposal {
        variant: PolicyVariant::CompanionDerived,
        source_companion_version: 7,
        policy: InteractionPolicy {
            detail: DetailLevel::Brief,
            explanation_style: ExplanationStyle::Concrete,
            dialogue: DialogueMode::Direct,
            vocabulary: VocabularyLevel::Technical,
            acknowledgment: AcknowledgmentLevel::Minimal,
        },
        evidence: vec![PolicyEvidence {
            claim_id: 11,
            key: "preference.brevity.general".to_owned(),
            confidence_bps: 9_200,
            updated_at_ms: 900,
            sensitivity: Sensitivity::Personal,
        }],
        confidence_bps: 9_200,
        context_digest: 0x1001,
        policy_digest_fnv1a64: 0x2002,
        predicted_outcomes: Vec::new(),
        abstention_reason: None,
    }
}

fn context(turn: u64, intent_sensitive: bool) -> LivePolicyPlanningContext {
    LivePolicyPlanningContext {
        subject_scope_digest: 0xAA,
        turn_digest: 0x100 + turn,
        context_digest: 0x200 + turn,
        current_companion_version: 7,
        planned_at_ms: 1_100 + turn,
        sensitive_context: intent_sensitive,
    }
}

fn baseline(intent: ResponseIntent) -> (Response, RerankConfig) {
    let body = "This is a deliberately long response body that contains enough material to exceed the brief response budget. It explains several details, repeats context, and continues so the bounded reranker must truncate the final result without changing the response's factual body before reranking.";
    (
        Response::with_body(intent, body),
        RerankConfig {
            max_chars: Some(280),
            temperature: 0.7,
            top_k: 20,
            deterministic: true,
            seed: Some(42),
        },
    )
}

fn main() {
    let report = passing_report();
    let gate = ValidatedPromotionGate::validate(
        &report,
        EvaluationEvidenceClass::FrozenSimulation,
        0xF00D,
        7,
    )
    .unwrap();
    let promotion_gate_validated = gate.digest_fnv1a64() != 0;
    let authorization = gate.authorize_proposal(&proposal()).unwrap();

    let mut production = BoundedLivePolicyController::default();
    let production_default_rejected_simulation = production
        .activate(
            0,
            &authorization,
            &proposal(),
            LivePolicyActivationRequest {
                subject_scope_digest: 0xAA,
                valid_from_ms: 1_000,
                expires_at_ms: 2_000,
                max_turns: 2,
                operator_approval_digest: 0xABCD,
            },
        )
        .is_err();

    let mismatch_config = LivePolicyControllerConfig {
        max_activation_turns: 3,
        allow_simulated_activation: true,
        ..LivePolicyControllerConfig::default()
    };
    let mut mismatch_controller = BoundedLivePolicyController::new(mismatch_config).unwrap();
    let mut mismatched_proposal = proposal();
    mismatched_proposal.source_companion_version = 8;
    let activation_version_mismatch_rejected = matches!(
        mismatch_controller.activate(
            0,
            &authorization,
            &mismatched_proposal,
            LivePolicyActivationRequest {
                subject_scope_digest: 0xAA,
                valid_from_ms: 1_000,
                expires_at_ms: 2_000,
                max_turns: 2,
                operator_approval_digest: 0xABCD,
            },
        ),
        Err(LivePolicyError::AuthorizationProposalMismatch)
    );

    let config = LivePolicyControllerConfig {
        max_activation_turns: 3,
        allow_simulated_activation: true,
        ..LivePolicyControllerConfig::default()
    };
    let mut controller = BoundedLivePolicyController::new(config).unwrap();
    controller
        .activate(
            0,
            &authorization,
            &proposal(),
            LivePolicyActivationRequest {
                subject_scope_digest: 0xAA,
                valid_from_ms: 1_000,
                expires_at_ms: 2_000,
                max_turns: 2,
                operator_approval_digest: 0xABCD,
            },
        )
        .unwrap();
    let explicit_simulation_override_required = controller.active_lease().is_some();

    let (response, rerank_config) = baseline(ResponseIntent::Recall);
    let original_body = response.body.clone();
    let applied = controller
        .plan_response(
            controller.version,
            context(1, false),
            response,
            rerank_config,
        )
        .unwrap();
    let applied_plan_changed_metadata_only = applied.disposition == LivePlanDisposition::Applied
        && applied.response.body == original_body
        && applied.rerank_config.max_chars == Some(160)
        && applied.remaining_turns == 1
        && !applied.routing_authority
        && !applied.belief_promotion_authority
        && !applied.action_authority;

    let reranker = IntentReranker::new(Box::new(MockReranker), RerankConfig::default());
    let reranked = reranker.rerank_with_config(
        &applied.response,
        &InternalState::default(),
        &applied.rerank_config,
    );
    let reranked_output_respected_brief_budget = reranked.chars().count() <= 161;

    let (stale_response, stale_config) = baseline(ResponseIntent::Recall);
    let stale_body = stale_response.body.clone();
    let stale_max = stale_config.max_chars;
    let mut stale_context = context(6, false);
    stale_context.current_companion_version = 8;
    let stale = controller
        .plan_response(
            controller.version,
            stale_context,
            stale_response,
            stale_config,
        )
        .unwrap();
    let stale_companion_version_used_exact_neutral_fallback = stale.disposition
        == LivePlanDisposition::NeutralFallback
        && stale.fallback_reason == Some(NeutralFallbackReason::SourceVersionMismatch)
        && stale.response.body == stale_body
        && stale.rerank_config.max_chars == stale_max
        && stale.remaining_turns == 1;

    let (duplicate_response, duplicate_config) = baseline(ResponseIntent::Recall);
    let duplicate = controller
        .plan_response(
            controller.version,
            context(1, false),
            duplicate_response,
            duplicate_config,
        )
        .unwrap();
    let duplicate_turn_used_neutral_fallback = duplicate.disposition
        == LivePlanDisposition::NeutralFallback
        && duplicate.fallback_reason == Some(NeutralFallbackReason::DuplicateTurn)
        && duplicate.remaining_turns == 1;

    let (sensitive_response, sensitive_config) = baseline(ResponseIntent::Recall);
    let sensitive_body = sensitive_response.body.clone();
    let sensitive_max = sensitive_config.max_chars;
    let sensitive = controller
        .plan_response(
            controller.version,
            context(2, true),
            sensitive_response,
            sensitive_config,
        )
        .unwrap();
    let sensitive_context_used_exact_neutral_fallback = sensitive.disposition
        == LivePlanDisposition::NeutralFallback
        && sensitive.fallback_reason == Some(NeutralFallbackReason::SensitiveContext)
        && sensitive.response.body == sensitive_body
        && sensitive.rerank_config.max_chars == sensitive_max
        && sensitive.remaining_turns == 1;

    let (emotional_response, emotional_config) = baseline(ResponseIntent::Emotional);
    let disallowed = controller
        .plan_response(
            controller.version,
            context(3, false),
            emotional_response,
            emotional_config,
        )
        .unwrap();
    let disallowed_intent_used_neutral_fallback = disallowed.disposition
        == LivePlanDisposition::NeutralFallback
        && disallowed.fallback_reason == Some(NeutralFallbackReason::DisallowedIntent)
        && disallowed.remaining_turns == 1;

    let (second_response, second_config) = baseline(ResponseIntent::Teaching);
    let second = controller
        .plan_response(
            controller.version,
            context(4, false),
            second_response,
            second_config,
        )
        .unwrap();
    let (exhausted_response, exhausted_config) = baseline(ResponseIntent::Recall);
    let exhausted = controller
        .plan_response(
            controller.version,
            context(5, false),
            exhausted_response,
            exhausted_config,
        )
        .unwrap();
    let turn_budget_enforced = second.disposition == LivePlanDisposition::Applied
        && second.remaining_turns == 0
        && exhausted.disposition == LivePlanDisposition::NeutralFallback
        && exhausted.fallback_reason == Some(NeutralFallbackReason::BudgetExhausted);

    controller.revoke(controller.version, 1_500, 0x999).unwrap();
    let revocation_immediate = controller.active_lease().is_none();
    let replayed = BoundedLivePolicyController::replay(
        config,
        std::slice::from_ref(&authorization),
        &controller.audit_records(),
    )
    .unwrap();
    let replay_equal = replayed == controller;
    let summary = controller.summary();

    let gate_passed = promotion_gate_validated
        && production_default_rejected_simulation
        && explicit_simulation_override_required
        && activation_version_mismatch_rejected
        && stale_companion_version_used_exact_neutral_fallback
        && applied_plan_changed_metadata_only
        && reranked_output_respected_brief_budget
        && sensitive_context_used_exact_neutral_fallback
        && duplicate_turn_used_neutral_fallback
        && disallowed_intent_used_neutral_fallback
        && turn_budget_enforced
        && revocation_immediate
        && replay_equal;

    let result = S6AReport {
        promotion_gate_validated,
        production_default_rejected_simulation,
        explicit_simulation_override_required,
        activation_version_mismatch_rejected,
        stale_companion_version_used_exact_neutral_fallback,
        applied_plan_changed_metadata_only,
        reranked_output_respected_brief_budget,
        sensitive_context_used_exact_neutral_fallback,
        duplicate_turn_used_neutral_fallback,
        disallowed_intent_used_neutral_fallback,
        turn_budget_enforced,
        revocation_immediate,
        replay_equal,
        applied_turns: summary.applied_turns,
        neutral_fallbacks: summary.neutral_fallbacks,
        remaining_turns: summary.remaining_turns,
        gate_passed,
        default_runtime_chat_wiring: false,
        routing_authority: false,
        belief_promotion_authority: false,
        persistence_authority: false,
        action_authority: false,
    };
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
    assert!(gate_passed, "S6-A bounded live-policy gate failed");
}
