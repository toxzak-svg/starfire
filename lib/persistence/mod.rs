//! Persistence Layer (Layer 1)
//!
//! Handles identity, memory with decay, session continuity, and storage.
//! This is the foundation — everything else runs on top of persistent state.

pub mod store;
pub mod memory;
pub mod identity;
pub mod session;
pub mod identity_guard;
pub mod tiers;

pub use store::{Store, MemorySnapshot, ReasoningEvent, ReasoningGap};
pub use memory::{Memory, MemoryDomain, Belief, BeliefState};
pub use identity::Identity;
pub use session::Session;
pub use identity_guard::IdentityGuard;
pub use tiers::{MemoryTier, WorkingMemory, ConsolidationManager};
