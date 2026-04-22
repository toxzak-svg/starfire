//! Layer Types for Neural Modules
//!
//! Different layer architectures that neurons can use internally.

use crate::neural::neuron::Activation;
use crate::neural::neuron::NeuronConfig;
use serde::{Deserialize, Serialize};

/// Type of layer computation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerType {
    /// Standard fully-connected
    Dense,
    /// Echo State Network (reservoir)
    Reservoir,
    /// LSTM-like with gates
    Gated,
    /// Attention-weighted
    Attention,
    /// Graph convolution
    GraphConv,
}

/// A layer within a neuron
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Layer type
    pub layer_type: LayerType,
    /// Input dimension
    pub input_dim: usize,
    /// Output dimension
    pub output_dim: usize,
    /// Weights (input -> output)
    pub weights: Vec<f32>,
    /// Biases
    pub biases: Vec<f32>,
    /// Activation function
    pub activation: Activation,
}

impl Layer {
    pub fn new(input_dim: usize, output_dim: usize, layer_type: LayerType, activation: Activation) -> Self {
        let weights = vec![0.0; input_dim * output_dim];
        let biases = vec![0.0; output_dim];

        Self {
            layer_type,
            input_dim,
            output_dim,
            weights,
            biases,
            activation,
        }
    }

    /// Initialize weights with Xavier/Glorot initialization
    pub fn xavier_init(&mut self) {
        let fan_in = self.input_dim as f32;
        let fan_out = self.output_dim as f32;
        let scale = (6.0 / (fan_in + fan_out)).sqrt();

        for w in self.weights.iter_mut() {
            *w = (rand::random::<f32>() * 2.0 - 1.0) * scale;
        }

        for b in self.biases.iter_mut() {
            *b = 0.0;
        }
    }

    /// Initialize as orthogonal (good for RNNs)
    pub fn orthogonal_init(&mut self) {
        // Simple orthogonal init for square matrices
        let min_dim = self.input_dim.min(self.output_dim);
        let mut mat = vec![0.0f32; min_dim * min_dim];

        for i in 0..min_dim {
            mat[i * min_dim + i] = 1.0;
        }

        // Copy into weights (simplified)
        for (i, w) in self.weights.iter_mut().enumerate() {
            if i < mat.len() {
                *w = mat[i];
            }
        }
    }

    /// Forward pass: compute output from input
    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0; self.output_dim];

        // y = Wx + b
        for j in 0..self.output_dim {
            for i in 0..self.input_dim {
                output[j] += self.weights[j * self.input_dim + i] * input[i];
            }
            output[j] += self.biases[j];
        }

        // Apply activation
        self.activation.apply_vec(&mut output);

        output
    }

    /// Get a weight at (output_idx, input_idx)
    pub fn get_weight(&self, out_idx: usize, in_idx: usize) -> f32 {
        if out_idx < self.output_dim && in_idx < self.input_dim {
            self.weights[out_idx * self.input_dim + in_idx]
        } else {
            0.0
        }
    }

    /// Set a weight at (output_idx, input_idx)
    pub fn set_weight(&mut self, out_idx: usize, in_idx: usize, value: f32) {
        if out_idx < self.output_dim && in_idx < self.input_dim {
            self.weights[out_idx * self.input_dim + in_idx] = value;
        }
    }
}

/// Dense (fully-connected) layer builder
pub fn dense(input_dim: usize, output_dim: usize, activation: Activation) -> Layer {
    Layer::new(input_dim, output_dim, LayerType::Dense, activation)
}

/// Reservoir (ESN) layer
pub fn reservoir(input_dim: usize, reservoir_dim: usize, output_dim: usize) -> Layer {
    Layer::new(input_dim, output_dim, LayerType::Reservoir, Activation::Tanh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dense_layer() {
        let mut layer = dense(2, 2, Activation::Sigmoid);
        layer.xavier_init();

        let input = vec![1.0, 0.0];
        let output = layer.forward(&input);

        assert_eq!(output.len(), 2);
    }

    #[test]
    fn test_weight_access() {
        let mut layer = dense(3, 2, Activation::Identity);
        layer.set_weight(1, 2, 0.5);

        assert_eq!(layer.get_weight(1, 2), 0.5);
        assert_eq!(layer.get_weight(0, 2), 0.0);
    }
}
