# H4 — Latent Distinction Induction from Undifferentiated CHARGE

## Question

Can Starfire begin with a deliberately insufficient ontology, discover that one routing class hides multiple stable response regimes, propose a reusable distinction, and retain that distinction only when it improves held-out CHARGE resolution?

This is an ontology-induction proof-of-mechanism experiment. It is not a consciousness, AGI, semantic-understanding, or open-ended self-improvement claim.

## Motivation

The existing CHARGE real-component specialization probe gives Starfire fixed human-authored charge kinds and scope classes. The router is role-blind, but the ontology is not: `EpistemicGap`, `Contradiction`, and `PredictionResidual` already exist before learning begins.

H4 removes that answer key.

The core hypothesis is that persistent routing inefficiency contains enough evidence to justify a new computational distinction. The system does not need to name the distinction in English. It needs to earn an opaque concept ID whose executable predicate transfers to unseen observations and improves a predeclared objective.

## Null hypothesis

A proposal mechanism operating over CHARGE residuals, persistence, and resolver traces will overfit training history, produce unstable partitions, or fail to improve held-out discharge efficiency relative to the undifferentiated routing baseline.

## Starting ontology

The experiment must expose one coarse charge kind to ontology search:

```text
Unresolved
```

Implementation may initially encode this as `ChargeKind::Custom("unresolved")` to avoid changing the production `ChargeKind` enum before the experiment earns the distinction.

The proposal mechanism may observe only:

- residual vector values
- magnitude
- persistence
- coarse scope class
- resolver trace
- measured resolver outcomes and declared compute cost

It must not observe the hidden source/emitter identity, human charge-kind label, component semantic role, fixture topic name, or oracle-best resolver.

## Hidden answer key

Generate observations from the same three response regimes already exercised by the real-component CHARGE probe:

1. knowledge-gap-like observations whose strongest resolver is memory
2. contradiction-like observations whose strongest resolver is reasoning
3. trajectory-residual-like observations whose strongest resolver is causal

All three are presented to ontology induction as `Unresolved`.

The hidden labels exist only for post-hoc diagnostics. They must never participate in proposal generation, routing, scoring, or promotion.

## Ontology mutation vocabulary

The foundation exposes six structural mutation descriptions:

- `Split`: replace one concept with two candidate partitions
- `Merge`: remove a distinction that no longer earns utility
- `Abstract`: introduce a parent over repeated common structure
- `Specialize`: add a narrower executable predicate beneath a concept
- `Relate`: record a predictive relation between induced concepts
- `Reify`: turn a repeated relation pattern into a candidate object

H4 should begin with `Split` and `Specialize`. The remaining operators are represented now so later experiments do not have to redesign the concept identity or promotion model.

## Candidate predicates

The first proposal search must remain deterministic and replayable. Search only over the CHARGE-native predicate vocabulary in `charge::ontology`:

- residual dimension threshold (`>=` or `<=`)
- persistence range
- resolver-trace membership
- boolean `AND`, `OR`, and `NOT`

No LLM-generated labels or free-form code predicates are allowed in H4.

A simple first search is acceptable:

1. enumerate residual dimensions
2. derive candidate thresholds from training quantiles
3. evaluate one-dimensional splits
4. compose the best surviving predicates with persistence or trace conditions
5. rank by training gain with an explicit complexity penalty
6. send only top-K proposals to holdout evaluation

## Machine-induced concept representation

A promoted concept is an opaque ID plus evidence:

```text
ConceptId(8472)
    parent: ConceptId(...)?
    predicate: executable CHARGE selector
    observations: N
    holdout_gain: measured independently
    routing_gain: ...
    prediction_gain: ...
    discharge_gain: ...
    compute_gain: ...
    recurrence_reduction: ...
    generation: G
```

The concept has no required natural-language name.

Its operational meaning is the cohort selected by its predicate and the measured utility of possessing that distinction.

## Promotion contract

A candidate distinction must not become part of the active routing ontology merely because proposal search found it.

