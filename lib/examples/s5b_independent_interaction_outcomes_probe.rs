use serde::Serialize;
use star::companion_interaction_outcomes::{
    InteractionOutcomeError, InteractionOutcomeLedger, ObservedOutcomeEvidence, ObservedSignal,
    PairedEvaluationEvidence, PairwisePreference,
};
use star::companion_interaction_policy::{
    PolicyContext, PolicyVariant, ShadowPolicyPlanner,
};
use star::companion_prediction_ledger::{
    PredictionLedger, PredictionStatus, WitnessSource,
};
use star::companion_state::{
    ClaimInput, ClaimSource, CompanionState, Retention, Sensitivity,
};

#[derive(Debug, Serialize)]
struct S5BReport {
    trials: u64,
    observed_signals: u64,
    paired_evaluations: u64,
    resolved_predictions: u64,
    pending_predictions: u64,
    abstained_arms: u64,
    neutral_signal_left_s4_unchanged: bool,
    direct_evidence_resolved_only_delivered_arm: bool,
    pure_shadow_direct_evidence_rejected: bool,
    response_generator_rejected: bool,
    external_evaluator_direct_channel_rejected: bool,
    pair_resolved_exactly_two_arms: bool,
    replay_equal: bool,
    s4_replay_equal: bool,
    source_state_unchanged: bool,
    gate_passed: bool,
    live_response_influence: bool,
    routing_authority: bool,
    belief_promotion_authority: bool,
    action_authority: bool,
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

fn context(seed: u64, issued_at_ms: u64) -> PolicyContext {
    PolicyContext {
        context_digest: 0x1000 + seed,
        subject_scope_digest: 0x2000 + seed,
        domain: Some("rust".to_owned()),
        technical_context: true,
        asks_for_explanation: true,
        emotional_signal: false,
        issued_at_ms,
        not_before_ms: issued_at_ms + 100,
        expires_at_ms: issued_at_ms + 1_000,
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
    let base_predictions = predictions.clone();
    let mut outcomes = InteractionOutcomeLedger::new(&predictions);

    let delivered_enrollment = planner
        .enroll(&state, &mut predictions, 0, context(1, 1_000))
        .unwrap();
    let delivered_trial = outcomes
        .register_enrollment(
            0,
            &predictions,
            &delivered_enrollment,
            Some(PolicyVariant::NeutralDefault),
        )
        .unwrap()
        .trial_id;

    let version_before_neutral = predictions.version;
    outcomes
        .record_observed_signal(
            outcomes.version,
            &mut predictions,
            delivered_trial,
            ObservedOutcomeEvidence {
                signal: ObservedSignal::NeutralFollowUp,
                source: WitnessSource::UserObservation,
                observed_at_ms: 1_150,
                evidence_digest: 0x301,
            },
        )
        .unwrap();
    let neutral_signal_left_s4_unchanged = predictions.version == version_before_neutral;

    let response_generator_rejected = matches!(
        outcomes.record_observed_signal(
            outcomes.version,
            &mut predictions,
            delivered_trial,
            ObservedOutcomeEvidence {
                signal: ObservedSignal::TaskCompleted,
                source: WitnessSource::ResponseGenerator,
                observed_at_ms: 1_200,
                evidence_digest: 0x302,
            },
        ),
        Err(InteractionOutcomeError::SelfGradingWitness)
    );
    let external_evaluator_direct_channel_rejected = matches!(
        outcomes.record_observed_signal(
            outcomes.version,
            &mut predictions,
            delivered_trial,
            ObservedOutcomeEvidence {
                signal: ObservedSignal::ExplicitPositiveRating,
                source: WitnessSource::ExternalEvaluator,
                observed_at_ms: 1_200,
                evidence_digest: 0x303,
            },
        ),
        Err(InteractionOutcomeError::WrongEvidenceChannel)
    );

    outcomes
        .record_observed_signal(
            outcomes.version,
            &mut predictions,
            delivered_trial,
            ObservedOutcomeEvidence {
                signal: ObservedSignal::TaskCompleted,
                source: WitnessSource::Environment,
                observed_at_ms: 1_250,
                evidence_digest: 0x304,
            },
        )
        .unwrap();

    let delivered = outcomes.trials().get(&delivered_trial).unwrap();
    let direct_evidence_resolved_only_delivered_arm = delivered.arms.iter().all(|arm| {
        let prediction = arm
            .prediction_id
            .and_then(|prediction_id| predictions.prediction(prediction_id))
            .unwrap();
        if arm.variant == PolicyVariant::NeutralDefault {
            matches!(&prediction.status, PredictionStatus::Resolved { .. })
        } else {
            matches!(&prediction.status, PredictionStatus::Pending)
        }
    });

    let shadow_enrollment = planner
        .enroll(
            &state,
            &mut predictions,
            predictions.version,
            context(2, 3_000),
        )
        .unwrap();
    let shadow_trial = outcomes
        .register_enrollment(
            outcomes.version,
            &predictions,
            &shadow_enrollment,
            None,
        )
        .unwrap()
        .trial_id;

    let predictions_before_rejected = predictions.clone();
    let outcomes_before_rejected = outcomes.clone();
    let pure_shadow_direct_evidence_rejected = matches!(
        outcomes.record_observed_signal(
            outcomes.version,
            &mut predictions,
            shadow_trial,
            ObservedOutcomeEvidence {
                signal: ObservedSignal::ExplicitPositiveRating,
                source: WitnessSource::UserObservation,
                observed_at_ms: 3_200,
                evidence_digest: 0x401,
            },
        ),
        Err(InteractionOutcomeError::NoDeliveredArm(id)) if id == shadow_trial
    ) && predictions == predictions_before_rejected
        && outcomes == outcomes_before_rejected;

    let resolved_before_pair = outcomes.summary().resolved_trial_predictions;
    outcomes
        .record_paired_evaluation(
            outcomes.version,
            &mut predictions,
            shadow_trial,
            PairedEvaluationEvidence {
                left_variant: PolicyVariant::CompanionDerived,
                right_variant: PolicyVariant::ContextOnly,
                preference: PairwisePreference::Left,
                left_render_digest: 0x501,
                right_render_digest: 0x502,
                evaluator_digest: 0x503,
                observed_at_ms: 3_250,
                evidence_digest: 0x504,
            },
        )
        .unwrap();
    let pair_resolved_exactly_two_arms =
        outcomes.summary().resolved_trial_predictions == resolved_before_pair + 2;

    let replayed = InteractionOutcomeLedger::replay(&base_predictions, outcomes.events()).unwrap();
    let replay_equal = replayed == outcomes;
    let s4_replay_equal = replayed.mirrored_prediction_ledger() == &predictions;
    let summary = outcomes.summary();
    let source_state_unchanged = state == source_before;
    let gate_passed = summary.trials == 2
        && summary.observed_signals == 2
        && summary.paired_evaluations == 1
        && summary.resolved_trial_predictions == 3
        && summary.pending_trial_predictions == 9
        && summary.abstained_trial_arms == 0
        && neutral_signal_left_s4_unchanged
        && direct_evidence_resolved_only_delivered_arm
        && pure_shadow_direct_evidence_rejected
        && response_generator_rejected
        && external_evaluator_direct_channel_rejected
        && pair_resolved_exactly_two_arms
        && replay_equal
        && s4_replay_equal
        && source_state_unchanged;

    let report = S5BReport {
        trials: summary.trials,
        observed_signals: summary.observed_signals,
        paired_evaluations: summary.paired_evaluations,
        resolved_predictions: summary.resolved_trial_predictions,
        pending_predictions: summary.pending_trial_predictions,
        abstained_arms: summary.abstained_trial_arms,
        neutral_signal_left_s4_unchanged,
        direct_evidence_resolved_only_delivered_arm,
        pure_shadow_direct_evidence_rejected,
        response_generator_rejected,
        external_evaluator_direct_channel_rejected,
        pair_resolved_exactly_two_arms,
        replay_equal,
        s4_replay_equal,
        source_state_unchanged,
        gate_passed,
        live_response_influence: false,
        routing_authority: false,
        belief_promotion_authority: false,
        action_authority: false,
    };

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    assert!(gate_passed, "S5-B independent-outcome gate failed");
}
