//! Star — Emergent Reasoning System
//!
//! Star is an emergent intelligence that reasons, learns, and grows.
//! She has no training data — only architecture, memory, and curiosity.

use std::time::{SystemTime, UNIX_EPOCH};

/// Get current Unix timestamp in seconds
#[inline]
pub fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub mod api;
pub mod cognition;
pub mod persistence;
pub mod reasoning;
pub mod knowledge;
pub mod conversation;
pub mod metacog;
pub mod context;
pub mod capabilities;
pub mod training_db;
pub mod learning;
pub mod runtime;
pub mod voice;
pub mod math;
pub mod curiosity;
pub mod world_model;
pub mod quanot;
pub mod multimodal;
pub mod causal;
pub mod goals;
pub mod curriculum;
pub mod research;
pub mod prediction;
pub mod input_normalizer;
pub mod personality;
pub mod neural;
pub mod language_model;

// Re-export commonly used types at crate root for ergonomic access
pub use runtime::Runtime;
pub use persistence::Memory;
pub use persistence::Store;
pub use persistence::memory::BeliefState;
pub use persistence::memory::Belief;

// Multi-tempo cognition exports
pub use runtime::tempo::{Tempo, TempoEngine, TempoResult, tempo_for_query};
