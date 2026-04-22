//! Neuron Trait and Neural Signal Types
//!
//! Defines the core interface for neural modules and the signal types
//! used for communication between neurons.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Unique identifier for a neuron
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NeuronId(pub String);

impl NeuronId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl std::fmt::Display for NeuronId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Signals passed between neurons in the network
#[derive(Debug, Clone)]
pub struct NeuralSignal {
    /// Main payload vector (typically 32-128 dimensions)
    pub vector: Vec<f32>,
    /// How much other neurons should attend to this signal (0-1)
    pub attention: f32,
    /// Confidence in the signal's correctness (0-1)
    pub confidence: f32,
    /// How novel/unexpected this signal is (0-1)
    pub novelty: f32,
    /// Which neuron produced this signal
    pub source: NeuronId,
    /// Timestamp for temporal ordering
    pub timestamp: i64,
}

impl NeuralSignal {
    pub fn new(source: NeuronId, vector: Vec<f32>) -> Self {
        Self {
            vector,
            attention: 1.0,
            confidence: 0.5,
            novelty: 0.5,
            source,
            timestamp: crate::now_timestamp(),
        }
    }

    pub fn with_attention(mut self, attention: f32) -> Self {
        self.attention = attention.clamp(0.0, 1.0);
        self
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_novelty(mut self, novelty: f32) -> Self {
        self.novelty = novelty.clamp(0.0, 1.0);
        self
    }

    pub fn dim(&self) -> usize {
        self.vector.len()
    }
}

/// Type of connection between neurons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Positive weight — amplifies signal
    Excitatory,
    /// Negative weight — suppresses signal
    Inhibitory,
    /// Modulates gain (attention/gating)
    Modulatory,
    /// Feedback to same neuron (recurrent)
    Recurrent,
}

/// Activation function for neurons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activation {
    /// Sigmoid: smooth 0-1 output
    Sigmoid,
    /// Tanh: smooth -1 to 1 output
    Tanh,
    /// ReLU: fast, sparse
    ReLU,
    /// Softmax: for attention weights
    Softmax,
    /// No activation (linear)
    Identity,
}

impl Activation {
    pub fn apply(&self, x: f32) -> f32 {
        match self {
            Activation::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            Activation::Tanh => x.tanh(),
            Activation::ReLU => x.max(0.0),
            Activation::Softmax => unimplemented!("Softmax is applied per-vector, not per-element"),
            Activation::Identity => x,
        }
    }

    pub fn apply_vec(&self, v: &mut [f32]) {
        match self {
            Activation::Softmax => {
                let max_val = v.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                let exp_sum: f32 = v.iter().map(|x| (x - max_val).exp()).sum();
                for x in v.iter_mut() {
                    *x = ((*x - max_val).exp()) / exp_sum;
                }
            }
            _ => {
                for x in v.iter_mut() {
                    *x = self.apply(*x);
                }
            }
        }
    }
}

/// Configuration for a neuron
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronConfig {
    /// Input dimension
    pub input_dim: usize,
    /// Hidden dimension (if applicable)
    pub hidden_dim: usize,
    /// Output dimension
    pub output_dim: usize,
    /// Activation function
    pub activation: Activation,
    /// Dropout rate (0-1)
    pub dropout: f32,
    /// Whether this neuron is trainable
    pub trainable: bool,
}

impl Default for NeuronConfig {
    fn default() -> Self {
        Self {
            input_dim: 64,
            hidden_dim: 64,
            output_dim: 64,
            activation: Activation::Tanh,
            dropout: 0.0,
            trainable: true,
        }
    }
}

/// State maintained by a neuron during forward/backward pass
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronState {
    /// Cached input (for backward pass)
    pub input: Option<Vec<f32>>,
    /// Cached output (for backward pass)
    pub output: Option<Vec<f32>>,
    /// Gradients accumulated during backward pass
    pub gradients: HashMap<NeuronId, Vec<f32>>,
    /// Activation trace (for eligibility traces)
    pub activation_trace: Vec<f32>,
}

impl Default for NeuronState {
    fn default() -> Self {
        Self {
            input: None,
            output: None,
            gradients: HashMap::new(),
            activation_trace: Vec::new(),
        }
    }
}

/// Core trait implemented by all neurons in the network
pub trait Neuron: Send + Sync {
    /// Get this neuron's unique ID
    fn id(&self) -> NeuronId;

    /// Get configuration
    fn config(&self) -> &NeuronConfig;

    /// Forward pass: process input signal and produce output
    fn forward(&mut self, input: &NeuralSignal) -> NeuralSignal;

    /// Reset internal state (between forward passes)
    fn reset(&mut self);

    /// Get current internal state for inspection
    fn state(&self) -> &NeuronState;

    /// Number of parameters in this neuron
    fn param_count(&self) -> usize;

    /// Get all weights (for inspection/serialization)
    fn get_weights(&self) -> HashMap<String, Vec<f32>>;

    /// Set weights (for deserialization/training)
    fn set_weights(&mut self, weights: &HashMap<String, Vec<f32>>);

    /// Clone the neuron (for creating copies during training)
    fn clone_box(&self) -> Box<dyn Neuron>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_signal() {
        let signal = NeuralSignal::new(NeuronId::new("test"), vec![0.1, 0.2, 0.3]);
        assert_eq!(signal.dim(), 3);
        assert_eq!(signal.attention, 1.0);
        assert_eq!(signal.confidence, 0.5);
    }

    #[test]
    fn test_activation_sigmoid() {
        let act = Activation::Sigmoid;
        assert!((act.apply(0.0) - 0.5).abs() < 1e-6);
        assert!((act.apply(10.0) - 1.0).abs() < 1e-4); // sigmoid(10) ≈ 0.99995, not exact 1.0
        assert!((act.apply(-10.0)).abs() < 1e-4);
    }

    #[test]
    fn test_activation_tanh() {
        let act = Activation::Tanh;
        assert!((act.apply(0.0)).abs() < 1e-6);
        assert!((act.apply(1.0) - 0.761594_f32).abs() < 1e-4);
    }

    #[test]
    fn test_activation_relu() {
        let act = Activation::ReLU;
        assert!(act.apply(-1.0) < 1e-6);
        assert_eq!(act.apply(1.0), 1.0);
    }
}
