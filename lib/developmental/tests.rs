use std::collections::BTreeMap;

use super::*;

fn sample_evidence(observation_id: &str) -> LearnedEvidence {
    LearnedEvidence {
        schema_version: DEVELOPMENTAL_EVIDENCE_SCHEMA_VERSION,
        source_model: "infant".to_string(),
        source_version: "fixture-v1".to_string(),
        observation_id: observation_id.to_string(),
        modality: LearnedModality::Synthetic,
        payload: LearnedPayload::AnomalyScore(0.25),
        confidence: 0.8,
        uncertainty: 0.2,
        provenance: Provenance {
            producer: "fixture-adapter".to_string(),
            model_id: "infant".to_string(),
            model_version: "fixture-v1".to_string(),
            checkpoint_digest: "sha256:fixture".to_string(),
            source_episode_id: "episode-1".to_string(),
            transformation_trace: vec!["identity".to_string()],
        },
        timestamp: 1_000,
    }
}

#[test]
fn replay_round_trip_is_deterministic() {
    let log = EvidenceReplayLog::new(vec![sample_evidence("obs-1"), sample_evidence("obs-2")]);
    let json_a = log.to_json().expect("serialize replay");
    let decoded = EvidenceReplayLog::from_json(&json_a).expect("deserialize replay");
    let json_b = decoded.to_json().expect("serialize replay again");
    assert_eq!(log, decoded);
    assert_eq!(json_a, json_b);
}

#[test]
fn offline_adapter_preserves_replay_order() {
    let log = EvidenceReplayLog::new(vec![sample_evidence("obs-1"), sample_evidence("obs-2")]);
    let policy = EvidenceValidationPolicy::replay(2_000);
    let mut source = OfflineReplaySource::from_log("fixture", log, &policy).expect("valid replay source");

    assert_eq!(source.remaining(), 2);
    assert_eq!(source.next_evidence().unwrap().unwrap().observation_id, "obs-1");
    assert_eq!(source.next_evidence().unwrap().unwrap().observation_id, "obs-2");
    assert!(source.next_evidence().unwrap().is_none());
}

#[test]
fn noop_adapter_emits_no_evidence() {
    let mut source = NoopDevelopmentalSource;
    assert_eq!(source.source_name(), "noop");
    assert!(source.next_evidence().unwrap().is_none());
}

#[test]
fn baseline_manifest_rejects_non_finite_metrics() {
    let mut repositories = BTreeMap::new();
    repositories.insert(
        "starfire".to_string(),
        RepositoryRevision {
            repository: "toxzak-svg/starfire".to_string(),
            commit_sha: "abc123".to_string(),
            dirty: false,
        },
    );

    let mut metrics = BTreeMap::new();
    metrics.insert("bad_metric".to_string(), f64::NAN);

    let manifest = BaselineManifest {
        schema_version: BASELINE_MANIFEST_SCHEMA_VERSION,
        experiment_id: "h-infant-0-test".to_string(),
        created_at: 1_000,
        repositories,
        checkpoints: vec![],
        task_versions: BTreeMap::new(),
        random_seeds: vec![1, 2, 3],
        runtime: RuntimeRecord {
            operating_system: "test".to_string(),
            architecture: "test".to_string(),
            hardware_summary: "fixture".to_string(),
            runtime_versions: BTreeMap::new(),
        },
        metrics,
        known_failures: vec![],
    };

    assert!(matches!(manifest.validate(), Err(ManifestValidationError::InvalidMetric(_))));
}
