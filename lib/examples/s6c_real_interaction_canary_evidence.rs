use serde::Serialize;
use star::companion_interaction_outcomes::{
    InteractionOutcomeLedger, ObservedSignal, PairwisePreference,
};
use star::companion_interaction_policy::{
    PolicyContext, PolicyVariant, ShadowPolicyPlanner,
};
use star::companion_policy_evaluation::{
    ArmComputeObservation, EvaluationSplit, EvaluationSplitPolicy, PolicyEvaluationConfig,
    PolicyEvaluationVerdict,
};
use star::companion_prediction_ledger::{PredictionLedger, WitnessSource};
use star::companion_real_interaction_canary::{
    CanaryEvidenceError, CanaryEvidenceLedger, CanaryEvidenceOrigin, CanaryEvaluationReport,
    CanaryStudyConfig, DirectCanaryAttestation, PairwiseCanaryAttestation,
};
use star::companion_state::{
    ClaimInput, ClaimSource, CompanionState, Retention, Sensitivity,
};

const PRODUCER_DIGEST: u64 = 0xA11CE;

#[derive(Debug, Default, Serialize)]
struct AdversarialChecks {
    production_rejected_synthetic: bool,
    duplicate_seal_rejected: bool,
    stale_version_rejected_atomically: bool,
    unsealed_trial_rejected_atomically: bool,
    response_generator_rejected_atomically: bool,
    same_identity_rejected_atomically: bool,
    consent_mismatch_rejected_atomically: bool,
    early_evidence_rejected_atomically: bool,
    expired_evidence_rejected_atomically: bool,
    duplicate_direct_rejected_atomically: bool,
    invalid_pairwise_rejected_atomically: bool,
    duplicate_pairwise_rejected_atomically: bool,
}

impl AdversarialChecks {
    fn all_passed(&self) -> bool {
        self.production_rejected_synthetic
            && self.duplicate_seal_rejected
            && self.stale_version_rejected_atomically
            && self.unsealed_trial_rejected_atomically
            && self.response_generator_rejected_atomically
            && self.same_identity_rejected_atomically
            && self.consent_mismatch_rejected_atomically
            && self.early_evidence_rejected_atomically
            && self.expired_evidence_rejected_atomically
            && self.duplicate_direct_rejected_atomically
            && self.invalid_pairwise_rejected_atomically
            && self.duplicate_pairwise_rejected_atomically
    }
}

