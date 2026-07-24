# Training Guide

## 1. Freeze the data

Run validation and record the canonical file digests before training. Never rewrite failed evaluation rows after observing model behavior. Create a new dataset version instead.

```bash
python tools/validate_datasets.py .
sha256sum -c SHA256SUMS
```

## 2. Preserve mirrored batches

The two rows in a mirror group contain the same candidate pair with opposite preferences under different contexts. Sample both sides at equal frequency. Do not drop one side during deduplication.

Recommended sampler constraints:

- balance by `difference_axis`
- balance by `quality_tier`
- balance preferred candidate A/B
- keep each mirror group wholly in one split
- avoid batches containing only one context regime

## 3. Curriculum

A practical schedule:

- Phase S: silver only, normal learning rate.
- Phase G: 50% silver, 50% gold, learning rate reduced 25%.
- Phase P: equal silver/gold/platinum, learning rate reduced another 25-50%.
- Calibration: no weight updates; fit only a bounded score calibration layer if needed.

Checkpoint selection should maximize the minimum axis accuracy or macro-average across axes. Aggregate accuracy can hide a model that learned three easy axes and failed the rest.

## 4. Required controls

Train and report at least:

- deterministic Starfire scorer only
- learned ranker plus deterministic scorer
- shuffled-label ranker
- reversed-label ranker
- untrained ranker
- context-zeroed ranker
- identity-relevance-zeroed ranker
- candidate-text-only ranker

The mirrored corpus makes a context-free model gravitate toward chance. If a text-only model scores well above chance, inspect leakage or an unintended global surface cue.

## 5. Metrics

Report:

- pairwise accuracy overall
- macro accuracy by axis
- macro accuracy by tier
- mirrored consistency: both sides of a mirror group correct
- mean signed margin
- calibration by preference margin
- exact deterministic replay
- latency and memory
- harmful and helpful flips versus the deterministic scorer
- hard-gate adversary survival

Do not promote from training accuracy.

## 6. Current-compatible RNN path

The canonical RNN JSONL works with Starfire's present `tools/stlm_l1d/train_phrase_critic.py` loader. Extra metadata is ignored. Use `rnn_phrase_ranker/train.jsonl`, never `full.jsonl`, for fitting.

## 7. Reference CNN path

`tools/train_cnn_pairwise.py` is a research trainer for a local-pattern critic:

- byte IDs 0-255 plus padding
- embedding layer
- temporal convolution widths 2, 3, 4, 5, and 7
- masked mean and max pooling
- one bounded context projection
- residual nonlinear score head
- pairwise softplus ranking loss

It does not grant live authority and does not include a Rust exporter.

## 8. Reference pooled-GRU path

`tools/train_rnn_pairwise.py` avoids final-state-only scoring:

- byte embedding
- compact GRU
- masked mean and max pooling across all valid recurrent states
- one bounded context projection after sequence encoding
- residual nonlinear score head
- pairwise softplus ranking loss

This is an architecture comparison path, not a drop-in replacement for the current Rust tanh-RNN model schema.

## 9. Promotion boundary

A passing offline result authorizes only the next preregistered experiment. It does not automatically authorize shadow attachment, canary deployment, live response selection, identity mutation, or verifier bypass.
