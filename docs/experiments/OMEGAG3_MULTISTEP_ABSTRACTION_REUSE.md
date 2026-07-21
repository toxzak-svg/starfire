# ΩG3 Multi-Step Abstraction and Reuse

Status: **preregistered before implementation and before any verdict-producing run**.

ΩG3 is blocked on the committed-source ΩG2 `PASS` merged to `main` at:

```text
ccdf0272454c4dca425895c97ed4c39ec61b669c
```

No ΩG3 implementation may alter this contract after the preregistration commit is created. Any correction to the fixture, search spaces, budgets, controls, gates, or claim boundary requires a separately identified preregistration amendment committed before the affected implementation or verdict run.

## Research break from ΩG2

ΩG2 established that Starfire can require a previously validated and admitted ΩG1 `AdjacentBefore` production as an executable parent, compose it through the frozen `SharedMiddleAnd` operator, independently validate the generic arity-3 child `ConsecutiveChain3`, and transfer that child across unseen vocabularies.

ΩG2 still handles one fixed composed production. It does not establish that Starfire can inspect several independently validated concrete compositions, identify a shared recursive structure, factor that structure into a parameterized production, and reuse the abstraction at lower matched search cost on unseen arities.

ΩG3 asks:

> Can Starfire derive one bounded parameterized chain-family production from independently synthesized and validated length-3, length-4, and length-5 concrete expressions, independently validate and admit that abstraction, and reuse it on unseen length-6 and length-7 tasks with exact transfer while reducing matched candidate search and execution cost relative to lower-level re-synthesis?

The causal requirement is:

> Removing the admitted ΩG2 executable parent must remove legal abstraction construction and all abstraction-authorized transfer. Removing only the abstraction while preserving the lower-level parents must preserve correctness under matched re-synthesis but eliminate the preregistered reuse advantage.

This is bounded abstraction and reuse. It is not unrestricted recursion, arbitrary program induction, natural-language concept learning, automatic ontology promotion, or live self-modification.

## Frozen dependency lineage

The ΩG3 validator must independently revalidate the complete parent chain:

```text
ΩG1 admitted production: AdjacentBefore, arity 2
ΩG2 admitted production: ConsecutiveChain3, arity 3
ΩG2 operator: SharedMiddleAnd
ΩG3 required ΩG2 result: PASS
```

The ΩG3 parent handle must bind:

- the ΩG2 cohort identifier;
- the ΩG2 problem digest;
- the ΩG2 proof identifier;
- the admitted ΩG2 production kind;
- the exact ΩG2 registry signature;
- the nested ΩG1 parent lineage carried by the ΩG2 registry.

Raw enum values, copied proof text, a serialized result document, a stale registry, a foreign certificate, or an unvalidated concrete expression cannot authorize abstraction.

## Frozen lower-level expression language L2

A concrete chain expression of arity `n >= 3` is formed from exactly:

```text
Base3(x1, x2, x3) := ConsecutiveChain3(x1, x2, x3)
AppendRight(E, xn, x[n+1]) := E && AdjacentBefore(xn, x[n+1])
```

For arity `n`, the concrete expression contains:

```text
1 ConsecutiveChain3 leaf
n - 3 AdjacentBefore leaves
n - 3 binary conjunction nodes
```

The lower-level re-synthesis search must exhaust:

```text
n! ordered root-local bindings
Catalan(n - 3) binary conjunction shapes
```

Therefore the exact candidate count is:

```text
L2_candidates(n) = n! * Catalan(n - 3)
```

The search may not read hidden role labels, fixture family labels, expected bindings, or abstraction labels.

## Frozen abstraction meta-grammar A1

ΩG3 evaluates exactly four parameterized schema candidates:

1. `RecursiveAppendAdjacent`
2. `EndpointBridge`
3. `DisjointAdjacentPairs`
4. `FixedArityThree`

Only `RecursiveAppendAdjacent` has the frozen semantics:

```text
ChainFamily(3, x1, x2, x3)
    := ConsecutiveChain3(x1, x2, x3)

ChainFamily(n + 1, x1, ..., xn, x[n+1])
    := ChainFamily(n, x1, ..., xn)
       && AdjacentBefore(xn, x[n+1])
```

The human-readable name is descriptive only. Admission authority derives from the validated training examples, parent lineage, schema identifier, problem digest, proof identifier, and registry admission.

The other candidates are frozen negative alternatives:

```text
EndpointBridge:
    uses only the first and last atoms

DisjointAdjacentPairs:
    uses AdjacentBefore on non-overlapping atom pairs

FixedArityThree:
    reproduces only the admitted ΩG2 arity-3 child
```

The abstraction search must evaluate all four candidates against every validated development example. Exactly one candidate must reproduce every concrete expression.

## Frozen fixture

Each root owns a private vocabulary whose atom names reveal neither position nor family. A task of arity `n` contains all `n` atoms exactly once in each history.

### Development examples

```text
4 roots of arity 3
4 roots of arity 4
4 roots of arity 5
12 development roots total
```

### Holdout reuse

```text
8 roots of unseen arity 6
```

### Future reuse

```text
24 roots of unseen arity 7
3 future families
8 roots per family
```

Future family labels are report metadata only and are unavailable to synthesis, abstraction search, validation, registry admission, or prediction.

## Frozen evidence construction

For each root, one hidden canonical ordering defines the positive chain. The canonical ordering is used only by fixture construction and the independent objective checker.

Discovery evidence contains:

```text
1 positive canonical history
n - 1 negative adjacent-swap histories
1 negative reversed history
n + 1 discovery histories total
```

Transfer evidence contains:

```text
1 positive canonical history
n - 1 negative non-identity cyclic rotations
1 negative endpoint-swap history
n + 1 transfer histories total
```

A bound chain expression predicts positive only when all atoms occur in its bound order.

The synthesizer sees histories and binary outcomes. It does not receive the hidden canonical ordering as a binding, role map, target expression, or family label.

## Frozen concrete-synthesis counts

For each root, all lower-level candidates are executed against every discovery history.

| Arity | Bindings `n!` | Conjunction shapes `Catalan(n-3)` | L2 candidates | Discovery histories | L2 executions | Exact L2 candidates |
|---:|---:|---:|---:|---:|---:|---:|
| 3 | 6 | 1 | 6 | 4 | 24 | 1 |
| 4 | 24 | 1 | 24 | 5 | 120 | 1 |
| 5 | 120 | 2 | 240 | 6 | 1,440 | 2 |
| 6 | 720 | 5 | 3,600 | 7 | 25,200 | 5 |
| 7 | 5,040 | 14 | 70,560 | 8 | 564,480 | 14 |

The development totals are frozen:

```text
concrete candidate programs: 1,080
concrete program-history executions: 6,336
exact lower-level candidates: 16
```

Proposal and independent validation must consume identical budgets and select the same canonical concrete solution for every development root.

## Frozen abstraction counts and compression gate

The abstraction search evaluates:

```text
4 schema candidates
12 validated development examples
48 schema-example evaluations
1 exact schema
```

Canonical concrete expression node cost is:

```text
arity 3: 1 node
arity 4: 3 nodes
arity 5: 5 nodes
```

Across four development roots at each arity:

```text
concrete node cost: 36
RecursiveAppendAdjacent schema node cost: 5
minimum required compression advantage: 31 nodes
```

No abstraction may pass by fitting only a subset of development arities, by memorizing concrete atom identities, or by storing the twelve concrete solutions as constants.

## Frozen reuse search and efficiency gates

After abstraction admission, root-local reuse still exhausts every ordered atom binding. It removes only the lower-level conjunction-shape search.

```text
A1_candidates(n) = n!
```

### Holdout arity 6

Per root:

```text
L2 candidates: 3,600
A1 candidates: 720
L2 executions: 25,200
A1 executions: 5,040
required reduction factor: exactly 5
```

Across eight roots:

```text
L2 candidates: 28,800
A1 candidates: 5,760
L2 executions: 201,600
A1 executions: 40,320
```

