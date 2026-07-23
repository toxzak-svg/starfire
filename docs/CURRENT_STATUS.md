# Starfire Current Status

> **Snapshot date:** 2026-07-23  
> **Branch covered:** `main`  
> **Purpose:** authoritative present-tense summary

This page answers one question: **what is actually true of Starfire right now?**

Plans describe intended work. Preregistrations freeze hypotheses. Result records describe specific runs. This document describes the current main-branch system without promoting draft work or rewriting historical evidence.

## Executive summary

Starfire is an active Rust research system with a functioning local runtime, persistent state, HTTP API, Next.js client, native reranker, typed response machinery, bounded retrieval infrastructure, and a substantial suite of cognitive experiments.

It can chat, remember, expose cognitive state, run explicit reasoning and thought endpoints, and alter parts of its response style through a persistent runtime voice profile. It is deployed as a Docker service on Render and connected to a web interface.

It is not currently AGI, an established emerging intelligence, a frontier-quality language model, an unrestricted autonomous agent, or validated evidence of consciousness.

The Emerging Intelligence critical path now contains three merged, default-off infrastructure stages:

- **EI-0A:** canonical developmental episode contracts;
- **EI-0B:** deterministic developmental environments and matched controls;
- **EI-0C:** append-only episode ledger and exact fresh-state replay.

None of these stages proves that Starfire improves from experience. EI-0D, the reversible learning-update lattice, is the first stage allowed to apply a bounded update, and only inside isolated offline experiment state.

## Active research direction: Emerging Intelligence

