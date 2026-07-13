use serde::Serialize;
use star::companion_interaction_policy::{
    PolicyContext, PolicyVariant, ShadowPolicyEnrollment, ShadowPolicyPlanner,
};
use star::companion_policy_outcomes::{
    EvaluationSplit, PolicyOutcomeCollector, PolicyOutcomeCollectorConfig, PolicyOutcomeMetrics,
    PolicyOutcomeObservation, PolicyOutcomeSplitPlan,
};
use star::companion_prediction_ledger::{
    PredictionLedger, PredictionStatus, WitnessSource,
};
use star::companion_state::{
    ClaimInput, ClaimSource, CompanionState, Retention, Sensitivity,
};
use std::collections::BTreeSet;

#[derive(Debug, Serialize)]
struct S5BReport {
    development_arms_resolved: usize,
    subject_holdout_arms_resolved: usize,
    temporal_holdout_arms_resolved: usize,
    split_assignment_deterministic: bool,
    complete_arm_coverage: bool,
    unique_arm_evidence: bool,
    delayed_witnesses_only: bool,
    independent_sources_only: bool,
    self_grading_rejected: bool,
    incomplete_batch_rejected: bool,
    atomic_rejection: bool,
    exact_replay: bool,
    metrics_preserved: bool,
    all_predictions_finalized: bool,
    gate_passed: bool,
    runtime_chat_wiring: bool,
    response_text_influence: bool,
    routing_authority: bool,
    belief_promotion_authority: bool,
    action_authority: bool,
}

fn state() -> CompanionState {
    let mut state = CompanionState::new();
    for (key, value, confidence_bps, at) in [
        ("preference.detail.general", "yes", 9_000, 10),
        ("preference.questions.general", "yes", 8_500, 20),
        (
            "preference.argument_style.general",
            "concrete",
            8_000,
            30,
        ),
        ("knowledge.strong_domain.rust", "rust", 9_500, 40),
    ] {
        state
            .record_claim(
                state.version,
                ClaimInput {
                    key: key.to_owned(),
                    value: value.to_owned(),
                    source: ClaimSource::UserStatement,
                    confidence_bps,
                    sensitivity: Sensitivity::Personal,
                    retention: Retention::Session,
                    observed_at_ms: at,
                },
            )
            .expect("frozen companion claim must be valid");
    }
    state
}

fn context(issued_at_ms: u64, subject_scope_digest: u64) -> PolicyContext {
    PolicyContext {
        context_digest: issued_at_ms ^ subject_scope_digest ^ 0x5b00_0001,
        subject_scope_digest,
        domain: Some("rust".to_owned()),
        technical_context: true,
        asks_for_explanation: true,
        emotional_signal: false,
        issued_at_ms,
        not_before_ms: issued_at_ms + 100,
        expires_at_ms: issued_at_ms + 2_000,
    }
}

fn enroll(
    planner: &ShadowPolicyPlanner,
    state: &CompanionState,
    issued_at_ms: u64,
    subject_scope_digest: u64,
) -> (PredictionLedger, ShadowPolicyEnrollment) {
    let mut ledger = PredictionLedger::new();
    let enrollment = planner
        .enroll(
            state,
            &mut ledger,
            0,
            context(issued_at_ms, subject_scope_digest),
        )
        .expect("frozen S5-A enrollment must succeed");
    (ledger, enrollment)
}

