# H13-C — Structural Role Transfer Stress

Status: **preregistered before implementation**  
Program alias: **R1-A**  
Parent gates: **R0-A merged; R0-B / H12-C PASS merged**  
Authority: **shadow-only**

## Research question

Can a proof-carrying structural role learned only from small development graphs be transported unchanged into larger, denser, partially altered future graphs when local degree and two-hop motifs no longer identify the role, while superficial and foreign controls remain rejected?

## Hypothesis

A transport certificate derived from independently validated H12 role members can preserve a bounded, target-blind global source-to-sink gateway signature. That signature should identify the same functional role after graph growth, path subdivision, irrelevant branching, vocabulary permutation, local-degree change, and bounded partial observation, while rejecting degree-matched and motif-matched bypass decoys.

A PASS would establish only bounded structural-role transfer under this frozen synthetic regime. It would not establish semantic concept learning, natural-language understanding, unrestricted ontology induction, or production authority.

## Frozen mechanism

The candidate mechanism may use only:

1. the raw development `StructuralCorpus`;
2. an H12 `LatentRoleProof` that must survive the existing independent H12 validator;
3. graph topology and opaque atom identities;
4. a proof-carrying transport signature independently recomputed from every validated development member.

The transport signature is frozen to these target-blind fields:

```text
source_ancestor_bucket          0 | 1 | 2+
sink_descendant_bucket         0 | 1 | 2+
reachable_source_sink_pairs     0 | 1 | 2 | 3 | 4+
mandatory_source_sink_pairs     0 | 1 | 2 | 3 | 4+
all_reachable_pairs_mandatory   bool
```

For a candidate node, source ancestors are zero-in-degree nodes that can reach it. Sink descendants are zero-out-degree nodes reachable from it. A source/sink pair is mandatory through the candidate exactly when removing the candidate destroys all directed paths for that pair.

The signature must not inspect:

```text
target labels
family labels
split labels
node names or lexical similarity
task outcomes
control identities
future transformation names
```

The proposer may emit a proof. Only the independent validator may produce an opaque validated transport certificate.

## Frozen fixture

### Development split

Eight small development graphs across two opaque families. Each graph contains one H12-discoverable role instance. The role is frozen before any holdout or future graph is evaluated.

### Holdout split

Eight unseen medium graphs across one unseen family.

### Future split

Fifty-six larger graphs: seven transformation families × eight deterministic seeds.

The seven future families are:

```text
path subdivision
irrelevant branch expansion
higher distractor density
local-degree change preserving gateway function
bounded partial observation of irrelevant structure
surface vocabulary permutation / isomorphic relabeling
combined transformation stress
```

Every future graph contains:

```text
one oracle functional-role member
one local-degree-matched bypass decoy
one two-hop-motif-matched bypass decoy
additional irrelevant nodes and edges
```

The oracle member set and control identities are retained only by the experiment harness and are never passed to induction, validation, or recognition.

## Frozen controls

1. local-degree-matched bypass decoy;
2. two-hop-motif-matched bypass decoy;
3. degree-preserving edge rewiring that destroys mandatory gateway function;
4. vocabulary-only identity control;
5. foreign-family transport certificate;
6. structurally valid but target-irrelevant H12 role;
7. role-identity permutation / proof tampering;
8. delayed admission: recognition before independent validation;
9. deterministic root-seeded random same-cardinality member selection;
10. oracle role upper bound.

## Frozen budgets

Per graph:

```text
node signature evaluations <= node_count
reachability traversals <= 32 * (node_count + edge_count)^2
candidate matches <= node_count
```

Across development validation:

```text
H12 proof validation is complete, not sampled
all development role members are recomputed
all transport-proof fields are recomputed
no cached proposer result is trusted
```

The executable must report exact observed counts and classify any budget overrun as `INFRASTRUCTURE_FAILURE`.

## Frozen metrics

```text
development proof validation count
holdout exact-role recall
future exact-role recall
false-positive members
local-degree-decoy selections
two-hop-decoy selections
rewired-control selections
vocabulary-control invariance
foreign-certificate rejection
irrelevant-role target selections
tampered-proof rejection
delayed-admission rejection
random-control exact successes
oracle successes
deterministic replay equality
source-corpus immutability
shadow-registry authority invariants
budget conformance
```

## Acceptance gates

All gates are conjunctive.

```text
development H12 proof independently validates
transport proof independently recomputes exactly
holdout exact role: 8/8
future exact role: 56/56
future false positives: 0
local-degree decoy selections: 0/56
two-hop decoy selections: 0/56
rewired control selections: 0/56
vocabulary permutation changes no verdicts
foreign certificate rejected
valid irrelevant role yields 0 target selections
tampered proof rejected
delayed admission rejected
random same-cardinality control exact successes: 0/56
oracle: 56/56
replay byte-identical
input corpus and graphs unchanged
registry contains no Live state and grants no runtime authority
all frozen budgets pass
```

## Terminal classifications

```text
PASS
REJECTED
CONTROL_FAILURE
INFRASTRUCTURE_FAILURE
INCONCLUSIVE
```

Classification order:

1. malformed fixture, incomplete cohort, nondeterminism, budget failure, or failed parent-proof validation → `INFRASTRUCTURE_FAILURE`;
2. a negative control recovers the target effect, a foreign/tampered proof is accepted, or a decoy is selected → `CONTROL_FAILURE`;
3. infrastructure and controls pass but holdout/future transfer misses any required target → `REJECTED`;
4. every conjunctive gate passes → `PASS`;
5. otherwise → `INCONCLUSIVE`.

The first complete verdict-producing CI run is terminal for this frozen experiment. No fixture, field, threshold, budget, control, or gate may be changed to rescue a non-PASS result.

## Authority boundary

H13-C may add a reusable shadow transport module, opaque proof/certificate types, deterministic fixtures, tests, CI, and a frozen result document.

It must not add:

```text
Live registry state
Runtime::chat() integration
response routing
PECS mutation
persistent ontology mutation
automatic concept promotion
autonomous action
self-modification authority
```

## Required implementation sequence

```text
1. this preregistration commit
2. mechanism and independent validator
3. frozen executable and unit tests
4. dedicated CI
5. first complete verdict
6. immutable result document
7. merge only if the exact frozen contract is satisfied
```
