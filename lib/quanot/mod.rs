//! Quanot — Quantum Neural Dynamics System (Rust Port)
//!
//! A reservoir computing system with chaotic dynamics for cognitive modeling.
//! Ported from Python to Rust for direct integration with Starfire.
//!
//! # Architecture
//!
//! - **Reservoir**: Echo State Network with chaotic modulation
//! - **Chaos**: Lyapunov exponents, RQA, strange attractors
//! - **Consciousness**: Φ proxy, GWT, AIS metrics
//! - **Creativity**: Novelty detection, conceptual blending, evaluation
//! - **Quantum-Inspired**: Simulated quantum annealing solver
//!
//! # Usage
//!
//! ```
//! use star::quanot::Quanot;
//!
//! let mut quanot = Quanot::new(128, 1000);
//! let result = quanot.process("Hello, world!");
//! println!("Consciousness proxy: {}", result.consciousness_proxy);
//! ```

pub mod reservoir;
pub mod chaos;
pub mod consciousness;
pub mod creativity;
pub mod quantum_inspired;
pub mod encoder;

use serde::{Deserialize, Serialize};

/// Main Quanot orchestrator — processes text through the full pipeline
#[derive(Debug, Clone)]
pub struct Quanot {
    pub reservoir: reservoir::Reservoir,
    pub encoder: encoder::TextEncoder,
    pub state_history: Vec<Vec<f64>>,
    pub max_history: usize,
    pub consciousness_tracker: consciousness::ConsciousnessTracker,
    pub creative_oscillator: creativity::CreativeOscillator,
}

impl Default for Quanot {
    fn default() -> Self {
        Self::new(128, 1000)
    }
}

impl Quanot {
    /// Create a new Quanot processor
    ///
    /// `input_dim`: dimension of input vectors (from encoder)
    /// `reservoir_size`: number of reservoir neurons
    pub fn new(input_dim: usize, reservoir_size: usize) -> Self {
        Self {
            reservoir: reservoir::Reservoir::new(input_dim, reservoir_size),
            encoder: encoder::TextEncoder::new(input_dim),
            state_history: Vec::with_capacity(10000),
            max_history: 10000,
            consciousness_tracker: consciousness::ConsciousnessTracker::new(reservoir_size),
            creative_oscillator: creativity::CreativeOscillator::new(),
        }
    }

    /// Process text input through the full Quanot pipeline
    pub fn process(&mut self, input: &str) -> QuanotResult {
        // 1. Encode text to vector
        let encoded = self.encoder.encode(input);

        // 2. Step the reservoir
        let state = self.reservoir.step(&encoded);

        // 3. Update history
        self.state_history.push(state.clone());
        if self.state_history.len() > self.max_history {
            self.state_history.remove(0);
        }

        // 4. Compute consciousness metrics
        let consciousness = self.consciousness_tracker.compute(&state, &self.state_history);

        // 5. Compute novelty
        let novelty = self.compute_novelty(&state);

        // 6. Compute creativity
        let creativity = self.creative_oscillator.step(
            &state,
            consciousness.phi,
            novelty,
        );

        QuanotResult {
            reservoir_state: state,
            consciousness_proxy: consciousness.phi,
            novelty,
            creativity_scores: creativity,
            chaos_metrics: self.compute_chaos_metrics(),
        }
    }

    /// Compute novelty of current state vs history (cosine distance to nearest neighbor)
    fn compute_novelty(&self, state: &[f64]) -> f64 {
        if self.state_history.len() < 2 {
            return 1.0; // First state is maximally novel
        }

        let mut max_similarity = -1.0f64;

        // Check recent history (last 100 states for efficiency)
        let check_range = self.state_history.len().saturating_sub(100)..;
        for prev in self.state_history[check_range].iter() {
            let sim = cosine_similarity(state, prev);
            max_similarity = max_similarity.max(sim);
        }

        (1.0 - max_similarity).clamp(0.0, 1.0)
    }

    /// Compute chaos metrics from state history
    fn compute_chaos_metrics(&self) -> chaos::ChaosMetrics {
        let recent: Vec<_> = self.state_history.iter().rev().take(500).cloned().collect();
        if recent.len() < 10 {
            return chaos::ChaosMetrics::default();
        }

        let trajectory: Vec<_> = recent.into_iter().rev().collect();
        chaos::ChaosMetrics::from_trajectory(&trajectory)
    }

    /// Reset all state
    pub fn reset(&mut self) {
        self.reservoir.reset();
        self.state_history.clear();
        self.consciousness_tracker.reset();
    }

    /// Get current reservoir state
    pub fn get_state(&self) -> &[f64] {
        self.reservoir.get_state()
    }
}

/// Result of processing through Quanot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuanotResult {
    /// The reservoir state vector
    pub reservoir_state: Vec<f64>,
    /// Consciousness proxy ψ (0-1)
    pub consciousness_proxy: f64,
    /// Novelty score (0-1)
    pub novelty: f64,
    /// Creativity metrics
    pub creativity_scores: creativity::CreativityOutput,
    /// Chaos metrics
    pub chaos_metrics: chaos::ChaosMetrics,
}

/// Cosine similarity between two vectors
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = (norm_a.sqrt() * norm_b.sqrt()).max(1e-10);
    dot / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quanot_process() {
        let mut quanot = Quanot::new(128, 100);
        let result = quanot.process("Hello world");

        assert_eq!(result.reservoir_state.len(), 100);
        assert!(result.consciousness_proxy >= 0.0);
        assert!(result.novelty >= 0.0);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = &[1.0, 0.0, 0.0];
        let b = &[1.0, 0.0, 0.0];
        let c = &[0.0, 1.0, 0.0];

        assert!((cosine_similarity(a, b) - 1.0).abs() < 1e-6);
        assert!((cosine_similarity(a, c) - 0.0).abs() < 1e-6);
    }
}
