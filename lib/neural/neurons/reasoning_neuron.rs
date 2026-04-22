//! Reasoning Neuron — Reasoning Engine as a Neural Module
//!
//! Wraps the symbolic reasoning engine as a neuron that performs
//! reasoning over knowledge and produces conclusions.

use crate::neural::{Activation, Neuron, NeuronConfig, NeuronId, NeuralSignal, NeuronState};
use crate::reasoning::ReasoningEngine;
use crate::persistence::{Memory, MemoryDomain, BeliefState};
use std::collections::HashMap;

/// Reasoning neuron dimensions
const REASONING_INPUT_DIM: usize = 64;
const REASONING_HIDDEN_DIM: usize = 96;
const REASONING_OUTPUT_DIM: usize = 64;

/// Reasoning neuron that performs symbolic reasoning
pub struct ReasoningNeuron {
    id: NeuronId,
    config: NeuronConfig,
    /// The underlying reasoning engine
    engine: ReasoningEngine,
    /// Working memory for current reasoning
    working_memory: Vec<String>,
    /// Current reasoning chain
    chain: Vec<String>,
    /// Confidence of current conclusion
    confidence: f32,
    /// Attention over input (from other neurons)
    input_attention: f32,
    /// Cached state
    state: NeuronState,
}

impl ReasoningNeuron {
    pub fn new(id: NeuronId) -> Self {
        Self {
            id,
            config: NeuronConfig {
                input_dim: REASONING_INPUT_DIM,
                hidden_dim: REASONING_HIDDEN_DIM,
                output_dim: REASONING_OUTPUT_DIM,
                activation: Activation::Tanh,
                dropout: 0.0,
                trainable: true,
            },
            engine: ReasoningEngine::new(),
            working_memory: Vec::new(),
            chain: Vec::new(),
            confidence: 0.5,
            input_attention: 1.0,
            state: NeuronState::default(),
        }
    }

    /// Add knowledge to the reasoning engine
    pub fn add_knowledge(&mut self, subject: &str, fact: &str) {
        self.engine.add_knowledge(subject, fact);
    }

    /// Perform reasoning on a query
    pub fn reason(&mut self, query: &str) -> String {
        let memories: Vec<Memory> = self.working_memory.iter()
            .map(|m| Memory::new_seeded(m, MemoryDomain::Episodic, 0.7))
            .collect();

        let result = self.engine.reason(query, &memories);

        // Store reasoning chain
        self.chain = result.reasoning_chain.clone();

        // Convert confidence
        self.confidence = match result.confidence {
            BeliefState::Knows => 0.9,
            BeliefState::Thinks => 0.7,
            BeliefState::Believes => 0.5,
            BeliefState::Suspects => 0.3,
            BeliefState::Unknown => 0.1,
        };

        result.answer.unwrap_or_else(|| "I don't know".to_string())
    }

    /// Encode reasoning output as a vector
    fn encode_reasoning(&self, answer: &str, chain: &[String]) -> Vec<f32> {
        let mut vec = vec![0.0; REASONING_OUTPUT_DIM];

        // Encode answer
        let mut ans_hash: f32 = 0.0;
        for (i, c) in answer.chars().enumerate() {
            ans_hash += c as usize as f32 * (i as f32 + 1.0);
        }

        // Spread across vector with confidence modulation
        for (i, v) in vec.iter_mut().enumerate() {
            let base = ((ans_hash * (i + 1) as f32 * 0.1).sin() + 1.0) / 2.0;
            *v = base * self.confidence;
        }

        // Incorporate chain length as additional signal
        let chain_signal = (chain.len() as f32 / 10.0).min(1.0);
        for v in vec.iter_mut().take(chain.len() % 10) {
            *v = (*v + chain_signal) / 2.0;
        }

        vec
    }

    /// Update working memory
    pub fn add_to_working_memory(&mut self, item: &str) {
        self.working_memory.push(item.to_string());
        if self.working_memory.len() > 10 {
            self.working_memory.remove(0);
        }
    }
}

