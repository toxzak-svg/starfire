# H5 residual-identity diagnostic plan

## Status

Partially implemented. H5-A and H5-B diagnostic execution are complete in
`h5_residual_identity_diagnostic`.

Latest local committed result: `754ba148` (`Improve H5 verifier task profiling`).

Current verdict:

- H5-A fixed-width residual projection diagnostic: passed
- original surface-verifier H5-B matrix: failed
- task-profiled H5-B matrix: passed
- H5-C ontology induction: not implemented yet

The original surface-verifier H5-B failure remains evidence that the verifier
ecology matters. It is no longer the gating matrix for H5-C. H5-C may proceed
only against the task-profiled verifier contract now exposed through
`star::charge::verifier`.

This plan follows the frozen H4 real closed-cycle rejection recorded on PR #14.

H4 established three facts that must be kept separate:

1. the closed-cycle evidence path worked over real Starfire emitters and real component outputs
2. one opaque memory-shaped distinction transferred strongly to unseen task families
3. the stronger latent-ontology claim failed because only one concept promoted and the exact matched-budget permuted-feature control retained almost all future utility

H5 is not an attempt to make H4 pass. H4 remains rejected.

H5 diagnoses why the feature-destroyed control remained competitive and whether any independently recoverable distinction exists after removing the easy memory split. The answer so far is split: fixed-width projection exposed residual-shape leakage, and the non-memory distinction appears only when the verifier distinguishes contradiction correction from causal-mechanism resolution.

---

## H4 result that motivates H5

The frozen H4 real-cycle run observed:

- 252 real subsystem-backed CHARGE observations
- 1,260 independently judged resolver attempts
- one promoted concept
- future routing efficiency approximately `1.9942x` the undifferentiated baseline
- `4/4` future-window wins
- approximately `1.9770x` worst-window retention versus baseline
- only `1.0243x` efficiency versus the exact matched-budget permuted-feature control

The promoted concept behaved post-hoc as a pure knowledge-gap / memory-routing distinction.

The unresolved question is therefore narrower than the original H4 question:

> After removing residual-shape leakage and the already discovered memory-shaped cohort, is there independently recoverable CHARGE structure that predicts whether reasoning or causal resolution will be more useful?

---

# Core hypotheses

H5 is split into sequential diagnostics. Later stages do not overwrite earlier failures.

## H5-A — residual-length leakage

### Hypothesis

Variable residual length materially contributes to the utility retained by the H4 permuted-feature control.

### Falsification question

> If every ontology observation is projected into a fixed-width, mask-blind representation, does the matched permuted-feature control lose future utility while the real-feature policy retains useful transfer?

### Important distinction

H5-A is a leakage diagnostic, not an ontology-success experiment.

A positive H5-A result means H4's original control was partially confounded by representational shape.

A negative H5-A result means residual length was not the main explanation and the diagnosis must move to outcome-matrix structure.

---

## H5-B — one easy split versus genuine multi-regime structure

### Hypothesis

Most H4 routing gain came from one easy memory specialization. After removing
that cohort, the remaining reasoning and causal regimes may or may not be
sufficiently distinct under independently judged component outcomes.

The surface-coverage verifier is now known to be too weak for this decision: it
fails H5-B even though the same emitted charges and same H4 memory exclusion
pass when contradiction correction and causal mechanism tasks are scored by the
task-profiled verifier.

### Falsification question

> On the non-memory remainder only, is there measurable resolver specialization between reasoning and causal components before ontology induction is attempted?

This stage asks whether there is a learnable signal to discover at all under the
verifier contract that will be used for H5-C.

If the task-profiled outcome matrix itself does not show stable
reasoning-versus-causal specialization, the ontology inducer must not be blamed
for failing to recover it.

---

## H5-C — recover reasoning-shaped versus causal-shaped tension

### Hypothesis

After fixed-width normalization and exclusion of the known memory-shaped cohort, executable CHARGE features contain enough stable structure to induce at least one transferable distinction between reasoning-favored and causal-favored unresolved state.

### Falsification question

> Can a frozen shadow ontology learned only from historical normalized CHARGE features improve future non-memory resolver selection, beat an undifferentiated non-memory baseline, and beat exact matched-budget feature-destroyed controls?

