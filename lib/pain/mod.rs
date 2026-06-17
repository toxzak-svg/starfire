//! Pain as Computational Inefficiency
//!
//! Turn wasted computation into "pain" — a drive to reduce it.
//!
//! Every time Star:
//! - Recomputes a derivation that could have been cached
//! - Walks a reasoning path that ends in contradiction
//! - Spends time on something irrelevant to the user's goal
//!
//! She logs a "pain event" associated with: the concepts involved,
//! the reasoning pattern, and the context.
//!
//! **Layer 1:** For each rule, concept, and strategy, maintain a "pain score"
//! with usage statistics.
//!
//! **Layer 2:** When selecting reasoning strategy, include "expected pain" as
//! a cost term. Prefer low-pain strategies.
//!
//! **Layer 3:** Periodically analyzes where pain concentrates. Proposes structural
//! fixes: new abstractions, deprecating rules, adding intermediate concepts.

pub mod event;
pub mod tracker;
pub mod cost_model;

pub use event::{PainEvent, PainSource, ReasoningPattern};
pub use tracker::PainTracker;
pub use cost_model::PainCostModel;

/// Initialize the pain tracking system
pub fn new() -> PainTracker {
    PainTracker::new()
}