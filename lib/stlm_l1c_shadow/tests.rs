#[cfg(test)]
mod tests {
    use super::*;
    use crate::omega_v1f2_shadow::event_from_intent;
    use crate::runtime::response_intent::ResponseIntent;

    fn eligible_event() -> PendingShadowEvent {
        event_from_intent(&ResponseIntent::ResearchStatus)
    }

    fn response() -> (String, ResponseFingerprint) {
        let body = serde_json::json!({
            "response": "The production response remains the only returned response."
        })
        .to_string();
        let fingerprint = ResponseFingerprint::frozen(body.as_bytes());
        (body, fingerprint)
    }

    #[test]
    fn authority_boundary_remains_closed() {
        let boundary = authority_boundary();
        assert!(boundary.typed_shadow_bundle_access);
        assert!(boundary.response_fingerprint_only);
        assert!(boundary.verified_improvisation_execution);
        assert!(boundary.neutral_control_comparison);
        assert!(boundary.independent_candidate_verification);
        assert!(boundary.ephemeral_fingerprint_trace);
        assert!(boundary.bounded_metadata_recording);
        assert!(!boundary.candidate_text_return);
        assert!(!boundary.candidate_text_persistence);
        assert!(!boundary.raw_prompt_access);
        assert!(!boundary.raw_live_response_access);
        assert!(!boundary.runtime_chat_response_influence);
        assert!(!boundary.http_response_influence);
        assert!(!boundary.general_persistence_authority);
        assert!(!boundary.tool_selection_authority);
        assert!(!boundary.autonomous_action_authority);
    }

    #[test]
    fn observation_replays_and_preserves_response_bytes() {
        let event = eligible_event();
        let (_, fingerprint) = response();
        let first = observe_for_probe(event.clone(), fingerprint, RecentLanguageTrace::default())
            .expect("first observation must complete");
        let second = observe_for_probe(event, fingerprint, RecentLanguageTrace::default())
            .expect("second observation must complete");
        let first_comparison = first.comparison.as_ref().expect("eligible comparison");
        let second_comparison = second.comparison.as_ref().expect("eligible comparison");
        assert_eq!(first.comparison_digest, second.comparison_digest);
        assert_eq!(first_comparison, second_comparison);
        assert!(first_comparison.response_bytes_preserved);
        assert!(first_comparison.independent_verifier_accepted);
        assert!(first_comparison.selection_verification_matches);
        assert!(first_comparison.neutral_control_diverged);
    }

    #[test]
    fn ledger_contains_metadata_not_candidate_or_lexical_text() {
        let event = eligible_event();
        let lexical_fragments = match &event {
            PendingShadowEvent::Eligible(bundle) => bundle
                .lexical_table
                .payload
                .claims
                .iter()
                .map(|claim| claim.positive_clause.clone())
                .collect::<Vec<_>>(),
            PendingShadowEvent::Ineligible(_) => panic!("fixture must be eligible"),
        };
        let (_, fingerprint) = response();
        let record = observe_for_probe(event, fingerprint, RecentLanguageTrace::default())
            .expect("observation must complete");
        let serialized = serde_json::to_string(&record).expect("record must serialize");
        assert!(lexical_fragments
            .iter()
            .all(|fragment| !serialized.contains(fragment)));
        assert!(!serialized.contains("production response remains"));
    }

    #[test]
    fn recent_language_trace_changes_selection_without_live_influence() {
        let event = eligible_event();
        let (_, fingerprint) = response();
        let first = observe_for_probe(event.clone(), fingerprint, RecentLanguageTrace::default())
            .expect("first observation must complete");
        let first_comparison = first.comparison.expect("eligible comparison");
        let trace = RecentLanguageTrace::new(
            vec![first_comparison.candidate_opening_fingerprint],
            vec![first_comparison.candidate_surface_fingerprint],
        )
        .expect("trace must validate");
        let second =
            observe_for_probe(event, fingerprint, trace).expect("second observation must complete");
        let second_comparison = second.comparison.expect("eligible comparison");
        assert!(
            first_comparison.candidate_opening_fingerprint
                != second_comparison.candidate_opening_fingerprint
                || first_comparison.candidate_surface_fingerprint
                    != second_comparison.candidate_surface_fingerprint
        );
        assert!(second_comparison.response_bytes_preserved);
    }
}
