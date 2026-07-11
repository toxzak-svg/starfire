# Ω2 Descendant Necessity — First Frozen Result

## Terminal classification

```text
REJECTED
```

## Frozen provenance

Preregistration commit, before implementation and before the first verdict-producing run:

```text
dec cc66fc2690d9e0f38517ce0be96ecf9cbc5c6
```

Canonical form:

```text
deccc66fc2690d9e0f38517ce0be96ecf9cbc5c6
```

First verdict-producing executable head:

```text
ce3b3190b6518350002dd605269554b24329895f
```

GitHub Actions:

```text
workflow: CHARGE CI
run id: 29150339350
run number: 106
job conclusion: success
artifact id: 8247994309
artifact digest: sha256:e3e121a08ab4bbd7ad5bce49a47244224f13e32acc1b1de85bf241f408d7de25
Ω2 report sha256: 7f58d48eb9fae99c3fd31fd3eaf831e98f8db4007d74f93ba30e79bc9681d64d
```

No Ω2 grammar rule, fixture structure, hidden outcome law, family, root count, control, budget, threshold, acceptance gate, or terminal-classification rule was changed after observing the result.

## Executive result

Ω2 succeeded at every frozen structural and control requirement but failed the actual withheld transfer objective.

The experiment demonstrated all of the following on all 56 roots:

```text
L0 raw Ω1 language cannot express the stage-2 partition
H(L0) contains zero descendant programs
correct Δ1 is synthesized, independently validated, and admitted
H(L1) expands to the exact 918-program descendant language
correct Δ2 is synthesized and independently validated
exact Δ1 replacement invalidates the previously proposed Δ2
wrong valid ancestor receives matched full search and cannot validate Δ2
counterfeit Δ2 proof is rejected
outcome-shuffled descendant synthesis is rejected
payload-only Δ2 is inert
late Δ2 admission is too late
all budgets are exact
fresh-state replay is exact
all invariants hold
```

But the timely stateful chain produced only:

```text
4/8 correct withheld predictions per root
```

instead of the frozen required:

```text
8/8
```

Therefore the terminal classification is `REJECTED`.

## Objective results

### Training

```text
stateful roots:                 0/16 full successes
stateful transfer predictions: 64/128 correct
all nine controls:             0 correct transfer predictions
```

### Holdout

```text
stateful roots:                 0/8 full successes
stateful transfer predictions: 32/64 correct
all nine controls:             0 correct transfer predictions
```

### Future

```text
stateful roots:                  0/32 full successes
stateful transfer predictions: 128/256 correct
all nine controls:              0 correct transfer predictions
```

Every future family independently produced:

```text
stateful success rate:        0.0
maximum control success rate: 0.0
maximum control correct predictions: 0
```

Future families:

```text
cellular_descendants
manufacturing_descendants
software_descendants
watershed_descendants
```

## Structural audit

Every structural audit passed on every one of the 56 roots:

```text
correct stage-1 base alias defects exact: 56/56
wrong stage-1 base alias defects exact:   56/56
correct stage-1 Ω1 search exact:          56/56
wrong stage-1 Ω1 search exact:            56/56
L0 raw stage-2 audit exact:               56/56
L0 descendant language empty exact:       56/56
L1 remaining alias defects exact:         56/56
L1 descendant frontier exact:             56/56
```

The frozen boundaries therefore held:

```text
Ω1 raw candidates:                 459
Ω1 raw program/history executions: 3,672
L0 best stage-2 repair:            8/16
L0 complete stage-2 raw repair:    false
L0 descendant candidates:          0
L1 descendant candidates:          918
L1 descendant executions:          7,344
L1 descendant unique partitions:   4
L1 winning repair:                 16/16
L1 runner-up repair:               8/16
L1 winner margin:                  8
L1 partition support:              4
```

## Control behavior

All nine non-stateful controls produced zero correct transfer predictions in training, holdout, and future.

### L0 raw search

The complete 459-program Ω1 raw search never produced a complete stage-2 repair.

```text
complete raw repairs: 0/56 roots
```

### L0 descendant search

Without an admitted ancestor refinement:

```text
ancestor terminals: 0
descendant candidates: 0
NoAncestorRefinement: 56/56 roots
```

### Δ1 endpoint only

The correct Ω1 program was available for endpoint execution but was not admitted to `StateLanguage`.

Result:

```text
NoAncestorRefinement: 56/56 roots
0 correct transfer predictions
```

Possessing or executing the program outside executable state did not create the descendant hypothesis-language terminal.

### Wrong valid ancestor

A genuine same-root Ω1 refinement for the independent `r` partition was synthesized, validated, and admitted on every root.

The path received the complete matched descendant search:

```text
918 candidates
7,344 candidate/history executions
```

but no descendant certificate passed validation:

```text
56/56 validation rejections
0 correct transfer predictions
```

### Exact Δ1 ablation and replacement

Correct Δ2 was first proposed under the correct `L1`.

Then exact Δ1 was removed by rebuilding state and replacing it with the genuine wrong-valid ancestor. The validator still received one admitted ancestor bit and the complete matched 918-candidate search.

Result:

```text
56/56 full validation rejections
0 correct transfer predictions
```

This control passed exactly as preregistered: the descendant proof was bound to the exact ancestor language that made it expressible.

### Δ2 payload only

Correct Δ1 was admitted and correct Δ2 was fully synthesized and independently validated on every root, but Δ2 was not admitted to layered executable state.

Result:

```text
56/56 validations succeeded
0/56 Δ2 admissions during prediction
0 correct transfer predictions
```

### Counterfeit Δ2 proof

Every tampered proof was rejected after complete independent recomputation:

```text
56/56 rejections
0 correct transfer predictions
```

### Outcome-shuffled descendant synthesis

The complete descendant search was executed against the deterministic count-preserving shuffled outcome incidence.

No shuffled proposal passed the complete-repair certificate gate:

```text
56/56 validation rejections
0 correct transfer predictions
```

### Delayed Δ2 admission

Correct Δ1 and Δ2 were synthesized and validated, but Δ2 was admitted only after the frozen transfer window.

Result:

```text
0 correct transfer predictions during the window
56/56 eventual Δ2 admissions after the window
```

## Frozen gates

Passed:

```text
cohort_exact:                                  true
structural_audits_exact:                       true
every_control_individual_prediction_zero:      true
l0_raw_complete_repair_absent_everywhere:      true
l0_descendant_frontier_empty_everywhere:       true
wrong_ancestor_full_search_rejected_everywhere:true
exact_ancestor_ablation_rejected_everywhere:   true
counterfeit_descendant_rejected_everywhere:    true
shuffled_descendant_rejected_everywhere:       true
payload_only_never_admitted:                   true
delayed_admission_zero_during_window:          true
delayed_admission_eventually_succeeds:         true
budgets_exact:                                 true
replay_exact:                                  true
invariants_hold:                               true
```

Failed:

```text
stateful_training:             false
stateful_holdout:              false
stateful_future:               false
all_future_families_transfer:  false
```

## Exact failure mechanism

The rejection was not caused by failure to discover or admit the descendant.

The problem occurred one generation earlier: the admitted Ω1 ancestor refinement was **behaviorally correct on the discovery histories but not invariant under the withheld block-order transformation**.

### Discovery-equivalent Ω1 representatives

The Ω1 search groups candidate programs by their binary partition over the eight discovery histories, then retains the lexicographically smallest program string as the executable representative of each partition.

For the target stage-1 partition `p`, the winning behavioral partition is:

```text
00001111
```

Under the frozen discovery block layout:

```text
AB | CD | EF
```

the canonical executable representative selected by the unchanged Ω1 tie-break is:

```text
first(A) < count(A)
```

Every history contains exactly one `A`, so:

```text
count(A) = 1
```

On discovery histories, `A` is inside the first two positions:

```text
p = 1 -> A at index 0 -> 0 < 1 -> true
p = 0 -> A at index 1 -> 1 < 1 -> false
```

Thus the program is perfectly behaviorally equivalent to the intended pair-order bit on the discovery support:

```text
first(A) < count(A) == p
```

But that equivalence is accidental and layout-dependent.

### Withheld transfer transformation

