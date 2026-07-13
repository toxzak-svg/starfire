# R0-A — H12 Latent-Role Substrate Consolidation

Status: **engineering consolidation only**

## Purpose

Port the smallest reusable H12 proof-carrying latent structural-role substrate onto current `main` without reopening, relabeling, or silently merging the closed experimental PR #27.

This change does not create a new scientific result. It preserves the prior accepted executable behavior as an engineering foundation for a separately preregistered H12-C conformance experiment.

## Provenance

The implementation in `lib/latent_roles.rs` is the exact repository blob from the H12 branch:

```text
source branch: research/h12-proof-carrying-latent-roles
source PR:     #27
blob SHA:      c185e74009497a2b32ea4a2dc8ae741b67117fff
```

No fingerprint field, induction rule, proof field, validator gate, registry transition, transfer recognizer, projection rule, budget counter, digest function, or unit test is changed in this port.

## Consolidated surface

The module provides:

```text
raw directed structural graphs
structural corpus validation
target-blind deterministic structural fingerprints
opaque latent-role identities
recurrence-bounded role induction
proof objects with exact membership and scope binding
independent complete role-proof recomputation
opaque validated certificates
scope-bound shadow abstraction registry
held-out structural-role recognition
role-conditioned inert evidence projection
matched control-group evidence projection
explicit discovery, validation, recognition, and scan budgets
```

## Authority boundary

The consolidated substrate remains research-only and shadow-only.

It has:

```text
no Live registry state
no Runtime::chat() attachment
no response-routing authority
no autonomous action authority
no automatic ontology promotion
no direct PECS mutation
no persistent production ontology mutation
```

`ExecutableShadow` means only that a validated abstraction may be used by an explicitly invoked research computation. It does not confer production authority.

Projected evidence remains inert. Any executable rule derived from it must still pass the existing graph-discovery, independent rule-validation, and certificate-admission path.

## Scientific boundary

The unresolved H12 control-conformance discrepancy remains unresolved here:

```text
implemented historical control:
deterministic mixed target/adversarial grouping

literal preregistered control still required:
root-seeded same-cardinality random partition
```

This PR must not be cited as H12-C, H13, ontology induction, grammar invention, causal abstraction, or integrated lifecycle validation.

## Verification

The dedicated R0-A workflow requires:

```text
rustfmt --edition 2021 --check lib/latent_roles.rs
cargo check -p star --lib --locked
cargo test -p star latent_roles:: --locked -- --test-threads=1
```

The existing unit tests cover:

```text
deterministic recurring-role induction
independent opaque certificate validation
one-member proof tamper rejection
foreign-scope rejection
validated transfer recognition
role-conditioned evidence projection
exact projection-scan accounting
```

## Next gate

After this engineering PR is clean and merged, create a separate preregistration and branch for **R0-B / H12-C literal control conformance**. Do not add H12-C controls or a new scientific verdict to this branch.