fn observations(
    enrollment: &ShadowPolicyEnrollment,
    source: WitnessSource,
    witness_id: &str,
) -> Vec<PolicyOutcomeObservation> {
    enrollment
        .batch
        .proposals
        .iter()
        .filter(|proposal| !proposal.is_abstention())
        .zip(enrollment.prediction_ids.iter().copied())
        .map(|(proposal, prediction_id)| {
            let satisfactory = matches!(
                proposal.variant,
                PolicyVariant::CompanionDerived
                    | PolicyVariant::ContextOnly
                    | PolicyVariant::RecencyOnly
            );
            PolicyOutcomeObservation {
                prediction_id,
                variant: proposal.variant,
                source,
                witness_id: witness_id.to_owned(),
                label: if satisfactory {
                    "interaction_satisfactory"
                } else {
                    "interaction_unsatisfactory"
                }
                .to_owned(),
                observed_at_ms: enrollment.batch.context.not_before_ms + 10,
                evidence_digest: (enrollment.batch.context.context_digest
                    ^ proposal.policy_digest_fnv1a64
                    ^ prediction_id)
                    | 1,
                metrics: PolicyOutcomeMetrics {
                    correction_count: if satisfactory { 0 } else { 1 },
                    clarification_turns: if satisfactory { 0 } else { 1 },
                    completion_bps: if satisfactory { 10_000 } else { 7_500 },
                    compute_micros: 5_000 + prediction_id,
                },
            }
        })
        .collect()
}

