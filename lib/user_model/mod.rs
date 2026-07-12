//! User-Cognition Model — Star's model of Zachary's mind
//!
//! Star maintains a model of Zachary's cognition parallel to her own:
//! - What he remembers well vs. poorly
//! - His typical stances (risk-averse vs. speculative)
//! - His preferred argument styles (concrete examples vs. abstractions)
//!
//! This allows Star to "compute less and delegate more" — knowing when to
//! ask Zachary a question versus when to reason internally.

pub mod companion_projection;
pub mod memory_model;
pub mod preference;
pub mod types;

pub use companion_projection::{
    project_legacy_user_model, CompanionProjectionError, CompanionProjectionPolicy,
    LegacyUserModelProjection,
};
pub use memory_model::UserMemoryModel;
pub use preference::{InferenceSource, PreferenceType, UserPreference};
pub use types::{ArgumentStyle, ReasoningStance, ResponsePattern, UserCognitionModel};
