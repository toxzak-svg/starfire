//! Knowledge Neuron — Knowledge Graph as a Neural Module
//!
//! Wraps the knowledge graph as a neuron that can retrieve and update
//! knowledge based on incoming signals.

use super::super::{Activation, Neuron, NeuronConfig, NeuronId, NeuralSignal, NeuronState};
use crate::reasoning::knowledge::KnowledgeGraph;
use std::collections::HashMap;

/// Knowledge neuron dimensions
const KNOWLEDGE_INPUT_DIM: usize = 64;
const KNOWLEDGE_HIDDEN_DIM: usize = 64;
const KNOWLEDGE_OUTPUT_DIM: usize = 64;

/// Knowledge neuron that wraps the knowledge graph
pub struct KnowledgeNeuron {
    id: NeuronId,
    config: NeuronConfig,
    /// The underlying knowledge graph
    knowledge: KnowledgeGraph,
    /// Query embedding (from current input)
    query_emb: Vec<f32>,
    /// Retrieved fact embeddings
    fact_embs: Vec<Vec<f32>>,
    /// Attention weights over facts
    attention_weights: Vec<f32>,
    /// Working memory
    working_memory: Vec<String>,
    /// Cached state
    state: NeuronState,
}

impl KnowledgeNeuron {
    pub fn new(id: NeuronId) -> Self {
        Self {
            id,
            config: NeuronConfig {
                input_dim: KNOWLEDGE_INPUT_DIM,
                hidden_dim: KNOWLEDGE_HIDDEN_DIM,
                output_dim: KNOWLEDGE_OUTPUT_DIM,
                activation: Activation::Tanh,
                dropout: 0.0,
                trainable: true,
            },
            knowledge: KnowledgeGraph::new(),
            query_emb: Vec::new(),
            fact_embs: Vec::new(),
            attention_weights: Vec::new(),
            working_memory: Vec::new(),
            state: NeuronState::default(),
        }
    }

    /// Add a fact to the knowledge graph
    pub fn add_fact(&mut self, subject: &str, verb: &str, object: &str, confidence: f64) {
        self.knowledge.ingest_fact(subject, verb, object, confidence);
    }

    /// Retrieve facts relevant to a query
    pub fn retrieve(&mut self, query: &str) -> Vec<String> {
        let facts = self.knowledge.get_facts_about(query);
        facts
    }

    /// Encode a fact as a vector for downstream processing
    fn encode_fact(&self, fact: &str) -> Vec<f32> {
        let mut vec = vec![0.0; KNOWLEDGE_OUTPUT_DIM];

        // Simple encoding: hash characters and normalize
        let mut hash: f32 = 0.0;
        for (i, c) in fact.chars().enumerate() {
            hash += c as usize as f32 * (i as f32 + 1.0);
        }

        // Spread hash across vector
        for (i, v) in vec.iter_mut().enumerate() {
            *v = ((hash * (i + 1) as f32 * 0.1).sin() + 1.0) / 2.0; // Map to 0-1
        }

        vec
    }

    /// Compute attention over retrieved facts
    fn compute_attention(&mut self, query: &[f32]) {
        self.attention_weights.clear();

        for fact_emb in &self.fact_embs {
            // Cosine similarity between query and fact
            let dot: f32 = query.iter().zip(fact_emb.iter()).map(|(a, b)| a * b).sum();
            let mag_query = query.iter().map(|x| x * x).sum::<f32>().sqrt();
            let mag_fact = fact_emb.iter().map(|x| x * x).sum::<f32>().sqrt();

            let similarity = if mag_query > 0.0 && mag_fact > 0.0 {
                dot / (mag_query * mag_fact)
            } else {
                0.0
            };

            self.attention_weights.push((similarity + 1.0) / 2.0); // Map to 0-1
        }

        // Softmax normalize
        if !self.attention_weights.is_empty() {
            let max_w = self.attention_weights.iter().cloned().fold(f32::MIN, f32::max);
            let exp_sum: f32 = self.attention_weights.iter()
                .map(|w| (w - max_w).exp())
                .sum();

            for w in &mut self.attention_weights {
                *w = ((*w - max_w).exp()) / exp_sum;
            }
        }
    }

