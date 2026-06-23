//! Language Model — Character-level RNN + intent-driven reranker
//!
//! A minimal character-level language model that learns to generate text
//! from conversation data. No external ML frameworks — pure Rust.
//!
//! Architecture: Embedding → LSTM → Linear → char probabilities
//!
//! ## Modules
//!
//! - [`model`], [`train`], [`generate`], [`vocabulary`] — the charRNN engine
//!   (existing, ~11MB, ships today).
//! - [`intent_reranker`] — the new intent-driven reranker layer (voice-refine
//!   Phase 3). Sits between `runtime::chat()` and `voice::VoiceEngine::speak()`.
//!   Default backend is `MockReranker` (deterministic, no model); the
//!   `CharRnnBackend` wraps the existing charRNN; `LmRsBackend` is the
//!   skeleton for the future lm.rs / qwen3-rs in-process SLM.

pub mod model;
pub mod train;
pub mod generate;
pub mod vocabulary;
pub mod intent_reranker;

pub use model::CharRNN;
pub use vocabulary::Vocabulary;

// Re-exports for ergonomic access to the reranker layer. Callers should
// import from here rather than reaching into the submodule.
pub use intent_reranker::{
    CharRnnBackend, IntentReranker, MockReranker, RerankConfig, RerankError, RerankPrompt,
    RerankerBackend,
};
#[cfg(feature = "lmrs-backend")]
pub use intent_reranker::LmRsBackend;