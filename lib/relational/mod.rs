//! Shadow-only bridge from longitudinal user predictions to Starfire CHARGE.
//!
//! The bridge accepts a prediction issued before an interaction and a later,
//! independently sourced outcome witness. It validates both records, computes the
//! residual itself, and may emit an unissued `PredictionResidual` charge.
//!
//! It is feature-gated and has no connection to `Runtime::chat()`, live routing,
//! user-belief mutation, ontology promotion, or autonomous action.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::charge::{Charge, ChargeKind, ChargeScope};

pub const RELATIONAL_EVIDENCE_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredictionHorizon {
    CurrentTurn,
    NextTurn,
    Session,
    NextSession,
    Longitudinal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictedOutcome {
    pub label: String,
    pub probability: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceProducer {
    pub name: String,
    pub version: String,
    pub state_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationalPrediction {
    pub schema_version: u16,
    pub prediction_id: String,
    /// Opaque identifier; a human-readable user name is unnecessary.
    pub subject_id: String,
    pub target: String,
    pub context_scope: String,
    pub issued_at_sequence: u64,
    pub horizon: PredictionHorizon,
    pub outcomes: Vec<PredictedOutcome>,
    pub producer: EvidenceProducer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeSource {
    ExplicitUserCorrection,
    ExplicitUserConfirmation,
    SubsequentUserBehavior,
    TaskMetric,
    ExternalEvaluator,
    /// Present in the wire schema so self-judgment is rejected explicitly.
    GeneratorSelfReport,
}

impl OutcomeSource {
    #[must_use]
    pub fn is_independent(self) -> bool {
        !matches!(self, Self::GeneratorSelfReport)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationalOutcomeWitness {
    pub schema_version: u16,
    pub prediction_id: String,
    pub observed_at_sequence: u64,
    pub observed_label: String,
    pub source: OutcomeSource,
    /// Confidence in the witness, never confidence claimed by the predictor.
    pub confidence: f64,
    pub evidence_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RelationalBridgeConfig {
    /// Minimum independently weighted RMS residual required to emit CHARGE.
    pub min_charge_magnitude: f64,
    pub probability_sum_tolerance: f64,
    pub require_future_observation: bool,
}

impl Default for RelationalBridgeConfig {
    fn default() -> Self {
        Self {
            min_charge_magnitude: 0.15,
            probability_sum_tolerance: 1e-6,
            require_future_observation: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationalResidual {
    pub prediction_id: String,
    pub target: String,
    pub context_scope: String,
    pub labels: Vec<String>,
    /// One-hot observation minus the assigned probability for each label.
    pub residual: Vec<f64>,
    /// Multiclass Brier score: sum of squared residual coordinates.
    pub brier_score: f64,
    pub observed_probability: f64,
    /// RMS residual weighted by independent witness confidence.
    pub magnitude: f64,
    pub witness_confidence: f64,
    pub evidence_id: String,
}

impl RelationalResidual {
    #[must_use]
    pub fn to_shadow_charge(&self) -> Charge {
        Charge::new(
            ChargeKind::PredictionResidual,
            self.residual.iter().map(|value| *value as f32).collect(),
            self.magnitude as f32,
            ChargeScope::Custom(format!("relational:{}:{}", self.target, self.context_scope)),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeStatus {
    BelowThreshold,
    EmittedShadowCharge,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BridgeAssessment {
    pub status: BridgeStatus,
    pub residual: RelationalResidual,
    /// Always unissued (`id == 0`); this module never inserts into a ledger.
    pub charge: Option<Charge>,
    /// The bridge cannot authorize promotion or action by itself.
    pub promotion_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationalReplayCase {
    pub prediction: RelationalPrediction,
    pub witness: RelationalOutcomeWitness,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationalReplayReport {
    pub case_count: usize,
    pub emitted_charge_count: usize,
    pub mean_brier_score: f64,
    pub mean_magnitude: f64,
    pub max_magnitude: f64,
    pub assessments: Vec<BridgeAssessment>,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum RelationalBridgeError {
    #[error("unsupported relational evidence schema version {0}")]
    UnsupportedSchemaVersion(u16),
    #[error("{field} cannot be blank")]
    BlankField { field: &'static str },
    #[error("prediction must contain at least two candidate outcomes")]
    TooFewOutcomes,
    #[error("duplicate predicted outcome label: {0}")]
    DuplicateOutcome(String),
    #[error("invalid probability for outcome {label}: {probability}")]
    InvalidProbability { label: String, probability: f64 },
    #[error("predicted probabilities sum to {sum}, outside tolerance {tolerance}")]
    ProbabilitySum { sum: f64, tolerance: f64 },
    #[error("invalid witness confidence {0}")]
    InvalidWitnessConfidence(f64),
    #[error("outcome witness references {actual}, expected {expected}")]
    PredictionIdMismatch { expected: String, actual: String },
    #[error("observed label is absent from prediction distribution: {0}")]
    UnknownObservedLabel(String),
    #[error("outcome must be observed after prediction issuance")]
    NonFutureOutcome,
    #[error("the response generator cannot serve as its own outcome witness")]
    NonIndependentWitness,
    #[error("invalid bridge configuration: {0}")]
    InvalidConfig(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub struct RelationalResidualBridge {
    config: RelationalBridgeConfig,
}

impl RelationalResidualBridge {
    pub fn new(config: RelationalBridgeConfig) -> Result<Self, RelationalBridgeError> {
        validate_config(config)?;
        Ok(Self { config })
    }

    #[must_use]
    pub fn config(&self) -> RelationalBridgeConfig {
        self.config
    }

    pub fn assess(
        &self,
        prediction: &RelationalPrediction,
        witness: &RelationalOutcomeWitness,
    ) -> Result<BridgeAssessment, RelationalBridgeError> {
        validate_prediction(prediction, self.config.probability_sum_tolerance)?;
        validate_witness(prediction, witness, self.config.require_future_observation)?;

        let mut labels = Vec::with_capacity(prediction.outcomes.len());
        let mut residual = Vec::with_capacity(prediction.outcomes.len());
        let mut observed_probability = None;

        for outcome in &prediction.outcomes {
            labels.push(outcome.label.clone());
            let observed = if outcome.label == witness.observed_label {
                1.0
            } else {
                0.0
            };
            residual.push(observed - outcome.probability);
            if outcome.label == witness.observed_label {
                observed_probability = Some(outcome.probability);
            }
        }

        let brier_score = residual.iter().map(|value| value * value).sum::<f64>();
        let rms = (brier_score / residual.len() as f64).sqrt();
        let magnitude = (rms * witness.confidence).clamp(0.0, 1.0);
        let relational_residual = RelationalResidual {
            prediction_id: prediction.prediction_id.clone(),
            target: prediction.target.clone(),
            context_scope: prediction.context_scope.clone(),
            labels,
            residual,
            brier_score,
            observed_probability: observed_probability.expect("validated observed label"),
            magnitude,
            witness_confidence: witness.confidence,
            evidence_id: witness.evidence_id.clone(),
        };
        let charge = (magnitude >= self.config.min_charge_magnitude)
            .then(|| relational_residual.to_shadow_charge());

        Ok(BridgeAssessment {
            status: if charge.is_some() {
                BridgeStatus::EmittedShadowCharge
            } else {
                BridgeStatus::BelowThreshold
            },
            residual: relational_residual,
            charge,
            promotion_eligible: false,
        })
    }

    pub fn replay(
        &self,
        cases: &[RelationalReplayCase],
    ) -> Result<RelationalReplayReport, RelationalBridgeError> {
        let assessments = cases
            .iter()
            .map(|case| self.assess(&case.prediction, &case.witness))
            .collect::<Result<Vec<_>, _>>()?;
        let emitted_charge_count = assessments
            .iter()
            .filter(|assessment| assessment.charge.is_some())
            .count();
        let divisor = assessments.len().max(1) as f64;
        let mean_brier_score = assessments
            .iter()
            .map(|assessment| assessment.residual.brier_score)
            .sum::<f64>()
            / divisor;
        let mean_magnitude = assessments
            .iter()
            .map(|assessment| assessment.residual.magnitude)
            .sum::<f64>()
            / divisor;
        let max_magnitude = assessments
            .iter()
            .map(|assessment| assessment.residual.magnitude)
            .fold(0.0, f64::max);

        Ok(RelationalReplayReport {
            case_count: assessments.len(),
            emitted_charge_count,
            mean_brier_score,
            mean_magnitude,
            max_magnitude,
            assessments,
        })
    }
}

impl Default for RelationalResidualBridge {
    fn default() -> Self {
        Self::new(RelationalBridgeConfig::default())
            .expect("default relational bridge configuration is valid")
    }
}

fn validate_config(config: RelationalBridgeConfig) -> Result<(), RelationalBridgeError> {
    if !config.min_charge_magnitude.is_finite()
        || !(0.0..=1.0).contains(&config.min_charge_magnitude)
    {
        return Err(RelationalBridgeError::InvalidConfig(
            "min_charge_magnitude must be finite and within 0..=1",
        ));
    }
    if !config.probability_sum_tolerance.is_finite() || config.probability_sum_tolerance < 0.0 {
        return Err(RelationalBridgeError::InvalidConfig(
            "probability_sum_tolerance must be finite and non-negative",
        ));
    }
    Ok(())
}

fn validate_prediction(
    prediction: &RelationalPrediction,
    probability_sum_tolerance: f64,
) -> Result<(), RelationalBridgeError> {
    if prediction.schema_version != RELATIONAL_EVIDENCE_SCHEMA_VERSION {
        return Err(RelationalBridgeError::UnsupportedSchemaVersion(
            prediction.schema_version,
        ));
    }
    require_non_blank("prediction_id", &prediction.prediction_id)?;
    require_non_blank("subject_id", &prediction.subject_id)?;
    require_non_blank("target", &prediction.target)?;
    require_non_blank("context_scope", &prediction.context_scope)?;
    require_non_blank("producer.name", &prediction.producer.name)?;
    require_non_blank("producer.version", &prediction.producer.version)?;
    if prediction.outcomes.len() < 2 {
        return Err(RelationalBridgeError::TooFewOutcomes);
    }

    let mut labels = BTreeSet::new();
    let mut sum = 0.0;
    for outcome in &prediction.outcomes {
        require_non_blank("outcome.label", &outcome.label)?;
        if !labels.insert(outcome.label.clone()) {
            return Err(RelationalBridgeError::DuplicateOutcome(
                outcome.label.clone(),
            ));
        }
        if !outcome.probability.is_finite() || !(0.0..=1.0).contains(&outcome.probability) {
            return Err(RelationalBridgeError::InvalidProbability {
                label: outcome.label.clone(),
                probability: outcome.probability,
            });
        }
        sum += outcome.probability;
    }
    if (sum - 1.0).abs() > probability_sum_tolerance {
        return Err(RelationalBridgeError::ProbabilitySum {
            sum,
            tolerance: probability_sum_tolerance,
        });
    }
    Ok(())
}

fn validate_witness(
    prediction: &RelationalPrediction,
    witness: &RelationalOutcomeWitness,
    require_future_observation: bool,
) -> Result<(), RelationalBridgeError> {
    if witness.schema_version != RELATIONAL_EVIDENCE_SCHEMA_VERSION {
        return Err(RelationalBridgeError::UnsupportedSchemaVersion(
            witness.schema_version,
        ));
    }
    require_non_blank("witness.prediction_id", &witness.prediction_id)?;
    require_non_blank("witness.observed_label", &witness.observed_label)?;
    require_non_blank("witness.evidence_id", &witness.evidence_id)?;
    if prediction.prediction_id != witness.prediction_id {
        return Err(RelationalBridgeError::PredictionIdMismatch {
            expected: prediction.prediction_id.clone(),
            actual: witness.prediction_id.clone(),
        });
    }
    if !witness.confidence.is_finite() || !(0.0..=1.0).contains(&witness.confidence) {
        return Err(RelationalBridgeError::InvalidWitnessConfidence(
            witness.confidence,
        ));
    }
    if !witness.source.is_independent() {
        return Err(RelationalBridgeError::NonIndependentWitness);
    }
    if require_future_observation && witness.observed_at_sequence <= prediction.issued_at_sequence {
        return Err(RelationalBridgeError::NonFutureOutcome);
    }
    if !prediction
        .outcomes
        .iter()
        .any(|outcome| outcome.label == witness.observed_label)
    {
        return Err(RelationalBridgeError::UnknownObservedLabel(
            witness.observed_label.clone(),
        ));
    }
    Ok(())
}

fn require_non_blank(field: &'static str, value: &str) -> Result<(), RelationalBridgeError> {
    if value.trim().is_empty() {
        return Err(RelationalBridgeError::BlankField { field });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prediction(primary_probability: f64) -> RelationalPrediction {
        RelationalPrediction {
            schema_version: RELATIONAL_EVIDENCE_SCHEMA_VERSION,
            prediction_id: "prediction-1".into(),
            subject_id: "subject-opaque-1".into(),
            target: "response_policy".into(),
            context_scope: "project:implementation_authorized".into(),
            issued_at_sequence: 10,
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
                version: "phase-3".into(),
                state_hash: Some("state-10".into()),
            },
        }
    }

    fn witness(label: &str) -> RelationalOutcomeWitness {
        RelationalOutcomeWitness {
            schema_version: RELATIONAL_EVIDENCE_SCHEMA_VERSION,
            prediction_id: "prediction-1".into(),
            observed_at_sequence: 11,
            observed_label: label.into(),
            source: OutcomeSource::ExplicitUserCorrection,
            confidence: 1.0,
            evidence_id: "turn-11".into(),
        }
    }

    #[test]
    fn confident_wrong_prediction_emits_unissued_shadow_charge() {
        let assessment = RelationalResidualBridge::default()
            .assess(&prediction(0.9), &witness("explain_first"))
            .unwrap();

        assert_eq!(assessment.status, BridgeStatus::EmittedShadowCharge);
        assert!(!assessment.promotion_eligible);
        assert!((assessment.residual.observed_probability - 0.1).abs() < 1e-12);
        assert!((assessment.residual.magnitude - 0.9).abs() < 1e-12);
        let charge = assessment.charge.unwrap();
        assert_eq!(charge.id, 0);
        assert_eq!(charge.kind, ChargeKind::PredictionResidual);
        assert_eq!(
            charge.scope,
            ChargeScope::Custom(
                "relational:response_policy:project:implementation_authorized".into()
            )
        );
    }

    #[test]
    fn correct_high_confidence_prediction_stays_below_threshold() {
        let assessment = RelationalResidualBridge::default()
            .assess(&prediction(0.9), &witness("direct_implementation"))
            .unwrap();

        assert_eq!(assessment.status, BridgeStatus::BelowThreshold);
        assert!(assessment.charge.is_none());
        assert!((assessment.residual.magnitude - 0.1).abs() < 1e-12);
    }

    #[test]
    fn response_generator_cannot_grade_itself() {
        let mut self_report = witness("explain_first");
        self_report.source = OutcomeSource::GeneratorSelfReport;
        assert_eq!(
            RelationalResidualBridge::default()
                .assess(&prediction(0.9), &self_report)
                .unwrap_err(),
            RelationalBridgeError::NonIndependentWitness
        );
    }

    #[test]
    fn witness_must_follow_prediction() {
        let mut early = witness("explain_first");
        early.observed_at_sequence = 10;
        assert_eq!(
            RelationalResidualBridge::default()
                .assess(&prediction(0.9), &early)
                .unwrap_err(),
            RelationalBridgeError::NonFutureOutcome
        );
    }

    #[test]
    fn malformed_probability_distribution_is_rejected() {
        let mut malformed = prediction(0.9);
        malformed.outcomes[1].probability = 0.9;
        assert!(matches!(
            RelationalResidualBridge::default().assess(&malformed, &witness("explain_first")),
            Err(RelationalBridgeError::ProbabilitySum { .. })
        ));
    }

    #[test]
    fn replay_is_deterministic_and_never_promotes() {
        let bridge = RelationalResidualBridge::default();
        let cases = vec![
            RelationalReplayCase {
                prediction: prediction(0.9),
                witness: witness("explain_first"),
            },
            RelationalReplayCase {
                prediction: prediction(0.9),
                witness: witness("direct_implementation"),
            },
        ];
        let first = bridge.replay(&cases).unwrap();
        let second = bridge.replay(&cases).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.case_count, 2);
        assert_eq!(first.emitted_charge_count, 1);
        assert!(first
            .assessments
            .iter()
            .all(|assessment| !assessment.promotion_eligible));
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );
    }
}
