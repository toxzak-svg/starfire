# REFLEX LAYER — Starfire Input Routing

## Concept
Port the Reflex Layer from `context/` (Zach's governed belief memory) into Starfire.
Sub-50ms regex classifier that runs before LLM/KV-cache work — routes every input
to the right memory partitions and authority mode with zero I/O.

**Pattern:** Input → Domain Classify → Authority Gate → Retrieval Plan

## What it does
Given raw user text, outputs:
- **Domain** — what kind of thing is this?
- **Authority** — who decides the truth?
- **Retrieval plan** — which memory partitions to query, in what order

## Domains
| Domain | Description | Authority | Retrieval Plan |
|--------|-------------|-----------|----------------|
| `Ego` | Questions about Star's identity/existence | `SelfSovereign` | HWS |
| `Identity` | User declares facts about themselves | `UserDeclaration` | HWS, UAS |
| `Preference` | User states likes/dislikes/wants | `UserDeclaration` | HWS, UAS |
| `Intent` | User expresses goals/plans | `UserDeclaration` | HWS, UAS, ETL |
| `Empirical` | Factual questions requiring evidence | `EvidenceBased` | HWS, EBG, ETL |
| `Procedural` | How-to, steps, workflows | `OutcomeWeighted` | HWS, PSM, ETL |
| `Social` | People, culture, relationships | `EvidenceBased` | HWS, EBG |
| `Aesthetic` | Beauty, design, style | `UserDeclaration` | HWS, UAS |
| `Meta` | About memory, history, the system | `SystemManaged` | HWS, UAS, EBG, ETL |
| `Mission` | Policy/mission constraint queries | `SystemManaged` | MPS |

## Authority Types
- `SelfSovereign` — Star decides from self-knowledge
- `UserDeclaration` — user says it, system records
- `EvidenceBased` — reasoning + external evidence
- `OutcomeWeighted` — success history weighted
- `SystemManaged` — internal state only

## Retrieval Plan Partitions (Starfire equivalents)
- `HWS` → Hot working set (recent conversation)
- `UAS` → User attribute store (identity/preferences)
- `EBG` → Evidence-backed beliefs (world model)
- `PSM` → Procedural memory (how-to)
- `ETL` → Episodic timeline (events)
- `MPS` → Mission/policy store

## Placement in Starfire pipeline
```
Raw input
  → InputNormalizer  (cleanup, casing)
  → ReflexLayer      (domain classify + route — ZERO I/O, <1ms)
  → [Authority check] — can skip LLM for ego/preference?
  → Memory search    (partition-aware)
  → LLM reasoning    (if needed)
  → Response
```

## Files
New: `lib/reflex/mod.rs` + `lib/reflex/rules.rs`

## Next
1. Implement `ReflexLayer` in Rust with compiled regex rules
2. Wire into `InputNormalizer::normalize()` or a new `ReflexLayer::classify()` method
3. Use domain+authority to pick memory partitions before full search
