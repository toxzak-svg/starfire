//! Regime Memory — tracks metastable reasoning regime transitions and dwell times
//!
//! Metastable regions = reasoning modes where trajectories "dwell" disproportionately long.
//! Not asymptotic fixed points — quasi-stable states with well-defined escape statistics.
//!
//! Uses:
//! - HMM-like state tracking: which regime, dwell time, transition counts
//! - Fragility from regime statistics: high escape rate = fragile
//! - Drives the slow ASRU loop

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::regime_classifier::{ReasoningRegime, RegimePrediction};
use super::fragility::{State, FragilityEstimator, AttractorFragility};

/// Statistics about a single regime's metastable behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeStats {
    /// How many times we've entered this regime
    pub visit_count: u64,
    /// Total time spent in this regime (arbitrary time units)
    pub total_dwell: u64,
    /// Mean dwell time per visit
    pub mean_dwell: f64,
    /// Variance in dwell time
    pub dwell_variance: f64,
    /// Most recent dwell time
    pub last_dwell: u64,
    /// Escape rate: fraction of transitions that left this regime
    pub escape_rate: f64,
    /// Mean escape rate (exponential decay parameter)
    pub mean_escape_rate: f64,
}

impl Default for RegimeStats {
    fn default() -> Self {
        Self {
            visit_count: 0,
            total_dwell: 0,
            mean_dwell: 0.0,
            dwell_variance: 0.0,
            last_dwell: 0,
            escape_rate: 0.0,
            mean_escape_rate: 0.0,
        }
    }
}

/// Transition between regimes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegimeTransition {
    pub from: ReasoningRegime,
    pub to: ReasoningRegime,
    pub dwell: u64,
}

impl RegimeStats {
    /// Update with a new dwell time observation
    pub fn record_dwell(&mut self, dwell: u64) {
        self.visit_count += 1;
        self.total_dwell += dwell;
        self.last_dwell = dwell;

        // Welford's online algorithm for variance
        let n = self.visit_count as f64;
        let delta = dwell as f64 - self.mean_dwell;
        self.mean_dwell += delta / n;
        let delta2 = dwell as f64 - self.mean_dwell;
        self.dwell_variance += delta * delta2;

        // Escape rate: if we've exited, this was a complete visit
        self.escape_rate = 1.0; // Reset on exit
    }

    /// Compute fragility: inverse of mean dwell time
    /// Low mean dwell = fragile (escapes quickly)
    /// High mean dwell = robust
    pub fn fragility(&self) -> f64 {
        if self.mean_dwell < 1.0 {
            return 1.0; // Max fragile
        }
        // Fragility = 1 / mean_dwell, normalized
        let raw = 1.0 / self.mean_dwell;
        raw.clamp(0.0, 1.0)
    }
}

/// Regime memory — tracks regime sequence, computes metastable statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeMemory {
    /// Per-regime statistics
    stats: HashMap<ReasoningRegime, RegimeStats>,
    /// Transition counts: (from, to) → count
    transitions: HashMap<(ReasoningRegime, ReasoningRegime), u64>,
    /// Total transitions observed
    total_transitions: u64,
    /// Current regime
    current_regime: ReasoningRegime,
    /// Dwell time in current regime
    current_dwell: u64,
    /// Regime history (circular buffer)
    history: Vec<ReasoningRegime>,
    max_history: usize,
    /// Global fragility estimate (exponential moving average)
    global_fragility: f64,
    ema_alpha: f64,
}

impl RegimeMemory {
    pub fn new(max_history: usize) -> Self {
        let mut stats = HashMap::new();
        for r in 0..ReasoningRegime::num_regimes() {
            stats.insert(ReasoningRegime::from_id(r as u8), RegimeStats::default());
        }
        Self {
            stats,
            transitions: HashMap::new(),
            total_transitions: 0,
            current_regime: ReasoningRegime::SteadyState,
            current_dwell: 0,
            history: Vec::with_capacity(max_history),
            max_history,
            global_fragility: 0.5,
            ema_alpha: 0.1,
        }
    }

