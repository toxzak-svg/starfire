# H7 Witnessed Congruence Split

Status: frozen diagnostic contract before first GitHub Actions run.

H7 does not repair H6 and does not promote concepts into live routing. H6 remains a rejected real closed-cycle result.

## Hypothesis

The current one-step behavioral identity of unresolved CHARGE histories may be too coarse. A resolver continuation may falsify the claim that two histories are the same computational state even when their complete one-step terminal witness signatures agree.

The primitive is a witnessed quotient refinement:

```text
C' = C union {w}
equivalence_C' = equivalence_C intersect kernel(F_w)
```

`w` is a length-two, non-repeating resolver word actually executed against one environment episode and one persistent `CognitiveCycleState`. The environment is reset once before the word and is not reset between its two resolver actions.

The fitting API is intentionally unable to receive residual coordinates, CHARGE kind, scope, hidden event class, task-family name, target answer, verifier evidence strings, or resolver-leader labels.

## Existing observation boundary

H7 reuses the frozen H6 cohort construction:

- 252 real subsystem-backed CHARGE observations;
- five independently judged one-step resolver attempts per observation;
- frozen H4 memory predicate exclusion;
- expected 168 retained non-memory observations;
- 48 training observations;
- 24 independent promotion holdout observations;
- 96 later future-transfer observations in four task-family windows.

The one-step matrix remains the parent behavioral identity. H7 adds state-threaded length-two continuation observations only for the retained non-memory cohort.

## Frozen constants

```text
SEED = 0x4837_5743_5350_4C54

MAX_CONTEXT_LENGTH = 2
ALLOW_SELF_REPEAT = false
PROPOSAL_BUDGET = 20
MAX_PROMOTED_SPLITS = 1
MAX_SIGNATURE_CLASSES = 16

WITNESS_DEADZONE = 0.05
WITNESS_STRONG_BOUNDARY = 0.25

MIN_TRAIN_SIGNATURE_SUPPORT = 8
MIN_HOLDOUT_SIGNATURE_SUPPORT = 4

COMPLEXITY_PENALTY = 0.02
MIN_TRAIN_DEFECT_GAIN_AFTER_PENALTY = 0.15

MIN_HOLDOUT_DEFECT_GAIN = 0.10
MAX_HOLDOUT_DEFECT_RATIO = 0.75

MIN_FUTURE_DEFECT_GAIN = 0.10
MAX_FUTURE_DEFECT_RATIO = 0.80
MIN_FUTURE_WINDOW_WINS = 4

MIN_CONTROL_DEFECT_MARGIN = 0.05
```

These values are frozen before the first CI execution of H7. A failed result must not be rescued by changing them in this experiment.

## Terminal witness

For raw `OutcomeWitness` evidence, directed normalized motion is

```text
higher-is-better: (after - before) / max(1, |before|, |after|)
lower-is-better:  (before - after) / max(1, |before|, |after|)
```

The value is not clamped at zero. Worsening objective motion remains observable.

The movement bin is:

```text
r < -0.25          -> StrongWorse
-0.25 <= r < -0.05 -> Worse
-0.05 <= r <= 0.05 -> Flat
0.05 < r <= 0.25   -> Better
r > 0.25           -> StrongBetter
```

The terminal witness is movement plus `Persisted` or `Resolved` disposition.

Compute cost is preserved for budget accounting but does not participate in behavioral equality.

## Parent state

The parent partition is the complete five-resolver one-step terminal-witness signature. Therefore H7 cannot call a one-step resolver preference a new distinction.

## Candidate continuations

The resolver alphabet is frozen to:

```text
reasoning
memory
causal
prediction
metacognition
```

Every ordered non-repeating length-two word is executed:

```text
5 * 4 = 20 proposals
```

For candidate `w`, the audit continuation set excludes `w` and `reverse(w)`.

The congruence defect of partition `P` is the fraction of comparable same-class history pairs whose terminal witnesses disagree under the audit continuations.

Candidate gain is:

```text
parent_defect - split_defect - 0.02
```

The fitter chooses the maximum gain with deterministic resolver-word tie-breaking.

## Promotion boundary

A candidate must first achieve training gain after penalty of at least `0.15`.

The word, one-step parent signature map, terminal-signature child map, and all constants are frozen before independent holdout application.

Unseen signatures abstain to the frozen parent class.

Holdout requires:

```text
absolute defect gain >= 0.10
defect ratio <= 0.75
at least two promoted child classes with support >= 4
```

A holdout failure means the observer is not promoted even diagnostically.

## Future transfer

A holdout-promoted observer is applied without refitting to the four later task-family windows.

Future transfer requires:

```text
absolute defect gain >= 0.10
defect ratio <= 0.80
window wins = 4/4
```

## Basis intervention

For residual width `d`, define

```text
v = normalize([1, 2, ..., d])
Qx = x - 2 v (v dot x)
```

This is a dense Householder reflection with condition number one.

The H7 fitting API never receives the residual payload. The original and basis-transformed diagnostic fits must therefore be exactly equal. Any difference is an implementation bug or illicit representation dependency.

## Outcome-destroyed control

For each continuation word and cohort split separately, permute the complete `(terminal witness, compute cost)` bundle across anchor IDs with a frozen seeded Fisher-Yates shuffle.

This preserves continuation marginals and compute marginals while destroying which continuation behavior belonged to which unresolved history.

Real future defect gain must exceed the outcome-destroyed control by at least `0.05`.

## More-state-but-wrong-operation control

The Independent Ledger Closure control receives the complete one-step terminal measurements, accepted discharge fractions, CHARGE magnitude semantics, and one-step compute costs.

For word `ab`, it predicts remaining unresolved magnitude by scalar closure:

```text
remaining_ab = (1 - accepted_fraction_a) * (1 - accepted_fraction_b)
predicted_motion = 1 - remaining_ab
```

This operation is commutative and cannot represent a genuine state-threaded continuation effect.

The synthetic terminal table is passed through the identical WCS fitter and holdout boundary.

Real future defect gain must exceed Ledger Closure by at least `0.05`.

## Right-absorption diagnostic

Report:

```text
R_absorb = P[F_ab(h) = F_b(h)]
```

The current H6 task verifier overwrites objective progress with the current action score, and the experimental component adapters do not consume preceding verifier state. A high right-absorption rate is therefore an expected killing result.

## Interpretation

A pass supports only this claim:

> A state-threaded resolver continuation reproducibly distinguishes unresolved histories that are indistinguishable by the complete one-step resolver-outcome signature, and the continuation-generated quotient predicts independent continuation behavior across later task families while surviving a true residual basis intervention and the frozen controls.

It is not an AGI, consciousness, semantic-concept, or causal-representation claim.

A clean failure means Starfire's current experimental dynamics may not instantiate higher-order resolver state for ontology induction to recover. In that case the architecture should stop repairing residual ontology machinery and first create genuinely state-threaded cognitive operations whose later behavior depends on earlier transformations.
