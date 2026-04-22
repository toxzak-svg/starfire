//! Goals Neuron — Goals Module as a Neural Module
//!
//! Wraps the hierarchical goal memory as a neuron that manages
//! goals and action planning.

use crate::neural::{Activation, Neuron, NeuronConfig, NeuronId, NeuralSignal, NeuronState};
use crate::goals::{GoalEngine, GoalId, Goal};
use std::collections::HashMap;

const GOALS_INPUT_DIM: usize = 64;
const GOALS_HIDDEN_DIM: usize = 64;
const GOALS_OUTPUT_DIM: usize = 64;

pub struct GoalsNeuron {
    id: NeuronId,
    config: NeuronConfig,
    engine: GoalEngine,
    current_focus: Option<String>,
    state: NeuronState,
}

impl GoalsNeuron {
    pub fn new(id: NeuronId) -> Self {
        Self {
            id,
            config: NeuronConfig {
                input_dim: GOALS_INPUT_DIM,
                hidden_dim: GOALS_HIDDEN_DIM,
                output_dim: GOALS_OUTPUT_DIM,
                activation: Activation::ReLU,
                dropout: 0.0,
                trainable: true,
            },
            engine: GoalEngine::new(),
            current_focus: None,
            state: NeuronState::default(),
        }
    }

    fn encode_goal(&self, goal: &Goal) -> Vec<f32> {
        let mut vec = vec![0.0; GOALS_INPUT_DIM];
        let content = &goal.content;
        for (i, c) in content.bytes().enumerate().take(GOALS_INPUT_DIM) {
            vec[i] = (c as f32) / 255.0;
        }
        vec
    }

    pub fn add_goal(&mut self, goal: &str, priority: f64) -> GoalId {
        let id = self.engine.create_goal(goal, None);
        if let Some(g) = self.engine.get_mut(&id) {
            g.set_priority(priority);
        }
        id
    }

    pub fn get_active_goals(&self) -> Vec<String> {
        self.engine.active_goals_sorted()
            .iter()
            .map(|g| g.content.clone())
            .collect()
    }
}

impl Neuron for GoalsNeuron {
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

        let goal_priority = (vector_mean as f64).max(0.0).min(1.0);

        if self.engine.is_empty() {
            self.add_goal("Explore consciousness", goal_priority * 0.9);
            self.add_goal("Learn about AI", goal_priority * 0.7);
        }

        let active_goals = self.get_active_goals();

        let mut output_vec = vec![0.0; GOALS_OUTPUT_DIM];
        for (i, goal_content) in active_goals.iter().take(4).enumerate() {
            let encoded = {
                let mut v = vec![0.0; GOALS_INPUT_DIM];
                for (j, c) in goal_content.bytes().enumerate().take(GOALS_INPUT_DIM) {
                    v[j] = (c as f32) / 255.0;
                }
                v
            };
            for (j, &val) in encoded.iter().take(16).enumerate() {
                output_vec[i * 16 + j] = val;
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
        self.current_focus = None;
        self.state = NeuronState::default();
    }

    fn state(&self) -> &NeuronState {
        &self.state
    }

    fn param_count(&self) -> usize {
        GOALS_HIDDEN_DIM * (GOALS_INPUT_DIM + GOALS_OUTPUT_DIM)
    }

    fn get_weights(&self) -> HashMap<String, Vec<f32>> {
        HashMap::new()
    }

    fn set_weights(&mut self, _weights: &HashMap<String, Vec<f32>>) {}

    fn clone_box(&self) -> Box<dyn Neuron> {
        Box::new(Self {
            id: self.id.clone(),
            config: self.config.clone(),
            engine: GoalEngine::new(),
            current_focus: None,
            state: NeuronState::default(),
        })
    }
}

impl Clone for GoalsNeuron {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            config: self.config.clone(),
            engine: GoalEngine::new(),
            current_focus: None,
            state: NeuronState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goals_neuron_creation() {
        let id = NeuronId::new("goals");
        let neuron = GoalsNeuron::new(id.clone());
        assert_eq!(neuron.id(), id);
    }

    #[test]
    fn test_goals_forward() {
        let id = NeuronId::new("goals");
        let mut neuron = GoalsNeuron::new(id.clone());

        let input = NeuralSignal::new(id.clone(), vec![0.5; 64]);
        let output = neuron.forward(&input);

        assert_eq!(output.vector.len(), GOALS_OUTPUT_DIM);
    }
}