    /// Update with a new regime observation
    pub fn observe(&mut self, prediction: &RegimePrediction) {
        let new_regime = prediction.regime;

        if new_regime != self.current_regime {
            // Regime transition!
            let from = self.current_regime;

            // Record dwell time for the regime we left
            if let Some(stats) = self.stats.get_mut(&from) {
                stats.record_dwell(self.current_dwell);
            }

            // Record transition
            *self.transitions.entry((from, new_regime)).or_insert(0) += 1;
            self.total_transitions += 1;

            // Compute transition probability (escape rate for current regime)
            if let Some(stats) = self.stats.get_mut(&from) {
                let from_transitions: u64 = self.transitions.iter()
                    .filter(|((f, _), _)| *f == from)
                    .map(|(_, &c)| c)
                    .sum();
            stats.escape_rate = from_transitions as f64 / self.total_transitions.max(1) as f64;
            }

            // Enter new regime
            self.current_regime = new_regime;
            self.current_dwell = 1;

            // Update history
            if self.history.len() >= self.max_history {
                self.history.remove(0);
            }
            self.history.push(new_regime);
        } else {
            // Same regime — increment dwell
            self.current_dwell += 1;
        }
    }

    /// Get current regime
    pub fn current_regime(&self) -> ReasoningRegime {
        self.current_regime
    }

    /// Get current dwell time
    pub fn current_dwell(&self) -> u64 {
        self.current_dwell
    }

    /// Get statistics for a regime
    pub fn stats(&self, regime: ReasoningRegime) -> Option<&RegimeStats> {
        self.stats.get(&regime)
    }

    /// Get all stats
    pub fn all_stats(&self) -> &HashMap<ReasoningRegime, RegimeStats> {
        &self.stats
    }

    /// Compute fragility for current regime
    /// Fragility = how easily does this regime lose its metastability?
    pub fn current_regime_fragility(&self) -> f64 {
        let regime_stats = self.stats.get(&self.current_regime);
        let dwell_fragility = regime_stats.map(|s| s.fragility()).unwrap_or(0.5);

        // Also factor in escape rate
        let escape_fragility = regime_stats.map(|s| s.escape_rate).unwrap_or(0.5);

        // Combined: fragile if escapes quickly OR mean dwell is low
        (dwell_fragility * 0.6 + escape_fragility * 0.4) as f64
    }

    /// Get transition probability from one regime to another
    pub fn transition_prob(&self, from: ReasoningRegime, to: ReasoningRegime) -> f64 {
        let total_from: u64 = self.transitions.iter()
            .filter(|((f, _), _)| *f == from)
            .map(|(_, &c)| c)
            .sum();
        if total_from == 0 {
            return 1.0 / ReasoningRegime::num_regimes() as f64; // Uniform
        }
        let count = *self.transitions.get(&(from, to)).unwrap_or(&0) as f64;
        count / total_from as f64
    }

    /// Predict most likely next regime given current
    pub fn most_likely_next(&self) -> ReasoningRegime {
        let mut best_prob = 0.0f64;
        let mut best_regime = ReasoningRegime::SteadyState;
        for r in 0..ReasoningRegime::num_regimes() {
            let regime = ReasoningRegime::from_id(r as u8);
            let prob = self.transition_prob(self.current_regime, regime);
            if prob > best_prob {
                best_prob = prob;
                best_regime = regime;
            }
        }
        best_regime
    }

    /// Get recent regime history
    pub fn history(&self) -> &[ReasoningRegime] {
        &self.history
    }

    /// Update global fragility estimate
    pub fn update_global_fragility(&mut self, afi: f64) {
        self.global_fragility = self.ema_alpha * afi as f64 + (1.0 - self.ema_alpha) * self.global_fragility;
    }

    pub fn global_fragility(&self) -> f64 {
        self.global_fragility
    }
}

/// Combined regime + fragility tracker for ASRU
pub struct RegimeTracker {
    pub memory: RegimeMemory,
    pub fragility: FragilityEstimator,
}

impl RegimeTracker {
    pub fn new(max_trajectory: usize, max_history: usize) -> Self {
        Self {
            memory: RegimeMemory::new(max_history),
            fragility: FragilityEstimator::new(max_trajectory),
        }
    }

    /// Update with new input + state
    pub fn update(&mut self, text: &str, state_features: &[f32]) {
        // Classify regime from text
        let regime_pred = RegimePrediction::classify(text);
        self.memory.observe(&regime_pred);

        // Update fragility estimator with state features
        self.fragility.observe_from_features(state_features);
    }

