# H12 Strict Random-Partition Conformance Result

## Terminal classification

**PASS**

This result closes the literal control-conformance gap documented in the archived H12 proof-carrying latent structural-role experiment. It does not reopen H12 as a live subsystem and does not supersede the later Ω1/ΩR1/ΩD1 representation-genesis research line.

## Frozen preregistration

The conformance contract was committed before implementation and before the first verdict-producing run:

```text
preregistration file:
  docs/experiments/H12_STRICT_RANDOM_PARTITION_CONFORMANCE.md

preregistration commit:
  c530177a0a6a93a19ef83dfaba90c24c55c00d7e
```

The frozen partition uses only root identity, canonical evidence-id order, evidence count, and the independently measured target-role projection cardinality. It does not inspect intervention identity, outcomes, objective state, H11 score, certificate status, family, or split.

## First non-verdict infrastructure run

```text
workflow: H12 Strict Random-Partition Conformance #1
run id:   29216173082
head:     0d7217553b61d7b37e0349314c04076510afd76a
result:   formatting check failure before compilation or experiment execution
```

No scientific classification was emitted. The source was formatted without changing the preregistered partition, worlds, evidence, gates, inference laws, or objective.

## First verdict-producing run

```text
workflow: H12 Strict Random-Partition Conformance #2
run id:   29216241558
head:     2f8cb8039671bd20a56acdbd1f31165740564970
workflow conclusion: success
terminal classification: PASS

artifact id:     8266677920
artifact digest: sha256:14a90a22d147d9916d031fd5e800f64cca5c03d0629356f835052ef8e388b645
```

The workflow completed:

```text
rustfmt of the CI workspace source
example compilation
H9 commitment-state tests
H10 rule-induction tests
H11 graph-discovery tests
frozen 56-root conformance replay
artifact preservation
```

The exact rustfmt output from this run was subsequently committed without semantic modification. The final branch workflow restores `rustfmt --check` so the committed source, not a mutated CI workspace, is the merge-gated artifact.

## Frozen objective results

```text
validated structural-role reference:
  training: 16/16
  holdout:   8/8
  future:   32/32

strict root-seeded random-partition control:
  training: 0/16
  holdout:  0/8
  future:   0/32

strict control target certificates:
  training: 0/16
  holdout:  0/8
  future:   0/32
```

Every future family transferred exactly:

```text
cellular:      stateful 8/8, strict control 0/8
manufacturing: stateful 8/8, strict control 0/8
software:      stateful 8/8, strict control 0/8
watershed:     stateful 8/8, strict control 0/8
```

## Exact conformance gates

All gates were true on every root:

```text
cohort exact:                           56/56
same target-group cardinality:          56/56
strict group contains target evidence:  56/56
strict group contains non-target:       56/56
strict group membership unique:         56/56
root-seeded replay exact:               56/56
H11 proposal/validation accounting:     56/56
independent role recomputation exact:   56/56
PECS state/provenance invariants:        56/56
three-scan closure budget exact:         56/56
```

## Interpretation

The stronger literal control does not recover the H12 effect. A deterministic root-seeded cyclic partition with the same four-episode cardinality as the validated target-role projection always mixed target and non-target evidence, never produced the target H11 certificate, and never enabled the held-out PECS objective. The independently recomputed structural-role partition retained complete reference success.

This closes the previously disclosed discrepancy between the intended random-grouping control and its earlier approximate implementation.

## Claim boundary

This `PASS` supports only the following narrow statement:

> Under the frozen 13-episode synthetic H12 construction, the validated structural-role evidence partition enables executable transfer, while a literal root-seeded same-cardinality random partition does not.

It does not establish open-world ontology induction, natural-language concept discovery, continuous latent learning, automatic ontology promotion, live routing readiness, AGI, consciousness, or human-level cognition.
