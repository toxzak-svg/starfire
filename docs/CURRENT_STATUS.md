# Starfire Current Status

> **Snapshot date:** 2026-07-23  
> **Branch covered:** `main`  
> **Purpose:** authoritative present-tense summary

This page records what is actually true on `main`. Plans describe intended work, preregistrations freeze future tests, and result records preserve specific evidence.

## Executive summary

Starfire is an active Rust research system with a functioning local runtime, persistent SQLite state, HTTP API, Next.js client, native reranker, typed response machinery, bounded retrieval infrastructure, and a large suite of controlled cognitive experiments.

It can chat, remember, expose cognitive state, run explicit reasoning and thought endpoints, and vary parts of its response style through persistent runtime voice state. It is not AGI, an established emerging intelligence, a frontier-quality language model, an unrestricted autonomous agent, or evidence of consciousness.

The Emerging Intelligence critical path now contains five merged, default-off offline stages:

- **EI-0A:** canonical developmental episode contracts;
- **EI-0B:** deterministic developmental environments and matched controls;
- **EI-0C:** append-only episode history and exact fresh-state replay;
- **EI-0D:** provenance-bound reversible learning updates with independent harmful-update detection and exact restoration;
- **EI-0E:** a digest-bound terminal preregistration with frozen source, seeds, budgets, thresholds, classifier, report schema, and dormant runner.

These stages provide and freeze the machinery needed to test learning. They do not prove that Starfire improves cumulatively from experience. The immediate stage is **EI-0F**, which must execute the exact frozen experiment once and report PASS or FAIL without tuning.

## Emerging Intelligence critical path

