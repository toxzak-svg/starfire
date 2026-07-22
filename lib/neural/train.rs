//! Training Infrastructure for Neural Networks
//!
//! Implements backpropagation, Hebbian learning, and related training algorithms.

use crate::neural::{Connection, ConnectionType, NeuralNet, NeuralSignal, NeuronId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Loss function for training
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LossFunction {
    /// Mean squared error
    MSE,
    /// Cross-entropy
    CrossEntropy,
    /// Contrastive (for similarity learning)
    Contrastive,
    /// Hebbian (local, no global loss)
    Hebbian,
}

impl LossFunction {
    /// Compute loss between prediction and target
    pub fn compute(&self, pred: &[f32], target: &[f32]) -> f32 {
        match self {
            LossFunction::MSE => {
                let mut loss = 0.0;
                for (p, t) in pred.iter().zip(target.iter()) {
                    let diff = p - t;
                    loss += diff * diff;
                }
                loss / pred.len() as f32
            }
            LossFunction::CrossEntropy => {
                let mut loss = 0.0;
                for (p, t) in pred.iter().zip(target.iter()) {
                    let p_clamped = p.max(1e-7).min(1.0 - 1e-7);
                    loss -= t * p_clamped.ln() + (1.0 - t) * (1.0 - p_clamped).ln();
                }
                loss / pred.len() as f32
            }
            LossFunction::Contrastive => {
                unimplemented!("Contrastive loss needs two vectors")
            }
            LossFunction::Hebbian => 0.0, // Local, no global loss
        }
    }

    /// Compute gradient of loss with respect to prediction
    pub fn gradient(&self, pred: &[f32], target: &[f32]) -> Vec<f32> {
        match self {
            LossFunction::MSE => {
                pred.iter()
                    .zip(target.iter())
                    .map(|(p, t)| 2.0 * (p - t) / pred.len() as f32)
                    .collect()
            }
            LossFunction::CrossEntropy => {
                pred.iter()
                    .zip(target.iter())
                    .map(|(p, t)| {
                        let p_clamped = p.max(1e-7).min(1.0 - 1e-7);
                        -(t / p_clamped) + (1.0 - t) / (1.0 - p_clamped)
                    })
                    .collect()
            }
            _ => vec![0.0; pred.len()],
        }
    }
}

/// Training state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trainer {
    /// Learning rate
    pub lr: f32,
    /// Momentum
    pub momentum: f32,
    /// Current epoch
    pub epoch: usize,
    /// Loss history
    pub loss_history: Vec<f32>,
    /// Weight velocity (for momentum)
    velocity: HashMap<(NeuronId, String), f32>,
}

impl Default for Trainer {
    fn default() -> Self {
        Self::new(0.01, 0.9)
    }
}

impl Trainer {
    pub fn new(lr: f32, momentum: f32) -> Self {
        Self {
            lr,
            momentum,
            epoch: 0,
            loss_history: Vec::new(),
            velocity: HashMap::new(),
        }
    }

    /// Update a single weight using gradient descent with momentum
    pub fn update_weight(
        &mut self,
        neuron_id: &NeuronId,
        weight_name: &str,
        gradient: f32,
        current_weight: f32,
    ) -> f32 {
        let key = (neuron_id.clone(), weight_name.to_string());

        // Get or initialize velocity
        let vel = self.velocity.entry(key).or_insert(0.0);

        // Update velocity: v = momentum * v - lr * grad
        *vel = self.momentum * *vel - self.lr * gradient;

        // Update weight: w = w + v
        current_weight + *vel
    }

    /// Apply Hebbian learning rule to a connection
    ///
    /// Δw = η * pre * post
    /// "Neurons that fire together, wire together"
    pub fn hebbian_update(connection: &mut Connection, pre: f32, post: f32, lr: f32) {
        let delta = lr * pre * post;

        match connection.conn_type {
            ConnectionType::Excitatory => {
                connection.weight = (connection.weight + delta).clamp(-1.0, 1.0);
            }
            ConnectionType::Inhibitory => {
                connection.weight = (connection.weight - delta).clamp(-1.0, 0.0);
            }
            ConnectionType::Modulatory => {
                // Modulatory: update eligibility trace
                connection.eligibility_trace = 0.9 * connection.eligibility_trace + delta;
            }
            ConnectionType::Recurrent => {
                // Recurrent: stronger update
                connection.weight = (connection.weight + delta * 1.5).clamp(-1.0, 1.0);
            }
        }
    }

    /// Apply novelty-gated plasticity
    ///
    /// Only update weights when novelty is high — prevents
    /// catastrophic forgetting of known patterns.
    pub fn novelty_gated_update(
        connection: &mut Connection,
        pre: f32,
        post: f32,
        novelty: f32,
        lr: f32,
    ) {
        // Scale learning by novelty
        let effective_lr = lr * novelty;
        Self::hebbian_update(connection, pre, post, effective_lr);
    }

