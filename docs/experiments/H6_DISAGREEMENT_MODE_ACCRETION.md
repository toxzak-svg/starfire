# H6 Disagreement-Mode Accretion

## Status

**REJECTED**

Diagnostic command:

```bash
cargo run -p star --example h6_disagreement_contrast_probe --locked
```

This experiment does not promote a concept, modify live routing, or integrate CHARGE into `Runtime`.

## 1. Structural diagnosis

The CHARGE cycle preserves unresolved magnitude and persistence, but not evolving unresolved geometry.

A resolver judgment can subtract accepted discharge from a charge and leave a smaller pending charge. The pending charge's residual representation is otherwise preserved. `CycleObservationRecorder` counterfactually replays every resolver from the same initial charge and records a static resolver-outcome matrix. Empirical ontology induction then searches predicates over the feature vocabulary already supplied to it.

The missing capability investigated here was therefore a **pre-concept interaction operator**: a state transition by which unresolved structures could change one shared internal representation before a developer-authored coordinate or promoted concept supplied the distinction.

The experiment asked whether independently judged resolver disagreement was the right pressure signal for such an operator.

A separate architectural caveat remains important: the CHARGE/cognitive-cycle path is still an experimental foundation and is not the state transition governing the main `Runtime` reasoning loop.

## 2. The invented primitive

The revised primitive is **disagreement-mode accretion**.

For an unresolved observation `i`, let:

- `x_i` be its H5 fixed-width, mask-blind residual projection;
- `u_i(r)` be the measured discharge fraction per compute cost for resolver `r`;
- `p_i` be `u_i` normalized into a resolver-preference distribution.

A pair `(i,j)` enters the interaction schedule only when:

```text
argmax(p_i) != argmax(p_j)
TV(p_i, p_j) >= 0.25
```

For each admitted pair:

```text
d_ij = (x_j - x_i) / ||x_j - x_i||
M <- M + d_ij d_ij^T
```

At most 64 pair interactions are accumulated into the same matrix `M`.

After accumulation:

```text
v   = deterministic dominant eigenvector(M)
tau = mean_ij 0.5 * (v·x_i + v·x_j)
```

The training cohort is partitioned by:

```text
v·x <= tau
v·x >  tau
```

The best empirical resolver is relearned independently on each side. The resulting probe is executable only if both sides satisfy frozen training and holdout support and the independent holdout gain is at least `0.04`.

The changed internal state is `M`, summarized by the resulting `TensionContrast { axis, threshold, source_pair_count, dominant_eigenvalue_fraction, ... }`.

The intended new future behavior was routing along an oblique residual direction that did not exist in the original H5 axis-aligned predicate vocabulary.

## 3. Why this is not an existing pattern with a new name

It is not memory. No task text, target answer, fact, or prior output is stored for later retrieval.

It is not clustering. There is no unsupervised partition followed by an arbitrary concept label. Resolver-outcome disagreement admits interactions; those interactions alter one second-order state; the state yields an executable projection.

It is not ordinary routing. A router chooses among resolvers using an existing representation. This operation attempts to manufacture a new projection direction before the side-specific routes are learned.

It is not reflection or planning. It generates no critique, chain of thought, plan, subgoal, retry loop, or textual self-description.

It is not the current empirical ontology inducer. The current inducer searches threshold predicates over supplied residual coordinates, persistence, and trace. Disagreement-mode accretion does not promote a concept and does not search one supplied coordinate at a time.

The mathematical ingredients are not claimed as novel mathematics. The revised operator is a disagreement-conditioned second-moment accumulation followed by dominant-eigendirection extraction. Its research novelty, if any, would have been its architectural placement: unresolved resolver disagreement becoming a persistent representational state before concept induction. The experiment did not validate that placement.

## 4. Implementation

Changed in the H6 research line:

- `lib/charge/contrast.rs`
  - `ContrastProbeConfig`
  - `TensionContrast`
  - `LearnedContrastProbe`
  - `ContrastProbeFit`
  - `disagreement_pair_schedule`
  - `valid_contrast_pairs`
  - `fit_disagreement_contrast`
  - `fit_contrast_from_pairs`
  - shared second-moment accumulation and deterministic power iteration
- `lib/charge/mod.rs`
  - exports the contrast experiment APIs
- `lib/examples/h6_disagreement_contrast_probe.rs`
  - real-subsystem observation collection
  - frozen H4 memory exclusion
  - H5 fixed projection boundary
  - task-profiled verifier judgments
  - matched controls
  - future transfer evaluation
  - static H5 comparison path
  - JSON diagnostics and frozen gates
