//! Independent numeric residual computation for developmental predictions.
//!
//! A predictor may emit a candidate next state, but it never judges the quality
//! of that prediction. This module compares a prior prediction with a separately
//! observed state and computes the residual directly from the two vectors.
//!
//! No CHARGE is created here. Residual-to-pressure calibration is a later,
//! separately gated experiment.

use super::evidence::{NumericStateObservation, NumericTransitionPrediction};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct NumericPredictionResidual {
    pub transition_id: String,
    pub state_space: String,
    pub dimension: usize,
    /// observed - predicted for each coordinate.
    pub signed_error: Vec<f32>,
    pub mean_absolute_error: f64,
    pub mean_squared_error: f64,
    pub root_mean_squared_error: f64,
    pub max_absolute_error: f64,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ResidualError {
    #[error(
        "transition id mismatch: prediction {prediction:?}, observation {observation:?}"
    )]
    TransitionIdMismatch {
        prediction: String,
        observation: String,
    },
    #[error(
        "state space mismatch: prediction {prediction:?}, observation {observation:?}"
    )]
    StateSpaceMismatch {
        prediction: String,
        observation: String,
    },
    #[error(
        "state dimension mismatch: prediction {prediction}, observation {observation}"
    )]
    DimensionMismatch {
        prediction: usize,
        observation: usize,
    },
    #[error("numeric transition state vectors cannot be empty")]
    EmptyState,
    #[error("numeric transition state vectors must contain only finite values")]
    NonFiniteState,
}

pub fn compare_numeric_transition(
    prediction: &NumericTransitionPrediction,
    observation: &NumericStateObservation,
) -> Result<NumericPredictionResidual, ResidualError> {
    if prediction.transition_id != observation.transition_id {
        return Err(ResidualError::TransitionIdMismatch {
            prediction: prediction.transition_id.clone(),
            observation: observation.transition_id.clone(),
        });
    }

    if prediction.predicted_next_state.space != observation.state.space {
        return Err(ResidualError::StateSpaceMismatch {
            prediction: prediction.predicted_next_state.space.clone(),
            observation: observation.state.space.clone(),
        });
    }

    let predicted = &prediction.predicted_next_state.values;
    let observed = &observation.state.values;

    if predicted.is_empty() || observed.is_empty() {
        return Err(ResidualError::EmptyState);
    }

    if predicted.len() != observed.len() {
        return Err(ResidualError::DimensionMismatch {
            prediction: predicted.len(),
            observation: observed.len(),
        });
    }

    if predicted
        .iter()
        .chain(observed.iter())
        .any(|value| !value.is_finite())
    {
        return Err(ResidualError::NonFiniteState);
    }

    let signed_error: Vec<f32> = observed
        .iter()
        .zip(predicted.iter())
        .map(|(observed_value, predicted_value)| observed_value - predicted_value)
        .collect();

    let dimension = signed_error.len();
    let mut absolute_sum = 0.0_f64;
    let mut squared_sum = 0.0_f64;
    let mut max_absolute_error = 0.0_f64;

    for error in &signed_error {
        let error = f64::from(*error);
        let absolute = error.abs();
        absolute_sum += absolute;
        squared_sum += error * error;
        max_absolute_error = max_absolute_error.max(absolute);
    }

    let mean_absolute_error = absolute_sum / dimension as f64;
    let mean_squared_error = squared_sum / dimension as f64;
    let root_mean_squared_error = mean_squared_error.sqrt();

    Ok(NumericPredictionResidual {
        transition_id: prediction.transition_id.clone(),
        state_space: prediction.predicted_next_state.space.clone(),
        dimension,
        signed_error,
        mean_absolute_error,
        mean_squared_error,
        root_mean_squared_error,
        max_absolute_error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::developmental::evidence::NamedVector;

    fn prediction(values: Vec<f32>) -> NumericTransitionPrediction {
        NumericTransitionPrediction {
            transition_id: "transition-1".to_string(),
            action: NamedVector {
                space: "synthetic.action.v1".to_string(),
                values: vec![0.1, -0.2, 0.3],
            },
            predicted_next_state: NamedVector {
                space: "synthetic.proprio.v1".to_string(),
                values,
            },
            horizon_steps: 1,
        }
    }

    fn observation(values: Vec<f32>) -> NumericStateObservation {
        NumericStateObservation {
            transition_id: "transition-1".to_string(),
            state: NamedVector {
                space: "synthetic.proprio.v1".to_string(),
                values,
            },
        }
    }

    #[test]
    fn exact_prediction_has_zero_residual() {
        let residual = compare_numeric_transition(
            &prediction(vec![0.25, 0.5, -0.75]),
            &observation(vec![0.25, 0.5, -0.75]),
        )
        .unwrap();

        assert_eq!(residual.signed_error, vec![0.0, 0.0, 0.0]);
        assert_eq!(residual.mean_absolute_error, 0.0);
        assert_eq!(residual.mean_squared_error, 0.0);
        assert_eq!(residual.root_mean_squared_error, 0.0);
        assert_eq!(residual.max_absolute_error, 0.0);
    }

    #[test]
    fn residual_is_computed_from_observation_minus_prediction() {
        let residual = compare_numeric_transition(
            &prediction(vec![1.0, 2.0, 3.0]),
            &observation(vec![2.0, 0.0, 4.0]),
        )
        .unwrap();

        assert_eq!(residual.signed_error, vec![1.0, -2.0, 1.0]);
        assert!((residual.mean_absolute_error - (4.0 / 3.0)).abs() < 1e-12);
        assert!((residual.mean_squared_error - 2.0).abs() < 1e-12);
        assert!((residual.root_mean_squared_error - 2.0_f64.sqrt()).abs() < 1e-12);
        assert_eq!(residual.max_absolute_error, 2.0);
    }

    #[test]
    fn transition_mismatch_is_rejected() {
        let mut observed = observation(vec![0.0, 0.0, 0.0]);
        observed.transition_id = "transition-2".to_string();

        assert!(matches!(
            compare_numeric_transition(
                &prediction(vec![0.0, 0.0, 0.0]),
                &observed
            ),
            Err(ResidualError::TransitionIdMismatch { .. })
        ));
    }

    #[test]
    fn state_space_mismatch_is_rejected() {
        let mut observed = observation(vec![0.0, 0.0, 0.0]);
        observed.state.space = "other-space".to_string();

        assert!(matches!(
            compare_numeric_transition(
                &prediction(vec![0.0, 0.0, 0.0]),
                &observed
            ),
            Err(ResidualError::StateSpaceMismatch { .. })
        ));
    }

    #[test]
    fn dimension_mismatch_is_rejected() {
        assert!(matches!(
            compare_numeric_transition(
                &prediction(vec![0.0, 0.0]),
                &observation(vec![0.0, 0.0, 0.0])
            ),
            Err(ResidualError::DimensionMismatch { .. })
        ));
    }
}
