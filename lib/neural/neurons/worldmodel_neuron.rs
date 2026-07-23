//! WorldModel Neuron — World Model as a Neural Module
//!
//! Wraps the entity/relation world model as a neuron that maintains
//! a representation of entities and their causal relationships.

use crate::neural::{Activation, Neuron, NeuronConfig, NeuronId, NeuralSignal, NeuronState};
use crate::world_model::WorldModel;
use crate::quanot::QuanotResult;
use std::collections::HashMap;

const WORLDMODEL_INPUT_DIM: usize = 64;
const WORLDMODEL_HIDDEN_DIM: usize = 64;
const WORLDMODEL_OUTPUT_DIM: usize = 64;

pub struct WorldModelNeuron {
    id: NeuronId,
    config: NeuronConfig,
    world_model: WorldModel,
    entity_count: usize,
    state: NeuronState,
}

impl WorldModelNeuron {
    pub fn new(id: NeuronId) -> Self {
        Self {
            id,
            config: NeuronConfig {
                input_dim: WORLDMODEL_INPUT_DIM,
                hidden_dim: WORLDMODEL_HIDDEN_DIM,
                output_dim: WORLDMODEL_OUTPUT_DIM,
                activation: Activation::Tanh,
                dropout: 0.0,
                trainable: true,
            },
            world_model: WorldModel::new(),
            entity_count: 0,
            state: NeuronState::default(),
        }
    }

    fn encode_entity(&self, entity: &str) -> Vec<f32> {
        let mut vec = vec![0.0; WORLDMODEL_INPUT_DIM];
        for (i, c) in entity.bytes().enumerate().take(WORLDMODEL_INPUT_DIM) {
            vec[i] = (c as f32) / 255.0;
        }
        vec
    }

    pub fn update_from_perception(&mut self, perception: QuanotResult) {
        use crate::world_model::perception::{QuanotPerception, CreativityOutput};
        let perc = QuanotPerception {
            reservoir_state: vec![0.0; 64],
            consciousness_proxy: perception.consciousness_proxy as f64,
            novelty: perception.novelty as f64,
            creativity_scores: CreativityOutput::default(),
        };
        self.world_model.update_from_perception(perc);
        self.entity_count = self.world_model.entity_count();
    }
}

impl Neuron for WorldModelNeuron {
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

        self.entity_count = self.world_model.entity_count();

        let mut output_vec = vec![0.0; WORLDMODEL_OUTPUT_DIM];
        output_vec[0] = self.entity_count as f32 / 10.0;
        output_vec[1] = vector_mean;

        let entities = self.world_model.find_entities("");
        for (i, entity) in entities.iter().take(4).enumerate() {
            let encoded = self.encode_entity(&entity.name);
            for (j, &v) in encoded.iter().take(16).enumerate() {
                output_vec[i * 16 + j + 4] = v;
            }
        }

        self.state.output = Some(output_vec.clone());

        NeuralSignal::new(
            self.id.clone(),
            output_vec,
        )
        .with_attention(input.attention)
        .with_confidence(0.8)
        .with_novelty(0.2)
    }

    fn reset(&mut self) {
        self.state = NeuronState::default();
    }

    fn state(&self) -> &NeuronState {
        &self.state
    }

    fn param_count(&self) -> usize {
        WORLDMODEL_HIDDEN_DIM * (WORLDMODEL_INPUT_DIM + WORLDMODEL_OUTPUT_DIM)
    }

    fn get_weights(&self) -> HashMap<String, Vec<f32>> {
        HashMap::new()
    }

    fn set_weights(&mut self, _weights: &HashMap<String, Vec<f32>>) {}

    fn clone_box(&self) -> Box<dyn Neuron> {
        Box::new(Self {
            id: self.id.clone(),
            config: self.config.clone(),
            world_model: WorldModel::new(),
            entity_count: 0,
            state: NeuronState::default(),
        })
    }
}

impl Clone for WorldModelNeuron {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            config: self.config.clone(),
            world_model: WorldModel::new(),
            entity_count: 0,
            state: NeuronState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worldmodel_neuron_creation() {
        let id = NeuronId::new("worldmodel");
        let neuron = WorldModelNeuron::new(id.clone());
        assert_eq!(neuron.id(), id);
    }

    #[test]
    fn test_worldmodel_forward() {
        let id = NeuronId::new("worldmodel");
        let mut neuron = WorldModelNeuron::new(id.clone());

        let input = NeuralSignal::new(id.clone(), vec![0.5; 64]);
        let output = neuron.forward(&input);

        assert_eq!(output.vector.len(), WORLDMODEL_OUTPUT_DIM);
    }
}
