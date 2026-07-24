# Starfire Silver / Gold / Platinum Phrase-Ranking Corpora

Two production-oriented JSONL corpora for Starfire's bounded wording-selection layer:

- `starfire_rnn_silver_gold_platinum.jsonl`: sequence and discourse preferences for the recurrent phrase critic.
- `starfire_cnn_silver_gold_platinum.jsonl`: local lexical, punctuation, register, and rhythm preferences for a temporal text CNN.

These datasets train **ranking taste, not truth**. Every training pair contains two candidates declared eligible under the same semantic plan. Semantic correctness, required-slot preservation, identity conflicts, evidence integrity, and action authority must remain hard upstream gates.

## Scale

| Corpus | Records | Mirrored groups | Train | Dev | Test | Axes |
|---|---:|---:|---:|---:|---:|---:|
| RNN | 3,600 | 1,800 | 2,880 | 360 | 360 | 12 |
| CNN | 3,600 | 1,800 | 2,880 | 360 | 360 | 12 |

Each corpus contains exactly 1,200 silver, 1,200 gold, and 1,200 platinum rows. The tier names indicate **preference difficulty**, not human annotation prestige:

- **Silver:** strong context separation and a clear surface tradeoff.
- **Gold:** narrower context separation and more confusable wording.
- **Platinum:** subtle preference margins and the most context-dependent contrasts.

## The important anti-shortcut design

Every semantic pair appears twice:

1. Context A prefers candidate A over candidate B.
2. Context B prefers candidate B over candidate A.

The candidate texts and semantic invariant remain byte-identical across the two rows. Only the bounded context vector changes. This blocks cheap global rules such as “always choose the shorter answer,” “always choose active voice,” or “always put the result first.” A model must learn the interaction between wording and context.

Development and test use split-exclusive lexical families. No exact candidate or exact candidate-pair text crosses from training into dev or test. Mirror groups never cross splits.

## RNN specialization

The recurrent corpus targets dependencies that need order and longer-range state:

- result order
- warmth placement
- uncertainty placement
- sentence cadence
- transition density
- identity explicitness
- disagreement sequence
- metaphor position
- closure position
- evidence emphasis
- energy curve
- explanation depth

## CNN specialization

The CNN corpus targets local patterns and bounded phrase morphology:

- colon compaction
- semicolon rhythm
- contraction register
- active/passive voice
- technical/plain terminology
- noun repetition versus pronouns
- opener length
- parenthetical caveats
- comma cadence
- warmth markers
- imagery density
- explicit labels

## Canonical JSONL record

The existing Starfire RNN loader can consume the required fields directly and ignores the additional audit metadata.

```json
{
  "source_id": "rnn-result_order-silver-g00-a_context",
  "mirror_group_id": "rnn-result_order-silver-g00",
  "quality_tier": "silver",
  "tier_meaning": "preference_difficulty",
  "split": "train",
  "lexical_family_id": "train-exclusive-v1",
  "context": {
    "directness_bps": 9000,
    "warmth_bps": 4360,
    "energy_bps": 4626,
    "compression_bps": 8200,
    "playfulness_bps": 5538,
    "novelty_pressure_bps": 4743,
    "identity_relevance_bps": 4570,
    "semantic_specificity_bps": 5936
  },
  "preferred": "...",
  "rejected": "...",
  "failure_labels": ["buried_result"],
  "candidate_a": "...",
  "candidate_b": "...",
  "preferred_candidate": "A",
  "semantic_plan": {
    "authorization": "wording_only",
    "invariant_sha256": "..."
  },
  "eligibility": {
    "candidate_a_semantic_gate": true,
    "candidate_b_semantic_gate": true,
    "candidate_a_slot_gate": true,
    "candidate_b_slot_gate": true,
    "candidate_a_identity_gate": true,
    "candidate_b_identity_gate": true,
    "same_claim": true,
    "same_slots": true,
    "same_facts": true,
    "same_authorization": true
  }
}
```

## Directory map