H5-C is the only stage that can support a stronger latent-concept claim.

---

# Non-negotiable experimental rules

1. H4 remains rejected. Do not edit its gates or reinterpret its pass/fail field.
2. H5 uses a new executable, new report, new documentation, and new frozen result.
3. No hidden emitter label, `ChargeKind`, scope text, topic text, task family, or oracle class may enter proposal search or routing.
4. Hidden source identity is post-hoc diagnostic information only.
5. Resolver self-reported discharge remains inadmissible evidence. Only independently judged and `CognitiveCycleState`-applied discharge enters ontology observations.
6. Training, promotion holdout, and future transfer families remain semantically disjoint.
7. Candidate budget is measured from actual proposal evaluations.
8. Routing budget is the exact number of future observations routed.
9. Every matched-budget control must report exact proposal and route counts or the experiment errors.
10. Final acceptance thresholds must be constants in source before the first run that reaches the H5-C acceptance binary.
11. Compile defects, panics, malformed reports, or control implementation defects are infrastructure failures, not H5 pass/fail verdicts.
12. Once a complete H5-C JSON verdict is emitted, do not tune the frozen seed, families, representation, proposal vocabulary, or gates in that experiment.

---

# Phase 0 — preserve the H4 evidence boundary

## Goal

Make the H4 rejection reproducible and directly consumable by H5 diagnostics.

## Tasks

- retain PR #14's frozen JSON verdict and full Actions artifact
- preserve the exact H4 task-family split
- preserve the exact real component fixture placement
- preserve the `CycleObservationRecorder` evidence path
- preserve the H4 one-concept report, including the effective future support and post-hoc purity
- add a machine-readable H4 result summary fixture only if needed for diagnostic comparison; do not use hidden labels in H5 routing

## Deliverable

A documented immutable H4 reference section in the H5 report containing:

```text
H4 status
H4 head SHA / run ID
H4 promoted concept count
H4 future efficiency
H4 baseline efficiency
H4 random-control efficiency
H4 permuted-control efficiency
H4 failed gate names
```

---

# Phase 1 — build a fixed-width residual representation

## Goal

Remove vector length and dimension existence as routing signals.

## Design

Add a separate experimental adapter. Do not silently change `ontology_feature_charge` used by H4.

Suggested API:

```rust
pub struct FixedResidualProjectionConfig {
    pub bins: usize,
}

pub struct FixedResidualProjection {
    pub values: Vec<f32>,
}

pub fn fixed_residual_projection(
    residual: &[f32],
    config: FixedResidualProjectionConfig,
) -> FixedResidualProjection;
```

### Representation constraints

Every residual, regardless of original length, must produce exactly the same number of visible floats.

Initial vocabulary should be deterministic and CPU-only.

Recommended fixed-width representation:

```text
global geometry:
  rms
  mean_abs
  stddev
  max_abs
  min
  max
  positive_fraction
  negative_fraction
  near_zero_fraction

absolute-magnitude quantiles:
  q10
  q25
  q50
  q75
  q90

signed-value quantiles:
  q10
  q25
  q50
  q75
  q90

normalized index-bin summaries, fixed B bins:
  bin_mean
  bin_mean_abs
  bin_rms
```

For `B = 8`, this creates a deterministic fixed-width feature vector whose width is independent of source residual length.

### Critical anti-leak rules

Do not expose:

- original residual length
- valid-element count
- padding mask
- missing-dimension mask
- emitter-specific dimension names
- raw residual appended after the projection

The fixed projection is deliberately lossy. That is the point of H5-A.

### Tests

Add contract tests proving:

- identical output width for residual lengths `1`, `3`, `32`, `64`, and `257`
- output is deterministic
- output contains only finite values for NaN/Inf inputs
- changing only residual length by appending zeros does not directly reveal the original length through a dedicated length field
- two charges with identical residual vectors and different kinds/scopes produce identical projected features
- H4's existing adapter remains unchanged

---

# Phase 2 — H5-A residual-shape leakage probe

## Goal

Measure whether the H4 permuted control benefited from variable residual shape.

## Data

Use the same real closed-cycle observation generator and same seven H4 task families.