impl Neuron for ReasoningNeuron {
    fn id(&self) -> NeuronId {
        self.id.clone()
    }

    fn config(&self) -> &NeuronConfig {
        &self.config
    }

    fn forward(&mut self, input: &NeuralSignal) -> NeuralSignal {
        self.input_attention = input.attention;

        // Decode input vector to extract query
        // In full implementation, would have learned representation
        // For now, use vector statistics as a pseudo-query

        let vector_sum: f32 = input.vector.iter().sum();
        let vector_mean = vector_sum / input.vector.len() as f32;

        // Map vector mean to a pseudo-query
        let query = if vector_mean > 0.5 {
            "What is consciousness?"
        } else if vector_mean > 0.0 {
            "How does reasoning work?"
        } else {
            "What are the limits of knowledge?"
        };

        // Perform reasoning
        let answer = self.reason(query);

        // Encode output
        let output_vec = self.encode_reasoning(&answer, &self.chain);

        // Store state
        self.state.input = Some(input.vector.clone());
        self.state.output = Some(output_vec.clone());

        NeuralSignal::new(
            self.id.clone(),
            output_vec,
        )
        .with_attention(input.attention * self.confidence)
        .with_confidence(self.confidence)
        .with_novelty(0.4) // Reasoning produces moderate novelty
    }

    fn reset(&mut self) {
        self.working_memory.clear();
        self.chain.clear();
        self.confidence = 0.5;
        self.input_attention = 1.0;
        self.state = NeuronState::default();
    }

    fn state(&self) -> &NeuronState {
        &self.state
    }

    fn param_count(&self) -> usize {
        REASONING_HIDDEN_DIM * (REASONING_INPUT_DIM + REASONING_OUTPUT_DIM)
    }

    fn get_weights(&self) -> HashMap<String, Vec<f32>> {
        let mut weights = HashMap::new();
        weights.insert("confidence".to_string(), vec![self.confidence]);
        weights
    }

    fn set_weights(&mut self, _weights: &HashMap<String, Vec<f32>>) {
        // Would apply learned weights
    }

    fn clone_box(&self) -> Box<dyn Neuron> {
        Box::new(Self {
            id: self.id.clone(),
            config: self.config.clone(),
            engine: ReasoningEngine::new(), // Fresh engine
            working_memory: Vec::new(),
            chain: Vec::new(),
            confidence: 0.5,
            input_attention: 1.0,
            state: NeuronState::default(),
        })
    }
}

impl Clone for ReasoningNeuron {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            config: self.config.clone(),
            engine: ReasoningEngine::new(),
            working_memory: Vec::new(),
            chain: Vec::new(),
            confidence: 0.5,
            input_attention: 1.0,
            state: NeuronState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reasoning_neuron_creation() {
        let id = NeuronId::new("reasoning");
        let neuron = ReasoningNeuron::new(id.clone());

        assert_eq!(neuron.id(), id);
    }

    #[test]
    fn test_add_knowledge() {
        let id = NeuronId::new("reasoning");
        let mut neuron = ReasoningNeuron::new(id);

        neuron.add_knowledge("fire", "hot");
        neuron.add_knowledge("water", "cold");
    }

    #[test]
    fn test_reasoning() {
        let id = NeuronId::new("reasoning");
        let mut neuron = ReasoningNeuron::new(id);

        neuron.add_knowledge("fire", "produces heat");

        let result = neuron.reason("What does fire produce?");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_forward_pass() {
        let id = NeuronId::new("reasoning");
        let mut neuron = ReasoningNeuron::new(id.clone());

        let input = NeuralSignal::new(id.clone(), vec![0.5; 64]);
        let output = neuron.forward(&input);

        assert_eq!(output.vector.len(), REASONING_OUTPUT_DIM);
        assert_eq!(output.source, id);
    }
}
