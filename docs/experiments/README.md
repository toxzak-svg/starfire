# Starfire Experiment Index

> **Index snapshot:** 2026-07-23

This directory contains preregistrations, implementation records, result templates, external result records, blockers, and bounded design notes. It is an evidence archive, not a flat list of shipped features.

## How to read an experiment

Before citing a result, identify:

1. the exact experiment identifier;
2. the frozen hypothesis and thresholds;
3. whether execution actually reached the evaluator;
4. the result classification;
5. the authority boundary;
6. the promotion rule;
7. whether a later remediation has a different identifier.

A merged implementation is not automatically a scientific PASS. A green workflow supports only the gates that workflow actually verifies.

## Emerging Intelligence critical path

The active program asks whether independently scored prior experience causes held-out improvement beyond matched controls and survives transfer.

```text
EI-0A canonical episodes
  → EI-0B deterministic environment and matched controls
  → EI-0C append-only history and fresh-state replay
  → EI-0D reversible fixed-schema updates and rollback
  → EI-0E frozen terminal preregistration
  → EI-0F exact-source terminal experiment
```

| Stage | Record | Current interpretation |
|---|---|---|
| EI-0A | [`EI_0A_EPISODE_CONTRACTS.md`](EI_0A_EPISODE_CONTRACTS.md) | Canonical episode infrastructure; no learning authority |
| EI-0B | [`EI_0B_DETERMINISTIC_ENVIRONMENT.md`](EI_0B_DETERMINISTIC_ENVIRONMENT.md) | Frozen tasks, partitions, evaluators and matched arms |
| EI-0C | [`EI_0C_APPEND_ONLY_LEDGER.md`](EI_0C_APPEND_ONLY_LEDGER.md) | Append-only canonical history and exact fresh-state replay |
| EI-0D implementation | [`EI_0D_REVERSIBLE_UPDATES.md`](EI_0D_REVERSIBLE_UPDATES.md) | Offline provenance-bound updates and exact rollback |
| EI-0D result | [`EI_0D_RESULT.md`](EI_0D_RESULT.md) | **PASS for bounded infrastructure only**; not evidence of cumulative improvement |
| EI-0E preregistration | [`EI_0E_TERMINAL_PREREGISTRATION.md`](EI_0E_TERMINAL_PREREGISTRATION.md) | **Frozen and merged** under `ei-0e-terminal-v1`; no terminal result yet |
| EI-0E manifest | [`EI_0E_TERMINAL_PREREGISTRATION.json`](EI_0E_TERMINAL_PREREGISTRATION.json) | Canonical hypotheses, source, seeds, arms, budgets, thresholds and classifier rules |
| EI-0E freeze lock | [`EI_0E_FREEZE_LOCK.json`](EI_0E_FREEZE_LOCK.json) | Exact Git-blob binding for the frozen preregistration package |
| EI-0F report schema | [`EI_0F_TERMINAL_REPORT.schema.json`](EI_0F_TERMINAL_REPORT.schema.json) | Canonical result format for the unexecuted terminal experiment |

EI-0E merged in PR [#196](https://github.com/toxzak-svg/starfire/pull/196) at `2da7eeed`. Its manifest SHA-256 is `5b83b27e5c218b6af2c53409d60fa6bf285adcde7ccb05b42505a5d0da290d73`. The next stage is exact-source execution under [issue #200](https://github.com/toxzak-svg/starfire/issues/200).

Authoritative tracking: [EI-0 master issue #149](https://github.com/toxzak-svg/starfire/issues/149).

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
| ΩV1-F2 | [`OMEGAV1F2_IMPLEMENTATION_STATUS.md`](OMEGAV1F2_IMPLEMENTATION_STATUS.md) | Post-response shadow boundary; not final collection evidence |

STLM records:

- [`STLM_L0_SEMANTIC_PROGRAM.md`](STLM_L0_SEMANTIC_PROGRAM.md)
- [`STLM_L1_INDEPENDENT_LANGUAGE_VERIFIER.md`](STLM_L1_INDEPENDENT_LANGUAGE_VERIFIER.md)
- [`../architecture/STATE_TRANSITION_LANGUAGE_MODEL.md`](../architecture/STATE_TRANSITION_LANGUAGE_MODEL.md)
- [`../../plans/STATE_TRANSITION_LANGUAGE_MODEL_PROGRAM.md`](../../plans/STATE_TRANSITION_LANGUAGE_MODEL_PROGRAM.md)

Language fluency or style improvement is not an intelligence gain unless it improves a frozen task metric without semantic drift.

## ARISE bounded reconstruction

ARISE-A0 and ARISE-A1 are merged, default-off, shadow-bounded research. They provide typed reverse-obligation planning and independent semantic reconstruction without live response or action authority.

- A0 record: [`ARISE_A0_EDGE_BRIDGE.md`](ARISE_A0_EDGE_BRIDGE.md)
- A0 merge: `24e7ce03`
- A1 merge: `ad03f7d6`

ARISE remains independent research during EI-0. It receives no EI critical-path credit unless the frozen terminal experiment attributes a causal held-out advantage to it.

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
| `NO VERDICT` | Infrastructure failed before the evaluator produced a result |

For EI-0F, the frozen classifier permits only `PASS` or `FAIL`. Crashes, source mismatch, corruption, missing data, nondeterminism, budget mismatch, or threshold ambiguity are FAIL conditions rather than an discretionary inconclusive category.

## Evidence preservation policy

1. Never rename a FAIL into a PASS.
2. Never edit thresholds after seeing a result without a separately identified remediation.
3. Never describe a shadow stage as live influence.
4. Never describe a live-text canary as action authority.
5. Preserve exact source and environment identity where available.
6. Record infrastructure failures separately from scientific failures unless a preregistration explicitly classifies them as FAIL.
7. Keep interpretation narrower than the experiment name when necessary.
8. Separate authority, capability, and evidence claims.

## Related documents

- [Current status](../CURRENT_STATUS.md)
- [Documentation index](../README.md)
- [Architecture](../architecture.md)
- [Plan index](../../plans/README.md)
