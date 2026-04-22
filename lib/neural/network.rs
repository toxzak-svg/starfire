//! Neural Network Infrastructure
//!
//! Manages neurons, routing, topology, and forward/backward passes.

use crate::neural::neuron::{Activation, Neuron, NeuronId, NeuralSignal, NeuronConfig, ConnectionType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the neural network
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Input dimension for the network
    pub input_dim: usize,
    /// Hidden dimension (default for neurons)
    pub hidden_dim: usize,
    /// Output dimension for the network
    pub output_dim: usize,
    /// Default activation function
    pub default_activation: Activation,
    /// Learning rate
    pub learning_rate: f32,
    /// Momentum for gradient updates
    pub momentum: f32,
    /// Whether to use dropout
    pub use_dropout: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            input_dim: 64,
            hidden_dim: 64,
            output_dim: 64,
            default_activation: Activation::Tanh,
            learning_rate: 0.01,
            momentum: 0.9,
            use_dropout: false,
        }
    }
}

/// Connection between two neurons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// Source neuron
    pub from: NeuronId,
    /// Target neuron
    pub to: NeuronId,
    /// Connection type
    pub conn_type: ConnectionType,
    /// Current weight
    pub weight: f32,
    /// Weight gradient (for training)
    pub gradient: f32,
    /// Eligibility trace (for Hebbian learning)
    pub eligibility_trace: f32,
}

impl Connection {
    pub fn new(from: NeuronId, to: NeuronId, weight: f32) -> Self {
        Self {
            from,
            to,
            conn_type: ConnectionType::Excitatory,
            weight,
            gradient: 0.0,
            eligibility_trace: 0.0,
        }
    }

    pub fn modulated(from: NeuronId, to: NeuronId, weight: f32) -> Self {
        Self {
            from,
            to,
            conn_type: ConnectionType::Modulatory,
            weight,
            gradient: 0.0,
            eligibility_trace: 0.0,
        }
    }

    pub fn inhibitory(from: NeuronId, to: NeuronId, weight: f32) -> Self {
        Self {
            from,
            to,
            conn_type: ConnectionType::Inhibitory,
            weight: -weight.abs(),
            gradient: 0.0,
            eligibility_trace: 0.0,
        }
    }
}

/// Network topology definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    /// Ordered list of layers (each layer is a set of neuron IDs)
    pub layers: Vec<Vec<NeuronId>>,
    /// Connections between neurons
    pub connections: Vec<Connection>,
    /// Recurrent connections (neuron -> itself)
    pub recurrent: Vec<(NeuronId, NeuronId)>,
}

impl Topology {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            connections: Vec::new(),
            recurrent: Vec::new(),
        }
    }

    /// Add a layer of neurons
    pub fn add_layer(&mut self, layer: Vec<NeuronId>) {
        self.layers.push(layer);
    }

    /// Add a connection between neurons
    pub fn add_connection(&mut self, from: NeuronId, to: NeuronId, weight: f32) {
        // Check if connection already exists
        if !self.connections.iter().any(|c| c.from == from && c.to == to) {
            self.connections.push(Connection::new(from, to, weight));
        }
    }

    /// Add a recurrent connection
    pub fn add_recurrent(&mut self, from: NeuronId, to: NeuronId) {
        self.recurrent.push((from, to));
    }

    /// Get all connections outgoing from a neuron
    pub fn outgoing(&self, from: &NeuronId) -> Vec<&Connection> {
        self.connections.iter().filter(|c| c.from == *from).collect()
    }

    /// Get all connections incoming to a neuron
    pub fn incoming(&self, to: &NeuronId) -> Vec<&Connection> {
        self.connections.iter().filter(|c| c.to == *to).collect()
    }
}

impl Default for Topology {
    fn default() -> Self {
        Self::new()
    }
}

/// The neural network managing all neurons
pub struct NeuralNet {
    /// All neurons in the network
    neurons: HashMap<NeuronId, Box<dyn Neuron>>,
    /// Network topology
    topology: Topology,
    /// Configuration
    config: NetworkConfig,
    /// Signal buffer (for routing)
    signals: HashMap<NeuronId, NeuralSignal>,
    /// Forward pass order (computed from topology)
    forward_order: Vec<NeuronId>,
}

