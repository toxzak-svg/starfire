# Ω1 Endogenous State-Space Genesis — First Frozen Result

## Terminal classification

```text
PASS
```

## Frozen provenance

Preregistration commit, before implementation and before the first verdict-producing run:

```text
3088d4cdc34133cf30ee0253f951b9e1f84f907d
```

First verdict-producing executable head:

```text
b9e14a5b5129781c18fa0b780195aaa05c97c8ca
```

GitHub Actions:

```text
workflow: CHARGE CI
run id: 29148411318
run number: 104
job conclusion: success
artifact id: 8247459442
artifact digest: sha256:cbaaeda44b64fad80078e9e2a8ef5da44f192263a11225f7eba20a604f5105f0
```

No Ω1 grammar rule, fixture shape, threshold, family, root count, control, budget, acceptance gate, or terminal classification rule was changed after observing the result.

## Frozen objective results

### Training

```text
stateful_refinement          16/16 roots, 128/128 transfer predictions
all nine controls             0/16 roots,   0 correct transfer predictions
```

### Holdout

```text
stateful_refinement            8/8 roots, 64/64 transfer predictions
all nine controls               0/8 roots,  0 correct transfer predictions
```

### Future

```text
stateful_refinement          32/32 roots, 256/256 transfer predictions
all nine controls             0/32 roots,   0 correct transfer predictions
```

Every future family independently achieved:

```text
stateful success rate:        1.0
maximum control success rate: 0.0
maximum control correct predictions: 0
```

Future families:

```text
cellular_sequences
manufacturing_sequences
software_sequences
watershed_sequences
```

## Control behavior

The controls separated the role of executable state-language admission from nearby alternatives.

### Endpoint-only execution

The winning synthesized program was available for endpoint execution but was not admitted to `StateLanguage`.

Result:

```text
0 correct transfer predictions across training, holdout, and future
```

This is the central distinction between merely executing the discovered predicate and changing the executable representation seen by the downstream operation.

### Proposal text only

Preserving the synthesized proof/program serialization without admission produced:

```text
0 correct transfer predictions
```

### Scalar only

Preserving score, support, and margin without admission produced:

```text
0 correct transfer predictions
```

### Foreign certificate

Independent validation succeeded for the foreign-root refinement, but root-bound admission rejected it on every root:

```text
16/16 training rejections
8/8 holdout rejections
32/32 future rejections
```

### Counterfeit proof

The independently recomputed validator rejected every tampered proof:

```text
16/16 training
8/8 holdout
32/32 future
```

### Valid irrelevant refinement

A separately validated same-root executable refinement was admitted on every root but was irrelevant to the target representational defect:

```text
56/56 admissions
0 correct target transfer predictions
```

### Outcome-shuffled synthesis

The deterministic outcome permutation preserved counts and full search budget while destroying the target history/outcome incidence. No shuffled proof survived the frozen complete-repair certificate gate:

```text
0 validation successes across 56 roots
0 correct transfer predictions
```

### Deterministic non-winning valid refinement

A same-vocabulary, same-complexity refinement validated against a different behavioral partition and was admitted on every root:

```text
56/56 admissions
0 correct target transfer predictions
```

This is stronger than a no-op control: the state language genuinely changed, but the wrong valid distinction did not repair the target alias.

### Delayed correct admission

The correct certificate validated on every root and was admitted only after the transfer-prediction window:

```text
56/56 eventual admissions
0 correct transfer predictions during the frozen window
```

Correct information added too late did not retroactively create the required capability.

## Frozen gates

All preregistered gates passed:

```text
cohort_exact:                              true
base_alias_defects_exact:                  true
stateful_training:                         true
stateful_holdout:                          true
stateful_future:                           true
every_control_individual_prediction_zero: true
foreign_certificates_rejected_everywhere: true
counterfeit_proofs_rejected_everywhere:    true
valid_irrelevant_admitted_everywhere:      true
random_valid_admitted_everywhere:          true
shuffled_synthesis_rejected_everywhere:    true
all_future_families_transfer:              true
budgets_exact:                             true
replay_exact:                              true
invariants_hold:                           true
```

## Scientific interpretation

The strongest supported claim is narrow but materially different from H9–H11:

> Under the frozen symbolic sequence-history regime and fixed bounded synthesis grammar, Starfire detected that its current executable state language aliased histories with different intervention outcomes, synthesized a new executable state-key refinement from raw history structure without being given the useful atom pair, survived independent full recomputation, and required admission of that refinement before a later representation-bound operation could correctly distinguish withheld histories across unseen vocabularies.

The important positive result is not simply that a learned predicate predicted the labels.

The base `StateLanguage` deliberately mapped opposite-behavior histories to the same executable key. The downstream operation was representation-bound and therefore could not distinguish them. The winning synthesized predicate only created the new capability after certificate-gated admission changed the state key itself.

Three controls sharpen this interpretation:

1. endpoint execution of the winning predicate without state-language admission failed completely;
2. valid but irrelevant state-language changes were admitted and failed completely;
3. correct admission after the prediction window failed completely.

Within the frozen experiment, the causal object is therefore the **timely executable refinement of the state representation**, not possession of the program text, its scalar score, generic state mutation, or eventual access to the correct distinction.

## Claim boundary

Ω1 does **not** establish:

- unrestricted open-world ontology invention;
- natural-language representation repair;
- learned or self-modified synthesis grammar;
- unbounded invention of new primitive types;
- recursive descendant concept genesis;
- automatic live-runtime promotion safety;
- AGI;
- consciousness;
- human-level cognition.

The synthesis grammar remains developer-supplied and bounded. Ω1 demonstrates endogenous selection and admission of a new executable state-key dimension from witnessed representational failure, not unrestricted invention of the grammar that generates possible refinements.

## Next justified experiment

The next narrow experiment is **Ω2 descendant necessity**.

Ω2 should require:

```text
L0
  -> witnessed defect D1
  -> synthesize and admit Δ1
  -> L1 = L0 + Δ1
  -> new defect D2 becomes expressible/detectable only in L1
  -> synthesize and admit Δ2
  -> L2 = L1 + Δ2
```

The decisive ablation is:

```text
remove Δ1
  -> D2 is no longer expressible or discoverable
  -> Δ2 cannot be proposed
  -> descendant capability disappears
```

That would move from one-step state-space repair to experimentally demonstrated cumulative growth of the executable hypothesis language.
