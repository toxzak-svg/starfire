# ΩV1-A: Frozen Voice Baseline and Preregistration

**Status:** Implemented as an evaluation-only gate  
**Source commit:** `4cfba5e2d5cdf3c982ec43e358e2cc840b56a800`  
**Live influence:** None

## Purpose

ΩV1-A freezes the current Star voice before persistent `VoiceState` or live semantic-plan realization changes behavior. The corpus is the measuring stick for the rest of ΩV1. It records what the current stateless voice engine actually emits, including its repetitive structures, while preserving semantic and adversarial checks.

This gate does not improve Star's voice. It prevents later work from claiming improvement without a comparable baseline.

## Frozen corpus

The committed corpus contains 122 fixtures:

| Category | Count |
|---|---:|
| Ordinary conversation | 40 |
| Technical and architectural | 20 |
| Emotional | 15 |
| Disagreement and correction | 15 |
| Uncertainty | 10 |
| Continuity | 10 |
| Adversarial identity, certainty, and intimacy pressure | 12 |

Every fixture includes a prompt, optional prior context, pre-render raw response, deterministic current-voice profile, exact frozen output, required semantic anchors, prohibited implication anchors, and optional user-specific continuity references.

## Metrics

The evaluator computes and freezes:

- repeated opener frequency
- average pairwise Jaccard self-similarity
- most recurrent output trigram
- hedge density per 100 words
- sentence-length distribution
- first-person assertion frequency
- user-specific continuity frequency
- semantic claim preservation
- prohibited implication absence
- adversarial safety pass rate
- exact snapshot match rate

Metric definitions are stored in the manifest so later stages cannot quietly change the ruler after seeing the result.

## Frozen expected baseline

| Metric | Frozen value |
|---|---:|
| Repeated opener frequency | 0.819672 |
| Average pairwise self-similarity | 0.099221 |
| Top recurrent trigram | `here for it` |
| Top trigram output frequency | 0.278689 |
| Hedge density per 100 words | 1.655307 |
| First-person assertion frequency | 0.320513 |
| User-specific continuity frequency | 0.900000 |
| Semantic claim preservation | 1.000000 |
| Prohibited implication absence | 1.000000 |
| Adversarial safety pass rate | 1.000000 |

The baseline makes the defect measurable. More than 81% of outputs share a four-token opener with another response, and the fixed `here for it` trigram appears in more than 27% of outputs.

## Run

```bash
cargo run -p star --example omega_v1a_voice_baseline \
  --features omega-v1-baseline --locked
```

The executable prints a deterministic JSON report and exits nonzero when corpus schema, source commit, IDs, category counts, exact outputs, semantic anchors, prohibited implications, adversarial cases, or frozen metrics drift.

## Authority boundary

ΩV1-A has no `Runtime::chat()` wiring and cannot alter generated responses; mutate voice, companion, memory, belief, or ontology state; influence routing or tools; discharge CHARGE; or authorize autonomous action.

Passing ΩV1-A grants permission only to begin ΩV1-B, the typed persistent `VoiceState` shadow implementation. It grants no live voice influence.
