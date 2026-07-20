use anyhow::{bail, Result};
use serde::Serialize;
use star::omega_v1_live_bridge::{
    authority_boundary, render_live_response, FallbackReason, LiveBridgeAuthorityBoundary,
    LiveBridgeDecision, LiveBridgeMode, ELIGIBLE_OPENER, MAX_OUTPUT_GROWTH_BYTES,
    MAX_PROTECTED_BODY_BYTES, REPLACEMENT_OPENERS,
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
    oversized_body_passthrough: bool,
    frozen_opener_table_only: bool,
    maximum_output_growth_bytes: usize,
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

fn main() -> Result<()> {
    let eligible = "Here for it. The protected semantic body stays exactly the same.";
    let applied = render_live_response(eligible);
    let replay = render_live_response(eligible);

    let ineligible_text = "The protected semantic body stays exactly the same.";
    let ineligible = render_live_response(ineligible_text);

    let empty = render_live_response(ELIGIBLE_OPENER);

    let oversized_body = "x".repeat(MAX_PROTECTED_BODY_BYTES + 1);
    let oversized_text = format!("{ELIGIBLE_OPENER}{oversized_body}");
    let oversized = render_live_response(&oversized_text);

    let decisions = [&applied, &ineligible, &empty, &oversized];
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

    let fallback_decisions = [&ineligible];
    let ineligible_passthrough_rate = fallback_decisions
        .iter()
        .filter(|decision| {
            decision.mode == LiveBridgeMode::NeutralFallback
                && decision.rendered_text.as_bytes() == decision.neutral_text.as_bytes()
                && decision.fallback_reason == Some(FallbackReason::IneligibleOpener)
        })
        .count() as f64
        / fallback_decisions.len() as f64;

    let exact_replay = applied == replay;
    let empty_body_passthrough = empty.mode == LiveBridgeMode::NeutralFallback
        && empty.rendered_text.as_bytes() == ELIGIBLE_OPENER.as_bytes()
        && empty.fallback_reason == Some(FallbackReason::EmptyProtectedBody);
    let oversized_body_passthrough = oversized.mode == LiveBridgeMode::NeutralFallback
        && oversized.rendered_text.as_bytes() == oversized_text.as_bytes()
        && oversized.fallback_reason == Some(FallbackReason::ProtectedBodyTooLarge);
    let frozen_opener_table_only = applied
        .selected_opener
        .as_deref()
        .is_some_and(|selected| REPLACEMENT_OPENERS.contains(&selected));
    let maximum_output_growth_bytes = applied
        .rendered_text
        .len()
        .saturating_sub(applied.neutral_text.len());

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

    let gate_passed = decisions.len() == 4
        && applied_case_count == 1
        && neutral_fallback_case_count == 3
        && exact_replay
        && body_preservation_rate == 1.0
        && ineligible_passthrough_rate == 1.0
        && empty_body_passthrough
        && oversized_body_passthrough
        && frozen_opener_table_only
        && maximum_output_growth_bytes <= MAX_OUTPUT_GROWTH_BYTES
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
        oversized_body_passthrough,
        frozen_opener_table_only,
        maximum_output_growth_bytes,
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
