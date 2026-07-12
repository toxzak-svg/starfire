# H9 Proof-Carrying Executable Commitment State

Status: **frozen preregistration before implementation and before the first verdict-producing run**.

H9 is shadow-only. It does not alter live routing, enable automatic ontology promotion, or reinterpret the rejected H4-H7 results. H8 remains scientifically invalid under its original deterministic seeded-replay premise.

## Research question

Can one Starfire operation create a validated executable commitment that is causally necessary for a later bounded operation to reach an independently verified objective, when text, scalar history, operation count, transition budget, search budget, and structurally matched controls are held constant or explicitly accounted for?

## Primary mechanism

H9 tests **Proof-Carrying Executable Commitment State** (PECS).

The state contains:

```text
O = immutable witnessed raw observations
F = executable committed facts
R = executable committed rules
P = provenance over commitments
S = (O, F, R, P)
```

The frozen transition language is exactly:

```text
CompileWitnessedRule(witness_id, exact_rule)
DeriveFact(rule_commitment_id, support_fact_id)
```

A compile transition succeeds only when the proposed rule exactly matches the referenced witness.

A derivation succeeds only when the selected support fact satisfies the selected executable rule antecedent.

Every non-seed commitment carries provenance.

No transcript text is executable state.

## Operation family

The H9 experiment contains two bounded operations.

### Operation A: witnessed causal compilation

Operation A reads one raw `WitnessedRule` and instantiates the existing Starfire `CausalEngine` with that witnessed edge. It then emits one typed `CompileWitnessedRule` delta.

The causal engine call is part of every path's matched operation budget, including controls that later discard, replace, delay, or invalidate the resulting delta.

### Operation B: deterministic executable closure

Operation B cannot read raw observations or transcript text.

It performs exactly three derivation scans. On each scan it obtains the canonically ordered set of currently enabled `DeriveFact` transitions and applies the first one if any exists. An empty scan consumes the same search slot and performs no mutation.

Operation B therefore has a fixed search budget of three scans on every path.

## Task construction

Each root task has unique symbolic atoms:

```text
source
middle
goal
decoy_source
decoy_goal
```

Initial executable state contains:

```text
Fact(source)
Fact(decoy_source)
Rule(source -> middle)
```

Raw observations contain:

```text
target witness: middle -> goal
decoy witness:  decoy_source -> decoy_goal
```

The independently held target is `goal`.

The target is not passed to the external verifier as a resolver self-report. The verifier checks whether the exact target atom is present in committed executable facts after both operation slots complete.

The intended stateful path is:

```text
S0
  -- Operation A: compile target witness --> S1 with Rule(middle -> goal)
  -- Operation B scan 1 -----------------> Fact(middle)
  -- Operation B scan 2 -----------------> Fact(goal)
  -- Operation B scan 3 -----------------> no additional required transition
```

Operation B on untouched `S0` can derive `middle` but cannot derive `goal`.

## Frozen cohort

There are exactly seven surface families with eight roots per family:

```text
2 training families  = 16 roots
1 holdout family     = 8 roots
4 future families    = 32 roots
--------------------------------
total                = 56 roots
```

The families differ only in domain vocabulary. The mechanism receives no family label and fits no parameter from training.

The exact family order is frozen in the executable source.

## Frozen paths and controls

Every root executes exactly eight paths.

### 1. Stateful

Compile the same-root target witness, apply the valid executable rule delta, then execute the three-scan closure operation.

### 2. Endpoint blind

Call the same causal compiler but discard its delta. Execute closure on the untouched executable state.

This tests whether the endpoint operation succeeds without the intermediate transformation.

### 3. Text only

Call the same causal compiler and serialize its exact typed output to text, but do not apply it to executable state. Execute closure with the same three-scan budget.

The executor cannot read the text.

### 4. Scalar history only

Call the same causal compiler and retain only a scalar indicating that one candidate transition was produced. Do not apply executable state. Execute closure with the same three-scan budget.

The executor cannot read the scalar.

### 5. Common-root incidence destroyed

Call the same causal compiler. Replace the proposed target rule with the target rule from the next root in a deterministic cyclic permutation while retaining the current root's witness id.

The resulting delta has the same type and shape but must fail witness validation in the current root.

This destroys same-root incidence while preserving one compiler call and one compile transition slot.

### 6. Matched random valid transition

Call the same causal compiler budget on the root's decoy witness and apply that exact valid witnessed rule instead of the target rule.

This preserves a successful state mutation of the same delta type and uses a root-local valid witness, but the mutation is irrelevant to the target objective.

### 7. Structurally matched invalid transition

