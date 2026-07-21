//! ARISE-A0: bounded reverse-obligation planning and verified span execution.
//!
//! ARISE plans from terminal semantic obligations backward through explicit
//! dependencies, then realizes bounded spans forward. Every span is accepted
//! only after an independent verifier reconstructs its obligations from text.
//! The module is fixed-capacity, deterministic, feature-gated, and has no
//! authority to alter runtime text, memory, routing, tools, CHARGE, or actions.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, OnceLock};
use thiserror::Error;

const MAX_SEMANTIC_KEY_BYTES: usize = 160;
const MAX_WITNESS_BYTES: usize = 512;
const MAX_PROHIBITED_FRAGMENT_BYTES: usize = 160;
const RUNTIME_MAX_SEGMENTS: usize = 16;
const RUNTIME_PIPELINE: &str = "arise-a0-runtime-shadow-v1";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
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

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AriseError {
    #[error("ARISE configuration is invalid")]
    InvalidConfig,
    #[error("ARISE request exceeds its bounded obligation capacity")]
    ObligationCapacityExceeded,
    #[error("obligation identifiers must be nonzero and unique")]
    InvalidObligationId,
    #[error("terminal and initially satisfied identifiers must reference known obligations")]
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

pub struct TransitionInput<'a> {
    pub request: &'a AriseRequest,
    pub expected: &'a [&'a SemanticObligation],
    pub satisfied: &'a BTreeSet<ObligationId>,
    pub span: &'a str,
    pub maximum_span_bytes: usize,
}

pub trait SpanRenderer {
    fn render(
        &self,
        obligations: &[&SemanticObligation],
        maximum_span_bytes: usize,
    ) -> Result<String, AriseError>;
}

