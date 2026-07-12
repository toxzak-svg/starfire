# R1 — Relational Residual Bridge

**Status:** shadow-only implementation contract  
**Producer:** IngExuity-compatible relational prediction records  
**Consumer:** Starfire CHARGE  
**Authority:** none  
**Runtime wiring:** none

## Purpose

R1 establishes the first typed boundary for relationally grounded cognitive development:

1. a system issues an explicit probability distribution before an interaction;
2. a later source records what actually happened;
3. Starfire validates that the outcome is later and independently witnessed;
4. Starfire computes the residual rather than trusting predictor-supplied error;
5. sufficiently large residuals become unissued `PredictionResidual` CHARGE objects;
6. nothing is routed, persisted, promoted, or acted on automatically.

This is infrastructure for later experiments, not evidence that relational prediction improves cognition.

## Research question

Can IngExuity-style user and response-policy predictions be translated into deterministic, provenance-preserving Starfire pressure without allowing the response generator to grade itself or acquire live authority?

## Wire contract

The prediction record contains:

- schema version;
- prediction ID;
- opaque subject ID;
- target and context scope;
- issuance sequence;
- explicit horizon;
- a complete labeled probability distribution;
- producer name, version, and optional state hash.

The later outcome witness contains:

- the same prediction ID;
- a strictly later observation sequence by default;
- one label from the original prediction distribution;
- witness confidence;
- evidence ID;
- an explicit source class.

Accepted independent source classes are:

- explicit user correction;
- explicit user confirmation;
- subsequent user behavior;
- task metric;
- external evaluator.

`generator_self_report` is represented in the schema only so it can be rejected with a typed error.

## Residual calculation

For labels `i = 1..K`:

```text
residual_i = observed_one_hot_i - predicted_probability_i
brier_score = sum(residual_i^2)
rms_residual = sqrt(brier_score / K)
charge_magnitude = clamp(rms_residual * witness_confidence, 0, 1)
```

The default bridge emits a shadow charge when magnitude is at least `0.15`.

The emitted object is:

```text
kind: PredictionResidual
scope: Custom("relational:<target>:<context_scope>")
id: 0
promotion_eligible: false
```

`id == 0` is deliberate: R1 does not insert into `ChargeLedger`. A later preregistered experiment must explicitly authorize persistence and routing.

## Implemented invariants

- unsupported schema versions are rejected;
- blank identifiers and provenance fields are rejected;
- fewer than two candidate outcomes are rejected;
- duplicate labels are rejected;
- probabilities must be finite, bounded, and sum to one within tolerance;
- witness confidence must be finite and bounded;
- prediction and witness IDs must match;
- the observed label must have been declared before the interaction;
- future ordering is required by default;
- generator self-judgment is rejected;
- residuals are computed inside Starfire;
- low residuals do not emit CHARGE;
- emitted CHARGE is unissued and shadow-only;
- replay is deterministic;
- every assessment reports `promotion_eligible = false`.

## Tests

The module tests cover:

1. confident wrong prediction -> shadow CHARGE;
2. correct high-confidence prediction -> below threshold;
3. generator self-report -> rejected;
4. non-future witness -> rejected;
5. malformed distribution -> rejected;
6. deterministic replay -> identical serialized report and no promotion.

## Explicit non-goals

R1 does not add:

- live `Runtime::chat()` integration;
- IngExuity database ingestion;
- automatic resolution of IngExuity predictions;
- user-belief mutation;
- response-policy selection;
- cognitive-operator routing;
- ontology or concept proposal;
- skill compilation;
- autonomous action;
- automatic promotion;
- an AGI claim.

## Build and run

```text
cargo check -p star --lib --features relational-evidence --locked
cargo test -p star relational:: --features relational-evidence --locked -- --test-threads=1
cargo run -p star --example r1_relational_residual_bridge --features relational-evidence --locked
```

## Next experiment

R2 should add an IngExuity-side producer/export contract and a frozen replay fixture with named controls:

1. recent-context majority baseline;
2. recency baseline;
3. user-memory prediction without CHARGE;
4. relational prediction with residual calculation but no persistence;
5. relational residual CHARGE with scrambled target/scope;
6. oracle label upper bound.

Only after calibrated predictions beat those baselines should persistent relational CHARGE or routing be considered.

## Claim boundary

A passing R1 implementation supports only this statement:

> Starfire can independently validate and score a later outcome against a previously issued relational probability distribution, then represent a sufficiently large error as shadow-only typed CHARGE without granting the predictor self-judgment, promotion, or action authority.
