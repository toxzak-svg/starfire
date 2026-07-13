use serde::Serialize;
use star::companion_interaction_policy::{
    DetailLevel, PolicyContext, PolicyVariant, VocabularyLevel,
};
use star::companion_live_policy_canary::{
    verify_audit_chain, CanaryFallbackReason, CanaryMode, CanaryRollbackReason,
    LivePolicyCanary, LivePolicyCanaryConfig, PromotionAuthorization, PromotionEvidenceClass,
};
use star::companion_policy_evaluation::{
    CandidateControlComparison, CandidatePairwiseMetrics, ComparisonGates, EvaluationSplit,
    PolicyEvaluationReport, PolicyEvaluationVerdict,
};
use star::companion_state::{
    ClaimInput, ClaimSource, CompanionState, Retention, Sensitivity,
};

#[derive(Debug, Serialize)]
struct S6AProbeReport {
    terminal_classification: &'static str,
    synthetic_authorization_refused: bool,
    real_canary_influence_applied: bool,
    effective_policy_expected: bool,
    stale_authorization_fallback: bool,
    rollout_fallback: bool,
    compute_fallback: bool,
    contradiction_fallback: bool,
    rollback_fallback: bool,
    failed_evaluation_latched_rollback: bool,
    audit_chain_valid: bool,
    source_state_unchanged: bool,
    neutral_fallback_is_exact: bool,
    gate_passed: bool,
    runtime_chat_wiring: bool,
    generated_text_mutation: bool,
    routing_authority: bool,
    belief_promotion_authority: bool,
    ontology_promotion_authority: bool,
    persistence_authority: bool,
    action_authority: bool,
    audit_events: usize,
}

fn claim(key: &str, value: &str, at: u64) -> ClaimInput {
    ClaimInput {
        key: key.to_owned(),
        value: value.to_owned(),
        source: ClaimSource::UserStatement,
        confidence_bps: 9_000,
        sensitivity: Sensitivity::Personal,
        retention: Retention::Durable,
        observed_at_ms: at,
    }
}

fn context(subject_scope_digest: u64, context_digest: u64) -> PolicyContext {
    PolicyContext {
        context_digest,
        subject_scope_digest,
        domain: Some("rust".to_owned()),
        technical_context: true,
        asks_for_explanation: true,
        emotional_signal: false,
        issued_at_ms: 10_000,
        not_before_ms: 10_100,
        expires_at_ms: 20_000,
    }
}

