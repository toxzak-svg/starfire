//! Dreaming as Synthetic Episodes
//!
//! Counterfactual self-experiences Star fabricates and uses as speculative training data.

pub mod dream_engine;
pub mod synthesizer;
pub mod validator;

pub use dream_engine::{DreamEngine, DreamTheme, DreamEpisode};
pub use synthesizer::DreamSynthesizer;
pub use validator::DreamValidator;