#![cfg(feature = "developmental-evidence")]

use star::developmental::{
    compare_numeric_transition, EvidenceValidationPolicy, LearnedEvidence,
    LearnedPayload,
};

#[test]
fn infant_prediction_and_independent_observation_produce_known_residual() {
    let prediction_raw = include_str!(
        "fixtures/infant_numeric_transition_prediction_v1.json"
    )
    .trim();
    let observation_raw = include_str!(
        "fixtures/infant_numeric_state_observation_v1.json"
    )
    .trim();

    let prediction_evidence: LearnedEvidence = serde_json::from_str(prediction_raw)
        .expect("Infant numeric prediction fixture must deserialize");
    let observation_evidence: LearnedEvidence = serde_json::from_str(observation_raw)
        .expect("independent numeric observation fixture must deserialize");

    prediction_evidence
        .validate(&EvidenceValidationPolicy::replay(1_700_000_101))
        .expect("prediction fixture must pass replay validation");
    observation_evidence
        .validate(&EvidenceValidationPolicy::replay(1_700_000_101))
        .expect("observation fixture must pass replay validation");

    assert!(prediction_evidence.timestamp < observation_evidence.timestamp);
    assert_ne!(prediction_evidence.source_model, observation_evidence.source_model);
    assert_ne!(
        prediction_evidence.provenance.producer,
        observation_evidence.provenance.producer
    );

    let prediction = match &prediction_evidence.payload {
        LearnedPayload::NumericTransitionPrediction(prediction) => prediction,
        other => panic!("expected numeric transition prediction, got {other:?}"),
    };
    let observation = match &observation_evidence.payload {
        LearnedPayload::NumericStateObservation(observation) => observation,
        other => panic!("expected numeric state observation, got {other:?}"),
    };

    let residual = compare_numeric_transition(prediction, observation)
        .expect("matching fixture pair must produce a residual");

    assert_eq!(residual.transition_id, "transition-numeric-0001");
    assert_eq!(residual.state_space, "synthetic.proprio.v1");
    assert_eq!(residual.dimension, 3);

    let expected_signed_error = [0.1_f32, -0.2_f32, 0.3_f32];
    for (actual, expected) in residual
        .signed_error
        .iter()
        .zip(expected_signed_error.iter())
    {
        assert!((actual - expected).abs() < 1e-6);
    }

    assert!((residual.mean_absolute_error - 0.2).abs() < 1e-6);
    assert!((residual.mean_squared_error - (0.14 / 3.0)).abs() < 1e-6);
    assert!(
        (residual.root_mean_squared_error - (0.14_f64 / 3.0).sqrt()).abs()
            < 1e-6
    );
    assert!((residual.max_absolute_error - 0.3).abs() < 1e-6);
}