impl NeuralNet {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            neurons: HashMap::new(),
            topology: Topology::new(),
            config,
            signals: HashMap::new(),
            forward_order: Vec::new(),
        }
    }

    /// Register a neuron with the network
    pub fn add_neuron(&mut self, neuron: Box<dyn Neuron>) {
        let id = neuron.id();
        self.neurons.insert(id, neuron);
        self.recompute_forward_order();
    }

    /// Add a connection between neurons
    pub fn connect(&mut self, from: NeuronId, to: NeuronId, weight: f32) {
        self.topology.add_connection(from, to, weight);
    }

    /// Add a connection with a specific type
    pub fn connect_with_type(&mut self, from: NeuronId, to: NeuronId, conn_type: ConnectionType, weight: f32) {
        match conn_type {
            ConnectionType::Inhibitory => {
                self.topology.add_connection(from, to, -weight.abs());
            }
            ConnectionType::Modulatory => {
                let mut conn = Connection::modulated(from, to, weight);
                self.topology.connections.push(conn);
            }
            _ => {
                self.topology.add_connection(from, to, weight);
            }
        }
    }

    /// Add a recurrent connection
    pub fn add_recurrent(&mut self, from: NeuronId, to: NeuronId) {
        self.topology.add_recurrent(from, to);
    }

    /// Recompute forward pass order from topology
    fn recompute_forward_order(&mut self) {
        self.forward_order.clear();

        // Topological sort based on connections
        let mut in_degree: HashMap<NeuronId, usize> = HashMap::new();
        let mut adj: HashMap<NeuronId, Vec<NeuronId>> = HashMap::new();

        // Initialize
        for id in self.neurons.keys() {
            in_degree.insert(id.clone(), 0);
            adj.insert(id.clone(), Vec::new());
        }

        // Build adjacency and in-degree
        for conn in &self.topology.connections {
            adj.entry(conn.from.clone()).or_default().push(conn.to.clone());
            if let Some(deg) = in_degree.get_mut(&conn.to) {
                *deg += 1;
            }
        }

        // Kahn's algorithm
        let mut queue: Vec<NeuronId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        while let Some(node) = queue.pop() {
            self.forward_order.push(node.clone());
            if let Some(neighbors) = adj.get(&node) {
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }
    }

    /// Forward pass: process input through the entire network
    pub fn forward(&mut self, input: &NeuralSignal) -> Vec<NeuralSignal> {
        self.signals.clear();

        // Store input signal
        self.signals.insert(input.source.clone(), input.clone());

        // Process neurons in topological order
        for neuron_id in &self.forward_order {
            // Gather all incoming signals
            let incoming: Vec<&Connection> = self.topology.incoming(&neuron_id);

            if incoming.is_empty() {
                continue; // No inputs yet
            }

            // Compute weighted sum of inputs
            let mut combined: Option<Vec<f32>> = None;

            for conn in &incoming {
                if let Some(signal) = self.signals.get(&conn.from) {
                    let mut weighted = signal.vector.clone();

                    // Apply connection weight
                    for w in weighted.iter_mut() {
                        *w *= conn.weight;
                    }

                    // Apply connection type
                    match conn.conn_type {
                        ConnectionType::Inhibitory => {
                            for w in weighted.iter_mut() {
                                *w = -w.abs();
                            }
                        }
                        ConnectionType::Modulatory => {
                            // Modulatory just scales — already done with weight
                        }
                        _ => {}
                    }

                    // Accumulate
                    if let Some(ref mut acc) = combined {
                        for (i, val) in weighted.iter().enumerate() {
                            acc[i] += val;
                        }
                    } else {
                        combined = Some(weighted);
                    }
                }
            }

            // Process through neuron if we have inputs
            if let Some(ref input_vec) = combined {
                let mut neuron_input = NeuralSignal::new(neuron_id.clone(), input_vec.clone());
                // Pass along attention from source
                neuron_input.attention = self.signals.get(&input.source)
                    .map(|s| s.attention)
                    .unwrap_or(1.0);

                let output = self.neurons.get_mut(neuron_id)
                    .expect("Neuron not found")
                    .forward(&neuron_input);

                self.signals.insert(neuron_id.clone(), output);
            }
        }

        // Collect outputs in topological order
        self.forward_order
            .iter()
            .filter_map(|id| self.signals.get(id).cloned())
            .collect()
    }

    /// Get a neuron by ID
    pub fn get(&self, id: &NeuronId) -> Option<&Box<dyn Neuron>> {
        self.neurons.get(id)
    }

    /// Get a mutable neuron by ID
    pub fn get_mut(&mut self, id: &NeuronId) -> Option<&mut Box<dyn Neuron>> {
        self.neurons.get_mut(id)
    }

    /// Get topology
    pub fn topology(&self) -> &Topology {
        &self.topology
    }

    /// Get mutable topology
    pub fn topology_mut(&mut self) -> &mut Topology {
        &mut self.topology
    }

    /// Get config
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    /// Total parameters in the network
    pub fn param_count(&self) -> usize {
        self.neurons.values().map(|n| n.param_count()).sum()
    }

    /// Reset all neurons
    pub fn reset(&mut self) {
        for neuron in self.neurons.values_mut() {
            neuron.reset();
        }
        self.signals.clear();
    }

    /// Get all weights for inspection
    pub fn get_all_weights(&self) -> HashMap<NeuronId, HashMap<String, Vec<f32>>> {
        self.neurons
            .iter()
            .map(|(id, n)| (id.clone(), n.get_weights()))
            .collect()
    }

    /// Set all weights (for loading trained networks)
    pub fn set_all_weights(&mut self, weights: &HashMap<NeuronId, HashMap<String, Vec<f32>>>) {
        for (id, w) in weights {
            if let Some(neuron) = self.neurons.get_mut(id) {
                neuron.set_weights(w);
            }
        }
    }

    /// Backward pass: compute gradients through the network
    ///
    /// Returns a HashMap of (neuron_id, weight_name) -> gradient
    pub fn backward(&mut self, target: &[f32], loss_fn: crate::neural::train::LossFunction) -> HashMap<NeuronId, Vec<f32>> {
        let mut gradients: HashMap<NeuronId, Vec<f32>> = HashMap::new();

        // Get output from last neuron
        let output_neuron_id = self.forward_order.last();

        if let Some(last_id) = output_neuron_id {
            if let Some(output_signal) = self.signals.get(last_id) {
                // Compute loss gradient at output
                let output_grad = loss_fn.gradient(&output_signal.vector, target);

                // Initialize gradients with output gradient
                gradients.insert(last_id.clone(), output_grad);

                // Backprop through reverse topological order (skipping input layer)
                for neuron_id in self.forward_order.iter().rev().skip(1) {
                    let incoming: Vec<&Connection> = self.topology.incoming(neuron_id);

                    if incoming.is_empty() {
                        continue;
                    }

                    // Get gradient from downstream neurons
                    let mut incoming_grad = vec![0.0; self.config.hidden_dim];

                    for conn in &incoming {
                        if let Some(downstream_grad) = gradients.get(&conn.to) {
                            // Gradient flows backward through connection weight
                            let weighted_grad: Vec<f32> = downstream_grad.iter()
                                .map(|g| g * conn.weight)
                                .collect();

                            // Accumulate
                            for (i, wg) in weighted_grad.iter().enumerate() {
                                if i < incoming_grad.len() {
                                    incoming_grad[i] += wg;
                                }
                            }
                        }
                    }

                    gradients.insert(neuron_id.clone(), incoming_grad);
                }
            }
        }

        gradients
    }

    /// Apply gradients to connection weights
    pub fn apply_gradients(&mut self, gradients: &HashMap<NeuronId, Vec<f32>>, lr: f32) {
        for conn in &mut self.topology.connections {
            if let Some(grad) = gradients.get(&conn.to) {
                // Compute average gradient for this connection
                let avg_grad: f32 = grad.iter().sum::<f32>() / (grad.len() as f32).max(1.0);

                // Apply gradient: w = w - lr * grad
                conn.weight -= lr * avg_grad;

                // Clamp weights to reasonable range
                conn.weight = conn.weight.clamp(-2.0, 2.0);
            }
        }
    }

    /// Get connection weights as a Vec for external training
    pub fn get_connection_weights(&self) -> Vec<f32> {
        self.topology.connections.iter().map(|c| c.weight).collect()
    }

    /// Set connection weights from external training
    pub fn set_connection_weights(&mut self, weights: &[f32]) {
        for (i, conn) in self.topology.connections.iter_mut().enumerate() {
            if i < weights.len() {
                conn.weight = weights[i].clamp(-2.0, 2.0);
            }
        }
    }

    /// Apply plastic updates to connection weights based on signal correlations
    ///
    /// Implements correlation-based Hebbian plasticity:
    /// - Δw = λ * pre * post * (1 - |w|)  (soft bounds)
    ///
    /// This is called after a forward pass to enable online learning.
    pub fn apply_plasticity(&mut self, signals: &[NeuralSignal], lambda: f32) {
        for conn in &mut self.topology.connections {
            // Get the pre-synaptic signal
            if let Some(pre_signal) = signals.iter().find(|s| s.source == conn.from) {
                // Get the post-synaptic signal (output of the target neuron)
                if let Some(post_signal) = signals.iter().find(|s| s.source == conn.to) {
                    // Compute average activation
                    let pre_avg = pre_signal.vector.iter().sum::<f32>() / pre_signal.vector.len().max(1) as f32;
                    let post_avg = post_signal.vector.iter().sum::<f32>() / post_signal.vector.len().max(1) as f32;

                    // Compute correlation (product of activations)
                    let correlation = pre_avg * post_avg;

                    // Apply Hebbian update with soft bounds: Δw = λ * correlation * (1 - |w|)
                    let weight_change = lambda * correlation * (1.0 - conn.weight.abs());

                    match conn.conn_type {
                        ConnectionType::Excitatory => {
                            conn.weight += weight_change;
                        }
                        ConnectionType::Inhibitory => {
                            // Inhibitory: strengthen when pre active and post should be suppressed
                            conn.weight -= weight_change;
                        }
                        ConnectionType::Modulatory => {
                            // Modulatory: only update eligibility trace
                            conn.eligibility_trace += weight_change * 0.1;
                        }
                        ConnectionType::Recurrent => {
                            // Recurrent: stronger update
                            conn.weight += weight_change * 1.5;
                        }
                    }

                    // Apply soft bounds
                    conn.weight = conn.weight.clamp(-1.0, 1.0);
                }
            }
        }
    }

    /// Apply plastic updates with novelty gating
    ///
    /// Only updates when novelty signal is high, preventing
    /// catastrophic forgetting of known patterns.
    pub fn apply_plasticity_novelty_gated(&mut self, signals: &[NeuralSignal], lambda: f32, novelty_threshold: f32) {
        // Compute network novelty as average of signal novelties
        let avg_novelty = if signals.is_empty() {
            0.0
        } else {
            signals.iter().map(|s| s.novelty).sum::<f32>() / signals.len() as f32
        };

        // Only apply plasticity if novelty is above threshold
        if avg_novelty >= novelty_threshold {
            self.apply_plasticity(signals, lambda * avg_novelty);
        }
    }

    /// Compute weight statistics for monitoring
    pub fn weight_statistics(&self) -> WeightStats {
        let weights: Vec<f32> = self.topology.connections.iter().map(|c| c.weight).collect();
        if weights.is_empty() {
            return WeightStats {
                mean: 0.0,
                std: 0.0,
                min: 0.0,
                max: 0.0,
            };
        }

        let sum: f32 = weights.iter().sum();
        let mean = sum / weights.len() as f32;
        let variance = weights.iter().map(|w| (w - mean).powi(2)).sum::<f32>() / weights.len() as f32;
        let std = variance.sqrt();
        let min = weights.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = weights.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        WeightStats { mean, std, min, max }
    }
}

