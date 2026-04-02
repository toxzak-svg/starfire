//! Star — Emergent Reasoning System
//!
//! Star is an emergent intelligence that reasons, learns, and grows.
//! She has no training data — only architecture, memory, and curiosity.

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

// Re-export commonly used types at crate root for ergonomic access
pub use runtime::Runtime;
pub use persistence::Memory;
pub use persistence::Store;
pub use persistence::memory::BeliefState;
pub use persistence::memory::Belief;