- `.github/workflows/charge-ci.yml`
  - runs H6 and preserves the report artifact
  - preserves the known H5-B `NotIdentifiable/pass:false` verdict without allowing unrelated H5-B failures through

The original H6 implementation in PR #19 generated one candidate axis per disagreement pair and selected the best candidate. Its first frozen run rejected the mechanism. The precise defect was conceptual: 64 disagreements were evaluated, but they never interacted or changed one shared state.

The directive allowed one revision when the first implementation exposed a precise conceptual defect. PR #20 replaced isolated pair candidates with the shared `M <- M + d d^T` state transition. No H6 thresholds or transfer gates were relaxed.

## 5. Falsification contract

Frozen before the first H6 run:

| Contract item | Value |
|---|---:|
| pair interactions | 64 |
| minimum preference TV distance | 0.25 |
| minimum training support per side | 12 |
| minimum holdout support per side | 6 |
| training complexity penalty | 0.003 |
| minimum holdout gain | 0.04 |
| minimum future/baseline efficiency ratio | 1.20x |
| future windows that must beat baseline | 100% |
| minimum future resolver-leader accuracy | 0.75 |
| minimum efficiency ratio over every structure-destroying control | 1.10x |
| minimum leader-accuracy margin over strongest control | 0.15 |

The hypothesis was false if disagreement-conditioned state could not produce a holdout-valid probe, failed future transfer, or survived only when the same effect also appeared under equal-budget structure-destroying controls.

The experiment was required to distinguish "more state" from "a new reasoning operation."

## 6. Results

### Observation cohort

| Measurement | Count |
|---|---:|
| real subsystem-backed CHARGE observations | 252 |
| task-profiled judged resolver attempts | 1,260 |
| excluded by frozen H4 memory predicate | 84 |
| retained non-memory observations | 168 |
| non-memory training observations | 48 |
| non-memory holdout observations | 24 |
| non-memory future observations | 96 |

The frozen H4 predicate excluded all 84 hidden `KnowledgeGap` observations. The retained non-memory cohort contained 84 `PredictionContradiction` and 84 `QuanotTrajectory` observations.

The global parent baseline resolver was `causal`.

### First implementation: isolated pair contrasts

The first implementation was rejected before the conceptual revision:

- real primitive applied: `false`
- 64 pair evaluations / 64 unique pairs
- future efficiency: `0.5029411764`
- baseline efficiency: `0.5029411764`
- future/baseline ratio: `1.000000`
- future resolver-leader accuracy: `0.500000`
- future window win fraction: `0.000000` (`0/4`)

The result exposed the implementation's conceptual defect: the pair axes competed, but unresolved disagreements never accumulated into changed shared state.

### Revised implementation: shared disagreement mode

The one permitted revision also failed.

Real fit diagnostics:

| Measurement | Value |
|---|---:|
| baseline training efficiency | 0.5100000001 |
| baseline holdout efficiency | 0.5000000000 |
| best training efficiency | 0.5100000001 |
| best training gain after penalty | 0.0000000000 |

The shared disagreement mode did not improve training over the causal baseline. Consequently it was not holdout-applied and emitted no executable `real_probe`.

Future metrics:

| Measurement | Value |
|---|---:|
| applied | false |
| pair evaluations | 64 |
| unique pairs | 64 |
| efficiency | 0.5029411764 |
| baseline efficiency | 0.5029411764 |
| baseline ratio | 1.000000 |
| resolver-leader accuracy | 0.500000 |
| future window win fraction | 0.000000 |
| worst future-window ratio | 1.000000 |

All four future task families remained exactly at the parent baseline route.

### Best existing static H5 path

The static H5 inducer promoted one route:

```text
ResidualThreshold {
    dimension: 2,
    threshold: 0.088675275,
    direction: AtLeast,
} -> reasoning
```

In the fixed H5 projection, dimension `2` is residual standard deviation.

Static H5 future metrics:

| Measurement | Value |
|---|---:|
| promoted concepts | 1 |
| candidates considered | 262 |
| efficiency | 0.5582386374 |
| baseline efficiency | 0.5029411764 |
| baseline ratio | 1.1099481680 |
| resolver-leader accuracy | 1.000000 |
| future window win fraction | 1.000000 (`4/4`) |
| worst future-window ratio | 1.0909090937 |

Per future family:

| Family | Efficiency | Baseline | Ratio | Leader accuracy |
|---|---:|---:|---:|---:|
| compiler / borrow checker / packet loss | 0.5454545469 | 0.5000000000 | 1.0909090937 | 1.000000 |
| mitochondria / glass / pipe pressure | 0.5750000016 | 0.5117647056 | 1.1235632221 | 1.000000 |
| HTTP 404 / seasons / combustion | 0.5500000010 | 0.5000000000 | 1.1000000021 | 1.000000 |
| index / bats / friction | 0.5625000000 | 0.5000000000 | 1.1250000000 | 1.000000 |

Static H5 did not meet H6's deliberately harsher `1.20x` future ratio gate, but it was materially and consistently stronger than the proposed primitive.

## 7. Ablations

### Matched random valid-pair accumulator

The random-pair control received the same 64 pair interactions and identical accumulator/fitter/holdout gate.

Fit diagnostics:

- best training efficiency: `0.5375000016`
- best training gain after penalty: `0.0245000016`
- holdout-applied: `true`

Future metrics:

- efficiency: `0.5061553040`
- future/baseline ratio: `1.0063906631`
- resolver-leader accuracy: `0.9479166667`
- future window win fraction: `0.75`
- worst future-window ratio: `0.9607279729`

This is the most damaging control result. Random valid residual interactions manufactured a more useful mode than resolver-disagreement-conditioned interactions. It still failed the H6 transfer contract because the efficiency effect was negligible and one future family regressed.

### Shuffled resolver-outcome pair selection

Whole resolver-outcome vectors were shuffled across observations before disagreement pair selection. The same fitter then consumed the selected pair indices on the true training observations.

Result:

- holdout-applied: `false`
- future/baseline ratio: `1.000000`
- resolver-leader accuracy: `0.500000`
- future window wins: `0/4`

### Independently permuted residual dimensions

The real disagreement pair schedule was retained, but every visible residual dimension was independently shuffled across observations in train, holdout, and future windows.

Result:

- holdout-applied: `false`
- future/baseline ratio: `1.000000`
- resolver-leader accuracy: `0.500000`
- future window wins: `0/4`

All frozen H6 gates for the real primitive were `false`.

## 8. Interpretation

**REJECTED**

The proposed endogenous transition was not detected.

Experience did create an explicit second-order state under the revised algorithm, but resolver disagreement did not organize the residual geometry into a useful training distinction. The state therefore failed before independent holdout activation and could not change future routing.

The structure-destroying controls do not rescue the theory. The random-pair control is stronger evidence against it: under the same pair/state budget, ignoring resolver disagreement generated a mode with much higher resolver-leader accuracy. The hypothesized pressure signal was worse than random pair selection for identifying useful residual geometry on this cohort.

The result does not show that unresolved-state interaction is impossible. It shows that **resolver-preference disagreement, transformed through normalized pair displacement second moments, is not the missing operation demonstrated by these Starfire observations**.

## 9. The uncomfortable implication

The strongest reusable distinction in the non-memory H5 cohort is currently simpler than the proposed reasoning primitive: a scalar threshold on **residual standard deviation** perfectly predicts the winning resolver across all four future task families.

That is uncomfortable in two directions.

First, the H5 "reasoning-favored regime" may not be evidence of a latent reasoning identity at all. It may be a stable heteroscedasticity difference between the residual geometries emitted by prediction contradictions and Quanot trajectories under this fixture and task-profiled verifier.

Second, resolver disagreement is not automatically epistemic structure. In H6 it was an actively bad selector of pair geometry relative to random valid pairs.

Starfire's current roadmap risks treating resolver-regime separability as concept formation before proving that the separating feature is representation-invariant or causally related to reasoning utility.

## 10. Next experiment

Run exactly one **basis-invariance intervention on the H5 fixed residual representation**.

Before ontology fitting, apply a frozen deterministic random orthogonal rotation to every fixed-width residual vector in train, holdout, and future cohorts. Give the static H5 inducer the rotated coordinates with the same candidate/complexity/support/holdout budgets. Compare it with:

1. the unrotated H5 path;
2. an equal-budget projection search that can form one learned linear direction;
3. a label/outcome-shuffled rotated control.

The decisive question is whether the `stddev >= ~0.088675` transfer effect is a genuine property of unresolved residual structure or an accident of the developer-chosen fixed feature basis.

If axis-aligned H5 utility collapses under orthogonal rotation while an equal-budget learned-direction path recovers transfer and shuffled outcomes do not, Starfire has evidence for **representation necessity** rather than ontology identity. If the effect survives arbitrary basis changes without a learned direction, the current interpretation of the H5 feature/result is wrong and the measurement pipeline must be re-examined.

Do not add automatic concept promotion or live routing before this basis-invariance question is answered.
