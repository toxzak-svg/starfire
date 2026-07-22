//! Neural Module Architecture — StarNet
//!
//! A meta-neural architecture where each Starfire module becomes a "neuron"
//! in a larger trainable network. Reasoning emerges from network dynamics
//! rather than being hard-coded.
//!
//! # Core Concept
//!
//! - Each module (Quanot, Knowledge, Reasoning, etc.) becomes a tiny neural model
//! - Neurons communicate via weighted connections
//! - Network topology encodes the information flow
//! - Connection weights are trainable via backpropagation

pub mod neuron;
pub mod network;
pub mod layer;
pub mod train;
pub mod neurons;

pub use neuron::{Neuron, NeuronId, NeuralSignal, ConnectionType, Activation, NeuronConfig, NeuronState};
pub use network::{NeuralNet, NetworkConfig, Topology, Connection, WeightStats};
pub use layer::{Layer, LayerType};
pub use train::{Trainer, LossFunction};

pub mod serialization {
    //! Serialization support for Neural Networks

    use serde::{Deserialize, Serialize};
    use crate::neural::{NeuralNet, NetworkConfig, Topology, Connection, NeuronId};

    /// Serializable representation of a connection
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SerializableConnection {
        pub from: String,
        pub to: String,
        pub weight: f32,
        pub conn_type: String,
        pub eligibility_trace: f32,
    }

    impl From<&Connection> for SerializableConnection {
        fn from(conn: &Connection) -> Self {
            Self {
                from: conn.from.to_string(),
                to: conn.to.to_string(),
                weight: conn.weight,
                conn_type: format!("{:?}", conn.conn_type),
                eligibility_trace: conn.eligibility_trace,
            }
        }
    }

    /// Serializable representation of topology
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SerializableTopology {
        pub connections: Vec<SerializableConnection>,
    }

    impl From<&Topology> for SerializableTopology {
        fn from(topology: &Topology) -> Self {
            Self {
                connections: topology.connections.iter().map(|c| c.into()).collect(),
            }
        }
    }

    /// Serializable network state (weights only, not full network)
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NetworkState {
        pub config: SerializableNetworkConfig,
        pub topology: SerializableTopology,
        pub neuron_weights: Vec<NeuronWeightSnapshot>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SerializableNetworkConfig {
        pub input_dim: usize,
        pub hidden_dim: usize,
        pub output_dim: usize,
        pub default_activation: String,
        pub learning_rate: f32,
        pub momentum: f32,
        pub use_dropout: bool,
    }

    impl From<&NetworkConfig> for SerializableNetworkConfig {
        fn from(config: &NetworkConfig) -> Self {
            Self {
                input_dim: config.input_dim,
                hidden_dim: config.hidden_dim,
                output_dim: config.output_dim,
                default_activation: format!("{:?}", config.default_activation),
                learning_rate: config.learning_rate,
                momentum: config.momentum,
                use_dropout: config.use_dropout,
            }
        }
    }

    impl From<&SerializableNetworkConfig> for NetworkConfig {
        fn from(config: &SerializableNetworkConfig) -> Self {
            use crate::neural::Activation;
            let activation = match config.default_activation.as_str() {
                "Sigmoid" => Activation::Sigmoid,
                "Tanh" => Activation::Tanh,
                "ReLU" => Activation::ReLU,
                "Softmax" => Activation::Softmax,
                _ => Activation::Identity,
            };
            Self {
                input_dim: config.input_dim,
                hidden_dim: config.hidden_dim,
                output_dim: config.output_dim,
                default_activation: activation,
                learning_rate: config.learning_rate,
                momentum: config.momentum,
                use_dropout: config.use_dropout,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NeuronWeightSnapshot {
        pub id: String,
        pub weights: std::collections::HashMap<String, Vec<f32>>,
    }

    /// Serialize network to JSON string
    pub fn serialize_network(net: &NeuralNet) -> Result<String, serde_json::Error> {
        let state = NetworkState {
            config: net.config().into(),
            topology: net.topology().into(),
            neuron_weights: net.get_all_weights()
                .into_iter()
                .map(|(id, w)| NeuronWeightSnapshot {
                    id: id.to_string(),
                    weights: w,
                })
                .collect(),
        };
        serde_json::to_string_pretty(&state)
    }

    /// Deserialize network from JSON string
    pub fn deserialize_network(json: &str) -> Result<NeuralNet, serde_json::Error> {
        let state: NetworkState = serde_json::from_str(json)?;

        let config: NetworkConfig = (&state.config).into();
        let mut net = NeuralNet::new(config);

        // Restore topology
        for conn in &state.topology.connections {
            let from = NeuronId::new(&conn.from);
            let to = NeuronId::new(&conn.to);
            net.connect(from, to, conn.weight);
        }

        // Restore neuron weights
        let weights: std::collections::HashMap<NeuronId, std::collections::HashMap<String, Vec<f32>>> = state
            .neuron_weights
            .into_iter()
            .map(|n| (NeuronId::new(&n.id), n.weights))
            .collect();
        net.set_all_weights(&weights);

        Ok(net)
    }

    /// Save network state to file
    pub fn save_network(net: &NeuralNet, path: &str) -> std::io::Result<()> {
        let json = serialize_network(net).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Load network state from file
    pub fn load_network(path: &str) -> Result<NeuralNet, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        deserialize_network(&json).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
