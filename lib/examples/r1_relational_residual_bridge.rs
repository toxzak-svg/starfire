#[cfg(not(feature = "relational-evidence"))]
fn main() {
    eprintln!("enable --features relational-evidence to run the R1 probe");
}

#[cfg(feature = "relational-evidence")]
fn main() {
    use star::relational::{
        EvidenceProducer, OutcomeSource, PredictedOutcome, PredictionHorizon,
        RelationalOutcomeWitness, RelationalPrediction, RelationalReplayCase,
        RelationalResidualBridge, RELATIONAL_EVIDENCE_SCHEMA_VERSION,
    };

    fn prediction(id: &str, primary_probability: f64) -> RelationalPrediction {
        RelationalPrediction {
            schema_version: RELATIONAL_EVIDENCE_SCHEMA_VERSION,
            prediction_id: id.into(),
            subject_id: "subject-opaque-demo".into(),
            target: "response_policy".into(),
            context_scope: "project:implementation_authorized".into(),
            issued_at_sequence: 20,
            horizon: PredictionHorizon::CurrentTurn,
            outcomes: vec![
                PredictedOutcome {
                    label: "direct_implementation".into(),
                    probability: primary_probability,
                },
                PredictedOutcome {
                    label: "explain_first".into(),
                    probability: 1.0 - primary_probability,
                },
            ],
            producer: EvidenceProducer {
                name: "ingexuity".into(),
                version: "r1-demo".into(),
                state_hash: Some("state-20".into()),
            },
        }
    }

    fn witness(id: &str, label: &str) -> RelationalOutcomeWitness {
        RelationalOutcomeWitness {
            schema_version: RELATIONAL_EVIDENCE_SCHEMA_VERSION,
            prediction_id: id.into(),
            observed_at_sequence: 21,
            observed_label: label.into(),
            source: OutcomeSource::ExplicitUserCorrection,
            confidence: 1.0,
            evidence_id: format!("turn-21:{id}"),
        }
    }

    let cases = vec![
        RelationalReplayCase {
            prediction: prediction("wrong-confident", 0.9),
            witness: witness("wrong-confident", "explain_first"),
        },
        RelationalReplayCase {
            prediction: prediction("correct-confident", 0.9),
            witness: witness("correct-confident", "direct_implementation"),
        },
    ];
    let report = RelationalResidualBridge::default()
        .replay(&cases)
        .expect("R1 replay fixture must be valid");

    assert_eq!(report.case_count, 2);
    assert_eq!(report.emitted_charge_count, 1);
    assert!(report
        .assessments
        .iter()
        .all(|assessment| !assessment.promotion_eligible));

    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("report must serialize")
    );
}
