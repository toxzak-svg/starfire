# H-Infant Typed Residual-Pressure Routing Probe

**Status:** diagnostic-only, no live runtime routing  
**Dependency:** stacked after the residual-revision usefulness gate

## Question

Can a structured `PredictionResidual` pressure candidate route one bounded model-repair attempt to the correct operator better than matched non-pressure routing?

This is the first experiment in the Infant–Starfire fusion stack that places developmental residual structure inside an actual Starfire `Charge`, passes the proposed repair through `CognitiveCycleState`, and accepts discharge only after an independent held-out witness.

It still does not alter `Runtime::chat()` or live routing.

## Hidden shift families

Every episode begins with:

```text
predicted_delta = 1.0 * action + 0.0
```

At a hidden step sampled from 60–120, one of two equally represented families appears.

### Scale shift

```text
true_scale ~ Uniform[-0.5, 0.2]
true_bias = 0
```

Required operator: `revise_scale`.

### Bias shift

```text
true_scale = 1
true_bias ~ Uniform[0.8, 1.5]
```

Required operator: `revise_bias`.

Actions alternate `+1, -1`, with a randomized starting sign. Gaussian observation noise has `sigma = 0.20`.

## Calibration and trigger

For each of 256 deterministic seeds:

1. Starfire independently computes 1,000 baseline residuals.
2. Starfire fits a scoped 99th-percentile `higher` MSE threshold.
3. During the episode, four consecutive threshold exceedances are required before one resolver attempt is allowed.

All trigger-based strategies receive the same:

- episode,
- calibration profile,
- trigger timing,
- raw residual evidence,
- held-out witness budget,
- maximum of one resolver attempt.

## Residual structure

Over the four-step persistent window:

```text
intercept = mean(residual)
slope = mean(action * residual)
```

For a bias shift, `intercept` dominates.

For a scale shift, `slope` dominates.

The typed pressure candidate stores:

```text
ChargeKind::PredictionResidual
residual = [intercept, slope]
```

and routes:

```text
abs(slope) > abs(intercept) -> revise_scale
otherwise                     -> revise_bias
```

The repair operator estimates only its own parameter:

```text
revise_scale -> scale = 1 + slope
revise_bias  -> bias  = intercept
```

## Independent discharge

The candidate repair is evaluated on a separate 32-transition held-out witness stream.

The candidate is independently verified only when:

```text
after_mse <= 0.25 * before_mse
```

For charge-bearing strategies, the resolver requests full discharge, but `RelativeImprovementJudge` receives a binary independently measured verifier witness:

```text
before = 0
after  = 1 if verified else 0
```

Only a resolved charge may commit the candidate model.

A failed verifier leaves the charge unresolved. The one-attempt budget prevents retrying a second resolver in this experiment.

## Strategies

### Typed pressure

Uses the true `[intercept, slope]` residual vector inside `PredictionResidual` and the Starfire cognitive-cycle discharge path.

### Structured direct, no pressure

Uses the same structured residual classifier and same held-out verifier, but bypasses `Charge` and `CognitiveCycleState`.

This control is important: it tests whether task utility comes from residual structure rather than merely wrapping the same classifier in a `Charge` container.

### Untyped matched trigger

Gets the same persistent trigger and evidence but cannot inspect residual structure for routing. It always attempts `revise_scale`.

### Random matched trigger

Gets the same trigger and evidence but chooses one resolver randomly.

### Scrambled pressure

Uses the Starfire charge/discharge path but swaps the residual coordinates before routing.

### No revision

Never attempts a repair.

### Oracle

At the hidden shift, applies the exact correct post-shift model. This is an upper-bound context strategy.

## Frozen gates

Across 256 seeds:

- typed resolution/commit rate `>= 95%`,
- typed false commit rate `<= 2%`,
- typed median detection delay `<= 4` steps,
- typed mean cumulative MSE `<= 1.05x` structured-direct mean MSE,
- typed mean cumulative MSE `<= 0.25x` untyped matched-trigger mean MSE,
- typed mean cumulative MSE `<= 0.30x` random matched-trigger mean MSE,
- typed resolver-correctness advantage over untyped `>= 45 percentage points`,
- typed excess mean cumulative MSE over oracle `<= 10.0`,
- typed mean cumulative MSE must beat:
  - untyped matched trigger,
  - random matched trigger,
  - scrambled pressure,
  - no revision,
- unverified commit rate must be exactly `0` for every strategy.

The executable exits non-zero if any gate fails.

## Interpretation

A pass would show that structured residual pressure can travel through the Starfire charge/cognitive-cycle accounting path without losing the routing utility already present in the residual structure, while independently rejecting unverified revisions.

A pass would **not** prove that the `Charge` container itself outperforms direct structured routing. The structured-direct control explicitly measures that distinction.

If the typed-pressure and structured-direct strategies are equivalent, the correct conclusion is:

> the residual structure is useful; the Starfire charge path preserves that utility and adds explicit accounting/independent discharge, but this diagnostic does not yet establish a unique task-performance benefit from the pressure container.

The next justified gate after a pass is multi-pressure prioritization or held-out transfer, not live runtime promotion.
