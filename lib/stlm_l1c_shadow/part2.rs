#[derive(Debug)]
struct EvaluatedObservation {
    record: L1CShadowLedgerRecord,
    trace_key: String,
    next_trace: RecentLanguageTrace,
}

#[must_use]
pub fn shadow_enabled() -> bool {
    std::env::var("STARFIRE_STLM_L1C_SHADOW")
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "on" | "enabled"
            )
        })
        .unwrap_or(false)
}

pub fn dispatch(event: PendingShadowEvent, response: ResponseFingerprint) {
    if !shadow_enabled() {
        return;
    }

    if let Err(error) = thread::Builder::new()
        .name("stlm-l1c-dispatch".to_owned())
        .spawn(move || dispatch_inner(event, response))
    {
        warn!("STLM L1-C shadow dispatcher unavailable: {error}");
    }
}

fn dispatch_inner(event: PendingShadowEvent, response: ResponseFingerprint) {
    match event {
        PendingShadowEvent::Ineligible(code) => {
            let record = ineligible_record(code, response);
            if let Err(error) = append_record(&record) {
                warn!("STLM L1-C ineligible metadata was not recorded: {error}");
            }
        }
        PendingShadowEvent::Eligible(bundle) => {
            let trace_key = bundle.intent.label().to_owned();
            let trace = match trace_snapshot(&trace_key) {
                Ok(trace) => trace,
                Err(error) => {
                    let record = failure_record(
                        Some(&bundle),
                        response,
                        &bounded_reason(&error.to_string()),
                        false,
                        false,
                        0,
                    );
                    let _ = append_record(&record);
                    return;
                }
            };
            let timeout_record = failure_record(
                Some(&bundle),
                response,
                "shadow_timeout",
                true,
                false,
                L1C_SHADOW_TIMEOUT_MS * 1_000,
            );
            let (sender, receiver) = mpsc::sync_channel(1);
            let worker_bundle = bundle.clone();
            let worker = thread::Builder::new()
                .name("stlm-l1c-selector".to_owned())
                .spawn(move || {
                    let result = std::panic::catch_unwind(|| {
                        evaluate_bundle(worker_bundle, response, trace)
                    });
                    let _ = sender.send(result);
                });

            if worker.is_err() {
                let record = failure_record(
                    Some(&bundle),
                    response,
                    "shadow_worker_unavailable",
                    false,
                    false,
                    0,
                );
                let _ = append_record(&record);
                return;
            }

            let mut record = match receiver
                .recv_timeout(Duration::from_millis(L1C_SHADOW_TIMEOUT_MS))
            {
                Ok(Ok(Ok(observation))) => {
                    let committed = commit_trace(&observation.trace_key, observation.next_trace)
                        .map_err(|error| {
                            warn!("STLM L1-C trace update was isolated: {error}");
                            error
                        })
                        .is_ok();
                    let mut record = observation.record;
                    record.trace_update_committed = committed;
                    record
                }
                Ok(Ok(Err(error))) => failure_record(
                    Some(&bundle),
                    response,
                    &bounded_reason(&error.to_string()),
                    false,
                    false,
                    0,
                ),
                Ok(Err(_)) => failure_record(
                    Some(&bundle),
                    response,
                    "shadow_worker_panic",
                    false,
                    true,
                    0,
                ),
                Err(mpsc::RecvTimeoutError::Timeout) => timeout_record,
                Err(mpsc::RecvTimeoutError::Disconnected) => failure_record(
                    Some(&bundle),
                    response,
                    "shadow_worker_disconnected",
                    false,
                    false,
                    0,
                ),
            };

            if record.comparison.is_some() && !record.trace_update_committed {
                record.failure_reason = Some("ephemeral_trace_update_isolated".to_owned());
            }
            if let Err(error) = append_record(&record) {
                warn!("STLM L1-C eligible metadata was not recorded: {error}");
            } else {
                info!(
                    "STLM L1-C shadow event={} verifier={} diverged={} elapsed_us={}",
                    record
                        .comparison
                        .as_ref()
                        .map(|comparison| comparison.event_id.as_str())
                        .unwrap_or("none"),
                    record
                        .comparison
                        .as_ref()
                        .is_some_and(|comparison| comparison.independent_verifier_accepted),
                    record
                        .comparison
                        .as_ref()
                        .is_some_and(|comparison| comparison.neutral_control_diverged),
                    record.elapsed_micros
                );
            }
        }
    }
}
