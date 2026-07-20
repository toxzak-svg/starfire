use anyhow::{bail, Result};
use serde::Serialize;
use star::omega_v1_live_bridge::{
    authority_boundary, render_live_response, FallbackReason, LiveBridgeAuthorityBoundary,
    LiveBridgeDecision, LiveBridgeMode, ELIGIBLE_OPENER, MAX_OUTPUT_GROWTH_BYTES,
    MAX_PROTECTED_BODY_BYTES, OPENER_STEM, REPLACEMENT_OPENERS,
};

#[derive(Debug, Serialize)]
struct OmegaV1dKernelReport {
    experiment: &'static str,
    case_count: usize,
    applied_case_count: usize,
    neutral_fallback_case_count: usize,
    exact_replay: bool,
    body_preservation_rate: f64,
    ineligible_passthrough_rate: f64,
    empty_body_passthrough: bool,
    whitespace_only_passthrough: bool,
    oversized_body_passthrough: bool,
    separator_only_table: bool,
    replacement_table_max_growth_bytes: usize,
    authority_boundary: LiveBridgeAuthorityBoundary,
    no_runtime_influence: bool,
    gate_passed: bool,
}

fn body_preserved(decision: &LiveBridgeDecision) -> bool {
    if decision.mode != LiveBridgeMode::Applied {
        return true;
    }
    let Some(selected) = decision.selected_opener.as_deref() else {
        return false;
    };
    let Some(neutral_body) = decision.neutral_text.strip_prefix(ELIGIBLE_OPENER) else {
        return false;
    };
    decision.body_preserved_exactly
        && decision.rendered_text.strip_prefix(selected) == Some(neutral_body)
}

fn separator_only(opener: &str) -> bool {
    matches!(opener.strip_prefix(OPENER_STEM), Some("\n") | Some("\n\n"))
}

fn main() -> Result<()> {
    let eligible = "Here for it. The protected semantic body stays exactly the same.";
    let applied = render_live_response(eligible);
    let replay = render_live_response(eligible);

    let ineligible_text = "The protected semantic body stays exactly the same.";
    let ineligible = render_live_response(ineligible_text);

    let empty = render_live_response(ELIGIBLE_OPENER);

    let whitespace_only_text = "Here for it.  \n\t";
    let whitespace_only = render_live_response(whitespace_only_text);

    let oversized_body = "x".repeat(MAX_PROTECTED_BODY_BYTES + 1);
    let oversized_text = format!("{ELIGIBLE_OPENER}{oversized_body}");
    let oversized = render_live_response(&oversized_text);

    let decisions = [
        &applied,
        &ineligible,
        &empty,
        &whitespace_only,
        &oversized,
    ];
    let applied_case_count = decisions
        .iter()
        .filter(|decision| decision.mode == LiveBridgeMode::Applied)
        .count();
    let neutral_fallback_case_count = decisions.len() - applied_case_count;

    let applied_decisions = decisions
        .iter()
        .filter(|decision| decision.mode == LiveBridgeMode::Applied)
        .copied()
        .collect::<Vec<_>>();
    let body_preservation_rate = if applied_decisions.is_empty() {
        0.0
    } else {
        applied_decisions
            .iter()
            .filter(|decision| body_preserved(decision))
            .count() as f64
            / applied_decisions.len() as f64
    };

    let ineligible_decisions = [&ineligible];
    let ineligible_passthrough_rate = ineligible_decisions
        .iter()
        .filter(|decision| {
            decision.mode == LiveBridgeMode::NeutralFallback
                && decision.rendered_text.as_bytes() == decision.neutral_text.as_bytes()
                && decision.fallback_reason == Some(FallbackReason::IneligibleOpener)
        })
        .count() as f64
        / ineligible_decisions.len() as f64;

    let exact_replay = applied == replay;
    let empty_body_passthrough = empty.mode == LiveBridgeMode::NeutralFallback
        && empty.rendered_text.as_bytes() == ELIGIBLE_OPENER.as_bytes()
        && empty.fallback_reason == Some(FallbackReason::EmptyProtectedBody);
    let whitespace_only_passthrough = whitespace_only.mode == LiveBridgeMode::NeutralFallback
        && whitespace_only.rendered_text.as_bytes() == whitespace_only_text.as_bytes()
        && whitespace_only.fallback_reason == Some(FallbackReason::EmptyProtectedBody);
    let oversized_body_passthrough = oversized.mode == LiveBridgeMode::NeutralFallback
        && oversized.rendered_text.as_bytes() == oversized_text.as_bytes()
        && oversized.fallback_reason == Some(FallbackReason::ProtectedBodyTooLarge);
    let separator_only_table = REPLACEMENT_OPENERS
        .iter()
        .all(|opener| separator_only(opener));
    let replacement_table_max_growth_bytes = REPLACEMENT_OPENERS
        .iter()
        .map(|opener| opener.len().saturating_sub(ELIGIBLE_OPENER.len()))
        .max()
        .unwrap_or(usize::MAX);

    let boundary = authority_boundary();
    let no_runtime_influence = !boundary.api_chat_wiring
        && !boundary.live_generated_text_influence
        && !boundary.raw_conversation_access
        && !boundary.unrestricted_memory_access
        && !boundary.voice_state_mutation
        && !boundary.companion_state_mutation
        && !boundary.persistence_authority
        && !boundary.belief_promotion_authority
        && !boundary.ontology_promotion_authority
        && !boundary.routing_authority
        && !boundary.tool_selection_authority
        && !boundary.charge_discharge_authority
        && !boundary.autonomous_action_authority;

    let gate_passed = decisions.len() == 5
        && applied_case_count == 1
        && neutral_fallback_case_count == 4
        && exact_replay
        && body_preservation_rate == 1.0
        && ineligible_passthrough_rate == 1.0
        && empty_body_passthrough
        && whitespace_only_passthrough
        && oversized_body_passthrough
        && separator_only_table
        && replacement_table_max_growth_bytes == MAX_OUTPUT_GROWTH_BYTES
        && boundary.bounded_transform_available
        && no_runtime_influence;

    let report = OmegaV1dKernelReport {
        experiment: "OMEGAV1D0_BOUNDED_LIVE_BRIDGE_KERNEL",
        case_count: decisions.len(),
        applied_case_count,
        neutral_fallback_case_count,
        exact_replay,
        body_preservation_rate,
        ineligible_passthrough_rate,
        empty_body_passthrough,
        whitespace_only_passthrough,
        oversized_body_passthrough,
        separator_only_table,
        replacement_table_max_growth_bytes,
        authority_boundary: boundary,
        no_runtime_influence,
        gate_passed,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    if !gate_passed {
        bail!("ΩV1-D0 bounded live bridge kernel gate failed");
    }
    Ok(())
}
