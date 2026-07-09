# H4 shadow promotion transfer

## Status

Frozen CPU acceptance probe for shadow-only latent-concept eligibility.

The experiment does not modify live CHARGE routing. It asks whether automatically induced concepts can earn eligibility by surviving future-window transfer and exact matched-budget controls.

## Why shadow mode exists

An induced ontology changes which resolver receives future CHARGE. That changes which outcomes are observed, which can later make the ontology appear increasingly correct because its own routing policy shaped the evidence stream.

H4 shadow mode breaks that feedback loop.

The active router remains untouched. The candidate ontology is fit from historical train/holdout windows, frozen, and then evaluated in parallel on later CHARGE windows for which all candidate-resolver outcomes are already measured.

The public `ShadowPromotionMonitor` intentionally exposes no method that applies an eligible ontology to the live router.

## Automatic phase transition

`ShadowPromotionMonitor::observe_window` advances through four phases:

```text
collect training windows
        |
        v
collect independent promotion holdout
        |
        v
automatically fit frozen candidate ontology
        |
        v
score candidate on future transfer windows
        |
        v
await exact matched-budget controls
```

A candidate cannot reach `Eligible` from future transfer alone. After the configured number of future windows, the monitor stops at:

```text
AwaitingMatchedBudgetControls
```

Controls are provided through `assess_controls`.

## Exact budget contract

The candidate's proposal budget is the actual `OntologyInductionSummary::candidates_considered` count produced during empirical induction.

Its future routing budget is the exact number of future observations scored in shadow mode. Both shadow and baseline select one resolver per future observation.

Every control submitted to the eligibility gate must report exactly:

```text
proposal_evaluations == candidate proposal evaluations
routing_evaluations  == candidate future observations
```

A mismatch is an error rather than a failed score. A cheaper control is not considered a valid matched-budget comparison.

## Frozen probe

The executable probe is:

```text
cargo run -p star --example h4_shadow_promotion_probe --locked
```

The final seed and gates are constants in the executable source before the CI acceptance run.

### Window structure

- 2 training windows
- 1 independent promotion holdout window
- 4 future transfer windows
- 108 observations per window
- every visible charge kind is `Custom("unresolved")`

Exact topic/surface strings change across training, holdout, and every future family.

### Hidden response structure

The stream contains three hidden resolver-response regimes:

- memory-favoring
- reasoning-favoring
- causal-favoring

Hidden class labels are not passed to the inducer, the shadow monitor, candidate generation, promotion, or routing.

The visible residual distributions deliberately overlap. Each residual dimension receives triangular noise and 12% of observations swap the class-favored residual dimension with another dimension. Future windows also drift progressively away from the train-window centers and favored resolver outcomes degrade slightly with drift.

Hidden labels are retained only for post-hoc concept purity diagnostics.

## Matched-budget controls

### Matched random partition search

The random control receives exactly the candidate's proposal evaluation count.

Each proposal is a complete hash partition with the same effective number of routing groups as the induced ontology plus parent fallback. Resolver leaders are learned from training history. The strongest training proposal is evaluated once on holdout and is used for future routing only if holdout support and gain pass the frozen promotion threshold.

The random control receives one routing decision per future observation.

### Matched permuted-feature search

Residual dimensions and persistence are independently permuted across observations while resolver outcomes remain attached to the original observations.

The control receives exactly the candidate's proposal evaluation count. Every proposal is a full concept policy sampled from the same residual-threshold vocabulary and with the same concept count as the induced ontology. The strongest training policy receives one holdout gate and, if accepted, one routing decision per future observation.

This is intentionally a strong control: full multi-concept policies are counted as one proposal evaluation even though the empirical inducer grows concepts incrementally.

## Frozen final gates

The shadow candidate exits successfully only if all gates pass:

1. at least 2 concepts were promoted during historical induction
2. future shadow efficiency is at least `1.35x` the undifferentiated baseline
3. shadow beats baseline in `100%` of future windows
4. the worst individual future window retains at least `1.20x` baseline efficiency
5. shadow future efficiency is at least `1.25x` every exact matched-budget control

Budget equality is checked before the control ratios are evaluated.

The gate values must not be changed after observing the CI acceptance result for the frozen seed.

## Eligibility boundary

`Eligible` means:

> A frozen latent-concept routing ontology induced from historical CHARGE-resolution evidence retained useful resolver discrimination on later drifting windows and beat exact matched-budget random and feature-destroyed search controls.

It does not mean:

- concepts should automatically enter the live router
- the ontology is semantically human-like
- the mechanism is stable under arbitrary distribution shift
- recursive ontology growth is proven
- relation induction or reification is proven
- AGI or consciousness has been demonstrated

The next step after a pass is a real subsystem-backed shadow stream where `OntologyObservation` values come from independently judged Starfire resolver attempts rather than generated response surfaces.