Transfer permutes the three pair blocks to:

```text
EF | AB | CD
```

The same atom `A` now appears at index 2 or 3 while its count remains 1:

```text
2 < 1 -> false
3 < 1 -> false
```

Therefore the admitted ancestor refinement collapses on every transfer history:

```text
Δ1_transfer = false
```

instead of preserving `p`.

### Descendant program

Under discovery, the winning descendant partition is represented by:

```text
bit[0] != (first(C) < first(D))
```

The raw `C/D` pair-order relation itself is stable under the block permutation.

On discovery:

```text
bit[0] = p
first(C) < first(D) = q
Δ2 = p != q
```

The layered key therefore distinguishes the target `p == q` outcome perfectly, up to Boolean complement.

On transfer, however:

```text
bit[0] = false
Δ2 = false != q = q
```

The executable state loses `p` and retains only `q`.

For each value of `q`, half the histories have `p == q` and half do not. The resulting deterministic transfer score is therefore exactly:

```text
4/8 correct per root
```

which matches the frozen run:

```text
training: 64/128
holdout:   32/64
future:   128/256
```

## Scientific interpretation

Ω2 does **not** support the preregistered descendant-necessity claim because the required descendant chain failed withheld transfer.

However, the rejection is narrower than “recursive representation growth did not work.” The experiment successfully established the structural prefix of the chain:

```text
L0 has no descendant terminals
Δ1 admission creates a 918-program descendant language
Δ2 becomes expressible and independently validates only under the correct ancestor language
replacing exact Δ1 destroys validation under matched compute
Δ2 admission is necessary on the discovery representation surface
```

What failed is transport of the executable ancestor representative outside the finite support on which its behavioral equivalence class was defined.

The strongest supported claim is:

> Under the frozen Ω2 discovery cube, Starfire created a genuine second-generation proof-carrying refinement whose expressibility and validation required the exact admitted Ω1 ancestor language, and all matched necessity controls passed; however, the chain failed withheld transfer because Ω1 selected a discovery-equivalent executable representative whose semantics were not invariant under the transfer transformation.

The strongest claim not supported is:

> Starfire has demonstrated cumulative two-generation executable representation growth that transfers to structurally transformed withheld histories.

It has not.

## Architectural implication

Ω2 exposes a new representational problem that Ω1's PASS did not test:

> **A behavioral partition over observed histories is not sufficient to define a stable executable representation.**

Two candidate programs may be indistinguishable on the discovery support while belonging to different transformation-behavior classes outside that support.

Ω1 currently quotients programs by:

```text
same binary partition on observed discovery histories
```

Ω2 shows that this quotient is too coarse for cumulative executable representation growth.

The missing object is a notion of **representative invariance / transport semantics**:

```text
programs should not be treated as interchangeable merely because
P1(h) == P2(h) for all observed discovery histories
```

when there exists an admissible structure-preserving transformation `T` such that:

```text
P1(T(h)) != P2(T(h))
```

The immediate bottleneck is therefore not descendant enumeration. It is choosing or validating an executable representative whose semantics survive the transformation class under which the representation is expected to generalize.

## No rescue iteration

Ω2 is terminal for this frozen experiment.

No Ω2 grammar rule, fixture structure, transfer permutation, canonical tie-break, support threshold, candidate family, control, budget, acceptance gate, or terminal classification will be changed to rescue the result.

Changing Ω1's representative selection or adding transformation-aware validation would be a separate foundation experiment with a new preregistration.

## Next justified experiment

The next narrow experiment should test **transformation-stable representative identification** before attempting another descendant chain.

A suitable target is:

```text
ΩR1 — Representative Transportability
```

Require the system to distinguish discovery-equivalent candidate programs that diverge under independently specified structure-preserving transformations, and to admit a representation only when its executable semantics are stable across the frozen transformation orbit.

The decisive comparison should be:

```text
partition-only equivalence
    vs
transformation-orbit equivalence
```

with the Ω2 failure pair as a motivating class but with new held-out vocabularies and transformations preregistered before execution.

Only after that substrate passes should descendant necessity be retested as a new experiment rather than Ω2 rescue tuning.
