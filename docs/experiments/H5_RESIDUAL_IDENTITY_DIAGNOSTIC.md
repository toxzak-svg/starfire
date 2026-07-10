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

The executable also emits a `h5b_task_profiled` comparison. That path keeps the
same emitted charges and the same frozen H4 memory exclusion, but judges
non-memory resolver attempts with a task-profiled verifier instead of the
original surface-coverage verifier.

## Local result

Last local verification: local workspace run of the command above.

Observation path:

- 252 real subsystem-backed CHARGE observations
- 1,260 original surface-verifier judged cycle attempts
- 1,260 task-profiled verifier judged cycle attempts
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

## Task-profiled verifier comparison

`h5b_task_profiled` passed the same identifiability gates in the latest local
run:

- retained non-memory observations: `96`
- reasoning leader fraction: at least the `0.2` floor
- causal leader fraction: `0.5`
- positive margin fraction: about `0.30`
- negative margin fraction: `0.5`
- directional windows: `3`
- passed: `true`

Supported conclusion:

> The H5-B failure is at least partly verifier/task-ecology limited. With a
> verifier that scores contradiction correction and causal mechanism tasks
> differently, the same non-memory fixture exposes stable enough opposing
> resolver regimes.

## Next direction

Do not run H5-C from the original surface-verifier matrix. The next useful work
is to promote the task-profiled verifier contract from example-local diagnostic
logic into a reusable, tested verifier layer, then rerun H5-B as a new frozen
diagnostic before attempting ontology induction.
