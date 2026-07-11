# H8 budget addendum: paired-eligibility prepass

Status: frozen before H8 implementation and before the first complete H8 GitHub Actions run.

This addendum corrects one incomplete operation count in `H8_TRANSFORMED_ACTION_ORDER_DIAMOND.md`. It does not change the hypothesis, candidate words, continuation serialization, controls, numerical gates, or decision logic.

## Why a prepass is required

The frozen contract requires a symmetric paired cohort:

> an anchor is eligible only when both possible first operations, reasoning and causal, leave a non-empty unresolved state after independent witnessing and `RelativeImprovementJudge` application.

Determining that intersection requires executing and witnessing the first reasoning operation and the first causal operation once per retained root before matched composite paths are evaluated.

## Frozen eligibility prepass

For every retained non-memory root in a chronological split, execute exactly:

```text
reasoning-first prepass: 1 resolver call + 1 objective evaluation
causal-first prepass:    1 resolver call + 1 objective evaluation
```

The prepass may be used only to:

```text
construct the same-root IntermediateState for each possible first operation
classify whether the root remains unresolved after each first operation
form the symmetric paired-eligible intersection
construct the frozen rewired-donor permutation from eligible intermediate bundles
```

The prepass may not be used to:

```text
select R;C versus C;R
fit a threshold or direction
inspect holdout or future candidate performance
change the continuation scaffold
change a numerical gate
```

Composite paths still execute their own two resolver calls and two objective evaluations. The prepass is not substituted for a path call.

## Corrected exact operation proxy

Let:

```text
R = retained non-memory roots in a split
N = paired-eligible roots in that split
```

Eligibility-prepass cost is:

```text
resolver calls       = 2R
objective evaluations = 2R
```

Matched composite evaluation contains:

```text
2 words
4 paths per word: stateful, blind, scalar-state, common-root-rewired
2 resolver calls per path
2 objective evaluations per path
```

Therefore admitted-path cost is:

```text
composite path evaluations = 8N
resolver calls              = 16N
objective evaluations       = 16N
```

Total H8 deterministic operation proxy per split is exactly:

```text
resolver calls       = 2R + 16N
objective evaluations = 2R + 16N
```

The JSON report must emit all four counts separately:

```text
eligibility_resolver_calls
eligibility_objective_evaluations
composite_resolver_calls
composite_objective_evaluations
```

and assert:

```text
eligibility_resolver_calls == 2R
eligibility_objective_evaluations == 2R
composite_resolver_calls == 16N
composite_objective_evaluations == 16N
```

Any mismatch is an infrastructure error. It produces no scientific verdict.
