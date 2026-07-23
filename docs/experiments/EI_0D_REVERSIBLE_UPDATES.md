# EI-0D Reversible Learning Updates

> **Stage:** EI-0D  
> **Status:** implementation under verification  
> **Authority:** isolated offline update transactions only  
> **Parent:** issue #192 and EI tracker #149  
> **Implementation:** PR #194  
> **Feature:** `emerging-intelligence-updates`  
> **Verification gate:** `.github/workflows/ei-0d-updates-ci.yml`

## Purpose

EI-0D defines the first mechanism allowed to apply a learning update, but only to an isolated, fixed-schema experiment state. Every mutation is causally bound to a sealed EI-0A episode, an EI-0C ledger root, an expected pre-state digest, a fixed EI-0B control-arm namespace, and independently produced admissibility and held-out safety verdicts.

The stage must prove that accepted updates apply atomically, harmful or inadmissible updates fail closed, and rollback restores byte-identical pre-state.

## Frozen update lattice

The initial lattice contains only bounded numeric adjustments to five fixed fields:

- route-cost weight;
- route-decoy bias;
- verified-evidence weight;
- attribute-rule coverage weight;
- attribute-rule decoy bias.

Updates cannot create fields, routes, attributes, concepts, tools, schemas, or ontology nodes.

## Transaction boundary

A valid transaction must:

1. validate the sealed source episode and verify its ID, digest, and accepted update ID inside the supplied EI-0C ledger;
2. bind to the exact target namespace, control arm, state schema, and pre-state digest;
3. validate the typed before value, after value, delta, per-update bound, and cumulative budget;
4. receive an independent admissibility verdict and an independent held-out safety verdict;
5. apply to a cloned state, validate the post-state, and commit only after every check succeeds;
6. emit a canonical transaction record containing exact pre-state, attempted post-state, and final-state bytes and digests;
7. restore the exact pre-state only when rollback is applied to the matching committed post-state and transaction.

No-update, memory-disabled, and fixed-policy controls use the same transaction interface but produce deterministic no-op records. Random-update uses the same bounded transaction path in a distinct state namespace.

## Harmful-update rule

A structurally valid update is automatically restored to pre-state when independent held-out evaluation detects any protected-partition loss across within-family holdout, renamed-vocabulary transfer, regression, or adversarial fixtures.

The learning candidate cannot self-certify safety.

## Claim boundary

A passing EI-0D probe supports only this claim:

> Starfire contains bounded offline update transactions that can apply admissible fixed-schema changes and restore exact prior state when independent evaluation rejects an update.

It does not support cumulative improvement, transfer learning, EI-0 PASS, live learning, ontology growth, unrestricted self-modification, AGI, or consciousness.

## Verification target

The permanent EI-0D gate must prove pinned formatting, full feature and probe compilation, scoped Clippy cleanliness, focused apply/reject/rollback/corruption/control-isolation tests, and two byte-identical executions of the deterministic probe.
