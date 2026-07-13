use serde::Serialize;
use star::companion_interaction_outcomes::{
    InteractionOutcomeLedger, ObservedOutcomeEvidence, ObservedSignal, PairedEvaluationEvidence,
    PairwisePreference,
};
use star::companion_interaction_policy::{
    PolicyContext, PolicyVariant, ShadowPolicyPlanner,
};
use star::companion_policy_evaluation::{
    evaluate_shadow_policies, ArmComputeObservation, EvaluationSplit, EvaluationSplitPolicy,
    PolicyEvaluationConfig, PolicyEvaluationReport, PolicyEvaluationVerdict,
};
use star::companion_prediction_ledger::{PredictionLedger, WitnessSource};
use star::companion_state::{
    ClaimInput, ClaimSource, CompanionState, Retention, Sensitivity,
};

#[derive(Debug, Serialize)]
struct S5CProbeReport {
    verdict: PolicyEvaluationVerdict,
    promotion_eligible: bool,
    holdout_comparisons: usize,
    all_holdout_evidence_sufficient: bool,
    all_holdout_performance_gates_passed: bool,
    development_excluded_from_verdict: bool,
    opaque_subject_holdout_present: bool,
    temporal_holdout_present: bool,
    deterministic_replay: bool,
    source_state_unchanged: bool,
    gate_passed: bool,
    live_response_influence: bool,
    routing_authority: bool,
    belief_promotion_authority: bool,
    action_authority: bool,
    report: PolicyEvaluationReport,
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

fn context(
    context_digest: u64,
    subject_scope_digest: u64,
    issued_at_ms: u64,
) -> PolicyContext {
    PolicyContext {
        context_digest,
        subject_scope_digest,
        domain: Some("rust".to_owned()),
        technical_context: true,
        asks_for_explanation: true,
        emotional_signal: false,
        issued_at_ms,
        not_before_ms: issued_at_ms + 100,
        expires_at_ms: issued_at_ms + 1_000,
    }
}

fn add_costs(costs: &mut Vec<ArmComputeObservation>, trial_id: u64) {
    for variant in PolicyVariant::all() {
        costs.push(ArmComputeObservation {
            trial_id,
            variant,
            compute_micros: if variant == PolicyVariant::CompanionDerived {
                110
            } else {
                100
            },
        });
    }
}

fn register_trial(
    state: &CompanionState,
    planner: &ShadowPolicyPlanner,
    predictions: &mut PredictionLedger,
    outcomes: &mut InteractionOutcomeLedger,
    costs: &mut Vec<ArmComputeObservation>,
    policy_context: PolicyContext,
    delivered_variant: Option<PolicyVariant>,
) -> u64 {
    let expected_prediction_version = predictions.version;
    let enrollment = planner
        .enroll(
            state,
            predictions,
            expected_prediction_version,
            policy_context,
        )
        .unwrap();
    let trial_id = outcomes
        .register_enrollment(
            outcomes.version,
            predictions,
            &enrollment,
            delivered_variant,
        )
        .unwrap()
        .trial_id;
    add_costs(costs, trial_id);
    trial_id
}

fn split_seed(split: EvaluationSplit, ordinal: u64) -> (u64, u64) {
    match split {
        EvaluationSplit::Development => (10_000 + ordinal * 10, 2_000 + ordinal * 2),
        EvaluationSplit::OpaqueSubjectHoldout => {
            (50_000 + ordinal * 10, 3_001 + ordinal * 2)
        }
        EvaluationSplit::TemporalHoldout => (100_000 + ordinal * 10, 4_000 + ordinal),
    }
}

fn populate_split(
    split: EvaluationSplit,
    state: &CompanionState,
    planner: &ShadowPolicyPlanner,
    predictions: &mut PredictionLedger,
    outcomes: &mut InteractionOutcomeLedger,
    costs: &mut Vec<ArmComputeObservation>,
    digest_seed: &mut u64,
) {
    let mut ordinal = 1_u64;
    for variant in PolicyVariant::all() {
        let (issued_at_ms, subject_scope_digest) = split_seed(split, ordinal);
        *digest_seed += 1;
        let trial_id = register_trial(
            state,
            planner,
            predictions,
            outcomes,
            costs,
            context(*digest_seed, subject_scope_digest, issued_at_ms),
            Some(variant),
        );
        *digest_seed += 1;
        outcomes
            .record_observed_signal(
                outcomes.version,
                predictions,
                trial_id,
                ObservedOutcomeEvidence {
                    signal: ObservedSignal::TaskCompleted,
                    source: WitnessSource::Environment,
                    observed_at_ms: issued_at_ms + 200,
                    evidence_digest: *digest_seed,
                },
            )
            .unwrap();
        ordinal += 1;
    }

    for control in [
        PolicyVariant::NeutralDefault,
        PolicyVariant::RecencyOnly,
        PolicyVariant::MajorityPrior,
        PolicyVariant::ContextOnly,
        PolicyVariant::ScrambledScope,
    ] {
        let (issued_at_ms, subject_scope_digest) = split_seed(split, ordinal);
        *digest_seed += 1;
        let trial_id = register_trial(
            state,
            planner,
            predictions,
            outcomes,
            costs,
            context(*digest_seed, subject_scope_digest, issued_at_ms),
            None,
        );
        *digest_seed += 4;
        outcomes
            .record_paired_evaluation(
                outcomes.version,
                predictions,
                trial_id,
                PairedEvaluationEvidence {
                    left_variant: PolicyVariant::CompanionDerived,
                    right_variant: control,
                    preference: PairwisePreference::Left,
                    left_render_digest: *digest_seed - 3,
                    right_render_digest: *digest_seed - 2,
                    evaluator_digest: *digest_seed - 1,
                    observed_at_ms: issued_at_ms + 200,
                    evidence_digest: *digest_seed,
                },
            )
            .unwrap();
        ordinal += 1;
    }
}

fn config() -> PolicyEvaluationConfig {
    PolicyEvaluationConfig {
        split_policy: EvaluationSplitPolicy {
            temporal_holdout_start_ms: 100_000,
            opaque_subject_modulus: 2,
            opaque_subject_remainder: 1,
        },
        min_resolved_per_arm_per_holdout: 2,
        min_direct_outcomes_per_arm_per_holdout: 1,
        min_pairwise_comparisons_per_control_per_holdout: 1,
        min_brier_improvement_ppm: 10_000,
        min_pairwise_win_margin_bps: 10_000,
        max_calibration_regression_bps: 0,
        max_correction_regression_bps: 0,
        max_clarification_regression_bps: 0,
        max_completion_regression_bps: 0,
        max_abandonment_regression_bps: 0,
        max_abstention_regression_bps: 0,
        max_compute_overhead_bps: 1_500,
    }
}

fn main() {
    let mut state = CompanionState::new();
    let first = state
        .record_claim(0, claim("preference.detail.general", "yes", 10))
        .unwrap();
    state
        .record_claim(
            first.version,
            claim("knowledge.strong_domain.rust", "rust", 20),
        )
        .unwrap();
    let source_before = state.clone();

    let planner = ShadowPolicyPlanner::default();
    let mut predictions = PredictionLedger::new();
    let mut outcomes = InteractionOutcomeLedger::new(&predictions);
    let mut costs = Vec::new();
    let mut digest_seed = 0x5000_u64;

    for split in [
        EvaluationSplit::Development,
        EvaluationSplit::OpaqueSubjectHoldout,
        EvaluationSplit::TemporalHoldout,
    ] {
        populate_split(
            split,
            &state,
            &planner,
            &mut predictions,
            &mut outcomes,
            &mut costs,
            &mut digest_seed,
        );
    }

    let report = evaluate_shadow_policies(&outcomes, &costs, config()).unwrap();
    let replayed = evaluate_shadow_policies(&outcomes, &costs, config()).unwrap();
    let deterministic_replay = report == replayed;
    let all_holdout_evidence_sufficient = report
        .holdout_comparisons
        .iter()
        .all(|comparison| comparison.gates.evidence_sufficient());
    let all_holdout_performance_gates_passed = report
        .holdout_comparisons
        .iter()
        .all(|comparison| comparison.gates.performance_passed());
    let opaque_subject_holdout_present = report.splits.iter().any(|split| {
        split.split == EvaluationSplit::OpaqueSubjectHoldout
            && split.arms.values().all(|metrics| metrics.trials > 0)
    });
    let temporal_holdout_present = report.splits.iter().any(|split| {
        split.split == EvaluationSplit::TemporalHoldout
            && split.arms.values().all(|metrics| metrics.trials > 0)
    });
    let source_state_unchanged = state == source_before;
    let gate_passed = report.verdict == PolicyEvaluationVerdict::Pass
        && report.promotion_eligible
        && report.holdout_comparisons.len() == 10
        && all_holdout_evidence_sufficient
        && all_holdout_performance_gates_passed
        && report.development_excluded_from_verdict
        && opaque_subject_holdout_present
        && temporal_holdout_present
        && deterministic_replay
        && source_state_unchanged
        && !report.live_response_influence
        && !report.routing_authority
        && !report.belief_promotion_authority
        && !report.action_authority;

    let probe_report = S5CProbeReport {
        verdict: report.verdict,
        promotion_eligible: report.promotion_eligible,
        holdout_comparisons: report.holdout_comparisons.len(),
        all_holdout_evidence_sufficient,
        all_holdout_performance_gates_passed,
        development_excluded_from_verdict: report.development_excluded_from_verdict,
        opaque_subject_holdout_present,
        temporal_holdout_present,
        deterministic_replay,
        source_state_unchanged,
        gate_passed,
        live_response_influence: report.live_response_influence,
        routing_authority: report.routing_authority,
        belief_promotion_authority: report.belief_promotion_authority,
        action_authority: report.action_authority,
        report,
    };

    println!("{}", serde_json::to_string_pretty(&probe_report).unwrap());
    assert!(gate_passed, "S5-C comparative-policy gate failed");
}