    /// Compute weighted sum of fact embeddings
    fn weighted_sum(&self) -> Vec<f32> {
        let mut output = vec![0.0; KNOWLEDGE_OUTPUT_DIM];

        for (fact_emb, &weight) in self.fact_embs.iter().zip(self.attention_weights.iter()) {
            for (i, val) in fact_emb.iter().enumerate() {
                output[i] += val * weight;
            }
        }

        output
    }
}

impl Neuron for KnowledgeNeuron {
    fn id(&self) -> NeuronId {
        self.id.clone()
    }

    fn config(&self) -> &NeuronConfig {
        &self.config
    }

    fn forward(&mut self, input: &NeuralSignal) -> NeuralSignal {
        // Input is a query embedding
        self.query_emb = input.vector.clone();

        // Retrieve relevant facts
        // In full implementation, would use embedding to search
        // For now, use a simple approach

        // Get entities and their facts
        let entities = self.knowledge.entities();
        let mut all_facts: Vec<String> = Vec::new();

        for entity in entities.iter().take(10) {
            let facts = self.knowledge.get_facts_about(entity);
            all_facts.extend(facts);
        }

        // Encode facts
        self.fact_embs = all_facts.iter()
            .map(|f| self.encode_fact(f))
            .collect();

        // Compute attention (use clone to avoid borrow conflict)
        let query_clone = self.query_emb.clone();
        self.compute_attention(&query_clone);

        // Compute output
        let output_vec = self.weighted_sum();

        // Store state
        self.state.input = Some(input.vector.clone());
        self.state.output = Some(output_vec.clone());

        NeuralSignal::new(
            self.id.clone(),
            output_vec,
        )
        .with_attention(1.0) // Knowledge is always attended to
        .with_confidence(0.8) // Knowledge retrieval is fairly reliable
        .with_novelty(0.3)   // Known facts have low novelty
    }

    fn reset(&mut self) {
        self.query_emb.clear();
        self.fact_embs.clear();
        self.attention_weights.clear();
        self.working_memory.clear();
        self.state = NeuronState::default();
    }

    fn state(&self) -> &NeuronState {
        &self.state
    }

    fn param_count(&self) -> usize {
        // Attention weights + any learned representations
        KNOWLEDGE_HIDDEN_DIM * KNOWLEDGE_OUTPUT_DIM
    }

    fn get_weights(&self) -> HashMap<String, Vec<f32>> {
        let mut weights = HashMap::new();
        weights.insert("attention".to_string(), self.attention_weights.clone());
        weights
    }

    fn set_weights(&mut self, _weights: &HashMap<String, Vec<f32>>) {
        // Would apply learned attention weights
    }

    fn clone_box(&self) -> Box<dyn Neuron> {
        Box::new(Self {
            id: self.id.clone(),
            config: self.config.clone(),
            knowledge: self.knowledge.clone(),
            query_emb: Vec::new(),
            fact_embs: Vec::new(),
            attention_weights: Vec::new(),
            working_memory: Vec::new(),
            state: NeuronState::default(),
        })
    }
}

impl Clone for KnowledgeNeuron {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            config: self.config.clone(),
            knowledge: self.knowledge.clone(),
            query_emb: Vec::new(),
            fact_embs: Vec::new(),
            attention_weights: Vec::new(),
            working_memory: Vec::new(),
            state: NeuronState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_neuron_creation() {
        let id = NeuronId::new("knowledge");
        let neuron = KnowledgeNeuron::new(id.clone());

        assert_eq!(neuron.id(), id);
    }

    #[test]
    fn test_add_and_retrieve() {
        let id = NeuronId::new("knowledge");
        let mut neuron = KnowledgeNeuron::new(id);

        neuron.add_fact("fire", "causes", "heat", 0.9);

        let facts = neuron.retrieve("fire");
        assert!(!facts.is_empty());
    }

    #[test]
    fn test_encode_fact() {
        let id = NeuronId::new("knowledge");
        let neuron = KnowledgeNeuron::new(id.clone());

        let vec = neuron.encode_fact("fire causes heat");
        assert_eq!(vec.len(), KNOWLEDGE_OUTPUT_DIM);
    }
}