PR [#148](https://github.com/toxzak-svg/starfire/pull/148) merged the active [Emerging Intelligence pivot](../plans/EMERGING_INTELLIGENCE_PIVOT.md).

The program defines emerging intelligence narrowly: accumulated experience must cause measurable improvement in future behavior, that improvement must transfer beyond exact development episodes, and the causal contribution of learned state must survive matched controls.

### Merged critical-path stages

| Stage | Main-branch implementation | What it establishes | What it does not establish |
|---|---|---|---|
| EI-0A | PR [#155](https://github.com/toxzak-svg/starfire/pull/155), `3fd8ec8` | Versioned cognitive episodes, deterministic sealing, provenance, validation, and exact replay | Persistence, learning, or live influence |
| EI-0B | PR [#176](https://github.com/toxzak-svg/starfire/pull/176), `087ca263` | Frozen task partitions, five matched isolated arms, independent evaluation, budgets, corruption checks, and deterministic reports | Cumulative improvement or transfer learning |
| EI-0C | PR [#187](https://github.com/toxzak-svg/starfire/pull/187), `979f4798` | Append-only canonical episode history, digest chaining, corruption rejection, and byte-exact fresh-state reconstruction | Live persistence or learning-update authority |

EI-0C's final gate passed pinned formatting, full feature and probe compilation, scoped Clippy, eleven focused adversarial tests, and two byte-identical probe executions. The bounded probe's deterministic ledger root was `d1ae74313bb2e50b16a0c8a01c8e4568`.

The immediate implementation target is now **EI-0D: reversible learning-update lattice and transactional rollback**, tracked in [issue #192](https://github.com/toxzak-svg/starfire/issues/192). The full critical path remains tracked in [issue #149](https://github.com/toxzak-svg/starfire/issues/149).

During EI-0, progress is measured by independently evaluated acquired capability, not by subsystem count. Existing modules count toward the critical path only if they improve a frozen EI metric under matched controls.

## Main-branch runtime

| Capability | Status | Notes |
|---|---|---|
| Interactive CLI | **Active** | `star chat` |
| Runtime status command | **Active** | `star status` |
| HTTP API | **Active** | Chat, reason, memory, identity, cognition, metacognition, thought |
| SQLite continuity | **Active** | Identity, memories, beliefs, sessions |
| Quanot reservoir | **Integrated** | Produces project-specific dynamics and proxy metrics |
| Symbolic reasoning | **Integrated** | Used alongside neural components |
| Prediction and metacognition | **Integrated** | Multiple modules with differing maturity |
| Native CharRNN reranker | **Bundled** | Asset-gated during Docker build |
| Runtime response plans | **Active for migrated handlers** | `RuntimeResponsePlan` inside the response path |
| Persistent runtime voice | **Active by default** | Disable with `STARFIRE_RUNTIME_VOICE=0` |
| Verifier-backed improvisation | **Merged, offline-only** | Bounded candidate search with independent verification; no live text authority |
| Bounded web retrieval | **Merged, feature-gated** | Local-first URL retrieval, SearXNG adapter, and deterministic extraction; no autonomous or live-chat authority |
| Next.js web chat | **Active** | Shows memory/cognition drawers and live labels |
| Telegram webhook | **Implemented** | Requires bot token; lacks independent webhook authentication |
| Built-in authentication | **Absent** | Hosted API is a research surface, not a private multitenant service |
| EI-0A episode contracts | **Implemented, feature-gated** | Canonical sealing and replay; no runtime wiring |
| EI-0B deterministic environment | **Implemented, feature-gated** | Frozen tasks and matched controls; offline-only |
| EI-0C append-only ledger | **Implemented, feature-gated** | Canonical history and fresh-state replay; not live SQLite |
| EI-0D reversible updates | **Planned** | Tracked in #192; isolated offline state only |
| Live EI learning authority | **Absent** | Explicitly prohibited before offline evidence and separate promotion |
| Unrestricted tools/actions | **Not authorized** | No general live autonomy boundary |
| Automatic ontology promotion | **Not authorized** | Remains prohibited during EI-0 |
| Repository-wide Clippy baseline | **Legacy debt tracked** | Rust 1.96+ exposes 86 existing findings outside EI-0B/C; tracked in #183 |

## Production deployment

The backend is defined by `render.yaml` and built from `Dockerfile`. The current documented hosted API is:

```text
https://starfire-cuee.onrender.com
```

The runtime container persists state under `/data`.

The hosted surface currently lacks built-in authentication, tenant isolation, production rate limits, and an independently authenticated Telegram webhook. Security and deployment isolation remain prerequisites before shared or public live EI evaluation.

## Current response path

The codebase still contains two response-style layers.

### Runtime-owned voice

Merged into the actual `Runtime::chat` response path:

- typed `ResponseIntent`;
- `RuntimeResponsePlan` snapshots;
- persistent directness, warmth, compression, and initiative;
- explicit correction detection;
- profile file `runtime_voice_profile.json`;
- kill switch `STARFIRE_RUNTIME_VOICE=0`.

### Legacy live wrapper seam

The production feature wiring still contains the outer live API layer and F2 observer lineage. Draft PR #153 attempted to collapse this boundary while carrying unrelated changes and was closed as superseded.

The remaining response-authority cleanup should be rebuilt narrowly on current `main`: one canonical text authority, one voice state, and an observer that cannot silently become another renderer. This cleanup improves attribution but does not itself count as an intelligence gain.

## Language and voice research

| Stage | Recorded state | Runtime meaning |
|---|---|---|
| ΩV1-A | Baseline frozen and externally reproduced | Evidence baseline |
| ΩV1-B | Typed VoiceState shadow contracts passed | No original live authority |
| ΩV1-C | Semantic plan shadow contracts passed | No original live authority |
| ΩV1-D0 | Bounded separator kernel passed externally | Kernel evidence |
| ΩV1-D1 | Narrow HTTP canary passed externally | Bounded response influence |
| ΩV1-E / STLM L1 | Independent verifier passed externally | Builder/offline verifier |
| ΩV1-F1 | Original learned selector run **failed** | Failure preserved |
| ΩV1-F1R1 | Bounded remediation passed externally | Offline selector evidence |
| ΩV1-F2 | Implemented as post-response shadow | Switch defaults off |
| STLM L1-A | Verifier-backed improvisation merged | Offline wording search only |
| STLM L1-C | Current shadow implementation remains under review | No live response authority |

The F2 runtime switch is:

```text
STARFIRE_OMEGA_V1F2_SHADOW=0
```

Improved wording is not evidence of increased intelligence unless it improves a frozen task metric without semantic drift.

## Companion, relational, and developmental work

The library contains the S3-S6 companion experiment ladder, R1 relational residual bridge, H-series developmental diagnostics, H13C structural-role transfer stress, ΩG1 bounded grammar extension, ΩG2 recursive grammar composition, ΩG3 multistep abstraction reuse, ΩG4 intervention-guided abstraction selection, and related bounded research artifacts.

These modules have differing evidence quality and authority boundaries. Their presence does not mean the named capability generalizes beyond its tested task or participates in ordinary chat. Under EI-0, they receive critical-path credit only when a frozen EI evaluation identifies a causal improvement over matched controls.

## Open and superseded research branches

Pre-pivot pull requests do not automatically belong to the EI critical path.

| Pull request | Current classification |
|---|---|
| [#128](https://github.com/toxzak-svg/starfire/pull/128) ARISE-A0 | Independent bounded research; no EI credit without frozen metric advantage |
| [#139](https://github.com/toxzak-svg/starfire/pull/139) ARISE-A1 | Stacked research; no new runtime authority during EI-0 |
| [#98](https://github.com/toxzak-svg/starfire/pull/98) ΩG2-S0 observer | Optional diagnostic work; no EI credit without causal behavioral evidence |
| [#73](https://github.com/toxzak-svg/starfire/pull/73) S6-D canary | Separate companion gate; must not bypass security or EI controls |
| [#179](https://github.com/toxzak-svg/starfire/pull/179) STLM L1-C shadow | Language shadow research; not on the EI capability critical path |

The original retrieval branch #144 and mixed response-authority branch #153 are superseded and must not be merged as historical bundles. The current retrieval implementation is already on `main` through PR #172.

## Current user-facing quality

Starfire's runtime is real, stateful, and inspectable, but its broad conversational fluency remains substantially below frontier hosted LLMs. Its strongest distinction is the explicit architecture and evidence discipline around memory, state, response authority, and experimental promotion.

Verifier-backed improvisation provides a bounded offline mechanism for wording variation without semantic drift. It does not yet make ordinary chat broadly human-fluent, and style gains must not be counted as intelligence gains.

## Highest-leverage work

1. implement EI-0D's bounded reversible update lattice and harmful-update rollback;
2. secure and isolate HTTP, Telegram, file, command, and user-state boundaries;
3. simplify the live response path so behavioral attribution is possible;
4. freeze EI-0E hypotheses, fixtures, seeds, budgets, thresholds, controls, and source hashes before the terminal experiment;
5. eliminate repository-wide Rust 1.96+ Clippy debt under issue #183 without mixing maintenance into EI evidence;
6. continue fluent realization work only under typed verification and separate metrics.

## Immediate engineering decision

Implement **EI-0D** as a default-off offline module containing:

- a fixed, typed update lattice for EI-0B task state;
- expected pre-state and post-state digests;
- independent admissibility evaluation;
- atomic apply and exact rollback;
- duplicate, stale, cross-namespace, malformed, and over-budget rejection;
- deliberately harmful but structurally valid update fixtures;
- regression and transfer checks that force rollback when damage exceeds the frozen bound;
- deterministic transaction records and an exact-replay probe.

EI-0D must not write to live SQLite, alter `Runtime::chat()`, promote ontology, grant tools or autonomy, or claim EI-0 PASS.

## Tracking sources

- [EI-0 master tracker](https://github.com/toxzak-svg/starfire/issues/149)
- [EI-0D implementation issue](https://github.com/toxzak-svg/starfire/issues/192)
- [EI-0A contract record](experiments/EI_0A_EPISODE_CONTRACTS.md)
- [EI-0B environment record](experiments/EI_0B_DETERMINISTIC_ENVIRONMENT.md)
- [EI-0C ledger record](experiments/EI_0C_APPEND_ONLY_LEDGER.md)
- [Emerging Intelligence pivot](../plans/EMERGING_INTELLIGENCE_PIVOT.md)
- [Documentation index](README.md)
- [Project README](../README.md)
- [Specification](../SPEC.md)
- [Architecture](architecture.md)
- [API reference](api.md)
- [Deployment](deployment.md)
- [Experiment index](experiments/README.md)
