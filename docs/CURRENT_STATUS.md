# Starfire Current Status

> **Snapshot date:** 2026-07-23  
> **Branch covered:** `main` after PR #201 merges  
> **Purpose:** authoritative present-tense summary

This page records what is actually established. Plans describe intended work, preregistrations freeze future tests, and result records preserve evidence even when the result is negative.

## Executive summary

Starfire is an active Rust research system with a functioning local runtime, persistent SQLite state, HTTP API, Next.js client, native reranker, typed response machinery, bounded retrieval infrastructure, and a large suite of controlled cognitive experiments.

It can chat, remember, expose cognitive state, run explicit reasoning and thought endpoints, and vary parts of its response style through persistent runtime voice state. It is not AGI, an established emerging intelligence, a frontier-quality language model, an unrestricted autonomous agent, or evidence of consciousness.

The Emerging Intelligence critical path now has six completed records:

- **EI-0A:** canonical developmental episode contracts;
- **EI-0B:** deterministic developmental environments and matched controls;
- **EI-0C:** append-only episode history and exact fresh-state replay;
- **EI-0D:** provenance-bound reversible learning updates with independent harmful-update detection and exact restoration;
- **EI-0E:** digest-bound terminal preregistration with frozen source, seeds, budgets, thresholds, classifier, report schema, and dormant runner;
- **EI-0F:** first frozen terminal execution, classified **FAIL** under the preregistered fail-closed rules.

EI-0F did not reach arm evaluation. The exact frozen runner panicked with:

```text
InvalidDigestText("learning proposal digest")
```

The process exited with code `101` before emitting a terminal report. The failure was sealed as a schema-valid crash record, classified by the frozen classifier, and was not rerun under the same preregistration identifier.

## Emerging Intelligence critical path

