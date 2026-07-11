# ΩD1 Transport-Certified Descendant Necessity — First Frozen Result

## Terminal classification

```text
PASS
```

## Frozen provenance

Preregistration was committed before implementation and before any verdict-producing run:

```text
docs/experiments/OMEGAD1_TRANSPORT_CERTIFIED_DESCENDANT_NECESSITY.md
26e7c836200e31ccc3b8cdfdfe3755428ab7d619
```

The exact frozen executable was run independently in two environments:

```text
local exact-branch execution: PASS
GitHub ΩD1 CI execution:      PASS
```

The raw structured JSON reports were byte-for-byte identical.

The GitHub workflow also completed:

```text
cargo check -p star --all-targets --locked
representation_genesis tests
representation_transport_orbit tests
representation_transport_descendants tests
Ω1 prerequisite execution
ΩR1 prerequisite execution
ΩD1 execution
artifact preservation
```

No ΩD1 transformation, fixture, outcome law, grammar, candidate count, root count, control, budget, tie-break, acceptance gate, or terminal-classification rule was changed after observing the result.

## Executive result

ΩD1 passed every preregistered gate.

Across all 56 roots, the primary chain:

```text
L0
  -> synthesize / validate / admit transport-stable Δ1
  -> L1 gains one executable ancestor terminal
  -> descendant frontier expands from 0 to 1,656 programs
  -> synthesize / independently validate / admit Δ2
  -> L2
  -> predict on two structural transformations unseen by ancestor calibration and descendant fitting
```

achieved:

```text
32/32 held-out predictions per root
```

The second-stage target was:

```text
Y2 = p != r
```

No single raw Ω1 program expressed that partition:

```text
828 raw programs
13,248 raw program/history executions
5 raw behavioral partitions
64 opposite-outcome pairs
best raw repair = 32/64
complete raw repair = false
```

After one correct transport-certified ancestor was admitted, the executable descendant hypothesis language became:

```text
0 ancestor terminals -> 0 descendant programs
1 ancestor terminal  -> 1,656 descendant programs
```

The correct descendant search then produced exactly:

```text
1,656 descendant candidates
26,496 descendant program/history executions
5 descendant behavioral partitions
winner = 64/64 repaired pairs
runner-up = 32/64
winner margin = 32
partition support = 8
```

## Objective results

### Training

Primary transport-certified descendant chain:

```text
16/16 roots achieved full success
512/512 held-out predictions correct
32/32 correct per root
```

Stationary-calibrated ancestor descendant chain:

```text
0/16 roots achieved full success
256/512 held-out predictions correct
16/32 correct per root
```

All other controls:

```text
0 correct held-out predictions
```

### Holdout

Primary transport-certified descendant chain:

```text
8/8 roots achieved full success
256/256 held-out predictions correct
32/32 correct per root
```

Stationary-calibrated ancestor descendant chain:

```text
0/8 roots achieved full success
128/256 held-out predictions correct
16/32 correct per root
```

All other controls:

```text
0 correct held-out predictions
```

### Future

Primary transport-certified descendant chain:

```text
32/32 roots achieved full success
1,024/1,024 held-out predictions correct
32/32 correct per root
```

Stationary-calibrated ancestor descendant chain:

```text
0/32 roots achieved full success
512/1,024 held-out predictions correct
16/32 correct per root
```

All other controls:

```text
0 correct held-out predictions
```

## Future-family transfer

Every future family independently reproduced the exact frozen result:

```text
family                    transport-certified chain    stationary chain
cellular_cascade          256/256                      128/256
manufacturing_cascade     256/256                      128/256
software_cascade          256/256                      128/256
watershed_cascade         256/256                      128/256
```

Strict full-root success rates:

```text
transport-certified chain: 1.0
stationary chain:           0.0
```

## Stage-1 structural result

Every root reproduced the ΩR1 transport boundary for the new cascade vocabularies.

### Correct ancestor

Behavior:

```text
Y1 = p
```

Calibration:

```text
[1,0,2,3]
[1,2,0,3]
```

Exact search:

```text
828 raw programs
13,248 discovery executions
5 discovery partitions
64/64 winning repair
32/64 runner-up
winner margin 32
support 8
72 discovery-equivalent executable representatives
2,304 transport executions
8 zero-violation representatives
selected stable program = first(A) < first(B)
```

The selected program was independently recomputed, certificate-bound, and admitted before it entered executable state.

