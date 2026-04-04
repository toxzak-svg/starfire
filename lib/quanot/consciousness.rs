//! Consciousness — Φ proxy, RQA, AIS, Global Workspace
//!
//! Ported from Python `consciousness.py`

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Consciousness metrics computed from reservoir state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsciousnessMetrics {
    /// Integrated Information proxy Φ (0-1)
    pub phi: f64,
    /// Information integration
    pub integration: f64,
    /// Information differentiation
    pub differentiation: f64,
    /// State entropy
    pub entropy: f64,
    /// Global Workspace broadcast readiness (0-1)
    pub workspace_broadcast: f64,
}

/// Tracks consciousness over time
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConsciousnessTracker {
    state_dim: usize,
    history: Vec<Vec<f64>>,
    rqa_history: VecDeque<ConsciousnessMetrics>,
    max_history: usize,
}

impl Default for ConsciousnessTracker {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl ConsciousnessTracker {
    pub fn new(state_dim: usize) -> Self {
        Self {
            state_dim,
            history: Vec::with_capacity(10000),
            rqa_history: VecDeque::with_capacity(100),
            max_history: 10000,
        }
    }

    /// Compute consciousness metrics from current state and history
    pub fn compute(&mut self, state: &[f64], state_history: &[Vec<f64>]) -> ConsciousnessMetrics {
        // Update history
        self.history.push(state.to_vec());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // Compute metrics
        let phi = self.compute_phi(state, state_history);
        let integration = self.compute_integration(state, state_history);
        let differentiation = self.compute_differentiation(state);
        let entropy = self.compute_entropy(state);
        let workspace_broadcast = self.compute_workspace_readiness(state);

        let metrics = ConsciousnessMetrics {
            phi,
            integration,
            differentiation,
            entropy,
            workspace_broadcast,
        };

        // Update RQA history
        self.rqa_history.push_back(metrics.clone());
        if self.rqa_history.len() > 100 {
            self.rqa_history.pop_front();
        }

        metrics
    }

    /// Φ proxy: ratio of integration to differentiation (IIT-inspired)
    fn compute_phi(&self, state: &[f64], _history: &[Vec<f64>]) -> f64 {
        let integration = self.compute_integration(state, _history);
        let differentiation = self.compute_differentiation(state);

        // Φ ≈ integration / (differentiation + ε)
        let phi = integration / (differentiation + 0.01);

        // Normalize to 0-1 range (rough approximation)
        (phi * 10.0).clamp(0.0, 1.0)
    }

    /// Information integration: how interconnected are the components?
    #[allow(dead_code)]
fn compute_integration_from_history(&self, state: &[Vec<f64>], history: &[Vec<f64>]) -> f64 {
        if history.len() < 10 || state.is_empty() {
            return 0.0;
        }

        // Measure: variance of mean activity (low variance = high integration)
        let recent: Vec<_> = history.iter().rev().take(100).collect();
        let means: Vec<f64> = recent.iter().map(|s| {
            s.iter().sum::<f64>() / s.len() as f64
        }).collect();

        let mean_of_means = means.iter().sum::<f64>() / means.len() as f64;
        let variance = means.iter()
            .map(|m| (m - mean_of_means).powi(2))
            .sum::<f64>() / means.len() as f64;

        // Low variance = high integration
        (1.0 / (1.0 + variance * 10.0)).min(1.0)
    }

    /// For single state
    fn compute_integration(&self, state: &[f64], _history: &[Vec<f64>]) -> f64 {
        if state.is_empty() {
            return 0.0;
        }
        // Use local variance as proxy
        let mean = state.iter().sum::<f64>() / state.len() as f64;
        let variance = state.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / state.len() as f64;

        // Sparsity = low variance components = high differentiation
        let sparse_count = state.iter().filter(|&&x| x.abs() < 0.1).count() as f64;
        let sparsity = sparse_count / state.len() as f64;

        // High differentiation = many independent components
        // Approximate via inverse of mean correlation
        (1.0 - variance.min(1.0)) * sparsity
    }

    /// Information differentiation: how many independent components?
    fn compute_differentiation(&self, state: &[f64]) -> f64 {
        if state.is_empty() {
            return 0.0;
        }

        // Sparsity = high differentiation
        let zero_count = state.iter().filter(|&&x| x.abs() < 0.1).count() as f64;
        let sparsity = zero_count / state.len() as f64;

        // Entropy of activation distribution
        let entropy = self.compute_entropy(state);

        // Differentiation = sparse AND high entropy
        sparsity * entropy
    }

    /// Compute entropy of state
    fn compute_entropy(&self, state: &[f64]) -> f64 {
        if state.is_empty() {
            return 0.0;
        }

        // Discretize into bins
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

        // Normalize (max entropy = ln(n_bins))
        let max_entropy = (n_bins as f64).ln();
        (entropy / max_entropy).min(1.0)
    }

    /// Global Workspace readiness: how many modules are active?
    fn compute_workspace_readiness(&self, state: &[f64]) -> f64 {
        if state.is_empty() {
            return 0.0;
        }

        // Count "active" components (above threshold)
        let threshold = 0.3;
        let active_count = state.iter().filter(|&&x| x.abs() > threshold).count() as f64;
        let active_ratio = active_count / state.len() as f64;

        // Also consider variance across components
        let mean = state.iter().sum::<f64>() / state.len() as f64;
        let variance = state.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / state.len() as f64;

        // High readiness = many active, diverse components
        let diversity = (variance * 10.0).min(1.0);

        (active_ratio + diversity) / 2.0
    }

    /// Reset tracker
    pub fn reset(&mut self) {
        self.history.clear();
        self.rqa_history.clear();
    }

    /// Get recent consciousness trend
    pub fn get_trend(&self) -> &'static str {
        if self.rqa_history.len() < 10 {
            return "unknown";
        }

        let recent: Vec<_> = self.rqa_history.iter().rev().take(10).collect();
        let avg_phi: f64 = recent.iter().map(|m| m.phi).sum::<f64>() / 10.0;

        if avg_phi < 0.3 {
            "low_consciousness"
        } else if avg_phi < 0.6 {
            "moderate_consciousness"
        } else {
            "high_consciousness"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consciousness_tracker() {
        let mut tracker = ConsciousnessTracker::new(100);
        let state = vec![0.5; 100];
        let history = vec![state.clone(); 100];
        let metrics = tracker.compute(&state, &history);

        assert!(metrics.phi >= 0.0);
        assert!(metrics.phi <= 1.0);
    }

    #[test]
    fn test_entropy() {
        let tracker = ConsciousnessTracker::new(10);
        let state = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let entropy = tracker.compute_entropy(&state);
        assert!(entropy > 0.0);
    }
}
