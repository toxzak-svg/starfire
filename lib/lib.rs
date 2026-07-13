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
pub mod pain;
pub mod dreaming;
pub mod concepts;
pub mod user_model;
pub mod companion_state;
pub mod neural;
pub mod language_model;
pub mod variation;
pub mod charge;
pub mod environment;
pub mod cognitive_cycle;
pub mod autonomy;
pub mod commitment_state;
pub mod rule_induction;
pub mod graph_discovery;
pub mod latent_roles;
pub mod structural_transfer;
pub mod representation_genesis;
pub mod representation_transport_orbit;
pub use representation_transport_orbit as representation_transport;
pub mod representation_transport_descendants;

// S3: explicit-statement companion observation. Disabled by default and
// intentionally proposal-only: no state mutation, persistence, Runtime::chat()
// wiring, response routing, belief promotion, or action authority.
#[cfg(feature = "companion-observer")]
#[deny(warnings)]
#[allow(clippy::derivable_impls)]
pub mod companion_observer;

// S4: falsifiable companion predictions. Disabled by default and intentionally
// ledger-only: no Runtime::chat() wiring, response-policy influence, routing,
// belief promotion, or autonomous side-effect authority.
#[cfg(feature = "companion-prediction-ledger")]
pub mod companion_prediction_ledger;

// S5-A: companion-derived interaction-policy proposals and matched controls.
// Disabled by default and shadow-only: policies are enrolled as S4 predictions
// but cannot alter generated text, routing, beliefs, persistence, or actions.
#[cfg(feature = "companion-interaction-policy")]
pub mod companion_interaction_policy;

// S5-B: independently witnessed outcomes for S5-A trials. Direct evidence may
// resolve only the delivered arm; unshown arms require external paired review.
// This remains evaluation-only and has no live response or action authority.
#[cfg(feature = "companion-interaction-outcomes")]
pub mod companion_interaction_outcomes;

// S5-C: comparative held-out evaluation of S5-A arms over frozen S5-B evidence.
// Development data cannot influence the verdict, and PASS grants no runtime,
// routing, belief-promotion, persistence, or autonomous action authority.
#[cfg(feature = "companion-policy-evaluation")]
pub mod companion_policy_evaluation;

// S6-A/S6-B: bounded companion-policy response-planning metadata and adversarial
// replay hardening. Both remain opt-in and add no default chat, routing, memory,
// belief, ontology, persistence, tool, or autonomous-action authority.
#[cfg(feature = "companion-bounded-live-policy")]
pub mod companion_bounded_live_policy;

// S6-C: session-scoped two-phase runtime canary. Preparation occurs against a
// cloned S6 controller and the response remains opaque until a matching S5-B
// delivered arm is registered. Disabled and not attached to Runtime::chat().
#[cfg(feature = "companion-runtime-canary")]
pub mod companion_runtime_canary;

// STLM L0-B: typed semantic response authorization, validation, canonical
// digesting, and in-memory replay. Disabled by default and intentionally has no
// renderer, Runtime::chat(), persistence, routing, mutation, tool, or action authority.
#[cfg(feature = "semantic-response-program")]
pub mod semantic_response;

// H-Infant-0: typed developmental evidence boundary only. Disabled by default
// and intentionally not wired into Runtime::chat(), routing, belief promotion,
// ontology promotion, or autonomous action selection.
#[cfg(feature = "developmental-evidence")]
pub mod developmental;

// R1: IngExuity–Starfire relational prediction residual boundary. Disabled by
// default and intentionally shadow-only: no Runtime::chat() wiring, action
// authority, belief mutation, routing authority, or automatic promotion.
#[cfg(feature = "relational-evidence")]
pub mod relational;

// Re-export commonly used types at crate root for ergonomic access
pub use runtime::Runtime;
pub use persistence::Memory;
pub use persistence::Store;
pub use persistence::memory::BeliefState;
pub use persistence::memory::Belief;

// Multi-tempo cognition exports
pub use runtime::tempo::{Tempo, TempoEngine, TempoResult, tempo_for_query};
