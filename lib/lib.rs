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
pub mod grammar_extension;
pub mod recursive_grammar_composition;
pub mod multistep_abstraction_reuse;
// ΩG4 is exercised through its public integration-test target. The production
// module remains available to normal library users and executable probes.
#[cfg(not(test))]
pub mod intervention_guided_abstraction_selection;
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

// S6-C: consented and independently witnessed canary-evidence intake. Disabled
// by default; retains typed fields and opaque digests only, imports atomically
// through S5-B, and grants no runtime, routing, memory, belief, tool, or action authority.
#[cfg(feature = "companion-real-interaction-canary")]
pub mod companion_real_interaction_canary;

// ARISE-A0: terminal-first reverse-obligation planning with bounded forward
// span realization and independent reconstruction. Disabled by default. The
// current integration exposes the reusable engine and an inert runtime-shadow
// observer only; it cannot alter returned text or acquire cognitive authority.
#[cfg(feature = "arise-edge")]
pub mod arise_edge;

// STLM L0-B: typed semantic response authorization, validation, canonical
// digesting, and in-memory replay. Disabled by default and intentionally has no
// renderer, Runtime::chat(), persistence, routing, mutation, tool, or action authority.
#[cfg(feature = "semantic-response-program")]
pub mod semantic_response;

// STLM L0-C: deterministic reference realization from validated semantic
// programs and bounded lexical bindings. Disabled by default and has no live
// chat influence, raw conversation or memory access, tools, persistence, or action authority.
#[cfg(feature = "deterministic-language-renderer")]
pub mod language_realization;

// STLM L1 / ΩV1-E: verifier-ready grammar-v2 realization plus a text-only
// inverse semantic verifier. Disabled by default and executed only as an offline
// Render builder gate. It has no Runtime::chat(), response, memory, persistence,
// routing, companion-state, belief, ontology, tool, CHARGE, or action authority.
#[cfg(feature = "independent-language-verifier")]
pub mod verifier_ready_realization;
#[cfg(feature = "independent-language-verifier")]
pub mod language_verification;

// ΩV1-F1: bounded offline learned expression selection over a closed grammar-v3
// surface lattice. The ranker is integer-only and every selected candidate is
// independently reconstructed from text. Disabled by default, with no live
// response, raw prompt, state mutation, persistence, routing, or action authority.
#[cfg(feature = "omega-v1-learned-expression")]
pub mod learned_expression;

// ΩV1-F1 projection packets are sealed before offline scoring. A stale or
// corrupted VoiceState-derived projection forces exact neutral fallback and
// cannot reach candidate scoring. This remains builder-only and has no live wiring.
#[cfg(feature = "omega-v1-learned-expression")]
pub mod omega_v1f1_projection_guard;

// ΩV1-F1R1 bounded surface-family and claim-first nested-verification layers.
// These are the exact externally passed evaluator modules promoted into the
// library so F2 may execute them in shadow without duplicating the verifier.
#[cfg(feature = "omega-v1-learned-expression")]
pub mod omega_v1f1r1_surface;
#[cfg(feature = "omega-v1-learned-expression")]
pub use omega_v1f1r1_surface as surface_diversity;
#[cfg(feature = "omega-v1-learned-expression")]
pub mod omega_v1f1r1_claim_first;

// ΩV1-F2: post-response learned-expression shadow observation. The live HTTP
// response is frozen first; only typed intent-derived semantics, sealed VoiceState
// projection data, bounded fingerprints, and metadata enter the isolated worker.
#[cfg(feature = "omega-v1-f2-shadow")]
pub mod omega_v1f2_shadow;

// ΩV1-A: frozen current-voice corpus, metrics, and preregistration gate.
// Disabled by default and evaluation-only. It cannot influence Runtime::chat(),
// mutate voice or companion state, promote beliefs or ontology, select tools,
// discharge CHARGE, or authorize autonomous action.
#[cfg(feature = "omega-v1-baseline")]
pub mod omega_v1_voice_baseline;

// ΩV1-B: typed persistent VoiceState, deterministic serialization, optimistic
// versioning, exact replay, bounded dimensions, and debug projection. Disabled
// by default and shadow-only: no Runtime::chat(), VoiceEngine, persistence,
// belief, ontology, routing, tool, CHARGE, or autonomous-action wiring.
#[cfg(feature = "voice-state-shadow")]
pub mod voice_state;

// ΩV1-C: complete typed SemanticResponsePlan migration over the frozen corpus
// and transitional handler boundary. It runs the old and new paths in matched
// shadow mode and retains exact neutral compatibility text. No live response,
// voice-state, memory, belief, ontology, routing, tool, CHARGE, or action authority.
#[cfg(feature = "omega-v1-semantic-plan")]
pub mod omega_v1_semantic_plan;

// ΩV1-D0: deterministic bounded opener-substitution kernel. The protected
// response body remains byte-exact and every failure returns the neutral text.
// This stage is probe-only until ΩV1-D1 explicitly wires the HTTP chat boundary.
#[cfg(feature = "omega-v1-live-bridge")]
pub mod omega_v1_live_bridge;

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
