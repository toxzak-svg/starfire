//! Star — Emergent Desktop Intelligence
//!
//! A reasoning intelligence that finds its power from architecture, not scale.
//! Runs locally, offline, indefinitely. Feels alive because it *is* alive
//! in the sense that it has genuine continuity, genuine uncertainty, and genuine understanding.

pub mod persistence;
pub mod reasoning;
pub mod metacog;
pub mod conversation;
pub mod runtime;
pub mod context;
pub mod api;

pub use persistence::{Store, Memory, Identity};
pub use conversation::Conversation;
pub use reasoning::ReasoningEngine;
pub use runtime::Runtime;
pub use context::{RingState, ReasoningMode, ContextState, OpenQuestion, ContextFuser};

use anyhow::Result;

/// Initialize the Star runtime with storage at the given path.
/// Call this once at startup.
pub fn init(data_dir: &std::path::Path) -> Result<Runtime> {
    Runtime::new(data_dir)
}