### Wrong but valid transport-certified ancestor

Behavior:

```text
Ywrong = s
```

Calibration:

```text
[0,3,1,2]
[0,1,3,2]
```

Exact result on every root:

```text
72 winning-class representatives
2,304 transport executions
8 zero-violation representatives
selected stable program = first(G) < first(H)
```

This was a genuine independently validated transport-certified same-root ancestor, not a synthetic invalid control.

### Stationary-calibrated ancestor

Behavior on discovery:

```text
Y1 = p
```

Matched-compute calibration:

```text
[0,2,1,3]
[0,3,2,1]
```

Exact result:

```text
72 zero-violation representatives
selected accidental program = first(A) < count(A)
```

The certificate was valid under its supplied stationary calibration suite and used the same complete ancestor compute budget.

## Descendant necessity controls

### L0 raw search

The full raw stage-2 search was executed twice per path.

Exact result on every root:

```text
828 raw candidates
13,248 executions per audit
5 raw partitions
64 opposite-outcome pairs
best repair = 32/64
complete repair = false
0 held-out predictions
```

### L0 descendant search without an ancestor

Fresh executable state contained:

```text
ancestor terminals = 0
descendant candidates = 0
```

Result:

```text
NoAncestorRefinement on every root
0 held-out predictions
```

No synthetic state bit was introduced to equalize the intentionally empty hypothesis language.

### Ancestor certificate payload only

Correct ancestor transport proposal and independent validation completed, but the transport certificate was not admitted.

Result:

```text
ancestor proof/certificate payload present
executable ancestor terminals = 0
NoAncestorRefinement on every root
0 held-out predictions
```

Possessing the certificate payload did not create the descendant language.

### Wrong transport-certified ancestor

The stable `s` ancestor was independently synthesized, validated, and admitted.

It received the complete matched descendant search:

```text
1 real admitted ancestor terminal
828 raw programs
1,656 descendant candidates
26,496 descendant executions
```

Its best descendant partition repaired only:

```text
32/64 opposite-outcome pairs
```

Independent validation therefore rejected every descendant certificate.

Result:

```text
56/56 wrong ancestor admissions succeeded
56/56 full descendant validations rejected
0 held-out predictions
```

### Exact ancestor replacement before descendant validation

This was the strongest matched necessity control.

For every root:

```text
1. independently synthesize / validate / admit correct transport-certified p ancestor
2. propose Δ2 under that exact L1
3. rebuild fresh executable state
4. independently synthesize / validate / admit genuine stable s ancestor
5. run complete Δ2 validation under the replacement state
```

The validator still received:

```text
1 real admitted ancestor terminal
1,656 descendant candidates
26,496 descendant executions
```

but the proposed descendant proof was bound to the original ancestor-state signature.

Result:

```text
56/56 exact replacement validations rejected
0 held-out predictions
```

Thus the new descendant was not merely enabled by “some extra bit” or by extra compute. It depended on the exact transport-certified ancestor state that made its partition expressible.

## Stationary ancestor descendant chain

This path is the decisive cumulative-transfer control.

The stationary-calibrated accidental ancestor:

```text
first(A) < count(A)
```

was valid on the discovery support, so the full descendant search still found the exact `p != r` partition and independent validation succeeded.

Therefore, on every root:

```text
ancestor validation succeeded
descendant proposal succeeded
descendant validation succeeded
descendant admission succeeded
```

But the held-out transformations moved `AB`, causing the ancestor bit to collapse exactly as in the Ω2 failure mode.

The cumulative chain therefore achieved exactly:

```text
16/32 held-out predictions per root
```

rather than `32/32`.

This shows that discovery-time validity of both generations is not sufficient for cumulative transfer. The ancestor representation itself must carry transport-stable executable semantics.

## Descendant admission controls

### Descendant payload only

The correct transport-certified ancestor was admitted and the correct descendant was independently validated, but the descendant certificate was not admitted to layered executable state.

Result:

```text
56/56 descendant validations succeeded
0 descendant admissions during prediction
0 held-out predictions
```

The correct ancestor alone cannot identify `p != r`; outcomes remain conflicting within each ancestor-only executable key.

### Counterfeit descendant proof

A structural proof field was tampered after correct proposal.

Independent full recomputation rejected every proof:

```text
56/56 validation rejections
0 held-out predictions
```

### Delayed descendant admission

