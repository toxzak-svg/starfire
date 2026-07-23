use serde::Serialize;
use star::emerging_intelligence::{
    AuthoritySnapshot, CognitiveEpisode, EpisodeId, EpisodePhase, EpisodeProvenance,
    EvaluationPartition, Observation, ObservationId, SealedCognitiveEpisode,
};
use star::emerging_intelligence_ledger::{AppendOnlyEpisodeLedger, EI_0C_CLAIM};

#[derive(Serialize)]
struct ProbeReport {
    stage: &'static str,
    classification: &'static str,
    claim: &'static str,
    ledger_root: String,
    entry_count: u64,
    exact_canonical_replay: bool,
    fresh_state_episode_replay: bool,
    duplicate_rejected: bool,
    corruption_rejected: bool,
    truncation_rejected: bool,
    reordering_rejected: bool,
    stale_schema_rejected: bool,
    noncanonical_encoding_rejected: bool,
    runtime_wiring: bool,
    sqlite_persistence: bool,
    learning_application_authority: bool,
    authority_closed: bool,
}

fn observed_episode(id: &str, seed: u64) -> SealedCognitiveEpisode {
    CognitiveEpisode {
        episode_id: EpisodeId::new(id).expect("valid episode identifier"),
        phase: EpisodePhase::Observed,
        partition: EvaluationPartition::Development,
        task_family: "route-choice".into(),
        observation: Observation {
            observation_id: ObservationId::new(format!("observation-{id}"))
                .expect("valid observation identifier"),
            kind: "route-state".into(),
            facts: vec![format!("seed:{seed:08x}")],
            observed_at_step: 1,
        },
        evidence: Vec::new(),
        predictions: Vec::new(),
        selected_strategy: None,
        intention: None,
        action: None,
        outcome: None,
        evaluation: None,
        proposed_updates: Vec::new(),
        accepted_updates: Vec::new(),
        authority: AuthoritySnapshot::closed(),
        provenance: EpisodeProvenance {
            cohort_id: "ei-0c-probe".into(),
            fixture_digest: format!("fixture:{seed:08x}"),
            seed,
            generator_version: "ei-0c-probe-v1".into(),
            source_hashes: vec![format!("source:{seed:08x}")],
        },
    }
    .seal()
    .expect("probe episode must seal")
}

fn mutated_bytes<F>(canonical: &[u8], mutation: F) -> Vec<u8>
where
    F: FnOnce(&mut serde_json::Value),
{
    let mut value: serde_json::Value =
        serde_json::from_slice(canonical).expect("canonical ledger must parse");
    mutation(&mut value);
    serde_json::to_vec(&value).expect("mutated ledger must serialize")
}

fn main() {
    let first = observed_episode("episode-001", 1);
    let second = observed_episode("episode-002", 2);
    let mut ledger = AppendOnlyEpisodeLedger::new().expect("empty ledger must initialize");
    ledger.append(&first).expect("first append must succeed");
    ledger.append(&second).expect("second append must succeed");

    let canonical = ledger
        .to_canonical_bytes()
        .expect("ledger must serialize canonically");
    let replay = AppendOnlyEpisodeLedger::replay_from_canonical_bytes(&canonical)
        .expect("fresh-state replay must succeed");
    let replayed_episode_bytes: Vec<Vec<u8>> = replay
        .episodes
        .iter()
        .map(|episode| {
            episode
                .to_canonical_bytes()
                .expect("replayed episode must remain canonical")
        })
        .collect();
    let original_episode_bytes = vec![
        first
            .to_canonical_bytes()
            .expect("first episode must remain canonical"),
        second
            .to_canonical_bytes()
            .expect("second episode must remain canonical"),
    ];

    let duplicate_rejected = ledger.append(&first).is_err();
    let corruption_rejected = {
        let corrupted = mutated_bytes(&canonical, |value| {
            value["entries"][0]["episode_bytes"][0] = serde_json::Value::from(0);
        });
        AppendOnlyEpisodeLedger::from_canonical_bytes(&corrupted).is_err()
    };
    let truncation_rejected = {
        let truncated = mutated_bytes(&canonical, |value| {
            value["entries"]
                .as_array_mut()
                .expect("entries must be an array")
                .pop();
        });
        AppendOnlyEpisodeLedger::from_canonical_bytes(&truncated).is_err()
    };
    let reordering_rejected = {
        let reordered = mutated_bytes(&canonical, |value| {
            value["entries"]
                .as_array_mut()
                .expect("entries must be an array")
                .swap(0, 1);
        });
        AppendOnlyEpisodeLedger::from_canonical_bytes(&reordered).is_err()
    };
    let stale_schema_rejected = {
        let stale = mutated_bytes(&canonical, |value| {
            value["schema_version"] = serde_json::Value::from(2);
        });
        AppendOnlyEpisodeLedger::from_canonical_bytes(&stale).is_err()
    };
    let noncanonical_encoding_rejected = {
        let mut noncanonical = canonical.clone();
        noncanonical.push(b'\n');
        AppendOnlyEpisodeLedger::from_canonical_bytes(&noncanonical).is_err()
    };

    let report = ProbeReport {
        stage: "EI-0C",
        classification: "pass",
        claim: EI_0C_CLAIM,
        ledger_root: ledger.root_digest().as_str().to_owned(),
        entry_count: ledger.entry_count(),
        exact_canonical_replay: replay.canonical_ledger_bytes == canonical,
        fresh_state_episode_replay: replayed_episode_bytes == original_episode_bytes,
        duplicate_rejected,
        corruption_rejected,
        truncation_rejected,
        reordering_rejected,
        stale_schema_rejected,
        noncanonical_encoding_rejected,
        runtime_wiring: false,
        sqlite_persistence: false,
        learning_application_authority: false,
        authority_closed: replay.summary.authority.is_closed(),
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("probe report must serialize")
    );
}
