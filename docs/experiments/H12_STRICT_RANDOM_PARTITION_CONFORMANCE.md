# H12 Strict Random-Partition Conformance

Status: **frozen preregistration before implementation and before the first verdict-producing run**.

This is a narrow archival conformance experiment for the unmerged H12 proof-carrying latent structural-role result. It does not reopen or retune the H12 mechanism, and it does not supersede the later Ω1/ΩR1/ΩD1 representation-genesis line.

## Unresolved conformance item

The accepted H12 executable replay used a deterministic four-episode mixed grouping:

```text
2 target-role episodes + 2 causally irrelevant-role episodes
```

That control produced zero target successes, but the H12 preregistration used stronger wording:

```text
deterministic root-seeded grouping
same target-group cardinality as the real role projection
frozen construction forces target and adversarial evidence into the same group
```

The prior control therefore established the intended causal contrast but did not literally implement the stated root-seeded partition procedure.

## Frozen question

Does the H12 target path remain successful while a literal root-seeded, same-cardinality random partition fails to produce an admissible target rule on every frozen root?

## Frozen substrate

The replay uses the unchanged H12 synthetic construction:

```text
7 surface/topology families
8 roots per family
16 training roots
8 holdout roots
32 future roots
56 total roots
```

Each root contains:

```text
4 target-role evidence episodes
5 causally irrelevant-role evidence episodes
4 degree-matched control evidence episodes
13 total evidence episodes
```

The stateful reference path must recover the same four-episode target-role projection through target-blind structural fingerprint recurrence, independent exact role recomputation, unchanged H11 graph-frontier discovery, unchanged H10 scoring/validation, H11 certificate admission, and three canonical PECS closure scans.

## Strict root-seeded random partition

For each root:

1. Sort all 13 evidence episodes by `evidence_id`.
2. Compute two deterministic SplitMix64 values from the root id and frozen salts.
3. Select a cyclic start offset in `[0, 12]`.
4. Select a non-unit stride in `[2, 11]`.
5. Enumerate all 13 episode positions by the cyclic permutation:

```text
index_i = (start + i * stride) mod 13
```

6. Locate the canonical first evidence episode—the minimum `evidence_id`—inside that permutation.
7. Select the circular four-position block containing that episode, where four is independently measured from the validated target-role projection rather than hard-coded into the grouping function.
8. Send only those four inert episodes through the unchanged H11/H10 proposal and independent validation path.

The permutation algorithm may inspect only:

```text
root id
evidence id ordering
evidence count
target projection cardinality
```

It may not inspect:

```text
intervention identity
outcome identity
objective atom
PECS state
H11 score
certificate result
family or split label
```

The non-unit cyclic stride is frozen because the evidence construction places the four target episodes contiguously in canonical id order. Any four-position arithmetic progression with stride other than `+1` or `-1` must include at least one non-target episode. The conformance harness must verify this property independently on every root.

## Frozen executable paths

Exactly two outcome paths are evaluated per root:

### 1. Validated structural-role reference

- induce all recurrent structural fingerprints from four discovery graphs;
- independently recompute exact role membership;
- recognize role instances in the held-out transfer graph;
- project evidence for every validated role;
- admit only independently validated H11 certificates;
- execute exactly three PECS closure scans.

### 2. Strict random-partition control

- construct the frozen root-seeded four-episode block;
- run the same H11/H10 proposal and validation functions;
- provide one pre-closure admission slot only if a valid certificate exists;
- execute exactly three PECS closure scans.

No text, scalar summary, grouping identity, or failed proof is executable state.

## Frozen gates

The experiment is `PASS` only if all gates hold:

```text
cohort exact:                         56 roots
stateful training success:            16/16
stateful holdout success:             8/8
stateful future success:              32/32
strict random control success:        0/56
strict random target certificates:    0/56
random block cardinality exact:       56/56
random block includes target evidence:56/56
random block includes non-target:     56/56
random block unique membership:       56/56
root-seeded replay exact:             56/56
H11 accounting exact:                 56/56
PECS invariants hold:                 56/56
closure scans:                        exactly 3 per path
```

## Failure taxonomy

```text
INFRASTRUCTURE_FAILURE
REFERENCE_FAILURE
CARDINALITY_FAILURE
MIXING_FAILURE
CONTROL_FAILURE
REPLAY_FAILURE
PROVENANCE_FAILURE
PASS
```

## Claim boundary

A `PASS` closes only the literal H12 random-grouping conformance gap under the frozen 13-episode synthetic construction. It does not strengthen H12 into an open-world ontology result, does not reopen the H12 branch for live use, and does not establish natural-language concept induction, automatic ontology promotion, production routing readiness, AGI, consciousness, or human-level cognition.