Create three observation views from the exact same judged outcome matrix:

### View R — H4 representation

```text
[rms, mean_abs, non_zero_fraction, positive_fraction, max_abs, ...raw residual]
```

### View F — fixed-width projection

The Phase 1 representation.

### View F-permuted — feature-destroyed fixed projection

Independently permute every fixed projected feature across observations while keeping outcomes attached.

All three views use the same observations and independently judged outcomes.

## Measurements

For each view record:

- candidate count
- promoted concept count
- training efficiency
- promotion holdout gain
- future efficiency
- future/baseline ratio
- window win fraction
- worst-window ratio
- post-hoc cohort purity

## H5-A predeclared diagnostic criteria

H5-A supports the residual-length-leakage hypothesis only if all are true:

1. fixed-width real-feature future efficiency retains at least `0.90x` H4 real-feature future efficiency
2. fixed-width real-feature efficiency is at least `1.15x` fixed-width permuted efficiency
3. fixed-width permuted efficiency is at most `0.90x` H4 variable-length permuted efficiency
4. fixed-width real features still beat the undifferentiated baseline by at least `1.25x`

These are diagnostic criteria, not live-promotion gates.

### Interpretation table

```text
real retains utility + permuted collapses
    -> residual-shape leakage was materially confounding H4

real and permuted both collapse
    -> H4 gain depended on representation details that did not survive normalization

real and permuted both remain strong
    -> shape leakage is not the main problem; inspect outcome matrix and search degeneracy

real collapses but permuted remains strong
    -> severe control/search pathology; stop ontology promotion work and audit the experiment
```

---

# Phase 3 — remove the easy memory split

## Goal

Test the remaining unresolved state rather than repeatedly rediscovering the already demonstrated memory cohort.

## Cohort exclusion rule

Do not use hidden `KnowledgeGap` labels to filter observations.

Use the frozen H4 promoted concept predicate itself as an executable exclusion gate:

```text
H4 ConceptId(1) predicate matches
    -> excluded from H5-B/H5-C search cohort

H4 ConceptId(1) predicate does not match
    -> retained
```

This is important: the system is allowed to use the executable distinction it actually learned, not the post-hoc semantic name assigned by the researcher.

The exclusion predicate must be frozen from the H4 report and not re-fit on H5 data.

## Report

Record post-hoc only:

- number excluded
- hidden emitter distribution excluded
- number retained
- hidden emitter distribution retained

If the frozen predicate does not isolate the future memory cohort similarly to H4, H5 must report that transfer failure explicitly.

---

# Phase 4 — H5-B resolver-specialization identifiability diagnostic

## Goal

Determine whether the remaining real component outcome matrix contains stable reasoning-versus-causal specialization.

## No ontology induction yet

This stage analyzes independently judged outcomes directly.

For every retained observation calculate each candidate's normalized discharge efficiency:

```text
efficiency = accepted_discharge / charge_magnitude / compute_cost
```

Primary comparison:

```text
reasoning efficiency
versus
causal efficiency
```

Also retain prediction and metacognition as nuisance candidates so the diagnostic does not silently assume a two-resolver world.

## Metrics

### Oracle leader distribution

Count the empirically strongest resolver per observation.

Report:

```text
reasoning-leading fraction
causal-leading fraction
prediction-leading fraction
metacognition-leading fraction
tie fraction
```

### Pairwise reasoning/causal margin

For each observation:

```text
margin = reasoning_efficiency - causal_efficiency
```

Report:

- mean margin
- median margin
- standard deviation
- q10/q25/q50/q75/q90
- fraction `margin >= +0.10`
- fraction `margin <= -0.10`
- ambiguous fraction `abs(margin) < 0.10`

### Cross-window stability

For every training, holdout, and future window separately report the leader distribution and reasoning/causal margin distribution.

### Post-hoc source association

Hidden emitter identity may be used only after all efficiency metrics are computed.

Measure whether reasoning-favored and causal-favored observations associate with different real emitters.

## H5-B identifiability gates

The remaining matrix is considered meaningfully multi-regime only if all are true:

