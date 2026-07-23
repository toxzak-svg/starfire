# Starfire Current Status

> **Snapshot date:** 2026-07-23  
> **Branch covered:** `main`  
> **Purpose:** authoritative present-tense summary

This page records what is actually true on `main`. Plans describe intended work, preregistrations freeze future tests, and result records preserve specific evidence.

## Executive summary

Starfire is an active Rust research system with a functioning local runtime, persistent SQLite state, HTTP API, Next.js client, native reranker, typed response machinery, bounded retrieval infrastructure, and a large suite of controlled cognitive experiments.

It can chat, remember, expose cognitive state, run explicit reasoning and thought endpoints, and vary parts of its response style through persistent runtime voice state. It is not AGI, an established emerging intelligence, a frontier-quality language model, an unrestricted autonomous agent, or evidence of consciousness.

The Emerging Intelligence critical path now contains four merged, default-off offline stages:

- **EI-0A:** canonical developmental episode contracts;
- **EI-0B:** deterministic developmental environments and matched controls;
- **EI-0C:** append-only episode history and exact fresh-state replay;
- **EI-0D:** provenance-bound reversible learning updates with independent harmful-update detection and exact restoration.

These stages provide the machinery needed to test learning. They do not prove that Starfire improves cumulatively from experience. The immediate stage is **EI-0E**, which freezes the terminal experiment before EI-0F is run.

## Emerging Intelligence critical path

| Stage | Main implementation | Establishes | Does not establish |
|---|---|---|---|
| EI-0A | PR [#155](https://github.com/toxzak-svg/starfire/pull/155), `3fd8ec8` | Versioned cognitive episodes, provenance, deterministic sealing and replay | Persistence, learning, or live influence |
| EI-0B | PR [#176](https://github.com/toxzak-svg/starfire/pull/176), `087ca263` | Frozen task partitions, five matched arms, independent evaluation and matched budgets | Improvement from experience |
| EI-0C | PR [#187](https://github.com/toxzak-svg/starfire/pull/187), `979f4798` | Append-only canonical history, digest chaining, corruption rejection and fresh-state reconstruction | Live persistence or learning authority |
| EI-0D | PR [#194](https://github.com/toxzak-svg/starfire/pull/194), `c41e6574` | Fixed-schema offline updates, causal provenance, independent safety evaluation, atomic apply and byte-exact rollback | Cumulative improvement, transfer learning or safe live learning |
| EI-0E | Issue [#195](https://github.com/toxzak-svg/starfire/issues/195) | Will freeze the exact terminal hypotheses, source, fixtures, controls, budgets, thresholds and classifier | Any experimental result |

EI-0D's permanent gate passed pinned formatting, full feature and probe compilation, scoped Clippy, focused reversible-update tests, and two byte-identical probe executions. Its retained evidence digest is `sha256:f5bf61c5667aea3253db2fffea4ede72b0c37375fcbbd2abe478673a3026d473`.

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
| Bounded web retrieval | **Merged, feature-gated** | Local-first retrieval and deterministic extraction; no autonomous chat authority |
| EI-0A contracts | **Implemented, feature-gated** | No runtime wiring |
| EI-0B environment | **Implemented, feature-gated** | Offline matched controls |
| EI-0C ledger | **Implemented, feature-gated** | In-memory canonical history; not live SQLite |
| EI-0D reversible updates | **Implemented, feature-gated** | Isolated offline transaction authority only |
| Live EI learning | **Absent** | Prohibited before terminal evidence and separate promotion |
| Automatic ontology promotion | **Not authorized** | Prohibited during EI-0 |
| Unrestricted tools/actions | **Not authorized** | No general live autonomy boundary |
| Repository-wide Clippy baseline | **Legacy debt tracked** | Rust 1.96+ findings outside scoped EI surfaces remain under issue #183 |

## Deployment and security

The backend remains defined by `render.yaml` and `Dockerfile`, with the documented hosted API at:

```text
https://starfire-cuee.onrender.com
```

The hosted surface lacks built-in authentication, tenant isolation, production rate limits, and independently authenticated Telegram webhooks. Security and deployment isolation remain prerequisites before shared or public live EI evaluation.

## Response and language path

The codebase still contains a runtime-owned response/voice path and legacy outer live-wrapper lineage. One canonical text authority and one canonical voice state are still needed for clean behavioral attribution.

Verifier-backed improvisation and the STLM/ΩV1 tracks provide bounded offline wording and voice experiments. They do not make ordinary chat frontier-fluent, and language-style gains are not intelligence gains unless they improve a frozen task metric without semantic drift.

## Other research

The companion ladder, relational residual bridge, H-series diagnostics, H13C transfer stress, ΩG grammar work, ARISE branches, canaries, and related experiments remain bounded or independent research. They receive EI critical-path credit only when a frozen experiment attributes a causal held-out advantage to them.

## Highest-leverage work

1. complete EI-0E by freezing the exact terminal source, fixtures, seeds, arms, budgets, thresholds and PASS/FAIL classifier;
2. run EI-0F once against the frozen preregistration and preserve PASS or FAIL without threshold tuning;
3. secure and isolate HTTP, Telegram, file, command and user-state boundaries;
4. simplify the live response path for clean attribution;
5. eliminate repository-wide Rust 1.96+ Clippy debt without mixing maintenance into EI evidence.

## Immediate engineering decision

Complete **EI-0E** under issue #195. The stage must produce canonical human-readable and machine-readable preregistration records, exact source and lockfile digests, complete fixture and arm manifests, numerical success and regression thresholds, deterministic commands, report schemas, missing-data rules, and a terminal classifier.

EI-0E must not inspect EI-0F results, tune thresholds against observed outcomes, change runtime behavior, expand the update lattice, or claim EI-0 PASS.

## Tracking sources

- [EI-0 master tracker](https://github.com/toxzak-svg/starfire/issues/149)
- [EI-0E preregistration issue](https://github.com/toxzak-svg/starfire/issues/195)
- [EI-0A contract record](experiments/EI_0A_EPISODE_CONTRACTS.md)
- [EI-0B environment record](experiments/EI_0B_DETERMINISTIC_ENVIRONMENT.md)
- [EI-0C ledger record](experiments/EI_0C_APPEND_ONLY_LEDGER.md)
- [EI-0D implementation record](experiments/EI_0D_REVERSIBLE_UPDATES.md)
- [EI-0D result record](experiments/EI_0D_RESULT.md)
- [EI-0E preregistration draft](experiments/EI_0E_TERMINAL_PREREGISTRATION.md)
- [Emerging Intelligence pivot](../plans/EMERGING_INTELLIGENCE_PIVOT.md)
- [Experiment index](experiments/README.md)
