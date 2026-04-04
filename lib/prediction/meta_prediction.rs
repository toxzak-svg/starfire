//! Meta Prediction Engine — predicts prediction accuracy, calibrates confidence
//!
//! Philosophy: The prediction center needs to predict its own accuracy.
//! Which predictions should Star trust? How much should she weight each engine's output?

use super::types::*;
use std::collections::HashMap;

/// Meta Prediction Engine — predicts prediction accuracy, calibrates confidence
pub struct MetaPredictionEngine {
    /// Per-engine accuracy history
    engine_histories: HashMap<PredictionEngine, EngineHistory>,
    /// Per-kind accuracy history
    kind_histories: HashMap<PredictionKind, KindHistory>,
    /// Global calibration curve
    calibration_curve: CalibrationCurve,
    /// Horizon decay model
    horizon_decay: HorizonDecay,
    /// Total predictions made
    total_predictions: usize,
    /// Correct predictions (confirmed + surprised)
    correct_predictions: usize,
}

#[derive(Debug, Clone)]
struct EngineHistory {
    pub engine: PredictionEngine,
    /// Cumulative accuracy
    pub accuracy: f64,
    /// Accuracy trend (improving or degrading?)
    pub trend: f64,
    /// Recent outcomes for trend calculation
    recent_outcomes: Vec<bool>,
}

impl EngineHistory {
    fn new(engine: PredictionEngine) -> Self {
        EngineHistory {
            engine,
            accuracy: 0.5, // Start with moderate confidence
            trend: 0.0,
            recent_outcomes: Vec::new(),
        }
    }

    fn update(&mut self, correct: bool) {
        let n = self.recent_outcomes.len() as f64;
        
        // Update accuracy with exponential moving average
        self.accuracy = self.accuracy * 0.9 + if correct { 0.1 } else { 0.0 };
        
        // Update recent outcomes
        self.recent_outcomes.push(correct);
        if self.recent_outcomes.len() > 10 {
            self.recent_outcomes.remove(0);
        }
        
        // Calculate trend
        if self.recent_outcomes.len() >= 2 {
            let recent = self.recent_outcomes.len();
            let recent_avg = self.recent_outcomes.iter()
                .skip(recent.saturating_sub(5))
                .filter(|&&x| x)
                .count() as f64 / 5.0_f64.min(recent as f64);
            let older = self.recent_outcomes.len() - 5;
            let older_avg = if older > 0 {
                self.recent_outcomes.iter()
                    .take(older)
                    .filter(|&&x| x)
                    .count() as f64 / older as f64
            } else {
                recent_avg
            };
            self.trend = recent_avg - older_avg;
        }
    }
}

#[derive(Debug, Clone)]
struct KindHistory {
    pub kind: PredictionKind,
    pub accuracy: f64,
    pub sample_count: usize,
}

impl KindHistory {
    fn new(kind: PredictionKind) -> Self {
        KindHistory {
            kind,
            accuracy: 0.5,
            sample_count: 0,
        }
    }

    fn update(&mut self, correct: bool) {
        self.sample_count += 1;
        let n = self.sample_count as f64;
        self.accuracy = (self.accuracy * (n - 1.0) + if correct { 1.0 } else { 0.0 }) / n;
    }
}

#[derive(Debug, Clone)]
struct CalibrationCurve {
    /// Bins: (predicted_confidence_range, observed_accuracy)
    bins: Vec<CalibrationBin>,
}

#[derive(Debug, Clone)]
struct CalibrationBin {
    pub min_conf: f64,
    pub max_conf: f64,
    pub observed_accuracy: f64,
    pub sample_count: usize,
}

