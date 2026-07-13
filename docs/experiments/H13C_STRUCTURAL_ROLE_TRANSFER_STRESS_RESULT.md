# H13-C — Structural Role Transfer Stress Result

Status: **PASS**  
Program alias: **R1-A**  
Preregistration commit: `92f2cdeaf3ad4cba85b61a5443c0d470092413ab`  
First complete verdict-producing head: `909cfcdc1a050766ff72d27c085646b9cd443881`

## Terminal classification

```text
PASS
```

This is the first complete verdict-producing execution of the frozen H13-C contract. The earlier workflow run stopped at formatting before compilation and therefore produced no scientific verdict.

## Frozen cohort

```text
development graphs: 8
holdout graphs:     8
future graphs:     56
future families:    7 × 8 deterministic roots
```

The development H12 role contained exactly `8` members. The H13-C validator independently revalidated the source H12 proof and recomputed the transport signature from all `8` development members.

## Primary transfer result

```text
holdout exact role: 8/8
future exact role: 56/56
future false-positive members: 0
```

Every future transformation family achieved `8/8` exact recognition:

```text
bounded partial observation:     8/8
combined stress:                 8/8
higher distractor density:       8/8
irrelevant branch expansion:     8/8
local-degree change:             8/8
path subdivision:                8/8
vocabulary permutation:          8/8
```

## Controls

```text
local-degree decoy selections:        0/56
two-hop motif decoy selections:       0/56
rewired-control selections:           0/56
vocabulary-only control selections:   0/56
vocabulary relabel invariance:        56/56
valid irrelevant-role target hits:    0/56
root-seeded random exact successes:   0/56
oracle successes:                     56/56
foreign certificate rejected:         true
tampered proof rejected:              true
delayed admission rejected:           true
```

## Replay, authority, and provenance

```text
byte-identical full replay:       true
source corpus immutable:          true
shadow authority invariants:      true
Live transport state available:   false
runtime routing authority:        none
PECS mutation authority:          none
automatic promotion authority:    none
autonomous action authority:      none
```

## Budget result

The probe performed `176` graph-budget checks across primary evaluations and controls. Every check passed.

```text
node signature evaluations:       27,664
reachability edge traversals:  5,573,408
candidate matches:                   120
graph budget passes:             176/176
```

Per-graph frozen limits were:

```text
node signature evaluations <= node_count
reachability traversals <= 32 * (node_count + edge_count)^2
candidate matches <= node_count
```

## Evidence

```text
workflow: H13-C Structural Role Transfer Stress
run number: 2
run id: 29227444421
head: 909cfcdc1a050766ff72d27c085646b9cd443881
conclusion: success
artifact id: 8270287724
artifact digest: sha256:2090190134346f49324f216977f089c65d800bea1102133763bfc11a134c609a
```

Preserved artifact contents:

```text
h13c-rustfmt.patch
h13c-structural-transfer-tests.log
h13c-structural-role-transfer-report.json
target/h13c-structural-role-transfer-report.json
```

The machine-readable report records every frozen gate as `true` and the terminal classification as `PASS`.

## Supported claim

This result supports the bounded statement that a proof-carrying structural role learned from small development graphs can be transported through an independently validated global gateway signature and recognized exactly in the frozen larger, denser, partially altered synthetic graphs while the named superficial, foreign, tampered, unadmitted, random, and irrelevant-role controls fail.

## Claim boundary

This result does **not** establish:

```text
unrestricted ontology learning
open-world semantic equivalence
natural-language concept acquisition
causal abstraction outside the frozen graph regime
automatic concept promotion
safe live self-modification
AGI or human-level cognition
```

The implementation remains shadow-only. The next representation-invention gate is the separately preregistered bounded grammar-extension experiment, ΩG1.
