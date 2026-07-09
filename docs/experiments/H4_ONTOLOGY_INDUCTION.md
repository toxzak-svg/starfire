# H4 — Latent Distinction Induction from Undifferentiated CHARGE

## Question

Can Starfire begin with a deliberately insufficient ontology, discover that one routing class hides multiple stable response regimes, propose a reusable distinction, and retain that distinction only when it improves held-out CHARGE resolution?

This is an ontology-induction proof-of-mechanism experiment. It is not a consciousness, AGI, semantic-understanding, or open-ended self-improvement claim.

## Motivation

The existing CHARGE real-component specialization probe gives Starfire fixed human-authored charge kinds and scope classes. The router is role-blind, but the ontology is not: `EpistemicGap`, `Contradiction`, and `PredictionResidual` already exist before learning begins.

H4 removes that answer key.

The core hypothesis is that persistent routing inefficiency contains enough evidence to justify a new computational distinction. The system does not need to name the distinction in English. It needs to earn an opaque concept ID whose executable predicate transfers to unseen observations and improves a predeclared objective.

## Starting ontology

The H4 probe exposes one charge kind to induction:

```text
Custom("unresolved")
```

The search observes residual values and empirical resolver outcomes. It does not receive hidden source identity, human charge-kind labels, component semantic roles, fixture topic semantics, or oracle-best resolver labels.

## Hidden response regimes

The deterministic CPU probe generates three hidden response regimes modeled after the specialization seen in the real-component CHARGE experiment:

1. gap-like observations favor memory
2. contradiction-like observations favor reasoning
3. residual-like observations favor causal resolution

Every observation is presented to induction as the same `Unresolved` kind. Hidden labels are retained only for the oracle upper-bound and post-hoc diagnostics.

## Candidate vocabulary

H4 deliberately starts with the smallest replayable predicate search:

```text
residual[dimension] >= threshold
residual[dimension] <= threshold
```

For each residual dimension, proposal search sorts training values and generates midpoint thresholds between adjacent unique values. Both directions are evaluated. Partitions below minimum support are rejected. Equivalent positive-membership partitions are deduplicated.

No LLM proposes predicates. No source code is generated at runtime. Every candidate can be replayed deterministically against historical `Charge` objects through `ConceptPredicate::matches`.

## Greedy induction

The active ontology begins empty, meaning all observations fall through to one parent resolver profile.

For each induction generation:

1. evaluate current training discharge efficiency
2. append each remaining candidate predicate to the active concept list
3. measure marginal training gain under ordered concept routing plus parent fallback
4. rank candidates by marginal gain, then one-split training gain
5. evaluate the strongest candidates on the independent holdout cohort
6. submit the candidate to `OntologyInducer`
7. promote only if support, holdout gain, and aggregate utility pass the fixed promotion contract
8. add the promoted predicate to active routing and advance the ontology generation

The probe allows at most two promoted concepts. This is sufficient to partition three regimes because unmatched observations continue through the undifferentiated parent fallback.

## Machine-induced concept representation

A promoted concept is an opaque machine ID plus executable evidence:

```text
ConceptId(1)
    predicate: ResidualThreshold { ... }
    observations: N
    holdout_gain: measured independently
    routing_gain: ...
    discharge_gain: ...
    generation: G
```

The concept receives no English semantic name during induction.

Its operational meaning is the cohort selected by its predicate and the measured resolution utility earned by keeping the distinction.

## Promotion contract

H4 uses the merged `OntologyInducer` foundation. Promotion requires all of:

- at least 16 training observations selected by the predicate
- at least `0.02` absolute holdout discharge-efficiency gain over the currently active ontology
- at least `0.04` aggregate utility gain

For H4, routing gain and discharge gain both record the independently measured holdout improvement. Candidate search also rejects undersupported complements before promotion.

## Policy comparison

Every policy gets one resolver attempt per observation, so mean discharged magnitude is also discharge per unit compute in this first probe.

### Undifferentiated

One global empirical resolver leader is learned from training history and used for every holdout observation.

### Induced

Promoted opaque predicates are checked in induction order. A matching concept uses the strongest empirical resolver for that concept's training cohort. Unmatched observations use the parent leader.

### Hidden-class oracle

The hidden three-class answer key learns one empirical resolver leader per hidden class. It is an upper-bound diagnostic and is never available to induction.

### Random partition

Training and holdout observations are independently assigned to the same number of groups as induced routing. Training groups learn empirical resolver leaders; holdout groups route by random membership.

### Permuted-feature search

Every residual dimension and persistence value is independently permuted across observations. The exact same proposal, promotion, and induced-routing algorithm is rerun. Resolver outcomes remain attached to observations, breaking the relation between visible features and response regime.

## Frozen H4 gates

The executable exits nonzero unless all gates pass:

1. at least two concepts are promoted
2. induced mean discharge efficiency is at least `1.25x` undifferentiated
3. induced mean remaining charge is at most `0.75x` undifferentiated
4. induced solve rate improves by at least `20` percentage points (i.e., `0.20` absolute)
5. induced retains at least `0.80x` hidden-class oracle efficiency
6. induced efficiency is at least `1.25x` random partition
7. induced efficiency is at least `1.25x` permuted-feature search
8. every promoted concept has positive holdout gain and minimum support

The seed and gate values are fixed in the probe source. A failure must not be repaired by tuning gates against the same seeded final result.

## Report

`h4_ontology_induction_probe` emits `h4-ontology-induction-report.json` with:

- fixed seed
- visible charge kind
- train and holdout observation counts
- top one-split training candidates
- promoted opaque concept IDs and predicates
- training and holdout support
- holdout and aggregate utility gain
- learned resolver leader for each promoted concept
- post-hoc dominant hidden class and purity fraction
- metrics for induced, oracle, random, permuted, and undifferentiated policies
- every gate ratio and margin
- individual criterion booleans
- overall pass/fail

Hidden labels are used for the oracle and post-hoc concept interpretation only. They do not enter candidate generation, promotion, or induced routing.

## CPU verification

The existing `charge-contract` workflow now:

1. compiles all Star targets
2. runs deterministic CHARGE tests
3. runs the real-component CHARGE specialization probe
4. runs H4 ontology induction
5. preserves the compiler log, real-component report, and H4 JSON report

## Claim boundary if H4 passes

A passing H4 result supports this limited claim:

> Starting from an intentionally undifferentiated CHARGE class, Starfire can use empirical resolution history to induce and retain opaque, executable distinctions that transfer to held-out observations and improve CHARGE resolution under a fixed compute budget.

It does not establish:

- human-like concept semantics
- language grounding
- autonomous scientific discovery
- unrestricted ontology growth
- AGI
- consciousness

## Next falsification pressure

H4 uses deliberately separable regimes as a proof of mechanism. Passing it should immediately lead to harder tests:

1. overlapping response regimes
2. drifting regime boundaries
3. trace and persistence predicate composition
4. recursive specialization beneath a promoted concept
5. multi-seed or bootstrap membership-stability gates
6. induction over real subsystem-backed CHARGE histories
7. `Merge` after a formerly useful distinction becomes redundant
8. `Relate` and `Reify` over repeated temporal concept-transition structure
