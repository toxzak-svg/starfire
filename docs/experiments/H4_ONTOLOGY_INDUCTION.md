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

The experiment exposes one coarse charge kind to ontology search:

```text
Unresolved
```

The implemented probe encodes this as `ChargeKind::Custom("unresolved")` so production CHARGE kinds remain unchanged until a distinction earns promotion.

The proposal mechanism observes only:

- residual vector values
- magnitude
- persistence
- coarse scope class
- resolver trace
- measured resolver outcomes and declared compute cost

It does not observe hidden source/emitter identity, human charge-kind label, component semantic role, fixture topic name, or oracle-best resolver.

## Hidden answer key

The H4 probe generates three hidden response regimes modeled after the real-component CHARGE specialization result:

1. gap-like observations whose strongest resolver is memory
2. contradiction-like observations whose strongest resolver is reasoning
3. residual-like observations whose strongest resolver is causal

All three are presented to ontology induction as `Unresolved`.

The hidden labels exist only for post-hoc diagnostics and hidden-class oracle evaluation. They do not participate in proposal generation, induced routing, proposal scoring, or promotion.

## Ontology mutation vocabulary

The foundation exposes six structural mutation descriptions:

- `Split`: replace one concept with two candidate partitions
- `Merge`: remove a distinction that no longer earns utility
- `Abstract`: introduce a parent over repeated common structure
- `Specialize`: add a narrower executable predicate beneath a concept
- `Relate`: record a predictive relation between induced concepts
- `Reify`: turn a repeated relation pattern into a candidate object

H4 currently exercises split-like threshold induction. The remaining operators are represented so later experiments do not have to redesign concept identity or promotion.

## Implemented candidate search

The first proposal search is deterministic and replayable.

For every residual dimension, the probe:

1. sorts observed training values
2. generates midpoint thresholds between adjacent unique values
3. evaluates both `>=` and `<=` directions
4. rejects either partition below minimum support
5. scores the split by weighted best-resolver outcome gain over the undifferentiated parent
6. subtracts a fixed complexity penalty
7. removes duplicate membership partitions
8. ranks remaining proposals by training gain
9. sends only the top-K proposals to holdout promotion

No LLM-generated labels or free-form code predicates are used.

## Machine-induced concept representation

A promoted concept is an opaque ID plus evidence:

```text
ConceptId(1)
    parent: None
    predicate: ResidualThreshold { ... }
    observations: N
    holdout_gain: measured independently
    routing_gain: ...
    discharge_gain: ...
    generation: G
```

The concept has no required natural-language name.

Its operational meaning is the cohort selected by its predicate and the measured utility of possessing that distinction.

## Promotion contract

A candidate distinction does not become active merely because proposal search found it.

`OntologyInducer` requires:

1. minimum observation support
2. minimum positive held-out gain
3. minimum positive aggregate utility gain

The H4 probe additionally rejects undersupported partitions during proposal search and deduplicates equivalent memberships before holdout evaluation.

## Implemented policy comparison

All policies use the same one-unit resolver compute budget per observation.

### Undifferentiated

All observations route under the single `Unresolved` parent using the strongest global empirical resolver profile learned on training observations.

### Induced

The two strongest promoted predicates specialize resolver leaders. Matching is deterministic and ordered by held-out promotion gain. Unmatched observations fall back to the parent resolver.

### Oracle hidden classes

The three hidden source classes use their empirically strongest training resolver. This is an upper-bound diagnostic, not a deployable policy.

### Random partition control

Training and holdout observations are assigned to the same number of routing groups randomly. Each training group learns its strongest empirical resolver.

### Permuted-feature control

Residual dimensions and persistence are independently permuted across observations before proposal search and induced-policy evaluation.

## Frozen primary gates in the probe

The process exits nonzero unless all gates pass:

1. at least two concepts are promoted
2. induced mean discharge efficiency beats undifferentiated by at least 1.25x
3. induced mean remaining charge is at most 75% of undifferentiated
4. induced solve rate improves by at least 20 percentage points
5. induced retains at least 80% of hidden-class oracle efficiency
6. induced beats random-partition efficiency by at least 1.25x
7. induced beats permuted-feature efficiency by at least 1.25x
8. every promoted concept has positive holdout gain and minimum support

The current experiment uses a fixed deterministic seed and fixed thresholds in source. A failing final run must not be repaired by post-hoc threshold tuning against the same result.

## Current diagnostic output

`h4_ontology_induction_probe` emits a JSON report containing:

- train and holdout observation counts
- promoted opaque concept IDs and executable predicates
- holdout support and gain
- aggregate utility gain
- dominant hidden class for post-hoc interpretation
- dominant-class fraction
- resolver leader per promoted concept
- policy metrics for induced, oracle, random, permuted, and undifferentiated routing
- every primary ratio and margin
- gate booleans
- overall pass/fail

The hidden diagnostic labels are serialized only after induced proposal, promotion, and policy evaluation have been fixed.

## CPU verification

`charge-contract` now compiles all Star targets, runs deterministic CHARGE tests, runs the existing real-component specialization probe, and executes H4. The H4 JSON report is archived with CHARGE diagnostics for seven days.

## Claim boundary if H4 passes

A passing result supports this limited claim:

> Starting from an intentionally undifferentiated CHARGE class, Starfire can use empirical resolution history to induce and retain opaque, executable distinctions that transfer to held-out observations and improve CHARGE resolution under a fixed compute budget.

It does not show:

- human-like concept semantics
- language grounding
- autonomous scientific discovery
- unrestricted ontology growth
- AGI
- consciousness

## Next experiments

H4 deliberately starts with a small threshold vocabulary. Follow-up experiments should challenge the mechanism rather than simply making the synthetic regimes easier:

1. use overlapping and drifting response regimes
2. add trace/persistence predicate composition
3. require recursive specialization of an already promoted concept
4. add bootstrap or multi-seed membership stability gates
5. run induction over real subsystem-backed CHARGE histories instead of generated response regimes
6. test `Merge` by making a previously useful distinction become redundant
7. test `Relate` and `Reify` on repeated temporal concept-transition structure
