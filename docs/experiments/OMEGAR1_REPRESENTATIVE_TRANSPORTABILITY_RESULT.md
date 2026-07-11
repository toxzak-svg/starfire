# ΩR1 Representative Transportability — First Frozen Result

## Terminal classification

```text
PASS
```

## Frozen provenance

Preregistration commit, before implementation and before the first verdict-producing run:

```text
a4f7472d0f69b351a2eaac9a21c26976cf1af5ce
```

First verdict-producing executable head:

```text
dd5e628f905c83015e822e44a608332d5fe9c66e
```

GitHub Actions:

```text
workflow: CHARGE CI
run id: 29150969618
run number: 109
job conclusion: success
artifact id: 8248150893
artifact digest: sha256:9250da350386cfb4a4e72aa345e2c5b140aa4ab9978c33e3b1b524fdb6e64289
ΩR1 report sha256: ac125479516bad37a0f741ea07617afe8667fb417ef0041de57e29334c9244f4
```

No ΩR1 transformation, fixture, grammar, candidate count, root count, control, budget, tie-break, acceptance gate, or terminal-classification rule was changed after observing the result.

## Executive result

ΩR1 passed every preregistered gate.

Across all 56 roots, the orbit-aware path:

```text
identified the same winning discovery behavioral partition as Ω1
retained all 72 discovery-equivalent executable representatives
used two same-history structure-preserving calibration transforms
executed exactly 2,304 transformed-history representative evaluations
reduced the equivalence class to exactly 8 zero-violation representatives
selected the canonical transport-stable program first(A) < first(B)
independently recomputed and validated the complete proof
admitted the root-bound transport certificate
achieved 32/32 predictions on two held-out transformations per root
```

The unchanged Ω1 partition-only baseline and the matched-compute target-stationary calibration both selected the accidental representative:

```text
first(A) < count(A)
```

and achieved exactly:

```text
16/32 held-out predictions per root
```

The correspondence-rewired control preserved the candidate class, transformation count, transformed-history multiset, and full 2,304-execution budget, but destroyed same-history correspondence. It produced:

```text
zero-violation representatives = 0
minimum transport violations = 4
validated certificates = 0
held-out predictions = 0/32 per root
```

All budgets were exact, both fresh-state replays matched for every root/path, all proof/admission controls passed, and all executable-state invariants held.

## Objective results

### Training

Orbit-aware stateful:

```text
16/16 roots achieved full success
512/512 held-out predictions correct
32/32 correct per root
```

Partition-only baseline:

```text
0/16 roots achieved full success
256/512 held-out predictions correct
16/32 correct per root
```

Target-stationary matched calibration:

```text
0/16 roots achieved full success
256/512 held-out predictions correct
16/32 correct per root
```

All remaining controls:

```text
0 correct held-out predictions
```

### Holdout

Orbit-aware stateful:

```text
8/8 roots achieved full success
256/256 held-out predictions correct
32/32 correct per root
```

Partition-only baseline:

```text
0/8 roots achieved full success
128/256 held-out predictions correct
16/32 correct per root
```

Target-stationary matched calibration:

```text
0/8 roots achieved full success
128/256 held-out predictions correct
16/32 correct per root
```

All remaining controls:

```text
0 correct held-out predictions
```

### Future

Orbit-aware stateful:

```text
32/32 roots achieved full success
1,024/1,024 held-out predictions correct
32/32 correct per root
```

Partition-only baseline:

```text
0/32 roots achieved full success
512/1,024 held-out predictions correct
16/32 correct per root
```

Target-stationary matched calibration:

```text
0/32 roots achieved full success
512/1,024 held-out predictions correct
16/32 correct per root
```

All remaining controls:

```text
0 correct held-out predictions
```

## Future-family transfer

Every future family independently reproduced the exact frozen result:

```text
family                       orbit-aware      partition-only      stationary
cellular_transport           256/256          128/256             128/256
manufacturing_transport      256/256          128/256             128/256
software_transport           256/256          128/256             128/256
watershed_transport          256/256          128/256             128/256
```

Strict full-root success rates:

```text
orbit-aware:     1.0
partition-only:  0.0
stationary:      0.0
```

## Structural audit

Every one of the 56 roots passed every preregistered structural audit:

```text
base alias defects exact:                       56/56
Ω1 partition search exact:                      56/56
Ω1 accidental representative exact:            56/56
primary moving-orbit frontier exact:            56/56
target-stationary frontier exact:               56/56
correspondence-rewired frontier exact:           56/56
```

The frozen discovery search held exactly:

```text
raw atoms:                                  8
discovery histories:                       16
history-pair evaluations:                 120
opposite-outcome alias defects:            64
candidate programs:                       828
discovery program/history executions:  13,248
unique behavioral partitions:              5
winner repaired defects:                  64/64
runner-up repaired defects:               32/64
winner margin:                              32
partition support:                           8
winning-class executable representatives:   72
```

## Primary moving-orbit result

Calibration transformations:

```text
T1 = [1,0,2,3]
T2 = [1,2,0,3]
```

These transformations preserve every within-pair order but move the target `AB` block from its discovery location.

No transformed outcome labels were used. Representative scoring was only:

```text
P(T(h)) == P(h)
```

for the same original history `h`.

Exact result on every root:

```text
winning-class representatives:       72
calibration transformations:          2
transport executions:             2,304
zero-violation representatives:       8
minimum transport violations:         0
selected transport violations:        0
selected canonical representative: first(A) < first(B)
```