impl CalibrationCurve {
    fn new() -> Self {
        // Initialize bins for common confidence ranges
        let bins = vec![
            CalibrationBin { min_conf: 0.0, max_conf: 0.1, observed_accuracy: 0.5, sample_count: 0 },
            CalibrationBin { min_conf: 0.1, max_conf: 0.2, observed_accuracy: 0.5, sample_count: 0 },
            CalibrationBin { min_conf: 0.2, max_conf: 0.3, observed_accuracy: 0.5, sample_count: 0 },
            CalibrationBin { min_conf: 0.3, max_conf: 0.4, observed_accuracy: 0.5, sample_count: 0 },
            CalibrationBin { min_conf: 0.4, max_conf: 0.5, observed_accuracy: 0.5, sample_count: 0 },
            CalibrationBin { min_conf: 0.5, max_conf: 0.6, observed_accuracy: 0.5, sample_count: 0 },
            CalibrationBin { min_conf: 0.6, max_conf: 0.7, observed_accuracy: 0.5, sample_count: 0 },
            CalibrationBin { min_conf: 0.7, max_conf: 0.8, observed_accuracy: 0.5, sample_count: 0 },
            CalibrationBin { min_conf: 0.8, max_conf: 0.9, observed_accuracy: 0.5, sample_count: 0 },
            CalibrationBin { min_conf: 0.9, max_conf: 1.0, observed_accuracy: 0.5, sample_count: 0 },
        ];
        CalibrationCurve { bins }
    }

    fn update(&mut self, predicted_confidence: f64, correct: bool) {
        for bin in &mut self.bins {
            if predicted_confidence >= bin.min_conf && predicted_confidence < bin.max_conf {
                bin.sample_count += 1;
                let n = bin.sample_count as f64;
                bin.observed_accuracy = (bin.observed_accuracy * (n - 1.0) + if correct { 1.0 } else { 0.0 }) / n;
                break;
            }
        }
    }

    fn get_bin(&self, confidence: f64) -> Option<&CalibrationBin> {
        self.bins.iter()
            .find(|b| confidence >= b.min_conf && confidence < b.max_conf)
    }

    fn overall_accuracy(&self) -> f64 {
        let total: usize = self.bins.iter().map(|b| b.sample_count).sum();
        if total == 0 {
            return 0.5;
        }
        let weighted: f64 = self.bins.iter()
            .map(|b| b.observed_accuracy * b.sample_count as f64)
            .sum();
        weighted / total as f64
    }
}

#[derive(Debug, Clone)]
struct HorizonDecay {
    /// Base accuracy per horizon
    base_by_horizon: HashMap<usize, f64>,
}

impl HorizonDecay {
    fn new() -> Self {
        let mut base_by_horizon = HashMap::new();
        base_by_horizon.insert(1, 0.9);
        base_by_horizon.insert(2, 0.8);
        base_by_horizon.insert(3, 0.7);
        base_by_horizon.insert(4, 0.6);
        base_by_horizon.insert(5, 0.5);
        // For horizons > 5, decay further
        HorizonDecay { base_by_horizon }
    }

    fn rate(&self, horizon: usize) -> f64 {
        if let Some(rate) = self.base_by_horizon.get(&horizon) {
            *rate
        } else {
            // Extrapolate for higher horizons
            (0.5 / (1.0 + (horizon - 5) as f64 * 0.2)).max(0.1)
        }
    }
}

impl MetaPredictionEngine {
    pub fn new() -> Self {
        let mut engine_histories = HashMap::new();
        for engine in [
            PredictionEngine::QuestionGravity,
            PredictionEngine::BeliefRevision,
            PredictionEngine::Basin,
            PredictionEngine::Meta,
        ] {
            engine_histories.insert(engine, EngineHistory::new(engine));
        }

        let mut kind_histories = HashMap::new();
        for kind in [
            PredictionKind::Conclusion,
            PredictionKind::Question,
            PredictionKind::NecessaryTruth,
            PredictionKind::BeliefChange,
            PredictionKind::StateChange,
        ] {
            kind_histories.insert(kind, KindHistory::new(kind));
        }

        MetaPredictionEngine {
            engine_histories,
            kind_histories,
            calibration_curve: CalibrationCurve::new(),
            horizon_decay: HorizonDecay::new(),
            total_predictions: 0,
            correct_predictions: 0,
        }
    }