1. reasoning leads on at least `20%` of retained observations
2. causal leads on at least `20%` of retained observations
3. at least `15%` have reasoning/causal margin `>= +0.10`
4. at least `15%` have reasoning/causal margin `<= -0.10`
5. the same directionality appears in at least `3/4` future windows
6. no single resolver leads more than `70%` of retained future observations

These gates now apply to the task-profiled verifier matrix. The original
surface-coverage verifier matrix is retained as a negative control and
documentation of verifier/task-ecology sensitivity.

If task-profiled H5-B fails, stop.

### Current H5-B outcome

At commit `754ba148`, the original surface matrix failed, but the task-profiled
matrix passed:

- retained non-memory observations: `96`
- reasoning leader fraction: `0.5`
- causal leader fraction: `0.5`
- positive margin fraction: about `0.302`
- negative margin fraction: `0.5`
- directional windows: `3`
- passed: `true`

This is enough to proceed to H5-C under the task-profiled verifier contract. It
does not rescue the original surface-verifier matrix.

### Supported conclusion after task-profiled H5-B failure

> The current real-component verifier/outcome matrix does not expose two stable non-memory resolver regimes strongly enough for ontology induction to recover reasoning-shaped versus causal-shaped tension.

The next work would then be improving the environment/verifier/component task ecology, not making the inducer more expressive.

---

# Phase 5 — H5-C frozen non-memory ontology induction

Only run this phase if task-profiled H5-B passes.

## Goal

Test whether fixed-width CHARGE features can recover useful non-memory resolver distinctions.

## Visible input

Every retained charge remains:

```text
ChargeKind::Custom("unresolved")
```

Visible features are only the fixed-width projection from Phase 1.

Do not append raw residuals.

Do not expose original residual length.

Do not expose the H4 memory predicate result as a feature. It is used only to define the excluded cohort.

Use the task-profiled verifier contract from `star::charge::verifier` for the
outcome matrix. The H5-C executable must report both:

- the canonical task-profiled verifier result used for fitting and gates
- the original surface-verifier H5-B failure as a non-gating negative control

## Proposal vocabulary

Start deliberately constrained:

```text
ResidualThreshold
And(max depth = 2)
Not(single threshold only)
```

No LLM-generated predicates.

No semantic feature naming.

No runtime code generation.

No hidden-label scoring during search.

### Why conjunctions are allowed

H4 showed that a single threshold could isolate the memory cohort but did not recover a second regime. H5-C specifically tests whether a simple compositional boundary is necessary for the remaining structure.

Conjunction complexity must be charged explicitly.

Suggested complexity penalties:

```text
single threshold: 0.003
Not(threshold):   0.004
And(2 terms):     0.008
```

Freeze these before the first H5-C acceptance run.

Current frozen candidate for the primary H5-C implementation:

```text
SEED: 0x4834_5245_414c_4359
TRAIN_WINDOWS: 2
HOLDOUT_WINDOWS: 1
TRANSFER_WINDOWS: 4
fixed projection width: existing H5-A config
verifier profile: TaskProfiled
task classes: KnowledgeGap, PredictionContradiction, CausalMechanism
candidate operators: ResidualThreshold, Not(threshold), And(2 thresholds)
complexity penalties: 0.003, 0.004, 0.008
promotion criteria: shared PromotionCriteria
```

## Search strategy

Greedy sequential promotion with parent fallback:

1. fit strongest undifferentiated parent resolver from training only
2. enumerate deterministic candidate predicates from training feature midpoints
3. evaluate training marginal utility relative to current active ontology
4. rank candidates by penalized training gain
5. evaluate strongest unique candidates on independent promotion holdout
6. promote only through the shared `PromotionCriteria`
7. freeze promoted predicates and resolver leaders
8. route future windows without updating concepts, thresholds, or leaders

## Controls

All controls receive the exact candidate's measured proposal and future routing budgets.

### Control 1 — random partition search

Same number of effective routing groups as the learned ontology plus parent fallback.

Training selects resolver leaders.

Strongest training partition receives one holdout gate.

### Control 2 — independently permuted fixed features

Every fixed projected feature independently shuffled across observations.

Outcome matrix remains attached.

Same predicate vocabulary, conjunction depth, complexity penalties, proposal budget, holdout procedure, and future route budget.

### Control 3 — residual-length-only oracle diagnostic

