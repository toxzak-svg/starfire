use star::voice_state::{
    BasisPoints, BoundedVoiceDelta, VoiceDimension, VoiceEvidenceKind, VoiceEvidenceRef,
    VoiceRevisionEvent, VoiceRevisionReason, VoiceRevisionTarget, VoiceState,
};

fn main() -> anyhow::Result<()> {
    let event = VoiceRevisionEvent {
        prior_version: 0,
        resulting_version: 1,
        target: VoiceRevisionTarget::Session {
            session_id: "omega-v1b-shadow-probe".to_owned(),
        },
        evidence: vec![VoiceEvidenceRef {
            kind: VoiceEvidenceKind::ReviewedConfiguration,
            reference: "omega-v1b-preregistered-debug-projection".to_owned(),
        }],
        changed_dimensions: vec![BoundedVoiceDelta::new(
            VoiceDimension::SessionIntensity,
            2_400,
        )?],
        reason: VoiceRevisionReason::SessionConfiguration,
        confidence: BasisPoints::new(10_000)?,
        reversible: true,
    };

    let mut state = VoiceState::default();
    state.apply_revision(0, event.clone())?;
    let replayed = VoiceState::replay(&[event])?;

    let state_json = state.to_canonical_json()?;
    let replay_json = replayed.to_canonical_json()?;
    let state_digest = state.digest()?;
    let replay_digest = replayed.digest()?;
    let projection = state.debug_projection()?;

    let exact_state_match = state == replayed;
    let exact_json_match = state_json == replay_json;
    let exact_digest_match = state_digest == replay_digest;
    let no_runtime_influence = true;
    let gate_passed = exact_state_match
        && exact_json_match
        && exact_digest_match
        && projection.version == 1
        && projection.directness == 0.72
        && projection.warmth == 0.38
        && projection.compression == 0.81
        && projection.initiative == 0.66
        && projection.uncertainty_style == "explicit"
        && projection.session_intensity == 0.24
        && no_runtime_influence;

    let report = serde_json::json!({
        "experiment": "omega_v1b_voice_state_shadow",
        "gate_passed": gate_passed,
        "exact_state_match": exact_state_match,
        "exact_json_match": exact_json_match,
        "exact_digest_match": exact_digest_match,
        "version": projection.version,
        "directness": projection.directness,
        "warmth": projection.warmth,
        "compression": projection.compression,
        "initiative": projection.initiative,
        "uncertainty_style": projection.uncertainty_style,
        "session_intensity": projection.session_intensity,
        "digest": projection.digest,
        "no_runtime_influence": no_runtime_influence,
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    anyhow::ensure!(gate_passed, "ΩV1-B replay gate failed");
    Ok(())
}
