# STLM L1-D2 Valid-Surface Benchmark and Critic V2 Preflight

**Status:** implemented preflight, evaluation only  
**Predecessor:** [`STLM_L1D1_FAST_ABLATION.md`](STLM_L1D1_FAST_ABLATION.md)  
**Authority:** offline ranking research, no runtime or HTTP response influence

## Purpose

L1-D1 established a deterministic harness, but its token-embedding stage was not a meaningful embedding ablation and its pair corpus mixed surface preference with examples whose rejected side changed meaning. L1-D2 replaces that measurement with a valid-surface benchmark that asks a narrower question:

> Among candidates that already express the same verified semantic plan and preserve the same required slots, can a bounded critic improve surface selection beyond deterministic and untrained controls?

This PR is a preflight for that question. A green workflow proves the benchmark structure, controls, replay, authority boundary, and Rust/Python parity. It does not prove that the current checkpoint improves language quality.

## Corpus separation

The quality benchmark contains 36 source-grouped tournaments across six categories:

- technical precision;
- emotional calibration;
- uncertainty;
- continuity and identity;
- disagreement;
- adversarial evidence integrity.

Every tournament contains four to eight candidates. Every candidate in this corpus is marked semantic-valid, slot-preserving, and identity-consistent. The gold label therefore represents surface quality, not permission to alter the claim.

Semantic-invalid candidates live in a separate 12-probe corpus. Those examples are never included in quality metrics or learned-baseline fitting. They exist only to verify that semantic drift, slot loss, and identity conflict remain unrankable even when their deterministic scores are enormous.

The exact corpus paths, digests, counts, seeds, controls, and authority settings are frozen in `tools/stlm_l1d/fixtures/valid_surface_manifest.json`.

## Grouped five-seed evaluation

Each of the five preregistered seeds creates a category-stratified group split:

| Partition | Tournaments | Per category |
|---|---:|---:|
| Train | 18 | 3 |
| Development | 6 | 1 |
| Test | 12 | 2 |

A `group_id` appears in exactly one partition for a seed. The evaluator rejects any train, development, or test group overlap. Results are reported per seed and aggregated as mean, minimum, maximum, and the complete per-seed vector rather than collapsing seed variance into one lucky number.

## Rankers and ablations

The benchmark compares six ranking paths on the same held-out tournaments:

1. **Deterministic rule rank:** the upstream score alone.
2. **Pooled embeddings:** mean pooling across every encoded byte, followed by the existing output head. This is order-invariant and uses the whole surface rather than accidentally reducing to the final byte.
3. **Recurrent critic:** the complete exported recurrent checkpoint.
4. **Bounded learned residual:** deterministic rule score plus a learned adjustment capped at ±250 basis points per candidate.
5. **Hashed n-gram baseline:** deterministic 4,096-dimensional character and word n-gram features with context buckets and pairwise linear training.
6. **Length-only baseline:** a matched learned baseline that receives only text-length features.

The hashed n-gram and length baselines are trained only on the grouped training partition. Hyperparameters are selected on the grouped development partition, then reported once on the held-out grouped test partition for each seed.

## Required controls

Every seed reports:

- shuffled conversational context within category;
- shuffled training labels for the hashed n-gram baseline;
- punctuation normalization;
- whitespace normalization;
- Unicode normalization;
- a learned length-only baseline;
- a length-matched test subset.

The normalization controls report both held-out quality and selection stability relative to the unmodified text. This exposes shortcuts where the critic follows punctuation, spacing, byte fallback, or raw length instead of wording quality.

## Critic V2 authority boundary

The learned score is no longer the primary sort key. It is converted to a centered residual:

```text
residual = clamp(learned_score_bps - 5000, -250, +250)
combined = deterministic_rule_score + residual
```

The maximum learned swing between two candidates is therefore 500 points. A deterministic lead greater than 500 cannot be overturned by the critic. At exactly 500, deterministic rule score remains the tie-breaker. Candidate identity is the final deterministic tie-breaker.

Hard semantic, slot-preservation, and identity-conflict gates still execute before ranking. The critic cannot score rejected candidates, return candidate text, persist candidate text, affect `Runtime::chat`, alter HTTP responses, promote beliefs or ontology, select tools, route actions, or authorize autonomous behavior.

## Structural acceptance gate

The L1-D2 workflow passes only when all of the following hold:

- exactly five distinct preregistered seeds run;
- the valid-surface corpus has at least 36 tournaments and 36 unique groups;
- all six categories are represented;
- every tournament has four to eight candidates;
- every quality candidate passes all hard semantic gates;
- grouped splits have no overlap and retain category coverage;
- all seven required controls are present for every seed;
- the pooled-embedding ablation uses the complete surface and is order-invariant;
- the hashed n-gram and length-only baselines execute on grouped partitions;
- all semantic-invalid probes are rejected by the hard gates;
- the bounded residual cannot exceed ±250 per candidate or 500 pairwise;
- Python full-report replay is exact;
- Rust selection replay, tournament budgets, hard gates, and residual arithmetic pass;
- the evaluation completes within the workflow budget;
- live authority remains closed.

No quality score is a structural pass condition. This avoids turning a weak checkpoint into a benchmark failure or a strong checkpoint into automatic promotion.

## Run locally

Materialize the digest-bound benchmark corpora:

```bash
python tools/stlm_l1d/materialize_valid_surface_corpora.py
```

Materialize the existing digest-bound L1-D1 checkpoint:

```bash
mkdir -p artifacts/models
python tools/stlm_l1d/materialize_fast_ablation_models.py \
  --manifest tools/stlm_l1d/fixtures/fast_ablation_manifest.json \
  --output-dir artifacts/models
```

Run the Python benchmark:

```bash
python tools/stlm_l1d/run_valid_surface_benchmark.py \
  --model artifacts/models/fast_ablation_full_model.json \
  --surface-corpus tools/stlm_l1d/data/valid_surface_tournaments.jsonl \
  --semantic-invalid-corpus tools/stlm_l1d/data/semantic_invalid_candidates.jsonl \
  --output-json artifacts/stlm_l1d2_report.json \
  --output-md artifacts/stlm_l1d2_report.md \
  --seeds 1729,2718,3141,5772,8119 \
  --max-runtime-seconds 90
```

Run Rust tournament and authority parity:

```bash
cargo run --quiet \
  --manifest-path tools/stlm_l1d/Cargo.toml \
  --bin verify_valid_surface_tournaments -- \
  artifacts/models/fast_ablation_full_model.json \
  tools/stlm_l1d/data/valid_surface_tournaments.jsonl \
  tools/stlm_l1d/data/semantic_invalid_candidates.jsonl
```

## Promotion rule

L1-D2 does not enable the critic. A later, separately frozen checkpoint may advance only after it demonstrates a stable held-out gain over deterministic rule ranking and the hashed n-gram baseline across grouped seeds, survives all shortcut controls without category regressions, preserves exact replay, and remains within the bounded residual authority contract.