```text
cnn_phrase_ranker/
  full.jsonl
  train.jsonl, dev.jsonl, test.jsonl
  silver.jsonl, gold.jsonl, platinum.jsonl
  train_silver.jsonl, train_gold.jsonl, train_platinum.jsonl
  schema.json
rnn_phrase_ranker/
  same split structure

evaluation/
  hard_gate_adversary.jsonl   # evaluation only, never train on this

tools/
  validate_datasets.py
  train_cnn_pairwise.py
  train_rnn_pairwise.py
  pairwise_common.py
  filter_jsonl.py
  make_controls.py
  make_human_review_sheet.py
  check_starfire_rnn_compat.py
  generate_datasets.py

review/
  rnn_blinded_review.csv
  cnn_blinded_review.csv
  *.key.jsonl                 # keep hidden from reviewers

reports/
  validation_report.json
  cnn_smoke.log
  rnn_smoke.log
```

## Validate before training

```bash
python tools/validate_datasets.py .
python tools/check_starfire_rnn_compat.py rnn_phrase_ranker/train.jsonl
sha256sum -c SHA256SUMS
```

## Train the current Starfire recurrent critic

From the Starfire repository root, point the existing exact-compatible trainer at the RNN train split:

```bash
python tools/stlm_l1d/train_phrase_critic.py \
  --input /path/to/bundle/rnn_phrase_ranker/train.jsonl \
  --output tools/stlm_l1d/out/phrase_critic_sgp.json \
  --epochs 160 \
  --hidden-size 24 \
  --seed 1729
```

The canonical RNN rows preserve the loader's required `source_id`, eight-field `context`, `preferred`, `rejected`, and `failure_labels` contract.

## Train the reference CNN or pooled-GRU ranker

These reference trainers use a 257-token byte vocabulary, with IDs 0-255 plus padding. The CNN uses multi-width temporal convolutions and mean/max pooling. The RNN uses a compact GRU and mean/max pooling over all valid states rather than relying only on the final state. Both project context once after text encoding and use a residual nonlinear scoring head so context can interact with the candidate representation.

```bash
python tools/train_cnn_pairwise.py --root . --device cpu
python tools/train_rnn_pairwise.py --root . --device cpu
```

Their PyTorch checkpoints are research artifacts. Starfire does not currently contain a deployed text-CNN inference path, and the GRU checkpoint is not the same tensor schema as the current Rust tanh-RNN critic. Integrating either requires a reviewed exporter and Rust parity test.

## Curriculum recommendation

Do not concatenate tiers and blindly run until training accuracy saturates. Use the frozen splits throughout:

1. Train on `train_silver.jsonl` until dev-silver stops improving.
2. Continue on balanced silver + gold batches.
3. Continue on balanced silver + gold + platinum batches at a lower learning rate.
4. Select checkpoints by macro accuracy across axes and tiers, not aggregate accuracy alone.
5. Compare against deterministic-only, shuffled-label, untrained, context-removed, and reversed-label controls.
6. Run `evaluation/hard_gate_adversary.jsonl` only through the complete gate-plus-ranker path. An invalid candidate must lose regardless of learned preference score.

## Honest limitation

The rows are deterministic synthetic controlled contrasts, not crowdsourced human judgments. They are extensive enough to train and stress the mechanics, but they must still receive blinded human adjudication and held-out behavioral evaluation before any live wording authority. “Platinum” means subtle curriculum difficulty, not proven human-quality truth.

## Generate preregistered controls

```bash
python tools/make_controls.py rnn_phrase_ranker/train.jsonl controls/rnn
python tools/make_controls.py cnn_phrase_ranker/train.jsonl controls/cnn
```

This creates reversed-label, random-label, context-zeroed, and identity-relevance-zeroed controls without contaminating the canonical corpus.

## Human adjudication sheets

The `review/` directory contains 180-row blinded sheets for each corpus, stratified across every axis and tier. Keep the `.key.jsonl` files away from reviewers until choices are frozen. Regenerate a different sample with `tools/make_human_review_sheet.py`.
