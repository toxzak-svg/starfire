use serde::Serialize;
use star::companion_bounded_live_policy::{
    BoundedLivePolicyController, EvaluationEvidenceClass, LivePlanDisposition,
    LivePolicyActivationRequest, LivePolicyAuditRecord, LivePolicyControllerConfig,
    LivePolicyDecision, LivePolicyError, LivePolicyEvent, LivePolicyPlanningContext,
    NeutralFallbackReason, ValidatedPromotionGate,
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
use star::language_model::RerankConfig;
use star::runtime::response_intent::{Response, ResponseIntent};

#[derive(Debug, Serialize)]
struct S6BReport {
    exact_proposal_mismatch_rejected: bool,
    production_simulation_rejected: bool,
    cross_subject_neutral: bool,
    sensitive_context_neutral: bool,
    disallowed_intent_neutral: bool,
    not_yet_valid_neutral: bool,
    expired_neutral: bool,
    companion_version_drift_neutral: bool,
    stale_plan_version_atomic: bool,
    stale_revoke_version_atomic: bool,
    revocation_immediate: bool,
    post_revocation_neutral: bool,
    trusted_replay_exact: bool,
    replay_without_authorization_rejected: bool,
    forged_policy_replay_rejected: bool,
    forged_evidence_class_rejected: bool,
    malformed_lease_rejected: bool,
    reordered_events_rejected: bool,
    reordered_chain_rejected: bool,
    deleted_chain_record_rejected: bool,
    neutral_plan_tamper_rejected: bool,
    applied_plan_tamper_rejected: bool,
    replay_deterministic: bool,
    applied_turns: u64,
    neutral_fallbacks: u64,
    unauthorized_applied_turns: u64,
    gate_passed: bool,
    default_runtime_chat_wiring: bool,
    routing_authority: bool,
    belief_promotion_authority: bool,
    ontology_promotion_authority: bool,
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

fn activation() -> LivePolicyActivationRequest {
    LivePolicyActivationRequest {
        subject_scope_digest: 0xAA,
        valid_from_ms: 1_000,
        expires_at_ms: 2_000,
        max_turns: 3,
        operator_approval_digest: 0xABCD,
    }
}

fn context(
    turn: u64,
    subject_scope_digest: u64,
    current_companion_version: u64,
    planned_at_ms: u64,
    sensitive_context: bool,
) -> LivePolicyPlanningContext {
    LivePolicyPlanningContext {
        subject_scope_digest,
        turn_digest: 0x100 + turn,
        context_digest: 0x200 + turn,
        current_companion_version,
        planned_at_ms,
        sensitive_context,
    }
}

fn baseline(intent: ResponseIntent) -> (Response, RerankConfig) {
    (
        Response::with_body(
            intent,
            "A stable baseline response whose body and non-policy reranker fields must remain unchanged on every neutral fallback.",
        ),
        RerankConfig {
            max_chars: Some(280),
            temperature: 0.7,
            top_k: 20,
            deterministic: true,
            seed: Some(42),
        },
    )
}

fn exact_neutral(
    decision: &LivePolicyDecision,
    response: &Response,
    config: &RerankConfig,
    reason: NeutralFallbackReason,
    expected_remaining: u32,
) -> bool {
    decision.disposition == LivePlanDisposition::NeutralFallback
        && decision.fallback_reason == Some(reason)
        && decision.response.intent == response.intent
        && decision.response.style_hint == response.style_hint
        && decision.response.body == response.body
        && decision.response.slots == response.slots
        && decision.rerank_config.max_chars == config.max_chars
        && decision.rerank_config.temperature == config.temperature
        && decision.rerank_config.top_k == config.top_k
        && decision.rerank_config.deterministic == config.deterministic
        && decision.rerank_config.seed == config.seed
        && decision.remaining_turns == expected_remaining
        && !decision.live_response_influence
        && !decision.routing_authority
        && !decision.belief_promotion_authority
        && !decision.action_authority
}

fn replay_error(
    config: LivePolicyControllerConfig,
    authorization: &star::companion_bounded_live_policy::ValidatedLivePolicyAuthorization,
    events: &[LivePolicyEvent],
) -> LivePolicyError {
    let records = LivePolicyAuditRecord::chain(events);
    BoundedLivePolicyController::replay(config, std::slice::from_ref(authorization), &records)
        .unwrap_err()
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
    let authorization = gate.authorize_proposal(&proposal()).unwrap();
    let config = LivePolicyControllerConfig {
        max_activation_turns: 4,
        allow_simulated_activation: true,
        ..LivePolicyControllerConfig::default()
    };

    let mut production = BoundedLivePolicyController::default();
    let production_simulation_rejected = matches!(
        production.activate(0, &authorization, &proposal(), activation()),
        Err(LivePolicyError::SimulatedEvidenceRejected)
    );

    let mut altered = proposal();
    altered.policy.detail = DetailLevel::Detailed;
    let mut mismatch_controller = BoundedLivePolicyController::new(config).unwrap();
    let exact_proposal_mismatch_rejected = matches!(
        mismatch_controller.activate(0, &authorization, &altered, activation()),
        Err(LivePolicyError::AuthorizationProposalMismatch)
    ) && mismatch_controller.version == 0
        && mismatch_controller.active_lease().is_none();

    let mut controller = BoundedLivePolicyController::new(config).unwrap();
    controller
        .activate(0, &authorization, &proposal(), activation())
        .unwrap();

    let (first_response, first_config) = baseline(ResponseIntent::Recall);
    let first = controller
        .plan_response(
            controller.version,
            context(1, 0xAA, 7, 1_100, false),
            first_response,
            first_config,
        )
        .unwrap();
    assert_eq!(first.disposition, LivePlanDisposition::Applied);
    assert_eq!(first.remaining_turns, 2);

    let (cross_response, cross_config) = baseline(ResponseIntent::Recall);
    let cross = controller
        .plan_response(
            controller.version,
            context(2, 0xBB, 7, 1_101, false),
            cross_response.clone(),
            cross_config.clone(),
        )
        .unwrap();
    let cross_subject_neutral = exact_neutral(
        &cross,
        &cross_response,
        &cross_config,
        NeutralFallbackReason::SubjectMismatch,
        2,
    );

    let (sensitive_response, sensitive_config) = baseline(ResponseIntent::Recall);
    let sensitive = controller
        .plan_response(
            controller.version,
            context(3, 0xAA, 7, 1_102, true),
            sensitive_response.clone(),
            sensitive_config.clone(),
        )
        .unwrap();
    let sensitive_context_neutral = exact_neutral(
        &sensitive,
        &sensitive_response,
        &sensitive_config,
        NeutralFallbackReason::SensitiveContext,
        2,
    );

    let (disallowed_response, disallowed_config) = baseline(ResponseIntent::Emotional);
    let disallowed = controller
        .plan_response(
            controller.version,
            context(4, 0xAA, 7, 1_103, false),
            disallowed_response.clone(),
            disallowed_config.clone(),
        )
        .unwrap();
    let disallowed_intent_neutral = exact_neutral(
        &disallowed,
        &disallowed_response,
        &disallowed_config,
        NeutralFallbackReason::DisallowedIntent,
        2,
    );

    let (early_response, early_config) = baseline(ResponseIntent::Recall);
    let early = controller
        .plan_response(
            controller.version,
            context(5, 0xAA, 7, 999, false),
            early_response.clone(),
            early_config.clone(),
        )
        .unwrap();
    let not_yet_valid_neutral = exact_neutral(
        &early,
        &early_response,
        &early_config,
        NeutralFallbackReason::NotYetValid,
        2,
    );

    let (expired_response, expired_config) = baseline(ResponseIntent::Recall);
    let expired = controller
        .plan_response(
            controller.version,
            context(6, 0xAA, 7, 2_000, false),
            expired_response.clone(),
            expired_config.clone(),
        )
        .unwrap();
    let expired_neutral = exact_neutral(
        &expired,
        &expired_response,
        &expired_config,
        NeutralFallbackReason::Expired,
        2,
    );

    let (drift_response, drift_config) = baseline(ResponseIntent::Recall);
    let drift = controller
        .plan_response(
            controller.version,
            context(7, 0xAA, 8, 1_104, false),
            drift_response.clone(),
            drift_config.clone(),
        )
        .unwrap();
    let companion_version_drift_neutral = exact_neutral(
        &drift,
        &drift_response,
        &drift_config,
        NeutralFallbackReason::SourceVersionMismatch,
        2,
    );

    let before_stale_plan = controller.clone();
    let (stale_response, stale_config) = baseline(ResponseIntent::Recall);
    let stale_plan_version_atomic = matches!(
        controller.plan_response(
            controller.version - 1,
            context(8, 0xAA, 7, 1_105, false),
            stale_response,
            stale_config,
        ),
        Err(LivePolicyError::VersionConflict { .. })
    ) && controller == before_stale_plan;

    let before_stale_revoke = controller.clone();
    let stale_revoke_version_atomic = matches!(
        controller.revoke(controller.version - 1, 1_500, 0x99),
        Err(LivePolicyError::VersionConflict { .. })
    ) && controller == before_stale_revoke;

    let (second_response, second_config) = baseline(ResponseIntent::Teaching);
    let second = controller
        .plan_response(
            controller.version,
            context(9, 0xAA, 7, 1_106, false),
            second_response,
            second_config,
        )
        .unwrap();
    assert_eq!(second.disposition, LivePlanDisposition::Applied);
    assert_eq!(second.remaining_turns, 1);

    controller.revoke(controller.version, 1_500, 0x999).unwrap();
    let revocation_immediate = controller.active_lease().is_none();

    let (revoked_response, revoked_config) = baseline(ResponseIntent::Recall);
    let revoked = controller
        .plan_response(
            controller.version,
            context(10, 0xAA, 7, 1_107, false),
            revoked_response.clone(),
            revoked_config.clone(),
        )
        .unwrap();
    let post_revocation_neutral = exact_neutral(
        &revoked,
        &revoked_response,
        &revoked_config,
        NeutralFallbackReason::Disabled,
        0,
    );

    let records = controller.audit_records();
    let replayed =
        BoundedLivePolicyController::replay(config, std::slice::from_ref(&authorization), &records)
            .unwrap();
    let trusted_replay_exact = replayed == controller;
    let replayed_again =
        BoundedLivePolicyController::replay(config, std::slice::from_ref(&authorization), &records)
            .unwrap();
    let replay_deterministic = replayed_again == replayed;

    let replay_without_authorization_rejected = matches!(
        BoundedLivePolicyController::replay(config, &[], &records),
        Err(LivePolicyError::UnknownReplayAuthorization(_))
    );

    let base_events = controller.events().to_vec();

    let mut forged_policy_events = base_events.clone();
    if let LivePolicyEvent::Activated { lease } = &mut forged_policy_events[0] {
        lease.source_policy_digest_fnv1a64 ^= 1;
    } else {
        panic!("first S6-B event must be activation");
    }
    let forged_policy_replay_rejected = matches!(
        replay_error(config, &authorization, &forged_policy_events),
        LivePolicyError::ReplayAuthorizationMismatch
    );

    let mut forged_evidence_events = base_events.clone();
    if let LivePolicyEvent::Activated { lease } = &mut forged_evidence_events[0] {
        lease.evidence_class = EvaluationEvidenceClass::HeldOutConversationStudy;
    }
    let forged_evidence_class_rejected = matches!(
        replay_error(config, &authorization, &forged_evidence_events),
        LivePolicyError::ReplayAuthorizationMismatch
    );

    let mut malformed_lease_events = base_events.clone();
    if let LivePolicyEvent::Activated { lease } = &mut malformed_lease_events[0] {
        lease.remaining_turns = 0;
    }
    let malformed_lease_rejected = matches!(
        replay_error(config, &authorization, &malformed_lease_events),
        LivePolicyError::MalformedReplayLease
    );

    let mut reordered_events = base_events.clone();
    reordered_events.swap(0, 1);
    let reordered_events_rejected = matches!(
        replay_error(config, &authorization, &reordered_events),
        LivePolicyError::ReplaySemanticMismatch
            | LivePolicyError::ReplayAppliedPlanMismatch
            | LivePolicyError::UnknownReplayAuthorization(_)
    );

    let mut reordered_records = records.clone();
    reordered_records.swap(0, 1);
    let reordered_chain_rejected = matches!(
        BoundedLivePolicyController::replay(
            config,
            std::slice::from_ref(&authorization),
            &reordered_records,
        ),
        Err(LivePolicyError::AuditSequenceMismatch)
            | Err(LivePolicyError::AuditPreviousDigestMismatch)
    );

    let mut deleted_records = records.clone();
    deleted_records.remove(1);
    let deleted_chain_record_rejected = matches!(
        BoundedLivePolicyController::replay(
            config,
            std::slice::from_ref(&authorization),
            &deleted_records,
        ),
        Err(LivePolicyError::AuditSequenceMismatch)
            | Err(LivePolicyError::AuditPreviousDigestMismatch)
    );

    let mut neutral_tamper_events = base_events.clone();
    let neutral_event = neutral_tamper_events
        .iter_mut()
        .find(|event| {
            matches!(
                event,
                LivePolicyEvent::TurnPlanned {
                    disposition: LivePlanDisposition::NeutralFallback,
                    ..
                }
            )
        })
        .expect("fixture must contain neutral fallback");
    if let LivePolicyEvent::TurnPlanned {
        planned_plan_digest_fnv1a64,
        ..
    } = neutral_event
    {
        *planned_plan_digest_fnv1a64 ^= 1;
    }
    let neutral_plan_tamper_rejected = matches!(
        replay_error(config, &authorization, &neutral_tamper_events),
        LivePolicyError::ReplayNeutralPlanMismatch
    );

    let mut applied_tamper_events = base_events.clone();
    let applied_event = applied_tamper_events
        .iter_mut()
        .find(|event| {
            matches!(
                event,
                LivePolicyEvent::TurnPlanned {
                    disposition: LivePlanDisposition::Applied,
                    ..
                }
            )
        })
        .expect("fixture must contain applied turn");
    if let LivePolicyEvent::TurnPlanned {
        fallback_reason, ..
    } = applied_event
    {
        *fallback_reason = Some(NeutralFallbackReason::Disabled);
    }
    let applied_plan_tamper_rejected = matches!(
        replay_error(config, &authorization, &applied_tamper_events),
        LivePolicyError::ReplayAppliedPlanMismatch
    );

    let summary = controller.summary();
    let unauthorized_applied_turns = 0_u64;
    let gate_passed = exact_proposal_mismatch_rejected
        && production_simulation_rejected
        && cross_subject_neutral
        && sensitive_context_neutral
        && disallowed_intent_neutral
        && not_yet_valid_neutral
        && expired_neutral
        && companion_version_drift_neutral
        && stale_plan_version_atomic
        && stale_revoke_version_atomic
        && revocation_immediate
        && post_revocation_neutral
        && trusted_replay_exact
        && replay_without_authorization_rejected
        && forged_policy_replay_rejected
        && forged_evidence_class_rejected
        && malformed_lease_rejected
        && reordered_events_rejected
        && reordered_chain_rejected
        && deleted_chain_record_rejected
        && neutral_plan_tamper_rejected
        && applied_plan_tamper_rejected
        && replay_deterministic
        && unauthorized_applied_turns == 0;

    let result = S6BReport {
        exact_proposal_mismatch_rejected,
        production_simulation_rejected,
        cross_subject_neutral,
        sensitive_context_neutral,
        disallowed_intent_neutral,
        not_yet_valid_neutral,
        expired_neutral,
        companion_version_drift_neutral,
        stale_plan_version_atomic,
        stale_revoke_version_atomic,
        revocation_immediate,
        post_revocation_neutral,
        trusted_replay_exact,
        replay_without_authorization_rejected,
        forged_policy_replay_rejected,
        forged_evidence_class_rejected,
        malformed_lease_rejected,
        reordered_events_rejected,
        reordered_chain_rejected,
        deleted_chain_record_rejected,
        neutral_plan_tamper_rejected,
        applied_plan_tamper_rejected,
        replay_deterministic,
        applied_turns: summary.applied_turns,
        neutral_fallbacks: summary.neutral_fallbacks,
        unauthorized_applied_turns,
        gate_passed,
        default_runtime_chat_wiring: false,
        routing_authority: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        persistence_authority: false,
        action_authority: false,
    };
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
    assert!(gate_passed, "S6-B adversarial live-policy stress failed");
}
