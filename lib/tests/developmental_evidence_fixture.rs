#![cfg(feature = "developmental-evidence")]

use star::developmental::{
    EvidenceValidationPolicy, LearnedEvidence, LearnedModality, LearnedPayload,
};

#[test]
fn canonical_infant_fixture_deserializes_and_validates() {
    let raw = include_str!("fixtures/infant_state_embedding_v1.json").trim();
    let evidence: LearnedEvidence =
        serde_json::from_str(raw).expect("Infant fixture must deserialize as LearnedEvidence");

    assert_eq!(evidence.schema_version, 1);
    assert_eq!(evidence.source_model, "infant-gym");
    assert_eq!(evidence.source_version, "f0-fixture-v1");
    assert_eq!(evidence.observation_id, "obs-fixture-0001");
    assert_eq!(evidence.modality, LearnedModality::Synthetic);
    assert_eq!(evidence.provenance.producer, "infant_gym");
    assert_eq!(
        evidence.provenance.checkpoint_digest,
        "sha256:fixture-deadbeef"
    );

    match &evidence.payload {
        LearnedPayload::StateEmbedding(values) => {
            assert_eq!(values, &vec![0.125_f32, -0.5_f32, 1.25_f32]);
        }
        other => panic!("expected state_embedding payload, got {other:?}"),
    }

    evidence
        .validate(&EvidenceValidationPolicy::replay(1_700_000_000))
        .expect("canonical Infant fixture must pass Starfire replay validation");
}
