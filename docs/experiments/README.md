# Starfire Experiment Index

> **Index snapshot:** 2026-07-23

This directory contains preregistrations, implementation records, result records, external evaluations, blockers, and bounded design notes. It is an evidence archive, not a flat list of shipped features.

## How to read an experiment

Before citing a result, identify:

1. the exact experiment identifier;
2. the frozen hypothesis and thresholds;
3. whether execution reached the evaluator;
4. the result classification;
5. the authority boundary;
6. the promotion rule;
7. whether a later remediation has a different identifier.

A merged implementation is not automatically a scientific PASS. A green workflow supports only the gates that workflow actually verifies.

## Emerging Intelligence critical path

The program asks whether independently scored prior experience causes held-out improvement beyond matched controls and survives transfer.

```text
EI-0A canonical episodes
  → EI-0B deterministic environment and matched controls
  → EI-0C append-only history and fresh-state replay
  → EI-0D reversible fixed-schema updates and rollback
  → EI-0E frozen terminal preregistration
  → EI-0F original exact-source execution: FAIL, preserved
  → EI-0F digest remediation preflight
  → EI-0F R1B freeze: invalid schema binding, unexecuted
  → EI-0F R2 schema-bound freeze
  → EI-0F R2 exact single execution: PASS
  → EI-0G read-only shadow preregistration
```

| Stage | Record | Current interpretation |
|---|---|---|
| EI-0A | [`EI_0A_EPISODE_CONTRACTS.md`](EI_0A_EPISODE_CONTRACTS.md) | Canonical episode infrastructure; no learning authority |
| EI-0B | [`EI_0B_DETERMINISTIC_ENVIRONMENT.md`](EI_0B_DETERMINISTIC_ENVIRONMENT.md) | Frozen tasks, partitions, evaluators and matched arms |
| EI-0C | [`EI_0C_APPEND_ONLY_LEDGER.md`](EI_0C_APPEND_ONLY_LEDGER.md) | Append-only canonical history and exact fresh-state replay |
| EI-0D implementation | [`EI_0D_REVERSIBLE_UPDATES.md`](EI_0D_REVERSIBLE_UPDATES.md) | Offline provenance-bound updates and exact rollback |
| EI-0D result | [`EI_0D_RESULT.md`](EI_0D_RESULT.md) | **PASS for bounded infrastructure only** |
| EI-0E preregistration | [`EI_0E_TERMINAL_PREREGISTRATION.md`](EI_0E_TERMINAL_PREREGISTRATION.md) | Frozen original terminal experiment under `ei-0e-terminal-v1` |
| EI-0E manifest | [`EI_0E_TERMINAL_PREREGISTRATION.json`](EI_0E_TERMINAL_PREREGISTRATION.json) | Original source, seeds, arms, budgets, thresholds and classifier |
| EI-0E freeze lock | [`EI_0E_FREEZE_LOCK.json`](EI_0E_FREEZE_LOCK.json) | Exact Git-blob binding for the original package |
| EI-0F original result | [`EI_0F_TERMINAL_RESULT.md`](EI_0F_TERMINAL_RESULT.md) | **FAIL**, preserved; rejected proposal digest before arm evaluation |
| EI-0F original report | [`EI_0F_TERMINAL_REPORT.json`](EI_0F_TERMINAL_REPORT.json) | Schema-valid fail-closed crash record |
| EI-0F original classification | [`EI_0F_TERMINAL_CLASSIFICATION.json`](EI_0F_TERMINAL_CLASSIFICATION.json) | Frozen classifier returned `FAIL` |
| EI-0F digest remediation | [`EI_0F_R1_DIGEST_REMEDIATION_PREREGISTRATION.md`](EI_0F_R1_DIGEST_REMEDIATION_PREREGISTRATION.md) | Five-arm canonical digest preflight |
| EI-0F R1B manifest | [`EI_0F_R1_TERMINAL_PREREGISTRATION.json`](EI_0F_R1_TERMINAL_PREREGISTRATION.json) | Preserved invalid and unexecuted because its reused report schema named the original ID |
| EI-0F R2 manifest | [`EI_0F_R2_TERMINAL_PREREGISTRATION.json`](EI_0F_R2_TERMINAL_PREREGISTRATION.json) | Authoritative schema-bound remediation under `ei-0f-remediation-v2` |
| EI-0F R2 schema | [`EI_0F_R2_TERMINAL_REPORT.schema.json`](EI_0F_R2_TERMINAL_REPORT.schema.json) | V2 report schema with exact matching preregistration identity |
| EI-0F R2 result | [`EI_0F_R2_RESULT_2026-07-23.md`](EI_0F_R2_RESULT_2026-07-23.md) | **PASS for the frozen bounded EI-0 claim** |
| EI-0F R2 report | [`EI_0F_R2_TERMINAL_REPORT.json`](EI_0F_R2_TERMINAL_REPORT.json) | Five complete matched arms, transfer, causal chain and rollback evidence |
| EI-0F R2 classification | [`EI_0F_R2_TERMINAL_CLASSIFICATION.json`](EI_0F_R2_TERMINAL_CLASSIFICATION.json) | Frozen classifier returned `PASS` with no failed conditions |
| EI-0F R2 evidence | [`EI_0F_R2_TERMINAL_EVIDENCE.json`](EI_0F_R2_TERMINAL_EVIDENCE.json) | Single-run identity, raw and canonical digests, no-rerun proof |

