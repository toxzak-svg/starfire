//! Scoped calibration of independently computed numeric prediction residuals.
//!
//! This module intentionally stops before pressure or CHARGE creation. It turns a
//! calibration set of residuals into a scoped threshold and can assess later
//! residuals against that threshold. The threshold is invalid outside its exact
//! declared scope.

use super::residual::NumericPredictionResidual;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const RESIDUAL_CALIBRATION_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidualCalibrationScope {
    /// Stable identifier for the exact predictor/checkpoint/protocol scope.
    pub predictor_scope: String,
    /// Environment or task-family scope in which calibration was measured.
    pub environment_scope: String,
    /// Semantic coordinate space of the predicted and observed state vectors.
    pub state_space: String,
    /// Prediction horizon represented by this calibration profile.
    pub horizon_steps: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResidualMetric {
    MeanSquaredError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantileMethod {
    Higher,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResidualCalibrationProfile {
    pub schema_version: u16,
    pub scope: ResidualCalibrationScope,
    pub metric: ResidualMetric,
    pub quantile: f64,
    pub method: QuantileMethod,
    pub calibration_count: usize,
    pub threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResidualAssessment {
    pub residual_mse: f64,
    pub threshold: f64,
    pub exceeded: bool,
    /// Ratio to the calibrated threshold when the threshold is positive.
    pub ratio_to_threshold: Option<f64>,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum CalibrationError {
    #[error("calibration residuals cannot be empty")]
    EmptyCalibration,
    #[error("calibration quantile must be finite and in (0, 1], got {0}")]
    InvalidQuantile(f64),
    #[error("required calibration scope field is empty: {0}")]
    EmptyScopeField(&'static str),
    #[error("calibration horizon_steps must be greater than zero")]
    ZeroHorizon,
    #[error(
        "residual state space mismatch: expected {expected:?}, got {actual:?}"
    )]
    StateSpaceMismatch { expected: String, actual: String },
    #[error("residual mean squared error must be finite and non-negative, got {0}")]
    InvalidResidual(f64),
    #[error("calibration scope mismatch")]
    ScopeMismatch,
}

impl ResidualCalibrationProfile {
    /// Fit a `higher` quantile threshold from independently computed residuals.
    ///
    /// This matches NumPy's `quantile(..., method="higher")`: sort the observed
    /// metric values and select `ceil((n - 1) * q)`.
    pub fn fit_higher_quantile(
        scope: ResidualCalibrationScope,
        residuals: &[NumericPredictionResidual],
        quantile: f64,
    ) -> Result<Self, CalibrationError> {
        validate_scope(&scope)?;
        if residuals.is_empty() {
            return Err(CalibrationError::EmptyCalibration);
        }
        if !quantile.is_finite() || quantile <= 0.0 || quantile > 1.0 {
            return Err(CalibrationError::InvalidQuantile(quantile));
        }

        let mut values = Vec::with_capacity(residuals.len());
        for residual in residuals {
            validate_residual_for_scope(&scope, residual)?;
            values.push(residual.mean_squared_error);
        }
        values.sort_by(|left, right| {
            left.partial_cmp(right)
                .expect("validated finite residuals must be comparable")
        });

        let position = (values.len().saturating_sub(1) as f64) * quantile;
        let index = position.ceil() as usize;
        let threshold = values[index.min(values.len() - 1)];

        Ok(Self {
            schema_version: RESIDUAL_CALIBRATION_SCHEMA_VERSION,
            scope,
            metric: ResidualMetric::MeanSquaredError,
            quantile,
            method: QuantileMethod::Higher,
            calibration_count: values.len(),
            threshold,
        })
    }

    pub fn assess(
        &self,
        scope: &ResidualCalibrationScope,
        residual: &NumericPredictionResidual,
    ) -> Result<ResidualAssessment, CalibrationError> {
        if scope != &self.scope {
            return Err(CalibrationError::ScopeMismatch);
        }
        validate_residual_for_scope(scope, residual)?;

        let residual_mse = residual.mean_squared_error;
        let exceeded = residual_mse > self.threshold;
        let ratio_to_threshold = if self.threshold > 0.0 {
            Some(residual_mse / self.threshold)
        } else {
            None
        };

        Ok(ResidualAssessment {
            residual_mse,
            threshold: self.threshold,
            exceeded,
            ratio_to_threshold,
        })
    }
}

fn validate_scope(scope: &ResidualCalibrationScope) -> Result<(), CalibrationError> {
    if scope.predictor_scope.trim().is_empty() {
        return Err(CalibrationError::EmptyScopeField("predictor_scope"));
    }
    if scope.environment_scope.trim().is_empty() {
        return Err(CalibrationError::EmptyScopeField("environment_scope"));
    }
    if scope.state_space.trim().is_empty() {
        return Err(CalibrationError::EmptyScopeField("state_space"));
    }
    if scope.horizon_steps == 0 {
        return Err(CalibrationError::ZeroHorizon);
    }
    Ok(())
}

fn validate_residual_for_scope(
    scope: &ResidualCalibrationScope,
    residual: &NumericPredictionResidual,
) -> Result<(), CalibrationError> {
    if residual.state_space != scope.state_space {
        return Err(CalibrationError::StateSpaceMismatch {
            expected: scope.state_space.clone(),
            actual: residual.state_space.clone(),
        });
    }
    if !residual.mean_squared_error.is_finite()
        || residual.mean_squared_error < 0.0
    {
        return Err(CalibrationError::InvalidResidual(
            residual.mean_squared_error,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> ResidualCalibrationScope {
        ResidualCalibrationScope {
            predictor_scope: "predictor-v1".to_string(),
            environment_scope: "environment-v1".to_string(),
            state_space: "state-space-v1".to_string(),
            horizon_steps: 1,
        }
    }

    fn residual(mse: f64) -> NumericPredictionResidual {
        NumericPredictionResidual {
            transition_id: format!("transition-{mse}"),
            state_space: "state-space-v1".to_string(),
            dimension: 1,
            signed_error: vec![0.0],
            mean_absolute_error: mse.sqrt(),
            mean_squared_error: mse,
            root_mean_squared_error: mse.sqrt(),
            max_absolute_error: mse.sqrt(),
        }
    }

    #[test]
    fn higher_quantile_matches_declared_index_rule() {
        let residuals = vec![
            residual(1.0),
            residual(2.0),
            residual(3.0),
            residual(4.0),
            residual(5.0),
        ];
        let profile = ResidualCalibrationProfile::fit_higher_quantile(
            scope(),
            &residuals,
            0.75,
        )
        .unwrap();

        // ceil((5 - 1) * 0.75) = 3 -> sorted value 4.0
        assert_eq!(profile.threshold, 4.0);
        assert_eq!(profile.calibration_count, 5);
    }

    #[test]
    fn assessment_uses_strictly_greater_than_threshold() {
        let profile = ResidualCalibrationProfile::fit_higher_quantile(
            scope(),
            &[residual(1.0), residual(2.0)],
            1.0,
        )
        .unwrap();

        assert!(!profile.assess(&scope(), &residual(2.0)).unwrap().exceeded);
        assert!(profile.assess(&scope(), &residual(2.1)).unwrap().exceeded);
    }

    #[test]
    fn scope_mismatch_is_rejected() {
        let profile = ResidualCalibrationProfile::fit_higher_quantile(
            scope(),
            &[residual(1.0)],
            0.99,
        )
        .unwrap();
        let mut other_scope = scope();
        other_scope.predictor_scope = "different-predictor".to_string();

        assert_eq!(
            profile.assess(&other_scope, &residual(2.0)),
            Err(CalibrationError::ScopeMismatch)
        );
    }

    #[test]
    fn state_space_mismatch_is_rejected_during_fit() {
        let mut wrong = residual(1.0);
        wrong.state_space = "other-space".to_string();

        assert!(matches!(
            ResidualCalibrationProfile::fit_higher_quantile(
                scope(),
                &[wrong],
                0.99
            ),
            Err(CalibrationError::StateSpaceMismatch { .. })
        ));
    }
}