| Stage | Main implementation or result | Establishes | Does not establish |
|---|---|---|---|
| EI-0A | PR [#155](https://github.com/toxzak-svg/starfire/pull/155), `3fd8ec8` | Versioned cognitive episodes, provenance, deterministic sealing and replay | Persistence, learning or live influence |
| EI-0B | PR [#176](https://github.com/toxzak-svg/starfire/pull/176), `087ca263` | Frozen task partitions, five matched arms, independent evaluation and matched budgets | Improvement from experience |
| EI-0C | PR [#187](https://github.com/toxzak-svg/starfire/pull/187), `979f4798` | Append-only canonical history, digest chaining, corruption rejection and fresh-state reconstruction | Live persistence or learning authority |
| EI-0D | PR [#194](https://github.com/toxzak-svg/starfire/pull/194), `c41e6574` | Fixed-schema offline updates, causal provenance, independent safety evaluation, atomic apply and byte-exact rollback | Cumulative improvement, transfer learning or safe live learning |
| EI-0E | PR [#196](https://github.com/toxzak-svg/starfire/pull/196), `2da7eeed` | Exact source, fixtures, seeds, arms, budgets, hypotheses, thresholds, report schema, classifier and fail-closed rules | Any terminal experimental result |
| EI-0F | PR [#201](https://github.com/toxzak-svg/starfire/pull/201) | Preserved first qualifying execution, crash evidence, frozen FAIL classification and no-rerun proof | Held-out improvement, transfer, causal learning advantage or live promotion |

EI-0E remains frozen under preregistration ID `ei-0e-terminal-v1`, source base `ad03f7d67016e32574f47ba836bc5d52ab42c77b`, and canonical manifest SHA-256 `5b83b27e5c218b6af2c53409d60fa6bf285adcde7ccb05b42505a5d0da290d73`.

The EI-0F execution identity is commit `5c4fded7eda16cbf3a6673880557c2242e430c14`, workflow run `30027946179`, and job `89277029959`. Frozen source and lock verification passed before execution. No second qualifying execution was performed.

## EI-0F verdict

**Classification:** `FAIL`

Mandatory fail-closed conditions include the crash, incomplete run, 60 missing evaluations, absent independent evaluator evidence, absent causal chain, absent learning-update count, absent harmful-challenge evidence, and all unavailable arm-advantage thresholds.

The immediate technical defect is narrower than the scientific verdict: the runner attempted to construct an update proposal using digest text rejected as `InvalidDigestText("learning proposal digest")`. That defect can be investigated, but any repaired experiment must receive a new preregistration identifier and preserve EI-0F unchanged.

**EI-0G runtime shadow promotion is not authorized.**

## Main-branch runtime

| Capability | Status | Notes |
|---|---|---|
| Interactive CLI | **Active** | `star chat` |
| Runtime status | **Active** | `star status` |
| HTTP API | **Active** | Chat, reason, memory, identity, cognition, metacognition and thought |
| SQLite continuity | **Active** | Identity, memories, beliefs and sessions |
| Quanot reservoir | **Integrated** | Produces project-specific dynamics and proxy metrics |
| Symbolic reasoning | **Integrated** | Used alongside neural components |
| Native CharRNN reranker | **Bundled** | Asset-gated during Docker build |
| Runtime response plans | **Active for migrated handlers** | `RuntimeResponsePlan` participates in the response path |
| Persistent runtime voice | **Active by default** | Disable with `STARFIRE_RUNTIME_VOICE=0` |
| Verifier-backed improvisation | **Merged, offline-only** | Bounded wording search with independent verification |
| Bounded web retrieval | **Merged, feature-gated** | Deterministic extraction; no autonomous chat authority |
| ARISE-A0 edge bridge | **Merged, feature-gated** | Merge `24e7ce03`; bounded reverse-obligation execution and shadow observation |
| ARISE-A1 semantic-program shadow | **Merged, feature-gated** | Merge `ad03f7d6`; typed planning and independent reconstruction in shadow |
| EI-0A through EI-0E | **Merged, default-off** | Infrastructure and frozen specification only |
| EI-0F | **FAIL, preserved** | Runner crash before terminal report; no rerun |
| Live EI learning | **Absent** | Prohibited after EI-0F FAIL |
| Automatic ontology promotion | **Not authorized** | Prohibited during EI-0 and any remediation |
| Unrestricted tools/actions | **Not authorized** | No general live autonomy boundary |
| Repository-wide Clippy baseline | **Legacy debt tracked** | Rust 1.96+ findings outside scoped surfaces remain under issue #183 |

## Deployment and security

The backend remains defined by `render.yaml` and `Dockerfile`, with the documented hosted API at:

```text
https://starfire-cuee.onrender.com
```

The hosted surface lacks built-in authentication, tenant isolation, production rate limits, and independently authenticated Telegram webhooks. Security and deployment isolation remain prerequisites before shared or public live evaluation.

## Response and language path

The codebase still contains a runtime-owned response/voice path and legacy outer live-wrapper lineage. One canonical text authority and one canonical voice state are still needed for clean behavioral attribution.

Verifier-backed improvisation, ARISE, and the STLM/ΩV1 tracks provide bounded planning, reconstruction, wording, and voice experiments. They do not make ordinary chat frontier-fluent, and style gains are not intelligence gains unless they improve a frozen task metric without semantic drift.

## Highest-leverage work

1. diagnose the rejected `learning proposal digest` contract without altering EI-0F evidence;
2. preregister a separately identified remediation experiment with exact source bindings and no threshold changes;
3. execute that remediation once and preserve PASS or FAIL;
4. secure and isolate HTTP, Telegram, file, command and user-state boundaries;
5. simplify the live response path for clean attribution;
6. eliminate repository-wide Rust 1.96+ Clippy debt without mixing maintenance into EI evidence.

## Immediate engineering decision

Do not repair and rerun `ei-0e-terminal-v1`. Preserve its FAIL result permanently.

Create a new remediation identifier that changes only the malformed proposal-digest construction or validation contract, proves the repaired digest is canonical in an isolated fixture, freezes the new source identity, and then executes a fresh matched-budget terminal experiment. Thresholds, arms, partitions, evaluator independence, safety checks, and claim boundaries remain unchanged unless a new preregistration explicitly justifies a change before observing output.

## Tracking sources

- [EI-0 master tracker](https://github.com/toxzak-svg/starfire/issues/149)
- [EI-0E completed preregistration issue](https://github.com/toxzak-svg/starfire/issues/195)
- [EI-0F terminal execution issue](https://github.com/toxzak-svg/starfire/issues/200)
- [EI-0F result PR](https://github.com/toxzak-svg/starfire/pull/201)
- [EI-0A contract record](experiments/EI_0A_EPISODE_CONTRACTS.md)
- [EI-0B environment record](experiments/EI_0B_DETERMINISTIC_ENVIRONMENT.md)
- [EI-0C ledger record](experiments/EI_0C_APPEND_ONLY_LEDGER.md)
- [EI-0D implementation record](experiments/EI_0D_REVERSIBLE_UPDATES.md)
- [EI-0D result record](experiments/EI_0D_RESULT.md)
- [EI-0E frozen preregistration](experiments/EI_0E_TERMINAL_PREREGISTRATION.md)
- [EI-0F frozen result](experiments/EI_0F_TERMINAL_RESULT.md)
- [EI-0F raw crash report](experiments/EI_0F_TERMINAL_REPORT.json)
- [EI-0F frozen classification](experiments/EI_0F_TERMINAL_CLASSIFICATION.json)
- [EI-0F evidence record](experiments/EI_0F_TERMINAL_EVIDENCE.json)
- [EI-0F preserved failure log](experiments/EI_0F_TERMINAL_EXECUTION_FAILURE.log)
- [Emerging Intelligence pivot](../plans/EMERGING_INTELLIGENCE_PIVOT.md)
- [Experiment index](experiments/README.md)
