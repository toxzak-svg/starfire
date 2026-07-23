pub fn observe_for_probe(
    event: PendingShadowEvent,
    response: ResponseFingerprint,
    trace: RecentLanguageTrace,
) -> Result<L1CShadowLedgerRecord, L1CShadowError> {
    match event {
        PendingShadowEvent::Ineligible(code) => Ok(ineligible_record(code, response)),
        PendingShadowEvent::Eligible(bundle) => {
            Ok(evaluate_bundle(bundle, response, trace)?.record)
        }
    }
}

fn evaluate_bundle(
    bundle: ShadowInputBundle,
    response: ResponseFingerprint,
    trace: RecentLanguageTrace,
) -> Result<EvaluatedObservation, L1CShadowError> {
    let started = Instant::now();
    trace.verify_integrity()?;
    let microstate = microstate_for(bundle.intent, bundle.sensitivity)?;
    let entropy_seed = entropy_seed(&bundle, response);
    let request = ImprovisationRequest::new(entropy_seed, microstate, trace.clone())?;
    let neutral = VerifierReadyRenderer.render(&bundle.program, &bundle.lexical_table)?;
    let lattice = ImprovisationLattice::build(&bundle.program, &bundle.lexical_table)?;
    let selection =
        VerifiedImprovisationSelector.select(&bundle.program, &bundle.lexical_table, &request)?;
    let independent_verification =
        if selection.payload.disposition == ImprovisationDisposition::VerifiedImprovisation {
            ImprovisationalVerifier
                .verify(
                    &bundle.program,
                    &bundle.lexical_table,
                    lattice.digest,
                    &selection.payload.text,
                )
                .ok()
        } else {
            None
        };
    let selection_verification_matches = independent_verification
        .as_ref()
        .is_some_and(|report| selection.payload.verification_digest == Some(report.digest));
    let candidate_json = serde_json::json!({ "response": &selection.payload.text }).to_string();
    let candidate_matches_returned_response =
        response_matches_bytes(response, candidate_json.as_bytes());
    let neutral_control_diverged = selection.payload.text != neutral.payload.text;
    let neutral_control_fingerprint = text_fingerprint(&neutral.payload.text);
    let mut next_trace = trace.clone();
    next_trace.record_text(&selection.payload.text)?;
    let trace_changed = next_trace != trace;
    let elapsed_micros = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);
    let payload = L1CComparisonPayload {
        authority_matrix_digest: authority_matrix_digest(),
        event_id: bundle.event_id.clone(),
        intent: bundle.intent.label().to_owned(),
        sensitivity: format!("{:?}", bundle.sensitivity).to_ascii_lowercase(),
        program_digest: bundle.program.digest.0,
        lexical_table_digest: bundle.lexical_table.digest.0,
        response_before_digest: response.before_digest,
        response_after_digest: response.after_digest,
        response_before_len: response.before_len,
        response_after_len: response.after_len,
        response_bytes_preserved: response.byte_identical(),
        entropy_seed,
        microstate,
        trace_openings_before: bounded_len(trace.opening_fingerprints.len()),
        trace_surfaces_before: bounded_len(trace.surface_fingerprints.len()),
        trace_openings_after: bounded_len(next_trace.opening_fingerprints.len()),
        trace_surfaces_after: bounded_len(next_trace.surface_fingerprints.len()),
        trace_changed,
        neutral_control_fingerprint,
        candidate_opening_fingerprint: selection.payload.opening_fingerprint,
        candidate_surface_fingerprint: selection.payload.surface_fingerprint,
        selection_digest: selection.digest.0,
        lattice_digest: selection.payload.lattice_digest.map(|digest| digest.0),
        verification_digest: independent_verification
            .as_ref()
            .map(|report| report.digest.0),
        selected_grammar_version: selection.payload.selected_grammar_version,
        disposition: format!("{:?}", selection.payload.disposition).to_ascii_lowercase(),
        variant_ids: selection
            .payload
            .variant_ids
            .iter()
            .map(|id| id.0)
            .collect(),
        score: selection.payload.score,
        complete_candidates_scored: selection.payload.complete_candidates_scored,
        fallback_reason: selection
            .payload
            .fallback_reason
            .map(|reason| bounded_reason(&reason)),
        independent_verifier_accepted: independent_verification.is_some(),
        selection_verification_matches,
        neutral_control_diverged,
        candidate_matches_returned_response,
    };
    let comparison_digest = comparison_digest(&payload)?;
    let record = L1CShadowLedgerRecord {
        schema_version: 1,
        implementation_version: L1C_IMPLEMENTATION_VERSION.to_owned(),
        utc_day_bucket: crate::now_timestamp() / 86_400,
        utc_hour_bucket: crate::now_timestamp() / 3_600,
        eligibility_code: "eligible".to_owned(),
        response_before_digest: response.before_digest,
        response_after_digest: response.after_digest,
        response_before_len: response.before_len,
        response_after_len: response.after_len,
        response_bytes_preserved: response.byte_identical(),
        comparison_digest: Some(comparison_digest),
        comparison: Some(payload),
        failure_reason: None,
        trace_update_committed: false,
        elapsed_micros,
        timed_out: false,
        panicked: false,
    };
    Ok(EvaluatedObservation {
        record,
        trace_key: bundle.intent.label().to_owned(),
        next_trace,
    })
}

fn ineligible_record(
    code: ShadowIneligibility,
    response: ResponseFingerprint,
) -> L1CShadowLedgerRecord {
    L1CShadowLedgerRecord {
        schema_version: 1,
        implementation_version: L1C_IMPLEMENTATION_VERSION.to_owned(),
        utc_day_bucket: crate::now_timestamp() / 86_400,
        utc_hour_bucket: crate::now_timestamp() / 3_600,
        eligibility_code: code.label().to_owned(),
        response_before_digest: response.before_digest,
        response_after_digest: response.after_digest,
        response_before_len: response.before_len,
        response_after_len: response.after_len,
        response_bytes_preserved: response.byte_identical(),
        comparison_digest: None,
        comparison: None,
        failure_reason: None,
        trace_update_committed: false,
        elapsed_micros: 0,
        timed_out: false,
        panicked: false,
    }
}

fn failure_record(
    bundle: Option<&ShadowInputBundle>,
    response: ResponseFingerprint,
    reason: &str,
    timed_out: bool,
    panicked: bool,
    elapsed_micros: u64,
) -> L1CShadowLedgerRecord {
    let eligibility_code = bundle
        .map(|bundle| format!("eligible_{}", bundle.intent.label()))
        .unwrap_or_else(|| "unknown".to_owned());
    let response_reason = if response.byte_identical() {
        bounded_reason(reason)
    } else {
        "response_fingerprint_changed".to_owned()
    };
    L1CShadowLedgerRecord {
        schema_version: 1,
        implementation_version: L1C_IMPLEMENTATION_VERSION.to_owned(),
        utc_day_bucket: crate::now_timestamp() / 86_400,
        utc_hour_bucket: crate::now_timestamp() / 3_600,
        eligibility_code,
        response_before_digest: response.before_digest,
        response_after_digest: response.after_digest,
        response_before_len: response.before_len,
        response_after_len: response.after_len,
        response_bytes_preserved: response.byte_identical(),
        comparison_digest: None,
        comparison: None,
        failure_reason: Some(response_reason),
        trace_update_committed: false,
        elapsed_micros,
        timed_out,
        panicked,
    }
}
