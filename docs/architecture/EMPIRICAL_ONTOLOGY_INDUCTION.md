# Empirical ontology induction

Starfire's ontology layer is no longer only a concept registry and promotion gate.
`charge::EmpiricalOntologyInducer` can fit an executable routing ontology from
measured CHARGE-resolution history.

This is a constrained empirical induction mechanism. It does not assign natural-language
meaning to concepts and it does not generate arbitrary code.

## Input contract

The inducer consumes two caller-owned cohorts:

- training observations for candidate generation, resolver selection, and candidate ranking
- holdout observations used only to accept or reject the single strongest training candidate at each generation

Each `OntologyObservation` contains:

- a replayable `Charge`
- zero or more measured attempts for each candidate resolver
- measured discharged magnitude
- declared compute cost

A caller can record an existing Starfire `Resolution` directly:

```rust
use star::charge::{OntologyObservation, Resolution};

let resolution = Resolution {
    discharged: 0.4,
    emitted: vec![],
    permitted_decay: 0.0,
    compute_cost: 3,
};

let observation = OntologyObservation::new(charge)
    .with_resolution("reasoning", &resolution);
```

The inducer does not execute resolvers. That boundary is deliberate: environment interaction,
independent discharge judgment, and resolver execution remain outside ontology search. The
inducer receives empirical history after those attempts have been measured.

## Resolver utility

For each observation and resolver, utility is:

```text
(discharge / incoming CHARGE magnitude) / compute cost
```

Repeated measurements for the same resolver and observation are averaged. Missing resolver
measurements receive zero utility for that observation. This prevents a sparsely sampled resolver
from winning by being evaluated only on favorable cases.

## Candidate search

At every generation, the current parent cohort is the set of training observations that do not
match an already promoted concept.

The first production search vocabulary is intentionally bounded and deterministic:

- residual dimension `>= threshold`
- residual dimension `<= threshold`
- persistence lower/upper ranges
- resolver-trace membership

Residual thresholds are midpoints between adjacent unique observed values. Search is bounded by
`max_thresholds_per_dimension`; when a dimension contains more possible midpoints than the budget,
the implementation samples midpoint positions deterministically across the sorted range.

Equivalent effective memberships are deduplicated.

## Greedy growth without holdout shopping

For every generation:

1. build the current ordered concept-routing policy from training history
2. generate candidates only inside the unmatched parent cohort
3. append each candidate and measure marginal training efficiency
4. subtract the configured complexity penalty
5. select exactly one strongest training candidate
6. require minimum support on both sides of the holdout parent cohort
7. evaluate that one candidate on holdout
8. submit it to `OntologyInducer` for support, holdout-gain, and utility promotion
9. stop growth if the candidate fails promotion
10. otherwise retain its opaque `ConceptId`, executable predicate, and empirically strongest resolver

The implementation deliberately does not test candidate two, candidate three, and so on against
holdout after a rejection. This avoids using holdout as a search oracle.

## Executable result

`fit` returns `LearnedOntology`.

```rust
let ontology = inducer.fit(&train, &holdout)?;
let decision = ontology.route(&charge);

println!("resolver = {}", decision.resolver);
println!("concept = {:?}", decision.concept);
```

Promoted concepts are checked in promotion order. The first matching concept selects its learned
resolver. A charge matching no promoted concept uses the strongest resolver for the final unmatched
parent cohort.

The result also exposes:

- promoted `ConceptRoute` values
- final parent resolver
- candidate count
- promoted concept count
- baseline training/holdout efficiency
- induced training/holdout efficiency

## Validation and rejection

Fitting rejects:

- empty train or holdout cohorts
- non-finite or non-positive CHARGE magnitudes
- observations with no measured resolver outcomes
- empty resolver names
- negative, non-finite, or over-magnitude discharge
- zero compute cost
- zero threshold-search budget
- negative or non-finite complexity penalties

## Current boundary

The first reusable engine specializes the root parent cohort through ordered promoted concepts.
It does not yet recursively induce children beneath an existing promoted concept, merge concepts,
induce relations, or reify repeated relation patterns.

Those remain explicit ontology mutations in `OntologyMutation` and should be added under separate
falsification tests rather than silently expanding this mechanism's claim.

The practical next integration is to feed this inducer independently judged outcomes from the
closed cognitive cycle, fit on historical windows, and compare concept-aware routing against the
current CHARGE routing signature on future windows.