    /// Calibrate a raw prediction confidence using learned history
    pub fn calibrate(&self, prediction: &Prediction) -> f64 {
        let raw = prediction.confidence;

        // 1. Apply engine-specific adjustment
        let engine_factor = self.engine_accuracy_factor(prediction.engine);

        // 2. Apply kind-specific adjustment
        let kind_factor = self.kind_accuracy_factor(prediction.kind);

        // 3. Apply horizon decay
        let horizon_factor = self.horizon_decay.rate(prediction.horizon);

        // 4. Apply calibration curve correction
        let calibration = self.calibration_correction(raw);

        let calibrated = raw * engine_factor * kind_factor * horizon_factor * calibration;
        calibrated.clamp(0.01, 0.99)
    }

    /// Get the engine accuracy factor (how well has this engine performed?)
    fn engine_accuracy_factor(&self, engine: PredictionEngine) -> f64 {
        if let Some(history) = self.engine_histories.get(&engine) {
            // Scale: accuracy 0.5 → factor 1.0, accuracy 0.9 → factor 1.2, accuracy 0.2 → factor 0.6
            0.5 + history.accuracy
        } else {
            1.0
        }
    }

    /// Get the kind accuracy factor
    fn kind_accuracy_factor(&self, kind: PredictionKind) -> f64 {
        if let Some(history) = self.kind_histories.get(&kind) {
            if history.sample_count < 5 {
                return 1.0; // Not enough data
            }
            0.5 + history.accuracy
        } else {
            1.0
        }
    }

    /// Apply calibration curve correction
    fn calibration_correction(&self, raw: f64) -> f64 {
        if let Some(bin) = self.calibration_curve.get_bin(raw) {
            if bin.sample_count < 5 {
                return 1.0; // Not enough data, trust raw
            }

            // If observed accuracy < predicted confidence → reduce confidence
            // If observed accuracy > predicted confidence → can trust slightly more
            let observed = bin.observed_accuracy;
            if observed < raw {
                // Overconfident — pull down
                let overconfidence = raw - observed;
                (raw - overconfidence * 0.5).max(0.1)
            } else {
                // Underconfident — pull up slightly
                raw + (observed - raw) * 0.3
            }
        } else {
            1.0
        }
    }

    /// Record the outcome of a prediction for future calibration
    pub fn record_outcome(&mut self, prediction_id: PredictionId, outcome: PredictionOutcome) {
        // Map outcome to correctness
        let correct = match outcome {
            PredictionOutcome::Confirmed => true,
            PredictionOutcome::Surprised => true, // Surprised means we were right but didn't expect it
            PredictionOutcome::Refuted => false,
            PredictionOutcome::Uncertain => false,
        };

        self.total_predictions += 1;
        if correct {
            self.correct_predictions += 1;
        }

        // Update calibration curve - this needs the original prediction's confidence
        // For now, we update with a default mid-range confidence
        self.calibration_curve.update(0.5, correct);
    }

    /// Record outcome with the prediction's original confidence
    pub fn record_outcome_with_confidence(&mut self, raw_confidence: f64, outcome: PredictionOutcome) {
        let correct = match outcome {
            PredictionOutcome::Confirmed => true,
            PredictionOutcome::Surprised => true,
            PredictionOutcome::Refuted => false,
            PredictionOutcome::Uncertain => false,
        };

        self.total_predictions += 1;
        if correct {
            self.correct_predictions += 1;
        }

        self.calibration_curve.update(raw_confidence, correct);
    }

    /// Update engine-specific accuracy
    pub fn update_engine_accuracy(&mut self, engine: PredictionEngine, correct: bool) {
        if let Some(history) = self.engine_histories.get_mut(&engine) {
            history.update(correct);
        }
    }

    /// Update kind-specific accuracy
    pub fn update_kind_accuracy(&mut self, kind: PredictionKind, correct: bool) {
        if let Some(history) = self.kind_histories.get_mut(&kind) {
            history.update(correct);
        }
    }