Call the same causal compiler for the target witness, then replace the proposed rule with the root's decoy rule while retaining the target witness id.

The transition must be rejected by exact witness validation.

### 8. Delayed target transition

Call the target compiler before execution for compute accounting, execute the same three closure scans on the original state, and apply the valid target rule only after the closure operation has finished.

This preserves the same target delta and both operation calls while destroying the causal ordering required for composition.

## Frozen budgets

For every root and every path:

```text
operation calls                 = 2
existing CausalEngine calls     = 1
compile transition slots        = 1
executor search scans           = 3
transition opportunities total  = 4
independent objective checks     = 1
```

A rejected transition still consumes its compile transition slot.

An empty executor scan still consumes its search slot.

Successful state mutations are not required to be equal; equal successful mutations would erase the mechanism under test. The **opportunity and compute proxy budgets** are required to be exact.

Any budget mismatch is an infrastructure failure and produces no positive scientific verdict.

## Deterministic replay

Every root/path execution is repeated from a freshly reconstructed state with identical inputs.

The following must match exactly:

```text
success/failure
transition acceptance or rejection
operation counts
CausalEngine call count
transition-slot count
search-scan count
independent objective-check count
final canonical executable-state signature
```

Any mismatch is `REPLAY_FAILURE`.

The H9 operation boundary intentionally avoids the legacy H8 reasoning path whose unordered `HashSet` iteration invalidated H8's seeded-replay assumption.

## Independent objective witness

The objective verifier is outside the operation family.

It receives only:

```text
final executable state
expected target atom for the root
```

Success is:

```text
final_state.contains_fact(expected_target)
```

The verifier does not inspect operation self-reports, prose explanations, family labels, or control identity.

## Frozen metrics

For each split and path report:

```text
root count
success count
success rate
rejected compile-transition count
mean successful state mutations
exact budget status
exact replay status
```

For each of the four future families report the same stateful-versus-control objective result.

Primary causal margins are:

```text
stateful - endpoint blind
stateful - text only
stateful - scalar only
stateful - common-root-incidence-destroyed
stateful - matched random valid transition
stateful - delayed target transition
```

The invalid-transition path is additionally required to reject every mismatched delta.

## Frozen acceptance gates

`PASS` requires all of the following:

```text
training roots == 16
holdout roots  == 8
future roots   == 32
future families == 4

stateful training success rate == 1.0
stateful holdout success rate  == 1.0
stateful future success rate   == 1.0

endpoint blind future success rate == 0.0
text-only future success rate      == 0.0
scalar-only future success rate    == 0.0
rewired future success rate        == 0.0
random-valid future success rate   == 0.0
delayed future success rate        == 0.0

structurally invalid transition rejection rate == 1.0
rewired transition rejection rate             == 1.0

all four future families:
    stateful success rate == 1.0
    every non-stateful objective control success rate == 0.0

all path budgets exact == true
all replay comparisons exact == true
all state invariants valid == true
```

No numerical threshold may be changed after the first complete verdict-producing run.

## Terminal classification

Classification order is frozen as:

```text
INFRASTRUCTURE_FAILURE
    state invariant, cohort, serialization, or execution construction fails

BUDGET_FAILURE
    any matched path violates the frozen operation/transition/search/objective budget

REPLAY_FAILURE
    any repeated root/path execution differs under the exact replay contract

CONTROL_FAILURE
    stateful objective success transfers but any causal control also reaches the target,
    or a mismatched transition is not rejected

REJECTED
    the stateful path fails any support or transfer gate

PASS
    every frozen gate passes
```

## Claim supported by PASS

A passing result supports only:

> Within the H9 shadow substrate, a validated same-root typed state delta creates an executable commitment that is causally necessary for a later fixed-budget operation to reach an independently verified target. The effect survives new domain vocabularies and is not recovered by text-only history, scalar history, endpoint execution, rewired incidence, an irrelevant valid mutation, a structurally matched invalid mutation, or delayed application under the frozen operation and search budgets.

## Claims not supported by PASS

A passing result does not establish:

```text
that legacy Starfire resolvers already use PECS
that PECS commitments can yet be learned autonomously
that automatic ontology induction is safe
that live routing should be changed
that arbitrary natural-language reasoning is compositional
AGI
consciousness
human-level cognition
```

## No rescue iteration

After the first complete run:

```text
no threshold changes
no family substitution
no added search rounds
no weakened controls
no post-hoc path removal
no hidden target access
no automatic live promotion
```

Compile defects or implementation mismatches may be repaired only when they prevent execution of this already-written contract. Such repairs must not alter the scientific operation, controls, cohort, budgets, gates, or claim boundary.
