//! Persistence Layer (Layer 1)
//!
//! Handles identity, memory with decay, session continuity, and storage.
//! This is the foundation — everything else runs on top of persistent state.

pub mod store;
pub mod memory;
pub mod identity;
pub mod session;
pub mod identity_guard;

pub use store::{Store, MemorySnapshot};
pub use memory::{Memory, MemoryDomain, Belief, BeliefState};
pub use identity::Identity;
pub use session::Session;
pub use identity_guard::IdentityGuard;
