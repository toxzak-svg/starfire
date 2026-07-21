use serde::Serialize;
use star::companion_prediction_ledger::{
    score_outcome, AbstentionInput, OutcomeProbability, OutcomeWitness, PredictionEvent,
    PredictionInput, PredictionLedger, PredictionLedgerError, PredictionProducer,
    PredictionProducerKind, WitnessSource,
};

#[derive(Debug, Serialize)]
struct S4Report {
    resolved_predictions: u64,
    abstentions: u64,
    expired_predictions: u64,
    candidate_mean_brier_ppm: u32,
    majority_mean_brier_ppm: u32,
    recency_mean_brier_ppm: u32,
    scrambled_scope_mean_brier_ppm: u32,
    oracle_mean_brier_ppm: u32,
    replay_equal: bool,
    self_grading_rejected: bool,
    premature_witness_rejected: bool,
    duplicate_resolution_rejected: bool,
    trivial_controls_beaten: bool,
    oracle_bound_respected: bool,
    gate_passed: bool,
    live_response_influence: bool,
    routing_authority: bool,
    belief_promotion_authority: bool,
    action_authority: bool,
}

fn producer() -> PredictionProducer {
    PredictionProducer {
        id: "s4-frozen-candidate-v1".to_owned(),
        kind: PredictionProducerKind::CompanionPolicy,
    }
}

fn outcomes(yes_bps: u16) -> Vec<OutcomeProbability> {
    vec![
        OutcomeProbability {
            label: "engaged".to_owned(),
            probability_bps: yes_bps,
        },
        OutcomeProbability {
            label: "not_engaged".to_owned(),
            probability_bps: 10_000 - yes_bps,
        },
    ]
}

fn mean_brier(probabilities: &[u16], observations: &[bool]) -> u32 {
    let total = probabilities
        .iter()
        .zip(observations)
        .map(|(yes_bps, observed)| {
            let label = if *observed {
                "engaged"
            } else {
                "not_engaged"
            };
            u64::from(score_outcome(&outcomes(*yes_bps), label).unwrap().score_ppm)
        })
        .sum::<u64>();
    (total / probabilities.len() as u64) as u32
}

