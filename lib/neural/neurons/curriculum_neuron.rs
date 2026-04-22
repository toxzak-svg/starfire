//! Curriculum Neuron — Curriculum Engine as a Neural Module
//!
//! Wraps the self-directed learning curriculum as a neuron that
//! identifies knowledge gaps and generates learning tasks.

use crate::neural::{Activation, Neuron, NeuronConfig, NeuronId, NeuralSignal, NeuronState};
use crate::curriculum::{CurriculumEngine, KnowledgeGap, LearningTask, GapId};
use std::collections::HashMap;

const CURRICULUM_INPUT_DIM: usize = 64;
const CURRICULUM_HIDDEN_DIM: usize = 64;
const CURRICULUM_OUTPUT_DIM: usize = 64;

pub struct CurriculumNeuron {
    id: NeuronId,
    config: NeuronConfig,
    engine: CurriculumEngine,
    state: NeuronState,
}

impl CurriculumNeuron {
    pub fn new(id: NeuronId) -> Self {
        Self {
            id,
            config: NeuronConfig {
                input_dim: CURRICULUM_INPUT_DIM,
                hidden_dim: CURRICULUM_HIDDEN_DIM,
                output_dim: CURRICULUM_OUTPUT_DIM,
                activation: Activation::ReLU,
                dropout: 0.0,
                trainable: true,
            },
            engine: CurriculumEngine::new(),
            state: NeuronState::default(),
        }
    }

    fn encode_topic(&self, topic: &str) -> Vec<f32> {
        let mut vec = vec![0.0; CURRICULUM_INPUT_DIM];
        for (i, c) in topic.bytes().enumerate().take(CURRICULUM_INPUT_DIM) {
            vec[i] = (c as f32) / 255.0;
        }
        vec
    }

    pub fn discover_gaps(&mut self, context: &str) {
        self.engine.discover_gaps(context);
    }

    pub fn get_gaps(&self) -> Vec<String> {
        self.engine.top_gaps(4)
            .iter()
            .map(|g| g.topic.clone())
            .collect()
    }
}

impl Neuron for CurriculumNeuron {
    fn id(&self) -> NeuronId {
        self.id.clone()
    }

    fn config(&self) -> &NeuronConfig {
        &self.config
    }

    fn forward(&mut self, input: &NeuralSignal) -> NeuralSignal {
        self.state.input = Some(input.vector.clone());

        let context = "What is consciousness? How does awareness arise?";

        if self.engine.gap_count() == 0 {
            self.discover_gaps(context);
        }

        let gaps = self.get_gaps();

        let mut output_vec = vec![0.0; CURRICULUM_OUTPUT_DIM];
        for (i, gap_topic) in gaps.iter().enumerate() {
            let encoded = self.encode_topic(gap_topic);
            for (j, &v) in encoded.iter().take(16).enumerate() {
                output_vec[i * 16 + j] = v;
            }
        }

        self.state.output = Some(output_vec.clone());

        NeuralSignal::new(
            self.id.clone(),
            output_vec,
        )
        .with_attention(input.attention)
        .with_confidence(0.7)
        .with_novelty(0.4)
    }

    fn reset(&mut self) {
        self.state = NeuronState::default();
    }

    fn state(&self) -> &NeuronState {
        &self.state
    }

    fn param_count(&self) -> usize {
        CURRICULUM_HIDDEN_DIM * (CURRICULUM_INPUT_DIM + CURRICULUM_OUTPUT_DIM)
    }

    fn get_weights(&self) -> HashMap<String, Vec<f32>> {
        HashMap::new()
    }

    fn set_weights(&mut self, _weights: &HashMap<String, Vec<f32>>) {}

    fn clone_box(&self) -> Box<dyn Neuron> {
        Box::new(Self {
            id: self.id.clone(),
            config: self.config.clone(),
            engine: CurriculumEngine::new(),
            state: NeuronState::default(),
        })
    }
}

impl Clone for CurriculumNeuron {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            config: self.config.clone(),
            engine: CurriculumEngine::new(),
            state: NeuronState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curriculum_neuron_creation() {
        let id = NeuronId::new("curriculum");
        let neuron = CurriculumNeuron::new(id.clone());
        assert_eq!(neuron.id(), id);
    }

    #[test]
    fn test_curriculum_forward() {
        let id = NeuronId::new("curriculum");
        let mut neuron = CurriculumNeuron::new(id.clone());

        let input = NeuralSignal::new(id.clone(), vec![0.5; 64]);
        let output = neuron.forward(&input);

        assert_eq!(output.vector.len(), CURRICULUM_OUTPUT_DIM);
    }
}
