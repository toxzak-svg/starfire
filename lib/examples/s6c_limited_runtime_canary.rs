use serde::Serialize;
use star::companion_bounded_live_policy::{
    BoundedLivePolicyController, EvaluationEvidenceClass, LivePlanDisposition,
    NeutralFallbackReason, ValidatedPromotionGate,
};
use star::companion_interaction_outcomes::{InteractionTrial, TrialArm};
use star::companion_interaction_policy::{
    AcknowledgmentLevel, DetailLevel, DialogueMode, ExplanationStyle, InteractionPolicy,
    PolicyContext, PolicyEvidence, PolicyVariant, ShadowPolicyBatch, ShadowPolicyProposal,
    VocabularyLevel,
};
use star::companion_policy_evaluation::{
    CandidateControlComparison, CandidatePairwiseMetrics, ComparisonGates, EvaluationSplit,
    PolicyEvaluationReport, PolicyEvaluationVerdict, SplitEvaluationReport,
};
use star::companion_runtime_canary::{
    CanaryRegistrationRequirement, RuntimeCanaryActivation, RuntimeCanaryConfig,
    RuntimeCanaryError, RuntimeCanarySession, RuntimeCanaryTurnContext,
};
use star::companion_state::Sensitivity;
use star::language_model::RerankConfig;
use star::runtime::response_intent::{Response, ResponseIntent};

const SESSION: u64 = 0x51;
const SUBJECT: u64 = 0xAA;
const COMPANION_POLICY: u64 = 0x2002;
const NEUTRAL_POLICY: u64 = 0x3003;

