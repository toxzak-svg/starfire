//! Concepts as Software Objects
//!
//! Treat every concept as a living object with state and methods, not just a node.
//!
//! **Lifecycle stages:**
//! - **Birth:** newly introduced
//! - **Adolescence:** heavily revised, lots of pain and contradictions
//! - **Maturity:** stable usage, frequent successful deployment
//! - **Senescence:** rarely used, often out-of-date or misleading
//! - **Death:** retired, replaced by descendants

pub mod concept;
pub mod lifecycle;
pub mod registry;

pub use concept::Concept;
pub use lifecycle::{LifecycleStage, LifecycleEvent};
pub use registry::ConceptRegistry;