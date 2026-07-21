use super::types::{
    AcceptedSpan, AriseConfig, AriseError, AriseExecutionTrace, AriseRequest,
    AriseTerminalClassification, ObligationId, PlannedSpan, RejectedSpan, ReversePlan,
    SemanticObligation, TransitionVerification, VerificationReason, MAX_CONFIG_OBLIGATIONS,
    MAX_CONFIG_OBLIGATIONS_PER_SPAN, MAX_CONFIG_REPAIR_DEPTH, MAX_CONFIG_SPAN_BYTES,
    MAX_PROHIBITED_FRAGMENT_BYTES, MAX_SEMANTIC_KEY_BYTES, MAX_WITNESS_BYTES,
};
use std::collections::{BTreeMap, BTreeSet};

pub struct TransitionInput<'a> {
    pub request: &'a AriseRequest,
    pub expected: &'a [SemanticObligation],
    pub satisfied: &'a BTreeSet<ObligationId>,
    pub span: &'a str,
    pub maximum_span_bytes: usize,
}

pub trait SpanRenderer {
    fn render(
        &self,
        obligations: &[SemanticObligation],
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
        obligations: &[SemanticObligation],
        maximum_span_bytes: usize,
    ) -> Result<String, AriseError> {
        let mut rendered = String::new();
        for obligation in obligations {
            if !rendered.is_empty() {
                rendered.push_str(". ");
            }
            rendered.push_str(
                obligation
                    .witness
                    .trim()
                    .trim_end_matches(['.', '!', '?']),
            );
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
                .copied()
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
        let required = plan
            .ordered_obligations
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
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
                &required,
                &span.obligations,
                0,
                &mut satisfied,
                &mut accepted_spans,
                &mut rejected_spans,
                &mut repair_count,
            )? {
                failed = true;
                break;
            }
        }

        let final_residual = residual_count(&required, &satisfied);
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
        required: &BTreeSet<ObligationId>,
        ids: &[ObligationId],
        depth: u8,
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
                    .cloned()
                    .ok_or(AriseError::UnknownObligationReference)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let text = match self
            .renderer
            .render(&expected, self.config.maximum_span_bytes)
        {
            Ok(text) => text,
            Err(_) => {
                rejected_spans.push(RejectedSpan {
                    obligations: ids.to_vec(),
                    text: String::new(),
                    reason: VerificationReason::RendererFailure,
                    repair_depth: depth,
                });
                return self.repair_or_reject(
                    request,
                    obligations,
                    required,
                    ids,
                    depth,
                    satisfied,
                    accepted_spans,
                    rejected_spans,
                    repair_count,
                );
            }
        };
        let verification = self.verifier.verify(TransitionInput {
            request,
            expected: &expected,
            satisfied,
            span: &text,
            maximum_span_bytes: self.config.maximum_span_bytes,
        });

        if verification.accepted {
            let residual_before = residual_count(required, satisfied);
            satisfied.extend(ids.iter().copied());
            let residual_after = residual_count(required, satisfied);
            accepted_spans.push(AcceptedSpan {
                obligations: ids.to_vec(),
                text,
                residual_before,
                residual_after,
                repair_depth: depth,
            });
            return Ok(residual_after < residual_before);
        }

