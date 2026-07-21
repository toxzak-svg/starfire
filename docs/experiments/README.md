# Starfire Experiment Index

> **Index snapshot:** 2026-07-21

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

A merged implementation candidate is not automatically a PASS. A successful deployment is evidence only for gates that were required before that deployment’s image could be published.

## Current headline

The most mature recent ladder is the **ΩV1 cognitive-to-voice / STLM track**:

```text
A baseline
  → B typed VoiceState shadow
  → C semantic-plan shadow
  → D0 bounded kernel
  → D1 narrow HTTP canary
  → E independent verifier
  → F1 original learned selector FAIL
  → F1R1 bounded remediation PASS
  → F2 post-response shadow implementation / collection boundary
```

The original F1 failure remains part of the record. F1R1 does not erase it.

## ΩV1 cognitive-to-voice track

| Stage | Primary record | Interpretation |
|---|---|---|
| ΩV1-A | [`OMEGAV1A_VOICE_BASELINE.md`](OMEGAV1A_VOICE_BASELINE.md) | Frozen voice baseline and metrics |
| ΩV1-A external verification | [`OMEGAV1A_RENDER_VERIFICATION.md`](OMEGAV1A_RENDER_VERIFICATION.md) | External builder reproduction |
| ΩV1-B | [`OMEGAV1B_VOICE_STATE_SHADOW.md`](OMEGAV1B_VOICE_STATE_SHADOW.md) | Typed persistent VoiceState in shadow |
| ΩV1-C | [`OMEGAV1C_SEMANTIC_PLAN_SHADOW.md`](OMEGAV1C_SEMANTIC_PLAN_SHADOW.md) | Typed complete semantic plans in matched shadow mode |
| ΩV1-D0 | [`OMEGAV1D_BOUNDED_LIVE_BRIDGE.md`](OMEGAV1D_BOUNDED_LIVE_BRIDGE.md) | Deterministic separator-only bounded kernel |
| ΩV1-D1 | [`OMEGAV1D1_HTTP_CANARY.md`](OMEGAV1D1_HTTP_CANARY.md) | Narrow successful `/chat` response-boundary canary |
| ΩV1-D1 result template | [`OMEGAV1D1_RESULT_TEMPLATE.md`](OMEGAV1D1_RESULT_TEMPLATE.md) | Frozen result schema |
| ΩV1-D1 blocker | [`OMEGAV1D1_BLOCKER.md`](OMEGAV1D1_BLOCKER.md) | Historical activation blocker |
| ΩV1-E | [`OMEGAV1E_INDEPENDENT_LANGUAGE_VERIFIER.md`](OMEGAV1E_INDEPENDENT_LANGUAGE_VERIFIER.md) | Builder-only inverse semantic verifier |
| ΩV1-E result template | [`OMEGAV1E_RESULT_TEMPLATE.md`](OMEGAV1E_RESULT_TEMPLATE.md) | Frozen result schema |
| ΩV1-F0 | [`OMEGAV1F0_LEARNED_EXPRESSION_RENDERER_PREREGISTRATION.md`](OMEGAV1F0_LEARNED_EXPRESSION_RENDERER_PREREGISTRATION.md) | Offline learned selector preregistration |
| ΩV1-F1 original result | [`OMEGAV1F1_EXTERNAL_FAIL_2026-07-20.md`](OMEGAV1F1_EXTERNAL_FAIL_2026-07-20.md) | **FAIL**, preserved |
| ΩV1-F1 result template | [`OMEGAV1F1_RESULT_TEMPLATE.md`](OMEGAV1F1_RESULT_TEMPLATE.md) | Frozen result schema |
| ΩV1-F1R1 preregistration | [`OMEGAV1F1R1_TEMPLATE_REMEDIATION_PREREGISTRATION.md`](OMEGAV1F1R1_TEMPLATE_REMEDIATION_PREREGISTRATION.md) | Separate bounded remediation |
| ΩV1-F1R1 external result | [`OMEGAV1F1R1_EXTERNAL_PASS_2026-07-20.md`](OMEGAV1F1R1_EXTERNAL_PASS_2026-07-20.md) | **PASS** within the remediation boundary |
| ΩV1-F2 preregistration | [`OMEGAV1F2_SHADOW_EVALUATION_PREREGISTRATION.md`](OMEGAV1F2_SHADOW_EVALUATION_PREREGISTRATION.md) | Post-response shadow collection contract |
| ΩV1-F2 implementation | [`OMEGAV1F2_IMPLEMENTATION_STATUS.md`](OMEGAV1F2_IMPLEMENTATION_STATUS.md) | Implemented boundary; not equivalent to final collection PASS |

Roadmap:

- [`../../plans/OMEGAV1_COGNITIVE_TO_VOICE_BRIDGE.md`](../../plans/OMEGAV1_COGNITIVE_TO_VOICE_BRIDGE.md)

## State Transition Language Model

STLM separates semantic authorization, deterministic realization, and independent reconstruction.

| Stage | Record |
|---|---|
| L0 semantic program | [`STLM_L0_SEMANTIC_PROGRAM.md`](STLM_L0_SEMANTIC_PROGRAM.md) |
| L1 independent verifier | [`STLM_L1_INDEPENDENT_LANGUAGE_VERIFIER.md`](STLM_L1_INDEPENDENT_LANGUAGE_VERIFIER.md) |
| Architecture | [`../architecture/STATE_TRANSITION_LANGUAGE_MODEL.md`](../architecture/STATE_TRANSITION_LANGUAGE_MODEL.md) |
| Program plan | [`../../plans/STATE_TRANSITION_LANGUAGE_MODEL_PROGRAM.md`](../../plans/STATE_TRANSITION_LANGUAGE_MODEL_PROGRAM.md) |

