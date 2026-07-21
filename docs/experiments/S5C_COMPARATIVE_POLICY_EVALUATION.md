# S5-C — Comparative Policy Evaluation

## Purpose

S5-C evaluates the S5-A companion-derived interaction policy against all five matched controls using only frozen S5-B trials and independently witnessed S4 outcomes.

It answers a narrower question than live personalization:

> Does the companion-derived policy clear predeclared held-out performance and non-regression gates strongly enough to become eligible for a later, reversible S6 live-use experiment?

A passing result does not itself modify generated text or grant runtime authority.

## Inputs

The evaluator accepts:

- the replayable `InteractionOutcomeLedger` produced by S5-B;
- the mirrored S4 prediction records and exact Brier scores;
- one positive compute-time observation for every trial arm;
- a frozen split policy and frozen comparison thresholds.

Missing, duplicate, zero, or unknown compute observations invalidate evaluation rather than being silently ignored.

## Split discipline

Every trial is assigned from pre-outcome metadata only:

1. trials issued at or after the frozen temporal boundary enter `TemporalHoldout`;
2. earlier trials whose opaque subject digest matches the frozen modular partition enter `OpaqueSubjectHoldout`;
3. all remaining trials enter `Development`.

Temporal assignment takes precedence over the subject partition. Development metrics are reported, but development evidence is structurally excluded from the verdict.

## Per-arm metrics

For each of the six S5-A variants and each split, S5-C reports:

- trial, prediction, resolution, pending, expiration, and abstention counts;
- total and mean multiclass Brier score;
- mean top-label calibration error;
- direct observed outcomes;
- positive outcomes, corrections, clarification requests, completions, and abandonments;
- candidate-relative pairwise wins, losses, and ties;
- total and mean compute cost;
- correction, clarification, completion, abandonment, and abstention rates.

Direct burden metrics remain tied to the actually delivered arm. Offline paired judgments remain a separate evidence channel.

## Candidate-control gates

The companion-derived arm is compared separately with:

- neutral default;
- recency only;
- majority prior;
- context only;
- scrambled scope.

Each comparison is repeated on both holdouts. A comparison first requires minimum resolved, direct, and pairwise evidence. It then checks:

- minimum Brier improvement;
- calibration non-regression;
- minimum candidate pairwise win margin;
- correction non-regression;
- clarification non-regression;
- completion non-regression;
- abandonment non-regression;
- abstention non-regression;
- bounded compute overhead.

The global verdict is:

- `INCONCLUSIVE` when any holdout comparison lacks its minimum evidence;
- `FAIL` when evidence is sufficient but any performance or non-regression gate fails;
- `PASS` only when all ten held-out candidate-control comparisons pass every gate.

`promotion_eligible` is true only for `PASS`. It means eligible for a separately reviewed S6 experiment, not automatically promoted.

## Frozen probe

The executable fixture creates development, opaque-subject holdout, and temporal-holdout trials for all six arms. It includes:

- one independently observed completion for every arm in every split;
- one pure-shadow candidate-versus-control comparison for each control in every split;
- complete per-arm compute observations;
- exact repeated evaluation for deterministic equality;
- source companion-state immutability checks.

The synthetic fixture is constructed so the companion-derived arm clears the frozen thresholds. This validates evaluator mechanics, split isolation, arithmetic, evidence accounting, and gate composition. It is not evidence that the current companion policy improves real conversations.

## Authority boundary

S5-C has no:

- `Runtime::chat()` integration;
- response rendering or response-selection authority;
- routing or CHARGE authority;
- companion-state mutation;
- persistence authority;
- belief or ontology promotion;
- autonomous side-effect authority.

S6 must remain a separate feature-gated change with explicit budgets, audit records, rollback, sensitive-claim exclusion, and immediate fallback to the neutral policy.
