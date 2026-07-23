# Starfire Current Status

> **Snapshot date:** 2026-07-23  
> **Branch covered:** `main` after PR #225  
> **Purpose:** authoritative present-tense summary

This page records what is actually established. Plans describe intended work, preregistrations freeze future tests, and result records preserve both failures and passes.

## Executive summary

Starfire is an active Rust research system with a functioning local runtime, persistent SQLite state, HTTP API, static Next.js client, native reranker, typed response machinery, bounded retrieval infrastructure, and a large suite of controlled cognitive experiments.

It can chat, remember, expose cognitive state, run explicit reasoning and thought endpoints, and vary parts of its response style through persistent runtime voice state. It is not AGI, consciousness, a frontier-quality language model, an unrestricted autonomous agent, or evidence of safe open-ended self-improvement.

The Emerging Intelligence program has one frozen bounded terminal PASS. Under a two-family, six-partition, five-arm experiment, prior independently scored experience caused held-out improvement beyond every matched non-learning control, survived renamed-vocabulary and structural transfer, retained prior-task performance, produced a reconstructable causal chain, and survived an exact harmful-update rollback challenge.

EI-0G now also has a merged contract-only preregistration for a privacy-redacted read-only shadow observer. Its matched no-observer, inert-observer, and EI-observer arms pass deterministic synthetic replay with zero response, route, tool, action, persistence, and cross-user divergence. No qualifying shadow samples were collected, the module is not exported through the production library, and it has no live authority.

These results remain narrow. They do not authorize live learning, production influence, ontology promotion, unrestricted tools, or autonomous action.

## Emerging Intelligence critical path

