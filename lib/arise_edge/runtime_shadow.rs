use super::engine::{split_clauses, AriseEngine, LexicalSpanRenderer, LexicalTransitionVerifier};
use super::types::{
    authority_boundary, AriseConfig, AriseRequest, AriseRuntimeSnapshot, AriseTerminalClassification,
    ObligationId, SemanticObligation, MAX_WITNESS_BYTES, RUNTIME_MAX_SEGMENTS,
    RUNTIME_PIPELINE,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

static NEXT_TRACE_ID: AtomicU64 = AtomicU64::new(1);
static RUNTIME_SNAPSHOT: OnceLock<Mutex<AriseRuntimeSnapshot>> = OnceLock::new();

fn runtime_snapshot_store() -> &'static Mutex<AriseRuntimeSnapshot> {
    RUNTIME_SNAPSHOT.get_or_init(|| Mutex::new(AriseRuntimeSnapshot::default()))
}

pub fn observe_runtime_response(intent_label: &str, body: &str) -> AriseRuntimeSnapshot {
    let trace_id = NEXT_TRACE_ID.fetch_add(1, Ordering::Relaxed);
    let safe_intent = canonical_intent_label(intent_label);
    let clauses = split_clauses(body);
    let snapshot = if clauses.is_empty() || clauses.len() > RUNTIME_MAX_SEGMENTS {
        rejected_runtime_snapshot(trace_id, &safe_intent, body)
    } else {
        let obligations = clauses
            .iter()
            .enumerate()
            .map(|(index, clause)| {
                let ordinal = u16::try_from(index + 1).unwrap_or(u16::MAX);
                let dependencies = if ordinal == 1 {
                    Vec::new()
                } else {
                    vec![ObligationId(ordinal - 1)]
                };
                SemanticObligation {
                    id: ObligationId(ordinal),
                    semantic_key: format!("runtime.{safe_intent}.segment.{ordinal}"),
                    dependencies,
                    witness: (*clause).to_string(),
                }
            })
            .collect::<Vec<_>>();
        let terminal_obligations = obligations
            .last()
            .map(|obligation| obligation.id)
            .into_iter()
            .collect();
        let request = AriseRequest {
            trace_id,
            intent_label: safe_intent.clone(),
            terminal_obligations,
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
                pipeline: RUNTIME_PIPELINE.to_string(),
                trace_id,
                intent_label: safe_intent,
                body_digest: stable_digest(body),
                span_count: trace.accepted_spans.len(),
                repair_count: trace.repair_count,
                initial_residual: trace.plan.initial_residual,
                final_residual: trace.final_residual,
                terminal_classification: trace.terminal_classification,
                authority: authority_boundary(),
            },
            Err(_) => rejected_runtime_snapshot(trace_id, &safe_intent, body),
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
        pipeline: RUNTIME_PIPELINE.to_string(),
        trace_id,
        intent_label: intent_label.to_string(),
        body_digest: stable_digest(body),
        terminal_classification: AriseTerminalClassification::Rejected,
        authority: authority_boundary(),
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

fn canonical_intent_label(value: &str) -> String {
    let canonical = value
        .trim()
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                Some(character.to_ascii_lowercase())
            } else if matches!(character, '_' | '-') {
                Some(character)
            } else if character.is_ascii_whitespace() {
                Some('_')
            } else {
                None
            }
        })
        .take(80)
        .collect::<String>()
        .trim_matches(|character| matches!(character, '_' | '-'))
        .to_string();
    if canonical.is_empty() {
        "unknown".to_string()
    } else {
        canonical
    }
}

fn stable_digest(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_shadow_reconstructs_completed_text_without_authority() {
        let snapshot = observe_runtime_response(
            "Teaching",
            "Moist air rises. Rising air cools. Water vapor condenses.",
        );
        assert_eq!(snapshot.terminal_classification, AriseTerminalClassification::Pass);
        assert_eq!(snapshot.final_residual, 0);
        assert_eq!(snapshot.intent_label, "teaching");
        assert!(!snapshot.authority.generated_text_influence);
        assert!(!snapshot.authority.persistence_authority);
        assert!(!snapshot.authority.charge_discharge_authority);
        assert!(!snapshot.authority.autonomous_action_authority);
    }

    #[test]
    fn runtime_shadow_trace_ids_are_monotonic() {
        let first = observe_runtime_response("statement", "First response.");
        let second = observe_runtime_response("statement", "Second response.");
        assert!(second.trace_id > first.trace_id);
    }

    #[test]
    fn runtime_shadow_rejects_unbounded_segment_count() {
        let body = (0..=RUNTIME_MAX_SEGMENTS)
            .map(|index| format!("Segment {index}."))
            .collect::<Vec<_>>()
            .join(" ");
        let snapshot = observe_runtime_response("statement", &body);
        assert_eq!(
            snapshot.terminal_classification,
            AriseTerminalClassification::Rejected
        );
        assert_eq!(snapshot.span_count, 0);
    }
}
