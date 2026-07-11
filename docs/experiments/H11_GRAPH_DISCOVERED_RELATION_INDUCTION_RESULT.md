# H11 Graph-Discovered Relation Induction — Frozen Result

Status: **PASS**

This document records the first complete verdict-producing H11 execution. It does not change the frozen graph shape, frontier eligibility rule, H10 scoring law, certificate gates, cohort, paths, controls, budgets, acceptance gates, or classification logic.

## Provenance

```text
preregistration commit: 3d22a43e595a547738e4964d1c57fa4fac360286
first verdict head:      667fff708d06c09b15d9b4965c53b38374c640f8
CHARGE CI run:           29143962558 (#96)
workflow conclusion:     success
artifact id:             8246133213
artifact digest:         sha256:fe5afd3070ed0789d7caece18602cce0fee60f4d10045883b07f21adbf273000
```

The preregistration preceded implementation and the first result. No H11 scientific constant was changed after observing this run.

## Terminal classification

```text
PASS
```

## Frozen objective results

```text
                             training  holdout  future
stateful graph-discovered inference      16/16     8/8    32/32
endpoint blind                             0/16     0/8     0/32
frontier/proof text only                   0/16     0/8     0/32
scalar only                                0/16     0/8     0/32
foreign proof                              0/16     0/8     0/32
frontier-tampered proof                    0/16     0/8     0/32
valid irrelevant graph discovery           0/16     0/8     0/32
counterfeit proof                          0/16     0/8     0/32
delayed correct admission                  0/16     0/8     0/32
```

## Discovery and proof-validation controls

```text
foreign proof rejection:
  training 16/16
  holdout   8/8
  future   32/32

frontier-tampered proof rejection:
  training 16/16
  holdout   8/8
  future   32/32

counterfeit proof rejection:
  training 16/16
  holdout   8/8
  future   32/32

valid irrelevant certificate acceptance:
  training 16/16
  holdout   8/8
  future   32/32
```

The valid-irrelevant path is a strong causal control: on every root it independently discovered a six-by-six frontier, inferred a unique evidence-supported rule, passed full proof recomputation, admitted one executable rule, and still achieved `0/56` objective successes because the admitted relation was irrelevant to the held-out objective.

The frontier-tamper path is separately important: changing only one claimed discovered frontier atom while preserving the otherwise correct inferred rule and scalar evidence statistics was rejected on every root after the validator independently rescanned the graph and recomputed all 576 candidate/episode scores.

## Future-family transfer

```text
cellular_regulation:       stateful 1.0, maximum control 0.0
manufacturing_processes:   stateful 1.0, maximum control 0.0
software_dependency:       stateful 1.0, maximum control 0.0
watershed_dynamics:        stateful 1.0, maximum control 0.0
```

## Frozen discovery and compute contract

Every root/path satisfied:

```text
raw graph atoms                         = 24
raw evidence episodes                   = 16
proposer frontier passes                = 1
validator frontier passes               = 1
proposer graph-incidence scans          = 16
validator graph-incidence scans         = 16
discovered antecedents                  = 6
discovered consequents                  = 6
discovered candidate rules              = 36
proposal candidate/episode evaluations  = 576
proof-validation recomputations         = 1
validation candidate/episode evaluations= 576
admission slots                         = 1
executor scans                           = 3
independent objective checks             = 1
```

All reported budgets were exact.

Every root/path was executed twice from a freshly reconstructed state. Exact replay passed for every path in training, holdout, and future splits, including canonical executable-state signatures.

All executable-state and inference-provenance invariants held.

## What H11 established

H11 removed the explicit H10 candidate universe.

The mechanism received a mixed symbolic intervention graph and, without receiving an antecedent candidate list, consequent candidate list, verifier target, hidden correct relation, family label, or split label:

1. counted intervention and outcome incidence from raw graph evidence;
2. constructed a canonical six-by-six relation frontier;
3. evaluated all 36 discovered candidate relations over all 16 evidence episodes;
4. inferred a unique winning rule under the unchanged H10 scoring law;
5. emitted an auditable proof containing the discovered frontier and evidence statistics;
6. independently rediscovered the frontier and recomputed the full ranking;
7. admitted only an opaque validated certificate into executable state;
8. used that admitted rule as a causally necessary intermediate state for a later fixed-budget PECS closure.

The effect was not recovered by preserving the frontier and proof only as text, preserving only scalar frontier/score information, omitting executable admission, supplying a foreign proof, tampering with the claimed frontier, admitting a valid but irrelevant discovered relation, counterfeiting the score, or admitting the correct rule after the executor's search window.

The narrow supported claim is therefore:

> Under the frozen symbolic mixed-graph regime, Starfire's H11 shadow substrate can discover a candidate relation frontier from raw graph incidence without being given the target relation endpoints as a candidate universe, infer a useful rule from that discovered frontier, independently recompute both discovery and evidence ranking, and use validated executable admission as a causally necessary intermediate computational state across unseen vocabularies.

## What H11 did not establish

H11 does not establish:

```text
open-world causal discovery
natural-language evidence extraction
learning the frontier eligibility rule
learning the scoring law
learning certificate thresholds
unbounded relation invention
automatic ontology induction
live-routing readiness
AGI
consciousness
human-level cognition
```

## Strongest remaining limitation

The explicit candidate universe is gone, but the representation and induction laws remain developer-defined.

H11 still receives:

```text
symbolically typed intervention episodes
explicit outcome atom identities
a fixed frontier eligibility rule (incidence >= 2)
the fixed H10 scoring law
fixed certificate thresholds
a finite synthetic graph construction
```

The next narrow experiment should attack the strongest remaining privileged boundary without relaxing H9-H11 causal controls.

The strongest candidate is to replace the fixed frontier eligibility rule with **evidence-derived structural abstraction**: discover reusable latent roles or relation classes from graph motifs across roots, then require those learned abstractions to generate candidate executable relations on held-out and future graph families under frozen matched-budget controls and independent validation.
