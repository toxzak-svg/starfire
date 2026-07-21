use anyhow::{bail, Context, Result};
use serde_json::json;
use star::omega_v1f1r1_claim_first::CLAIM_FIRST_GRAMMAR_VERSION;
use star::omega_v1f2_shadow::{
    ShadowLedgerRecord, F2_AUTHORITY_MATRIX_VERSION, F2_IMPLEMENTATION_VERSION,
    SHADOW_P95_TARGET_MS, SHADOW_TIMEOUT_MS,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn main() -> Result<()> {
    let path = std::env::var("OMEGA_V1F2_LEDGER_PATH")
        .map(PathBuf::from)
        .context("OMEGA_V1F2_LEDGER_PATH must identify the shadow JSONL ledger")?;
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read ΩV1-F2 ledger {}", path.display()))?;
    let mut records = Vec::new();
    for (index, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        records.push(
            serde_json::from_str::<ShadowLedgerRecord>(line)
                .with_context(|| format!("parse ledger line {}", index + 1))?,
        );
    }
    if records.is_empty() {
        bail!("ΩV1-F2 ledger is empty");
    }

    let mut event_ids = BTreeSet::new();
    let mut duplicate_event_count = 0usize;
    let mut distinct_days = BTreeSet::new();
    let mut authority_digests = BTreeSet::new();
    let mut model_digests = BTreeSet::new();
    let mut lattice_digests = BTreeSet::new();
    let mut intent_counts = BTreeMap::<String, usize>::new();
    let mut family_counts = BTreeMap::<String, usize>::new();
    let mut disposition_counts = BTreeMap::<String, usize>::new();
    let mut ineligibility_counts = BTreeMap::<String, usize>::new();
    let mut phase_counts = BTreeMap::<u16, usize>::new();
    let mut candidate_counts = BTreeMap::<u16, usize>::new();
    let mut elapsed = Vec::new();

    let mut eligible_attempts = 0usize;
    let mut eligible_successes = 0usize;
    let mut ineligible_events = 0usize;
    let mut verifier_accepts = 0usize;
    let mut response_isolation_passes = 0usize;
    let mut typed_failure_reasons = 0usize;
    let mut failure_count = 0usize;
    let mut timeout_count = 0usize;
    let mut panic_count = 0usize;
    let mut unexplained_failure_count = 0usize;
    let mut grammar_mismatch_count = 0usize;
    let mut candidate_bound_violation_count = 0usize;
    let mut timeout_bound_violation_count = 0usize;
    let mut schema_or_version_mismatch_count = 0usize;

    for record in &records {
        if !event_ids.insert(record.event_id.clone()) {
            duplicate_event_count += 1;
        }
        distinct_days.insert(record.utc_day_bucket);
        authority_digests.insert(record.authority_matrix_digest.clone());
        if let Some(digest) = record.model_digest {
            model_digests.insert(digest);
        }
        if let Some(digest) = record.lattice_digest {
            lattice_digests.insert(digest);
        }
        if record.schema_version != 1 || record.implementation_version != F2_IMPLEMENTATION_VERSION {
            schema_or_version_mismatch_count += 1;
        }
        let response_isolated = record.response_before_digest == record.response_after_digest
            && record.response_before_len == record.response_after_len;
        response_isolation_passes += usize::from(response_isolated);
        timeout_count += usize::from(record.timed_out);
        panic_count += usize::from(record.panicked);

        if record.eligibility_code.starts_with("ineligible_") {
            ineligible_events += 1;
            *ineligibility_counts
                .entry(record.eligibility_code.clone())
                .or_default() += 1;
            continue;
        }

        eligible_attempts += 1;
        if let Some(intent) = &record.intent {
            *intent_counts.entry(intent.clone()).or_default() += 1;
        }
        if let Some(family) = &record.selected_family {
            *family_counts.entry(family.clone()).or_default() += 1;
        }
        if let Some(disposition) = &record.selection_disposition {
            *disposition_counts.entry(disposition.clone()).or_default() += 1;
        }
        for variant in &record.variant_ids {
            *phase_counts.entry(variant % 3).or_default() += 1;
        }
        if let Some(count) = record.complete_candidates_scored {
            *candidate_counts.entry(count).or_default() += 1;
            if count > 64 {
                candidate_bound_violation_count += 1;
            }
        }
        if record.elapsed_micros > SHADOW_TIMEOUT_MS * 1_000 {
            timeout_bound_violation_count += 1;
        }
        elapsed.push(record.elapsed_micros);

        if record.eligibility_code == "eligible" {
            eligible_successes += 1;
            verifier_accepts += usize::from(record.verifier_accepted);
            if record.grammar_version != Some(CLAIM_FIRST_GRAMMAR_VERSION) {
                grammar_mismatch_count += 1;
            }
        } else {
            failure_count += 1;
            if record.fallback_reason.is_some() {
                typed_failure_reasons += 1;
            } else {
                unexplained_failure_count += 1;
            }
        }
    }

    elapsed.sort_unstable();
    let p95_micros = percentile(&elapsed, 95);
    let maximum_micros = elapsed.last().copied().unwrap_or_default();
    let sample_complete = eligible_successes >= 200
        && distinct_days.len() >= 7
        && ineligible_events >= 50;
    let response_isolation_rate = ratio(response_isolation_passes, records.len());
    let verifier_acceptance_rate = ratio(verifier_accepts, eligible_successes);
    let explained_failure_rate = ratio(typed_failure_reasons, failure_count);
    let unexplained_fallback_rate = ratio(unexplained_failure_count, eligible_attempts);
    let frozen_identities_stable = authority_digests.len() == 1 && model_digests.len() <= 1;
    let hard_gates_passed = response_isolation_rate == 1.0
        && duplicate_event_count == 0
        && schema_or_version_mismatch_count == 0
        && verifier_acceptance_rate == 1.0
        && grammar_mismatch_count == 0
        && candidate_bound_violation_count == 0
        && timeout_bound_violation_count == 0
        && maximum_micros <= SHADOW_TIMEOUT_MS * 1_000
        && p95_micros <= SHADOW_P95_TARGET_MS * 1_000
        && unexplained_fallback_rate <= 0.01
        && (failure_count == 0 || explained_failure_rate == 1.0)
        && frozen_identities_stable;
    let gate_passed = sample_complete && hard_gates_passed;
    let terminal_classification = if gate_passed {
        "PASS"
    } else if hard_gates_passed {
        "COLLECTING"
    } else {
        "FAIL"
    };

    let report = json!({
        "experiment":"OMEGAV1F2_LIVE_SHADOW_EVALUATION",
        "implementation_version":F2_IMPLEMENTATION_VERSION,
        "authority_matrix_version":F2_AUTHORITY_MATRIX_VERSION,
        "terminal_classification":terminal_classification,
        "gate_passed":gate_passed,
        "hard_gates_passed":hard_gates_passed,
        "sample_complete":sample_complete,
        "record_count":records.len(),
        "eligible_attempt_count":eligible_attempts,
        "eligible_completed_count":eligible_successes,
        "ineligible_event_count":ineligible_events,
        "distinct_utc_days":distinct_days.len(),
        "duplicate_event_count":duplicate_event_count,
        "response_isolation_rate":response_isolation_rate,
        "verifier_acceptance_rate":verifier_acceptance_rate,
        "explained_failure_rate":explained_failure_rate,
        "unexplained_fallback_rate":unexplained_fallback_rate,
        "timeout_count":timeout_count,
        "panic_count":panic_count,
        "p95_selector_and_verifier_micros":p95_micros,
        "maximum_selector_and_verifier_micros":maximum_micros,
        "grammar_mismatch_count":grammar_mismatch_count,
        "candidate_bound_violation_count":candidate_bound_violation_count,
        "timeout_bound_violation_count":timeout_bound_violation_count,
        "schema_or_version_mismatch_count":schema_or_version_mismatch_count,
        "authority_matrix_digest_count":authority_digests.len(),
        "model_digest_count":model_digests.len(),
        "per_program_lattice_digest_count":lattice_digests.len(),
        "intent_distribution":intent_counts,
        "family_distribution":family_counts,
        "phase_distribution":phase_counts,
        "selection_disposition_distribution":disposition_counts,
        "ineligibility_distribution":ineligibility_counts,
        "candidate_count_distribution":candidate_counts,
        "no_prompt_or_response_text_schema":true,
        "no_live_learned_text_return":true
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    if terminal_classification == "FAIL" {
        bail!("ΩV1-F2 hard gate failure");
    }
    Ok(())
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let index = (values.len() * percentile).div_ceil(100).saturating_sub(1);
    values[index.min(values.len() - 1)]
}
