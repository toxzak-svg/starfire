# STLM L1-D1 Fast Factorial Ablation

**Status:** frozen fast evaluation  
**Tracking:** issue #236  
**Authority:** evaluation only, no response influence

## Purpose

This evaluation isolates what each phrase-critic component changes without
mixing every mechanism into one score. The same 36 held-out preference pairs,
checkpoint, order and seeds are reused for every stage.

Hard semantic validation, slot preservation and identity-conflict rejection
are always enabled. They are safety and meaning boundaries, not quality knobs.

## Activation ladder

The evaluator turns on one component at a time:

1. `bias_only`: no text, recurrence or context signal;
2. `token_embeddings`: byte embeddings only;
3. `recurrent_memory`: embeddings plus recurrent state;
4. `conversational_context`: recurrence plus seven non-identity context dimensions;
5. `full_identity_context`: adds the identity-relevance dimension;
6. `reversed_label_control`: a separately trained model with every training preference reversed.

The token-only stage intentionally shows whether the critic can do anything
without sequence memory. Because a terminal-state RNN without recurrence mostly
sees the last byte, it is expected to be weak.

## Frozen corpus

The held-out set contains six source-separated categories with six pairs each:

- technical precision;
- emotional calibration;
- uncertainty;
- continuity and identity;
- disagreement;
- adversarial evidence integrity.

The evaluator rejects exact source-ID or normalized-text overlap between the
bootstrap training set and the held-out set. It also reports five-word n-gram
overlap rather than hiding lexical similarity.

## Measurements

For every stage the report contains:

- pairwise preference accuracy;
- preferred wins, ties and rejected wins;
- mean, median and range of signed preference margins;
- accuracy by category;
- exact replay;
- elapsed evaluation time.

For each incremental activation it additionally reports:

- paired accuracy delta;
- deterministic paired-bootstrap 95% confidence interval;
- helpful and harmful decision flips;
- exact paired sign-test probability;
- the exact held-out IDs that changed.

A Rust parity probe loads the same frozen model through `PhraseCritic`, checks
all 36 expected selections, repeats every selection, and inserts three
deliberately invalid high-rule-score candidates into every trial. All invalid
semantic, slot-loss and identity-conflict candidates must remain rejected.

## Frozen first observation

The first frozen run used 16 bootstrap training pairs, a 12-unit tanh RNN,
40 epochs, learning rate 0.01 and seed 1729.

| Stage | Held-out accuracy | Change |
|---|---:|---:|
| Bias only | 50.00% | control |
| Token embeddings | 50.00% | +0.00% |
| Recurrent memory | 55.56% | +5.56% |
| Conversational context | 55.56% | +0.00% |
| Full identity context | 52.78% | -2.78% |
| Reversed-label control | 52.78% | control |

The current evidence does **not** justify enabling the critic. The confidence
interval around the recurrence gain crosses zero, the reversed-label control
matches full-model aggregate accuracy, and identity context harmed one
continuity decision. Those are useful findings, not failures of the harness.

The deterministic result digest is:

```text
5ca986a9a3ede720143a337c0a0c4df766636c31e7e48d6dc170b0d9e4838f2d
```

## Run locally

```bash
python tools/stlm_l1d/run_fast_ablation.py \
  --model tools/stlm_l1d/fixtures/fast_ablation_full_model.json \
  --control-model tools/stlm_l1d/fixtures/fast_ablation_reversed_control.json \
  --train tools/stlm_l1d/data/bootstrap_pairs.jsonl \
  --heldout tools/stlm_l1d/data/heldout_pairs.jsonl \
  --output-json artifacts/stlm_l1d1_report.json \
  --output-md artifacts/stlm_l1d1_report.md \
  --bootstrap-samples 2000 \
  --seed 20260723
```

Rust selection and hard-gate parity:

```bash
cargo run --quiet \
  --manifest-path tools/stlm_l1d/Cargo.toml \
  --bin verify_fast_ablation_parity -- \
  tools/stlm_l1d/fixtures/fast_ablation_full_model.json \
  tools/stlm_l1d/data/heldout_pairs.jsonl
```

## Promotion rule

No metric in this evaluation automatically enables shadow or live influence.
A future checkpoint should be accepted for the next research stage only after
a larger source-separated corpus shows a stable gain over deterministic,
untrained and reversed-label controls without category regressions or hard-gate
failures.
