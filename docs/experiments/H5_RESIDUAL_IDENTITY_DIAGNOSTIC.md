# H5 residual identity diagnostic

## Status

Implemented as a diagnostic-only executable:

```text
cargo run -p star --example h5_residual_identity_diagnostic --locked
```

This does not promote concepts into live routing. It preserves H4 as a rejected
real closed-cycle result and asks two narrower questions:

1. whether fixed-width residual projection removes utility from the
   feature-destroyed control while preserving real-feature utility
2. whether the non-memory remainder exposes stable reasoning-favored and
   causal-favored resolver regimes strongly enough to justify H5-C ontology
   induction

## Local result

Last local verification: local workspace run of the command above.

Observation path:

- 252 real subsystem-backed CHARGE observations
- 1,260 independently judged cycle attempts
- visible kind always `Custom("unresolved")`
- same H4 task-family split and closed-cycle judging path

## H5-A result

H5-A passed its diagnostic criteria in the local run.

Key values:

- H4 variable future efficiency: about `0.6686`
- H4 variable permuted future efficiency: about `0.6667`
- H5 fixed future efficiency: about `0.6686`
- H5 fixed permuted future efficiency: about `0.3343`
- fixed retained H4 utility: `true`
- fixed beat fixed-permuted: `true`
- fixed-permuted fell below H4-permuted: `true`
- fixed beat baseline: `true`

Supported conclusion:

> Fixed-width projection preserved the real-feature routing utility while
> reducing the permuted-feature control, supporting residual-shape leakage as a
> material H4 confound.

This does not establish that non-memory ontology induction is justified.

## H5-B result

H5-B failed its identifiability gates after excluding the frozen H4 memory
predicate.

Exclusion:

- excluded by H4 memory predicate: `48`
- retained non-memory observations: `96`
- excluded hidden distribution: `KnowledgeGap = 48`
- retained hidden distribution: `PredictionContradiction = 48`,
  `QuanotTrajectory = 48`

Overall retained non-memory leader distribution:

- reasoning: about `0.49`
- causal: `0.5`
- prediction: about `0.01`
- metacognition: `0.0`
- ties: `0.0`

Failed gates:

- positive reasoning-over-causal margin floor: `false`
- stable future directionality: `false`

Supported conclusion:

> After removing the frozen H4 memory-shaped cohort, the current real-component
> outcome matrix does not expose stable positive reasoning-over-causal margins
> strongly enough across future windows to justify H5-C ontology fitting.

## Next direction

Stop ontology induction on this fixture matrix for now. The next useful work is
to improve the verifier/task ecology so the non-memory remainder contains
cleaner opposing reasoning-favored and causal-favored regimes, then rerun H5-B
as a new frozen diagnostic.
