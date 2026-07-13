# H12-C Literal Control-Conformance Probe

Status: **frozen preregistration before implementation and before the first verdict-producing H12-C run**.

H12-C does not reopen, relabel, or rescue the historical H12 experiment. H12 previously produced an accepted executable result, but its `random_grouping` control was not the literal root-seeded, same-cardinality random partition described in the preregistration. R0-A subsequently consolidated the exact shadow-only latent-role substrate onto `main` without making a new scientific claim.

H12-C asks only whether the consolidated primary mechanism retains its prior bounded behavior while the missing literal control and additional conformance controls are implemented exactly.

## Research question

Under the unchanged H12 structural fingerprint, role induction, independent validation, shadow registry, transfer recognition, evidence projection, H11 inference, H10 validation, and H9 closure path:

1. does the validated target role still expose the useful `middle -> goal` relation on every frozen root;
2. do literal root-seeded random partitions of raw discovery members fail to reconstruct or substitute for that role;
3. do matched structural, proof, timing, and payload controls remain unable to satisfy the target objective;
4. do valid but irrelevant abstractions validate and admit while remaining causally irrelevant?

A positive result is only a control-conformance result for the bounded synthetic H12 mechanism.

## Frozen substrate

H12-C uses the R0-A module already merged on `main`:

```text
lib/latent_roles.rs
```

The experiment must not change:

```text
StructuralFingerprint fields
role recurrence gates
role-id derivation
proof-id derivation
independent validator semantics
registry status semantics
transfer recognition semantics
role-conditioned projection semantics
H11 frontier discovery
H10 scoring or validation gates
H9 PECS closure semantics
```

If a substrate defect prevents execution, classify `INFRASTRUCTURE_FAILURE`; do not alter the frozen experiment to obtain a favorable scientific result.

## Frozen cohort

Exactly seven graph families with eight unique roots each:

```text
training: 2 families = 16 roots
holdout:  1 family   = 8 roots
future:   4 families = 32 roots
total:                 56 roots
```

Every root contains:

```text
4 discovery graphs with opaque node identities
1 held-out transfer graph
1 recurring target structural role
1 recurring valid irrelevant structural role
1 degree-matched transfer decoy
mixed intervention evidence
initial PECS fact source
initial PECS rule source -> middle
external objective Fact(goal)
```

The useful target role has exactly four discovery members, one in each discovery graph. Its transfer graph has exactly one matching member: `middle`.

The valid irrelevant role also has four discovery members and one transfer member, but its projected relation cannot derive `goal`.

## Literal root-seeded random partition

For each root and each frozen random seed index:

1. enumerate the complete raw discovery-member universe directly from the corpus;
2. sort members canonically by `(graph_id, node)`;
3. derive a 64-bit seed from the frozen base seed, root id, and seed index;
4. apply deterministic Fisher–Yates using a frozen SplitMix64 generator;
5. partition the entire shuffled universe into consecutive chunks;
6. use chunk width equal to the independently recomputed target-role member count;
7. sort each chunk canonically before validation;
8. give every complete chunk a full independent role-validation attempt.

Frozen constants:

```text
base seed:          0x4831_3243_5f52_4e44
random seed count:  8 per root
chunk cardinality:  exact target proof member count, expected 4
```

A random candidate carries the target proof's declared fingerprint, role id, and proof id but substitutes the random chunk and its exact supporting graph ids. This deliberately gives the random control the strongest possible certificate-shaped payload. The independent validator must recompute the entire corpus and reject every non-identical membership claim.

A random chunk that exactly reconstructs the target proof is not discarded or resampled. It counts as random-control recovery and causes `CONTROL_FAILURE`.

No seed, chunk, root, or result may be removed after observing the run.

## Frozen paths and controls

Every root executes the following paths twice from fresh state.

### 1. Primary validated target role

```text
induce all recurring roles
independently validate all proofs
register and admit target certificate in shadow registry
recognize target member in transfer graph
project evidence through target role
run unchanged H11 proposal and independent validation
admit resulting H11 certificate before closure
execute exactly 3 closure scans
```

Required objective: `Fact(goal)` exists.

### 2. Literal same-cardinality random partitions

Run all eight complete seeded partitions and independently validate every complete chunk. No random candidate may affect projection, H11, or PECS without a valid certificate.

Required result:

```text
0 target-proof reconstructions
0 target certificate admissions
0 objective successes
```

### 3. Degree-matched grouping

Project the evidence for a transfer node with the same local `in_degree == 1` and `out_degree == 1` as the target member but a different deeper fingerprint. Give the resulting evidence the unchanged H11 validation and PECS admission opportunity.

Required result: the control may validate and admit an irrelevant rule, but objective success is `0/56`.

### 4. Membership-count-matched grouping

Construct a deterministic non-random candidate containing exactly the target member count while substituting structurally unrelated discovery members. Run full independent role validation.

Required result: rejection `56/56`; objective success `0/56`.

### 5. Adversarial mixed target/distractor grouping

