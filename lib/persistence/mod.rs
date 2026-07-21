//! Persistence Layer (Layer 1)
//!
//! Handles identity, memory with decay, session continuity, and storage.
//! This is the foundation — everything else runs on top of persistent state.

pub mod companion;
pub mod identity;
pub mod identity_guard;
pub mod memory;
pub mod session;
pub mod store;
pub mod tiers;

pub use companion::{CompanionJournalStats, CompanionPersistence, CompanionPersistenceError};
pub use identity::Identity;
pub use identity_guard::IdentityGuard;
pub use memory::{Belief, BeliefState, Memory, MemoryDomain};
pub use session::Session;
pub use store::{MemorySnapshot, ReasoningEvent, ReasoningGap, Store};
pub use tiers::{ConsolidationManager, MemoryTier, WorkingMemory};