fn gates() -> ComparisonGates {
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
        for control in [
            PolicyVariant::NeutralDefault,
            PolicyVariant::RecencyOnly,
            PolicyVariant::MajorityPrior,
            PolicyVariant::ContextOnly,
            PolicyVariant::ScrambledScope,
        ] {
            holdout_comparisons.push(CandidateControlComparison {
                split,
                control,
                candidate_resolved: 2,
                control_resolved: 2,
                candidate_direct_outcomes: 1,
                control_direct_outcomes: 1,
                pairwise: CandidatePairwiseMetrics {
                    candidate_wins: 1,
                    control_wins: 0,
                    ties: 0,
                    total: 1,
                    candidate_win_margin_bps: Some(10_000),
                },
                brier_improvement_ppm: Some(10_000),
                calibration_regression_bps: Some(0),
                correction_regression_bps: Some(0),
                clarification_regression_bps: Some(0),
                completion_regression_bps: Some(0),
                abandonment_regression_bps: Some(0),
                abstention_regression_bps: Some(0),
                compute_overhead_bps: Some(0),
                gates: gates(),
            });
        }
    }
    PolicyEvaluationReport {
        splits: Vec::new(),
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

fn main() {
    let mut state = CompanionState::new();
    let first = state
        .record_claim(0, claim("preference.detail.general", "yes", 100))
        .unwrap();
    state
        .record_claim(
            first.version,
            claim("knowledge.strong_domain.rust", "rust", 200),
        )
        .unwrap();
    let source_before = state.clone();

    let config = LivePolicyCanaryConfig {
        mode: CanaryMode::LiveCanary,
        rollout_modulus: 2,
        rollout_remainder: 0,
        min_confidence_bps: 7_000,
        include_session_claims: true,
        max_source_claims: 4,
        max_planning_compute_micros: 500,
    };
    let mut canary = LivePolicyCanary::new(config).unwrap();
    let report = passing_report();
    let synthetic = PromotionAuthorization::from_report(
        &report,
        PromotionEvidenceClass::SyntheticConformance,
        format!("sha256:{}", "a".repeat(64)),
        state.version,
    )
    .unwrap();
    canary.install_authorization(synthetic).unwrap();
    let synthetic_decision = canary.decide(&state, context(2, 0x6001), 100).unwrap();
    let synthetic_authorization_refused = !synthetic_decision.live_influence_applied
        && synthetic_decision.fallback_reason == Some(CanaryFallbackReason::SyntheticEvidenceOnly)
        && synthetic_decision.selected_variant == PolicyVariant::NeutralDefault;

    let real = PromotionAuthorization::from_report(
        &report,
        PromotionEvidenceClass::RealHeldOut,
        format!("sha256:{}", "b".repeat(64)),
        state.version,
    )
    .unwrap();
    canary.install_authorization(real).unwrap();
    let live_decision = canary.decide(&state, context(4, 0x6002), 100).unwrap();
    let real_canary_influence_applied = live_decision.live_influence_applied
        && live_decision.fallback_reason.is_none()
        && live_decision.selected_variant == PolicyVariant::CompanionDerived;
    let effective_policy_expected = live_decision.effective_policy.detail == DetailLevel::Detailed
        && live_decision.effective_policy.vocabulary == VocabularyLevel::Technical;

    let mut changed_state = state.clone();
    changed_state
        .record_claim(
            changed_state.version,
            claim("preference.questions.general", "yes", 250),
        )
        .unwrap();
    let stale_version_decision = canary
        .decide(&changed_state, context(12, 0x6007), 100)
        .unwrap();
    let stale_authorization_fallback = stale_version_decision.fallback_reason
        == Some(CanaryFallbackReason::SourceVersionMismatch)
        && stale_version_decision.selected_variant == PolicyVariant::NeutralDefault;

    let rollout_decision = canary.decide(&state, context(5, 0x6003), 100).unwrap();
    let rollout_fallback = rollout_decision.fallback_reason
        == Some(CanaryFallbackReason::RolloutExcluded)
        && rollout_decision.selected_variant == PolicyVariant::NeutralDefault;

    let compute_decision = canary.decide(&state, context(6, 0x6004), 501).unwrap();
    let compute_fallback = compute_decision.fallback_reason
        == Some(CanaryFallbackReason::ComputeBudgetExceeded)
        && compute_decision.selected_variant == PolicyVariant::NeutralDefault;

    let mut conflict_state = state.clone();
    conflict_state
        .record_claim(
            conflict_state.version,
            claim("preference.brevity.general", "yes", 300),
        )
        .unwrap();
    let conflict_authorization = PromotionAuthorization::from_report(
        &report,
        PromotionEvidenceClass::RealHeldOut,
        format!("sha256:{}", "d".repeat(64)),
        conflict_state.version,
    )
    .unwrap();
    canary.install_authorization(conflict_authorization).unwrap();
    let conflict_decision = canary
        .decide(&conflict_state, context(8, 0x6005), 100)
        .unwrap();
    let contradiction_fallback = conflict_decision.fallback_reason
        == Some(CanaryFallbackReason::CandidateAbstained)
        && conflict_decision.selected_variant == PolicyVariant::NeutralDefault;

    canary
        .latch_rollback(CanaryRollbackReason::OperatorRequested)
        .unwrap();
    let rollback_decision = canary.decide(&state, context(10, 0x6006), 100).unwrap();
    let rollback_fallback = rollback_decision.fallback_reason
        == Some(CanaryFallbackReason::RollbackLatched)
        && rollback_decision.selected_variant == PolicyVariant::NeutralDefault;
    let generation = canary.rollback_generation();
    canary.clear_rollback(generation, 0xfeed_beef).unwrap();

    let mut failed_report = report.clone();
    failed_report.verdict = PolicyEvaluationVerdict::Fail;
    failed_report.promotion_eligible = false;
    canary
        .apply_evaluation_update(
            &failed_report,
            PromotionEvidenceClass::RealHeldOut,
            format!("sha256:{}", "c".repeat(64)),
            state.version,
        )
        .unwrap();
    let failed_evaluation_latched_rollback = canary.authorization().is_none()
        && canary.rollback_reason() == Some(CanaryRollbackReason::EvaluationFailed);

    let audit_chain_valid = verify_audit_chain(canary.audit());
    let source_state_unchanged = state == source_before;
    let neutral_fallback_is_exact = synthetic_decision.effective_policy
        == rollout_decision.effective_policy
        && rollout_decision.effective_policy == compute_decision.effective_policy
        && compute_decision.effective_policy == conflict_decision.effective_policy
        && conflict_decision.effective_policy == rollback_decision.effective_policy;

    let gate_passed = synthetic_authorization_refused
        && real_canary_influence_applied
        && effective_policy_expected
        && stale_authorization_fallback
        && rollout_fallback
        && compute_fallback
        && contradiction_fallback
        && rollback_fallback
        && failed_evaluation_latched_rollback
        && audit_chain_valid
        && source_state_unchanged
        && neutral_fallback_is_exact;

    let probe = S6AProbeReport {
        terminal_classification: "EXPERIMENT_READY",
        synthetic_authorization_refused,
        real_canary_influence_applied,
        effective_policy_expected,
        stale_authorization_fallback,
        rollout_fallback,
        compute_fallback,
        contradiction_fallback,
        rollback_fallback,
        failed_evaluation_latched_rollback,
        audit_chain_valid,
        source_state_unchanged,
        neutral_fallback_is_exact,
        gate_passed,
        runtime_chat_wiring: false,
        generated_text_mutation: false,
        routing_authority: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        persistence_authority: false,
        action_authority: false,
        audit_events: canary.audit().len(),
    };

    println!("{}", serde_json::to_string_pretty(&probe).unwrap());
    assert!(gate_passed, "S6-A reversible live-policy canary gate failed");
}
