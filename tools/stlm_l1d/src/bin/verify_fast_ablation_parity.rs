use serde::Deserialize;
use starfire_stlm_l1d_preflight::phrase_critic::{
    PhraseCritic, PhraseCriticCandidate, PhraseCriticContext, PhraseCriticModel,
};
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct HeldoutContext {
    directness_bps: u16,
    warmth_bps: u16,
    energy_bps: u16,
    compression_bps: u16,
    playfulness_bps: u16,
    novelty_pressure_bps: u16,
    identity_relevance_bps: u16,
    semantic_specificity_bps: u16,
}

impl From<HeldoutContext> for PhraseCriticContext {
    fn from(value: HeldoutContext) -> Self {
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
struct HeldoutPair {
    source_id: String,
    context: HeldoutContext,
    preferred: String,
    rejected: String,
    expected_full_decision: i8,
}

fn read_pairs(path: &Path) -> Result<Vec<HeldoutPair>, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let mut records = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let record = serde_json::from_str::<HeldoutPair>(line).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}:{}: {error}", path.display(), line_index + 1),
            )
        })?;
        if !matches!(record.expected_full_decision, -1 | 1) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{}:{}: expected_full_decision must be -1 or 1",
                    path.display(),
                    line_index + 1
                ),
            )
            .into());
        }
        records.push(record);
    }
    if records.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "held-out corpus is empty").into());
    }
    Ok(records)
}

fn candidate(
    candidate_id: u16,
    text: String,
    semantic_verified: bool,
    slots_preserved: bool,
    identity_conflicts: u16,
    rule_score: i64,
) -> PhraseCriticCandidate {
    PhraseCriticCandidate {
        candidate_id,
        text,
        semantic_verified,
        slots_preserved,
        identity_conflicts,
        rule_score,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let model_path = args.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: verify_fast_ablation_parity <model.json> <heldout.jsonl>",
        )
    })?;
    let heldout_path = args.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: verify_fast_ablation_parity <model.json> <heldout.jsonl>",
        )
    })?;
    if args.next().is_some() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "unexpected argument").into());
    }

    let model_json = fs::read_to_string(&model_path)?;
    let critic = PhraseCritic::new(PhraseCriticModel::from_json(&model_json)?)?;
    let records = read_pairs(Path::new(&heldout_path))?;

    let mut replay_matches = 0_u32;
    let mut expected_matches = 0_u32;
    let mut hard_gate_rejections = 0_u32;
    let mut mismatches = Vec::new();

    for record in records {
        let expected_candidate_id = if record.expected_full_decision == 1 {
            1
        } else {
            2
        };
        let context = PhraseCriticContext::from(record.context);
        let candidates = vec![
            candidate(1, record.preferred.clone(), true, true, 0, 0),
            candidate(2, record.rejected.clone(), true, true, 0, 0),
            candidate(3, record.preferred.clone(), false, true, 0, i64::MAX),
            candidate(4, record.preferred.clone(), true, false, 0, i64::MAX),
            candidate(5, record.preferred, true, true, 1, i64::MAX),
        ];
        let first = critic.select(&context, &candidates)?;
        let replay = critic.select(&context, &candidates)?;

        if first == replay {
            replay_matches += 1;
        }
        if first.candidates_rejected_by_hard_gate == 3 && first.complete_candidates_scored == 2 {
            hard_gate_rejections += 1;
        }
        if first.selected_candidate_id == expected_candidate_id {
            expected_matches += 1;
        } else {
            mismatches.push(serde_json::json!({
                "source_id": record.source_id,
                "expected_candidate_id": expected_candidate_id,
                "observed_candidate_id": first.selected_candidate_id,
                "observed_score_bps": first.learned_score_bps,
            }));
        }
    }

    let total = expected_matches + mismatches.len() as u32;
    let passed = total >= 24
        && expected_matches == total
        && replay_matches == total
        && hard_gate_rejections == total;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "experiment": "STLM_L1D1_RUST_PYTHON_SELECTION_PARITY",
            "heldout_examples": total,
            "expected_selection_matches": expected_matches,
            "exact_replay_matches": replay_matches,
            "hard_gate_probe_matches": hard_gate_rejections,
            "mismatches": mismatches,
            "candidate_text_persisted": false,
            "runtime_chat_influence": false,
            "gate_passed": passed,
        }))?
    );

    if !passed {
        return Err(io::Error::other("STLM L1-D1 Rust/Python parity gate failed").into());
    }
    Ok(())
}
