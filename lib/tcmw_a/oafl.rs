//! OAFL — Online Anticipation Feedback Loop (Layer 5)
//!
//! Closes the learning loop: ✅ Perfect → ⚠️ Partial → ❌ Miss → EMA updates + grammar revision.

use serde::{Deserialize, Serialize};
use super::aih::IntentPrediction;
use super::bge::ArchetypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchQuality {
    Perfect,
    Partial,
    Miss,
}

impl MatchQuality {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchQuality::Perfect => "perfect",
            MatchQuality::Partial => "partial",
            MatchQuality::Miss => "miss",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionDelta {
    pub predicted: IntentPrediction,
    pub actual_action: String,
    pub delta_magnitude: f64,
    pub archetype_id: Option<ArchetypeId>,
    pub timestamp: i64,
    pub quality: MatchQuality,
}

impl PredictionDelta {
    pub fn new(predicted: IntentPrediction, actual_action: &str, quality: MatchQuality) -> Self {
        let delta_magnitude = match quality {
            MatchQuality::Perfect => 1.0,
            MatchQuality::Partial => 0.5,
            MatchQuality::Miss => 0.0,
        };
        let archetype_id = predicted.archetype_id;
        Self {
            predicted,
            actual_action: actual_action.to_string(),
            delta_magnitude,
            archetype_id,
            timestamp: crate::now_timestamp(),
            quality,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OAFL {
    ema_alpha: f64,
    revision_threshold: f64,
    delta_window: Vec<PredictionDelta>,
    window_size: usize,
    archetype_miss_rates: std::collections::HashMap<ArchetypeId, MissTracker>,
    grammar_revision_needed: bool,
}

#[derive(Debug, Clone, Default)]
struct MissTracker {
    total: usize,
    misses: usize,
}

impl Default for OAFL {
    fn default() -> Self {
        Self::new(0.3, 0.4)
    }
}

impl OAFL {
    pub fn new(ema_alpha: f64, revision_threshold: f64) -> Self {
        Self {
            ema_alpha,
            revision_threshold,
            delta_window: Vec::new(),
            window_size: 20,
            archetype_miss_rates: std::collections::HashMap::new(),
            grammar_revision_needed: false,
        }
    }

    pub fn compute_delta(&self, prediction: &IntentPrediction, actual_action: &str) -> PredictionDelta {
        let quality = self.classify_match(prediction, actual_action);
        PredictionDelta::new(prediction.clone(), actual_action, quality)
    }

    fn classify_match(&self, prediction: &IntentPrediction, actual: &str) -> MatchQuality {
        if prediction.rank == 1 && self.actions_match(&prediction.action, actual) {
            MatchQuality::Perfect
        } else if prediction.rank <= 5 && prediction.rank > 0 && self.actions_match(&prediction.action, actual) {
            MatchQuality::Partial
        } else {
            MatchQuality::Miss
        }
    }

    fn actions_match(&self, predicted: &str, actual: &str) -> bool {
        let p = predicted.to_lowercase();
        let a = actual.to_lowercase();
        if p == a {
            return true;
        }
        if p.contains(&a) || a.contains(&p) {
            return true;
        }
        let p_tokens: std::collections::HashSet<_> = p.split_whitespace().collect();
        let a_tokens: std::collections::HashSet<_> = a.split_whitespace().collect();
        let overlap: Vec<_> = p_tokens.intersection(&a_tokens).collect();
        let overlap_ratio = overlap.len() as f64 / p_tokens.len().max(a_tokens.len()) as f64;
        overlap_ratio >= 0.5
    }

    pub fn record(&mut self, delta: PredictionDelta) {
        self.delta_window.push(delta.clone());
        if self.delta_window.len() > self.window_size {
            self.delta_window.remove(0);
        }
        if let Some(arch_id) = &delta.archetype_id {
            let tracker = self.archetype_miss_rates
                .entry(*arch_id)
                .or_insert_with(MissTracker::default);
            tracker.total += 1;
            if matches!(delta.quality, MatchQuality::Miss) {
                tracker.misses += 1;
            }
        }
        if let Some(arch_id) = &delta.archetype_id {
            if let Some(tracker) = self.archetype_miss_rates.get(arch_id) {
                let miss_rate = tracker.misses as f64 / tracker.total.max(1) as f64;
                if miss_rate > self.revision_threshold {
                    self.grammar_revision_needed = true;
                }
            }
        }
    }

    pub fn ema_adjustment(&self, _archetype_id: &ArchetypeId, delta: &PredictionDelta) -> f64 {
        let base = match delta.quality {
            MatchQuality::Perfect => -0.02,
            MatchQuality::Partial => 0.005,
            MatchQuality::Miss => 0.05,
        };
        base * self.ema_alpha
    }

    pub fn is_revision_needed(&self) -> bool {
        self.grammar_revision_needed
    }

    pub fn consume_revision_flag(&mut self) {
        self.grammar_revision_needed = false;
    }

    pub fn miss_rate(&self) -> f64 {
        if self.delta_window.is_empty() {
            return 0.0;
        }
        let misses = self.delta_window.iter()
            .filter(|d| matches!(d.quality, MatchQuality::Miss))
            .count();
        misses as f64 / self.delta_window.len() as f64
    }

    pub fn recent_deltas(&self) -> &[PredictionDelta] {
        &self.delta_window
    }

    pub fn stats(&self) -> OAFLStats {
        OAFLStats {
            total_predictions: self.delta_window.len(),
            miss_rate: self.miss_rate(),
            perfect_count: self.delta_window.iter().filter(|d| matches!(d.quality, MatchQuality::Perfect)).count(),
            partial_count: self.delta_window.iter().filter(|d| matches!(d.quality, MatchQuality::Partial)).count(),
            miss_count: self.delta_window.iter().filter(|d| matches!(d.quality, MatchQuality::Miss)).count(),
            revision_needed: self.grammar_revision_needed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAFLStats {
    pub total_predictions: usize,
    pub miss_rate: f64,
    pub perfect_count: usize,
    pub partial_count: usize,
    pub miss_count: usize,
    pub revision_needed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_prediction(action: &str, rank: usize) -> IntentPrediction {
        IntentPrediction { rank, action: action.to_string(), probability: 0.8, horizon: 1, causal_parents: vec![], archetype_id: None }
    }

    #[test]
    fn test_perfect_match() {
        let oafl = OAFL::default();
        let pred = make_prediction("open VSCode", 1);
        let delta = oafl.compute_delta(&pred, "open VSCode");
        assert_eq!(delta.quality, MatchQuality::Perfect);
    }

    #[test]
    fn test_miss() {
        let oafl = OAFL::default();
        let pred = make_prediction("open VSCode", 1);
        let delta = oafl.compute_delta(&pred, "run tests");
        assert_eq!(delta.quality, MatchQuality::Miss);
    }

    #[test]
    fn test_actions_match_containment() {
        let oafl = OAFL::default();
        assert!(oafl.actions_match("open VSCode", "open VSCode now"));
        assert!(oafl.actions_match("open VSCode now", "open VSCode"));
    }

    #[test]
    fn test_actions_match_tokens() {
        let oafl = OAFL::default();
        assert!(oafl.actions_match("run tests", "run all tests"));
        assert!(oafl.actions_match("open file", "read file"));
    }

    #[test]
    fn test_ema_adjustment_signs() {
        let oafl = OAFL::default();
        let arch = ArchetypeId::new();
        let perfect = PredictionDelta::new(make_prediction("test", 1), "test", MatchQuality::Perfect);
        let miss = PredictionDelta::new(make_prediction("test", 1), "other", MatchQuality::Miss);
        assert!(oafl.ema_adjustment(&arch, &perfect) < 0.0);
        assert!(oafl.ema_adjustment(&arch, &miss) > 0.0);
    }

    #[test]
    fn test_revision_threshold_trigger() {
        let mut oafl = OAFL::new(0.3, 0.3);
        let arch = ArchetypeId::new();
        let pred = IntentPrediction { rank: 1, action: "test".to_string(), probability: 0.8, horizon: 1, causal_parents: vec![], archetype_id: Some(arch) };
        for _ in 0..5 {
            let delta = PredictionDelta::new(pred.clone(), "miss", MatchQuality::Miss);
            oafl.record(delta);
        }
        assert!(oafl.is_revision_needed());
    }

    #[test]
    fn test_stats() {
        let mut oafl = OAFL::default();
        let pred = make_prediction("test", 1);
        oafl.record(oafl.compute_delta(&pred, "test"));
        oafl.record(oafl.compute_delta(&pred, "close"));
        let stats = oafl.stats();
        assert_eq!(stats.total_predictions, 2);
        assert_eq!(stats.perfect_count, 1);
        assert_eq!(stats.miss_count, 1);
    }
}
