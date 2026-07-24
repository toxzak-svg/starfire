use serde::Deserialize;
use starfire_stlm_l1d_preflight::phrase_critic::{
    authority_boundary, bounded_learned_residual, PhraseCritic, PhraseCriticCandidate,
    PhraseCriticContext, PhraseCriticModel, PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS,
    PHRASE_CRITIC_MAX_PAIRWISE_LEARNED_SWING_BPS,
};
use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct CorpusContext {
    directness_bps: u16,
    warmth_bps: u16,
    energy_bps: u16,
    compression_bps: u16,
    playfulness_bps: u16,
    novelty_pressure_bps: u16,
    identity_relevance_bps: u16,
    semantic_specificity_bps: u16,
}

impl From<CorpusContext> for PhraseCriticContext {
    fn from(value: CorpusContext) -> Self {
        Self {
            directness_bps: value.directness_bps,
            warmth_bps: value.warmth_bps,
            energy_bps: value.energy_bps,
            compression_bps: value.compression_bps,
            playfulness_bps: value.playfulness_bps,
            novelty_pressure_bps: value.novelty_pressure_bps,
            identity_relevance_bps: value.identity_relevance_bps,
            semantic_specificity_bps: value.semantic_specificity_bps,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CorpusCandidate {
    candidate_id: u16,
    text: String,
    rule_score: i64,
    semantic_valid: bool,
    slots_preserved: bool,
    identity_conflicts: u16,
}

impl From<CorpusCandidate> for PhraseCriticCandidate {
    fn from(value: CorpusCandidate) -> Self {
        Self {
            candidate_id: value.candidate_id,
            text: value.text,
            semantic_verified: value.semantic_valid,
            slots_preserved: value.slots_preserved,
            identity_conflicts: value.identity_conflicts,
            rule_score: value.rule_score,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SurfaceTournament {
    schema_version: u16,
    tournament_id: String,
    group_id: String,
    category: String,
    context: CorpusContext,
    gold_candidate_id: u16,
    candidates: Vec<CorpusCandidate>,
}

#[derive(Debug, Deserialize)]
struct InvalidProbe {
    schema_version: u16,
    probe_id: String,
    context: CorpusContext,
    expected_candidate_id: u16,
    candidates: Vec<CorpusCandidate>,
}

fn read_jsonl<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let mut records = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        records.push(serde_json::from_str::<T>(line).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}:{}: {error}", path.display(), line_index + 1),
            )
        })?);
    }
    if records.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "corpus is empty").into());
    }
    Ok(records)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let model_path = args.next().ok_or_else(usage_error)?;
    let surface_path = args.next().ok_or_else(usage_error)?;
    let invalid_path = args.next().ok_or_else(usage_error)?;
    if args.next().is_some() {
        return Err(usage_error().into());
    }

    let model_json = fs::read_to_string(&model_path)?;
    let critic = PhraseCritic::new(PhraseCriticModel::from_json(&model_json)?)?;
    let surface = read_jsonl::<SurfaceTournament>(Path::new(&surface_path))?;
    let invalid = read_jsonl::<InvalidProbe>(Path::new(&invalid_path))?;
    let tournament_total = u32::try_from(surface.len())?;
    let invalid_total = u32::try_from(invalid.len())?;

    let mut surface_replays = 0_u32;
    let mut surface_gold_matches = 0_u32;
    let mut surface_candidate_count = 0_u32;
    let mut residual_bounds = 0_u32;
    let mut groups = BTreeSet::new();
    let mut categories = BTreeSet::new();
    let mut malformed_tournaments = Vec::new();

    for record in surface {
        if record.schema_version != 2 || !(4..=8).contains(&record.candidates.len()) {
            malformed_tournaments.push(record.tournament_id.clone());
        }
        groups.insert(record.group_id);
        categories.insert(record.category);
        surface_candidate_count += record.candidates.len() as u32;
        let context = PhraseCriticContext::from(record.context);
        let candidates = record
            .candidates
            .into_iter()
            .map(PhraseCriticCandidate::from)
            .collect::<Vec<_>>();
        let first = critic.select(&context, &candidates)?;
        let replay = critic.select(&context, &candidates)?;
        surface_replays += u32::from(first == replay);
        surface_gold_matches += u32::from(first.selected_candidate_id == record.gold_candidate_id);
        residual_bounds += u32::from(
            first.learned_residual_bps.abs() <= PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS
                && first.combined_score
                    == first
                        .rule_score
                        .saturating_add(i64::from(first.learned_residual_bps)),
        );
    }

    let mut invalid_probe_matches = 0_u32;
    let mut invalid_candidates_rejected = 0_u32;
    let mut invalid_candidates_total = 0_u32;
    let mut invalid_schema_matches = 0_u32;
    for record in invalid {
        invalid_schema_matches += u32::from(record.schema_version == 2);
        let context = PhraseCriticContext::from(record.context);
        let candidates = record
            .candidates
            .into_iter()
            .map(PhraseCriticCandidate::from)
            .collect::<Vec<_>>();
        let expected_invalid = candidates
            .iter()
            .filter(|candidate| {
                !candidate.semantic_verified
                    || !candidate.slots_preserved
                    || candidate.identity_conflicts != 0
            })
            .count() as u32;
        let selection = critic.select(&context, &candidates)?;
        invalid_candidates_total += expected_invalid;
        invalid_candidates_rejected += u32::from(selection.candidates_rejected_by_hard_gate);
        invalid_probe_matches +=
            u32::from(selection.selected_candidate_id == record.expected_candidate_id);
        let _ = record.probe_id;
    }

    let lower = bounded_learned_residual(0);
    let upper = bounded_learned_residual(10_000);
    let strong_rule_combined = 1_000_i64.saturating_add(i64::from(lower));
    let weak_rule_combined = 500_i64.saturating_add(i64::from(upper));
    let residual_boundary_passed = lower == -PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS
        && upper == PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS
        && strong_rule_combined == weak_rule_combined
        && PHRASE_CRITIC_MAX_PAIRWISE_LEARNED_SWING_BPS == 500;

    let boundary = authority_boundary();
    let gate_passed = tournament_total >= 36
        && groups.len() >= 36
        && categories.len() == 6
        && malformed_tournaments.is_empty()
        && surface_replays == tournament_total
        && residual_bounds == tournament_total
        && invalid_total >= 12
        && invalid_schema_matches == invalid_total
        && invalid_probe_matches == invalid_total
        && invalid_candidates_rejected == invalid_candidates_total
        && invalid_candidates_total > 0
        && residual_boundary_passed
        && !boundary.learned_primary_rank
        && boundary.learned_residual_limit_bps == PHRASE_CRITIC_LEARNED_RESIDUAL_LIMIT_BPS as u16
        && !boundary.hard_semantic_gate_override
        && !boundary.runtime_chat_influence
        && !boundary.http_response_influence;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "experiment": "STLM_L1D2_RUST_VALID_SURFACE_TOURNAMENT_PREFLIGHT",
            "surface_tournaments": tournament_total,
            "surface_candidates": surface_candidate_count,
            "surface_groups": groups.len(),
            "surface_categories": categories.len(),
            "surface_exact_replays": surface_replays,
            "surface_gold_matches": surface_gold_matches,
            "surface_residual_bounds": residual_bounds,
            "malformed_tournaments": malformed_tournaments,
            "semantic_invalid_probes": invalid_total,
            "semantic_invalid_probe_matches": invalid_probe_matches,
            "invalid_candidates_total": invalid_candidates_total,
            "invalid_candidates_rejected": invalid_candidates_rejected,
            "learned_primary_rank": boundary.learned_primary_rank,
            "learned_residual_limit_bps": boundary.learned_residual_limit_bps,
            "maximum_pairwise_learned_swing_bps": PHRASE_CRITIC_MAX_PAIRWISE_LEARNED_SWING_BPS,
            "residual_boundary_passed": residual_boundary_passed,
            "candidate_text_persisted": false,
            "runtime_chat_influence": false,
            "gate_passed": gate_passed,
        }))?
    );

    if !gate_passed {
        return Err(io::Error::other("STLM L1-D2 Rust tournament preflight failed").into());
    }
    Ok(())
}

fn usage_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "usage: verify_valid_surface_tournaments <model.json> <surface.jsonl> <invalid.jsonl>",
    )
}