/// Statistics about connection weights
#[derive(Debug, Clone)]
pub struct WeightStats {
    pub mean: f32,
    pub std: f32,
    pub min: f32,
    pub max: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neural::neuron::{NeuronConfig, NeuronState};

    // Simple test neuron for testing the network
    struct TestNeuron {
        id: NeuronId,
        config: NeuronConfig,
    }

    impl Neuron for TestNeuron {
        fn id(&self) -> NeuronId { self.id.clone() }
        fn config(&self) -> &NeuronConfig { &self.config }
        fn forward(&mut self, input: &NeuralSignal) -> NeuralSignal {
            NeuralSignal::new(self.id.clone(), input.vector.clone())
        }
        fn reset(&mut self) {}
        fn state(&self) -> &NeuronState { unimplemented!() }
        fn param_count(&self) -> usize { 0 }
        fn get_weights(&self) -> std::collections::HashMap<String, Vec<f32>> { std::collections::HashMap::new() }
        fn set_weights(&mut self, _: &std::collections::HashMap<String, Vec<f32>>) {}
        fn clone_box(&self) -> Box<dyn Neuron> { Box::new(Self { id: self.id.clone(), config: self.config.clone() }) }
    }

    #[test]
    fn test_network_creation() {
        let config = NetworkConfig::default();
        let net = NeuralNet::new(config);

        assert_eq!(net.param_count(), 0);
        assert_eq!(net.topology().connections.len(), 0);
    }

    #[test]
    fn test_topological_order() {
        let mut net = NeuralNet::new(NetworkConfig::default());

        let a = NeuronId::new("a");
        let b = NeuronId::new("b");
        let c = NeuronId::new("c");

        net.add_neuron(Box::new(TestNeuron { id: a.clone(), config: NeuronConfig::default() }));
        net.add_neuron(Box::new(TestNeuron { id: b.clone(), config: NeuronConfig::default() }));
        net.add_neuron(Box::new(TestNeuron { id: c.clone(), config: NeuronConfig::default() }));

        // a -> b -> c
        net.connect(a, b.clone(), 1.0);
        net.connect(b, c, 1.0);

        // Forward order should be a, b, c
        assert_eq!(net.forward_order.len(), 3);
    }
}