        rejected_spans.push(RejectedSpan {
            obligations: ids.to_vec(),
            text,
            reason: verification.reason,
            repair_depth: depth,
        });
        self.repair_or_reject(
            request,
            obligations,
            required,
            ids,
            depth,
            satisfied,
            accepted_spans,
            rejected_spans,
            repair_count,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn repair_or_reject(
        &self,
        request: &AriseRequest,
        obligations: &BTreeMap<ObligationId, &SemanticObligation>,
        required: &BTreeSet<ObligationId>,
        ids: &[ObligationId],
        depth: u8,
        satisfied: &mut BTreeSet<ObligationId>,
        accepted_spans: &mut Vec<AcceptedSpan>,
        rejected_spans: &mut Vec<RejectedSpan>,
        repair_count: &mut u32,
    ) -> Result<bool, AriseError> {
        if ids.len() <= 1 || depth >= self.config.maximum_repair_depth {
            return Ok(false);
        }

        *repair_count = repair_count.saturating_add(1);
        let middle = ids.len() / 2;
        let left = self.execute_group(
            request,
            obligations,
            required,
            &ids[..middle],
            depth.saturating_add(1),
            satisfied,
            accepted_spans,
            rejected_spans,
            repair_count,
        )?;
        if !left {
            return Ok(false);
        }
        self.execute_group(
            request,
            obligations,
            required,
            &ids[middle..],
            depth.saturating_add(1),
            satisfied,
            accepted_spans,
            rejected_spans,
            repair_count,
        )
    }
}

fn residual_count(required: &BTreeSet<ObligationId>, satisfied: &BTreeSet<ObligationId>) -> usize {
    required
        .iter()
        .filter(|obligation| !satisfied.contains(obligation))
        .count()
}

fn validate_config(config: AriseConfig) -> Result<(), AriseError> {
    if config.maximum_obligations == 0
        || config.maximum_obligations > MAX_CONFIG_OBLIGATIONS
        || config.maximum_obligations_per_span == 0
        || config.maximum_obligations_per_span > MAX_CONFIG_OBLIGATIONS_PER_SPAN
        || config.maximum_obligations_per_span > config.maximum_obligations
        || config.maximum_span_bytes == 0
        || config.maximum_span_bytes > MAX_CONFIG_SPAN_BYTES
        || config.maximum_repair_depth > MAX_CONFIG_REPAIR_DEPTH
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
    if !valid_label(&request.intent_label) {
        return Err(AriseError::InvalidTextBoundary);
    }

    let mut obligations = BTreeMap::new();
    for obligation in &request.obligations {
        if obligation.id.0 == 0 || obligations.insert(obligation.id, obligation).is_some() {
            return Err(AriseError::InvalidObligationId);
        }
        if !valid_semantic_key(&obligation.semantic_key)
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
    validate_all_acyclic(&obligations)?;

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

    let mut prohibited = BTreeSet::new();
    for fragment in &request.prohibited_fragments {
        if !valid_bounded_text(fragment, MAX_PROHIBITED_FRAGMENT_BYTES)
            || !prohibited.insert(fragment.trim().to_ascii_lowercase())
        {
            return Err(AriseError::InvalidTextBoundary);
        }
    }
    inverse_witness_map(&request.obligations)?;

    Ok(obligations)
}

fn validate_all_acyclic(
    obligations: &BTreeMap<ObligationId, &SemanticObligation>,
) -> Result<(), AriseError> {
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for id in obligations.keys() {
        cycle_visit(*id, obligations, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn cycle_visit(
    id: ObligationId,
    obligations: &BTreeMap<ObligationId, &SemanticObligation>,
    visiting: &mut BTreeSet<ObligationId>,
    visited: &mut BTreeSet<ObligationId>,
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
        cycle_visit(*dependency, obligations, visiting, visited)?;
    }
    visiting.remove(&id);
    visited.insert(id);
    Ok(())
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
        reverse_visit(*dependency, obligations, visiting, visited, ordered)?;
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

fn valid_label(value: &str) -> bool {
    valid_bounded_text(value, 80)
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

fn valid_semantic_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_SEMANTIC_KEY_BYTES
        && value.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '_' | '-')
        })
}

fn valid_bounded_text(value: &str, maximum_bytes: usize) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.len() <= maximum_bytes && !trimmed.chars().any(char::is_control)
}

pub(crate) fn canonical_clause(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(['.', '!', '?'])
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub(crate) fn split_clauses(value: &str) -> Vec<&str> {
    value
        .split(['.', '!', '?'])
        .map(str::trim)
        .filter(|clause| !clause.is_empty())
        .collect()
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
        assert_eq!(
            trace.terminal_classification,
            AriseTerminalClassification::Pass
        );
        assert_eq!(trace.final_residual, 0);
        assert!(trace
            .accepted_spans
            .iter()
            .all(|span| span.residual_after < span.residual_before));
    }

    #[test]
    fn residual_accounting_does_not_depend_on_initial_vector_order() {
        let mut request = chain_request();
        request.initially_satisfied = vec![ObligationId(2), ObligationId(1)];
        let trace = AriseEngine::default()
            .execute(&request)
            .expect("unsorted initial state should remain valid");
        assert_eq!(trace.plan.initial_residual, 1);
        assert_eq!(trace.final_residual, 0);
    }

    #[derive(Debug, Clone, Copy)]
    struct DropLastWhenGrouped;

    impl SpanRenderer for DropLastWhenGrouped {
        fn render(
            &self,
            obligations: &[SemanticObligation],
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
        assert_eq!(
            trace.terminal_classification,
            AriseTerminalClassification::Pass
        );
        assert_eq!(trace.final_residual, 0);
        assert_eq!(trace.repair_count, 1);
        assert_eq!(trace.accepted_spans.len(), 2);
        assert_eq!(trace.rejected_spans.len(), 1);
    }

    #[test]
    fn disconnected_dependency_cycle_is_rejected() {
        let request = AriseRequest {
            trace_id: 9,
            intent_label: "invalid".to_string(),
            terminal_obligations: vec![ObligationId(1)],
            initially_satisfied: Vec::new(),
            obligations: vec![
                obligation(1, &[], "Reachable"),
                obligation(2, &[3], "Cycle left"),
                obligation(3, &[2], "Cycle right"),
            ],
            prohibited_fragments: Vec::new(),
        };
        assert_eq!(
            AriseEngine::default().plan(&request),
            Err(AriseError::InvalidDependencyGraph)
        );
    }

    #[test]
    fn verifier_rejects_unauthorized_surface() {
        let request = chain_request();
        let expected = vec![request.obligations[0].clone()];
        let verification = LexicalTransitionVerifier.verify(TransitionInput {
            request: &request,
            expected: &expected,
            satisfied: &BTreeSet::new(),
            span: "Moist air rises. An unsupported claim appears.",
            maximum_span_bytes: 512,
        });
        assert!(!verification.accepted);
        assert_eq!(verification.reason, VerificationReason::UnsupportedSurface);
    }
}