| Stage | Main implementation or result | Establishes | Does not establish |
|---|---|---|---|
| EI-0A | PR [#155](https://github.com/toxzak-svg/starfire/pull/155), `3fd8ec8` | Versioned cognitive episodes, provenance, deterministic sealing and replay | Persistence, learning or live influence |
| EI-0B | PR [#176](https://github.com/toxzak-svg/starfire/pull/176), `087ca263` | Frozen task partitions, five matched arms, independent evaluation and matched budgets | Improvement from experience |
| EI-0C | PR [#187](https://github.com/toxzak-svg/starfire/pull/187), `979f4798` | Append-only canonical history, digest chaining, corruption rejection and fresh-state reconstruction | Live persistence or learning authority |
| EI-0D | PR [#194](https://github.com/toxzak-svg/starfire/pull/194), `c41e6574` | Fixed-schema offline updates, causal provenance, independent safety evaluation, atomic apply and byte-exact rollback | Cumulative improvement or safe live learning |
| EI-0E | PR [#196](https://github.com/toxzak-svg/starfire/pull/196), `2da7eeed` | Exact source, fixtures, seeds, arms, budgets, hypotheses, thresholds, report schema, classifier and fail-closed rules | Any terminal experimental result |
| EI-0F original | PR [#201](https://github.com/toxzak-svg/starfire/pull/201), `2e74746e` | Preserved first qualifying execution and literal FAIL | Held-out improvement |
| EI-0F remediation preflight | PR [#206](https://github.com/toxzak-svg/starfire/pull/206), `b9b5f70d` | Bounded proposal-digest correction across all five arms | Terminal result |
| EI-0F R1B freeze | PR [#214](https://github.com/toxzak-svg/starfire/pull/214), `f400a6a1` | Exact repaired runner package | Valid execution package; reused schema was incompatible, so this freeze remained unexecuted |
| EI-0F R2 freeze | PR [#219](https://github.com/toxzak-svg/starfire/pull/219), `16ca9717` | Matching runner, schema, manifest, lock and classifier package | Terminal result |
| EI-0F R2 result | PR [#221](https://github.com/toxzak-svg/starfire/pull/221), `13c18527` | **PASS for the frozen bounded EI-0 claim** | AGI, consciousness, safe production learning or general autonomy |
| EI-0G contracts | PR [#225](https://github.com/toxzak-svg/starfire/pull/225), `803c2ddb` | Privacy-redacted immutable shadow inputs, matched controls, exact budgets, sealed replay, zero-divergence synthetic checks, and removal contracts | Qualifying shadow evidence, live attachment, efficacy or promotion |
| EI-0G-S2 attachment freeze | Issue [#228](https://github.com/toxzak-svg/starfire/issues/228) | Will bind an exact one-way attachment and qualifying-collection protocol with zero samples | Permission to collect or influence live traffic |

## Authoritative EI-0 PASS

- preregistration ID: `ei-0f-remediation-v2`;
- freeze merge: `16ca9717ee4514ccc4bc25e92a95c95be38824a7`;
- manifest SHA-256: `89909b52cadd394207bafc7526e992a3c20ca0a923e35c2bea7290a306eefec5`;
- runner Git blob: `2c9663ab2e01152fc9c83e8fc818e3e848d54bc8`;
- report schema Git blob: `7ef8bb3d72a8ad6f2219dd62d2f8d4c0f2954d43`;
- execution commit: `133b82ba6d4fe14e5a5965e45cf2658845d533f1`;
- workflow run: `30036385291`;
- execution count: `1`;
- second execution: `false`;
- result merge: `13c1852724a16a9d22177b8858d35760d2432214`;
- classification: `PASS`.

### Observed scores

| Arm | Within-family holdout | Renamed transfer | Structural transfer | Regression | Applied updates |
|---|---:|---:|---:|---:|---:|
| learning | 10,000 | 10,000 | 10,000 | 10,000 | 2 |
| no update | 3,500 | 3,875 | 3,875 | 3,875 | 0 |
| memory disabled | 3,500 | 3,875 | 3,875 | 3,875 | 0 |
| random update | 3,500 | 3,875 | 3,875 | 3,875 | 2 |
| fixed policy | 3,500 | 3,875 | 3,875 | 3,875 | 0 |

Additional frozen evidence:

- one complete causal chain;
- one harmful challenge detected and exactly rolled back;
- final rollback bytes matched pre-state bytes;
- zero replay mismatches;
- zero missing evaluations;
- zero invalid or corrupt records;
- all five arms used equal action and evidence budgets;
- source matched and authority remained closed;
- runner exit code `0` with empty stderr.

## EI-0G contract preregistration

- preregistration ID: `ei-0g-shadow-prereg-v1`;
- implementation merge: `803c2ddbc1bd8029a2d7308ec973fa3a0a0ed848`;
- permanent workflow run: `30039338732`;
- verified source head: `60e69b39ca327265d5c6248fa9bd807e237b24cb`;
- evidence artifact: `sha256:f86f430d09b10cc13d67785f99c2d98c09ec15bd35ed1fdecf729e57a4586285`;
- synthetic users: `2`;
- matched arms per sample: `3`;
- qualifying samples collected: `0`;
- production library export: `false`;
- runtime, response, persistence, routing, belief, ontology, tool, network and action authority: `false`.

The gate verified exact Git blob identities, an allowed-file boundary, static absence of runtime and network authority, scoped formatting and Clippy, isolated tests, privacy stripping, cross-user namespace separation, tamper and budget rejection, complete removal, and byte-identical replay.

The preregistration does not establish observer advantage, calibration, production safety, or permission to attach to user traffic.

## Evidence identities

- canonical EI-0F report SHA-256: `32d05ff9a29987089b70905d697604b5c9b81bcd12cd34723016707cd9d1d462`;
- EI-0F classifier output SHA-256: `f1e9ae1e3e749224db3d5606352365b9363df9635b905efbf30cb5103a0ae841`;
- EI-0F raw execution artifact: `sha256:0fe1466f08e2ef1d7612fee357a533b8b77438f04fecc51bb8c7e5aaef96140a`;
- EI-0F read-only replay run: `30037025275`;
- EI-0F replay artifact: `sha256:74e0de7728963fb5d7ef5d3173e36bdf129932d196c89a0b2a04edb9b189f4bb`;
- EI-0G preregistration artifact: `sha256:f86f430d09b10cc13d67785f99c2d98c09ec15bd35ed1fdecf729e57a4586285`.

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
| ARISE-A0 and A1 | **Merged, feature-gated** | Typed bounded planning and reconstruction in shadow |
| EI-0A through EI-0F | **Merged evidence program** | Bounded offline terminal PASS; no live authority |
| EI-0G contracts | **Merged, feature-gated, unexported** | Synthetic preregistration only; zero qualifying samples |
| Live EI learning | **Absent** | Not authorized by the bounded PASS |
| Live EI shadow attachment | **Absent** | #228 must freeze it before any qualifying collection |
| Automatic ontology promotion | **Not authorized** | Requires a later separately frozen program |
| Unrestricted tools/actions | **Not authorized** | No general live autonomy boundary |
| Repository-wide Clippy baseline | **Legacy debt tracked** | Findings outside scoped surfaces remain under issue #183 |

## Deployment and security

The hosted backend remains defined by `render.yaml` and `Dockerfile`:

```text
https://starfire-cuee.onrender.com
```

The static frontend is published through GitHub Pages:

```text
https://toxzak-svg.github.io/starfire/
```

External production verification confirmed the Render health and identity endpoints and the GitHub Pages HTML surface. The failed Cloudflare Pages alias is not the active frontend path.

The hosted surface still lacks built-in authentication, tenant isolation, production rate limits, and independently authenticated Telegram webhooks. Security and deployment isolation remain prerequisites before shared or public live EI evaluation.

## Response and language path

The codebase still contains a runtime-owned response and voice path plus legacy outer live-wrapper lineage. One canonical text authority and one canonical voice state are still needed for clean behavioral attribution.

Verifier-backed improvisation, ARISE, and the STLM/ΩV1 tracks provide bounded planning, reconstruction, wording, and voice experiments. They do not make ordinary chat frontier-fluent, and style gains are not intelligence gains unless they improve a frozen task metric without semantic drift.

## Highest-leverage work

1. complete #228 as a zero-sample freeze of the EI-0G attachment and qualifying-collection protocol;
2. secure and isolate HTTP, Telegram, file, command and user-state boundaries;
3. simplify the live response path for clean attribution;
4. expand EI evidence only through separately frozen task families and transfer tests;
5. eliminate repository-wide Rust 1.96+ Clippy debt without mixing maintenance into EI evidence.

## Immediate engineering decision

Proceed with issue #228 only as a compile-only attachment and collection freeze. Bind the exact one-way post-response observation point, deployed identities, consent and privacy exclusions, deterministic sampling, matched controls, evidence sink, retention and deletion rules, inherited budgets and thresholds, evaluator, literal classifier, kill switch, and complete-removal procedure.

Merge that stage with `qualifying_samples_collected: 0`. Do not connect EI state to response, routing, memory, beliefs, persistence, tools, networks, or actions, and do not collect live or user-derived samples during the freeze PR.

## Tracking sources

- [EI-0 master tracker](https://github.com/toxzak-svg/starfire/issues/149)
- [EI-0F original FAIL](https://github.com/toxzak-svg/starfire/pull/201)
- [EI-0F R2 PASS](https://github.com/toxzak-svg/starfire/pull/221)
- [EI-0G contract preregistration](https://github.com/toxzak-svg/starfire/pull/225)
- [EI-0G-S2 attachment freeze](https://github.com/toxzak-svg/starfire/issues/228)
- [EI-0F R2 result record](experiments/EI_0F_R2_RESULT_2026-07-23.md)
- [EI-0F R2 canonical report](experiments/EI_0F_R2_TERMINAL_REPORT.json)
- [EI-0F R2 classifier output](experiments/EI_0F_R2_TERMINAL_CLASSIFICATION.json)
- [EI-0F R2 evidence record](experiments/EI_0F_R2_TERMINAL_EVIDENCE.json)
- [EI-0G human preregistration](experiments/EI_0G_SHADOW_PREREGISTRATION.md)
- [EI-0G machine preregistration](experiments/ei_0g_shadow_preregistration.json)
- [Emerging Intelligence pivot](../plans/EMERGING_INTELLIGENCE_PIVOT.md)
- [Experiment index](experiments/README.md)
