//! User-Cognition Model — Star's model of Zachary's mind
//!
//! Star maintains a model of Zachary's cognition parallel to her own:
//! - What he remembers well vs. poorly
//! - His typical stances (risk-averse vs. speculative)
//! - His preferred argument styles (concrete examples vs. abstractions)
//!
//! This allows Star to "compute less and delegate more" — knowing when to
//! ask Zachary a question versus when to reason internally.

pub mod preference;
pub mod memory_model;
pub mod types;

pub use preference::{UserPreference, PreferenceType, InferenceSource};
pub use memory_model::UserMemoryModel;
pub use types::{UserCognitionModel, ResponsePattern, ReasoningStance, ArgumentStyle};