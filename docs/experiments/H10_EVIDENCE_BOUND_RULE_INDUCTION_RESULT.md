# H10 Evidence-Bound Rule Induction — Frozen Result

Status: **PASS**

This document records the first complete verdict-producing H10 execution. It does not change the frozen mechanism, evidence table, scoring rule, certificate gates, cohort, controls, budgets, acceptance gates, or classification logic.

## Provenance

```text
preregistration commit: 8d58e8675322810217ee9e9209a5c6c9ceace4a7
first verdict head:      30017e432d1af23441972dd6f5952d5a9bfd18e7
CHARGE CI run:           29143574436 (#94)
workflow conclusion:     success
artifact id:             8245997006
artifact digest:         sha256:b77af5b66ac33ae0bc025e1c14c965d7b90f72089796689a9ce67d580863eded
```

The preregistration preceded implementation and the first result. No H10 scientific constant was changed after observing this run.

## Terminal classification

```text
PASS
```

## Frozen objective results

```text
                           training  holdout  future
stateful inferred commitment          16/16     8/8    32/32
endpoint blind                         0/16     0/8     0/32
text-only proposal                     0/16     0/8     0/32
scalar-only proposal                   0/16     0/8     0/32
rewired foreign proof                  0/16     0/8     0/32
valid permuted irrelevant inference    0/16     0/8     0/32
counterfeit proof                      0/16     0/8     0/32
delayed correct admission              0/16     0/8     0/32
```

## Proof-validation controls

```text
foreign proof rejection:
  training 16/16
  holdout   8/8
  future   32/32

counterfeit proof rejection:
  training 16/16
  holdout   8/8
  future   32/32

valid permuted certificate acceptance:
  training 16/16
  holdout   8/8
  future   32/32
```

The valid-permuted path is important: it performed a successful inference, passed independent proof recomputation, admitted one executable rule, and still achieved `0/56` objective successes because the admitted rule was causally irrelevant to the held-out objective.

## Future-family transfer

```text
biology:       stateful 1.0, maximum control 0.0
manufacturing: stateful 1.0, maximum control 0.0
software:      stateful 1.0, maximum control 0.0
hydrology:     stateful 1.0, maximum control 0.0
```

## Frozen compute and replay contract

Every root/path satisfied:

```text
inference calls                      = 1
candidate rules scored               = 9
evidence episodes per candidate      = 10
proposal scoring evaluations         = 90
proof-validation recomputations      = 1
validation scoring evaluations       = 90
admission slots                      = 1
executor scans                       = 3
independent objective checks         = 1
```

All reported budgets were exact.

Every root/path was executed twice from a freshly reconstructed state. Exact replay passed for every path in training, holdout, and future splits, including canonical executable-state signatures.

All executable-state and inference-provenance invariants held.

## What H10 established

Under the frozen symbolic intervention-evidence regime, the exact target bridge was not supplied as a raw witnessed rule.

Instead:

1. a target-blind proposer scored all nine candidate rules from ten intervention episodes;
2. the proposer emitted an auditable proof;
3. an independent validator recomputed the complete candidate ranking from raw evidence;
4. only a valid opaque certificate was admitted into executable commitment state;
5. the admitted rule changed the reachable closure of the later fixed-budget executor;
6. removing executable admission, preserving only text or scalars, rewiring proof incidence, admitting an irrelevant but valid inferred rule, counterfeiting the proof, or applying the correct rule too late did not recover the objective.

The narrow supported claim is therefore:

> Starfire's H10 shadow substrate can infer a useful executable rule from non-privileged symbolic intervention evidence, independently validate the proof, and use the resulting commitment as a causally necessary intermediate computational state across unseen domain vocabularies.

## What H10 did not establish

H10 does not establish:

```text
open-world causal discovery
natural-language evidence extraction
learning the scoring law
learning the certificate gates
unbounded candidate generation
new operator invention
automatic ontology induction
live-routing readiness
AGI
consciousness
human-level cognition
```

## Strongest remaining limitation

The evidence schema, candidate universe size, scoring law, and certificate thresholds are still developer-defined and fixed.

H10 demonstrates **selection and validation of a useful rule from non-privileged evidence**, not autonomous invention of the representational vocabulary or induction law itself.

The next narrow experiment should therefore attack one of those remaining privileged boundaries without relaxing H10's causal controls. The strongest candidate is:

> infer the candidate relation vocabulary and rule proposal from a larger mixed evidence graph in which the useful antecedent/consequent pair is not supplied as a three-by-three candidate universe, while preserving independent proof recomputation and PECS admission.
