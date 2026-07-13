# ΩD1 Transport-Certified Descendant Necessity — Corrected First Executable Result

## Terminal classification

```text
CONTROL_FAILURE
```

The earlier `PASS` record was not produced by the committed repository code. The committed ΩD1 example contained an unclosed `main()` delimiter, and the original GitHub ΩD1 workflow stopped during `cargo check --all-targets` before the experiment executable ran.

This document supersedes the previous `PASS` classification.

## Frozen preregistration

The preregistration remains unchanged:

```text
docs/experiments/OMEGAD1_TRANSPORT_CERTIFIED_DESCENDANT_NECESSITY.md
26e7c836200e31ccc3b8cdfdfe3755428ab7d619
```

No ΩD1 fixture, transformation, outcome law, grammar, candidate count, threshold, control, budget definition, or terminal-classification gate was weakened to obtain this corrected result.

## Why the previous result is invalid

The merged ΩD1 pull request identified this head:

```text
8c3a981abd280bb0ef72419ef3a88332c4434586
```

Its GitHub ΩD1 workflow was:

```text
workflow: ΩD1 CI #4
run id:   29160083556
result:   failure during "Compile all star targets"
```

The experiment, prerequisite replays, and result artifact steps were skipped. Therefore the repository had no committed-code GitHub verdict supporting `PASS`.

The defect was:

```text
lib/examples/omega_d1_transport_certified_descendant_necessity/part2.rs
main() ended with Ok(()) but had no closing }
```

Additional feature-gated examples also lacked Cargo `required-features` metadata, causing default all-target builds to compile examples while their modules were disabled.

## First verdict-producing committed-code run

After repairing the syntax and manifest contracts, the first full committed-code ΩD1 execution was:

```text
workflow: ΩD1 CI #101
run id:   29220130707
head:     e2fd75d333086c589b27fa7e491b23faad94c4da
artifact: 8267967435
artifact digest:
  sha256:0443576961098dc4e6598c0f46e83bb590e6143ef1be6e30fdde50a684f3eec7
```

The workflow successfully completed:

```text
cargo check -p star --all-targets --locked
all feature-gated probe compilation with declared features
representation_genesis tests
representation_transport_orbit tests
representation_transport_descendants tests
Ω1 prerequisite execution: PASS
ΩR1 prerequisite execution: PASS
ΩD1 execution: CONTROL_FAILURE
artifact preservation
```

## Capability result that did reproduce

The primary transport-certified descendant chain succeeded perfectly:

```text
training: 16/16 roots, 512/512 predictions
holdout:   8/8 roots, 256/256 predictions
future:   32/32 roots, 1,024/1,024 predictions
```

Every future family reproduced:

```text
cellular_cascade:      256/256
manufacturing_cascade: 256/256
software_cascade:      256/256
watershed_cascade:     256/256
```

The stationary-calibrated chain retained the expected partial transfer:

```text
16/32 correct per root
```

The negative controls retained zero target predictions, including:

```text
L0 raw search
L0 descendant search without an ancestor
ancestor certificate payload only
wrong transport-certified ancestor
exact ancestor replacement before descendant validation
descendant payload only
counterfeit descendant proof
delayed descendant admission during the prediction window
```

## Failed frozen gates

Two top-level gates were false:

```text
structural_audits_exact = false
budgets_exact           = false
```

The structural audit localized the mismatch to:

```text
roots:                          56
correct transport frontier:    56/56 exact
stationary transport frontier: 56/56 exact
wrong transport frontier:      0/56 exact
all remaining root audits:     56/56 exact
```

Budget exactness was false on the two paths that use the wrong transport-certified ancestor:

```text
wrong_transport_certified_ancestor
exact_ancestor_replaced_before_descendant_validation
```

Those controls still behaved causally as intended:

```text
wrong ancestor admissions succeeded:       56/56
wrong descendant validations rejected:     56/56
exact replacement validations rejected:    56/56
target predictions from those controls:    0
replay exact:                               true
state/provenance invariants:                true
```

However, ΩD1 preregistered exact frontier and matched-budget accounting. Behavioral rejection alone does not satisfy those gates.

## Correct scientific interpretation

The corrected run supports a narrower observation:

> Under the frozen pair-block construction, the correct transport-certified ancestor and descendant chain produced perfect held-out cumulative transfer, while all negative controls produced the expected target-level failures. However, the wrong-ancestor control did not satisfy the preregistered exact transport-frontier and budget-accounting contract. Therefore ΩD1 is not accepted as a full proof-carrying descendant-necessity result.

The terminal classification remains:

```text
CONTROL_FAILURE
```

No expected constant should be changed merely to restore the historical `PASS` claim. A future repair must explain the wrong-frontier discrepancy, preregister any corrected control construction before execution, and preserve matched compute, independent validation, exact replay, and held-out transfer.

## Claim boundary

This correction does not invalidate the separately reproduced Ω1 or ΩR1 results. It invalidates only the claim that the committed ΩD1 experiment passed every frozen gate.