The implementation preserved the output vector produced during the original 13,248 discovery evaluations. Orbit scoring therefore did not perform hidden original-history re-execution; the declared compute ledger matches executed work.

## Partition-only baseline

The unchanged Ω1 search and validation path selected:

```text
first(A) < count(A)
```

on every root vocabulary.

It perfectly represented the target partition on the fixed discovery layout but collapsed when the target block moved under held-out transformations.

Exact result:

```text
16/32 correct per root
```

The comparison therefore reproduces the failure mode exposed by Ω2 under a new eight-atom/four-bit fixture.

## Target-stationary matched-compute control

Calibration transformations:

```text
S1 = [0,2,1,3]
S2 = [0,3,2,1]
```

They execute the same number of transformations and the same exact transport budget as the primary path:

```text
72 representatives
2 transformations
16 histories
2,304 transport executions
```

But the target `AB` block remains in its discovery position.

Exact result on every root:

```text
zero-violation representatives: 72
minimum violations:               0
selected representative: first(A) < count(A)
held-out transfer:              16/32
```

This control is decisive against the explanation that the primary advantage came merely from additional program execution or a second search stage. The calibration orbit had to constrain the transformation degree of freedom that caused the original ambiguity.

## Correspondence-rewired control

This control used the same moving-target transformations and the same exact 2,304 transport evaluations, but compared each transformed history with the next original history in deterministic cyclic order rather than its own source history.

Exact result on every root:

```text
zero-violation representatives: 0
minimum transport violations:   4
validation successes:            0
validation rejections:         56/56
held-out predictions:            0
```

Thus the primary effect depends on the specific same-history correspondence relation required by the invariance claim, not merely on the transformed-history multiset.

## Proof/admission controls

### Payload only

Correct transport proofs and certificates were produced and independently validated on every root, but no executable admission occurred.

```text
validation successes: 56/56
admissions during window: 0
final admissions: 0
held-out predictions: 0
```

### Counterfeit proof

A structural proof field was tampered after correct proposal.

Independent full recomputation rejected every proof:

```text
56/56 validation rejections
0 predictions
```

### Foreign-root certificate

A valid certificate was independently produced under a different root with identical candidate-space dimensions and then presented to the target root.

```text
56/56 donor validations succeeded
56/56 target admissions rejected
0 predictions
```

### Delayed admission

Correct proposal and validation completed before the objective window, but certificate admission occurred only afterward.

```text
predictions during window: 0
admissions during window: 0
56/56 eventual admissions after the window
```

## Frozen gate result

Every gate passed:

```text
cohort_exact:                               true
structural_audits_exact:                    true
orbit_aware_training:                       true
orbit_aware_holdout:                        true
orbit_aware_future:                         true
partition_only_exact_half_transfer:         true
stationary_exact_half_transfer:             true
rewired_full_search_rejected_everywhere:    true
payload_only_inert_everywhere:              true
counterfeit_rejected_everywhere:            true
foreign_root_rejected_everywhere:           true
delayed_zero_during_window:                 true
delayed_eventual_admission:                 true
all_future_families_transfer:               true
budgets_exact:                              true
replay_exact:                               true
invariants_hold:                            true
```

## Scientific interpretation

ΩR1 supports the preregistered claim:

> Under the frozen symbolic pair-block regime, discovery-partition equivalence alone was insufficient to identify a transport-stable executable representation, while independent same-history evaluation over a preregistered structure-preserving calibration orbit distinguished 72 discovery-equivalent representatives, selected a zero-violation within-pair program, and transferred perfectly to block permutations not used during proposal or validation. Matched-compute target-stationary and correspondence-rewired controls failed as predicted, and executable admission remained necessary.

This directly resolves the foundation failure exposed by Ω2 at the level tested:

```text
same discovery behavioral partition
    !=
same executable semantics under admissible structural transformation
```

The useful equivalence relation must include the transformation orbit on which the representation is expected to remain semantically stable.

## What ΩR1 does not establish

The PASS does not establish:

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

## Architectural implication

The Ω1 partition quotient:

```text
P1 ~ P2 iff they agree on observed discovery histories
```

is too coarse when executable representations must generalize across known classes of structural transformation.

ΩR1 validates a stronger bounded criterion:

```text
P1 ~T P2 only after considering their behavior over the frozen transformation orbit T
```

The critical advance is not merely a new tie-break. It is a new evidence type for executable representation identity:

```text
discovery behavioral equivalence
+
same-history transformation consistency
+
independent proof recomputation
+
root-bound executable admission
```

## No rescue iteration

ΩR1 passed on its first complete verdict-producing run.

No transformation, fixture, grammar, candidate count, root count, control, budget, tie-break, gate, or classification rule was changed after observing the result.

## Next justified experiment

The next justified experiment is a **new descendant-necessity study using transport-certified ancestor refinements**.

It must not be presented as rerunning or rescuing Ω2. It should be separately preregistered with:

```text
new fixture identities
transport-certified Δ1 admission
new descendant target relation
new held-out transformations not used for ancestor transport calibration
exact ablation of the transport certificate
matched valid-but-wrong transport-certified ancestor control
proof-bound Δ2 admission
```

Only such a study can test whether the ΩR1 substrate actually enables cumulative executable representation growth rather than merely repairing one-generation transport.
