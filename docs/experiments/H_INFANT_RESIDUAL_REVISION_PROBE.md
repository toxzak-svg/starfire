# H-Infant Residual Revision Probe

**Status:** diagnostic-only, no live runtime integration  
**Dependency:** stacked after the evidence, numeric residual, cross-language fixture, and residual-calibration gates

## Question

Does **persistent calibrated residual exceedance** provide a useful trigger for bounded model revision under a hidden dynamics regime shift?

This is the first experiment in the Infant–Starfire fusion program that tests downstream utility rather than only interoperability or measurement validity.

It still does **not** emit CHARGE.

## Environment

Each episode has 200 steps.

A hidden shift step is sampled uniformly from steps 70 through 130.

Before the shift:

```text
observed_delta = +1 * action + noise
```

After the shift:

```text
observed_delta = -1 * action + noise
```

Actions are `-1` or `+1`. Gaussian observation noise has `sigma = 0.05`.

Every strategy receives:

- the exact same episode,
- the exact same two-model bank,
- at most one model revision.

Only the revision trigger differs.

## Calibration

For each seed, a separate 1,000-sample regime-A calibration stream is generated.

Starfire:

1. computes numeric residuals independently,
2. fits a scoped 99th-percentile `higher` MSE threshold,
3. freezes the threshold before the episode begins.

The threshold is specific to:

- predictor scope,
- environment scope,
- state space,
- prediction horizon.

## Strategies

### Persistent calibrated

Revise after **three consecutive** calibrated threshold exceedances.

### Single exceedance

Revise after the first calibrated threshold exceedance.

### Scheduled

Revise at fixed step 100.

### Random

Revise once at a deterministic pseudorandom step.

### No revision

Never revise.

### Oracle-after-first-shifted-observation

Revise immediately after observing the first shifted transition.

This oracle is an upper-bound context strategy under the same observation timing: it does not revise before the first shifted outcome exists.

## Revision budget

Every non-`no_revision` strategy may switch from model A to model B at most once.

No strategy may search a larger model bank or consume additional revision opportunities.

## Primary metric

Cumulative prediction MSE over the full episode.

Secondary metrics:

- post-shift MSE,
- false revision rate,
- no-revision rate,
- detection delay.

## Frozen gates

Across 128 deterministic seeds:

- persistent false revision rate `<= 5%`,
- persistent median detection delay `<= 3` steps,
- persistent mean cumulative MSE `<= 10%` of no-revision mean MSE,
- single-exceedance false revision rate minus persistent false revision rate `>= 30 percentage points`,
- persistent excess mean cumulative MSE over the observation-timed oracle `<= 10.0`,
- persistent mean cumulative MSE must beat:
  - single exceedance,
  - scheduled revision,
  - random revision,
  - no revision.

The executable exits non-zero if any gate fails.

## Interpretation

A pass would establish that persistent calibrated residual exceedance can be a useful **bounded model-revision trigger** in this diagnostic family.

A pass would **not** establish:

- that the signal should automatically become CHARGE,
- that the threshold transfers across predictor or environment scopes,
- that a neural model should control Starfire routing,
- that ontology promotion should be automatic,
- that the live runtime should be modified.

The next gate after a pass is a closed-cycle comparison where a typed residual-pressure candidate competes against matched non-pressure routing and is judged only by independent task outcomes.
