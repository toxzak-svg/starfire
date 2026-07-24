# STLM L1-D3 Recovery Preregistration

Status: diagnostic-only recovery program. No critic promotion, runtime chat influence, HTTP response influence, routing authority, semantic authority, or model-weight change is permitted by this document or by L1-D3A.

## Trigger

STLM L1-D2 showed that the frozen recurrent phrase critic does not generalize to valid-surface tournaments. The deterministic scorer remained stronger, while the bounded learned residual damaged otherwise correct selections. L1-D3 therefore begins with failure attribution rather than additional training.

## L1-D3A question

L1-D3A asks why the frozen critic fails and which evidence must exist before any replacement model is trained or promoted.

It must measure:

- every seed-specific held-out tournament and candidate score;
- deterministic-correct to bounded-wrong flips;
- bounded corrections of deterministic errors;
- category-specific regressions;
- critic correlation with length, punctuation, and authored rule score;
- context-shuffle sensitivity;
- punctuation, whitespace, Unicode, and terminal-ending sensitivity;
- output saturation and residual clipping;
- bootstrap training-target contamination by semantic, confidence, evidence, identity, and authority labels;
- coupling between authored `rule_score` and `gold_candidate_id`;
- deterministic report replay and frozen source digests.

L1-D3A is successful when those diagnostics are complete and reproducible. A successful L1-D3A run does not mean the critic is useful.

## Frozen diagnosis hierarchy

The initial hypotheses are ordered as follows:

1. Training-target mismatch. The bootstrap pair corpus mixes surface-quality preferences with semantic, evidence, confidence, identity, and authority differences.
2. Objective mismatch. Pairwise preferred/rejected training does not directly model four-to-eight-candidate valid-surface tournaments.
3. Encoder shortcut exposure. A raw-byte terminal-state RNN can over-weight punctuation, endings, length, and final-state artifacts.
4. Context-conditioning weakness. Repeating one context projection at every byte can wash out or distort context instead of conditioning a pooled surface representation.
5. Residual mismatch. The model is trained as an unrestricted preference scorer and only clipped into a residual after training.
6. Benchmark provenance uncertainty. Authored rule scores and gold labels require an independent-provenance audit in Dataset V2.

Longer training on the current 16-pair bootstrap corpus and wider residual authority are explicitly rejected as recovery actions.

## Recovery sequence

### L1-D3A: Failure attribution and dataset contract

Freeze the diagnostic report, harmful-flip ledger, shortcut measurements, rule/gold coupling audit, and promotion criteria. Preserve L1-D2 as a negative result.

### L1-D3B: Valid-surface Dataset V2

Construct source-separated Silver, Gold, and Platinum tournament sets. Every tournament must preserve semantic plan, claims, confidence, entities, quantities, slots, and action status across candidates. Gold adjudication and deterministic rule scoring must have independent provenance. Paraphrase siblings and generator lineages must not cross grouped splits.

### L1-D3C: Matched-budget critic laboratory

Compare a correctly trained version of the current RNN, a compact surface CNN, and a compact CNN plus gated recurrent model under identical data, parameter, epoch, and compute budgets. No transformer or sentence generator is introduced.

### L1-D3D: Residual-native objective

Train the model to emit its actual bounded correction rather than an unrestricted probability. The objective must combine listwise ranking, pairwise margin, deterministic-error correction, residual magnitude penalty, neutral-perturbation consistency, and context-contrast terms.

### L1-D3E: Selective intervention

Add abstention and deterministic-gap gating. The critic must return zero residual when confidence is insufficient or the deterministic winner is not a plausible near-tie.

### L1-D3F: Frozen promotion evaluation

Run one preregistered evaluation across five seeds and source-grouped held-out data. No tuning may use Platinum or the original L1-D2 holdout.

## Promotion gates

A learned residual may be proposed for a separate explicit promotion review only when every gate passes:

1. Mean top-1 gain over the deterministic ranker is at least 300 basis points.
2. The paired 95 percent confidence interval for the gain excludes zero.
3. At least four of five seeds improve.
4. No category regresses by more than 200 basis points.
5. Newly damaged deterministic-correct selections remain below 200 basis points.
6. The learned system beats the hashed n-gram baseline.
7. Correct context beats shuffled context on a preregistered context-sensitive subset.
8. A context-neutral subset remains stable under context shuffling.
9. Neutral punctuation, whitespace, Unicode, and length controls retain at least 9000 basis points selection stability.
10. Semantic-invalid hard-gate rejection remains complete.
11. Python and Rust inference are exactly identical.
12. Runtime chat and HTTP authority remain closed throughout experimentation.
13. No successful workflow may promote a model automatically.

## Authority boundary

The critic may rank wording only after semantic validity, slot preservation, identity consistency, evidence custody, and action authority are already resolved. It may not rewrite claims, change confidence, create facts, promote identities or concepts, route tools, choose actions, or influence live output until a later explicit promotion review.

## L1-D3A artifacts

The workflow must upload:

- `stlm_l1d3a_report.json` with complete traces and ledgers;
- `stlm_l1d3a_report.md` with the human-readable diagnosis;
- materialization logs for the frozen checkpoint and corpus;
- the report digest and deterministic replay result.

The source of truth is the JSON report. The Markdown file is a rendering of that report.