Not eligible for promotion and not counted as a matched search control.

Route by original residual length using training-learned leaders.

Purpose:

> quantify how much routing utility raw vector length alone carries

This control must be clearly marked post-hoc diagnostic because H5-C's candidate cannot see residual length.

### Control 4 — parent-plus-frozen-memory baseline

Apply the H4 memory predicate, route matches to its frozen memory leader, and use one strongest non-memory parent resolver for everything else.

This is the most important baseline.

H5-C must beat it.

Otherwise the new ontology has not added useful structure beyond H4.

---

# H5-C frozen final gates

The acceptance executable must exit nonzero unless all pass.

1. H5-B identifiability gates passed before ontology fitting
2. all visible H5-C kinds are `Custom("unresolved")`
3. all H5-C features have identical fixed width
4. original residual length and masks are unavailable to proposal search and routing
5. at least `1` new non-memory concept is promoted
6. every promoted concept has training support `>= 16`
7. every promoted concept has holdout support `>= 8`
8. every promoted concept has positive holdout gain
9. future induced efficiency is at least `1.15x` the non-memory undifferentiated baseline
10. future induced efficiency is at least `1.10x` the parent-plus-frozen-memory baseline measured on the full future stream
11. induced policy beats the non-memory baseline in at least `3/4` future windows
12. worst future non-memory window retains at least `1.02x` baseline efficiency
13. future induced efficiency is at least `1.15x` the exact matched-budget random control
14. future induced efficiency is at least `1.15x` the exact matched-budget permuted-fixed-feature control
15. exact proposal-budget equality holds for both matched search controls
16. exact future-routing-budget equality holds for both matched search controls
17. at least one promoted concept's effective future cohort has post-hoc dominant resolver-margin direction purity `>= 0.70`

The primary acceptance rule is conjunctive: `17/17` or reject.

Do not weaken these gates after a complete verdict.

---

# Multi-seed replication

A one-seed pass is not enough for the final H5 claim.

After the frozen primary acceptance run, run the exact same protocol over eight predeclared seeds:

```text
0x4835_0000_0000_0001
0x4835_0000_0000_0002
0x4835_0000_0000_0003
0x4835_0000_0000_0004
0x4835_0000_0000_0005
0x4835_0000_0000_0006
0x4835_0000_0000_0007
0x4835_0000_0000_0008
```

The task families, feature representation, candidate vocabulary, budgets, penalties, and gates remain unchanged.

## Replication criterion

At least `6/8` seeds must satisfy all utility-direction gates:

- induced > non-memory baseline
- induced > parent-plus-frozen-memory baseline
- induced > random control
- induced > permuted fixed-feature control

At least `5/8` seeds must promote a non-memory concept with future margin-direction purity `>= 0.70`.

The primary frozen acceptance seed still retains its own `17/17` conjunctive verdict.

---

# Implementation map

## New files

Suggested:

```text
lib/charge/fixed_features.rs
lib/examples/h5_residual_identity_diagnostic.rs
lib/examples/h5_non_memory_ontology_probe.rs
docs/experiments/H5_RESIDUAL_IDENTITY_DIAGNOSTIC.md
docs/experiments/H5_NON_MEMORY_ONTOLOGY.md
```

## Existing files expected to change

```text
lib/charge/mod.rs
.github/workflows/charge-ci.yml
```

Avoid modifying H4 experiment source except for bug fixes necessary to reproduce the already recorded H4 result. Any such fix must preserve the recorded verdict and be separately documented.

## Reusable types worth adding

```rust
FixedResidualProjectionConfig
FixedResidualProjection
ResolverMarginSummary
ResolverLeaderDistribution
IdentifiabilityCriteria
IdentifiabilityAssessment
```

Do not bury these in a 1,000-line example if they are useful across future CHARGE experiments.

---

# CI plan

Keep existing CHARGE CI gates.

Add, in order:

```text
cargo test -p star charge::fixed_features --locked -- --test-threads=1
cargo run -p star --example h5_residual_identity_diagnostic --locked
cargo run -p star --example h5_non_memory_ontology_probe --locked
```

Artifact bundle should retain:

```text
charge-cargo-check.log
charge-test.log
charge-real-component-report.json
h4-shadow-promotion-report.json
h4-real-cycle-shadow-report.json
h5-residual-identity-diagnostic-report.json
h5-non-memory-ontology-report.json
h5-multiseed-summary.json
```

If H5-B identifiability fails, the H5-C executable should emit a structured rejected/skipped report explaining that ontology fitting was not scientifically justified. CI may intentionally exit nonzero on the acceptance job while still uploading all diagnostics.

---

# Decision tree

```text
H5-A: does fixed-width normalization hurt the permuted control?
│
├── YES
│   └── residual shape was a material H4 confound
│       proceed to H5-B
│
└── NO
    └── residual length was not the main explanation
        proceed to H5-B anyway, but treat search/outcome degeneracy as primary suspect

H5-B: do stable reasoning-favored and causal-favored non-memory regimes exist?
│
├── NO
│   └── STOP ontology work on this fixture matrix
│       improve environment/verifier/task ecology
│       rerun as a new hypothesis
│
└── YES
    └── proceed to H5-C

H5-C: can normalized ontology induction recover transferable non-memory structure?
│
├── FAIL
│   └── current predicate/search mechanism is insufficient
│       next candidate: relational/temporal concept operators or learned residual embedding
│
└── PASS
    └── run eight-seed replication
        │
        ├── replication FAIL
        │   └── unstable proof of mechanism; no live promotion
        │
        └── replication PASS
            └── support shadow-only automatic promotion research
                next: naturally occurring online closed-cycle histories
```

Current path through the tree:

```text
H5-A: YES
surface-verifier H5-B: NO, retained as negative control
task-profiled H5-B: YES
H5-C primary acceptance seed: PASS
next node: eight-seed H5-C replication
```

---

# Claim boundaries

## If H5-A passes

Supported:

> Variable residual shape materially confounded H4's feature-destroyed control, and a fixed-width label-blind representation preserved more real-feature routing utility than matched permuted features.

Not supported:

- multiple latent concepts exist
- reasoning/causal regimes are identifiable
- live ontology promotion is justified

## If task-profiled H5-B passes

Supported:

> After excluding the frozen H4 memory-shaped cohort, independently judged real component outcomes contain stable opposing reasoning-versus-causal resolver regimes across future windows.

Not supported:

- CHARGE features can recover those regimes
- ontology induction works

## If H5-C and replication pass

Supported:

> After removing residual-length information and excluding the previously discovered memory-shaped cohort, deterministic shadow ontology induction can recover at least one executable non-memory distinction from normalized CHARGE features that transfers across unseen task families and beats exact matched-budget random, feature-destroyed, and H4 parent-plus-memory baselines with multi-seed replication.

Still not supported:

- human-like semantics
- autonomous scientific discovery
- unrestricted ontology growth
- live automatic promotion safety
- AGI
- consciousness

## If primary H5-C passes before replication

Supported:

> The frozen primary shadow diagnostic recovered one executable non-memory
> distinction from fixed-width H4-retained CHARGE features that transferred on
> the primary seed and beat the exact matched-budget controls in the local
> verdict.

Not supported:

- replicated H5-C support
- live ontology promotion
- autonomous scientific discovery
- AGI
- consciousness

---

# Recommended execution order

1. done: preserve H4 as the immutable rejected real closed-cycle record
2. done: implement fixed-width residual projection with tests
3. done: implement H5-A on the H5 diagnostic stream
4. done: inspect the predeclared H5-A diagnostic verdict
5. done: implement H5-B direct outcome-matrix identifiability report
6. done: add task-profiled verifier scoring and make it the canonical H5-B gate
7. done: implement `h5_non_memory_ontology_probe`
8. done: freeze H5-C source constants and candidate vocabulary before the first complete verdict
9. done: run the primary H5-C acceptance seed once
10. done: preserve the complete verdict without tuning
11. next: run the eight predeclared replication seeds
12. only after replicated success discuss shadow automatic promotion

## Bottom line

Do not make the ontology inducer broadly more complicated yet.

The immediate engineering target is now narrower:

> Run the predeclared eight-seed H5-C replication without tuning the primary
> verdict, then preserve both passes and failures as scientific evidence.