The STLM documents should be read together with the corresponding executable probes and Cargo feature requirements.

## Companion interaction ladder

The companion program progresses through increasingly demanding boundaries:

```text
S3 explicit-statement observer
  → S4 prediction ledger
  → S5-A shadow interaction policy
  → S5-B independent outcomes
  → S5-C comparative evaluation
  → S6-A bounded live policy
  → S6-B adversarial stress
  → S6-C real-interaction canary evidence
```

The feature names in `lib/Cargo.toml` are the clearest dependency map:

- `companion-observer`;
- `companion-prediction-ledger`;
- `companion-interaction-policy`;
- `companion-interaction-outcomes`;
- `companion-policy-evaluation`;
- `companion-bounded-live-policy`;
- `companion-live-policy-stress`;
- `companion-real-interaction-canary`.

Representative current record:

- [`S6A_BOUNDED_LIVE_POLICY_RESULT.md`](S6A_BOUNDED_LIVE_POLICY_RESULT.md)

These stages do not collectively prove a finished companion product. Check each authority matrix for live influence, persistence, and user-consent scope.

## Developmental and residual cognition

### H5 residual identity

- Plan: [`../plans/H5_RESIDUAL_IDENTITY_DIAGNOSTIC_PLAN.md`](../plans/H5_RESIDUAL_IDENTITY_DIAGNOSTIC_PLAN.md)
- Experiment: [`H5_RESIDUAL_IDENTITY_DIAGNOSTIC.md`](H5_RESIDUAL_IDENTITY_DIAGNOSTIC.md)

The H5 work is diagnostic. It investigates whether non-memory residual structure carries stable task-relevant identity. It does not authorize automatic concept promotion.

### H6 disagreement-mode accretion

- [`H6_DISAGREEMENT_MODE_ACCRETION.md`](H6_DISAGREEMENT_MODE_ACCRETION.md)

### H9 executable commitment state

- [`H9_EXECUTABLE_COMMITMENT_STATE.md`](H9_EXECUTABLE_COMMITMENT_STATE.md)

### H12 and structural transfer

- [`../research/R0A_H12_LATENT_ROLE_SUBSTRATE_CONSOLIDATION.md`](../research/R0A_H12_LATENT_ROLE_SUBSTRATE_CONSOLIDATION.md)

Additional H-series executable probes live under `lib/examples/` and may have research records outside this index. Their names identify tested mechanisms, not automatically generalized capabilities.

## Endogenous state and autonomous-kernel work

- [`OMEGA1_ENDOGENOUS_STATE_SPACE_GENESIS.md`](OMEGA1_ENDOGENOUS_STATE_SPACE_GENESIS.md)
- [`A1_BOUNDED_AUTONOMOUS_KERNEL.md`](A1_BOUNDED_AUTONOMOUS_KERNEL.md)

The “autonomous” label is bounded by the experiment contract. It does not imply unrestricted external action.

## Relational and IngExuity work

The relational bridge is implemented behind `relational-evidence` and evaluated through the R1 executable path.

Start with:

- [`../INGEXUITY_STARFIRE_INTEGRATION.md`](../INGEXUITY_STARFIRE_INTEGRATION.md)
- `lib/examples/r1_relational_residual_bridge.rs`

The current architectural decision is to keep Starfire as the cognitive substrate and treat IngExuity-derived work as relational/user-modeling machinery rather than an external wrapper.

## Grammar and abstraction sequence

Main includes executable examples for:

- `omega_g1_bounded_grammar_extension`;
- `omega_g2_recursive_grammar_composition`;
- `omega_g3_multistep_abstraction_reuse`;
- `omega_g4_intervention_guided_abstraction_selection`.

Read the example output, controls, and parent plan before describing a stage as complete. An executable that passes its own fixtures may still lack held-out transfer, matched-budget advantage, or live integration.

## Draft work outside main

ARISE-A0 currently lives in draft pull request `#128` and is not indexed as a main-branch experiment record here. Draft branches may have real external evidence, but they remain outside the authoritative main status until merged.

## Result classifications

Use these classifications literally:

| Classification | Meaning |
|---|---|
| `PASS` | Every frozen required gate passed for that experiment |
| `FAIL` | At least one frozen required gate failed |
| `COLLECTING` | Implementation is active but the frozen sample is incomplete |
| `BLOCKED` | A prerequisite prevents execution |
| `NO VERDICT` | Build or infrastructure failed before the evaluator produced a result |

Do not infer `PASS` from a green-looking deployment unless the image could not have been published without the experiment gate succeeding.

## Authority checklist

Before promoting an experiment, verify each item separately:

- raw prompt access;
- conversation-history access;
- unrestricted memory access;
- persistent-state mutation;
- returned-text influence;
- belief promotion;
- ontology promotion;
- routing authority;
- tool selection;
- CHARGE discharge;
- external action;
- rollback and kill switch;
- neutral fallback;
- privacy of recorded evidence.

## Evidence preservation policy

1. Never rename a FAIL into a PASS.
2. Never edit thresholds after seeing the result without a separately identified remediation.
3. Never describe a shadow stage as live influence.
4. Never describe a live-text canary as action authority.
5. Preserve external environment and commit identity where available.
6. Record infrastructure failures separately from scientific failures.
7. Keep interpretation narrower than the experiment name when necessary.

## Related documents

- [Current status](../CURRENT_STATUS.md)
- [Documentation index](../README.md)
- [Architecture](../architecture.md)
- [Plan index](../../plans/README.md)