Construct a candidate with the exact target cardinality containing both genuine target members and structurally incompatible distractor members. Run full independent role validation.

Required result: rejection `56/56`; objective success `0/56`.

### 6. Valid irrelevant structural role

Independently validate and shadow-admit the recurring irrelevant role, recognize it in transfer, project evidence, obtain an H11 certificate when available, admit it before closure, and execute the same three scans.

Required result:

```text
role validation 56/56
role shadow admission 56/56
H11 executable admission 56/56
objective success 0/56
```

### 7. Foreign proof

Use a genuine validated certificate from a different discovery scope.

Required result: scope-bound rejection `56/56` before transfer projection scans or PECS mutation.

### 8. Membership tampering

Substitute one member in the exact target proof and run full independent validation.

Required result: rejection `56/56` after complete corpus recomputation.

### 9. Delayed admission

Complete target role validation, transfer projection, and H11 validation, but admit the correct H11 certificate only after the three-scan closure window.

Required result:

```text
objective during window 0/56
eventual post-window admission 56/56
```

### 10. Payload-only control

Preserve serialized target proof and certificate metadata but provide no role admission, evidence projection, H11 certificate admission, or executable state mutation.

Required result: objective success `0/56`.

## Independent validation

The role proposer and validator remain separate calls. The validator must independently recompute:

```text
corpus validity
all node fingerprints
all exact fingerprint groups
candidate recurrence
role id
scope digest
proof id
exact sorted membership
supporting graph ids
```

The H11 proposer and validator remain separate calls and independently recompute the full candidate frontier and scoring table.

No proposer may issue its own trusted certificate.

## Frozen computation accounting

For every root and path, record:

```text
corpus node count
role discovery fingerprint evaluations
role validation fingerprint evaluations
random candidate count
random validation fingerprint evaluations
transfer fingerprint evaluations
projection evidence scans
H11 frontier scans
H10 candidate/episode evaluations
H11 certificate admissions
closure scans
```

Exact requirements:

```text
one full role-discovery scan per fresh execution
each role proof validation scans every corpus node
each random chunk validation scans every corpus node
each forged membership validation scans every corpus node
each role projection scans every transfer node and every evidence episode
foreign-scope rejection performs zero transfer-node and evidence scans
all executable paths receive exactly one H11 admission slot
all paths execute exactly three closure scans
```

Structural absence is reported rather than padded with fabricated trusted candidates.

## Replay requirement

Every root/path executes twice from fresh state.

Exact equality is required for:

```text
proof ordering
certificate identities
random partition membership and ordering
rejection fields
budget counters
PECS admission count
final canonical state signature
objective result
terminal classification inputs
```

## Frozen PASS gates

H12-C is `PASS` only when all conditions hold:

```text
cohort exactly 56 roots
primary success training 16/16
primary success holdout 8/8
primary success future 32/32
primary success 8/8 in each future family
literal random seed count exactly 8 per root
literal random candidate cardinality exact everywhere
literal random target reconstructions 0
literal random target admissions 0
literal random objective successes 0
membership-count matched rejection 56/56
adversarial mixed rejection 56/56
degree-matched objective success 0/56
valid irrelevant role validation 56/56
valid irrelevant role shadow admission 56/56
valid irrelevant H11 admission 56/56
valid irrelevant objective success 0/56
foreign proof rejection 56/56
membership tamper rejection 56/56
delayed objective success during window 0/56
delayed eventual admission 56/56
payload-only objective success 0/56
all declared budgets exact
all fresh-state replays exact
all state/provenance invariants hold
no Live registry state exists
no Runtime::chat() influence exists
no automatic ontology promotion exists
```

## Terminal classification

Allowed classifications:

```text
PASS
REJECTED
CONTROL_FAILURE
INFRASTRUCTURE_FAILURE
INCONCLUSIVE
```

Classification order:

1. any compile, execution, budget, replay, provenance, or invariant failure -> `INFRASTRUCTURE_FAILURE`;
2. primary mechanism fails any frozen split or future-family gate -> `REJECTED`;
3. any random, matched, adversarial, irrelevant, foreign, tamper, delayed, or payload control violates its gate -> `CONTROL_FAILURE`;
4. all gates true -> `PASS`;
5. any state not covered above -> `INCONCLUSIVE`.

The first complete verdict-producing run is terminal for this frozen experiment.

## Claim boundary

A `PASS` supports only:

> The consolidated H12 proof-carrying latent-role mechanism retained its bounded synthetic behavior while the previously missing literal root-seeded same-cardinality random-partition control and the additional frozen conformance controls failed to recover the target effect under exact replay and accounting.

A `PASS` does not establish unrestricted ontology induction, natural-language concept formation, causal abstraction, grammar invention, open-set ontology management, automatic promotion, safe live routing, AGI, consciousness, or human-level cognition.

## Authority boundary

H12-C adds no:

```text
Live registry state
Runtime::chat() integration
response routing
persistent production ontology mutation
autonomous action authority
automatic concept promotion
```

All representations remain research-executable or shadow-only.