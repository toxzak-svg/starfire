# H5-C non-memory ontology probe

## Status

Primary shadow diagnostic implemented. The first complete H5-C acceptance
verdict is `PASS`.

The executable now emits a complete structured verdict:

```text
cargo run -p star --example h5_non_memory_ontology_probe --locked
```

Current report status is `COMPLETE_VERDICT` with final verdict `PASS`.

Latest local result:

- H5-C gates: `17/17`
- deterministic fresh report repeats: `3/3`
- retained non-memory observations across all windows: `168`
- future non-memory route evaluations: `96`
- proposal evaluations: `194`
- promoted concepts: `1`
- promoted predicate: `ResidualThreshold { dimension: 2, threshold: 0.088675305, direction: AtLeast }`
- promoted resolver: `reasoning`
- future support: `48`
- future margin-direction purity: `1.0`
- induced future efficiency: `1.0`
- induced versus non-memory parent: about `1.9883x`
- induced versus parent plus frozen H4 memory baseline: about `1.4956x`
- induced versus matched random control: passed
- induced versus matched permuted fixed-feature control: passed

This is a primary shadow diagnostic pass only. It is not replicated H5 support,
not live ontology promotion authority, not AGI evidence, and not a consciousness
claim.

## Input boundary

H5-C may consume only the canonical H5 diagnostic stream after:

1. fixed-width, mask-blind residual projection
2. frozen H4 memory-cohort exclusion
3. independently judged resolver outcomes from the task-profiled verifier in
   `star::charge::verifier`

The original surface-verifier H5-B failure is retained as a negative verifier
ecology control. It is not the fitting matrix for H5-C.

## Claim

H5-C asks:

> Can a shadow ontology learned only from normalized CHARGE features recover a
> transferable non-memory distinction that improves future resolver routing
> beyond an undifferentiated non-memory parent and exact matched-budget controls?

## Frozen primary constants

These constants should be copied into source before the first complete H5-C
acceptance verdict:

```text
SEED: 0x4834_5245_414c_4359
TRAIN_WINDOWS: 2
HOLDOUT_WINDOWS: 1
TRANSFER_WINDOWS: 4
verifier profile: TaskProfiled
candidate operators: ResidualThreshold, Not(threshold), And(2 thresholds)
complexity penalties: 0.003, 0.004, 0.008
```

## Required controls

H5-C must report:

- non-memory undifferentiated parent baseline
- parent-plus-frozen-memory baseline on the full future stream
- exact matched-budget random partition search
- exact matched-budget independently permuted fixed-feature search
- residual-length-only oracle diagnostic, marked post-hoc and non-gating

## Acceptance boundary

The primary verdict is conjunctive. A one-seed H5-C pass is only a
proof-of-mechanism candidate, not a live-promotion result.

Replicated support requires the predeclared multi-seed pass described in
`docs/plans/H5_RESIDUAL_IDENTITY_DIAGNOSTIC_PLAN.md`.

## Next implementation checklist

1. Done: add `lib/examples/h5_non_memory_ontology_probe.rs`.
2. Done: freeze primary constants, candidate vocabulary, required controls, and
   claim boundary in source before any complete verdict.
3. Done: reuse the H5 diagnostic stream construction rather than forking task
   data.
4. Done: route only retained non-memory observations during H5-C fitting.
5. Done: keep hidden labels for post-hoc reporting only.
6. Done: emit a JSON report with frozen constants, proposal counts, route counts,
   promoted concepts, gates, controls, and final verdict.
7. Done: exit nonzero only for infrastructure/report errors or a configured
   acceptance job; do not hide failed scientific gates.
8. Next: preserve the primary verdict without tuning.
9. Next: run the eight predeclared replication seeds before claiming replicated
   H5-C support.
