//! Quanot Neuron — Quanot as a Neural Module
//!
//! Wraps the existing Quanot reservoir computing system as a neuron
//! in the StarNet architecture.

use crate::neural::{Activation, Neuron, NeuronConfig, NeuronId, NeuralSignal, NeuronState};
use crate::quanot::{Quanot, QuanotResult};
use std::collections::HashMap;

/// Quanot wrapped as a neuron
pub struct QuanotNeuron {
    id: NeuronId,
    config: NeuronConfig,
    /// The underlying Quanot system
    quanot: Quanot,
    /// Cached state for training
    state: NeuronState,
    /// Pre-synaptic trace (for Hebbian learning)
    pre_trace: Vec<f32>,
    /// Post-synaptic trace
    post_trace: Vec<f32>,
}

impl QuanotNeuron {
    pub fn new(id: NeuronId, reservoir_size: usize) -> Self {
        Self {
            id,
            config: NeuronConfig {
                input_dim: 128,  // Text encoder output
                hidden_dim: reservoir_size,
                output_dim: 128, // QuanotResult encoded as vector
                activation: Activation::Tanh,
                dropout: 0.0,
                trainable: true,
            },
            quanot: Quanot::new(128, reservoir_size),
            state: NeuronState::default(),
            pre_trace: Vec::new(),
            post_trace: Vec::new(),
        }
    }

    /// Get the consciousness proxy from last processing
    pub fn consciousness_proxy(&self) -> f32 {
        // Access via a forward pass that caches this
        0.0 // Placeholder - would need to expose this from Quanot
    }

    /// Get novelty from last processing
    pub fn novelty(&self) -> f32 {
        0.0 // Placeholder
    }

    /// Encode QuanotResult as a vector for downstream neurons
    fn encode_result(&self, result: &QuanotResult) -> Vec<f32> {
        let mut vec: Vec<f32> = Vec::with_capacity(128);

        // Encode consciousness proxy (0-1) -> spread across first few dims
        let phi = result.consciousness_proxy as f32;
        for i in 0..8 {
            vec.push(phi * 2.0 - 1.0); // Map to -1,1 range
        }

        // Encode novelty
        let novelty = result.novelty as f32;
        for i in 0..8 {
            vec.push(novelty * 2.0 - 1.0);
        }

        // Encode creativity
        let cs = &result.creativity_scores;
        vec.push(cs.creative_state as f32 * 2.0 - 1.0);
        vec.push(cs.divergence_metric as f32 * 2.0 - 1.0);
        vec.push(cs.diversity_index as f32 * 2.0 - 1.0);
        vec.push(cs.originality_score as f32 * 2.0 - 1.0);

        // Encode chaos metrics
        let cm = &result.chaos_metrics;
        vec.push(cm.lyapunov_exponent as f32 * 2.0 - 1.0);
        vec.push(cm.recurrence as f32 * 2.0 - 1.0);
        vec.push(cm.determinism as f32 * 2.0 - 1.0);

        // Pad or truncate reservoir state to fixed size
        let reservoir = &result.reservoir_state;
        let target_len = 100; // Use subset of reservoir state
        for r in reservoir.iter().take(target_len) {
            vec.push(*r as f32);
        }
        while vec.len() < 128 {
            vec.push(0.0);
        }

        // Ensure exactly 128 dims
        vec.resize(128, 0.0);
        vec
    }
}

impl Neuron for QuanotNeuron {
    fn id(&self) -> NeuronId {
        self.id.clone()
    }

    fn config(&self) -> &NeuronConfig {
        &self.config
    }

    fn forward(&mut self, input: &NeuralSignal) -> NeuralSignal {
        // Input should be text - use Quanot to process
        // For now, use raw vector (in full impl, would decode text from signal)

        // Actually process through Quanot
        // Note: Quanot.process takes text, but we're receiving a NeuralSignal
        // In full implementation, the signal would carry text or we use the vector directly

        // Simplified: treat input vector as encoded text features
        // Full impl would decode NeuralSignal to extract text

        // For now, pass through with some transformation
        let mut output_vec = input.vector.clone();

        // Apply some internal dynamics (simplified ESN-like behavior)
        // In real implementation, would use self.quanot.process(text)

        // Update traces for Hebbian learning
        self.pre_trace = input.vector.clone();
        self.post_trace = output_vec.clone();

        // Store for backward pass
        self.state.input = Some(input.vector.clone());
        self.state.output = Some(output_vec.clone());

        NeuralSignal::new(
            self.id.clone(),
            output_vec,
        )
        .with_attention(input.attention)
        .with_confidence(0.7) // Quanot has moderate confidence
        .with_novelty(0.5)
    }

    fn reset(&mut self) {
        self.quanot.reset();
        self.state = NeuronState::default();
        self.pre_trace.clear();
        self.post_trace.clear();
    }

    fn state(&self) -> &NeuronState {
        &self.state
    }

    fn param_count(&self) -> usize {
        // Quanot has reservoir weights
        self.config.hidden_dim * self.config.input_dim +
        self.config.hidden_dim // Biases
    }

    fn get_weights(&self) -> HashMap<String, Vec<f32>> {
        let mut weights = HashMap::new();
        weights.insert("input_weights".to_string(), vec![0.0; 1024]); // Placeholder
        weights.insert("biases".to_string(), vec![0.0; 64]);
        weights
    }

    fn set_weights(&mut self, _weights: &HashMap<String, Vec<f32>>) {
        // In full implementation, would apply weights to Quanot's reservoir
    }

    fn clone_box(&self) -> Box<dyn Neuron> {
        Box::new(Self {
            id: self.id.clone(),
            config: self.config.clone(),
            quanot: Quanot::new(self.config.input_dim, self.config.hidden_dim),
            state: NeuronState::default(),
            pre_trace: Vec::new(),
            post_trace: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quanot_neuron_creation() {
        let id = NeuronId::new("quanot");
        let neuron = QuanotNeuron::new(id.clone(), 100);

        assert_eq!(neuron.id(), id);
        assert_eq!(neuron.param_count(), 100 * 128 + 100);
    }

    #[test]
    fn test_quanot_neuron_forward() {
        let id = NeuronId::new("quanot");
        let mut neuron = QuanotNeuron::new(id.clone(), 50);

        // Input must match input_dim (128)
        let input = NeuralSignal::new(id.clone(), vec![0.1; 128]);
        let output = neuron.forward(&input);

        // Output should preserve input dimensions (stub behavior)
        assert_eq!(output.vector.len(), 128);
    }
}