#[derive(Debug, Serialize)]
struct S6CProbeReport {
    mechanism_gate_passed: bool,
    underlying_s5c_verdict: PolicyEvaluationVerdict,
    underlying_s5c_promotion_eligible: bool,
    canary_promotion_eligible: bool,
    synthetic_evidence_count: u64,
    real_evidence_count: u64,
    raw_content_retained: bool,
    canary_replay_equal: bool,
    s5b_replay_equal: bool,
    source_state_unchanged: bool,
    adversarial: AdversarialChecks,
    live_response_influence: bool,
    routing_authority: bool,
    belief_promotion_authority: bool,
    persistence_authority: bool,
    tool_authority: bool,
    action_authority: bool,
    report: CanaryEvaluationReport,
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

fn split_seed(split: EvaluationSplit, ordinal: u64) -> (u64, u64) {
    match split {
        EvaluationSplit::Development => (10_000 + ordinal * 10, 2_000 + ordinal * 2),
        EvaluationSplit::OpaqueSubjectHoldout => {
            (50_000 + ordinal * 10, 3_001 + ordinal * 2)
        }
        EvaluationSplit::TemporalHoldout => (100_000 + ordinal * 10, 4_000 + ordinal),
    }
}

fn canary_config(allow_synthetic_fixture: bool) -> CanaryStudyConfig {
    CanaryStudyConfig {
        study_digest: 0x600C,
        protocol_digest: 0xC001,
        split_policy: EvaluationSplitPolicy {
            temporal_holdout_start_ms: 100_000,
            opaque_subject_modulus: 2,
            opaque_subject_remainder: 1,
        },
        allow_synthetic_fixture,
    }
}

fn evaluation_config() -> PolicyEvaluationConfig {
    PolicyEvaluationConfig {
        split_policy: canary_config(true).split_policy,
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
    let enrollment = planner
        .enroll(state, predictions, predictions.version, policy_context)
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

fn direct_attestation(
    trial_id: u64,
    consent_digest: u64,
    observed_at_ms: u64,
    witness_digest: u64,
) -> DirectCanaryAttestation {
    DirectCanaryAttestation {
        origin: CanaryEvidenceOrigin::SyntheticFixture,
        trial_id,
        signal: ObservedSignal::TaskCompleted,
        source: WitnessSource::Environment,
        witness_digest,
        producer_digest: PRODUCER_DIGEST,
        consent_digest,
        observed_at_ms,
        evidence_digest: 0xE000_0000 + trial_id,
    }
}

fn pairwise_attestation(
    trial_id: u64,
    control: PolicyVariant,
    consent_digest: u64,
    observed_at_ms: u64,
    digest_seed: u64,
) -> PairwiseCanaryAttestation {
    PairwiseCanaryAttestation {
        origin: CanaryEvidenceOrigin::SyntheticFixture,
        trial_id,
        left_variant: PolicyVariant::CompanionDerived,
        right_variant: control,
        preference: PairwisePreference::Left,
        left_render_digest: digest_seed + 1,
        right_render_digest: digest_seed + 2,
        blinded_order_digest: digest_seed + 3,
        evaluator_digest: digest_seed + 4,
        producer_digest: PRODUCER_DIGEST,
        consent_digest,
        observed_at_ms,
        evidence_digest: digest_seed + 5,
    }
}

fn direct_rejected_atomically<F>(
    canary: &mut CanaryEvidenceLedger,
    outcomes: &mut InteractionOutcomeLedger,
    predictions: &mut PredictionLedger,
    attestation: DirectCanaryAttestation,
    predicate: F,
) -> bool
where
    F: FnOnce(&CanaryEvidenceError) -> bool,
{
    let canary_before = canary.clone();
    let outcomes_before = outcomes.clone();
    let predictions_before = predictions.clone();
    let result = canary.import_direct(canary.version, outcomes, predictions, attestation);
    let rejected_as_expected = match &result {
        Err(error) => predicate(error),
        Ok(_) => false,
    };
    rejected_as_expected
        && *canary == canary_before
        && *outcomes == outcomes_before
        && *predictions == predictions_before
}

fn pairwise_rejected_atomically<F>(
    canary: &mut CanaryEvidenceLedger,
    outcomes: &mut InteractionOutcomeLedger,
    predictions: &mut PredictionLedger,
    attestation: PairwiseCanaryAttestation,
    predicate: F,
) -> bool
where
    F: FnOnce(&CanaryEvidenceError) -> bool,
{
    let canary_before = canary.clone();
    let outcomes_before = outcomes.clone();
    let predictions_before = predictions.clone();
    let result = canary.import_pairwise(canary.version, outcomes, predictions, attestation);
    let rejected_as_expected = match &result {
        Err(error) => predicate(error),
        Ok(_) => false,
    };
    rejected_as_expected
        && *canary == canary_before
        && *outcomes == outcomes_before
        && *predictions == predictions_before
}

#[allow(clippy::too_many_arguments)]
fn populate_split(
    split: EvaluationSplit,
    state: &CompanionState,
    planner: &ShadowPolicyPlanner,
    predictions: &mut PredictionLedger,
    outcomes: &mut InteractionOutcomeLedger,
    canary: &mut CanaryEvidenceLedger,
    costs: &mut Vec<ArmComputeObservation>,
    digest_seed: &mut u64,
    checks: &mut AdversarialChecks,
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
        let consent_digest = 0xC000_0000 + trial_id;
        let operator_digest = 0xD000_0000 + trial_id;
        canary
            .seal_trial(
                canary.version,
                outcomes,
                trial_id,
                consent_digest,
                operator_digest,
            )
            .unwrap();

        if !checks.production_rejected_synthetic {
            let mut production = CanaryEvidenceLedger::new(canary_config(false)).unwrap();
            production
                .seal_trial(0, outcomes, trial_id, consent_digest, operator_digest)
                .unwrap();
            let mut production_outcomes = outcomes.clone();
            let mut production_predictions = predictions.clone();
            checks.production_rejected_synthetic = direct_rejected_atomically(
                &mut production,
                &mut production_outcomes,
                &mut production_predictions,
                direct_attestation(trial_id, consent_digest, issued_at_ms + 200, 0x7001),
                |error| matches!(error, CanaryEvidenceError::SyntheticFixtureRejected),
            );

            let canary_before = canary.clone();
            checks.duplicate_seal_rejected = matches!(
                canary.seal_trial(
                    canary.version,
                    outcomes,
                    trial_id,
                    consent_digest,
                    operator_digest,
                ),
                Err(CanaryEvidenceError::DuplicateSeal(id)) if id == trial_id
            ) && *canary == canary_before;

            let canary_before = canary.clone();
            checks.stale_version_rejected_atomically = matches!(
                canary.seal_trial(
                    canary.version.saturating_sub(1),
                    outcomes,
                    trial_id,
                    consent_digest,
                    operator_digest,
                ),
                Err(CanaryEvidenceError::VersionConflict { .. })
            ) && *canary == canary_before;

            checks.unsealed_trial_rejected_atomically = direct_rejected_atomically(
                canary,
                outcomes,
                predictions,
                direct_attestation(u64::MAX, consent_digest, issued_at_ms + 200, 0x7002),
                |error| matches!(error, CanaryEvidenceError::UnsealedTrial(id) if *id == u64::MAX),
            );

            let mut response_generator =
                direct_attestation(trial_id, consent_digest, issued_at_ms + 200, 0x7003);
            response_generator.source = WitnessSource::ResponseGenerator;
            checks.response_generator_rejected_atomically = direct_rejected_atomically(
                canary,
                outcomes,
                predictions,
                response_generator,
                |error| matches!(error, CanaryEvidenceError::ResponseGeneratorWitnessRejected),
            );

            let mut same_identity =
                direct_attestation(trial_id, consent_digest, issued_at_ms + 200, PRODUCER_DIGEST);
            same_identity.witness_digest = PRODUCER_DIGEST;
            checks.same_identity_rejected_atomically = direct_rejected_atomically(
                canary,
                outcomes,
                predictions,
                same_identity,
                |error| matches!(error, CanaryEvidenceError::WitnessNotIndependent),
            );

            let mut wrong_consent =
                direct_attestation(trial_id, consent_digest, issued_at_ms + 200, 0x7004);
            wrong_consent.consent_digest += 1;
            checks.consent_mismatch_rejected_atomically = direct_rejected_atomically(
                canary,
                outcomes,
                predictions,
                wrong_consent,
                |error| matches!(error, CanaryEvidenceError::ConsentMismatch),
            );

            checks.early_evidence_rejected_atomically = direct_rejected_atomically(
                canary,
                outcomes,
                predictions,
                direct_attestation(trial_id, consent_digest, issued_at_ms, 0x7005),
                |error| matches!(error, CanaryEvidenceError::EvidenceBeforeWindow),
            );

            checks.expired_evidence_rejected_atomically = direct_rejected_atomically(
                canary,
                outcomes,
                predictions,
                direct_attestation(trial_id, consent_digest, issued_at_ms + 1_001, 0x7006),
                |error| matches!(error, CanaryEvidenceError::EvidenceAfterWindow),
            );
        }

        let valid = direct_attestation(
            trial_id,
            consent_digest,
            issued_at_ms + 200,
            0x7100_0000 + trial_id,
        );
        canary
            .import_direct(canary.version, outcomes, predictions, valid.clone())
            .unwrap();
        if !checks.duplicate_direct_rejected_atomically {
            checks.duplicate_direct_rejected_atomically = direct_rejected_atomically(
                canary,
                outcomes,
                predictions,
                valid,
                |error| {
                    matches!(error, CanaryEvidenceError::DuplicateDirectEvidence(id) if *id == trial_id)
                },
            );
        }
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
        *digest_seed += 10;
        let trial_id = register_trial(
            state,
            planner,
            predictions,
            outcomes,
            costs,
            context(*digest_seed, subject_scope_digest, issued_at_ms),
            None,
        );
        let consent_digest = 0xC000_0000 + trial_id;
        canary
            .seal_trial(
                canary.version,
                outcomes,
                trial_id,
                consent_digest,
                0xD000_0000 + trial_id,
            )
            .unwrap();

        let valid = pairwise_attestation(
            trial_id,
            control,
            consent_digest,
            issued_at_ms + 200,
            *digest_seed + 0x1000,
        );
        if !checks.invalid_pairwise_rejected_atomically {
            let mut invalid = valid.clone();
            invalid.right_render_digest = invalid.left_render_digest;
            checks.invalid_pairwise_rejected_atomically = pairwise_rejected_atomically(
                canary,
                outcomes,
                predictions,
                invalid,
                |error| matches!(error, CanaryEvidenceError::InvalidPairwiseEvidence),
            );
        }
        canary
            .import_pairwise(canary.version, outcomes, predictions, valid.clone())
            .unwrap();
        if !checks.duplicate_pairwise_rejected_atomically {
            checks.duplicate_pairwise_rejected_atomically = pairwise_rejected_atomically(
                canary,
                outcomes,
                predictions,
                valid,
                |error| matches!(error, CanaryEvidenceError::DuplicatePairwiseEvidence { .. }),
            );
        }
        ordinal += 1;
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
    let mut canary = CanaryEvidenceLedger::new(canary_config(true)).unwrap();
    let mut costs = Vec::new();
    let mut digest_seed = 0x6000_u64;
    let mut checks = AdversarialChecks::default();

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
            &mut canary,
            &mut costs,
            &mut digest_seed,
            &mut checks,
        );
    }

    let report = canary
        .evaluate(&outcomes, &costs, evaluation_config())
        .unwrap();
    let canary_replay_equal =
        CanaryEvidenceLedger::replay(canary.config(), canary.events()).unwrap() == canary;
    let s5b_replay_equal =
        InteractionOutcomeLedger::replay(&base_predictions, outcomes.events()).unwrap() == outcomes;
    let source_state_unchanged = state == source_before;

    let mechanism_gate_passed = checks.all_passed()
        && report.s5c_report.verdict == PolicyEvaluationVerdict::Pass
        && report.s5c_report.promotion_eligible
        && !report.promotion_eligible
        && report.evidence.synthetic_evidence > 0
        && report.evidence.real_evidence == 0
        && !report.evidence.raw_content_retained
        && report.all_trials_sealed
        && report.opaque_subject_holdout_present
        && report.temporal_holdout_present
        && !report.all_evidence_real
        && canary_replay_equal
        && s5b_replay_equal
        && source_state_unchanged
        && !report.live_response_influence
        && !report.routing_authority
        && !report.belief_promotion_authority
        && !report.persistence_authority
        && !report.tool_authority
        && !report.action_authority;

    let probe = S6CProbeReport {
        mechanism_gate_passed,
        underlying_s5c_verdict: report.s5c_report.verdict,
        underlying_s5c_promotion_eligible: report.s5c_report.promotion_eligible,
        canary_promotion_eligible: report.promotion_eligible,
        synthetic_evidence_count: report.evidence.synthetic_evidence,
        real_evidence_count: report.evidence.real_evidence,
        raw_content_retained: report.evidence.raw_content_retained,
        canary_replay_equal,
        s5b_replay_equal,
        source_state_unchanged,
        adversarial: checks,
        live_response_influence: report.live_response_influence,
        routing_authority: report.routing_authority,
        belief_promotion_authority: report.belief_promotion_authority,
        persistence_authority: report.persistence_authority,
        tool_authority: report.tool_authority,
        action_authority: report.action_authority,
        report,
    };

    println!("{}", serde_json::to_string_pretty(&probe).unwrap());
    assert!(
        mechanism_gate_passed,
        "S6-C canary-evidence mechanism gate failed"
    );
}
