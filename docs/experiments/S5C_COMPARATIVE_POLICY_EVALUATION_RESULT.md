# S5-C Comparative Policy Evaluation Result

Status: **PASS — synthetic evaluator conformance only**

## Authoritative committed-source run

The first complete run after sparse-evidence arithmetic hardening compiled the evaluator and probe, executed the deterministic contract tests, and produced the frozen verdict:

```text
workflow: Companion Policy Evaluation S5-C CI
run id:   29228321094
head:     13da53cc01d56ec7d60bf1a4ba635d076ed824d6
verdict:  PASS
artifact: 8270587975
artifact digest: sha256:78918450cabaffb24e1185c2729887faa06cbc50b891b79f3edc9a195fe33b0d
```

## Frozen result

```text
holdout candidate-control comparisons: 10
all holdout evidence gates:             passed
all holdout performance gates:          passed
development excluded from verdict:      true
opaque-subject holdout present:          true
temporal holdout present:                true
deterministic replay:                    true
source companion state unchanged:        true
promotion eligible:                      true
gate passed:                             true
```

Every authority flag remained false:

```text
live response influence:    false
routing authority:          false
belief-promotion authority: false
action authority:           false
```

## Arithmetic and lint disposition

Sparse splits now return `None` for means and rates with zero denominators rather than evaluating a division. A dedicated regression test covers the zero-evidence path.

Clippy reports four `manual_checked_div` notices for these explicit zero-guard helpers. The S5-C workflow records that exact named exception while failing on every other diagnostic attributed to the evaluator or probe. Repository-wide legacy warnings remain outside this experiment's scope.

## Interpretation

This `PASS` establishes that the S5-C implementation correctly constructs the frozen synthetic split, metric, comparison, replay, and verdict contracts. It does **not** establish that the companion-derived policy improves real conversations.

`promotion_eligible = true` means only that the synthetic evaluator would permit a separately reviewed S6 experiment under its frozen thresholds. It does not authorize automatic promotion, `Runtime::chat()` influence, response routing, persistent policy selection, belief or ontology mutation, or autonomous action.
