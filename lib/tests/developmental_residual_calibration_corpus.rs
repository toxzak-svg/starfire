#![cfg(feature = "developmental-evidence")]

use std::collections::BTreeMap;

use serde::Deserialize;
use star::developmental::{
    compare_numeric_transition, NamedVector, NumericStateObservation,
    NumericTransitionPrediction, ResidualCalibrationProfile,
    ResidualCalibrationScope,
};

#[derive(Debug, Deserialize)]
struct Corpus {
    schema_version: u16,
    source: Source,
    scope: CorpusScope,
    calibration_rule: CalibrationRule,
    calibration: PairSet,
    test: TestSet,
    python_reference: PythonReference,
}

#[derive(Debug, Deserialize)]
struct Source {
    source_head_sha: String,
    source_artifact_digest: String,
    source_corpus_sha256: String,
}

#[derive(Debug, Deserialize)]
struct CorpusScope {
    state_space: String,
    prediction_horizon_steps: u32,
    seed: u64,
    epochs: u64,
    train_samples: usize,
    subset_stride: usize,
    subset_samples: usize,
}

#[derive(Debug, Deserialize)]
struct CalibrationRule {
    quantile: f64,
    method: String,
    metric: String,
    comparison: String,
}

#[derive(Debug, Deserialize)]
struct PairSet {
    predicted: Vec<Vec<f32>>,
    observed: Vec<Vec<f32>>,
}

#[derive(Debug, Deserialize)]
struct TestSet {
    observed: Vec<Vec<f32>>,
    predicted_by_condition: BTreeMap<String, Vec<Vec<f32>>>,
}

#[derive(Debug, Deserialize)]
struct PythonReference {
    threshold_mse: f64,
    flagged_rate_by_condition: BTreeMap<String, f64>,
    mean_mse_by_condition: BTreeMap<String, f64>,
}

fn residual_from_pair(
    transition_id: String,
    state_space: &str,
    horizon_steps: u32,
    predicted: &[f32],
    observed: &[f32],
) -> star::developmental::NumericPredictionResidual {
    let prediction = NumericTransitionPrediction {
        transition_id: transition_id.clone(),
        action: NamedVector {
            space: "synthetic.action.v1".to_string(),
            values: vec![0.0, 0.0, 0.0],
        },
        predicted_next_state: NamedVector {
            space: state_space.to_string(),
            values: predicted.to_vec(),
        },
        horizon_steps,
    };
    let observation = NumericStateObservation {
        transition_id,
        state: NamedVector {
            space: state_space.to_string(),
            values: observed.to_vec(),
        },
    };

    compare_numeric_transition(&prediction, &observation)
        .expect("raw corpus pair must be structurally comparable")
}

#[test]
fn starfire_recomputes_calibration_from_certified_raw_infant_corpus() {
    let raw = include_str!(
        "fixtures/infant_residual_calibration_subset_v1.json"
    );
    let corpus: Corpus =
        serde_json::from_str(raw).expect("calibration corpus must deserialize");

    assert_eq!(corpus.schema_version, 1);
    assert_eq!(
        corpus.source.source_head_sha,
        "fcb2f174c21b21a66742e190d7a611780ae87281"
    );
    assert_eq!(
        corpus.source.source_artifact_digest,
        "sha256:458493c90b7d7bd7ceff831fbe9e2b02919374407cc06315625f3c2e53241939"
    );
    assert_eq!(
        corpus.source.source_corpus_sha256,
        "baed41e7e8a35474def48d9941eff92592665352528d125a3a9b49d79b5bbb92"
    );
    assert_eq!(corpus.scope.subset_stride, 8);
    assert_eq!(corpus.scope.subset_samples, 32);
    assert_eq!(corpus.calibration_rule.method, "higher");
    assert_eq!(
        corpus.calibration_rule.metric,
        "per_transition_mean_squared_error"
    );
    assert_eq!(
        corpus.calibration_rule.comparison,
        "residual_mse > threshold_mse"
    );

    assert_eq!(
        corpus.calibration.predicted.len(),
        corpus.calibration.observed.len()
    );
    assert_eq!(
        corpus.calibration.predicted.len(),
        corpus.scope.subset_samples
    );

    let calibration_residuals: Vec<_> = corpus
        .calibration
        .predicted
        .iter()
        .zip(corpus.calibration.observed.iter())
        .enumerate()
        .map(|(index, (predicted, observed))| {
            residual_from_pair(
                format!("calibration-{index}"),
                &corpus.scope.state_space,
                corpus.scope.prediction_horizon_steps,
                predicted,
                observed,
            )
        })
        .collect();

    let scope = ResidualCalibrationScope {
        predictor_scope: format!(
            "infant/f0_true_transition_probe/seed{}/epochs{}/train{}",
            corpus.scope.seed,
            corpus.scope.epochs,
            corpus.scope.train_samples
        ),
        environment_scope: "synthetic_environment/movement_v1".to_string(),
        state_space: corpus.scope.state_space.clone(),
        horizon_steps: corpus.scope.prediction_horizon_steps,
    };

    let profile = ResidualCalibrationProfile::fit_higher_quantile(
        scope.clone(),
        &calibration_residuals,
        corpus.calibration_rule.quantile,
    )
    .expect("Starfire must fit the calibration profile from raw residuals");

    assert!(
        (profile.threshold - corpus.python_reference.threshold_mse).abs()
            < 1e-15,
        "Rust threshold {} differs from Python reference {}",
        profile.threshold,
        corpus.python_reference.threshold_mse
    );

    assert_eq!(corpus.test.observed.len(), corpus.scope.subset_samples);

    for (condition, predictions) in &corpus.test.predicted_by_condition {
        assert_eq!(predictions.len(), corpus.test.observed.len());

        let mut flagged = 0usize;
        let mut mse_sum = 0.0_f64;
        for (index, (predicted, observed)) in predictions
            .iter()
            .zip(corpus.test.observed.iter())
            .enumerate()
        {
            let residual = residual_from_pair(
                format!("test-{condition}-{index}"),
                &corpus.scope.state_space,
                corpus.scope.prediction_horizon_steps,
                predicted,
                observed,
            );
            mse_sum += residual.mean_squared_error;
            let assessment = profile
                .assess(&scope, &residual)
                .expect("held-out residual must match calibration scope");
            if assessment.exceeded {
                flagged += 1;
            }
        }

        let flagged_rate = flagged as f64 / predictions.len() as f64;
        let mean_mse = mse_sum / predictions.len() as f64;
        let expected_flagged_rate = corpus.python_reference.flagged_rate_by_condition
            [condition];
        let expected_mean_mse = corpus.python_reference.mean_mse_by_condition
            [condition];

        assert!(
            (flagged_rate - expected_flagged_rate).abs() < 1e-15,
            "condition {condition}: Rust flagged rate {flagged_rate} differs from Python reference {expected_flagged_rate}"
        );
        assert!(
            (mean_mse - expected_mean_mse).abs() < 5e-13,
            "condition {condition}: Rust mean MSE {mean_mse} differs from Python reference {expected_mean_mse}"
        );
    }

    assert_eq!(
        corpus.python_reference.flagged_rate_by_condition["true_action"],
        0.03125
    );
    for control in [
        "persistence",
        "zero_action",
        "shuffled_action",
        "corrupted_action",
    ] {
        assert_eq!(
            corpus.python_reference.flagged_rate_by_condition[control],
            1.0
        );
    }
}
