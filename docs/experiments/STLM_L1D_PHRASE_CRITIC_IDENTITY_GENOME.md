# STLM L1-D Phrase Critic and Identity Genome

**Status:** isolated preflight implementation  
**Tracking:** issue #234  
**Authority:** offline-only, default-disconnected

## Purpose

L1-D gives a second small recurrent model one narrow job: rank wording candidates that have already passed independent semantic verification, slot-preservation checks, and identity-conflict checks. It does not generate meaning, interpret the user, rewrite claims, or override a hard verifier.

The paired identity work replaces larger and larger prose seeds with typed claims carrying confidence, provenance, evidence, contradiction state, persistence, expression weight, and retrieval tags.

## Architecture

```text
Semantic response program
        |
Committed bounded candidate lattice
        |
Independent semantic + slot verification
        |
Identity contradiction gate
        |
Tiny recurrent phrase critic
        |
Candidate identifier + bounded scores only
```

The current preflight does not connect the selected candidate to `Runtime::chat`, HTTP, the voice engine, persistence, routing, tools, CHARGE, beliefs, ontology, or actions.

## Recurrent critic

`lib/phrase_critic.rs` implements a portable tanh RNN inference contract:

- 128 byte-token buckets;
- eight bounded context dimensions;
- maximum 128 hidden units;
- maximum 32 candidates and 1,024 bytes per candidate;
- JSON model weights with strict shape and finite-value validation;
- deterministic score replay;
- hard rejection of semantic drift, missing slots, and identity conflicts;
- learned score, deterministic rule score, then candidate ID as the stable ranking order;
- no selected text in the returned selection record.

The learned score is preference, not truth. A candidate with a perfect learned score is still ineligible when any hard gate fails.

## Identity genome

`lib/identity_genome.rs` defines six claim types:

1. invariant;
2. value;
3. self-hypothesis;
4. behavioral tendency;
5. autobiographical evidence;
6. relationship fact.

Each claim carries:

- a stable identifier;
- statement and claim type;
- confidence in basis points;
- provenance;
- evidence references;
- contradiction references;
- persistence class;
- expression weight;
- contextual tags;
- quarantine state.

Self-hypotheses cannot be stored at invariant-like confidence. Invariants require high confidence, evidence, and a clear contradiction state. Contradictory revisable claims are quarantined and excluded from retrieval. Invariant contradictions fail closed for explicit review rather than being silently rewritten.

Contextual retrieval is deterministic and returns at most eight claim IDs. It does not dump the complete identity into every response.

## Training data

`tools/stlm_l1d/data/bootstrap_pairs.jsonl` is a deliberately small bootstrap corpus, not evidence of general conversational quality. Each record contains:

- a source ID;
- the eight bounded context dimensions;
- preferred wording;
- rejected wording;
- failure labels such as `semantic_drift`, `fake_emotion`, `wrong_confidence`, `identity_overclaim`, `bland`, or `overexplained`.

A credible trained critic requires a much larger corpus split by source and scenario so near-duplicate phrasings cannot leak between training and evaluation. Bootstrap pairs may validate mechanics but cannot authorize live influence.

## Kaggle-ready training

The trainer is `tools/stlm_l1d/train_phrase_critic.py`. In a Kaggle notebook or another PyTorch environment, run from the repository root:

```bash
python tools/stlm_l1d/train_phrase_critic.py \
  --input tools/stlm_l1d/data/bootstrap_pairs.jsonl \
  --output /kaggle/working/phrase_critic_v1.json \
  --epochs 160 \
  --hidden-size 24 \
  --seed 1729
```

The script trains a pairwise-ranking tanh RNN and exports the exact tensor orientation consumed by Rust, plus a SHA-256 sidecar. Kaggle is compute only. A checkpoint must return through a separate review with its dataset digest, split manifest, seed, metrics, and held-out comparison report.

## Required evaluation before any shadow attachment

A later preregistered evaluation must compare:

- deterministic STLM scorer only;
- deterministic scorer plus trained critic;
- shuffled-label critic control;
- untrained critic control;
- identity context removed;
- conversational microstate removed.

At minimum it should report pairwise preference accuracy, semantic-verifier survival, slot preservation, identity-conflict rejection, repetition, blandness, calibration, latency, exact replay, and failures by scenario family.

No PASS may grant live text authority automatically. A distinct shadow-attachment stage is required, followed by a separately consented canary if the evidence supports it.

## Preflight commands

```bash
cargo fmt --manifest-path tools/stlm_l1d/Cargo.toml -- --check
cargo test --manifest-path tools/stlm_l1d/Cargo.toml
cargo run --quiet --manifest-path tools/stlm_l1d/Cargo.toml
python -m py_compile tools/stlm_l1d/train_phrase_critic.py
```

The probe emits metadata only. Candidate text is neither returned in its report nor persisted.