fn main() {
    let planner = ShadowPolicyPlanner::default();
    let state = state();
    let collector = PolicyOutcomeCollector::new(
        PolicyOutcomeSplitPlan::new(50_000, 4, 1).expect("split plan must be valid"),
        PolicyOutcomeCollectorConfig::default(),
    );

    let (mut development_ledger, development_enrollment) =
        enroll(&planner, &state, 10_000, 8);
    let development_issue_events = development_enrollment
        .transitions
        .iter()
        .map(|transition| transition.event.clone())
        .collect::<Vec<_>>();
    let development = collector
        .collect(
            &mut development_ledger,
            development_enrollment.ledger_version_after,
            &development_enrollment,
            observations(
                &development_enrollment,
                WitnessSource::ExternalEvaluator,
                "held-out-evaluator-v1",
            ),
        )
        .expect("development outcomes must collect");

    let (mut subject_ledger, subject_enrollment) = enroll(&planner, &state, 20_000, 9);
    let subject = collector
        .collect(
            &mut subject_ledger,
            subject_enrollment.ledger_version_after,
            &subject_enrollment,
            observations(
                &subject_enrollment,
                WitnessSource::UserObservation,
                "delayed-user-observation",
            ),
        )
        .expect("subject holdout outcomes must collect");

    let (mut temporal_ledger, temporal_enrollment) = enroll(&planner, &state, 50_000, 8);
    let temporal = collector
        .collect(
            &mut temporal_ledger,
            temporal_enrollment.ledger_version_after,
            &temporal_enrollment,
            observations(
                &temporal_enrollment,
                WitnessSource::Environment,
                "task-environment-v1",
            ),
        )
        .expect("temporal holdout outcomes must collect");

    let split_assignment_deterministic = development.split == EvaluationSplit::Development
        && subject.split == EvaluationSplit::SubjectHoldout
        && temporal.split == EvaluationSplit::TemporalHoldout
        && collector.split_plan().classify(&development_enrollment) == development.split
        && collector.split_plan().classify(&subject_enrollment) == subject.split
        && collector.split_plan().classify(&temporal_enrollment) == temporal.split;

    let all_collections = [&development, &subject, &temporal];
    let complete_arm_coverage = all_collections.iter().all(|collection| {
        collection.outcomes.len() == PolicyVariant::all().len()
            && collection
                .outcomes
                .iter()
                .map(|outcome| outcome.variant)
                .collect::<BTreeSet<_>>()
                == PolicyVariant::all().into_iter().collect()
    });
    let unique_arm_evidence = all_collections.iter().all(|collection| {
        collection
            .outcomes
            .iter()
            .map(|outcome| outcome.witness.evidence_digest)
            .collect::<BTreeSet<_>>()
            .len()
            == collection.outcomes.len()
    });
    let delayed_witnesses_only = all_collections.iter().all(|collection| {
        collection.outcomes.iter().all(|outcome| {
            let prediction = match collection.split {
                EvaluationSplit::Development => development_ledger.prediction(outcome.prediction_id),
                EvaluationSplit::SubjectHoldout => subject_ledger.prediction(outcome.prediction_id),
                EvaluationSplit::TemporalHoldout => temporal_ledger.prediction(outcome.prediction_id),
            }
            .expect("resolved prediction must remain in ledger");
            outcome.witness.observed_at_ms >= prediction.not_before_ms
                && outcome.witness.observed_at_ms <= prediction.expires_at_ms
        })
    });
    let independent_sources_only = all_collections.iter().all(|collection| {
        collection.outcomes.iter().all(|outcome| {
            outcome.witness.source != WitnessSource::ResponseGenerator
                && outcome.witness_id != outcome.producer_id
        })
    });
    let metrics_preserved = development.outcomes.iter().any(|outcome| {
        outcome.metrics.completion_bps == 7_500 && outcome.metrics.correction_count == 1
    }) && development.outcomes.iter().any(|outcome| {
        outcome.metrics.completion_bps == 10_000 && outcome.metrics.correction_count == 0
    });
    let all_predictions_finalized = [&development_ledger, &subject_ledger, &temporal_ledger]
        .iter()
        .all(|ledger| {
            ledger
                .predictions()
                .values()
                .all(|prediction| matches!(prediction.status, PredictionStatus::Resolved { .. }))
        });

    let replay_events = development_issue_events
        .into_iter()
        .chain(
            development
                .transitions
                .iter()
                .map(|transition| transition.event.clone()),
        )
        .collect::<Vec<_>>();
    let exact_replay = PredictionLedger::replay(&replay_events)
        .is_ok_and(|replayed| replayed == development_ledger);

    let (mut rejection_ledger, rejection_enrollment) = enroll(&planner, &state, 30_000, 12);
    let rejection_before = rejection_ledger.clone();
    let self_grading_rejected = collector
        .collect(
            &mut rejection_ledger,
            rejection_enrollment.ledger_version_after,
            &rejection_enrollment,
            observations(
                &rejection_enrollment,
                WitnessSource::ResponseGenerator,
                "response-generator",
            ),
        )
        .is_err();
    let unchanged_after_self_grading = rejection_ledger == rejection_before;

    let mut incomplete = observations(
        &rejection_enrollment,
        WitnessSource::ExternalEvaluator,
        "independent-evaluator-v2",
    );
    incomplete.pop();
    let incomplete_batch_rejected = collector
        .collect(
            &mut rejection_ledger,
            rejection_enrollment.ledger_version_after,
            &rejection_enrollment,
            incomplete,
        )
        .is_err();
    let atomic_rejection = unchanged_after_self_grading && rejection_ledger == rejection_before;

    let gate_passed = split_assignment_deterministic
        && complete_arm_coverage
        && unique_arm_evidence
        && delayed_witnesses_only
        && independent_sources_only
        && self_grading_rejected
        && incomplete_batch_rejected
        && atomic_rejection
        && exact_replay
        && metrics_preserved
        && all_predictions_finalized;

    let report = S5BReport {
        development_arms_resolved: development.outcomes.len(),
        subject_holdout_arms_resolved: subject.outcomes.len(),
        temporal_holdout_arms_resolved: temporal.outcomes.len(),
        split_assignment_deterministic,
        complete_arm_coverage,
        unique_arm_evidence,
        delayed_witnesses_only,
        independent_sources_only,
        self_grading_rejected,
        incomplete_batch_rejected,
        atomic_rejection,
        exact_replay,
        metrics_preserved,
        all_predictions_finalized,
        gate_passed,
        runtime_chat_wiring: false,
        response_text_influence: false,
        routing_authority: false,
        belief_promotion_authority: false,
        action_authority: false,
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("S5-B report must serialize")
    );
    assert!(gate_passed, "S5-B independent outcome collection gate failed");
}
