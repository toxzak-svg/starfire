//! Creativity — Novelty detection, creative evaluation, oscillation
//!
//! Ported from Python `creativity.py`

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Creativity output from the creative oscillation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativityOutput {
    /// Current creative state (0-1)
    pub creative_state: f64,
    /// Divergence metric (0-1)
    pub divergence_metric: f64,
    /// Diversity index (0-1)
    pub diversity_index: f64,
    /// Originality score (0-1)
    pub originality_score: f64,
    /// Oscillation phase (radians)
    pub oscillation_phase: f64,
}

impl Default for CreativityOutput {
    fn default() -> Self {
        Self {
            creative_state: 0.0,
            divergence_metric: 0.0,
            diversity_index: 0.0,
            originality_score: 0.0,
            oscillation_phase: 0.0,
        }
    }
}

/// Creative oscillation controller
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CreativeOscillator {
    /// Current phase: 'ordered' or 'exploratory'
    state: CreativePhase,
    /// Order threshold for triggering chaos
    order_threshold: f64,
    /// Chaos threshold for returning to order
    chaos_threshold: f64,
    /// Maximum exploration steps
    max_exploration: usize,
    /// Convergence rate
    convergence_rate: f64,
    /// Exploration counter
    exploration_count: usize,
    /// Best exploration value
    best_value: f64,
    /// Phase accumulated over time
    phase_accumulator: f64,
    /// Phase frequency (rad per step)
    phase_frequency: f64,
    /// History of outputs
    history: VecDeque<CreativityOutput>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CreativePhase {
    Ordered,
    Exploratory,
}

impl Default for CreativeOscillator {
    fn default() -> Self {
        Self::new()
    }
}

impl CreativeOscillator {
    pub fn new() -> Self {
        Self {
            state: CreativePhase::Ordered,
            order_threshold: 0.7,
            chaos_threshold: 0.3,
            max_exploration: 50,
            convergence_rate: 0.1,
            exploration_count: 0,
            best_value: 0.0,
            phase_accumulator: 0.0,
            phase_frequency: 0.1,
            history: VecDeque::with_capacity(1000),
        }
    }

    /// Step the creative oscillator
    pub fn step(&mut self, state: &[f64], _consciousness: f64, _novelty: f64) -> CreativityOutput {
        let creative_state = self.compute_creative_state(state);
        let divergence = self.compute_divergence(state);
        let diversity = self.compute_diversity(state);
        let originality = self.compute_originality(state);

        // Update phase
        self.phase_accumulator += self.phase_frequency;
        let phase = self.phase_accumulator;

        // Check for phase transition
        self.check_transition(creative_state, divergence);

        let output = CreativityOutput {
            creative_state,
            divergence_metric: divergence,
            diversity_index: diversity,
            originality_score: originality,
            oscillation_phase: phase,
        };

        // Update best
        if creative_state > self.best_value {
            self.best_value = creative_state;
        }

        // Record history
        self.history.push_back(output.clone());
        if self.history.len() > 1000 {
            self.history.pop_front();
        }

        output
    }

    fn compute_creative_state(&self, state: &[f64]) -> f64 {
        if state.is_empty() {
            return 0.0;
        }

        // Creativity = combination of:
        // 1. Activation spread (how many non-zero components)
        // 2. Non-linearity (tanh activation spread)
        let zero_count = state.iter().filter(|&&x| x.abs() < 0.1).count() as f64;
        let sparsity = 1.0 - zero_count / state.len() as f64;

        // Activation variance
        let mean = state.iter().sum::<f64>() / state.len() as f64;
        let variance = state.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / state.len() as f64;

        (sparsity + variance.min(1.0)) / 2.0
    }

    fn compute_divergence(&self, state: &[f64]) -> f64 {
        if state.is_empty() || self.history.is_empty() {
            return 0.0;
        }

        // Distance from average of recent states
        let recent: Vec<_> = self.history.iter().rev().take(10).collect();
        if recent.is_empty() {
            return 0.0;
        }

        let avg_state: Vec<f64> = (0..state.len())
            .map(|_i| {
                recent.iter()
                    .map(|o| o.diversity_index)
                    .sum::<f64>() / recent.len() as f64
            })
            .collect();

        let diff: f64 = state.iter()
            .zip(avg_state.iter())
            .map(|(s, a)| (s - a).powi(2))
            .sum::<f64>()
            .sqrt();

        (diff / (state.len() as f64).sqrt()).min(1.0)
    }

    fn compute_diversity(&self, state: &[f64]) -> f64 {
        if state.is_empty() {
            return 0.0;
        }

        // Diversity = entropy of activation distribution
        let n_bins = 10;
        let min_val = state.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = state.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = (max_val - min_val).max(1e-10);

        let mut counts = vec![0usize; n_bins];
        for v in state {
            let bin = ((v - min_val) / range * n_bins as f64) as usize;
            let bin = bin.min(n_bins - 1);
            counts[bin] += 1;
        }

        let n = state.len() as f64;
        let mut entropy = 0.0f64;
        for &c in &counts {
            if c > 0 {
                let p = c as f64 / n;
                entropy -= p * p.ln();
            }
        }

        let max_entropy = (n_bins as f64).ln();
        (entropy / max_entropy).min(1.0)
    }

    fn compute_originality(&self, state: &[f64]) -> f64 {
        if state.is_empty() || self.history.len() < 10 {
            return 1.0; // First state is maximally original
        }

        // Distance from all historical states
        let mut total_distance = 0.0f64;
        let check: Vec<_> = self.history.iter().rev().take(50).collect();

        for prev_output in &check {
            let dist = (prev_output.diversity_index - self.compute_diversity(state)).abs();
            total_distance += dist;
        }

        let avg_distance = total_distance / check.len() as f64;
        avg_distance.min(1.0)
    }

    fn check_transition(&mut self, creative_state: f64, divergence: f64) {
        match self.state {
            CreativePhase::Ordered => {
                if divergence < self.chaos_threshold {
                    // Too ordered — inject chaos!
                    self.state = CreativePhase::Exploratory;
                    self.exploration_count = 0;
                }
            }
            CreativePhase::Exploratory => {
                self.exploration_count += 1;

                if self.exploration_count >= self.max_exploration
                    || creative_state > self.order_threshold
                {
                    // Max exploration or found good enough
                    self.state = CreativePhase::Ordered;
                }
            }
        }
    }

    /// Get current phase
    pub fn phase(&self) -> &'static str {
        match self.state {
            CreativePhase::Ordered => "ordered",
            CreativePhase::Exploratory => "exploratory",
        }
    }

    /// Get status summary
    pub fn status(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("phase".to_string(), self.phase().to_string());
        map.insert("exploration_count".to_string(), self.exploration_count.to_string());
        map.insert("best_value".to_string(), format!("{:.3}", self.best_value));
        map
    }
}

