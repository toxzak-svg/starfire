use serde::{Deserialize, Serialize};
use thiserror::Error;

pub(crate) const MAX_SEMANTIC_KEY_BYTES: usize = 160;
pub(crate) const MAX_WITNESS_BYTES: usize = 512;
pub(crate) const MAX_PROHIBITED_FRAGMENT_BYTES: usize = 160;
pub(crate) const MAX_CONFIG_OBLIGATIONS: usize = 256;
pub(crate) const MAX_CONFIG_OBLIGATIONS_PER_SPAN: usize = 16;
pub(crate) const MAX_CONFIG_SPAN_BYTES: usize = 4_096;
pub(crate) const MAX_CONFIG_REPAIR_DEPTH: u8 = 8;
pub(crate) const RUNTIME_MAX_SEGMENTS: usize = 16;
pub(crate) const RUNTIME_PIPELINE: &str = "arise-a0-runtime-shadow-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObligationId(pub u16);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticObligation {
    pub id: ObligationId,
    pub semantic_key: String,
    pub dependencies: Vec<ObligationId>,
    pub witness: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AriseRequest {
    pub trace_id: u64,
    pub intent_label: String,
    pub terminal_obligations: Vec<ObligationId>,
    pub initially_satisfied: Vec<ObligationId>,
    pub obligations: Vec<SemanticObligation>,
    pub prohibited_fragments: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AriseConfig {
    pub maximum_obligations: usize,
    pub maximum_obligations_per_span: usize,
    pub maximum_span_bytes: usize,
    pub maximum_repair_depth: u8,
}

impl Default for AriseConfig {
    fn default() -> Self {
        Self {
            maximum_obligations: 32,
            maximum_obligations_per_span: 4,
            maximum_span_bytes: 512,
            maximum_repair_depth: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedSpan {
    pub obligations: Vec<ObligationId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReversePlan {
    pub ordered_obligations: Vec<ObligationId>,
    pub spans: Vec<PlannedSpan>,
    pub initial_residual: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationReason {
    Pass,
    EmptySpan,
    RendererFailure,
    SpanBudgetExceeded,
    ProhibitedSurface,
    UnsupportedSurface,
    UnexpectedObligation,
    MissingObligation,
    DependencyUnsatisfied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionVerification {
    pub accepted: bool,
    pub reconstructed: Vec<ObligationId>,
    pub reason: VerificationReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedSpan {
    pub obligations: Vec<ObligationId>,
    pub text: String,
    pub residual_before: usize,
    pub residual_after: usize,
    pub repair_depth: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectedSpan {
    pub obligations: Vec<ObligationId>,
    pub text: String,
    pub reason: VerificationReason,
    pub repair_depth: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AriseTerminalClassification {
    Pass,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AriseExecutionTrace {
    pub trace_id: u64,
    pub plan: ReversePlan,
    pub accepted_spans: Vec<AcceptedSpan>,
    pub rejected_spans: Vec<RejectedSpan>,
    pub repair_count: u32,
    pub final_residual: usize,
    pub terminal_classification: AriseTerminalClassification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AriseAuthorityBoundary {
    pub runtime_shadow_observation: bool,
    pub generated_text_influence: bool,
    pub raw_prompt_access: bool,
    pub memory_access: bool,
    pub persistence_authority: bool,
    pub routing_authority: bool,
    pub belief_promotion_authority: bool,
    pub ontology_promotion_authority: bool,
    pub tool_selection_authority: bool,
    pub charge_discharge_authority: bool,
    pub autonomous_action_authority: bool,
}

#[must_use]
pub const fn authority_boundary() -> AriseAuthorityBoundary {
    AriseAuthorityBoundary {
        runtime_shadow_observation: true,
        generated_text_influence: false,
        raw_prompt_access: false,
        memory_access: false,
        persistence_authority: false,
        routing_authority: false,
        belief_promotion_authority: false,
        ontology_promotion_authority: false,
        tool_selection_authority: false,
        charge_discharge_authority: false,
        autonomous_action_authority: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AriseRuntimeSnapshot {
    pub enabled: bool,
    pub pipeline: String,
    pub trace_id: u64,
    pub intent_label: String,
    pub body_digest: u64,
    pub span_count: usize,
    pub repair_count: u32,
    pub initial_residual: usize,
    pub final_residual: usize,
    pub terminal_classification: AriseTerminalClassification,
    pub authority: AriseAuthorityBoundary,
}

impl Default for AriseRuntimeSnapshot {
    fn default() -> Self {
        Self {
            enabled: true,
            pipeline: RUNTIME_PIPELINE.to_string(),
            trace_id: 0,
            intent_label: "unknown".to_string(),
            body_digest: 0,
            span_count: 0,
            repair_count: 0,
            initial_residual: 0,
            final_residual: 0,
            terminal_classification: AriseTerminalClassification::Rejected,
            authority: authority_boundary(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AriseError {
    #[error("ARISE configuration is invalid or exceeds frozen edge bounds")]
    InvalidConfig,
    #[error("ARISE request exceeds its bounded obligation capacity")]
    ObligationCapacityExceeded,
    #[error("obligation identifiers must be nonzero and unique")]
    InvalidObligationId,
    #[error("terminal and initially satisfied identifiers must be unique and known")]
    UnknownObligationReference,
    #[error("semantic keys, witnesses, or prohibited fragments are malformed")]
    InvalidTextBoundary,
    #[error("obligation dependencies must be unique, known, and acyclic")]
    InvalidDependencyGraph,
    #[error("lexical witnesses must reconstruct to exactly one obligation")]
    AmbiguousWitness,
    #[error("span renderer failed: {0}")]
    Renderer(String),
}
