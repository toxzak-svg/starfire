//! FewShot Neuron — Few-Shot Learning as a Neural Module
//!
//! Wraps the rapid hypothesis formation system as a neuron that
//! learns patterns from few examples.

use crate::neural::{Activation, Neuron, NeuronConfig, NeuronId, NeuralSignal, NeuronState};
use crate::learning::{FewShotLearner, Example};
use std::collections::HashMap;

const FEWSHOT_INPUT_DIM: usize = 64;
const FEWSHOT_HIDDEN_DIM: usize = 64;
const FEWSHOT_OUTPUT_DIM: usize = 64;

pub struct FewShotNeuron {
    id: NeuronId,
    config: NeuronConfig,
    learner: FewShotLearner,
    state: NeuronState,
}

impl FewShotNeuron {
    pub fn new(id: NeuronId) -> Self {
        Self {
            id,
            config: NeuronConfig {
                input_dim: FEWSHOT_INPUT_DIM,
                hidden_dim: FEWSHOT_HIDDEN_DIM,
                output_dim: FEWSHOT_OUTPUT_DIM,
                activation: Activation::Tanh,
                dropout: 0.0,
                trainable: true,
            },
            learner: FewShotLearner::new(),
            state: NeuronState::default(),
        }
    }


    pub fn add_example(&mut self, input: &str, output: &str, domain: &str) {
        let example = Example::new(input, output, domain);
        self.learner.add_example(example);
    }
}

impl Neuron for FewShotNeuron {
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

        if self.learner.hypotheses().is_empty() {
            self.add_example("fire is hot", "heat + danger", "physics");
            self.add_example("water is cold", "liquid + essential", "physics");
        }

        let hypotheses: Vec<String> = self.learner.hypotheses()
            .iter()
            .map(|h| format!("{}({:.2})", h.pattern, h.confidence))
            .collect();

        let mut output_vec = vec![0.0; FEWSHOT_OUTPUT_DIM];
        let mut idx = 0;
        for hyp in hypotheses.iter().take(4) {
            for c in hyp.bytes().take(FEWSHOT_OUTPUT_DIM / 4) {
                output_vec[idx] = (c as f32) / 255.0;
                idx = (idx + 1) % FEWSHOT_OUTPUT_DIM;
            }
        }

        self.state.output = Some(output_vec.clone());

        NeuralSignal::new(
            self.id.clone(),
            output_vec,
        )
        .with_attention(input.attention * vector_mean)
        .with_confidence(0.7)
        .with_novelty(0.3)
    }

    fn reset(&mut self) {
        self.state = NeuronState::default();
    }

    fn state(&self) -> &NeuronState {
        &self.state
    }

    fn param_count(&self) -> usize {
        FEWSHOT_HIDDEN_DIM * (FEWSHOT_INPUT_DIM + FEWSHOT_OUTPUT_DIM)
    }

    fn get_weights(&self) -> HashMap<String, Vec<f32>> {
        HashMap::new()
    }

    fn set_weights(&mut self, _weights: &HashMap<String, Vec<f32>>) {}

    fn clone_box(&self) -> Box<dyn Neuron> {
        Box::new(Self {
            id: self.id.clone(),
            config: self.config.clone(),
            learner: FewShotLearner::new(),
            state: NeuronState::default(),
        })
    }
}

impl Clone for FewShotNeuron {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            config: self.config.clone(),
            learner: FewShotLearner::new(),
            state: NeuronState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fewshot_neuron_creation() {
        let id = NeuronId::new("fewshot");
        let neuron = FewShotNeuron::new(id.clone());
        assert_eq!(neuron.id(), id);
    }

    #[test]
    fn test_fewshot_forward() {
        let id = NeuronId::new("fewshot");
        let mut neuron = FewShotNeuron::new(id.clone());

        let input = NeuralSignal::new(id.clone(), vec![0.5; 64]);
        let output = neuron.forward(&input);

        assert_eq!(output.vector.len(), FEWSHOT_OUTPUT_DIM);
    }
}
