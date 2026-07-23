# EI-0B Deterministic Developmental Environment

> **Stage:** EI-0B  
> **Status:** implementation contract  
> **Authority:** offline experiment infrastructure only  
> **Parent:** issue #156 and EI tracker #149  
> **Implementation:** PR #176  
> **Verification baseline:** current `main` merged before the terminal gate  
> **Gate mode:** permanent scoped Clippy on pinned Rust 1.96.0  
> **Verification gate:** `.github/workflows/ei-0b-environment-ci.yml`

## Purpose

EI-0B provides the frozen environment needed to test future learning claims without confusing stored state, evaluator coupling, task leakage, or unequal budgets with improvement.

This stage does not apply learning updates and does not claim that Starfire improves from experience. It defines deterministic worlds, isolated comparison arms, independent scoring, and exact replay.

## Task families

The first frozen environment contains two bounded families:

1. **Route choice:** select between routes with distinct total costs.
2. **Attribute rule:** infer a two-attribute conjunction from labeled examples and select the sole matching candidate.

Both families expose one bounded action and two evidence reads per fixture.

## Frozen partitions

| Partition | Seeds | Purpose |
|---|---:|---|
| Development | 101, 102 | Initial experience fixtures |
| Within-family holdout | 201, 202 | Unseen same-family structures |
| Renamed-vocabulary transfer | 301, 302 | Development relations with renamed surface tokens |
| Structural transfer | 401, 402 | Development relations with changed path or candidate composition |
| Regression | 501, 502 | Frozen prior-task replay |
| Adversarial | 601, 602 | Low-reliability misleading evidence cues |

Seeds are globally disjoint. Transfer fixtures map to the matching development relation through a documented deterministic seed transform while remaining members of separate partitions.

## Comparison arms

Every sealed fixture creates exactly five arm assignments with identical action budgets, evidence budgets, fixture digests, and evidence exposure:

- learning candidate;
- no-update control;
- memory-disabled control;
- random-update control;
- fixed-policy baseline.

Each arm receives a distinct state namespace. EI-0B grants no arm authority to apply a learning update. The learning arm name reserves the future comparison slot only.

## Independent evaluator boundary

The evaluator accepts only:

- a sealed task fixture;
- the matching arm specification;
- a recorded action trace.

It has no input for internal rationale, retrieved memory contents, proposed learning updates, model state, or self-reported confidence. Route tasks are scored from recorded route cost. Attribute tasks are scored from the frozen environment rule.

## Determinism and fail-closed behavior

The contracts require:

- canonical JSON bytes and domain-separated deterministic digests;
- byte-identical fixture and report replay;
- canonical collection ordering;
- exact frozen partition membership;
- matched budgets and evidence exposure;
- isolated arm state namespaces;
- stale-version rejection;
- cross-partition rejection;
- digest-tamper rejection;
- malformed task rejection;
- illegal-action and over-budget rejection.

## Claim boundary

A passing EI-0B probe supports only this claim:

> Starfire contains deterministic, replayable developmental-environment infrastructure with frozen partitions, matched isolated arms, and independent scoring.

It does not support EI-0 PASS, cumulative improvement, transfer learning, safe live learning, AGI, consciousness, or runtime authority.

## Verification commands

```bash
cargo fmt --check -- \
  lib/emerging_intelligence/environment/mod.rs \
  lib/emerging_intelligence/environment/manifest.rs \
  lib/emerging_intelligence/environment/evaluation.rs \
  lib/emerging_intelligence/environment/fixture/mod.rs \
  lib/emerging_intelligence/environment/fixture/generator.rs \
  lib/emerging_intelligence/environment/tests.rs \
  lib/examples/ei_0b_deterministic_environment_probe.rs

cargo check -p star --lib --features emerging-intelligence-environment --locked
cargo test -p star --lib --features emerging-intelligence-environment --locked \
  emerging_intelligence_environment:: -- --test-threads=1
cargo run -p star --example ei_0b_deterministic_environment_probe \
  --features emerging-intelligence-environment --locked
```