    /// Get current reasoning regime
    pub fn current_regime(&self) -> ReasoningRegime {
        self.memory.current_regime()
    }

    /// Get dwell time in current regime
    pub fn current_dwell(&self) -> u64 {
        self.memory.current_dwell()
    }

    /// Compute composite fragility (AFI from trajectory + regime statistics)
    pub fn compute_afi(&mut self) -> AttractorFragility {
        let trajectory_afi = self.fragility.compute_afi();
        let regime_fragility = self.memory.current_regime_fragility() as f32;

        // Combined AFI: 70% trajectory-based, 30% regime-transition-based
        let combined_afi = 0.7 * trajectory_afi.afi + 0.3 * regime_fragility;

        AttractorFragility {
            lyapunov_leading: trajectory_afi.lyapunov_leading,
            rqa_det_drop: trajectory_afi.rqa_det_drop,
            afi: combined_afi.clamp(0.0, 1.0),
            trajectory_len: trajectory_afi.trajectory_len,
        }
    }

    /// Check if current regime is fragile
    pub fn is_fragile(&self, threshold: f32) -> bool {
        self.memory.current_regime_fragility() > threshold as f64
    }

    /// Get most likely next regime
    pub fn predicted_next_regime(&self) -> ReasoningRegime {
        self.memory.most_likely_next()
    }

    /// Reset fragility trajectory (call on regime change)
    pub fn reset_trajectory(&mut self) {
        self.fragility.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regime_tracking() {
        let mut tracker = RegimeTracker::new(100, 50);

        // Observe emotional text
        tracker.update("I feel really sad and lonely today", &[0.1, 0.8, 0.1, 0.0, 0.2, 0.5, 0.3, 0.6]);
        assert_eq!(tracker.current_regime(), ReasoningRegime::EmotionalResonance);
        assert_eq!(tracker.current_dwell(), 1);

        // Same regime
        tracker.update("I'm so frustrated about this", &[0.1, 0.7, 0.1, 0.0, 0.2, 0.4, 0.2, 0.5]);
        assert_eq!(tracker.current_regime(), ReasoningRegime::EmotionalResonance);
        assert_eq!(tracker.current_dwell(), 2);

        // Transition to causal
        tracker.update("Because the temperature rose, the pressure increased", &[0.2, 0.1, 0.8, 0.0, 0.1, 0.6, 0.1, 0.2]);
        assert_eq!(tracker.current_regime(), ReasoningRegime::CausalReasoning);
        assert_eq!(tracker.current_dwell(), 1);

        // Check fragility is computed
        let afi = tracker.compute_afi();
        assert!(afi.afi >= 0.0 && afi.afi <= 1.0);
    }

    #[test]
    fn test_transition_probability() {
        let mut tracker = RegimeTracker::new(100, 50);

        // Force a few transitions
        tracker.update("I feel sad", &[0.1, 0.9, 0.0, 0.0, 0.1, 0.4, 0.2, 0.7]);
        tracker.update("Because the", &[0.1, 0.1, 0.9, 0.0, 0.1, 0.5, 0.1, 0.2]);
        tracker.update("I feel sad", &[0.1, 0.9, 0.0, 0.0, 0.1, 0.4, 0.2, 0.7]);
        tracker.update("Because the", &[0.1, 0.1, 0.9, 0.0, 0.1, 0.5, 0.1, 0.2]);

        let prob = tracker.memory.transition_prob(
            ReasoningRegime::EmotionalResonance,
            ReasoningRegime::CausalReasoning
        );
        assert!(prob > 0.0); // Should have seen this transition
    }

    #[test]
    fn test_dwell_tracking() {
        let mut tracker = RegimeTracker::new(100, 50);

        // Stay in one regime for several observations
        for _ in 0..5 {
            tracker.update("I feel really frustrated and angry", &[0.1, 0.9, 0.1, 0.0, 0.2, 0.5, 0.3, 0.6]);
        }
        assert_eq!(tracker.current_dwell(), 5);

        // Check mean dwell recorded
        let stats = tracker.memory.stats(ReasoningRegime::EmotionalResonance).unwrap();
        assert!(stats.visit_count >= 1);
    }
}