    /// Train network on a single example
    pub fn train_step(
        &mut self,
        net: &mut NeuralNet,
        input: &NeuralSignal,
        target: &[f32],
        loss_fn: LossFunction,
    ) -> f32 {
        // Forward pass
        let outputs = net.forward(input);

        // Get output from last neuron (simplified - assumes last is output)
        let pred = outputs.last().map(|s| s.vector.as_slice()).unwrap_or(target);

        // Compute loss
        let loss = loss_fn.compute(pred, target);
        self.loss_history.push(loss);

        // Compute gradients via backward pass
        let gradients = net.backward(target, loss_fn);

        // Apply gradients to connection weights
        net.apply_gradients(&gradients, self.lr);

        self.epoch += 1;
        loss
    }

    /// Train on a batch of examples
    pub fn train_batch(
        &mut self,
        net: &mut NeuralNet,
        batch: &[(NeuralSignal, Vec<f32>)],
        loss_fn: LossFunction,
    ) -> f32 {
        let mut total_loss = 0.0;

        for (input, target) in batch {
            total_loss += self.train_step(net, input, target, loss_fn);
        }

        total_loss / batch.len() as f32
    }

    /// Run multiple epochs of training
    pub fn train_epochs(
        &mut self,
        net: &mut NeuralNet,
        dataset: &[(NeuralSignal, Vec<f32>)],
        loss_fn: LossFunction,
        num_epochs: usize,
    ) -> Vec<f32> {
        let mut epoch_losses = Vec::new();

        for _ in 0..num_epochs {
            let epoch_loss = self.train_batch(net, dataset, loss_fn);
            epoch_losses.push(epoch_loss);
            self.epoch += 1;
        }

        epoch_losses
    }

    /// Train with Hebbian updates (local learning only)
    pub fn train_hebbian_step(
        &mut self,
        net: &mut NeuralNet,
        signals: &[NeuralSignal],
    ) {
        // Apply Hebbian updates to all connections based on signals
        for signal in signals {
            // Find connections originating from this neuron
            for conn in &mut net.topology_mut().connections {
                if conn.from == signal.source {
                    // Get pre-synaptic activity (this neuron's output)
                    let pre = signal.vector.iter().sum::<f32>() / signal.vector.len() as f32;

                    // Post-synaptic activity would come from the target neuron
                    // Simplified: use attention as proxy for post-synaptic activity
                    let post = signal.attention;

                    Self::novelty_gated_update(
                        conn,
                        pre,
                        post,
                        signal.novelty,
                        self.lr,
                    );
                }
            }
        }

        self.epoch += 1;
    }

    /// Apply Hebbian learning to entire network
    pub fn hebbian_epoch(&mut self, net: &mut NeuralNet, inputs: &[NeuralSignal]) {
        for input in inputs {
            let outputs = net.forward(input);
            self.train_hebbian_step(net, &outputs);
        }
    }
}

/// Gradient accumulation for backprop through time
#[derive(Debug, Clone)]
pub struct BPTTState {
    /// Stored activations for each timestep
    pub activations: Vec<Vec<f32>>,
    /// Stored gradients for each timestep
    pub gradients: Vec<Vec<f32>>,
    /// Truncated BPTT horizon
    pub horizon: usize,
}

impl BPTTState {
    pub fn new(horizon: usize) -> Self {
        Self {
            activations: Vec::new(),
            gradients: Vec::new(),
            horizon,
        }
    }

    /// Store activations from a forward pass step
    pub fn store_activation(&mut self, activation: Vec<f32>) {
        self.activations.push(activation);
        if self.activations.len() > self.horizon {
            self.activations.remove(0);
        }
    }

    /// Truncate and prepare for backward pass
    pub fn truncate(&mut self) {
        if self.activations.len() > self.horizon {
            let start = self.activations.len() - self.horizon;
            self.activations = self.activations[start..].to_vec();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mse_loss() {
        let loss_fn = LossFunction::MSE;
        let pred = vec![0.8, 0.7, 0.9];
        let target = vec![1.0, 0.5, 1.0];

        let loss = loss_fn.compute(&pred, &target);
        assert!(loss > 0.0);
    }

    #[test]
    fn test_mse_gradient() {
        let loss_fn = LossFunction::MSE;
        let pred = vec![0.8, 0.7];
        let target = vec![1.0, 0.5];

        let grad = loss_fn.gradient(&pred, &target);
        assert_eq!(grad.len(), 2);
    }

    #[test]
    fn test_hebbian_update() {
        let mut conn = Connection::new(NeuronId::new("a"), NeuronId::new("b"), 0.5);

        Trainer::hebbian_update(&mut conn, 1.0, 1.0, 0.1);

        // Should increase (positive correlation)
        assert!(conn.weight > 0.5);
    }

    #[test]
    fn test_trainer_update() {
        let mut trainer = Trainer::new(0.01, 0.9);
        let neuron_id = NeuronId::new("test");

        let new_weight = trainer.update_weight(&neuron_id, "weight_0", 0.1, 0.5);

        // Should have moved in negative direction (gradient was positive)
        assert!(new_weight < 0.5);
    }
}
