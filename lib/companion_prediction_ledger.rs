//! Falsifiable companion prediction ledger.
//!
//! Predictions are unresolved commitments issued before the outcome window.
//! They may be resolved only by later, independent evidence. The response
//! generator cannot witness its own predictions, and no ledger result has
//! response-routing, belief-promotion, or action authority.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub type PredictionId = u64;
pub type AbstentionId = u64;

const TOTAL_PROBABILITY_BPS: u32 = 10_000;
const BRIER_DENOMINATOR: u64 = 100_000_000;
const CALIBRATION_BUCKET_WIDTH_BPS: u16 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionProducerKind {
    CompanionPolicy,
    Imported,
    ResponseGenerator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionProducer {
    pub id: String,
    pub kind: PredictionProducerKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeProbability {
    pub label: String,
    pub probability_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionInput {
    pub subject_scope: String,
    pub producer: PredictionProducer,
    pub outcomes: Vec<OutcomeProbability>,
    pub issued_at_ms: u64,
    pub not_before_ms: u64,
    pub expires_at_ms: u64,
    pub context_digest: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WitnessSource {
    UserObservation,
    Environment,
    ExternalEvaluator,
    ResponseGenerator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeWitness {
    pub source: WitnessSource,
    pub label: String,
    pub observed_at_ms: u64,
    pub evidence_digest: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrierScore {
    pub sum_squared_basis_points: u64,
    pub score_ppm: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionStatus {
    Pending,
    Resolved {
        witness: OutcomeWitness,
        score: BrierScore,
    },
    Expired {
        expired_at_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionRecord {
    pub id: PredictionId,
    pub subject_scope: String,
    pub producer: PredictionProducer,
    pub outcomes: Vec<OutcomeProbability>,
    pub issued_at_ms: u64,
    pub not_before_ms: u64,
    pub expires_at_ms: u64,
    pub context_digest: u64,
    pub status: PredictionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbstentionInput {
    pub subject_scope: String,
    pub producer: PredictionProducer,
    pub reason: String,
    pub occurred_at_ms: u64,
    pub context_digest: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbstentionRecord {
    pub id: AbstentionId,
    pub subject_scope: String,
    pub producer: PredictionProducer,
    pub reason: String,
    pub occurred_at_ms: u64,
    pub context_digest: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionEvent {
    Issued {
        prediction: PredictionRecord,
    },
    Resolved {
        prediction_id: PredictionId,
        witness: OutcomeWitness,
        score: BrierScore,
    },
    Expired {
        prediction_id: PredictionId,
        expired_at_ms: u64,
    },
    Abstained {
        abstention: AbstentionRecord,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionTransition {
    pub version: u64,
    pub prediction_id: Option<PredictionId>,
    pub abstention_id: Option<AbstentionId>,
    pub event: PredictionEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionLedgerSummary {
    pub issued: u64,
    pub pending: u64,
    pub resolved: u64,
    pub expired: u64,
    pub abstentions: u64,
    pub total_brier_score_ppm: u64,
    pub mean_brier_score_ppm: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalibrationBucket {
    pub lower_confidence_bps: u16,
    pub upper_confidence_bps: u16,
    pub count: u64,
    pub mean_confidence_bps: u16,
    pub observed_accuracy_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionLedger {
    pub version: u64,
    predictions: BTreeMap<PredictionId, PredictionRecord>,
    abstentions: BTreeMap<AbstentionId, AbstentionRecord>,
    next_prediction_id: PredictionId,
    next_abstention_id: AbstentionId,
}

impl Default for PredictionLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl PredictionLedger {
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: 0,
            predictions: BTreeMap::new(),
            abstentions: BTreeMap::new(),
            next_prediction_id: 1,
            next_abstention_id: 1,
        }
    }

    #[must_use]
    pub fn predictions(&self) -> &BTreeMap<PredictionId, PredictionRecord> {
        &self.predictions
    }

    #[must_use]
    pub fn abstentions(&self) -> &BTreeMap<AbstentionId, AbstentionRecord> {
        &self.abstentions
    }

    #[must_use]
    pub fn prediction(&self, prediction_id: PredictionId) -> Option<&PredictionRecord> {
        self.predictions.get(&prediction_id)
    }

    pub fn issue(
        &mut self,
        expected_version: u64,
        input: PredictionInput,
    ) -> Result<PredictionTransition, PredictionLedgerError> {
        self.require_version(expected_version)?;
        let prediction = prepare_prediction(self.next_prediction_id, input)?;
        let prediction_id = prediction.id;
        let event = PredictionEvent::Issued { prediction };
        let version = self.apply_event(expected_version, event.clone())?;
        Ok(PredictionTransition {
            version,
            prediction_id: Some(prediction_id),
            abstention_id: None,
            event,
        })
    }

    pub fn abstain(
        &mut self,
        expected_version: u64,
        input: AbstentionInput,
    ) -> Result<PredictionTransition, PredictionLedgerError> {
        self.require_version(expected_version)?;
        let abstention = prepare_abstention(self.next_abstention_id, input)?;
        let abstention_id = abstention.id;
        let event = PredictionEvent::Abstained { abstention };
        let version = self.apply_event(expected_version, event.clone())?;
        Ok(PredictionTransition {
            version,
            prediction_id: None,
            abstention_id: Some(abstention_id),
            event,
        })
    }

    pub fn resolve(
        &mut self,
        expected_version: u64,
        prediction_id: PredictionId,
        witness: OutcomeWitness,
    ) -> Result<PredictionTransition, PredictionLedgerError> {
        self.require_version(expected_version)?;
        let prediction = self
            .predictions
            .get(&prediction_id)
            .ok_or(PredictionLedgerError::UnknownPrediction(prediction_id))?;
        require_pending(prediction)?;
        let witness = prepare_witness(prediction, witness)?;
        let score = score_outcome(&prediction.outcomes, &witness.label)?;
        let event = PredictionEvent::Resolved {
            prediction_id,
            witness,
            score,
        };
        let version = self.apply_event(expected_version, event.clone())?;
        Ok(PredictionTransition {
            version,
            prediction_id: Some(prediction_id),
            abstention_id: None,
            event,
        })
    }

    pub fn expire(
        &mut self,
        expected_version: u64,
        prediction_id: PredictionId,
        expired_at_ms: u64,
    ) -> Result<PredictionTransition, PredictionLedgerError> {
        self.require_version(expected_version)?;
        let prediction = self
            .predictions
            .get(&prediction_id)
            .ok_or(PredictionLedgerError::UnknownPrediction(prediction_id))?;
        require_pending(prediction)?;
        if expired_at_ms < prediction.expires_at_ms {
            return Err(PredictionLedgerError::NotExpiredYet {
                prediction_id,
                expires_at_ms: prediction.expires_at_ms,
                attempted_at_ms: expired_at_ms,
            });
        }
        let event = PredictionEvent::Expired {
            prediction_id,
            expired_at_ms,
        };
        let version = self.apply_event(expected_version, event.clone())?;
        Ok(PredictionTransition {
            version,
            prediction_id: Some(prediction_id),
            abstention_id: None,
            event,
        })
    }

    pub fn replay(events: &[PredictionEvent]) -> Result<Self, PredictionLedgerError> {
        let mut ledger = Self::new();
        for event in events {
            let expected_version = ledger.version;
            ledger.apply_event(expected_version, event.clone())?;
        }
        Ok(ledger)
    }

    #[must_use]
    pub fn summary(&self) -> PredictionLedgerSummary {
        let mut pending = 0_u64;
        let mut resolved = 0_u64;
        let mut expired = 0_u64;
        let mut total_brier_score_ppm = 0_u64;

        for prediction in self.predictions.values() {
            match &prediction.status {
                PredictionStatus::Pending => pending += 1,
                PredictionStatus::Resolved { score, .. } => {
                    resolved += 1;
                    total_brier_score_ppm += u64::from(score.score_ppm);
                }
                PredictionStatus::Expired { .. } => expired += 1,
            }
        }

        PredictionLedgerSummary {
            issued: self.predictions.len() as u64,
            pending,
            resolved,
            expired,
            abstentions: self.abstentions.len() as u64,
            total_brier_score_ppm,
            mean_brier_score_ppm: (resolved > 0)
                .then_some((total_brier_score_ppm / resolved) as u32),
        }
    }

    #[must_use]
    pub fn calibration_buckets(&self) -> Vec<CalibrationBucket> {
        let mut counts = [0_u64; 10];
        let mut confidence_totals = [0_u64; 10];
        let mut correct_totals = [0_u64; 10];

        for prediction in self.predictions.values() {
            let PredictionStatus::Resolved { witness, .. } = &prediction.status else {
                continue;
            };
            let Some(top) = highest_probability(&prediction.outcomes) else {
                continue;
            };
            let bucket = usize::from((top.probability_bps / CALIBRATION_BUCKET_WIDTH_BPS).min(9));
            counts[bucket] += 1;
            confidence_totals[bucket] += u64::from(top.probability_bps);
            if top.label == witness.label {
                correct_totals[bucket] += 1;
            }
        }

        counts
            .iter()
            .enumerate()
            .filter_map(|(bucket, count)| {
                if *count == 0 {
                    return None;
                }
                let lower = bucket as u16 * CALIBRATION_BUCKET_WIDTH_BPS;
                let upper = if bucket == 9 {
                    TOTAL_PROBABILITY_BPS as u16
                } else {
                    lower + CALIBRATION_BUCKET_WIDTH_BPS - 1
                };
                Some(CalibrationBucket {
                    lower_confidence_bps: lower,
                    upper_confidence_bps: upper,
                    count: *count,
                    mean_confidence_bps: (confidence_totals[bucket] / *count) as u16,
                    observed_accuracy_bps: ((correct_totals[bucket]
                        * u64::from(TOTAL_PROBABILITY_BPS))
                        / *count) as u16,
                })
            })
            .collect()
    }

    fn require_version(&self, expected_version: u64) -> Result<(), PredictionLedgerError> {
        if expected_version != self.version {
            return Err(PredictionLedgerError::VersionConflict {
                expected: expected_version,
                actual: self.version,
            });
        }
        Ok(())
    }

    fn apply_event(
        &mut self,
        expected_version: u64,
        event: PredictionEvent,
    ) -> Result<u64, PredictionLedgerError> {
        self.require_version(expected_version)?;

        match &event {
            PredictionEvent::Issued { prediction } => {
                validate_prediction_record(prediction)?;
                if prediction.id != self.next_prediction_id {
                    return Err(PredictionLedgerError::UnexpectedPredictionId {
                        expected: self.next_prediction_id,
                        actual: prediction.id,
                    });
                }
                if self.predictions.contains_key(&prediction.id) {
                    return Err(PredictionLedgerError::DuplicatePredictionId(prediction.id));
                }
                self.predictions.insert(prediction.id, prediction.clone());
                self.next_prediction_id += 1;
            }
            PredictionEvent::Resolved {
                prediction_id,
                witness,
                score,
            } => {
                let prediction = self
                    .predictions
                    .get(prediction_id)
                    .ok_or(PredictionLedgerError::UnknownPrediction(*prediction_id))?;
                require_pending(prediction)?;
                let witness = prepare_witness(prediction, witness.clone())?;
                let computed = score_outcome(&prediction.outcomes, &witness.label)?;
                if computed != *score {
                    return Err(PredictionLedgerError::ScoreMismatch {
                        prediction_id: *prediction_id,
                    });
                }
                let prediction = self
                    .predictions
                    .get_mut(prediction_id)
                    .ok_or(PredictionLedgerError::UnknownPrediction(*prediction_id))?;
                prediction.status = PredictionStatus::Resolved {
                    witness,
                    score: *score,
                };
            }
            PredictionEvent::Expired {
                prediction_id,
                expired_at_ms,
            } => {
                let prediction = self
                    .predictions
                    .get(prediction_id)
                    .ok_or(PredictionLedgerError::UnknownPrediction(*prediction_id))?;
                require_pending(prediction)?;
                if *expired_at_ms < prediction.expires_at_ms {
                    return Err(PredictionLedgerError::NotExpiredYet {
                        prediction_id: *prediction_id,
                        expires_at_ms: prediction.expires_at_ms,
                        attempted_at_ms: *expired_at_ms,
                    });
                }
                let prediction = self
                    .predictions
                    .get_mut(prediction_id)
                    .ok_or(PredictionLedgerError::UnknownPrediction(*prediction_id))?;
                prediction.status = PredictionStatus::Expired {
                    expired_at_ms: *expired_at_ms,
                };
            }
            PredictionEvent::Abstained { abstention } => {
                validate_abstention_record(abstention)?;
                if abstention.id != self.next_abstention_id {
                    return Err(PredictionLedgerError::UnexpectedAbstentionId {
                        expected: self.next_abstention_id,
                        actual: abstention.id,
                    });
                }
                if self.abstentions.contains_key(&abstention.id) {
                    return Err(PredictionLedgerError::DuplicateAbstentionId(abstention.id));
                }
                self.abstentions.insert(abstention.id, abstention.clone());
                self.next_abstention_id += 1;
            }
        }

        self.version += 1;
        Ok(self.version)
    }
}

pub fn score_outcome(
    outcomes: &[OutcomeProbability],
    observed_label: &str,
) -> Result<BrierScore, PredictionLedgerError> {
    let outcomes = validate_outcomes(outcomes.to_vec())?;
    let observed_label = canonical_label(observed_label)
        .ok_or(PredictionLedgerError::EmptyWitnessLabel)?;
    if !outcomes.iter().any(|outcome| outcome.label == observed_label) {
        return Err(PredictionLedgerError::UnknownOutcomeLabel(observed_label));
    }

    let sum_squared_basis_points = outcomes
        .iter()
        .map(|outcome| {
            let observed = if outcome.label == observed_label {
                i64::from(TOTAL_PROBABILITY_BPS)
            } else {
                0
            };
            let residual = observed - i64::from(outcome.probability_bps);
            (residual * residual) as u64
        })
        .sum::<u64>();
    let score_ppm = ((sum_squared_basis_points * 1_000_000) / BRIER_DENOMINATOR) as u32;

    Ok(BrierScore {
        sum_squared_basis_points,
        score_ppm,
    })
}

fn prepare_prediction(
    id: PredictionId,
    input: PredictionInput,
) -> Result<PredictionRecord, PredictionLedgerError> {
    let subject_scope = normalize_text(&input.subject_scope)
        .ok_or(PredictionLedgerError::EmptySubjectScope)?;
    let producer = validate_producer(input.producer)?;
    let outcomes = validate_outcomes(input.outcomes)?;
    if input.not_before_ms < input.issued_at_ms {
        return Err(PredictionLedgerError::OutcomeWindowBeforeIssue);
    }
    if input.expires_at_ms <= input.not_before_ms {
        return Err(PredictionLedgerError::InvalidExpirationWindow);
    }

    Ok(PredictionRecord {
        id,
        subject_scope,
        producer,
        outcomes,
        issued_at_ms: input.issued_at_ms,
        not_before_ms: input.not_before_ms,
        expires_at_ms: input.expires_at_ms,
        context_digest: input.context_digest,
        status: PredictionStatus::Pending,
    })
}

fn prepare_abstention(
    id: AbstentionId,
    input: AbstentionInput,
) -> Result<AbstentionRecord, PredictionLedgerError> {
    let subject_scope = normalize_text(&input.subject_scope)
        .ok_or(PredictionLedgerError::EmptySubjectScope)?;
    let producer = validate_producer(input.producer)?;
    let reason = normalize_text(&input.reason).ok_or(PredictionLedgerError::EmptyAbstentionReason)?;
    Ok(AbstentionRecord {
        id,
        subject_scope,
        producer,
        reason,
        occurred_at_ms: input.occurred_at_ms,
        context_digest: input.context_digest,
    })
}

fn prepare_witness(
    prediction: &PredictionRecord,
    witness: OutcomeWitness,
) -> Result<OutcomeWitness, PredictionLedgerError> {
    if witness.source == WitnessSource::ResponseGenerator {
        return Err(PredictionLedgerError::SelfGradingWitness);
    }
    if witness.observed_at_ms < prediction.not_before_ms {
        return Err(PredictionLedgerError::WitnessTooEarly {
            prediction_id: prediction.id,
            not_before_ms: prediction.not_before_ms,
            observed_at_ms: witness.observed_at_ms,
        });
    }
    if witness.observed_at_ms > prediction.expires_at_ms {
        return Err(PredictionLedgerError::WitnessAfterExpiration {
            prediction_id: prediction.id,
            expires_at_ms: prediction.expires_at_ms,
            observed_at_ms: witness.observed_at_ms,
        });
    }
    let label = canonical_label(&witness.label).ok_or(PredictionLedgerError::EmptyWitnessLabel)?;
    if !prediction
        .outcomes
        .iter()
        .any(|outcome| outcome.label == label)
    {
        return Err(PredictionLedgerError::UnknownOutcomeLabel(label));
    }
    Ok(OutcomeWitness { label, ..witness })
}

fn validate_prediction_record(
    prediction: &PredictionRecord,
) -> Result<(), PredictionLedgerError> {
    if prediction.id == 0 {
        return Err(PredictionLedgerError::UnexpectedPredictionId {
            expected: 1,
            actual: 0,
        });
    }
    if !matches!(prediction.status, PredictionStatus::Pending) {
        return Err(PredictionLedgerError::IssuedRecordNotPending(prediction.id));
    }
    let rebuilt = prepare_prediction(
        prediction.id,
        PredictionInput {
            subject_scope: prediction.subject_scope.clone(),
            producer: prediction.producer.clone(),
            outcomes: prediction.outcomes.clone(),
            issued_at_ms: prediction.issued_at_ms,
            not_before_ms: prediction.not_before_ms,
            expires_at_ms: prediction.expires_at_ms,
            context_digest: prediction.context_digest,
        },
    )?;
    if rebuilt != *prediction {
        return Err(PredictionLedgerError::ReplayPredictionMismatch(prediction.id));
    }
    Ok(())
}

fn validate_abstention_record(
    abstention: &AbstentionRecord,
) -> Result<(), PredictionLedgerError> {
    if abstention.id == 0 {
        return Err(PredictionLedgerError::UnexpectedAbstentionId {
            expected: 1,
            actual: 0,
        });
    }
    let rebuilt = prepare_abstention(
        abstention.id,
        AbstentionInput {
            subject_scope: abstention.subject_scope.clone(),
            producer: abstention.producer.clone(),
            reason: abstention.reason.clone(),
            occurred_at_ms: abstention.occurred_at_ms,
            context_digest: abstention.context_digest,
        },
    )?;
    if rebuilt != *abstention {
        return Err(PredictionLedgerError::ReplayAbstentionMismatch(abstention.id));
    }
    Ok(())
}

fn validate_producer(
    mut producer: PredictionProducer,
) -> Result<PredictionProducer, PredictionLedgerError> {
    producer.id = normalize_text(&producer.id).ok_or(PredictionLedgerError::EmptyProducerId)?;
    Ok(producer)
}

fn validate_outcomes(
    outcomes: Vec<OutcomeProbability>,
) -> Result<Vec<OutcomeProbability>, PredictionLedgerError> {
    if outcomes.len() < 2 {
        return Err(PredictionLedgerError::TooFewOutcomes);
    }
    let mut labels = BTreeSet::new();
    let mut probability_mass = 0_u32;
    let mut normalized = Vec::with_capacity(outcomes.len());

    for outcome in outcomes {
        let label = canonical_label(&outcome.label)
            .ok_or(PredictionLedgerError::EmptyOutcomeLabel)?;
        if !labels.insert(label.clone()) {
            return Err(PredictionLedgerError::DuplicateOutcomeLabel(label));
        }
        if u32::from(outcome.probability_bps) > TOTAL_PROBABILITY_BPS {
            return Err(PredictionLedgerError::ProbabilityOutOfRange {
                label,
                probability_bps: outcome.probability_bps,
            });
        }
        probability_mass += u32::from(outcome.probability_bps);
        normalized.push(OutcomeProbability {
            label,
            probability_bps: outcome.probability_bps,
        });
    }

    if probability_mass != TOTAL_PROBABILITY_BPS {
        return Err(PredictionLedgerError::InvalidProbabilityMass {
            actual_bps: probability_mass,
        });
    }
    Ok(normalized)
}

fn require_pending(prediction: &PredictionRecord) -> Result<(), PredictionLedgerError> {
    if !matches!(prediction.status, PredictionStatus::Pending) {
        return Err(PredictionLedgerError::PredictionAlreadyFinalized(prediction.id));
    }
    Ok(())
}

fn highest_probability(outcomes: &[OutcomeProbability]) -> Option<&OutcomeProbability> {
    let mut highest = None;
    for outcome in outcomes {
        if highest
            .as_ref()
            .is_none_or(|current: &&OutcomeProbability| {
                outcome.probability_bps > current.probability_bps
            })
        {
            highest = Some(outcome);
        }
    }
    highest
}

fn normalize_text(raw: &str) -> Option<String> {
    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

fn canonical_label(raw: &str) -> Option<String> {
    normalize_text(raw).map(|label| label.to_ascii_lowercase())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PredictionLedgerError {
    #[error("ledger version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: u64, actual: u64 },
    #[error("subject scope must not be empty")]
    EmptySubjectScope,
    #[error("producer id must not be empty")]
    EmptyProducerId,
    #[error("prediction requires at least two outcomes")]
    TooFewOutcomes,
    #[error("outcome label must not be empty")]
    EmptyOutcomeLabel,
    #[error("duplicate outcome label: {0}")]
    DuplicateOutcomeLabel(String),
    #[error("probability for {label} is outside 0..=10000 bps: {probability_bps}")]
    ProbabilityOutOfRange {
        label: String,
        probability_bps: u16,
    },
    #[error("probability mass must equal 10000 bps, got {actual_bps}")]
    InvalidProbabilityMass { actual_bps: u32 },
    #[error("outcome window begins before prediction issue time")]
    OutcomeWindowBeforeIssue,
    #[error("expiration must occur after the outcome window begins")]
    InvalidExpirationWindow,
    #[error("abstention reason must not be empty")]
    EmptyAbstentionReason,
    #[error("witness label must not be empty")]
    EmptyWitnessLabel,
    #[error("response generator cannot witness or grade its own prediction")]
    SelfGradingWitness,
    #[error("witness for prediction {prediction_id} arrived at {observed_at_ms}, before {not_before_ms}")]
    WitnessTooEarly {
        prediction_id: PredictionId,
        not_before_ms: u64,
        observed_at_ms: u64,
    },
    #[error("witness for prediction {prediction_id} arrived at {observed_at_ms}, after expiration {expires_at_ms}")]
    WitnessAfterExpiration {
        prediction_id: PredictionId,
        expires_at_ms: u64,
        observed_at_ms: u64,
    },
    #[error("unknown outcome label: {0}")]
    UnknownOutcomeLabel(String),
    #[error("unknown prediction: {0}")]
    UnknownPrediction(PredictionId),
    #[error("prediction already resolved or expired: {0}")]
    PredictionAlreadyFinalized(PredictionId),
    #[error("prediction {prediction_id} expires at {expires_at_ms}; attempted expiry at {attempted_at_ms}")]
    NotExpiredYet {
        prediction_id: PredictionId,
        expires_at_ms: u64,
        attempted_at_ms: u64,
    },
    #[error("unexpected prediction id: expected {expected}, actual {actual}")]
    UnexpectedPredictionId {
        expected: PredictionId,
        actual: PredictionId,
    },
    #[error("duplicate prediction id: {0}")]
    DuplicatePredictionId(PredictionId),
    #[error("unexpected abstention id: expected {expected}, actual {actual}")]
    UnexpectedAbstentionId {
        expected: AbstentionId,
        actual: AbstentionId,
    },
    #[error("duplicate abstention id: {0}")]
    DuplicateAbstentionId(AbstentionId),
    #[error("issued prediction record was not pending: {0}")]
    IssuedRecordNotPending(PredictionId),
    #[error("replayed prediction record failed canonical reconstruction: {0}")]
    ReplayPredictionMismatch(PredictionId),
    #[error("replayed abstention record failed canonical reconstruction: {0}")]
    ReplayAbstentionMismatch(AbstentionId),
    #[error("stored score does not match recomputed score for prediction {prediction_id}")]
    ScoreMismatch { prediction_id: PredictionId },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn producer() -> PredictionProducer {
        PredictionProducer {
            id: "companion-policy-v1".to_owned(),
            kind: PredictionProducerKind::CompanionPolicy,
        }
    }

    fn binary_input(yes_bps: u16) -> PredictionInput {
        PredictionInput {
            subject_scope: "opaque-user/session".to_owned(),
            producer: producer(),
            outcomes: vec![
                OutcomeProbability {
                    label: "yes".to_owned(),
                    probability_bps: yes_bps,
                },
                OutcomeProbability {
                    label: "no".to_owned(),
                    probability_bps: 10_000 - yes_bps,
                },
            ],
            issued_at_ms: 100,
            not_before_ms: 110,
            expires_at_ms: 200,
            context_digest: 7,
        }
    }

    #[test]
    fn issued_prediction_stays_pending_and_replays_exactly() {
        let mut ledger = PredictionLedger::new();
        let issued = ledger.issue(0, binary_input(7_000)).unwrap();
        let prediction_id = issued.prediction_id.unwrap();

        assert!(matches!(
            ledger.prediction(prediction_id).unwrap().status,
            PredictionStatus::Pending
        ));
        let replayed = PredictionLedger::replay(&[issued.event]).unwrap();
        assert_eq!(ledger, replayed);
    }

    #[test]
    fn independent_delayed_witness_is_required_once() {
        let mut ledger = PredictionLedger::new();
        let issued = ledger.issue(0, binary_input(7_000)).unwrap();
        let prediction_id = issued.prediction_id.unwrap();

        let self_grade = ledger
            .resolve(
                ledger.version,
                prediction_id,
                OutcomeWitness {
                    source: WitnessSource::ResponseGenerator,
                    label: "yes".to_owned(),
                    observed_at_ms: 110,
                    evidence_digest: 1,
                },
            )
            .unwrap_err();
        assert_eq!(self_grade, PredictionLedgerError::SelfGradingWitness);

        let premature = ledger
            .resolve(
                ledger.version,
                prediction_id,
                OutcomeWitness {
                    source: WitnessSource::Environment,
                    label: "yes".to_owned(),
                    observed_at_ms: 109,
                    evidence_digest: 2,
                },
            )
            .unwrap_err();
        assert!(matches!(
            premature,
            PredictionLedgerError::WitnessTooEarly { .. }
        ));

        ledger
            .resolve(
                ledger.version,
                prediction_id,
                OutcomeWitness {
                    source: WitnessSource::Environment,
                    label: "yes".to_owned(),
                    observed_at_ms: 110,
                    evidence_digest: 3,
                },
            )
            .unwrap();

        let duplicate = ledger
            .resolve(
                ledger.version,
                prediction_id,
                OutcomeWitness {
                    source: WitnessSource::ExternalEvaluator,
                    label: "yes".to_owned(),
                    observed_at_ms: 120,
                    evidence_digest: 4,
                },
            )
            .unwrap_err();
        assert_eq!(
            duplicate,
            PredictionLedgerError::PredictionAlreadyFinalized(prediction_id)
        );
    }

    #[test]
    fn brier_score_is_exact_and_replay_verifies_it() {
        let mut ledger = PredictionLedger::new();
        let issued = ledger.issue(0, binary_input(7_000)).unwrap();
        let prediction_id = issued.prediction_id.unwrap();
        let resolved = ledger
            .resolve(
                ledger.version,
                prediction_id,
                OutcomeWitness {
                    source: WitnessSource::UserObservation,
                    label: "yes".to_owned(),
                    observed_at_ms: 150,
                    evidence_digest: 9,
                },
            )
            .unwrap();

        let PredictionStatus::Resolved { score, .. } =
            &ledger.prediction(prediction_id).unwrap().status
        else {
            panic!("prediction should be resolved");
        };
        assert_eq!(score.sum_squared_basis_points, 18_000_000);
        assert_eq!(score.score_ppm, 180_000);

        let replayed = PredictionLedger::replay(&[issued.event, resolved.event]).unwrap();
        assert_eq!(ledger, replayed);
    }

    #[test]
    fn abstention_and_expiry_are_versioned_without_fake_resolution() {
        let mut ledger = PredictionLedger::new();
        ledger
            .abstain(
                0,
                AbstentionInput {
                    subject_scope: "opaque-user/session".to_owned(),
                    producer: producer(),
                    reason: "insufficient evidence".to_owned(),
                    occurred_at_ms: 50,
                    context_digest: 3,
                },
            )
            .unwrap();
        let issued = ledger.issue(ledger.version, binary_input(5_000)).unwrap();
        let prediction_id = issued.prediction_id.unwrap();
        ledger.expire(ledger.version, prediction_id, 200).unwrap();

        let summary = ledger.summary();
        assert_eq!(summary.issued, 1);
        assert_eq!(summary.expired, 1);
        assert_eq!(summary.resolved, 0);
        assert_eq!(summary.abstentions, 1);
        assert_eq!(summary.mean_brier_score_ppm, None);
    }

    #[test]
    fn calibration_uses_top_label_confidence_and_observed_accuracy() {
        let mut ledger = PredictionLedger::new();
        for (yes_bps, label) in [(7_500, "yes"), (7_000, "no")] {
            let issued = ledger.issue(ledger.version, binary_input(yes_bps)).unwrap();
            ledger
                .resolve(
                    ledger.version,
                    issued.prediction_id.unwrap(),
                    OutcomeWitness {
                        source: WitnessSource::ExternalEvaluator,
                        label: label.to_owned(),
                        observed_at_ms: 150,
                        evidence_digest: u64::from(yes_bps),
                    },
                )
                .unwrap();
        }

        let buckets = ledger.calibration_buckets();
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].count, 2);
        assert_eq!(buckets[0].mean_confidence_bps, 7_250);
        assert_eq!(buckets[0].observed_accuracy_bps, 5_000);
    }
}
