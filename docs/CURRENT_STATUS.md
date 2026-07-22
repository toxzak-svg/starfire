# Starfire Current Status

> **Snapshot date:** 2026-07-22  
> **Branch covered:** `main`  
> **Purpose:** authoritative present-tense summary

This document answers one question: **what is actually true of Starfire right now?**

Plans describe intended work. Preregistrations describe frozen hypotheses. Result records describe specific runs. This page describes the current main-branch system without promoting draft work or rewriting historical evidence.

## Executive summary

Starfire is an active Rust research system with a functioning local runtime, persistent state, HTTP API, Next.js client, trained reranker, typed response machinery, and a substantial suite of bounded cognitive experiments.

It can chat, remember, expose cognitive state, run explicit reasoning and thought endpoints, and alter parts of its response style through a persistent runtime voice profile. It is deployed as a Docker service on Render and connected to a web interface.

It is not currently AGI, an established emerging intelligence, a frontier-quality language model, an unrestricted autonomous agent, or validated evidence of consciousness.

## Active research direction: Emerging Intelligence

PR [#148](https://github.com/toxzak-svg/starfire/pull/148) merged the active [Emerging Intelligence pivot](../plans/EMERGING_INTELLIGENCE_PIVOT.md).

The program defines emerging intelligence narrowly: accumulated experience must cause measurable improvement in future behavior, that improvement must transfer beyond exact development episodes, and the causal contribution of learned state must survive matched controls.

This is a research target, not a current capability claim. No EI terminal experiment has run. The immediate implementation target is **EI-0A: Canonical episode contracts**, with no runtime wiring or learning authority. The complete critical path and pre-pivot work disposition are tracked in [issue #149](https://github.com/toxzak-svg/starfire/issues/149).

During EI-0, progress is measured by acquired and independently evaluated capability rather than subsystem count. Existing modules may contribute only when they improve a frozen EI metric under the required controls.

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
| Next.js web chat | **Active** | Shows memory/cognition drawers and live labels |
| Telegram webhook | **Implemented** | Requires bot token; lacks independent webhook authentication |
| Built-in authentication | **Absent** | Hosted API is a research surface, not a private multitenant service |
| Canonical EI episode contract | **Planned** | EI-0A is next; not implemented on this snapshot |
| Live EI learning authority | **Absent** | Explicitly prohibited before offline evidence and separate promotion |
| Unrestricted tools/actions | **Not authorized** | No general live autonomy boundary |
| Automatic ontology promotion | **Not authorized** | Remains gated pending stronger controls and transfer evidence |

## Production deployment

The backend is defined by `render.yaml` and built from `Dockerfile`.

Current hosted API:

```text
https://starfire-cuee.onrender.com
```

The Docker builder runs asset checks and the frozen ΩV1 verification chain before compiling:

```bash
cargo build --release --locked \
  -p star_bin \
  --bin star \
  --features starfire-live
```

The runtime container persists state under `/data`.

The hosted surface currently lacks built-in authentication, tenant isolation, production rate limits, and an independently authenticated Telegram webhook. Security and deployment isolation are prerequisites before shared or public live EI evaluation.

## Current response path

The codebase presently contains two response-style layers.

### 1. Runtime-owned voice

Merged into the actual `Runtime::chat` response path:

- typed `ResponseIntent`;
- `RuntimeResponsePlan` snapshots;
- persistent directness, warmth, compression, and initiative;
- explicit correction detection;
- profile file `runtime_voice_profile.json`;
- kill switch `STARFIRE_RUNTIME_VOICE=0`.

This is active by default outside tests.

### 2. Live Integration 1 wrapper

The `starfire-live` executable feature still routes startup through `src/live_api.rs`.

That wrapper:

- runs the protected API on loopback;
- processes successful `/chat` envelopes;
- maintains a separate `VoiceState`;
- adds a `live` metadata object;
- writes `live_chat_trace.jsonl`;
- exposes `/live/status`;
- fails open to the protected response.

### Known seam

Comments added during the runtime-owned voice change describe the HTTP proxy as legacy and emphasize `Runtime::chat` as text authority. The actual feature wiring still starts the wrapper in the production image. Both layers therefore remain present.

**Recommended cleanup:** split the F2 observer feature from the outer wrapper, decide which voice state is canonical, and collapse the response path to one clearly named authority boundary.

This cleanup supports EI attribution but does not itself count as an intelligence gain.

## ΩV1 cognitive-to-voice track

| Stage | Recorded state | Runtime meaning |
|---|---|---|
| ΩV1-A | Baseline frozen and externally reproduced | Evidence baseline |
| ΩV1-B | Typed VoiceState shadow contracts passed | No original live authority |
| ΩV1-C | Semantic plan shadow contracts passed | No original live authority |
| ΩV1-D0 | Bounded separator kernel passed externally | Kernel evidence |
| ΩV1-D1 | Narrow HTTP canary passed externally | Bounded response influence |
| ΩV1-E / STLM L1 | Independent verifier passed externally | Builder/offline verifier |
| ΩV1-F1 | Original learned selector run **failed** | Failure preserved |
| ΩV1-F1R1 | Bounded remediation passed externally | Offline learned selector evidence |
| ΩV1-F2 | Implemented as post-response shadow | Compiled into production, switch defaults off |

The F2 runtime switch is:

```text
STARFIRE_OMEGA_V1F2_SHADOW=0
```

Changing it to `1` begins the code path but does not retroactively classify the collection experiment as PASS.

## Companion and relational work

The library contains the S3-S6 companion experiment ladder and the R1 relational residual bridge. These features provide observation, prediction, policy proposal, independent outcomes, evaluation, bounded canary, and relational-transfer machinery under stage-specific constraints.

They should not be described as a finished emotional companion product. Most of this surface remains experiment, shadow, or evidence infrastructure.

## H, ΩG, and developmental work

Main includes executable probes and modules for:

- H-series developmental and residual diagnostics;
- H13C structural-role transfer stress;
- R1 relational residual work;
- ΩG1 bounded grammar extension;
- ΩG2 recursive grammar composition;
- ΩG3 multistep abstraction reuse;
- ΩG4 intervention-guided abstraction selection;
- endogenous state-space and bounded autonomous-kernel research.

These are research artifacts. Their presence does not mean they are wired into ordinary chat or that their named capability has generalized beyond the tested task. Under the EI program, they count toward the critical path only if a frozen EI evaluation shows that they improve later behavior over matched controls.

## Work not on main

Several open pull requests predate the EI pivot. They are not automatically part of the EI critical path and must preserve their original evidence boundaries.

| Pull request | Current classification under EI-0 |
|---|---|
| [#145](https://github.com/toxzak-svg/starfire/pull/145) Rust warning and technical-debt cleanup | Compatible maintenance; evaluate and merge on its own verification |
| [#144](https://github.com/toxzak-svg/starfire/pull/144) bounded retrieval stack | Compatible while feature-gated and non-authoritative; live or shared use requires security prerequisites |
| [#128](https://github.com/toxzak-svg/starfire/pull/128) ARISE-A0 | Independent bounded research; not an EI capability without frozen metric advantage |
| [#139](https://github.com/toxzak-svg/starfire/pull/139) ARISE-A1 | Stacked draft research; no new runtime authority during EI-0 without an explicit gate |
| [#98](https://github.com/toxzak-svg/starfire/pull/98) ΩG2-S0 observer | Optional diagnostic research; no EI credit without causal behavioral evidence |
| [#73](https://github.com/toxzak-svg/starfire/pull/73) S6-D canary | Separate companion gate; must not bypass EI security or evidence controls |

The authoritative disposition is maintained in [EI-0 tracker #149](https://github.com/toxzak-svg/starfire/issues/149). Old experimental records must not be rewritten merely to fit the new program.

## Current user-facing quality

Starfire’s runtime is real, stateful, and inspectable, but its broad conversational fluency remains substantially below frontier hosted LLMs. Its strongest distinction is the explicit architecture and evidence discipline around memory, state, response authority, and experimental promotion.

The highest-leverage work is now:

1. implementing EI-0A canonical episode contracts without runtime learning authority;
2. securing and isolating HTTP, Telegram, file, command, and user-state boundaries;
3. simplifying the live response path so behavioral attribution is possible;
4. building frozen developmental environments and matched controls;
5. improving fluent realization under typed verification without counting style gains as intelligence gains;
6. turning the experimental record into a compact, navigable scorecard.

## Known documentation policy

During this refresh:

- living docs were rewritten to match current code and the merged EI program;
- stale Railway instructions were removed from authoritative guides;
- historical preregistrations and result records were preserved;
- old files may still contain now-outdated language when that language is part of the historical record.

Use the [documentation index](README.md) to distinguish living documents from evidence records.

## Next engineering decisions

### Immediate critical-path decision

Implement **EI-0A** as a small, authority-closed code slice containing typed episode, prediction, outcome, evaluation, authority, and provenance contracts plus canonical serialization, digest, and invariant tests.

EI-0A must not alter `Runtime::chat()`, persistence authority, tool selection, response generation, ontology, or autonomous action.

### Supporting response-architecture decision

The response path still requires one canonical text authority:

#### Option A: runtime-owned response path

Make `Runtime::chat` the sole text authority, expose its typed plan and voice snapshot directly through `lib/api.rs`, and retire the outer text-transforming proxy.

#### Option B: explicit response-boundary service

Keep a separate response service, but rename it, remove duplicate voice state, and make the protected/runtime boundary intentional rather than inherited.

Maintaining both indefinitely will make behavior harder to reason about and EI experiments harder to attribute.

## Tracking sources

- [EI-0 master tracker](https://github.com/toxzak-svg/starfire/issues/149)
- [Emerging Intelligence pivot](../plans/EMERGING_INTELLIGENCE_PIVOT.md)
- [Project README](../README.md)
- [Specification](../SPEC.md)
- [Architecture](architecture.md)
- [API reference](api.md)
- [Deployment](deployment.md)
- [Experiment index](experiments/README.md)
- [Plan index](../plans/README.md)