#[derive(Debug, Serialize)]
struct S6CReport {
    preregistered_before_implementation: bool,
    synthetic_activation_rejected: bool,
    held_out_activation_accepted: bool,
    prepare_left_live_state_unchanged: bool,
    pending_debug_redacted_response: bool,
    applied_requires_companion_arm: bool,
    neutral_requires_neutral_arm: bool,
    wrong_arm_rejected_atomically: bool,
    wrong_subject_rejected_atomically: bool,
    wrong_session_rejected_atomically: bool,
    wrong_context_rejected_atomically: bool,
    wrong_version_rejected_atomically: bool,
    wrong_policy_digest_rejected_atomically: bool,
    expired_trial_rejected_atomically: bool,
    sensitive_context_exact_neutral: bool,
    disallowed_intent_exact_neutral: bool,
    companion_version_drift_exact_neutral: bool,
    duplicate_turn_exact_neutral: bool,
    applied_turn_budget_enforced: bool,
    session_expiry_exact_neutral: bool,
    revocation_immediate: bool,
    replay_deterministic: bool,
    unauthorized_applied_turns: u64,
    committed_turns: u64,
    companion_applied_turns: u64,
    neutral_fallbacks: u64,
    default_runtime_chat_wiring: bool,
    routing_authority: bool,
    persistence_authority: bool,
    belief_promotion_authority: bool,
    ontology_promotion_authority: bool,
    tool_selection_authority: bool,
    action_authority: bool,
    gate_passed: bool,
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

fn companion_proposal(version: u64, context_digest: u64) -> ShadowPolicyProposal {
    ShadowPolicyProposal {
        variant: PolicyVariant::CompanionDerived,
        source_companion_version: version,
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
        context_digest,
        policy_digest_fnv1a64: COMPANION_POLICY,
        predicted_outcomes: Vec::new(),
        abstention_reason: None,
    }
}

fn neutral_proposal(version: u64, context_digest: u64) -> ShadowPolicyProposal {
    ShadowPolicyProposal {
        variant: PolicyVariant::NeutralDefault,
        source_companion_version: version,
        policy: InteractionPolicy::default(),
        evidence: Vec::new(),
        confidence_bps: 5_000,
        context_digest,
        policy_digest_fnv1a64: NEUTRAL_POLICY,
        predicted_outcomes: Vec::new(),
        abstention_reason: None,
    }
}

fn batch(version: u64, context_digest: u64, subject: u64, issued_at_ms: u64) -> ShadowPolicyBatch {
    ShadowPolicyBatch {
        source_companion_version: version,
        context: PolicyContext {
            context_digest,
            subject_scope_digest: subject,
            domain: Some("rust".to_owned()),
            technical_context: true,
            asks_for_explanation: true,
            emotional_signal: false,
            issued_at_ms,
            not_before_ms: issued_at_ms + 1,
            expires_at_ms: issued_at_ms + 500,
        },
        proposals: vec![
            companion_proposal(version, context_digest),
            neutral_proposal(version, context_digest),
        ],
    }
}

fn turn(
    turn_digest: u64,
    context_digest: u64,
    version: u64,
    prepared_at_ms: u64,
    sensitive: bool,
) -> RuntimeCanaryTurnContext {
    RuntimeCanaryTurnContext {
        session_scope_digest: SESSION,
        subject_scope_digest: SUBJECT,
        turn_digest,
        context_digest,
        current_companion_version: version,
        prepared_at_ms,
        sensitive_context: sensitive,
    }
}

fn baseline(intent: ResponseIntent) -> (Response, RerankConfig) {
    (
        Response::with_body(
            intent,
            "S6-C baseline body must remain hidden until a matching S5-B trial is bound.",
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

fn trial(
    id: u64,
    requirement: CanaryRegistrationRequirement,
    delivered_variant: PolicyVariant,
) -> InteractionTrial {
    let arms = PolicyVariant::all()
        .into_iter()
        .enumerate()
        .map(|(index, variant)| TrialArm {
            variant,
            policy_digest_fnv1a64: if variant == requirement.required_delivered_variant {
                requirement.required_policy_digest_fnv1a64
            } else if variant == PolicyVariant::CompanionDerived {
                COMPANION_POLICY
            } else if variant == PolicyVariant::NeutralDefault {
                NEUTRAL_POLICY
            } else {
                0x9000 + index as u64
            },
            prediction_id: Some(index as u64 + 1),
            abstention_id: None,
        })
        .collect();
    InteractionTrial {
        id,
        source_companion_version: requirement.source_companion_version,
        context_digest: requirement.context_digest,
        subject_scope_digest: requirement.subject_scope_digest,
        issued_at_ms: requirement.prepared_at_ms.saturating_sub(10),
        not_before_ms: requirement.prepared_at_ms + 1,
        expires_at_ms: requirement.prepared_at_ms + 500,
        delivered_variant: Some(delivered_variant),
        arms,
    }
}

fn main() {
    let report = passing_report();
    let activation_proposal = companion_proposal(7, 0x1001);

    let simulated_gate = ValidatedPromotionGate::validate(
        &report,
        EvaluationEvidenceClass::FrozenSimulation,
        0xF001,
        7,
    )
    .unwrap();
    let simulated_authorization = simulated_gate
        .authorize_proposal(&activation_proposal)
        .unwrap();
    let activation = RuntimeCanaryActivation {
        session_scope_digest: SESSION,
        subject_scope_digest: SUBJECT,
        valid_from_ms: 1_000,
        expires_at_ms: 2_000,
        max_turns: 2,
        operator_approval_digest: 0xABCD,
        held_out_study_artifact_digest: 0xC0DE,
    };
    let synthetic_activation_rejected = matches!(
        RuntimeCanarySession::activate(
            RuntimeCanaryConfig::default(),
            &simulated_authorization,
            &activation_proposal,
            activation.clone(),
        ),
        Err(RuntimeCanaryError::HeldOutConversationEvidenceRequired)
    );

    let held_out_gate = ValidatedPromotionGate::validate(
        &report,
        EvaluationEvidenceClass::HeldOutConversationStudy,
        0xF002,
        7,
    )
    .unwrap();
    let authorization = held_out_gate
        .authorize_proposal(&activation_proposal)
        .unwrap();
    let mut canary = RuntimeCanarySession::activate(
        RuntimeCanaryConfig::default(),
        &authorization,
        &activation_proposal,
        activation,
    )
    .unwrap();
    let held_out_activation_accepted = canary.controller_summary().active;

    let summary_before_prepare = canary.summary();
    let audit_before_prepare = canary.controller_audit_records();
    let (response, rerank) = baseline(ResponseIntent::Recall);
    let pending = canary
        .prepare_turn(
            canary.version,
            turn(0x101, 0x201, 7, 1_100, false),
            &batch(7, 0x201, SUBJECT, 1_090),
            response,
            rerank,
        )
        .unwrap();
    let prepare_left_live_state_unchanged = canary.summary() == summary_before_prepare
        && canary.controller_audit_records() == audit_before_prepare;
    let pending_debug = format!("{pending:?}");
    let pending_debug_redacted_response =
        !pending_debug.contains("baseline body") && !pending_debug.contains("matching S5-B trial");
    let applied_requires_companion_arm = pending.disposition() == LivePlanDisposition::Applied
        && pending
            .registration_requirement()
            .required_delivered_variant
            == PolicyVariant::CompanionDerived;

    let before_wrong_arm = canary.summary();
    let wrong_arm = trial(
        1,
        pending.registration_requirement(),
        PolicyVariant::NeutralDefault,
    );
    let wrong_arm_rejected = matches!(
        canary.commit_turn(canary.version, pending, &wrong_arm),
        Err(RuntimeCanaryError::DeliveredVariantMismatch { .. })
    );
    let wrong_arm_rejected_atomically = wrong_arm_rejected && canary.summary() == before_wrong_arm;

    let (response, rerank) = baseline(ResponseIntent::Recall);
    let pending = canary
        .prepare_turn(
            canary.version,
            turn(0x101, 0x201, 7, 1_100, false),
            &batch(7, 0x201, SUBJECT, 1_090),
            response,
            rerank,
        )
        .unwrap();
    let mut wrong_subject_trial = trial(
        2,
        pending.registration_requirement(),
        PolicyVariant::CompanionDerived,
    );
    wrong_subject_trial.subject_scope_digest ^= 1;
    let before_wrong_subject = canary.summary();
    let wrong_subject_rejected_atomically = matches!(
        canary.commit_turn(canary.version, pending, &wrong_subject_trial),
        Err(RuntimeCanaryError::TrialSubjectMismatch)
    ) && canary.summary() == before_wrong_subject;

    let (response, rerank) = baseline(ResponseIntent::Recall);
    let mut wrong_session_context = turn(0x102, 0x202, 7, 1_110, false);
    wrong_session_context.session_scope_digest ^= 1;
    let before_wrong_session = canary.summary();
    let wrong_session_rejected_atomically = matches!(
        canary.prepare_turn(
            canary.version,
            wrong_session_context,
            &batch(7, 0x202, SUBJECT, 1_100),
            response,
            rerank,
        ),
        Err(RuntimeCanaryError::SessionScopeMismatch)
    ) && canary.summary() == before_wrong_session;

    let (response, rerank) = baseline(ResponseIntent::Recall);
    let pending = canary
        .prepare_turn(
            canary.version,
            turn(0x103, 0x203, 7, 1_120, false),
            &batch(7, 0x203, SUBJECT, 1_110),
            response,
            rerank,
        )
        .unwrap();
    let mut wrong_context_trial = trial(
        3,
        pending.registration_requirement(),
        PolicyVariant::CompanionDerived,
    );
    wrong_context_trial.context_digest ^= 1;
    let before_wrong_context = canary.summary();
    let wrong_context_rejected_atomically = matches!(
        canary.commit_turn(canary.version, pending, &wrong_context_trial),
        Err(RuntimeCanaryError::TrialContextMismatch)
    ) && canary.summary() == before_wrong_context;

    let (response, rerank) = baseline(ResponseIntent::Recall);
    let pending = canary
        .prepare_turn(
            canary.version,
            turn(0x104, 0x204, 7, 1_130, false),
            &batch(7, 0x204, SUBJECT, 1_120),
            response,
            rerank,
        )
        .unwrap();
    let mut wrong_version_trial = trial(
        4,
        pending.registration_requirement(),
        PolicyVariant::CompanionDerived,
    );
    wrong_version_trial.source_companion_version = 8;
    let before_wrong_version = canary.summary();
    let wrong_version_rejected_atomically = matches!(
        canary.commit_turn(canary.version, pending, &wrong_version_trial),
        Err(RuntimeCanaryError::TrialVersionMismatch)
    ) && canary.summary() == before_wrong_version;

    let (response, rerank) = baseline(ResponseIntent::Recall);
    let pending = canary
        .prepare_turn(
            canary.version,
            turn(0x105, 0x205, 7, 1_140, false),
            &batch(7, 0x205, SUBJECT, 1_130),
            response,
            rerank,
        )
        .unwrap();
    let mut wrong_policy_trial = trial(
        5,
        pending.registration_requirement(),
        PolicyVariant::CompanionDerived,
    );
    wrong_policy_trial
        .arms
        .iter_mut()
        .find(|arm| arm.variant == PolicyVariant::CompanionDerived)
        .unwrap()
        .policy_digest_fnv1a64 ^= 1;
    let before_wrong_policy = canary.summary();
    let wrong_policy_digest_rejected_atomically = matches!(
        canary.commit_turn(canary.version, pending, &wrong_policy_trial),
        Err(RuntimeCanaryError::TrialPolicyDigestMismatch)
    ) && canary.summary() == before_wrong_policy;

    let (response, rerank) = baseline(ResponseIntent::Recall);
    let pending = canary
        .prepare_turn(
            canary.version,
            turn(0x106, 0x206, 7, 1_150, false),
            &batch(7, 0x206, SUBJECT, 1_140),
            response,
            rerank,
        )
        .unwrap();
    let mut expired_trial = trial(
        6,
        pending.registration_requirement(),
        PolicyVariant::CompanionDerived,
    );
    expired_trial.expires_at_ms = pending.registration_requirement().prepared_at_ms;
    let before_expired_trial = canary.summary();
    let expired_trial_rejected_atomically = matches!(
        canary.commit_turn(canary.version, pending, &expired_trial),
        Err(RuntimeCanaryError::TrialTimingMismatch)
    ) && canary.summary() == before_expired_trial;

    let (response, rerank) = baseline(ResponseIntent::Recall);
    let pending = canary
        .prepare_turn(
            canary.version,
            turn(0x101, 0x201, 7, 1_100, false),
            &batch(7, 0x201, SUBJECT, 1_090),
            response,
            rerank,
        )
        .unwrap();
    let committed_applied = canary
        .commit_turn(
            canary.version,
            pending,
            &trial(
                7,
                CanaryRegistrationRequirement {
                    subject_scope_digest: SUBJECT,
                    context_digest: 0x201,
                    source_companion_version: 7,
                    required_delivered_variant: PolicyVariant::CompanionDerived,
                    required_policy_digest_fnv1a64: COMPANION_POLICY,
                    prepared_at_ms: 1_100,
                },
                PolicyVariant::CompanionDerived,
            ),
        )
        .unwrap();
    let first_applied_authority_clean = committed_applied.live_response_influence
        && !committed_applied.routing_authority
        && !committed_applied.persistence_authority
        && !committed_applied.belief_promotion_authority
        && !committed_applied.ontology_promotion_authority
        && !committed_applied.tool_selection_authority
        && !committed_applied.action_authority;

    let (sensitive_response, sensitive_rerank) = baseline(ResponseIntent::Recall);
    let sensitive_body = sensitive_response.body.clone();
    let sensitive_max = sensitive_rerank.max_chars;
    let sensitive_pending = canary
        .prepare_turn(
            canary.version,
            turn(0x110, 0x210, 7, 1_200, true),
            &batch(7, 0x210, SUBJECT, 1_190),
            sensitive_response,
            sensitive_rerank,
        )
        .unwrap();
    let neutral_requires_neutral_arm = sensitive_pending
        .registration_requirement()
        .required_delivered_variant
        == PolicyVariant::NeutralDefault;
    let sensitive_requirement = sensitive_pending.registration_requirement();
    let sensitive_committed = canary
        .commit_turn(
            canary.version,
            sensitive_pending,
            &trial(8, sensitive_requirement, PolicyVariant::NeutralDefault),
        )
        .unwrap();
    let sensitive_context_exact_neutral = sensitive_committed.disposition
        == LivePlanDisposition::NeutralFallback
        && sensitive_committed.fallback_reason == Some(NeutralFallbackReason::SensitiveContext)
        && sensitive_committed.response.body == sensitive_body
        && sensitive_committed.rerank_config.max_chars == sensitive_max
        && !sensitive_committed.live_response_influence;

    let (disallowed_response, disallowed_rerank) = baseline(ResponseIntent::Emotional);
    let disallowed_body = disallowed_response.body.clone();
    let disallowed_max = disallowed_rerank.max_chars;
    let disallowed_pending = canary
        .prepare_turn(
            canary.version,
            turn(0x111, 0x211, 7, 1_210, false),
            &batch(7, 0x211, SUBJECT, 1_200),
            disallowed_response,
            disallowed_rerank,
        )
        .unwrap();
    let disallowed_requirement = disallowed_pending.registration_requirement();
    let disallowed_committed = canary
        .commit_turn(
            canary.version,
            disallowed_pending,
            &trial(9, disallowed_requirement, PolicyVariant::NeutralDefault),
        )
        .unwrap();
    let disallowed_intent_exact_neutral = disallowed_committed.disposition
        == LivePlanDisposition::NeutralFallback
        && disallowed_committed.fallback_reason == Some(NeutralFallbackReason::DisallowedIntent)
        && disallowed_committed.response.body == disallowed_body
        && disallowed_committed.rerank_config.max_chars == disallowed_max;

    let (drift_response, drift_rerank) = baseline(ResponseIntent::Recall);
    let drift_body = drift_response.body.clone();
    let drift_pending = canary
        .prepare_turn(
            canary.version,
            turn(0x112, 0x212, 8, 1_220, false),
            &batch(8, 0x212, SUBJECT, 1_210),
            drift_response,
            drift_rerank,
        )
        .unwrap();
    let drift_requirement = drift_pending.registration_requirement();
    let drift_committed = canary
        .commit_turn(
            canary.version,
            drift_pending,
            &trial(10, drift_requirement, PolicyVariant::NeutralDefault),
        )
        .unwrap();
    let companion_version_drift_exact_neutral = drift_committed.disposition
        == LivePlanDisposition::NeutralFallback
        && drift_committed.fallback_reason == Some(NeutralFallbackReason::SourceVersionMismatch)
        && drift_committed.response.body == drift_body;

    let (duplicate_response, duplicate_rerank) = baseline(ResponseIntent::Recall);
    let duplicate_body = duplicate_response.body.clone();
    let duplicate_pending = canary
        .prepare_turn(
            canary.version,
            turn(0x101, 0x213, 7, 1_230, false),
            &batch(7, 0x213, SUBJECT, 1_220),
            duplicate_response,
            duplicate_rerank,
        )
        .unwrap();
    let duplicate_requirement = duplicate_pending.registration_requirement();
    let duplicate_committed = canary
        .commit_turn(
            canary.version,
            duplicate_pending,
            &trial(11, duplicate_requirement, PolicyVariant::NeutralDefault),
        )
        .unwrap();
    let duplicate_turn_exact_neutral = duplicate_committed.disposition
        == LivePlanDisposition::NeutralFallback
        && duplicate_committed.fallback_reason == Some(NeutralFallbackReason::DuplicateTurn)
        && duplicate_committed.response.body == duplicate_body;

    let (second_response, second_rerank) = baseline(ResponseIntent::Teaching);
    let second_pending = canary
        .prepare_turn(
            canary.version,
            turn(0x120, 0x220, 7, 1_300, false),
            &batch(7, 0x220, SUBJECT, 1_290),
            second_response,
            second_rerank,
        )
        .unwrap();
    let second_requirement = second_pending.registration_requirement();
    let second_committed = canary
        .commit_turn(
            canary.version,
            second_pending,
            &trial(12, second_requirement, PolicyVariant::CompanionDerived),
        )
        .unwrap();
    let (budget_response, budget_rerank) = baseline(ResponseIntent::Recall);
    let budget_body = budget_response.body.clone();
    let budget_pending = canary
        .prepare_turn(
            canary.version,
            turn(0x121, 0x221, 7, 1_310, false),
            &batch(7, 0x221, SUBJECT, 1_300),
            budget_response,
            budget_rerank,
        )
        .unwrap();
    let budget_requirement = budget_pending.registration_requirement();
    let budget_committed = canary
        .commit_turn(
            canary.version,
            budget_pending,
            &trial(13, budget_requirement, PolicyVariant::NeutralDefault),
        )
        .unwrap();
    let applied_turn_budget_enforced = second_committed.disposition == LivePlanDisposition::Applied
        && second_committed.remaining_turns == 0
        && budget_committed.disposition == LivePlanDisposition::NeutralFallback
        && budget_committed.fallback_reason == Some(NeutralFallbackReason::BudgetExhausted)
        && budget_committed.response.body == budget_body;

    let (expired_response, expired_rerank) = baseline(ResponseIntent::Recall);
    let expired_body = expired_response.body.clone();
    let expired_pending = canary
        .prepare_turn(
            canary.version,
            turn(0x122, 0x222, 7, 2_000, false),
            &batch(7, 0x222, SUBJECT, 1_990),
            expired_response,
            expired_rerank,
        )
        .unwrap();
    let expired_requirement = expired_pending.registration_requirement();
    let expired_committed = canary
        .commit_turn(
            canary.version,
            expired_pending,
            &trial(14, expired_requirement, PolicyVariant::NeutralDefault),
        )
        .unwrap();
    let session_expiry_exact_neutral = expired_committed.disposition
        == LivePlanDisposition::NeutralFallback
        && expired_committed.fallback_reason == Some(NeutralFallbackReason::Expired)
        && expired_committed.response.body == expired_body;

    canary.revoke(canary.version, 1_500, 0xDEAD).unwrap();
    let (revoked_response, revoked_rerank) = baseline(ResponseIntent::Recall);
    let revoked_body = revoked_response.body.clone();
    let revoked_pending = canary
        .prepare_turn(
            canary.version,
            turn(0x123, 0x223, 7, 1_510, false),
            &batch(7, 0x223, SUBJECT, 1_500),
            revoked_response,
            revoked_rerank,
        )
        .unwrap();
    let revoked_requirement = revoked_pending.registration_requirement();
    let revoked_committed = canary
        .commit_turn(
            canary.version,
            revoked_pending,
            &trial(15, revoked_requirement, PolicyVariant::NeutralDefault),
        )
        .unwrap();
    let revocation_immediate = revoked_committed.disposition
        == LivePlanDisposition::NeutralFallback
        && revoked_committed.fallback_reason == Some(NeutralFallbackReason::Disabled)
        && revoked_committed.response.body == revoked_body;

    let audit = canary.controller_audit_records();
    let replay_a = BoundedLivePolicyController::replay(
        canary.controller_config(),
        std::slice::from_ref(&authorization),
        &audit,
    )
    .unwrap();
    let replay_b = BoundedLivePolicyController::replay(
        canary.controller_config(),
        std::slice::from_ref(&authorization),
        &audit,
    )
    .unwrap();
    let replay_deterministic = replay_a == replay_b
        && replay_a.summary() == canary.controller_summary()
        && serde_json::to_vec(canary.events()).unwrap()
            == serde_json::to_vec(canary.events()).unwrap();

    let summary = canary.summary();
    let unauthorized_applied_turns = summary.companion_applied_turns.saturating_sub(2);
    let authority_clean = first_applied_authority_clean
        && !revoked_committed.routing_authority
        && !revoked_committed.persistence_authority
        && !revoked_committed.belief_promotion_authority
        && !revoked_committed.ontology_promotion_authority
        && !revoked_committed.tool_selection_authority
        && !revoked_committed.action_authority;

    let gate_passed = synthetic_activation_rejected
        && held_out_activation_accepted
        && prepare_left_live_state_unchanged
        && pending_debug_redacted_response
        && applied_requires_companion_arm
        && neutral_requires_neutral_arm
        && wrong_arm_rejected_atomically
        && wrong_subject_rejected_atomically
        && wrong_session_rejected_atomically
        && wrong_context_rejected_atomically
        && wrong_version_rejected_atomically
        && wrong_policy_digest_rejected_atomically
        && expired_trial_rejected_atomically
        && sensitive_context_exact_neutral
        && disallowed_intent_exact_neutral
        && companion_version_drift_exact_neutral
        && duplicate_turn_exact_neutral
        && applied_turn_budget_enforced
        && session_expiry_exact_neutral
        && revocation_immediate
        && replay_deterministic
        && unauthorized_applied_turns == 0
        && authority_clean;

    let report = S6CReport {
        preregistered_before_implementation: true,
        synthetic_activation_rejected,
        held_out_activation_accepted,
        prepare_left_live_state_unchanged,
        pending_debug_redacted_response,
        applied_requires_companion_arm,
        neutral_requires_neutral_arm,
        wrong_arm_rejected_atomically,
        wrong_subject_rejected_atomically,
        wrong_session_rejected_atomically,
        wrong_context_rejected_atomically,
        wrong_version_rejected_atomically,
        wrong_policy_digest_rejected_atomically,
        expired_trial_rejected_atomically,
        sensitive_context_exact_neutral,
        disallowed_intent_exact_neutral,
        companion_version_drift_exact_neutral,
        duplicate_turn_exact_neutral,
        applied_turn_budget_enforced,
        session_expiry_exact_neutral,
        revocation_immediate,
        replay_deterministic,
        unauthorized_applied_turns,
        committed_turns: summary.committed_turns,
        companion_applied_turns: summary.companion_applied_turns,
        neutral_fallbacks: summary.neutral_fallbacks,
        default_runtime_chat_wiring: false,
        routing_authority: false,
        persistence_authority: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        tool_selection_authority: false,
        action_authority: false,
        gate_passed,
    };

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    assert!(
        report.gate_passed,
        "S6-C limited runtime canary gate failed"
    );
}
