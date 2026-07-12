# H-Infant Multi-Pressure Prioritization Probe

**Status:** diagnostic-only, no live scheduler integration

## Question

When several unresolved pressures compete for one attention slot per step, does generic persistence-aware pressure accounting prioritize a quieter but persistent developmental residual better than magnitude-only, random, or oldest-first policies?

The scheduler is forbidden from inspecting `ChargeKind`.

## Pressure stream

At every step a fresh transient distractor appears:

- kind cycles across `EpistemicGap`, `GoalTension`, and `Contradiction`,
- magnitude is uniformly sampled from `3.2..3.8`,
- persistence is `0`.

A calibrated developmental `PredictionResidual` appears when prediction error exceeds its scoped threshold:

- magnitude is fixed at `1.0`,
- persistence increments across consecutive exceedances,
- residual structure is `[intercept, slope]` for affine model repair.

Thus a fresh distractor is always louder than the developmental residual. The relevant pressure can win generically only by persisting.

## Attention budget

Exactly one pressure may receive attention per step.

Selecting a distractor consumes the step's attention slot.

Selecting the developmental residual consumes the single model-revision attempt and invokes the same typed resolver plus independent held-out witness used in the parent probe.

## Policies

- **persistence-weighted pressure:** highest `magnitude * (1 + persistence)` using actual `Charge` fields.
- **direct persistence-weighted:** identical score outside the Charge container.
- **magnitude-only pressure:** highest raw magnitude.
- **random pressure:** random active pressure.
- **oldest-first pressure:** highest persistence.
- **no revision**.
- **oracle**.

The direct persistence-weighted control prevents claiming a unique task-performance benefit from the `Charge` container if the generic score itself is what helps.

## Evidence timing

Post-shift actions follow four-step blocks `++--` with randomized sign. Early residual windows are intentionally incomplete; a four-sample window contains both action signs and supports reliable separation of intercept and slope.

## Frozen gates

Across 256 deterministic seeds:

- persistence-weighted commit rate `>= 95%`,
- false commit rate `<= 2%`,
- median detection delay `<= 4` steps,
- mean MSE `<= 1.05x` direct persistence-weighted MSE,
- mean MSE `<= 0.25x` magnitude-only MSE,
- mean MSE `<= 0.50x` random MSE,
- mean MSE `<= 0.50x` oldest-first MSE,
- excess mean MSE over oracle `<= 10.0`,
- median persistence when the relevant pressure is selected `>= 3`,
- persistence-weighted must beat magnitude-only, random, oldest-first, and no revision,
- unverified commit rate must be exactly `0` for every policy.

The executable exits non-zero if any gate fails.

## Interpretation

A pass would support a narrow claim:

> Generic pressure persistence can improve attention allocation when transient high-magnitude distractors compete with a lower-magnitude but recurrent model failure.

If the Charge-backed and direct persistence-weighted policies tie, the correct interpretation is that the priority rule is useful and the Charge representation preserves it with explicit typed accounting; the experiment would not prove a unique task-performance benefit from the container itself.

A pass justifies held-out transfer of the generic priority rule, not live scheduler promotion.