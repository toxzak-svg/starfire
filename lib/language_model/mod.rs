//! Language Model — Character-level RNN
//!
//! A minimal character-level language model that learns to generate text
//! from conversation data. No external ML frameworks — pure Rust.
//!
//! Architecture: Embedding → LSTM → Linear → char probabilities

pub mod model;
pub mod train;
pub mod generate;
pub mod vocabulary;

pub use model::CharRNN;
pub use vocabulary::Vocabulary;