fn main() {
    let observations = [true, true, false, true, false, true];
    let candidate_probabilities = [7_500, 7_000, 3_500, 8_000, 3_000, 7_500];
    let majority_probabilities = [6_670; 6];
    let recency_probabilities = [5_000, 8_000, 8_000, 2_000, 8_000, 2_000];
    let scrambled_scope_probabilities = [3_500, 8_000, 3_000, 7_500, 7_500, 7_000];
    let oracle_probabilities = observations.map(|observed| if observed { 10_000 } else { 0 });

    let mut ledger = PredictionLedger::new();
    let mut events = Vec::<PredictionEvent>::new();
    let mut self_grading_rejected = false;
    let mut premature_witness_rejected = false;
    let mut duplicate_resolution_rejected = false;

    for (index, (yes_bps, observed)) in candidate_probabilities
        .iter()
        .zip(observations)
        .enumerate()
    {
        let issued_at_ms = 1_000 + index as u64 * 100;
        let issued = ledger
            .issue(
                ledger.version,
                PredictionInput {
                    subject_scope: format!("opaque-subject-{index}"),
                    producer: producer(),
                    outcomes: outcomes(*yes_bps),
                    issued_at_ms,
                    not_before_ms: issued_at_ms + 10,
                    expires_at_ms: issued_at_ms + 90,
                    context_digest: 100 + index as u64,
                },
            )
            .unwrap();
        let prediction_id = issued.prediction_id.unwrap();
        events.push(issued.event);

        if index == 0 {
            self_grading_rejected = matches!(
                ledger.resolve(
                    ledger.version,
                    prediction_id,
                    OutcomeWitness {
                        source: WitnessSource::ResponseGenerator,
                        label: "engaged".to_owned(),
                        observed_at_ms: issued_at_ms + 20,
                        evidence_digest: 700,
                    },
                ),
                Err(PredictionLedgerError::SelfGradingWitness)
            );
            premature_witness_rejected = matches!(
                ledger.resolve(
                    ledger.version,
                    prediction_id,
                    OutcomeWitness {
                        source: WitnessSource::Environment,
                        label: "engaged".to_owned(),
                        observed_at_ms: issued_at_ms + 5,
                        evidence_digest: 701,
                    },
                ),
                Err(PredictionLedgerError::WitnessTooEarly { .. })
            );
        }

        let resolved = ledger
            .resolve(
                ledger.version,
                prediction_id,
                OutcomeWitness {
                    source: WitnessSource::ExternalEvaluator,
                    label: if observed {
                        "engaged".to_owned()
                    } else {
                        "not_engaged".to_owned()
                    },
                    observed_at_ms: issued_at_ms + 20,
                    evidence_digest: 1_000 + index as u64,
                },
            )
            .unwrap();
        events.push(resolved.event);

        if index == 0 {
            duplicate_resolution_rejected = matches!(
                ledger.resolve(
                    ledger.version,
                    prediction_id,
                    OutcomeWitness {
                        source: WitnessSource::UserObservation,
                        label: "engaged".to_owned(),
                        observed_at_ms: issued_at_ms + 30,
                        evidence_digest: 702,
                    },
                ),
                Err(PredictionLedgerError::PredictionAlreadyFinalized(id)) if id == prediction_id
            );
        }
    }

    let abstained = ledger
        .abstain(
            ledger.version,
            AbstentionInput {
                subject_scope: "opaque-subject-abstention".to_owned(),
                producer: producer(),
                reason: "insufficient evidence".to_owned(),
                occurred_at_ms: 2_000,
                context_digest: 2_000,
            },
        )
        .unwrap();
    events.push(abstained.event);

    let expiring = ledger
        .issue(
            ledger.version,
            PredictionInput {
                subject_scope: "opaque-subject-expiry".to_owned(),
                producer: producer(),
                outcomes: outcomes(5_000),
                issued_at_ms: 2_100,
                not_before_ms: 2_110,
                expires_at_ms: 2_150,
                context_digest: 2_100,
            },
        )
        .unwrap();
    let expiring_id = expiring.prediction_id.unwrap();
    events.push(expiring.event);
    let expired = ledger
        .expire(ledger.version, expiring_id, 2_150)
        .unwrap();
    events.push(expired.event);

    let summary = ledger.summary();
    let candidate_mean_brier_ppm = mean_brier(&candidate_probabilities, &observations);
    let majority_mean_brier_ppm = mean_brier(&majority_probabilities, &observations);
    let recency_mean_brier_ppm = mean_brier(&recency_probabilities, &observations);
    let scrambled_scope_mean_brier_ppm =
        mean_brier(&scrambled_scope_probabilities, &observations);
    let oracle_mean_brier_ppm = mean_brier(&oracle_probabilities, &observations);
    let replay_equal = PredictionLedger::replay(&events).is_ok_and(|replayed| replayed == ledger);
    let trivial_controls_beaten = candidate_mean_brier_ppm < majority_mean_brier_ppm
        && candidate_mean_brier_ppm < recency_mean_brier_ppm
        && candidate_mean_brier_ppm < scrambled_scope_mean_brier_ppm;
    let oracle_bound_respected = oracle_mean_brier_ppm <= candidate_mean_brier_ppm;
    let gate_passed = summary.resolved == observations.len() as u64
        && summary.abstentions == 1
        && summary.expired == 1
        && replay_equal
        && self_grading_rejected
        && premature_witness_rejected
        && duplicate_resolution_rejected
        && trivial_controls_beaten
        && oracle_bound_respected;

    let report = S4Report {
        resolved_predictions: summary.resolved,
        abstentions: summary.abstentions,
        expired_predictions: summary.expired,
        candidate_mean_brier_ppm,
        majority_mean_brier_ppm,
        recency_mean_brier_ppm,
        scrambled_scope_mean_brier_ppm,
        oracle_mean_brier_ppm,
        replay_equal,
        self_grading_rejected,
        premature_witness_rejected,
        duplicate_resolution_rejected,
        trivial_controls_beaten,
        oracle_bound_respected,
        gate_passed,
        live_response_influence: false,
        routing_authority: false,
        belief_promotion_authority: false,
        action_authority: false,
    };

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    assert!(gate_passed, "S4 falsifiable prediction-ledger gate failed");
}