/// Novelty detector using k-NN distance
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NoveltyDetector {
    history: Vec<Vec<f64>>,
    max_history: usize,
    k: usize,
    baseline: f64,
}

impl Default for NoveltyDetector {
    fn default() -> Self {
        Self::new(1000, 5)
    }
}

impl NoveltyDetector {
    pub fn new(max_history: usize, k: usize) -> Self {
        Self {
            history: Vec::with_capacity(max_history),
            max_history,
            k,
            baseline: 0.0,
        }
    }

    /// Add a state to history
    pub fn add(&mut self, state: &[f64]) {
        let norm = Self::normalize(state);
        self.history.push(norm);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Compute novelty score
    pub fn novelty(&self, state: &[f64]) -> f64 {
        if self.history.len() < self.k {
            return 1.0;
        }

        let norm = Self::normalize(state);
        let history: Vec<_> = self.history.iter().collect();

        // Compute distances to all history
        let mut distances: Vec<f64> = history
            .iter()
            .map(|h| Self::euclidean(&norm, h))
            .collect();

        // Sort and get k nearest
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let k_nearest: f64 = distances[..self.k].iter().sum::<f64>() / self.k as f64;

        // Normalize
        (k_nearest / (self.k as f64)).min(1.0)
    }

    fn normalize(v: &[f64]) -> Vec<f64> {
        let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-10);
        v.iter().map(|x| x / norm).collect()
    }

    fn euclidean(a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creative_oscillator() {
        let mut osc = CreativeOscillator::new();
        let state = vec![0.5; 100];
        let output = osc.step(&state, 0.7, 0.3);

        assert!(output.creative_state >= 0.0);
        assert!(output.oscillation_phase >= 0.0);
    }

    #[test]
    fn test_novelty_detector() {
        let mut detector = NoveltyDetector::new(100, 5);
        let state1 = vec![1.0, 0.0, 0.0];
        let state2 = vec![0.0, 1.0, 0.0];

        detector.add(&state1);
        let novelty = detector.novelty(&state2);
        assert!(novelty > 0.0);
    }
}
