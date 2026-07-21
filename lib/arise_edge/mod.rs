//! ARISE-A0: bounded reverse-obligation planning and verified span execution.
//!
//! ARISE plans from terminal semantic obligations backward through explicit
//! dependencies, then realizes bounded spans forward. Every span is accepted
//! only after an independent verifier reconstructs its obligations from text.
//! The module is fixed-capacity, deterministic, feature-gated, and has no
//! authority to alter runtime text, memory, routing, tools, CHARGE, or actions.

// Rust 1.92's manual-pattern lint flags the equivalent explicit punctuation
// closures used by the frozen A0 verifier. Keep the allowance scoped to this
// module so repository-wide Clippy debt cannot hide ARISE findings.
#[allow(clippy::manual_pattern_char_comparison)]
mod engine;
mod runtime_shadow;
mod types;

pub use engine::{
    AriseEngine, LexicalSpanRenderer, LexicalTransitionVerifier, SpanRenderer, TransitionInput,
    TransitionVerifier,
};
pub use runtime_shadow::{live_runtime_snapshot, observe_runtime_response};
pub use types::{
    authority_boundary, AcceptedSpan, AriseAuthorityBoundary, AriseConfig, AriseError,
    AriseExecutionTrace, AriseRequest, AriseRuntimeSnapshot, AriseTerminalClassification,
    ObligationId, PlannedSpan, RejectedSpan, ReversePlan, SemanticObligation,
    TransitionVerification, VerificationReason,
};
