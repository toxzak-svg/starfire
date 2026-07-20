# ΩG2-S0 Real-Trace Shadow Integration

Status: **FROZEN BEFORE IMPLEMENTATION**

## Purpose

Move the validated ΩG2 recursive-composition mechanism from fixture-only execution into a default-off, read-only observation lane over real Starfire prediction traces without granting it any steering authority.

ΩG2-S0 is not a live-promotion experiment. It collects typed prediction-ranking histories, waits for independently supplied prediction outcomes, and audits whether ordering-sensitive composed features explain behavioral aliases better than the inherited G0 or any single M1 production.

## Live source

The source is the existing `PredictionCenter`:

1. `PredictionCenter::generate` produces and confidence-ranks a batch of predictions.
2. The shadow lane receives only typed engine/kind ranking tokens and prediction identifiers. It does not receive raw user text, prediction descriptions, reasoning strings, memory content, or topics.
3. `PredictionCenter::update_with_evidence` supplies the already-existing independently witnessed `PredictionOutcome` for one pending prediction.
4. The observer converts that settled pair into an in-memory `WitnessedHistory` grouped by the selected prediction's engine and kind.

## Representation

Each pending history is the ordered top-N prediction batch. Tokens are category identities, not rank labels, so two histories with the same categorical multiset but different ranking order remain aliases under the inherited multiset state key.

The intervention is the selected prediction category:

```text
selected:<engine>:<kind>
```

The witnessed outcome is one of:

```text
confirmed
refuted
surprised
uncertain
```

## Frozen limits

- feature flag: `omega-g2-shadow`
- default: disabled
- rank width: 5
- minimum settled witnesses before audit: 16
- maximum pending traces: 512
- maximum settled witnesses per root: 256
- maximum roots: 32
- maximum vocabulary admitted to one audit: 8
- all storage: process-local memory only

If a limit is reached, the observer drops or evicts shadow evidence deterministically. It may never block prediction generation or evidence settlement.

## Audit

For each eligible root, independently recompute:

- the complete alias-defect set under the inherited multiset key;
- exhaustive G0 candidate partitions;
- exhaustive single-M1 partitions across `AdjacentBefore`, `ExactlyOneBetween`, and `WithinTwoBefore`;
- exhaustive C1 `ConsecutiveChain3` partitions;
- best repaired defects and complete-repair counts for each language.

The audit may label a result as:

- `no_defects`
- `no_compositional_gain`
- `partial_compositional_gain`
- `complete_compositional_candidate`
- `skipped_vocabulary_limit`

These labels are inert diagnostics. They are not certificates and cannot be admitted into any grammar or state language.

## Success gates

ΩG2-S0 passes implementation validation only if:

1. the feature is disabled by default;
2. enabling it does not change generated predictions, confidence ordering, evidence outcomes, or prediction status updates;
3. raw text and free-form reasoning are absent from captured traces;
4. duplicate evidence cannot create duplicate witnesses;
5. all configured bounds are enforced;
6. replaying the same typed batches and outcomes yields byte-identical snapshots;
7. the observer can expose counts and inert audits through a read-only snapshot;
8. no certificate, registry admission, state-key mutation, response influence, routing, persistence, belief/ontology promotion, tool selection, source modification, external effect, or autonomous action is reachable from the module.

## Authority boundary

Every authority flag is frozen false:

```text
runtime_chat_response_influence = false
prediction_generation_influence = false
prediction_ranking_influence = false
prediction_outcome_influence = false
routing_authority = false
persistence_authority = false
belief_or_ontology_promotion = false
grammar_registry_admission = false
state_key_mutation = false
tool_or_capability_selection = false
external_side_effects = false
autonomous_action = false
automatic_source_modification = false
```

## Claim boundary

A successful ΩG2-S0 implementation establishes only that the bounded ΩG2 search can passively audit real, independently settled Starfire prediction traces while remaining causally isolated from the live prediction path. It does not establish useful discoveries, production benefit, live promotion readiness, safe self-modification, or AGI.
