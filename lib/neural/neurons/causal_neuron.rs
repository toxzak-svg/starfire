//! Causal Neuron — Causal Engine as a Neural Module
//!
//! Wraps the causal discovery and reasoning engine as a neuron that
//! performs causal inference over inputs.

use crate::neural::{Activation, Neuron, NeuronConfig, NeuronId, NeuralSignal, NeuronState};
use crate::causal::{CausalEngine, CausalEdgeId};
use std::collections::HashMap;

const CAUSAL_INPUT_DIM: usize = 64;
const CAUSAL_HIDDEN_DIM: usize = 64;
const CAUSAL_OUTPUT_DIM: usize = 64;

pub struct CausalNeuron {
    id: NeuronId,
    config: NeuronConfig,
    engine: CausalEngine,
    working_memory: Vec<(String, String)>,
    state: NeuronState,
}

impl CausalNeuron {
    pub fn new(id: NeuronId) -> Self {
        Self {
            id,
            config: NeuronConfig {
                input_dim: CAUSAL_INPUT_DIM,
                hidden_dim: CAUSAL_HIDDEN_DIM,
                output_dim: CAUSAL_OUTPUT_DIM,
                activation: Activation::Tanh,
                dropout: 0.0,
                trainable: true,
            },
            engine: CausalEngine::new(),
            working_memory: Vec::new(),
            state: NeuronState::default(),
        }
    }

    fn encode_concept(&self, concept: &str) -> Vec<f32> {
        let mut vec = vec![0.0; CAUSAL_INPUT_DIM];
        for (i, c) in concept.bytes().enumerate().take(CAUSAL_INPUT_DIM) {
            vec[i] = (c as f32) / 255.0;
        }
        vec
    }

    fn decode_vector(&self, vec: &[f32]) -> String {
        vec.iter()
            .map(|&v| (v * 255.0) as u8 as char)
            .collect::<String>()
            .trim_matches('\0')
            .to_string()
    }

    pub fn add_causal_edge(&mut self, cause: &str, effect: &str, confidence: f64) -> CausalEdgeId {
        self.engine.add_edge(cause, effect, confidence, None)
    }

    pub fn get_causes(&self, effect: &str) -> Vec<String> {
        self.engine.get_causes_of(effect).iter().map(|e| e.cause.clone()).collect()
    }

    pub fn get_effects(&self, cause: &str) -> Vec<String> {
        self.engine.get_effects_of(cause).iter().map(|e| e.effect.clone()).collect()
    }
}

impl Neuron for CausalNeuron {
    fn id(&self) -> NeuronId {
        self.id.clone()
    }

    fn config(&self) -> &NeuronConfig {
        &self.config
    }

    fn forward(&mut self, input: &NeuralSignal) -> NeuralSignal {
        self.state.input = Some(input.vector.clone());

        let vector_sum: f32 = input.vector.iter().sum();
        let vector_mean = vector_sum / input.vector.len() as f32;

        let query = if vector_mean > 0.5 {
            "fire"
        } else if vector_mean > 0.0 {
            "water"
        } else {
            "sun"
        };

        let causes = self.get_causes(query);
        let _effects = self.get_effects(query);

        let mut output_vec = vec![0.0; CAUSAL_OUTPUT_DIM];
        for (i, cause) in causes.iter().take(8).enumerate() {
            let encoded = self.encode_concept(cause);
            for (j, &v) in encoded.iter().take(8).enumerate() {
                output_vec[i * 8 + j] = v;
            }
        }

        self.state.output = Some(output_vec.clone());

        NeuralSignal::new(
            self.id.clone(),
            output_vec,
        )
        .with_attention(input.attention)
        .with_confidence(0.75)
        .with_novelty(0.4)
    }

    fn reset(&mut self) {
        self.working_memory.clear();
        self.state = NeuronState::default();
    }

    fn state(&self) -> &NeuronState {
        &self.state
    }

    fn param_count(&self) -> usize {
        CAUSAL_HIDDEN_DIM * (CAUSAL_INPUT_DIM + CAUSAL_OUTPUT_DIM)
    }

    fn get_weights(&self) -> HashMap<String, Vec<f32>> {
        HashMap::new()
    }

    fn set_weights(&mut self, _weights: &HashMap<String, Vec<f32>>) {}

    fn clone_box(&self) -> Box<dyn Neuron> {
        Box::new(Self {
            id: self.id.clone(),
            config: self.config.clone(),
            engine: CausalEngine::new(),
            working_memory: Vec::new(),
            state: NeuronState::default(),
        })
    }
}

impl Clone for CausalNeuron {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            config: self.config.clone(),
            engine: CausalEngine::new(),
            working_memory: Vec::new(),
            state: NeuronState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_neuron_creation() {
        let id = NeuronId::new("causal");
        let neuron = CausalNeuron::new(id.clone());
        assert_eq!(neuron.id(), id);
    }

    #[test]
    fn test_add_causal_edge() {
        let id = NeuronId::new("causal");
        let mut neuron = CausalNeuron::new(id);
        let edge_id = neuron.add_causal_edge("fire", "heat", 0.9);
        assert_eq!(edge_id, CausalEdgeId::new(0));
    }

    #[test]
    fn test_causal_forward() {
        let id = NeuronId::new("causal");
        let mut neuron = CausalNeuron::new(id.clone());
        neuron.add_causal_edge("fire", "heat", 0.9);

        let input = NeuralSignal::new(id.clone(), vec![0.6; 64]);
        let output = neuron.forward(&input);

        assert_eq!(output.vector.len(), CAUSAL_OUTPUT_DIM);
    }
}
