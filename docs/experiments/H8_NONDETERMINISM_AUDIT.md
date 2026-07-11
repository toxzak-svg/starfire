# H8 seeded-replay nondeterminism audit

Status: post-run reproducibility audit. This document does not alter the H8 executable, frozen constants, continuation serialization, candidates, controls, gates, fixture, or decision logic.

## Why run 87 is not admissible

CHARGE CI run 87 was the first complete JSON emission, but pull-request review identified two implementation mismatches with the preregistered comparison semantics:

1. candidate selection used `left + 1e-12 >= right` although the contract said exact ties choose `R;C`;
2. positive fraction used `stateful > blind + 1e-12` although the contract defined strict `stateful > blind`.

A manual audit found the same unregistered epsilon relaxation on gate comparisons. These were corrected to the already-written contract. No numerical constant or mathematical operation changed.

Run 87 is retained as an invalid implementation trace, not a scientific result.

## Contract-exact run 89

The first contract-exact implementation ran at:

```text
head = 4802b630853ca0be98977ec348322329d157b7fe
CHARGE CI run = 89
run id = 29142558051
```

It completed compilation, deterministic CHARGE/cognitive-cycle/environment tests, preserved H4-H6 diagnostics, H8 execution, and artifact upload.

The H8 executable emitted `NOT_COMPOSABLE`.

## Reproducibility failure discovered by cross-run comparison

The contract corrections cannot alter resolver execution, continuation serialization, fixture generation, common-root seed, candidate words, or control execution.

Nevertheless, comparing the run-87 trace with contract-exact run 89 showed movement in actual component-path values, including:

```text
training C;R stateful score: 0.1500000000 -> 0.1750000000
training C;R blind score:    0.1916666667 -> 0.1833333333
future C;R stateful score:   0.1081439394 -> 0.1102272727
future C;R blind score:      0.1081439394 -> 0.1102272727
```

Reasoning-first continuation prompt byte means also changed. Those bytes contain the actual first resolver output. Therefore the resolver output itself changed across process runs.

This movement is not causally attributable to the comparison-rule corrections.

## Source-level nondeterminism

The real `ReasoningEngine` fallback can enter synthesis from multiple working-memory items. `AnalogyEngine::find_structural_mapping` constructs standard-library `HashSet<&str>` values and then performs:

```text
shared   = words_a.intersection(words_b).copied().collect()
a_unique = words_a.difference(words_b).copied().collect()
b_unique = words_b.difference(words_a).copied().collect()
```

The vectors are not sorted before `.first()` selects tokens used in the emitted analogy text.

Standard `HashSet` iteration order is not part of H8's seeded state and may vary between process executions. H8's `SEED xor anchor_id` resets the target environment, but it does not control the hash-builder state used by the actual reasoning component.

Thus the operation under test is not deterministic under the declared seed.

## Frozen audit action

This document-only commit intentionally leaves the H8 executable byte-for-byte unchanged from head `4802b630853ca0be98977ec348322329d157b7fe`.

The automatically triggered CHARGE CI run is a reproducibility audit, not a new primary trial and not a rescue iteration.

Audit criterion:

```text
compare the complete raw H8 JSON from run 89 to the new docs-only-triggered run
```

If any resolver-dependent score, prompt-byte diagnostic, eligibility count, candidate, or terminal metric changes, seeded replay is not controlling the operation under test.

The scientific disposition is then:

> H8 is experimentally invalid under its deterministic seeded-replay premise. Its emitted terminal classification must not be interpreted as evidence for or against compositional state transformation.

No H8 threshold, prompt, feature, control, candidate, or mechanism may be changed after this audit to rescue the experiment.

A deterministic resolver-semantics fix would be a separate foundation change and would require a newly preregistered experiment rather than rerunning H8 as though the original trial had succeeded.
