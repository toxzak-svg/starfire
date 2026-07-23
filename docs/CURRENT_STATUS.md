# Starfire Current Status

> **Snapshot date:** 2026-07-21  
> **Branch covered:** `main`  
> **Purpose:** authoritative present-tense summary

This document answers one question: **what is actually true of Starfire right now?**

Plans describe intended work. Preregistrations describe frozen hypotheses. Result records describe specific runs. This page describes the current main-branch system without promoting draft work or rewriting historical evidence.

## Executive summary

Starfire is an active Rust research system with a functioning local runtime, persistent state, HTTP API, Next.js client, trained reranker, typed response machinery, and a substantial suite of bounded cognitive experiments.

It can chat, remember, expose cognitive state, run explicit reasoning and thought endpoints, and alter parts of its response style through a persistent runtime voice profile. It is deployed as a Docker service on Render and connected to a web interface.

It is not currently AGI, a frontier-quality language model, an unrestricted autonomous agent, or validated evidence of consciousness.

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
  --features starfire-observer
```

The runtime container persists state under `/data`.

## Current response path

`Runtime::chat` is the sole production text authority. The `star` binary starts
`star::api::start` directly; it does not start an HTTP wrapper or forward requests
through a second server.

### Runtime-owned voice

The actual `Runtime::chat` response path includes:

- typed `ResponseIntent`;
- `RuntimeResponsePlan` snapshots;
- persistent directness, warmth, compression, and initiative;
- explicit correction detection;
- profile file `runtime_voice_profile.json`;
- kill switch `STARFIRE_RUNTIME_VOICE=0`.

This is active by default outside tests.

### Optional F2 observer

The `starfire-observer` executable feature enables the library-owned
`omega-v1-f2-shadow` observer. When both that feature and its runtime switch are
enabled, it receives an F2 event only after finalized `/chat` response JSON has
been produced. It is shadow-only: it has no HTTP server, no response transform,
and no ability to alter returned bytes.

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

These are research artifacts. Their presence does not mean they are wired into ordinary chat or that their named capability has generalized beyond the tested task.

## Work not on main

### ARISE-A0

The ARISE-A0 reverse-obligation edge bridge exists in draft pull request `#128` on an experiment branch. It has external Vercel compilation/test evidence but remains draft and unmerged as of this snapshot.

Therefore:

- it is not part of `main`;
- it is not part of the production Docker image;
- it must not be listed as a shipped Starfire capability;
- its evidence should be evaluated within the authority boundary documented in that pull request.

## Current user-facing quality

Starfire’s runtime is real, stateful, and inspectable, but its broad conversational fluency remains substantially below frontier hosted LLMs. Its strongest distinction is the explicit architecture and evidence discipline around memory, state, response authority, and experimental promotion.

The highest-leverage product work is not adding more named modules. It is:

1. improving fluent generation without discarding typed plans and verification;
2. making state and trace inspection understandable in the UI;
3. separating private single-user deployment from public demo deployment;
4. running held-out behavioral evaluations that measure user-visible improvement.

## Known documentation policy

During this refresh:

- living docs were rewritten to match current code;
- stale Railway instructions were removed from authoritative guides;
- historical preregistrations and result records were preserved;
- old files may still contain now-outdated language when that language is part of the historical record.

Use the [documentation index](README.md) to distinguish living documents from evidence records.

## Next engineering decisions

The direct response boundary is now established. The next architectural work is
to improve fluent generation while preserving typed plans, deterministic rendering,
and independent verification.

## Source links

- [Project README](../README.md)
- [Specification](../SPEC.md)
- [Architecture](architecture.md)
- [API reference](api.md)
- [Deployment](deployment.md)
- [Experiment index](experiments/README.md)
- [Plan index](../plans/README.md)