`OntologyInducer` provides a shared promotion gate. H4 must predeclare exact thresholds before the final run.

At minimum promotion requires:

1. minimum observation support
2. positive held-out gain
3. positive aggregate utility gain

The final H4 probe should also reject:

- empty partitions
- severely imbalanced partitions below minimum support
- duplicate predicates with equivalent membership
- concepts whose gain disappears under hidden-label permutation control
- concepts that improve training but regress held-out charge remaining

## Experimental policies

Evaluate at least these policies under the same compute budget:

### Undifferentiated

All observations route under the single `Unresolved` class using the strongest global empirical resolver profile.

### Induced

Promoted opaque concepts may specialize resolver profiles. Unmatched observations fall back to the undifferentiated parent.

### Oracle hidden classes

The original three hidden source classes select their empirically strongest resolver. This is an upper-bound diagnostic, not a deployable policy.

### Random partition control

Assign observations to the same number and approximate sizes of classes as the induced ontology, but randomize membership.

### Permuted-feature control

Run the same proposal search after independently permuting residual dimensions, persistence, and trace features across observations.

### Training-only overfit control

Report the best training proposal even if it fails promotion, making the train/holdout gap visible.

## Predeclared primary gates

The initial implementation should choose exact numerical thresholds from a pilot and freeze them before the final falsification run. The final process must exit nonzero unless all gates pass.

Recommended gate shape:

1. at least two non-parent concepts are promoted
2. induced mean discharge efficiency beats undifferentiated by at least 1.25x
3. induced mean remaining charge is at most 75% of undifferentiated
4. induced solve rate improves by at least 20 percentage points
5. induced retains at least 80% of hidden-class oracle efficiency
6. induced beats random-partition efficiency by at least 1.25x
7. induced beats permuted-feature efficiency by at least 1.25x
8. every promoted concept has positive holdout gain and minimum support
9. promoted membership is stable across at least 80% of bootstrap resamples or deterministic environment seeds
10. CHARGE accounting remains within the existing ledger tolerance

These numbers are recommendations until the pilot protocol is committed. Do not tune them after observing the final seeded result.

## Diagnostic question: did it rediscover the hidden regimes?

After all routing and promotion decisions are frozen, compare induced concept membership with the hidden three-class answer key.

Report:

- adjusted Rand index
- normalized mutual information
- induced concept × hidden class contingency table
- dominant hidden class per concept
- resolver leader per induced concept

Passing H4 must not require semantic equivalence to human labels. A novel partition may be better than the answer key. The primary claim is held-out computational utility.

However, substantial agreement with the hidden regimes would be strong evidence that Starfire independently recovered distinctions similar to the human-authored CHARGE categories.

## Required artifacts

The CI workflow should preserve:

- `h4-ontology-induction-report.json`
- candidate proposal table
- promoted concept definitions and predicates
- train and holdout memberships by opaque observation ID
- policy metrics
- hidden-label diagnostics produced only after evaluation
- compiler/test log

## Claim boundary if H4 passes

A passing result would support this claim:

> Starting from an intentionally undifferentiated CHARGE class, Starfire can use empirical resolution history to induce and retain opaque, executable distinctions that transfer to held-out observations and improve CHARGE resolution under a fixed compute budget.

It would not show:

- human-like concept semantics
- language grounding
- autonomous scientific discovery
- unrestricted ontology growth
- AGI
- consciousness

## Implementation sequence

1. Land the ontology representation and promotion contract.
2. Add a deterministic observation record containing only H4-visible features and outcomes.
3. Build one-dimensional residual threshold proposal search.
4. Add split scoring with complexity penalty and minimum support.
5. Add holdout promotion and parent fallback routing.
6. Add random-partition and permuted-feature controls.
7. Add hidden-label diagnostics after policy evaluation.
8. Freeze pilot-derived gates in the experiment source.
9. Add a CPU CI workflow step and archive diagnostics.
10. Run the final seeded falsification probe without adjusting gates.

The first code foundation in this branch completes step 1. It deliberately does not claim that proposal search or spontaneous distinction formation already exists.
