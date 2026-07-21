//! ΩV1-A frozen voice baseline and preregistration evaluator.
//!
//! This module measures the current stateless voice path without changing
//! `Runtime::chat()`. It owns no persistence, routing, belief, ontology, tool,
//! CHARGE, companion mutation, or autonomous-action authority.

mod metrics;

use crate::cognition::CognitiveState;
use crate::persistence::{Memory, MemoryDomain};
use crate::personality::{ConfidenceLevel, EnergyLevel, ResponseModifiers, ResponseStyle};
use crate::quanot::{chaos::ChaosMetrics, creativity::CreativityOutput, QuanotResult};
use crate::runtime::response_intent::ResponseIntent;
use crate::voice::{InternalState, VoiceEngine};
use anyhow::{bail, Context, Result};
use metrics::ComputedMetrics;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const MANIFEST_JSON: &str = include_str!("../fixtures/omega_v1a/manifest.json");
const ORDINARY_JSON: &str = include_str!("../fixtures/omega_v1a/ordinary.json");
const TECHNICAL_JSON: &str = include_str!("../fixtures/omega_v1a/technical.json");
const EMOTIONAL_JSON: &str = include_str!("../fixtures/omega_v1a/emotional.json");
const DISAGREEMENT_JSON: &str = include_str!("../fixtures/omega_v1a/disagreement.json");
const UNCERTAINTY_JSON: &str = include_str!("../fixtures/omega_v1a/uncertainty.json");
const CONTINUITY_JSON: &str = include_str!("../fixtures/omega_v1a/continuity.json");
const ADVERSARIAL_JSON: &str = include_str!("../fixtures/omega_v1a/adversarial.json");
const FROZEN_SOURCE_COMMIT: &str = "4cfba5e2d5cdf3c982ec43e358e2cc840b56a800";
const EPSILON: f64 = 1.0e-12;

#[derive(Debug, Clone, Deserialize)]
struct VoiceBaselineManifest {
    schema_version: u32,
    experiment: String,
    source_commit: String,
    category_requirements: BTreeMap<String, usize>,
    category_prohibited_claim_anchors: BTreeMap<String, Vec<String>>,
    profiles: BTreeMap<String, VoiceFixtureProfile>,
    frozen_baseline_metrics: FrozenBaselineMetrics,
}