Correct ancestor and descendant proposal/validation completed before the objective window, but descendant admission was delayed until afterward.

Result:

```text
0 held-out predictions during the frozen window
56/56 eventual descendant admissions after the window
```

## Budget integrity

Every declared budget was exact.

### Transport ancestor proposal or validation

```text
vocabulary scans = 16
history-pair evaluations = 120
raw candidates = 828
discovery executions = 13,248
unique discovery partitions = 5
winning-class representatives = 72
calibration transformations = 2
transport executions = 2,304
```

### Raw stage-2 audit

```text
vocabulary scans = 16
history-pair evaluations = 120
raw candidates = 828
raw executions = 13,248
unique raw partitions = 5
```

### Descendant proposal or validation with one admitted ancestor

```text
vocabulary scans = 16
history-pair evaluations = 120
ancestor terminals = 1
raw programs = 828
descendant candidates = 1,656
descendant executions = 26,496
unique descendant partitions = 5
```

### Descendant proposal without an admitted ancestor

```text
ancestor terminals = 0
descendant candidates = 0
descendant executions = 0
```

### Held-out objective

Every path executed exactly:

```text
discovery key-index passes = 1
held-out transformations = 2
transformation applications = 32
prediction attempts = 32
objective checks = 32
```

## Replay and invariants

Every root/path was replayed exactly twice from fresh state.

Result:

```text
replay_exact = true everywhere
budgets_exact = true everywhere
invariants_hold = true everywhere
```

The local exact-branch JSON and GitHub CI JSON were byte-for-byte identical.

## Frozen gate result

Every preregistered gate passed:

```text
cohort_exact
structural_audits_exact
stateful_training
stateful_holdout
stateful_future
l0_raw_zero_predictions
l0_descendant_empty_everywhere
ancestor_payload_does_not_create_terminal
wrong_transport_ancestor_rejected_everywhere
exact_ancestor_replacement_rejected_everywhere
stationary_chain_exact_half_transfer
descendant_payload_only_inert_everywhere
counterfeit_descendant_rejected_everywhere
delayed_zero_during_window
delayed_eventual_admission
all_future_families_transfer
budgets_exact
replay_exact
invariants_hold
```

## Scientific interpretation

ΩD1 supports the preregistered bounded claim:

> Under the frozen symbolic pair-block regime, an independently validated and admitted transport-stable ancestor refinement created an executable hypothesis-language terminal that did not exist in L0; the new two-bit descendant partition was not expressible by any single raw Ω1 program, but became expressible and independently valid under the exact transport-certified ancestor state. A separately transport-certified wrong ancestor and exact ancestor replacement both failed under matched full descendant-search compute, while a discovery-valid but stationary-calibrated accidental ancestor supported descendant fitting yet failed cumulative held-out transfer. Timely descendant admission was necessary for the final capability.

This establishes a bounded two-generation cumulative executable representation result under the frozen grammar and transformation regime:

```text
validated transport evidence
  -> stable executable ancestor identity
  -> new descendant hypothesis-language terminal
  -> independently validated descendant
  -> new layered executable state
  -> transferable capability
```

The exact ancestor is causally necessary at two distinct levels:

```text
1. hypothesis-language availability
2. cumulative transfer stability
```

## What ΩD1 does not establish

The PASS does not establish:

- automatic discovery of transformation groups;
- learned invention of the descendant grammar itself;
- arbitrary-depth recursive representation growth;
- grammar mutation;
- natural-language concept formation;
- open-world semantic equivalence;
- safe production self-modification;
- AGI;
- consciousness;
- human-level cognition.

## No rescue iteration

ΩD1 passed under the frozen implementation and contract.

No transformation, fixture, outcome law, grammar, candidate count, root count, control, budget, tie-break, acceptance gate, or terminal-classification rule was changed after observing the result.

## Next justified experiment

The next justified experiment is no longer another fixed-grammar descendant instantiation.

A third-generation study should require a composition operator that is **absent from the current developer-supplied descendant grammar**, forcing the system to:

```text
observe a residual executable alias after two admitted generations
propose a bounded grammar extension
independently validate that extension against matched simpler grammars and rewired controls
admit the grammar extension only from proof-carrying evidence
use the newly extended language to synthesize a third-generation refinement
show that removing the grammar extension eliminates the capability under matched compute
```

That would test language growth rather than merely deeper search inside a fixed developer-supplied hypothesis language.