### Future arity 7

Per root:

```text
L2 candidates: 70,560
A1 candidates: 5,040
L2 executions: 564,480
A1 executions: 40,320
required reduction factor: exactly 14
```

Across twenty-four roots:

```text
L2 candidates: 1,693,440
A1 candidates: 120,960
L2 executions: 13,547,520
A1 executions: 967,680
```

Both matched paths must remain correct. The ΩG3 claim requires exact reuse transfer plus the frozen cost reduction. A correctness-only result without the reduction is `REJECTED`.

## Frozen transfer gates

The admitted abstraction must produce:

```text
holdout perfect roots: 8 / 8
holdout correct transfer predictions: 56 / 56
future perfect roots: 24 / 24
future correct transfer predictions: 192 / 192
```

Each future family must independently produce:

```text
perfect roots: 8 / 8
correct transfer predictions: 64 / 64
```

The matched lower-level re-synthesis baseline must also be correct on every transfer history, but must consume the larger frozen budgets.

## Frozen negative and integrity controls

The following controls must pass:

1. **ΩG2 parent ablation**
   - legal abstraction candidates: `0`
   - abstraction-authorized transfer predictions: `0`

2. **Parent proof text only**
   - legal abstraction candidates: `0`
   - transfer predictions: `0`

3. **Single-example support**
   - abstraction validation rejected

4. **Single-arity support**
   - abstraction validation rejected

5. **Fixed-arity memorizer**
   - arity-6 and arity-7 abstraction transfer predictions: `0`

6. **Concrete-example atom memorization**
   - foreign-vocabulary admission rejected

7. **Shuffled outcome labels**
   - concrete proof or abstraction proof rejected

8. **Counterfeit concrete certificate**
   - rejected atomically

9. **Counterfeit abstraction certificate**
   - rejected atomically

10. **Foreign abstraction certificate**
    - rejected atomically

11. **Stale ΩG2 registry**
    - parent revalidation rejected

12. **Raw schema injection**
    - rejected atomically

13. **Duplicate abstraction admission**
    - rejected atomically

14. **Problem digest mismatch**
    - rejected

15. **Fresh-state replay**
    - byte-exact report reproduction required

## Frozen authority boundary

ΩG3 remains offline, deterministic, feature-independent, and shadow-only.

All authority flags must remain false:

- no `Runtime::chat()` wiring;
- no generated-response influence;
- no routing authority;
- no persistence mutation;
- no belief or ontology promotion;
- no PECS or CHARGE mutation;
- no tool or capability selection;
- no external side effects;
- no autonomous action;
- no automatic source modification.

## Frozen terminal classifications

```text
DEPENDENCY_FAILURE
    required ΩG1 or ΩG2 executable parent lineage is absent or invalid

CONTROL_FAILURE
    a budget, integrity, ablation, immutability, authority, or matched-control gate fails

REPLAY_FAILURE
    fresh-state report reproduction is not byte-exact

PASS
    every frozen gate passes

REJECTED
    execution is valid but the abstraction, transfer, compression, or reuse advantage is insufficient
```

## Supported claim if PASS

A `PASS` supports only:

> Under the frozen bounded chain fixture, Starfire factored independently synthesized and validated arity-3, arity-4, and arity-5 lower-level expressions into one proof-carrying recursive parameterized production, independently validated and admitted that abstraction, and reused it on unseen arity-6 and arity-7 tasks with exact transfer and the preregistered matched reduction in candidate search and program-history executions. The abstraction required the admitted ΩG2 executable parent, while lower-level re-synthesis remained correct but more expensive.

## Claims not established

A `PASS` does not establish unrestricted program synthesis, arbitrary recursion depth, general concept formation, open-world abstraction, natural-language grammar induction, automatic ontology promotion, safe live self-modification, autonomous agency, consciousness, human-level cognition, or AGI.

## Next step if PASS

The next experiment must not merely increase chain length. It should test abstraction selection under competing reusable structures, where more than one compression hypothesis fits development evidence and held-out intervention is required to choose the causally useful abstraction.