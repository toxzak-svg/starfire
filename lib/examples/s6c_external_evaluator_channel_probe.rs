use serde::Serialize;
use star::companion_interaction_outcomes::{
    InteractionOutcomeError, InteractionOutcomeLedger, ObservedSignal,
};
use star::companion_interaction_policy::{
    PolicyContext, PolicyVariant, ShadowPolicyPlanner,
};
use star::companion_policy_evaluation::EvaluationSplitPolicy;
use star::companion_prediction_ledger::{PredictionLedger, WitnessSource};
use star::companion_real_interaction_canary::{
    CanaryEvidenceError, CanaryEvidenceLedger, CanaryEvidenceOrigin, CanaryStudyConfig,
    DirectCanaryAttestation,
};
use star::companion_state::{
    ClaimInput, ClaimSource, CompanionState, Retention, Sensitivity,
};

#[derive(Debug, Serialize)]
struct ExternalEvaluatorChannelReport {
    rejected_as_wrong_channel: bool,
    canary_unchanged: bool,
    outcomes_unchanged: bool,
    predictions_unchanged: bool,
    gate_passed: bool,
}

fn claim(key: &str, value: &str, observed_at_ms: u64) -> ClaimInput {
    ClaimInput {
        key: key.to_owned(),
        value: value.to_owned(),
        source: ClaimSource::UserStatement,
        confidence_bps: 9_000,
        sensitivity: Sensitivity::Personal,
        retention: Retention::Durable,
        observed_at_ms,
    }
}

fn main() {
    let mut state = CompanionState::new();
    let first = state
        .record_claim(0, claim("preference.detail.general", "brief", 10))
        .unwrap();
    state
        .record_claim(
            first.version,
            claim("knowledge.strong_domain.rust", "rust", 20),
        )
        .unwrap();

    let planner = ShadowPolicyPlanner::default();
    let mut predictions = PredictionLedger::new();
    let expected_prediction_version = predictions.version;
    let enrollment = planner
        .enroll(
            &state,
            &mut predictions,
            expected_prediction_version,
            PolicyContext {
                context_digest: 0xC001,
                subject_scope_digest: 0xAA,
                domain: Some("rust".to_owned()),
                technical_context: true,
                asks_for_explanation: true,
                emotional_signal: false,
                issued_at_ms: 1_000,
                not_before_ms: 1_100,
                expires_at_ms: 2_000,
            },
        )
        .unwrap();

    let mut outcomes = InteractionOutcomeLedger::new(&predictions);
    let trial_id = outcomes
        .register_enrollment(
            outcomes.version,
            &predictions,
            &enrollment,
            Some(PolicyVariant::CompanionDerived),
        )
        .unwrap()
        .trial_id;

    let mut canary = CanaryEvidenceLedger::new(CanaryStudyConfig {
        study_digest: 0x600C,
        protocol_digest: 0xC001,
        split_policy: EvaluationSplitPolicy {
            temporal_holdout_start_ms: 10_000,
            opaque_subject_modulus: 2,
            opaque_subject_remainder: 1,
        },
        allow_synthetic_fixture: true,
    })
    .unwrap();
    let consent_digest = 0xC000_0000 + trial_id;
    canary
        .seal_trial(
            canary.version,
            &outcomes,
            trial_id,
            consent_digest,
            0xD000_0000 + trial_id,
        )
        .unwrap();

    let canary_before = canary.clone();
    let outcomes_before = outcomes.clone();
    let predictions_before = predictions.clone();

    let result = canary.import_direct(
        canary.version,
        &mut outcomes,
        &mut predictions,
        DirectCanaryAttestation {
            origin: CanaryEvidenceOrigin::SyntheticFixture,
            trial_id,
            signal: ObservedSignal::TaskCompleted,
            source: WitnessSource::ExternalEvaluator,
            witness_digest: 0xE001,
            producer_digest: 0xA11CE,
            consent_digest,
            observed_at_ms: 1_200,
            evidence_digest: 0xEE00_0000 + trial_id,
        },
    );

    let rejected_as_wrong_channel = matches!(
        result,
        Err(CanaryEvidenceError::Outcome(
            InteractionOutcomeError::WrongEvidenceChannel
        ))
    );
    let canary_unchanged = canary == canary_before;
    let outcomes_unchanged = outcomes == outcomes_before;
    let predictions_unchanged = predictions == predictions_before;
    let gate_passed = rejected_as_wrong_channel
        && canary_unchanged
        && outcomes_unchanged
        && predictions_unchanged;

    let report = ExternalEvaluatorChannelReport {
        rejected_as_wrong_channel,
        canary_unchanged,
        outcomes_unchanged,
        predictions_unchanged,
        gate_passed,
    };
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    assert!(
        gate_passed,
        "external evaluator escaped the S5-B pairwise-only channel"
    );
}
