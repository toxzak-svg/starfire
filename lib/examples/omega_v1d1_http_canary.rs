use anyhow::{bail, Result};
use serde::Serialize;
use star::api::{
    finalize_chat_response, http_canary_authority_boundary, HttpCanaryAuthorityBoundary,
};
use star::omega_v1_live_bridge::{
    authority_boundary as d0_authority_boundary, ELIGIBLE_OPENER, MAX_OUTPUT_GROWTH_BYTES,
    OPENER_STEM, REPLACEMENT_OPENERS,
};

#[derive(Debug, Serialize)]
struct OmegaV1d1HttpCanaryReport {
    experiment: &'static str,
    parent_d0_commit: &'static str,
    case_count: usize,
    exact_replay: bool,
    protected_body_preserved: bool,
    ineligible_passthrough: bool,
    json_shape_preserved: bool,
    replacement_table_confined: bool,
    maximum_output_growth_bytes: usize,
    d0_kernel_authority_still_shadow_only: bool,
    authority_boundary: HttpCanaryAuthorityBoundary,
    gate_passed: bool,
}

fn main() -> Result<()> {
    let eligible = "Here for it. The completed cognition-produced body stays byte-exact.";
    let protected_body = eligible
        .strip_prefix(ELIGIBLE_OPENER)
        .expect("frozen eligible fixture must contain the exact opener");
    let first = finalize_chat_response(eligible.to_string());
    let replay = finalize_chat_response(eligible.to_string());

    let selected = REPLACEMENT_OPENERS
        .iter()
        .find(|candidate| first.starts_with(**candidate))
        .copied();
    let protected_body_preserved = selected
        .and_then(|opener| first.strip_prefix(opener))
        .is_some_and(|body| body.as_bytes() == protected_body.as_bytes());
    let exact_replay = first == replay;

    let ineligible = "The HTTP response has no eligible opener.";
    let ineligible_passthrough =
        finalize_chat_response(ineligible.to_string()).as_bytes() == ineligible.as_bytes();

    let json = serde_json::json!({ "response": first }).to_string();
    let parsed: serde_json::Value = serde_json::from_str(&json)?;
    let json_shape_preserved = parsed.as_object().is_some_and(|object| {
        object.len() == 1
            && object
                .get("response")
                .is_some_and(serde_json::Value::is_string)
    });

    let replacement_table_confined = REPLACEMENT_OPENERS.iter().all(|opener| {
        matches!(opener.strip_prefix(OPENER_STEM), Some("\n") | Some("\n\n"))
    });
    let maximum_output_growth_bytes = REPLACEMENT_OPENERS
        .iter()
        .map(|opener| opener.len().saturating_sub(ELIGIBLE_OPENER.len()))
        .max()
        .unwrap_or(usize::MAX);

    let d0 = d0_authority_boundary();
    let d0_kernel_authority_still_shadow_only = d0.bounded_transform_available
        && !d0.api_chat_wiring
        && !d0.live_generated_text_influence
        && !d0.raw_conversation_access
        && !d0.unrestricted_memory_access
        && !d0.voice_state_mutation
        && !d0.companion_state_mutation
        && !d0.persistence_authority
        && !d0.belief_promotion_authority
        && !d0.ontology_promotion_authority
        && !d0.routing_authority
        && !d0.tool_selection_authority
        && !d0.charge_discharge_authority
        && !d0.autonomous_action_authority;

    let boundary = http_canary_authority_boundary();
    let authority_is_http_only = boundary.api_chat_wiring
        && boundary.live_generated_text_influence
        && !boundary.raw_prompt_access
        && !boundary.unrestricted_conversation_access
        && !boundary.unrestricted_memory_access
        && !boundary.voice_state_mutation
        && !boundary.companion_state_mutation
        && !boundary.persistence_authority
        && !boundary.belief_promotion_authority
        && !boundary.ontology_promotion_authority
        && !boundary.routing_authority
        && !boundary.tool_selection_authority
        && !boundary.charge_discharge_authority
        && !boundary.autonomous_action_authority
        && !boundary.non_chat_http_influence
        && !boundary.cli_influence;

    let gate_passed = exact_replay
        && protected_body_preserved
        && ineligible_passthrough
        && json_shape_preserved
        && replacement_table_confined
        && maximum_output_growth_bytes <= MAX_OUTPUT_GROWTH_BYTES
        && d0_kernel_authority_still_shadow_only
        && authority_is_http_only;

    let report = OmegaV1d1HttpCanaryReport {
        experiment: "OMEGAV1D1_HTTP_CHAT_CANARY",
        parent_d0_commit: "87304d21c19b2c18ecb43e12d0b0a84d01750ba4",
        case_count: 2,
        exact_replay,
        protected_body_preserved,
        ineligible_passthrough,
        json_shape_preserved,
        replacement_table_confined,
        maximum_output_growth_bytes,
        d0_kernel_authority_still_shadow_only,
        authority_boundary: boundary,
        gate_passed,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    if !gate_passed {
        bail!("ΩV1-D1 HTTP canary gate failed");
    }

    Ok(())
}
