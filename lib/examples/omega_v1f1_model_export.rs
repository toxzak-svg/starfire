use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::json;
use star::learned_expression::{
    LearnedExpressionModel, LearnedVoiceProjection, PairwisePreference, PreferredSide,
    VariantProfile, MAX_MODEL_BYTES,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

const F1: &str = include_str!("../fixtures/omega_v1f1/manifest.json");
const SHARDS: [&str; 7] = [
    include_str!("../fixtures/omega_v1a/ordinary.json"),
    include_str!("../fixtures/omega_v1a/technical.json"),
    include_str!("../fixtures/omega_v1a/emotional.json"),
    include_str!("../fixtures/omega_v1a/disagreement.json"),
    include_str!("../fixtures/omega_v1a/uncertainty.json"),
    include_str!("../fixtures/omega_v1a/continuity.json"),
    include_str!("../fixtures/omega_v1a/adversarial.json"),
];

#[derive(Deserialize)]
struct Manifest {
    schema_version: u16,
    experiment: String,
    split: SplitPolicy,
    preference_evidence: PreferencePolicy,
    projection_profiles: ProjectionProfiles,
}

#[derive(Deserialize)]
struct SplitPolicy {
    modulus: u16,
    test_remainder: u16,
    validation_remainder: u16,
    expected_totals: BTreeMap<String, usize>,
}

#[derive(Deserialize)]
struct PreferencePolicy {
    schema_version: u16,
    profile_preferences: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct ProjectionProfiles {
    direct: [u16; 7],
    warm: [u16; 7],
    neutral: [u16; 7],
}

#[derive(Deserialize)]
struct Fixture {
    id: String,
    profile: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Preference {
    Direct,
    Warm,
}

fn main() -> Result<()> {
    let output = std::env::var("OMEGA_V1F2_MODEL_OUT")
        .map(PathBuf::from)
        .context("OMEGA_V1F2_MODEL_OUT must identify the artifact output path")?;
    let manifest: Manifest = serde_json::from_str(F1)?;
    validate_manifest(&manifest)?;

    let mut fixtures = Vec::new();
    for shard in SHARDS {
        fixtures.extend(serde_json::from_str::<Vec<Fixture>>(shard)?);
    }
    if fixtures.len() != 122 {
        bail!("frozen ΩV1 corpus drift: expected 122 fixtures");
    }

    let mut split_counts = BTreeMap::<String, usize>::new();
    let mut training = Vec::new();
    for fixture in &fixtures {
        let suffix = suffix(&fixture.id)?;
        let remainder = suffix % manifest.split.modulus;
        let split = if remainder == manifest.split.test_remainder {
            "test"
        } else if remainder == manifest.split.validation_remainder {
            "validation"
        } else {
            "train"
        };
        *split_counts.entry(split.to_owned()).or_default() += 1;
        if split != "train" {
            continue;
        }
        let preference = preference(&manifest, &fixture.profile)?;
        training.push(PairwisePreference {
            projection: projection(
                &manifest.projection_profiles,
                &fixture.id,
                preference,
                suffix,
            )?,
            left: VariantProfile::direct(),
            right: VariantProfile::warm(),
            preferred: match preference {
                Preference::Direct => PreferredSide::Left,
                Preference::Warm => PreferredSide::Right,
            },
        });
    }

    if split_counts != manifest.split.expected_totals || training.len() != 74 {
        bail!("frozen ΩV1-F1R1 split drift");
    }

    let model = LearnedExpressionModel::train(&training, 8, 200)?;
    model.verify_integrity()?;
    let bytes = model.artifact_bytes()?;
    if bytes.is_empty() || bytes.len() > MAX_MODEL_BYTES {
        bail!("exported ΩV1-F1R1 artifact is outside bounds");
    }
    let replayed: LearnedExpressionModel = serde_json::from_slice(&bytes)?;
    replayed.verify_integrity()?;
    if replayed != model {
        bail!("exported ΩV1-F1R1 artifact replay mismatch");
    }
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, &bytes)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "experiment":"OMEGAV1F1R1_MODEL_EXPORT",
            "source_experiment":manifest.experiment,
            "fixture_count":fixtures.len(),
            "training_count":training.len(),
            "split_counts":split_counts,
            "model_digest":model.digest.0,
            "model_parameter_count":model.parameter_count(),
            "model_artifact_bytes":bytes.len(),
            "artifact_replay_exact":true,
            "gate_passed":true
        }))?
    );
    Ok(())
}

fn validate_manifest(manifest: &Manifest) -> Result<()> {
    if manifest.schema_version != 2
        || manifest.experiment != "OMEGAV1F1R1_OFFLINE_LEARNED_SELECTOR"
        || manifest.preference_evidence.schema_version != 2
        || manifest.split.modulus != 5
        || manifest.split.test_remainder != 0
        || manifest.split.validation_remainder != 4
    {
        bail!("frozen ΩV1-F1R1 manifest drift");
    }
    Ok(())
}

fn suffix(id: &str) -> Result<u16> {
    Ok(id.rsplit_once('-').context("bad fixture id")?.1.parse()?)
}

fn preference(manifest: &Manifest, profile: &str) -> Result<Preference> {
    match manifest
        .preference_evidence
        .profile_preferences
        .get(profile)
        .map(String::as_str)
        .context("missing profile preference")?
    {
        "direct" => Ok(Preference::Direct),
        "warm" => Ok(Preference::Warm),
        other => bail!("invalid frozen preference {other}"),
    }
}

fn projection(
    profiles: &ProjectionProfiles,
    id: &str,
    preference: Preference,
    suffix: u16,
) -> Result<LearnedVoiceProjection> {
    let target = match preference {
        Preference::Direct => profiles.direct,
        Preference::Warm => profiles.warm,
    };
    let values = if suffix.is_multiple_of(2) {
        target
    } else {
        mix(profiles.neutral, target, 1, 4)
    };
    LearnedVoiceProjection::new(
        u64::from(suffix),
        values[0],
        values[1],
        values[2],
        values[3],
        values[4],
        values[5],
        values[6],
        format!(
            "omega-v1f1r1-projection-v1:{id}:{}:{:016x}",
            match preference {
                Preference::Direct => "direct",
                Preference::Warm => "warm",
            },
            hash_values(&values)
        ),
    )
    .map_err(Into::into)
}

fn mix(left: [u16; 7], right: [u16; 7], numerator: i32, denominator: i32) -> [u16; 7] {
    let mut values = [0; 7];
    for index in 0..7 {
        let value = i32::from(left[index])
            + (i32::from(right[index]) - i32::from(left[index])) * numerator / denominator;
        values[index] = u16::try_from(value.clamp(0, 10_000)).unwrap_or(0);
    }
    values
}

fn hash_values(values: &[u16; 7]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in values.iter().flat_map(|value| value.to_le_bytes()) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