### Authoritative PASS identity

- preregistration ID: `ei-0f-remediation-v2`;
- freeze merge: `16ca9717ee4514ccc4bc25e92a95c95be38824a7`;
- manifest SHA-256: `89909b52cadd394207bafc7526e992a3c20ca0a923e35c2bea7290a306eefec5`;
- execution commit: `133b82ba6d4fe14e5a5965e45cf2658845d533f1`;
- workflow run: `30036385291`;
- result merge: `13c1852724a16a9d22177b8858d35760d2432214`;
- execution count: one;
- second execution: false;
- classification: `PASS`.

The learning arm scored 10,000 basis points on all six partitions. Every matched control scored 3,500 on within-family holdout and 3,875 on the remaining partitions. The run preserved two accepted learning updates, one complete causal chain, one detected harmful challenge with exact rollback, zero replay mismatches, zero missing evaluations, zero invalid records, equal budgets, source match, closed authority, and empty stderr.

This PASS supports the bounded experimental claim only. It does not establish AGI, consciousness, general open-ended self-improvement, safe production learning, or permission for live influence.

Authoritative tracking: [EI-0 master issue #149](https://github.com/toxzak-svg/starfire/issues/149), [R2 execution issue #220](https://github.com/toxzak-svg/starfire/issues/220), [PASS PR #221](https://github.com/toxzak-svg/starfire/pull/221), and [EI-0G preregistration issue #222](https://github.com/toxzak-svg/starfire/issues/222).

## ΩV1 cognitive-to-voice and STLM

The ΩV1/STLM work separates semantic authorization, deterministic realization, independent reconstruction, learned selection, and shadow observation.

| Stage | Primary record | Interpretation |
|---|---|---|
| ΩV1-A | [`OMEGAV1A_VOICE_BASELINE.md`](OMEGAV1A_VOICE_BASELINE.md) | Frozen baseline |
| ΩV1-A verification | [`OMEGAV1A_RENDER_VERIFICATION.md`](OMEGAV1A_RENDER_VERIFICATION.md) | External reproduction |
| ΩV1-B | [`OMEGAV1B_VOICE_STATE_SHADOW.md`](OMEGAV1B_VOICE_STATE_SHADOW.md) | Typed voice state in shadow |
| ΩV1-C | [`OMEGAV1C_SEMANTIC_PLAN_SHADOW.md`](OMEGAV1C_SEMANTIC_PLAN_SHADOW.md) | Typed semantic-plan shadow |
| ΩV1-D0 | [`OMEGAV1D_BOUNDED_LIVE_BRIDGE.md`](OMEGAV1D_BOUNDED_LIVE_BRIDGE.md) | Bounded separator kernel |
| ΩV1-D1 | [`OMEGAV1D1_HTTP_CANARY.md`](OMEGAV1D1_HTTP_CANARY.md) | Narrow HTTP response-boundary canary |
| ΩV1-E | [`OMEGAV1E_INDEPENDENT_LANGUAGE_VERIFIER.md`](OMEGAV1E_INDEPENDENT_LANGUAGE_VERIFIER.md) | Independent semantic verifier |
| ΩV1-F0 | [`OMEGAV1F0_LEARNED_EXPRESSION_RENDERER_PREREGISTRATION.md`](OMEGAV1F0_LEARNED_EXPRESSION_RENDERER_PREREGISTRATION.md) | Learned selector preregistration |
| ΩV1-F1 | [`OMEGAV1F1_EXTERNAL_FAIL_2026-07-20.md`](OMEGAV1F1_EXTERNAL_FAIL_2026-07-20.md) | **FAIL**, preserved |
| ΩV1-F1R1 | [`OMEGAV1F1R1_EXTERNAL_PASS_2026-07-20.md`](OMEGAV1F1R1_EXTERNAL_PASS_2026-07-20.md) | Separate bounded remediation PASS |
| ΩV1-F2 | [`OMEGAV1F2_IMPLEMENTATION_STATUS.md`](OMEGAV1F2_IMPLEMENTATION_STATUS.md) | Post-response shadow boundary; not final evidence |

Language fluency or style improvement is not an intelligence gain unless it improves a frozen task metric without semantic drift.

## ARISE bounded reconstruction

ARISE-A0 and ARISE-A1 are merged, default-off, shadow-bounded research. They provide typed reverse-obligation planning and independent semantic reconstruction without live response or action authority.

- A0 record: [`ARISE_A0_EDGE_BRIDGE.md`](ARISE_A0_EDGE_BRIDGE.md)
- A0 merge: `24e7ce03`
- A1 merge: `ad03f7d6`

ARISE remains independent research. It receives no EI critical-path credit unless a frozen experiment attributes a causal held-out advantage to it.

## Companion interaction ladder

```text
S3 observer
  → S4 prediction ledger
  → S5-A shadow policy
  → S5-B independent outcomes
  → S5-C comparative evaluation
  → S6-A bounded live policy
  → S6-B adversarial stress
  → S6-C real-interaction canary evidence
```

Representative result: [`S6A_BOUNDED_LIVE_POLICY_RESULT.md`](S6A_BOUNDED_LIVE_POLICY_RESULT.md).

These stages do not collectively prove a finished companion product. Read each authority matrix independently.

## Developmental, residual, relational and grammar research

- H5 plan: [`../plans/H5_RESIDUAL_IDENTITY_DIAGNOSTIC_PLAN.md`](../plans/H5_RESIDUAL_IDENTITY_DIAGNOSTIC_PLAN.md)
- H5 experiment: [`H5_RESIDUAL_IDENTITY_DIAGNOSTIC.md`](H5_RESIDUAL_IDENTITY_DIAGNOSTIC.md)
- H6: [`H6_DISAGREEMENT_MODE_ACCRETION.md`](H6_DISAGREEMENT_MODE_ACCRETION.md)
- H9: [`H9_EXECUTABLE_COMMITMENT_STATE.md`](H9_EXECUTABLE_COMMITMENT_STATE.md)
- H12: [`../research/R0A_H12_LATENT_ROLE_SUBSTRATE_CONSOLIDATION.md`](../research/R0A_H12_LATENT_ROLE_SUBSTRATE_CONSOLIDATION.md)
- Endogenous state: [`OMEGA1_ENDOGENOUS_STATE_SPACE_GENESIS.md`](OMEGA1_ENDOGENOUS_STATE_SPACE_GENESIS.md)
- Bounded autonomous kernel: [`A1_BOUNDED_AUTONOMOUS_KERNEL.md`](A1_BOUNDED_AUTONOMOUS_KERNEL.md)
- IngExuity integration: [`../INGEXUITY_STARFIRE_INTEGRATION.md`](../INGEXUITY_STARFIRE_INTEGRATION.md)

Executable grammar and abstraction probes include ΩG1 through ΩG4. Their names identify tested mechanisms, not automatically generalized capabilities.

## Result classifications

| Classification | Meaning |
|---|---|
| `PASS` | Every frozen required gate passed within the stated boundary |
| `FAIL` | At least one frozen required gate failed |
| `COLLECTING` | Implementation is active but the frozen sample is incomplete |
| `BLOCKED` | A prerequisite prevents execution |
| `NO VERDICT` | Infrastructure failed before the evaluator produced a result unless the preregistration classifies that failure as FAIL |

For EI-0 terminal experiments, crashes, source mismatch, corruption, missing data, nondeterminism, budget mismatch, or threshold ambiguity are literal FAIL conditions.

## Evidence preservation policy

1. Never rename a FAIL into a PASS.
2. Never edit thresholds after seeing a result without a separately identified remediation.
3. Never describe a shadow stage as live influence.
4. Never describe a live-text canary as action authority.
5. Preserve exact source and environment identity where available.
6. Record infrastructure failures separately unless a preregistration explicitly classifies them as FAIL.
7. Keep interpretation narrower than the experiment name when necessary.
8. Separate authority, capability, and evidence claims.

## Related documents

- [Current status](../CURRENT_STATUS.md)
- [Documentation index](../README.md)
- [Architecture](../architecture.md)
- [Plan index](../../plans/README.md)
