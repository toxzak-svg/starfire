# H5-C non-memory ontology probe

## Status

Planned next implementation. No H5-C acceptance verdict exists yet.

This experiment must not be treated as started until the executable exists and
emits a complete structured report:

```text
cargo run -p star --example h5_non_memory_ontology_probe --locked
```

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

1. Add `lib/examples/h5_non_memory_ontology_probe.rs`.
2. Reuse the H5 diagnostic stream construction rather than forking task data.
3. Route only retained non-memory observations during H5-C fitting.
4. Keep hidden labels for post-hoc reporting only.
5. Emit a JSON report with frozen constants, proposal counts, route counts,
   promoted concepts, gates, controls, and final verdict.
6. Exit nonzero only for infrastructure/report errors or a configured
   acceptance job; do not hide failed scientific gates.
