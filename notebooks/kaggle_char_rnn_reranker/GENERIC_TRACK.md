# Generic CharRNN Reranker Track

## Boundary

This track never reads the private `starfire-personal-reranker-corpus` dataset,
its checkpoints, or its training outputs. It produces a separately named
candidate checkpoint and metrics artifact.

## Source

- Dataset: `OpenAssistant/oasst1`, pinned to its downloaded revision in the
  output manifest.
- License: Apache-2.0.
- Selection: English, non-deleted, non-synthetic, reviewed assistant and
  prompter messages reconstructed into alternating conversation paths.
- Exclusions: messages labelled as spam, unsuitable, toxic, sexual, or
  personally identifying; malformed paths; duplicate normalized text.

## Kaggle job contract

The generic kernel uses a GPU but downloads the public corpus itself with
internet enabled. It saves:

- `starfire_generic_reranker_v1.bin` in the exact `CharRNN::save` format;
- `metrics.json` with source revision, row counts, filtering counts, finite
  training/validation NLL, perplexity, and the fixed seed;
- `samples.json` with deterministic prompt completions;
- `rejected.json` instead of a checkpoint if any finite-loss, source, or
  format gate fails.

## Promotion gates

1. Source manifest and license are present.
2. No non-finite loss or weight occurs.
3. Held-out validation NLL is better than the randomly initialized control.
4. The downloaded binary passes local `CharRNN::load` and the reranker smoke.
5. A blinded human review compares generic candidate, personal candidate, and
   deterministic fallback on representative runtime response plans.

The generic candidate is not an automatic fallback replacement and is not mixed
with the personal model.