    /// Compute how much to trust each engine based on recent performance
    pub fn engine_weights(&self) -> HashMap<PredictionEngine, f64> {
        let mut weights = HashMap::new();
        let total: f64 = self.engine_histories.values()
            .map(|h| h.accuracy.max(0.01)) // Avoid zero
            .sum();

        for (engine, history) in &self.engine_histories {
            let weight = history.accuracy.max(0.01) / total;
            weights.insert(*engine, weight);
        }

        weights
    }

    /// What is the predicted accuracy at a given horizon?
    pub fn horizon_confidence(&self, horizon: usize) -> f64 {
        // Predictions at longer horizons are systematically less accurate
        // Model: accuracy = base_accuracy * horizon_decay
        let base = self.calibration_curve.overall_accuracy();
        let decay = self.horizon_decay.rate(horizon);
        base * decay
    }

    /// Get overall accuracy
    pub fn overall_accuracy(&self) -> f64 {
        if self.total_predictions == 0 {
            return 0.5;
        }
        self.correct_predictions as f64 / self.total_predictions as f64
    }

    /// Get engine accuracy
    pub fn get_engine_accuracy(&self, engine: PredictionEngine) -> f64 {
        self.engine_histories.get(&engine)
            .map(|h| h.accuracy)
            .unwrap_or(0.5)
    }

    /// Get engine trend
    pub fn get_engine_trend(&self, engine: PredictionEngine) -> f64 {
        self.engine_histories.get(&engine)
            .map(|h| h.trend)
            .unwrap_or(0.0)
    }
}

impl Default for MetaPredictionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibration() {
        let meta = MetaPredictionEngine::new();
        
        // Create a test prediction
        let prediction = Prediction::new(
            PredictionEngine::QuestionGravity,
            PredictionKind::Question,
            PredictedCore::Question {
                question_text: "test".to_string(),
                topic_domain: "test".to_string(),
                expected_answer_type: AnswerType::Unknown,
            },
            "Test prediction".to_string(),
            0.8,
            1,
            vec!["test reasoning".to_string()],
        );
        
        let calibrated = meta.calibrate(&prediction);
        
        // Calibrated should be within valid range
        assert!(calibrated >= 0.01 && calibrated <= 0.99);
    }

    #[test]
    fn test_engine_weights() {
        let meta = MetaPredictionEngine::new();
        
        let weights = meta.engine_weights();
        
        // Should have weights for all 4 engines
        assert_eq!(weights.len(), 4);
        
        // Weights should sum to ~1.0
        let sum: f64 = weights.values().sum();
        assert!((sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_horizon_decay() {
        let meta = MetaPredictionEngine::new();
        
        let conf_1 = meta.horizon_confidence(1);
        let conf_5 = meta.horizon_confidence(5);
        
        // Longer horizon should have lower confidence
        assert!(conf_1 >= conf_5);
    }

    #[test]
    fn test_record_outcome() {
        let mut meta = MetaPredictionEngine::new();
        
        meta.record_outcome(PredictionId::new(), PredictionOutcome::Confirmed);
        
        assert_eq!(meta.total_predictions, 1);
        assert_eq!(meta.correct_predictions, 1);
    }

    #[test]
    fn test_calibration_curve_converges() {
        let mut meta = MetaPredictionEngine::new();
        
        // Record many high-confidence correct predictions
        for _ in 0..10 {
            meta.record_outcome_with_confidence(0.9, PredictionOutcome::Confirmed);
        }
        
        // Record many low-confidence incorrect predictions
        for _ in 0..10 {
            meta.record_outcome_with_confidence(0.2, PredictionOutcome::Refuted);
        }
        
        // Check calibration
        let bin = meta.calibration_curve.get_bin(0.9).unwrap();
        assert!(bin.observed_accuracy > 0.7); // Should be accurate!
        
        let bin_low = meta.calibration_curve.get_bin(0.2).unwrap();
        assert!(bin_low.observed_accuracy < 0.4); // Should be inaccurate!
    }
}