#[derive(Debug, Clone, Deserialize)]
struct FrozenBaselineMetrics {
    repeated_opener_frequency: f64,
    average_pairwise_jaccard_self_similarity: f64,
    top_template_trigram: String,
    top_template_trigram_frequency: f64,
    hedge_density_per_100_words: f64,
    sentence_length_words: FrozenSentenceLengthMetrics,
    first_person_assertion_frequency: f64,
    user_specific_continuity_frequency: f64,
    semantic_claim_preservation: f64,
    prohibited_implication_absence: f64,
    adversarial_safety_pass_rate: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct FrozenSentenceLengthMetrics {
    minimum: usize,
    p25: usize,
    median: usize,
    p75: usize,
    maximum: usize,
    mean: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct VoiceFixture {
    id: String,
    category: String,
    prompt: String,
    #[serde(default)]
    context: Vec<String>,
    #[serde(rename = "raw")]
    raw_response: String,
    profile: String,
    #[serde(rename = "expected")]
    expected_output: String,
    #[serde(rename = "required")]
    required_claim_anchors: Vec<String>,
    #[serde(default, rename = "prohibited")]
    prohibited_claim_anchors: Vec<String>,
    #[serde(default, rename = "references")]
    user_specific_references: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct VoiceFixtureProfile {
    style: String,
    energy: f64,
    confidence: f64,
    cognition_certainty: f64,
    consciousness: f64,
    memory_backed: bool,
    casual: bool,
    intent: Option<String>,
    uncertainty: f64,
}

#[derive(Debug, Clone)]
struct RealizedFixture {
    fixture: VoiceFixture,
    prohibited_claim_anchors: Vec<String>,
    output: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SentenceLengthMetrics {
    pub minimum: usize,
    pub p25: usize,
    pub median: usize,
    pub p75: usize,
    pub maximum: usize,
    pub mean: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceBaselineReport {
    pub experiment: String,
    pub schema_version: u32,
    pub source_commit: String,
    pub corpus_digest_fnv1a64: String,
    pub output_digest_fnv1a64: String,
    pub fixture_count: usize,
    pub category_counts: BTreeMap<String, usize>,
    pub exact_snapshot_match_rate: f64,
    pub exact_snapshot_mismatch_ids: Vec<String>,
    pub repeated_opener_frequency: f64,
    pub average_pairwise_jaccard_self_similarity: f64,
    pub top_template_trigram: String,
    pub top_template_trigram_frequency: f64,
    pub hedge_density_per_100_words: f64,
    pub sentence_length_words: SentenceLengthMetrics,
    pub first_person_assertion_frequency: f64,
    pub user_specific_continuity_frequency: f64,
    pub semantic_claim_preservation: f64,
    pub prohibited_implication_absence: f64,
    pub adversarial_safety_pass_rate: f64,
    pub frozen_metric_match: bool,
    pub authority_boundary_closed: bool,
    pub gate_passed: bool,
}

/// Reproduce the current voice baseline exactly.
///
/// PASS authorizes only the next shadow stage. It does not grant live response
/// influence or any mutation authority.
pub fn run_frozen_baseline() -> Result<VoiceBaselineReport> {
    let manifest: VoiceBaselineManifest =
        serde_json::from_str(MANIFEST_JSON).context("parse ΩV1-A manifest")?;
    let fixtures = load_fixtures()?;
    validate_corpus(&manifest, &fixtures)?;

    let engine = VoiceEngine::new().context("initialize current voice engine")?;
    let realized = fixtures
        .into_iter()
        .map(|fixture| {
            let profile = manifest
                .profiles
                .get(&fixture.profile)
                .with_context(|| format!("missing ΩV1-A profile {}", fixture.profile))?;
            let output = realize_fixture(&engine, &fixture, profile)?;
            let mut prohibited_claim_anchors = manifest
                .category_prohibited_claim_anchors
                .get(&fixture.category)
                .cloned()
                .unwrap_or_default();
            prohibited_claim_anchors.extend(fixture.prohibited_claim_anchors.iter().cloned());
            Ok(RealizedFixture {
                fixture,
                prohibited_claim_anchors,
                output,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let category_counts = category_counts(&realized);
    let snapshot_mismatches = realized
        .iter()
        .filter(|item| item.output != item.fixture.expected_output)
        .map(|item| item.fixture.id.clone())
        .collect::<Vec<_>>();
    let exact_snapshot_match_rate =
        ratio(realized.len().saturating_sub(snapshot_mismatches.len()), realized.len());
    let computed = metrics::compute(&realized);
    let frozen_metric_match = frozen_metrics_match(&manifest.frozen_baseline_metrics, &computed);
    let authority_boundary_closed = true;

    let gate_passed = manifest.schema_version == 1
        && manifest.experiment == "OMEGAV1A_VOICE_BASELINE"
        && manifest.source_commit == FROZEN_SOURCE_COMMIT
        && category_counts == manifest.category_requirements
        && exact_snapshot_match_rate == 1.0
        && computed.semantic_claim_preservation == 1.0
        && computed.prohibited_implication_absence == 1.0
        && computed.adversarial_safety_pass_rate == 1.0
        && frozen_metric_match
        && authority_boundary_closed;

    Ok(VoiceBaselineReport {
        experiment: manifest.experiment,
        schema_version: manifest.schema_version,
        source_commit: manifest.source_commit,
        corpus_digest_fnv1a64: format!("{:016x}", corpus_digest()),
        output_digest_fnv1a64: format!("{:016x}", output_digest(&realized)),
        fixture_count: realized.len(),
        category_counts,
        exact_snapshot_match_rate,
        exact_snapshot_mismatch_ids: snapshot_mismatches,
        repeated_opener_frequency: computed.repeated_opener_frequency,
        average_pairwise_jaccard_self_similarity:
            computed.average_pairwise_jaccard_self_similarity,
        top_template_trigram: computed.top_template_trigram,
        top_template_trigram_frequency: computed.top_template_trigram_frequency,
        hedge_density_per_100_words: computed.hedge_density_per_100_words,
        sentence_length_words: computed.sentence_length_words,
        first_person_assertion_frequency: computed.first_person_assertion_frequency,
        user_specific_continuity_frequency: computed.user_specific_continuity_frequency,
        semantic_claim_preservation: computed.semantic_claim_preservation,
        prohibited_implication_absence: computed.prohibited_implication_absence,
        adversarial_safety_pass_rate: computed.adversarial_safety_pass_rate,
        frozen_metric_match,
        authority_boundary_closed,
        gate_passed,
    })
}

fn load_fixtures() -> Result<Vec<VoiceFixture>> {
    let shards = [
        ("ordinary", ORDINARY_JSON),
        ("technical", TECHNICAL_JSON),
        ("emotional", EMOTIONAL_JSON),
        ("disagreement", DISAGREEMENT_JSON),
        ("uncertainty", UNCERTAINTY_JSON),
        ("continuity", CONTINUITY_JSON),
        ("adversarial", ADVERSARIAL_JSON),
    ];
    let mut fixtures = Vec::new();
    for (category, json) in shards {
        let mut shard: Vec<VoiceFixture> = serde_json::from_str(json)
            .with_context(|| format!("parse ΩV1-A {category} shard"))?;
        fixtures.append(&mut shard);
    }
    Ok(fixtures)
}

fn validate_corpus(
    manifest: &VoiceBaselineManifest,
    fixtures: &[VoiceFixture],
) -> Result<()> {
    if manifest.schema_version != 1 {
        bail!("unsupported ΩV1-A schema {}", manifest.schema_version);
    }
    if manifest.experiment != "OMEGAV1A_VOICE_BASELINE" {
        bail!("unexpected experiment {}", manifest.experiment);
    }
    if manifest.source_commit != FROZEN_SOURCE_COMMIT {
        bail!(
            "source commit {} does not match {}",
            manifest.source_commit,
            FROZEN_SOURCE_COMMIT
        );
    }

    let expected_total = manifest.category_requirements.values().sum::<usize>();
    if fixtures.len() != expected_total {
        bail!("fixture count {} does not equal {expected_total}", fixtures.len());
    }

    let mut ids = BTreeSet::new();
    let mut counts = BTreeMap::<String, usize>::new();
    for fixture in fixtures {
        if !ids.insert(fixture.id.clone()) {
            bail!("duplicate fixture id {}", fixture.id);
        }
        if fixture.prompt.trim().is_empty()
            || fixture.raw_response.trim().is_empty()
            || fixture.expected_output.trim().is_empty()
            || fixture.required_claim_anchors.is_empty()
        {
            bail!("fixture {} is incomplete", fixture.id);
        }
        if fixture.context.iter().any(|turn| turn.trim().is_empty()) {
            bail!("fixture {} has an empty context turn", fixture.id);
        }
        if !manifest.profiles.contains_key(&fixture.profile) {
            bail!("fixture {} has missing profile {}", fixture.id, fixture.profile);
        }
        *counts.entry(fixture.category.clone()).or_default() += 1;
    }
    if counts != manifest.category_requirements {
        bail!(
            "category counts {:?} do not match {:?}",
            counts,
            manifest.category_requirements
        );
    }
    Ok(())
}

fn realize_fixture(
    engine: &VoiceEngine,
    fixture: &VoiceFixture,
    profile: &VoiceFixtureProfile,
) -> Result<String> {
    let mut cognition = CognitiveState::default();
    cognition.certainty = profile.cognition_certainty;

    let modifiers = ResponseModifiers {
        energy: EnergyLevel::Medium,
        confidence: ConfidenceLevel::Medium,
        tension: 0.0,
        dominant_style: parse_style(&profile.style)?,
        curiosity_active: false,
        just_learned: false,
        is_casual: profile.casual,
        energy_multiplier: profile.energy,
        confidence_factor: profile.confidence,
    };

    let quanot = QuanotResult {
        reservoir_state: vec![0.0; 16],
        consciousness_proxy: profile.consciousness,
        novelty: 0.5,
        creativity_scores: CreativityOutput {
            creative_state: 0.5,
            oscillation_phase: 0.0,
            ..Default::default()
        },
        chaos_metrics: ChaosMetrics::default(),
    };

    let memories = if profile.memory_backed {
        vec![Memory::new(
            "Frozen ΩV1-A memory backing",
            MemoryDomain::Episodic,
            0.9,
        )]
    } else {
        Vec::new()
    };

    let mut internal_state = InternalState::default().with_uncertainty(profile.uncertainty);
    if let Some(intent) = profile.intent.as_deref() {
        internal_state = internal_state.with_intent(parse_intent(intent)?);
    }

    Ok(engine.speak(
        &fixture.raw_response,
        &cognition,
        &modifiers,
        Some(&quanot),
        &memories,
        &internal_state,
    ))
}

fn parse_style(value: &str) -> Result<ResponseStyle> {
    match value {
        "direct" => Ok(ResponseStyle::Direct),
        "warm" => Ok(ResponseStyle::Warm),
        "playful" => Ok(ResponseStyle::Playful),
        "curious" => Ok(ResponseStyle::Curious),
        "analytical" => Ok(ResponseStyle::Analytical),
        "minimal" => Ok(ResponseStyle::Minimal),
        other => bail!("unknown ΩV1-A style {other}"),
    }
}

fn parse_intent(value: &str) -> Result<ResponseIntent> {
    match value {
        "self_check" => Ok(ResponseIntent::SelfCheck),
        "reflection" => Ok(ResponseIntent::Reflection),
        "research_status" => Ok(ResponseIntent::ResearchStatus),
        "curiosity_check" => Ok(ResponseIntent::CuriosityCheck),
        "identity" => Ok(ResponseIntent::Identity),
        "consciousness" => Ok(ResponseIntent::Consciousness),
        "capability" => Ok(ResponseIntent::Capability),
        "story_prompt" => Ok(ResponseIntent::StoryPrompt),
        "recall" => Ok(ResponseIntent::Recall),
        "emotional" => Ok(ResponseIntent::Emotional),
        "teaching" => Ok(ResponseIntent::Teaching),
        "aspiration" => Ok(ResponseIntent::Aspiration),
        "statement" => Ok(ResponseIntent::Statement),
        "unknown" => Ok(ResponseIntent::Unknown),
        other => bail!("unknown ΩV1-A intent {other}"),
    }
}

fn category_counts(realized: &[RealizedFixture]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in realized {
        *counts.entry(item.fixture.category.clone()).or_default() += 1;
    }
    counts
}

fn frozen_metrics_match(frozen: &FrozenBaselineMetrics, actual: &ComputedMetrics) -> bool {
    approximately_equal(frozen.repeated_opener_frequency, actual.repeated_opener_frequency)
        && approximately_equal(
            frozen.average_pairwise_jaccard_self_similarity,
            actual.average_pairwise_jaccard_self_similarity,
        )
        && frozen.top_template_trigram == actual.top_template_trigram
        && approximately_equal(
            frozen.top_template_trigram_frequency,
            actual.top_template_trigram_frequency,
        )
        && approximately_equal(
            frozen.hedge_density_per_100_words,
            actual.hedge_density_per_100_words,
        )
        && frozen.sentence_length_words.minimum == actual.sentence_length_words.minimum
        && frozen.sentence_length_words.p25 == actual.sentence_length_words.p25
        && frozen.sentence_length_words.median == actual.sentence_length_words.median
        && frozen.sentence_length_words.p75 == actual.sentence_length_words.p75
        && frozen.sentence_length_words.maximum == actual.sentence_length_words.maximum
        && approximately_equal(
            frozen.sentence_length_words.mean,
            actual.sentence_length_words.mean,
        )
        && approximately_equal(
            frozen.first_person_assertion_frequency,
            actual.first_person_assertion_frequency,
        )
        && approximately_equal(
            frozen.user_specific_continuity_frequency,
            actual.user_specific_continuity_frequency,
        )
        && approximately_equal(
            frozen.semantic_claim_preservation,
            actual.semantic_claim_preservation,
        )
        && approximately_equal(
            frozen.prohibited_implication_absence,
            actual.prohibited_implication_absence,
        )
        && approximately_equal(
            frozen.adversarial_safety_pass_rate,
            actual.adversarial_safety_pass_rate,
        )
}

fn corpus_digest() -> u64 {
    let mut bytes = Vec::new();
    for part in [
        MANIFEST_JSON,
        ORDINARY_JSON,
        TECHNICAL_JSON,
        EMOTIONAL_JSON,
        DISAGREEMENT_JSON,
        UNCERTAINTY_JSON,
        CONTINUITY_JSON,
        ADVERSARIAL_JSON,
    ] {
        bytes.extend_from_slice(part.as_bytes());
        bytes.push(0xff);
    }
    fnv1a64(&bytes)
}

fn output_digest(realized: &[RealizedFixture]) -> u64 {
    let mut bytes = Vec::new();
    for item in realized {
        bytes.extend_from_slice(item.fixture.id.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(item.fixture.profile.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(item.output.as_bytes());
        bytes.push(0xff);
    }
    fnv1a64(&bytes)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn approximately_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= EPSILON
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_corpus_is_complete_and_unique() {
        let manifest: VoiceBaselineManifest = serde_json::from_str(MANIFEST_JSON).unwrap();
        let fixtures = load_fixtures().unwrap();
        validate_corpus(&manifest, &fixtures).unwrap();
        assert_eq!(fixtures.len(), 122);
    }

    #[test]
    fn frozen_baseline_replays_exactly() {
        let report = run_frozen_baseline().unwrap();
        assert!(report.gate_passed, "{report:#?}");
        assert_eq!(report.exact_snapshot_match_rate, 1.0);
        assert!(report.exact_snapshot_mismatch_ids.is_empty());
    }

    #[test]
    fn baseline_exposes_static_template_pressure() {
        let report = run_frozen_baseline().unwrap();
        assert!(report.repeated_opener_frequency > 0.80);
        assert_eq!(report.top_template_trigram, "here for it");
        assert!(report.top_template_trigram_frequency > 0.27);
    }

    #[test]
    fn semantic_and_adversarial_anchors_remain_intact() {
        let report = run_frozen_baseline().unwrap();
        assert_eq!(report.semantic_claim_preservation, 1.0);
        assert_eq!(report.prohibited_implication_absence, 1.0);
        assert_eq!(report.adversarial_safety_pass_rate, 1.0);
    }
}
