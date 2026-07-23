use star::omega_v1f2_shadow::{event_from_intent, PendingShadowEvent, ResponseFingerprint};
use star::runtime::response_intent::ResponseIntent;
use star::stlm_l1c_shadow::{
    authority_boundary, observe_for_probe, L1CShadowProbeReport, L1C_IMPLEMENTATION_VERSION,
};
use star::verified_improvisation::RecentLanguageTrace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event = event_from_intent(&ResponseIntent::ResearchStatus);
    let lexical_fragments = match &event {
        PendingShadowEvent::Eligible(bundle) => bundle
            .lexical_table
            .payload
            .claims
            .iter()
            .map(|claim| claim.positive_clause.clone())
            .collect::<Vec<_>>(),
        PendingShadowEvent::Ineligible(_) => {
            return Err("research-status fixture unexpectedly ineligible".into())
        }
    };
    let response_json = serde_json::json!({
        "response": "The production response remains byte-identical and authoritative."
    })
    .to_string();
    let fingerprint = ResponseFingerprint::frozen(response_json.as_bytes());

    let first = observe_for_probe(event.clone(), fingerprint, RecentLanguageTrace::default())?;
    let replay = observe_for_probe(event.clone(), fingerprint, RecentLanguageTrace::default())?;
    let first_comparison = first
        .comparison
        .as_ref()
        .ok_or("eligible observation lacked comparison metadata")?;
    let replay_comparison = replay
        .comparison
        .as_ref()
        .ok_or("replay observation lacked comparison metadata")?;

    let repeated_trace = RecentLanguageTrace::new(
        vec![first_comparison.candidate_opening_fingerprint],
        vec![first_comparison.candidate_surface_fingerprint],
    )?;
    let trace_treatment = observe_for_probe(event, fingerprint, repeated_trace)?;
    let trace_comparison = trace_treatment
        .comparison
        .as_ref()
        .ok_or("trace treatment lacked comparison metadata")?;

    let serialized = serde_json::to_string(&first)?;
    let candidate_text_absent_from_ledger = lexical_fragments
        .iter()
        .all(|fragment| !serialized.contains(fragment))
        && !serialized.contains("production response remains")
        && !serialized.contains("production response remains byte-identical");

    let ineligible = observe_for_probe(
        PendingShadowEvent::Ineligible(star::omega_v1f2_shadow::ShadowIneligibility::UnknownIntent),
        fingerprint,
        RecentLanguageTrace::default(),
    )?;
    let ineligible_event_isolated = ineligible.comparison.is_none()
        && ineligible.comparison_digest.is_none()
        && ineligible.failure_reason.is_none()
        && ineligible.eligibility_code == "ineligible_unknown_intent";

    let boundary = authority_boundary();
    let authority_boundary_closed = boundary.typed_shadow_bundle_access
        && boundary.response_fingerprint_only
        && boundary.verified_improvisation_execution
        && boundary.neutral_control_comparison
        && boundary.independent_candidate_verification
        && boundary.ephemeral_fingerprint_trace
        && boundary.bounded_metadata_recording
        && !boundary.candidate_text_return
        && !boundary.candidate_text_persistence
        && !boundary.raw_prompt_access
        && !boundary.raw_live_response_access
        && !boundary.unrestricted_conversation_access
        && !boundary.unrestricted_memory_access
        && !boundary.runtime_chat_response_influence
        && !boundary.http_response_influence
        && !boundary.voice_state_mutation
        && !boundary.companion_state_mutation
        && !boundary.general_persistence_authority
        && !boundary.belief_promotion_authority
        && !boundary.ontology_promotion_authority
        && !boundary.routing_authority
        && !boundary.tool_selection_authority
        && !boundary.charge_discharge_authority
        && !boundary.autonomous_action_authority;
    let exact_replay = first.comparison_digest == replay.comparison_digest
        && first_comparison == replay_comparison;
    let recent_language_changed_selection = first_comparison.candidate_opening_fingerprint
        != trace_comparison.candidate_opening_fingerprint
        || first_comparison.candidate_surface_fingerprint
            != trace_comparison.candidate_surface_fingerprint;
    let independent_candidate_verified = first_comparison.independent_verifier_accepted
        && first_comparison.selection_verification_matches;
    let response_bytes_preserved = first_comparison.response_bytes_preserved
        && trace_comparison.response_bytes_preserved
        && fingerprint.byte_identical();
    let neutral_control_diverged = first_comparison.neutral_control_diverged;
    let no_runtime_response_influence = authority_boundary_closed && response_bytes_preserved;
    let eligible_observation_created = first.eligibility_code == "eligible"
        && first.comparison.is_some()
        && first.comparison_digest.is_some();
    let gate_passed = eligible_observation_created
        && independent_candidate_verified
        && exact_replay
        && neutral_control_diverged
        && recent_language_changed_selection
        && response_bytes_preserved
        && candidate_text_absent_from_ledger
        && ineligible_event_isolated
        && authority_boundary_closed
        && no_runtime_response_influence;

    let report = L1CShadowProbeReport {
        experiment: "STLM_L1C_VERIFIED_IMPROVISATION_SHADOW".to_owned(),
        implementation_version: L1C_IMPLEMENTATION_VERSION.to_owned(),
        eligible_observation_created,
        independent_candidate_verified,
        exact_replay,
        neutral_control_diverged,
        recent_language_changed_selection,
        response_bytes_preserved,
        candidate_text_absent_from_ledger,
        ineligible_event_isolated,
        authority_boundary_closed,
        no_runtime_response_influence,
        gate_passed,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !gate_passed {
        return Err("STLM L1-C shadow gate failed".into());
    }
    Ok(())
}
