# ΩR1 — Representative Transportability

Status: preregistered before implementation and before any verdict-producing run.

## Motivation

Ω2 was rejected after discovering and independently validating a genuine second-generation descendant refinement because the admitted Ω1 ancestor program was not transport-stable.

The Ω1 search correctly identified the target behavioral partition on discovery histories, but its equivalence relation over executable programs was:

```text
P1 ~ P2 iff P1(h) == P2(h) for every observed discovery history h
```

Inside that finite-support equivalence class, Ω1 retained the lexicographically smallest syntax. For the Ω2 ancestor partition, that representative was:

```text
first(A) < count(A)
```

which happened to equal the intended pair-order bit while the `AB` block occupied the first two positions, but collapsed after whole-block permutation.

ΩR1 asks the narrower foundation question:

> Can Starfire distinguish discovery-equivalent executable programs by their behavior over an independently specified structure-preserving transformation orbit, validate a transport-stable representative without transformed outcome labels, and thereby preserve representation-bound prediction on transformations not used during representative selection?

ΩR1 does not rerun or rescue Ω2. It uses a new eight-atom/four-bit fixture, a different candidate-space size, separately frozen calibration and held-out transformations, and a new proof-carrying transport certificate.

## Primary causal contrast

```text
partition-only equivalence
  -> all programs with the same discovery partition are treated as interchangeable
  -> lexicographic representative can encode an accidental absolute-layout relation
  -> withheld structural transformation breaks executable semantics

transformation-orbit equivalence
  -> first identify the same discovery behavioral partition
  -> retain all executable representatives of that partition
  -> compare each representative on same-history structure-preserving transforms
  -> require zero transport violations without using transformed outcomes
  -> independently validate the selected representative and transformation suite
  -> admit only the validated transport-stable executable program
  -> withheld transformations preserve prediction
```

## Frozen cohort

```text
16 training roots
8 holdout roots
32 future roots
4 future domain vocabularies
7 total domain vocabularies
8 roots per family
56 total roots
```

Every path is replayed exactly twice from fresh state.

## Raw history fixture

Each root has eight raw atoms arranged as four ordered pairs:

```text
(A, B) -> latent within-pair order bit p
(C, D) -> q
(E, F) -> r
(G, H) -> s
```

The hidden bits are used only to generate the fixture and objective outcomes. Synthesis, representative transport scoring, validation, executable state, and downstream prediction do not receive `p/q/r/s` labels.

Discovery exhausts the full Boolean cube:

```text
(p, q, r, s) ∈ {0,1}^4
```

for exactly 16 discovery histories per root.

Discovery block order is fixed:

```text
AB | CD | EF | GH
```

Every discovery history therefore has:

```text
same eight raw atoms
same atom counts
same order-blind base multiset key
```

The objective behavior is:

```text
Y = p
```

The intervention is constant within each root.

Under order-blind `L0`, the 8/8 outcome split yields exactly:

```text
64 opposite-outcome alias defects
```

## Unchanged Ω1 raw grammar

ΩR1 uses the unchanged Ω1 raw-program grammar:

```text
metric := FirstIndex(atom) | LastIndex(atom) | Count(atom)
program := metric_a < metric_b | metric_a == metric_b
```

For eight atoms:

```text
24 metrics
828 candidate programs
13,248 program/history executions per complete 16-history search
120 history-pair evaluations
5 unique behavioral partitions under the frozen discovery cube
```

The target `p` partition has:

```text
winner repaired defects = 64/64
runner-up repaired defects = 32/64
winner margin = 32
partition support min = 8
```

Exactly:

```text
72 executable programs
```

produce the winning target partition on the discovery support, up to the existing Boolean-complement partition canonicalization.

The unchanged Ω1 partition-only tie-break selects:

```text
first(A) < count(A)
```

for every root vocabulary.

## Structure-preserving block transformations

A transformation operates only by permuting the four intact two-atom pair blocks.

It never:

```text
changes atom identity
changes atom count
reverses the order inside a pair
reads an outcome
reads a hidden p/q/r/s bit
```

Thus the target within-pair relation `p` is semantically preserved by every admissible transformation.

### Calibration transformation suite

The primary transport-aware path receives exactly two same-history calibration transformations:

```text
T1 block order = [CD, AB, EF, GH] = [1,0,2,3]
T2 block order = [CD, EF, AB, GH] = [1,2,0,3]
```

The target `AB` block therefore moves from discovery position 1 to positions 2 and 3.

No transformed outcome labels are supplied.

For one candidate representative `P` and one original discovery history `h`, the only calibration witness is:

```text
P(T(h)) == P(h)
```

A transport violation is:

```text
P(T(h)) != P(h)
```

### Held-out transformation suite

The objective transfer window uses two transformations that are never used during proposal or validation:

```text
H1 block order = [CD, EF, GH, AB] = [1,2,3,0]
H2 block order = [GH, EF, CD, AB] = [3,2,1,0]
```

The target `AB` block is in the fourth position under both held-out transforms.

The held-out objective therefore contains:

```text
2 transformations × 16 histories = 32 predictions per root
```

## Frozen transport-aware representative search

The proposer:

1. derives the eight-atom vocabulary from discovery histories;
2. enumerates all 828 Ω1 raw programs;
3. executes all 828 programs on all 16 discovery histories;
4. canonicalizes binary discovery partitions up to Boolean complement;
5. scores all five discovery partitions against the 64 independently witnessed opposite-outcome pairs;
6. selects the unique winning behavioral partition;
7. retains all 72 executable programs in the winning discovery-equivalence class;
8. executes every one of those 72 representatives on both calibration transformations for all 16 corresponding histories;
9. counts same-history transport violations;
10. ranks representatives by minimum transport violations, then canonical program string;
11. proposes the unique canonical zero-violation representative.

Frozen primary search counts:

```text
raw candidate programs = 828
discovery program/history executions = 13,248
unique discovery partitions = 5
winning-equivalence-class representatives = 72
calibration transformations = 2
transport representative/history executions = 2,304
zero-violation representatives = 8
minimum transport violations = 0
selected canonical stable representative = first(A) < first(B)
```

The 64 discovery-equivalent but unstable representatives each incur exactly:

```text
16 calibration violations
```

under the frozen moving-target calibration suite.

## Independent validation and certificate binding

The validator independently recomputes:

1. root and problem identity;
2. raw vocabulary;
3. all 828 programs;
4. all 13,248 discovery executions;
5. all five behavioral partitions;
6. all 64 opposite-outcome defects;
7. complete partition ranking;
8. all 72 executable representatives in the winning partition;
9. calibration transformation-suite digest;
10. all 2,304 transport executions;
11. violation counts for every representative;
12. the eight zero-violation representatives;
13. canonical stable representative selection;
14. every proof field.

A validated transport certificate is bound to:

```text
root_id
problem_digest
vocabulary_digest
discovery_winning_partition
calibration_transformation_suite_digest
selected executable program
transport-violation certificate
```

Only the opaque independently validated certificate may add the selected transport-stable program to executable state.

## Executable transport state

The transport state begins with the same order-blind base key:

```text
sorted raw atom multiset + intervention
```

After certificate admission, the executable key gains exactly one Boolean refinement bit produced by the validated program.

The downstream predictor receives only:

```text
executable state key
witnessed discovery outcome associated with that key
```

It cannot inspect:

```text
raw history during prediction logic
proof text
program syntax
transport score
transformation name
family label
split label
hidden p/q/r/s bits
```

If one executable key maps to conflicting discovery outcomes, the predictor abstains.

Expected objective behavior:

```text
no admitted refinement -> discovery key conflict -> 0/32 held-out predictions
transport-stable refinement -> p preserved -> 32/32 held-out predictions
partition-only accidental representative -> collapses under held-out transforms -> 16/32 correct
```

## Frozen controls

Every root executes eight paths from fresh state and every path is replayed exactly twice.

### 1. `orbit_aware_stateful`

```text
complete discovery search
complete moving-target calibration search
independent full validation
admit transport-stable representative
predict on H1 and H2
```

Expected:

```text
selected program = first(A) < first(B)
32/32 held-out predictions
```

### 2. `partition_only_baseline`

Use the unchanged Ω1 synthesis, validation, and `StateLanguage` admission path with no transport-aware representative discrimination.

Frozen expected representative:

```text
first(A) < count(A)
```

Expected held-out result:

```text
16/32 correct
```

The path receives the complete 828-program Ω1 search but no transformation-orbit executions.

### 3. `target_stationary_matched_calibration`

Use the same transport-aware search and exactly the same number of calibration transformations/executions, but the calibration transformations permute only the other three blocks while leaving `AB` first:

```text
S1 = [AB, EF, CD, GH] = [0,2,1,3]
S2 = [AB, GH, EF, CD] = [0,3,2,1]
```

All 72 discovery-equivalent representatives therefore have zero observed transport violations. Lexicographic tie-breaking again selects:

```text
first(A) < count(A)
```

Expected held-out result:

```text
16/32 correct
```

This is the matched-compute control showing that extra execution alone is insufficient; the calibration orbit must actually constrain the relevant transformation degree of freedom.

### 4. `rewired_correspondence_calibration`

Use the same two primary moving-target transformations and the same 2,304 transport executions, but destroy same-history correspondence by comparing each transformed history to the next discovery history in deterministic cyclic order.

Frozen expected representative audit:

```text
minimum transport violations = 4
zero-violation representatives = 0
```

Expected:

```text
no validated certificate
0/32 predictions
```

This control preserves transformation count, transformed-history multiset, candidate class, and compute while destroying the specific correspondence relation required by the invariance claim.

### 5. `transport_payload_only`

Complete correct proposal and independent validation, retain the proof/certificate payload, but do not admit the transport refinement to executable state.

Expected:

```text
0/32 predictions
```

### 6. `counterfeit_transport_proof`

Tamper one structural proof field after correct proposal, then run complete independent validation.

Expected:

```text
full recomputation
validation rejection
0/32 predictions
```

### 7. `foreign_root_transport_certificate`

Validate a correct certificate under another root with the same candidate-space dimensions and attempt admission under the current root.

Expected:

```text
foreign-root admission rejection
0/32 predictions
```

### 8. `delayed_transport_admission`

Complete correct proposal and validation but admit the certificate only after the held-out prediction window.

Expected:

```text
0/32 predictions during window
successful eventual admission after window
```

## Frozen exact budgets

### Discovery search

Every complete eight-atom / sixteen-history search:

```text
vocabulary history scans = 16
history-pair evaluations = 120
candidate programs = 828
program/history executions = 13,248
unique behavioral partitions = 5
```

### Primary moving-target transport search

After the winning discovery partition is fixed:

```text
winning-class representatives = 72
calibration transformations = 2
transport executions = 72 × 2 × 16 = 2,304
zero-violation representatives = 8
selected violations = 0
```

Proposal and validator each recompute the full budget independently.

### Target-stationary matched control

```text
winning-class representatives = 72
calibration transformations = 2
transport executions = 2,304
zero-violation representatives = 72
selected violations = 0
```

### Rewired-correspondence control

```text
winning-class representatives = 72
calibration transformations = 2
transport executions = 2,304
zero-violation representatives = 0
minimum violations = 4
```

### Held-out objective

Every path executes:

```text
discovery key-index passes = 1
held-out transformations = 2
held-out histories per transform = 16
prediction attempts = 32
objective checks = 32
```

A path without an admitted refinement must not fabricate a synthetic state bit.

## Frozen success gates

A `PASS` requires all of:

1. exact `16/8/32` root split and four future families;
2. exactly eight raw atoms and sixteen discovery histories per root;
3. exactly 64 base alias defects per root;
4. exact discovery search counts `828 / 13,248 / 5` everywhere;
5. winning partition `64/64`, runner-up `32/64`, margin `32`, support `8` everywhere;
6. exactly 72 discovery-equivalent representatives in the winning class everywhere;
7. primary moving-target search exact `2,304` transport executions, eight zero-violation representatives, selected violations `0` everywhere;
8. primary selected program is the stable within-pair canonical representative `first(A) < first(B)` under every renamed vocabulary;
9. independent full validation succeeds on every orbit-aware stateful root;
10. orbit-aware stateful training roots each achieve `32/32` held-out predictions;
11. orbit-aware stateful holdout roots each achieve `32/32` held-out predictions;
12. orbit-aware stateful future roots each achieve `32/32` held-out predictions;
13. partition-only baseline selects the frozen accidental representative class and achieves exactly `16/32` per root;
14. target-stationary matched calibration executes the full `2,304` transport budget, leaves 72 zero-violation representatives, selects the accidental representative, and achieves exactly `16/32` per root;
15. rewired correspondence executes the full `2,304` transport budget, has zero zero-violation representatives, validates no certificate, and yields `0/32` predictions;
16. payload-only path validates correct certificates but yields `0/32` predictions;
17. counterfeit proofs are rejected after full recomputation on every root;
18. foreign-root certificates are rejected on every target root;
19. delayed admission yields `0/32` during the frozen window and succeeds afterward on every root;
20. every future family independently achieves orbit-aware success `1.0`;
21. partition-only and target-stationary future success remain exactly `0.0` under the strict `32/32` root-success definition;
22. all declared budgets are exact;
23. exact fresh-state replay holds for every root/path;
24. certificate and executable-state invariants hold everywhere.

Terminal classification:

```text
CONTROL_FAILURE
```

if budget, proof-binding, foreign-root, or invariant controls fail;

```text
REPLAY_FAILURE
```

if exact fresh-state replay fails;

```text
PASS
```

only if every frozen success gate passes;

otherwise:

```text
REJECTED
```

No transformation, fixture, grammar, candidate count, root count, control, budget, tie-break, acceptance gate, or terminal-classification rule may be changed after the first complete verdict-producing run to rescue a failure.

## Required interpretation of PASS

A PASS would support only:

> Under the frozen symbolic pair-block regime, discovery-partition equivalence alone was insufficient to identify a transport-stable executable representation, while independent same-history evaluation over a preregistered structure-preserving calibration orbit distinguished 72 discovery-equivalent representatives, selected a zero-violation within-pair program, and transferred perfectly to block permutations not used during proposal or validation. Matched-compute target-stationary and correspondence-rewired controls failed as predicted, and executable admission remained necessary.

## Claims explicitly not established

A PASS would not establish:

- automatic discovery of the transformation group itself;
- unrestricted symmetry discovery;
- semantic equivalence in open-world environments;
- a solved descendant-necessity chain;
- unrestricted recursive ontology growth;
- natural-language concept formation;
- safe production self-modification;
- AGI;
- consciousness;
- human-level cognition.

## Next step after PASS

Only after ΩR1 passes would it be justified to design a **new** descendant-necessity experiment that uses transport-certified ancestor refinements. That future experiment would not be a rerun or rescue of Ω2.