| Stage | Main implementation | Establishes | Does not establish |
|---|---|---|---|
| EI-0A | PR [#155](https://github.com/toxzak-svg/starfire/pull/155), `3fd8ec8` | Versioned cognitive episodes, provenance, deterministic sealing and replay | Persistence, learning or live influence |
| EI-0B | PR [#176](https://github.com/toxzak-svg/starfire/pull/176), `087ca263` | Frozen task partitions, five matched arms, independent evaluation and matched budgets | Improvement from experience |
| EI-0C | PR [#187](https://github.com/toxzak-svg/starfire/pull/187), `979f4798` | Append-only canonical history, digest chaining, corruption rejection and fresh-state reconstruction | Live persistence or learning authority |
| EI-0D | PR [#194](https://github.com/toxzak-svg/starfire/pull/194), `c41e6574` | Fixed-schema offline updates, causal provenance, independent safety evaluation, atomic apply and byte-exact rollback | Cumulative improvement, transfer learning or safe live learning |
| EI-0E | PR [#196](https://github.com/toxzak-svg/starfire/pull/196), `2da7eeed` | Exact terminal source, fixtures, seeds, arms, budgets, hypotheses, thresholds, report schema, deterministic classifier and fail-closed rules | Any terminal experimental result |
| EI-0F | Issue [#200](https://github.com/toxzak-svg/starfire/issues/200) | Will execute and classify the frozen terminal experiment | Live promotion or broad intelligence claims |

EI-0E is frozen under preregistration ID `ei-0e-terminal-v1`, source base `ad03f7d67016e32574f47ba836bc5d52ab42c77b`, and canonical manifest SHA-256 `5b83b27e5c218b6af2c53409d60fa6bf285adcde7ccb05b42505a5d0da290d73`.

Its permanent read-only gate verified the manifest and lock twice, replayed deterministic PASS and FAIL classifier vectors twice, validated canonical schemas and closed authority, checked pinned formatting, and compiled the dormant EI-0F runner without executing it. The retained evidence artifact digest is `sha256:68bb833ceb1e78c64313c1e690a5946476c39e69c86b72a0e20013811177aa30`.

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
| ARISE-A0 edge bridge | **Merged, feature-gated** | Bounded reverse-obligation execution and shadow observation |
| ARISE-A1 semantic-program shadow | **Merged, feature-gated** | Typed terminal-first planning and independent reconstruction in shadow |
| EI-0A contracts | **Implemented, feature-gated** | No runtime wiring |
| EI-0B environment | **Implemented, feature-gated** | Offline matched controls |
| EI-0C ledger | **Implemented, feature-gated** | Canonical history; not live SQLite |
| EI-0D reversible updates | **Implemented, feature-gated** | Isolated offline transaction authority only |
| EI-0E preregistration | **Frozen and merged** | Specification and verification only; no terminal result |
| Live EI learning | **Absent** | Prohibited before terminal evidence and separate promotion |
| Automatic ontology promotion | **Not authorized** | Prohibited during EI-0 |
| Unrestricted tools/actions | **Not authorized** | No general live autonomy boundary |
| Repository-wide Clippy baseline | **Legacy debt tracked** | Rust 1.96+ findings outside scoped surfaces remain under issue #183 |

## Deployment and security

The backend remains defined by `render.yaml` and `Dockerfile`, with the documented hosted API at:

```text
https://starfire-cuee.onrender.com
```

The hosted surface lacks built-in authentication, tenant isolation, production rate limits, and independently authenticated Telegram webhooks. Security and deployment isolation remain prerequisites before shared or public live EI evaluation.

## Response and language path

The codebase still contains a runtime-owned response/voice path and legacy outer live-wrapper lineage. One canonical text authority and one canonical voice state are still needed for clean behavioral attribution.

Verifier-backed improvisation, ARISE, and the STLM/ΩV1 tracks provide bounded planning, reconstruction, wording, and voice experiments. They do not make ordinary chat frontier-fluent, and style gains are not intelligence gains unless they improve a frozen task metric without semantic drift.

## Other research

The companion ladder, relational residual bridge, H-series diagnostics, H13C transfer stress, ΩG grammar work, canaries, and related experiments remain bounded or independent research. They receive EI critical-path credit only when a frozen experiment attributes a causal held-out advantage to them.

## Highest-leverage work

1. execute EI-0F exactly once against the frozen EI-0E package and preserve PASS or FAIL without threshold tuning;
2. secure and isolate HTTP, Telegram, file, command and user-state boundaries;
3. simplify the live response path for clean attribution;
4. eliminate repository-wide Rust 1.96+ Clippy debt without mixing maintenance into EI evidence;
5. keep non-EI research independent until a frozen metric establishes causal relevance.

## Immediate engineering decision

Execute **EI-0F** under issue #200 from the exact EI-0E merge and frozen source identity. Preserve the canonical report, raw logs, environment identity, fresh-state replay, arm-level scores, causal chains, harmful-update rollback evidence, negative transfer, regression, artifact digests, and deterministic classifier output.

Do not modify source, seeds, fixtures, budgets, thresholds, evaluators, runner behavior, schema, classifier or claim language after seeing output. Any mandatory failure is recorded as FAIL, not repaired or reinterpreted.

## Tracking sources

- [EI-0 master tracker](https://github.com/toxzak-svg/starfire/issues/149)
- [EI-0E completed preregistration issue](https://github.com/toxzak-svg/starfire/issues/195)
- [EI-0F terminal execution issue](https://github.com/toxzak-svg/starfire/issues/200)
- [EI-0A contract record](experiments/EI_0A_EPISODE_CONTRACTS.md)
- [EI-0B environment record](experiments/EI_0B_DETERMINISTIC_ENVIRONMENT.md)
- [EI-0C ledger record](experiments/EI_0C_APPEND_ONLY_LEDGER.md)
- [EI-0D implementation record](experiments/EI_0D_REVERSIBLE_UPDATES.md)
- [EI-0D result record](experiments/EI_0D_RESULT.md)
- [EI-0E frozen preregistration](experiments/EI_0E_TERMINAL_PREREGISTRATION.md)
- [EI-0E canonical manifest](experiments/EI_0E_TERMINAL_PREREGISTRATION.json)
- [EI-0E freeze lock](experiments/EI_0E_FREEZE_LOCK.json)
- [EI-0F report schema](experiments/EI_0F_TERMINAL_REPORT.schema.json)
- [Emerging Intelligence pivot](../plans/EMERGING_INTELLIGENCE_PIVOT.md)
- [Experiment index](experiments/README.md)