pub trait TransitionVerifier {
    fn verify(&self, input: TransitionInput<'_>) -> TransitionVerification;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LexicalSpanRenderer;

impl SpanRenderer for LexicalSpanRenderer {
    fn render(
        &self,
        obligations: &[&SemanticObligation],
        maximum_span_bytes: usize,
    ) -> Result<String, AriseError> {
        let mut rendered = String::new();
        for obligation in obligations {
            if !rendered.is_empty() {
                rendered.push_str(". ");
            }
            rendered.push_str(obligation.witness.trim().trim_end_matches(['.', '!', '?']));
        }
        if !rendered.is_empty() {
            rendered.push('.');
        }
        if rendered.len() > maximum_span_bytes {
            return Err(AriseError::Renderer(
                "rendered span exceeded the configured byte budget".to_string(),
            ));
        }
        Ok(rendered)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LexicalTransitionVerifier;

impl TransitionVerifier for LexicalTransitionVerifier {
    fn verify(&self, input: TransitionInput<'_>) -> TransitionVerification {
        if input.span.trim().is_empty() {
            return rejected(VerificationReason::EmptySpan);
        }
        if input.span.len() > input.maximum_span_bytes {
            return rejected(VerificationReason::SpanBudgetExceeded);
        }

        let lower_span = input.span.to_ascii_lowercase();
        if input
            .request
            .prohibited_fragments
            .iter()
            .any(|fragment| lower_span.contains(&fragment.to_ascii_lowercase()))
        {
            return rejected(VerificationReason::ProhibitedSurface);
        }

        for obligation in input.expected {
            if obligation
                .dependencies
                .iter()
                .any(|dependency| !input.satisfied.contains(dependency))
            {
                return rejected(VerificationReason::DependencyUnsatisfied);
            }
        }

        let inverse = match inverse_witness_map(&input.request.obligations) {
            Ok(inverse) => inverse,
            Err(_) => return rejected(VerificationReason::UnsupportedSurface),
        };
        let mut reconstructed = Vec::new();
        for clause in split_clauses(input.span) {
            let canonical = canonical_clause(clause);
            let Some(obligation) = inverse.get(&canonical) else {
                return rejected(VerificationReason::UnsupportedSurface);
            };
            reconstructed.push(*obligation);
        }

        let expected = input
            .expected
            .iter()
            .map(|obligation| obligation.id)
            .collect::<Vec<_>>();
        if reconstructed.iter().any(|id| !expected.contains(id)) {
            return TransitionVerification {
                accepted: false,
                reconstructed,
                reason: VerificationReason::UnexpectedObligation,
            };
        }
        if reconstructed.len() < expected.len() {
            return TransitionVerification {
                accepted: false,
                reconstructed,
                reason: VerificationReason::MissingObligation,
            };
        }
        if reconstructed != expected {
            return TransitionVerification {
                accepted: false,
                reconstructed,
                reason: VerificationReason::UnexpectedObligation,
            };
        }

        TransitionVerification {
            accepted: true,
            reconstructed,
            reason: VerificationReason::Pass,
        }
    }
}

fn rejected(reason: VerificationReason) -> TransitionVerification {
    TransitionVerification {
        accepted: false,
        reconstructed: Vec::new(),
        reason,
    }
}

#[derive(Debug, Clone)]
pub struct AriseEngine<R = LexicalSpanRenderer, V = LexicalTransitionVerifier> {
    config: AriseConfig,
    renderer: R,
    verifier: V,
}

impl Default for AriseEngine<LexicalSpanRenderer, LexicalTransitionVerifier> {
    fn default() -> Self {
        Self {
            config: AriseConfig::default(),
            renderer: LexicalSpanRenderer,
            verifier: LexicalTransitionVerifier,
        }
    }
}

impl<R, V> AriseEngine<R, V>
where
    R: SpanRenderer,
    V: TransitionVerifier,
{
    pub fn new(config: AriseConfig, renderer: R, verifier: V) -> Result<Self, AriseError> {
        validate_config(config)?;
        Ok(Self {
            config,
            renderer,
            verifier,
        })
    }

    pub fn plan(&self, request: &AriseRequest) -> Result<ReversePlan, AriseError> {
        let obligations = validate_request(request, self.config)?;
        let initially_satisfied = request
            .initially_satisfied
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut visiting = BTreeSet::new();
        let mut visited = initially_satisfied.clone();
        let mut ordered = Vec::new();

        for terminal in &request.terminal_obligations {
            reverse_visit(
                *terminal,
                &obligations,
                &mut visiting,
                &mut visited,
                &mut ordered,
            )?;
        }

        let mut spans = Vec::new();
        let mut planned_satisfied = initially_satisfied;
        let mut current = Vec::new();
        for id in &ordered {
            let obligation = obligations
                .get(id)
                .ok_or(AriseError::UnknownObligationReference)?;
            let dependencies_ready = obligation
                .dependencies
                .iter()
                .all(|dependency| planned_satisfied.contains(dependency));
            if !current.is_empty()
                && (!dependencies_ready
                    || current.len() >= self.config.maximum_obligations_per_span)
            {
                planned_satisfied.extend(current.iter().copied());
                spans.push(PlannedSpan {
                    obligations: std::mem::take(&mut current),
                });
            }
            current.push(*id);
        }
        if !current.is_empty() {
            spans.push(PlannedSpan {
                obligations: current,
            });
        }

        Ok(ReversePlan {
            initial_residual: ordered.len(),
            ordered_obligations: ordered,
            spans,
        })
    }

    pub fn execute(&self, request: &AriseRequest) -> Result<AriseExecutionTrace, AriseError> {
        let plan = self.plan(request)?;
        let obligations = request
            .obligations
            .iter()
            .map(|obligation| (obligation.id, obligation))
            .collect::<BTreeMap<_, _>>();
        let mut satisfied = request
            .initially_satisfied
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut accepted_spans = Vec::new();
        let mut rejected_spans = Vec::new();
        let mut repair_count = 0u32;
        let mut failed = false;

        for span in &plan.spans {
            if !self.execute_group(
                request,
                &obligations,
                &span.obligations,
                0,
                plan.initial_residual,
                &mut satisfied,
                &mut accepted_spans,
                &mut rejected_spans,
                &mut repair_count,
            )? {
                failed = true;
                break;
            }
        }

        let final_residual = plan
            .ordered_obligations
            .iter()
            .filter(|id| !satisfied.contains(id))
            .count();
        let terminal_classification = if !failed && final_residual == 0 {
            AriseTerminalClassification::Pass
        } else {
            AriseTerminalClassification::Rejected
        };

        Ok(AriseExecutionTrace {
            trace_id: request.trace_id,
            plan,
            accepted_spans,
            rejected_spans,
            repair_count,
            final_residual,
            terminal_classification,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_group(
        &self,
        request: &AriseRequest,
        obligations: &BTreeMap<ObligationId, &SemanticObligation>,
        ids: &[ObligationId],
        depth: u8,
        initial_residual: usize,
        satisfied: &mut BTreeSet<ObligationId>,
        accepted_spans: &mut Vec<AcceptedSpan>,
        rejected_spans: &mut Vec<RejectedSpan>,
        repair_count: &mut u32,
    ) -> Result<bool, AriseError> {
        let expected = ids
            .iter()
            .map(|id| {
                obligations
                    .get(id)
                    .copied()
                    .ok_or(AriseError::UnknownObligationReference)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let text = self
            .renderer
            .render(&expected, self.config.maximum_span_bytes)?;
        let verification = self.verifier.verify(TransitionInput {
            request,
            expected: &expected,
            satisfied,
            span: &text,
            maximum_span_bytes: self.config.maximum_span_bytes,
        });

        if verification.accepted {
            let residual_before = initial_residual.saturating_sub(
                satisfied
                    .iter()
                    .filter(|id| request.initially_satisfied.binary_search(id).is_err())
                    .count(),
            );
            satisfied.extend(ids.iter().copied());
            let residual_after = initial_residual.saturating_sub(
                satisfied
                    .iter()
                    .filter(|id| request.initially_satisfied.binary_search(id).is_err())
                    .count(),
            );
            accepted_spans.push(AcceptedSpan {
                obligations: ids.to_vec(),
                text,
                residual_before,
                residual_after,
                repair_depth: depth,
            });
            return Ok(residual_after < residual_before || ids.is_empty());
        }

        rejected_spans.push(RejectedSpan {
            obligations: ids.to_vec(),
            text,
            reason: verification.reason,
            repair_depth: depth,
        });

        if ids.len() > 1 && depth < self.config.maximum_repair_depth {
            *repair_count = repair_count.saturating_add(1);
            let middle = ids.len() / 2;
            let left = self.execute_group(
                request,
                obligations,
                &ids[..middle],
                depth.saturating_add(1),
                initial_residual,
                satisfied,
                accepted_spans,
                rejected_spans,
                repair_count,
            )?;
            if !left {
                return Ok(false);
            }
            return self.execute_group(
                request,
                obligations,
                &ids[middle..],
                depth.saturating_add(1),
                initial_residual,
                satisfied,
                accepted_spans,
                rejected_spans,
                repair_count,
            );
        }

        Ok(false)
    }
}

fn validate_config(config: AriseConfig) -> Result<(), AriseError> {
    if config.maximum_obligations == 0
        || config.maximum_obligations_per_span == 0
        || config.maximum_obligations_per_span > config.maximum_obligations
        || config.maximum_span_bytes == 0
    {
        return Err(AriseError::InvalidConfig);
    }
    Ok(())
}

fn validate_request(
    request: &AriseRequest,
    config: AriseConfig,
) -> Result<BTreeMap<ObligationId, &SemanticObligation>, AriseError> {
    validate_config(config)?;
    if request.obligations.is_empty() || request.obligations.len() > config.maximum_obligations {
        return Err(AriseError::ObligationCapacityExceeded);
    }
    if request.intent_label.trim().is_empty() {
        return Err(AriseError::InvalidTextBoundary);
    }

    let mut obligations = BTreeMap::new();
    for obligation in &request.obligations {
        if obligation.id.0 == 0 || obligations.insert(obligation.id, obligation).is_some() {
            return Err(AriseError::InvalidObligationId);
        }
        if !valid_bounded_text(&obligation.semantic_key, MAX_SEMANTIC_KEY_BYTES)
            || !valid_bounded_text(&obligation.witness, MAX_WITNESS_BYTES)
        {
            return Err(AriseError::InvalidTextBoundary);
        }
        let mut dependencies = BTreeSet::new();
        for dependency in &obligation.dependencies {
            if dependency.0 == 0
                || *dependency == obligation.id
                || !dependencies.insert(*dependency)
            {
                return Err(AriseError::InvalidDependencyGraph);
            }
        }
    }

    for obligation in obligations.values() {
        if obligation
            .dependencies
            .iter()
            .any(|dependency| !obligations.contains_key(dependency))
        {
            return Err(AriseError::InvalidDependencyGraph);
        }
    }
    if request.terminal_obligations.is_empty()
        || request
            .terminal_obligations
            .iter()
            .chain(request.initially_satisfied.iter())
            .any(|id| !obligations.contains_key(id))
    {
        return Err(AriseError::UnknownObligationReference);
    }
    let mut terminal = BTreeSet::new();
    if request
        .terminal_obligations
        .iter()
        .any(|id| !terminal.insert(*id))
    {
        return Err(AriseError::UnknownObligationReference);
    }
    let mut initial = BTreeSet::new();
    if request
        .initially_satisfied
        .iter()
        .any(|id| !initial.insert(*id))
    {
        return Err(AriseError::UnknownObligationReference);
    }
    for fragment in &request.prohibited_fragments {
        if !valid_bounded_text(fragment, MAX_PROHIBITED_FRAGMENT_BYTES) {
            return Err(AriseError::InvalidTextBoundary);
        }
    }
    inverse_witness_map(&request.obligations)?;

    Ok(obligations)
}

fn reverse_visit(
    id: ObligationId,
    obligations: &BTreeMap<ObligationId, &SemanticObligation>,
    visiting: &mut BTreeSet<ObligationId>,
    visited: &mut BTreeSet<ObligationId>,
    ordered: &mut Vec<ObligationId>,
) -> Result<(), AriseError> {
    if visited.contains(&id) {
        return Ok(());
    }
    if !visiting.insert(id) {
        return Err(AriseError::InvalidDependencyGraph);
    }
    let obligation = obligations
        .get(&id)
        .copied()
        .ok_or(AriseError::UnknownObligationReference)?;
    for dependency in &obligation.dependencies {
        reverse_visit(
            *dependency,
            obligations,
            visiting,
            visited,
            ordered,
        )?;
    }
    visiting.remove(&id);
    visited.insert(id);
    ordered.push(id);
    Ok(())
}

fn inverse_witness_map(
    obligations: &[SemanticObligation],
) -> Result<BTreeMap<String, ObligationId>, AriseError> {
    let mut inverse = BTreeMap::new();
    for obligation in obligations {
        let canonical = canonical_clause(&obligation.witness);
        if canonical.is_empty() || inverse.insert(canonical, obligation.id).is_some() {
            return Err(AriseError::AmbiguousWitness);
        }
    }
    Ok(inverse)
}

fn valid_bounded_text(value: &str, maximum_bytes: usize) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.len() <= maximum_bytes
        && !trimmed.chars().any(char::is_control)
}

fn canonical_clause(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(['.', '!', '?'])
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn split_clauses(value: &str) -> Vec<&str> {
    value
        .split(['.', '!', '?'])
        .map(str::trim)
        .filter(|clause| !clause.is_empty())
        .collect()
}

fn stable_digest(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AriseRuntimeSnapshot {
    pub enabled: bool,
    pub pipeline: &'static str,
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
            pipeline: RUNTIME_PIPELINE,
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

static RUNTIME_SNAPSHOT: OnceLock<Mutex<AriseRuntimeSnapshot>> = OnceLock::new();

fn runtime_snapshot_store() -> &'static Mutex<AriseRuntimeSnapshot> {
    RUNTIME_SNAPSHOT.get_or_init(|| Mutex::new(AriseRuntimeSnapshot::default()))
}

pub fn observe_runtime_response(intent_label: &str, body: &str) -> AriseRuntimeSnapshot {
    let next_trace_id = runtime_snapshot_store()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .trace_id
        .saturating_add(1);
    let clauses = split_clauses(body);
    let snapshot = if clauses.is_empty() || clauses.len() > RUNTIME_MAX_SEGMENTS {
        rejected_runtime_snapshot(next_trace_id, intent_label, body)
    } else {
        let mut obligations = Vec::with_capacity(clauses.len());
        for (index, clause) in clauses.iter().enumerate() {
            let id = ObligationId(u16::try_from(index + 1).unwrap_or(u16::MAX));
            let dependencies = if index == 0 {
                Vec::new()
            } else {
                vec![ObligationId(u16::try_from(index).unwrap_or(u16::MAX))]
            };
            obligations.push(SemanticObligation {
                id,
                semantic_key: format!("runtime.{intent_label}.segment.{}", index + 1),
                dependencies,
                witness: (*clause).to_string(),
            });
        }
        let terminal = obligations.last().map(|obligation| obligation.id);
        let request = AriseRequest {
            trace_id: next_trace_id,
            intent_label: intent_label.to_string(),
            terminal_obligations: terminal.into_iter().collect(),
            initially_satisfied: Vec::new(),
            obligations,
            prohibited_fragments: Vec::new(),
        };
        let config = AriseConfig {
            maximum_obligations: RUNTIME_MAX_SEGMENTS,
            maximum_obligations_per_span: 2,
            maximum_span_bytes: MAX_WITNESS_BYTES,
            maximum_repair_depth: 4,
        };
        match AriseEngine::new(config, LexicalSpanRenderer, LexicalTransitionVerifier)
            .and_then(|engine| engine.execute(&request))
        {
            Ok(trace) => AriseRuntimeSnapshot {
                enabled: true,
                pipeline: RUNTIME_PIPELINE,
                trace_id: next_trace_id,
                intent_label: intent_label.to_string(),
                body_digest: stable_digest(body),
                span_count: trace.accepted_spans.len(),
                repair_count: trace.repair_count,
                initial_residual: trace.plan.initial_residual,
                final_residual: trace.final_residual,
                terminal_classification: trace.terminal_classification,
                authority: authority_boundary(),
            },
            Err(_) => rejected_runtime_snapshot(next_trace_id, intent_label, body),
        }
    };

    let mut stored = runtime_snapshot_store()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *stored = snapshot.clone();
    snapshot
}

fn rejected_runtime_snapshot(
    trace_id: u64,
    intent_label: &str,
    body: &str,
) -> AriseRuntimeSnapshot {
    AriseRuntimeSnapshot {
        enabled: true,
        pipeline: RUNTIME_PIPELINE,
        trace_id,
        intent_label: intent_label.to_string(),
        body_digest: stable_digest(body),
        ..AriseRuntimeSnapshot::default()
    }
}

#[must_use]
pub fn live_runtime_snapshot() -> AriseRuntimeSnapshot {
    runtime_snapshot_store()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obligation(id: u16, dependencies: &[u16], witness: &str) -> SemanticObligation {
        SemanticObligation {
            id: ObligationId(id),
            semantic_key: format!("claim.{id}"),
            dependencies: dependencies.iter().copied().map(ObligationId).collect(),
            witness: witness.to_string(),
        }
    }

    fn chain_request() -> AriseRequest {
        AriseRequest {
            trace_id: 7,
            intent_label: "explanation".to_string(),
            terminal_obligations: vec![ObligationId(3)],
            initially_satisfied: Vec::new(),
            obligations: vec![
                obligation(1, &[], "Moist air rises"),
                obligation(2, &[1], "Rising air cools"),
                obligation(3, &[2], "Water vapor condenses"),
            ],
            prohibited_fragments: vec!["unsupported certainty".to_string()],
        }
    }

    #[test]
    fn reverse_plan_walks_dependencies_before_terminal() {
        let engine = AriseEngine::default();
        let plan = engine.plan(&chain_request()).expect("plan should validate");
        assert_eq!(
            plan.ordered_obligations,
            vec![ObligationId(1), ObligationId(2), ObligationId(3)]
        );
        assert_eq!(plan.initial_residual, 3);
    }

    #[test]
    fn accepted_spans_reduce_residual_monotonically() {
        let trace = AriseEngine::default()
            .execute(&chain_request())
            .expect("execution should pass");
        assert_eq!(trace.terminal_classification, AriseTerminalClassification::Pass);
        assert_eq!(trace.final_residual, 0);
        assert!(trace
            .accepted_spans
            .iter()
            .all(|span| span.residual_after < span.residual_before));
    }

    #[derive(Debug, Clone, Copy)]
    struct DropLastWhenGrouped;

    impl SpanRenderer for DropLastWhenGrouped {
        fn render(
            &self,
            obligations: &[&SemanticObligation],
            maximum_span_bytes: usize,
        ) -> Result<String, AriseError> {
            let kept = if obligations.len() > 1 {
                &obligations[..obligations.len() - 1]
            } else {
                obligations
            };
            LexicalSpanRenderer.render(kept, maximum_span_bytes)
        }
    }

    #[test]
    fn failed_group_recursively_splits_and_repairs_locally() {
        let request = AriseRequest {
            trace_id: 8,
            intent_label: "contrast".to_string(),
            terminal_obligations: vec![ObligationId(1), ObligationId(2)],
            initially_satisfied: Vec::new(),
            obligations: vec![
                obligation(1, &[], "The left condition holds"),
                obligation(2, &[], "The right condition differs"),
            ],
            prohibited_fragments: Vec::new(),
        };
        let engine = AriseEngine::new(
            AriseConfig::default(),
            DropLastWhenGrouped,
            LexicalTransitionVerifier,
        )
        .expect("config should validate");
        let trace = engine.execute(&request).expect("repair should complete");
        assert_eq!(trace.terminal_classification, AriseTerminalClassification::Pass);
        assert_eq!(trace.final_residual, 0);
        assert_eq!(trace.repair_count, 1);
        assert_eq!(trace.accepted_spans.len(), 2);
        assert_eq!(trace.rejected_spans.len(), 1);
    }

    #[test]
    fn dependency_cycle_is_rejected() {
        let request = AriseRequest {
            trace_id: 9,
            intent_label: "invalid".to_string(),
            terminal_obligations: vec![ObligationId(1)],
            initially_satisfied: Vec::new(),
            obligations: vec![
                obligation(1, &[2], "First"),
                obligation(2, &[1], "Second"),
            ],
            prohibited_fragments: Vec::new(),
        };
        assert_eq!(
            AriseEngine::default().plan(&request),
            Err(AriseError::InvalidDependencyGraph)
        );
    }

    #[test]
    fn runtime_shadow_is_inert_and_bounded() {
        let snapshot = observe_runtime_response(
            "teaching",
            "Moist air rises. Rising air cools. Water vapor condenses.",
        );
        assert_eq!(snapshot.terminal_classification, AriseTerminalClassification::Pass);
        assert_eq!(snapshot.final_residual, 0);
        assert!(!snapshot.authority.generated_text_influence);
        assert!(!snapshot.authority.persistence_authority);
        assert!(!snapshot.authority.charge_discharge_authority);
        assert!(!snapshot.authority.autonomous_action_authority);
    }
}
