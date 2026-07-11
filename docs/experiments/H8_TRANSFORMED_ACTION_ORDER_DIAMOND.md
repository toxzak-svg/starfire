# H8 Transformed Action-Order Diamond

Status: frozen diagnostic contract before the first complete H8 GitHub Actions run.

H8 is diagnostic-only. It does not alter live routing, enable automatic concept promotion, repair H6, revise H7, or claim a latent ontology.

## Architectural reason for this experiment

H7 executed all 20 ordered non-repeating length-two resolver words in one environment episode and one `CognitiveCycleState`, but the component adapters still resolved the original task on both steps. The second resolver did not consume the first resolver's output or an explicit intermediate unresolved object. H7 therefore measured environment/ledger continuation, not a full action algebra over transformed unresolved state.

The H7 frozen result was:

```text
training parent congruence defect = 0.0
future parent congruence defect   = 0.03829787234042553
right absorption P[F_ab(h)=F_b(h)] = 0.85
candidate word = none
```

The exact weakened assumption under attack is:

> Starfire lacks a substrate in which one computational operation transforms the unresolved object that a later computational operation receives.

H8 introduces that substrate only inside an isolated executable and asks whether the actual reasoning and causal components exhibit a reproducible, independently witnessed action-order effect once the second component is forced to consume the intermediate state.

## Research question

For the two ordered words

```text
R;C = reasoning then causal
C;R = causal then reasoning
```

let `u` be one retained unresolved root. After the first operation `a`, form the witnessed intermediate object

```text
I(u,a) = (
    root_prompt,
    first_output,
    signed_objective_motion,
    accepted_fraction,
    unresolved_fraction
)
```

where every scalar is measured after the first operation on the same root episode and `first_output` is the actual component output.

The second operation receives a deterministic serialization of `I(u,a)` and the original prompt. No resolver identity, hidden event class, task-family label, target answer, residual coordinate, resolver leader, or future outcome is supplied.

The primary question is:

> Does one frozen action order achieve a transferable objective advantage because the second operation consumes the same-root intermediate state, rather than because the final resolver overwrites the first, because more scalar state was carried, or because extra compute was spent?

## Observation boundary

H8 reconstructs the frozen H5/H6 chronological fixture:

```text
252 real subsystem-backed unresolved observations
84 frozen H4 memory exclusions expected
168 retained non-memory observations expected
48 training observations
24 promotion-holdout observations
96 future-transfer observations in four later family windows
```

The three real emitters remain:

```text
metacognitive knowledge gap
prediction contradiction
Quanot trajectory residual
```

The visible CHARGE kind remains `Custom("unresolved")`.

The frozen H4 memory predicate is used only to preserve the established non-memory cohort boundary:

```text
ontology_feature_charge residual[2] <= 0.171875
```

After exclusion, H8 selection and execution receive no residual coordinates.

## Admissible intermediate information

At the second decision in a word, the following are historically measurable:

```text
root prompt
first operation output
first-step objective witness
first-step accepted discharge
current unresolved magnitude
```

The serialized intermediate object may therefore contain:

```text
first_output
signed_objective_motion
accepted_fraction
unresolved_fraction
```

It may not contain:

```text
hidden EventClass
TaskFamily name or window index
verifier target
verifier evidence text
resolver leader
future score
reverse-word outcome
raw or fixed residual coordinates
CHARGE kind or scope
first resolver name
second resolver name
```

The operation order is fixed externally by the candidate word; it is not encoded in the intermediate payload.

## Common-root discipline

For anchor `i`, every H8 path uses the common reset seed

```text
SEED xor anchor_id
```

The seed is independent of word orientation and control mode.

A paired anchor is eligible only when both possible first operations, reasoning and causal, leave a non-empty unresolved state after independent witnessing and `RelativeImprovementJudge` application.

If either first operation fully resolves the root, the anchor is excluded from both `R;C` and `C;R` comparisons and from every control. The exclusion is symmetric and is reported.

No path is allowed to stop after the first step once an anchor is admitted to the paired experiment. Every evaluated word consumes exactly two resolver calls and exactly two objective evaluations.

## Stateful execution

For word `a;b`:

1. reset the target-verifier environment from the common root seed;
2. admit the frozen fixed-width CHARGE into a new `CognitiveCycleState`;
3. execute actual Starfire component `a` on the original task;
4. apply its output to the environment;
5. construct an `OutcomeWitness` from objective progress before and after step 1;
6. evaluate the attempted discharge through `RelativeImprovementJudge`;
7. apply the judgment to the cognitive cycle;
8. construct `I(u,a)` from the actual output and witnessed cycle state;
9. serialize `I(u,a)` deterministically into the continuation prompt;
10. execute actual Starfire component `b` on that continuation prompt;
11. apply the second output to the same environment episode;
12. independently measure terminal objective progress and apply the second judgment.

The target verifier continues to score only the emitted operation output. Intermediate witness scalars are not read directly by the verifier.

## Continuation serialization

The exact serialization is frozen as:

```text
{root_prompt}

Historical computation on this same unresolved state:
prior_output: {first_output}
prior_objective_motion: {motion:.6}
prior_accepted_fraction: {accepted_fraction:.6}
current_unresolved_fraction: {unresolved_fraction:.6}
Continue resolving the original question from this historical state.
```

This scaffold is developer-authored and therefore cannot itself be called a learned concept. H8 tests whether a compositional substrate changes actual component behavior; it does not test concept creation.

## Candidate structures

Exactly two structures are proposed:

```text
R;C
C;R
```

The training candidate is the word with the larger mean stateful terminal objective score. Exact ties choose `R;C`.

The candidate and all thresholds are frozen before promotion-holdout evaluation.

## Matched controls

### Blind/right-absorption control

Execute the same word with the same two component calls, common root, cycle updates, and objective evaluations, but give the second resolver the original task rather than `I(u,a)`.

This is the H7-style null:

```text
F_ab(u) approximately F_b(u)
```

Stateful advantage is measured against the same-word blind path.

### Scalar-state control

The second resolver receives the same continuation scaffold and the exact witnessed scalars:

```text
signed_objective_motion
accepted_fraction
unresolved_fraction
```

but `first_output` is replaced by an empty string.

This control carries persistent historical state but removes the operation-produced content that could transform the next computation.

### Common-root-rewired control

For each chronological split and word separately, collect all eligible same-word intermediate bundles. Sort by anchor ID and apply a frozen nonzero cyclic shift derived from `SEED`, the split salt, and the word salt.

The second resolver receives another anchor's complete intermediate bundle while retaining its own root prompt, environment episode, and unresolved CHARGE.

This preserves exactly:

```text
number of intermediate bundles
first-output marginal multiset
witness-scalar marginal multiset
resolver-call count
objective-evaluation count
word frequency
chronological split
```

and destroys precisely:

```text
same-root intermediate incidence
```

No hidden class or family label is used to form the permutation.

## Frozen constants

```text
SEED = 0x4838_4143_5444_4941

CANDIDATE_WORDS = [R;C, C;R]
PROPOSAL_BUDGET = 2
COMPLEXITY_PENALTY = 0.01

MIN_TRAIN_ELIGIBLE = 32
MIN_HOLDOUT_ELIGIBLE = 16
MIN_FUTURE_ELIGIBLE = 64

MIN_TRAIN_GAIN_AFTER_PENALTY = 0.05
MIN_TRAIN_ORDER_ADVANTAGE = 0.03

MIN_HOLDOUT_GAIN = 0.05
MIN_HOLDOUT_ORDER_ADVANTAGE = 0.03
MIN_HOLDOUT_POSITIVE_FRACTION = 0.60
MAX_HOLDOUT_RIGHT_ABSORPTION = 0.75

MIN_FUTURE_GAIN = 0.05
MIN_FUTURE_ORDER_ADVANTAGE = 0.03
MIN_FUTURE_WINDOW_WINS = 4
MIN_WORST_FAMILY_GAIN = 0.01
MAX_FUTURE_RIGHT_ABSORPTION = 0.75

MIN_REWIRED_MARGIN = 0.03
MIN_SCALAR_MARGIN = 0.03
```

These values are frozen before the first complete H8 CI result. No failed gate may be rescued by threshold adjustment.

## Metrics

For word `w`, let

```text
S_w = mean terminal objective score under same-root stateful execution
B_w = mean terminal objective score under blind execution
Q_w = mean terminal objective score under scalar-state execution
P_w = mean terminal objective score under common-root-rewired execution
```

Define:

```text
composition_gain(w) = S_w - B_w
scalar_margin(w) = S_w - Q_w
rewired_margin(w) = S_w - P_w
```

For candidate `w*` and reverse word `rev(w*)`:

```text
order_advantage = S_w* - S_rev(w*)
```

Pairwise positive fraction is:

```text
P[stateful_score_i(w*) > blind_score_i(w*)]
```

Right absorption is:

```text
P[stateful_score_i(w*) = blind_score_i(w*)]
```

with exact `f64::to_bits()` equality after the deterministic verifier score.

Each future family window wins only when

```text
mean_stateful_score(w*) > mean_blind_score(w*)
```

by more than `1e-12`.

The worst-family gain is the minimum family-wise stateful-minus-blind mean difference.

## Frozen decision logic

`PASS` requires all of:

```text
training eligible >= 32
holdout eligible >= 16
future eligible >= 64

training composition gain - 0.01 >= 0.05
training order advantage >= 0.03

holdout composition gain >= 0.05
holdout order advantage >= 0.03
holdout positive fraction >= 0.60
holdout right absorption <= 0.75

future composition gain >= 0.05
future order advantage >= 0.03
future family wins = 4/4
worst-family gain >= 0.01
future right absorption <= 0.75

future rewired margin >= 0.03
future scalar margin >= 0.03
```

Otherwise classify in this order:

```text
NOT_COMPOSABLE
    paired eligible support fails in any split

RIGHT_ABSORPTION
    candidate has positive stateful gain but holdout or future absorption exceeds its maximum

CONTROL_FAILURE
    primary transfer gates would pass but rewired or scalar margin fails

ORDER_UNSTABLE
    composition gain transfers but order advantage fails on holdout or future

TRANSFER_ONLY
    future composition gain and 4/4 family wins pass but a promotion-holdout gate fails

REJECTED
    all other failures
```

`PASS` supports only:

> Explicitly threading the independently witnessed intermediate unresolved state into the next real Starfire component creates a reproducible action-order effect that transfers across later task families and cannot be explained by final-resolver overwrite, scalar history alone, or a rewired intermediate-state multiset under the frozen fixture.

It does not support:

```text
latent concept creation
ontology induction
AGI
consciousness
human-level reasoning
```

## Budget ledger

For every paired eligible anchor and every word/control path:

```text
resolver calls per path = 2
objective evaluations per path = 2
```

Paths are:

```text
stateful
blind
scalar-state
common-root-rewired
```

Words are:

```text
R;C
C;R
```

Therefore, for `N` eligible anchors in a split:

```text
candidate structures proposed = 2
composite path evaluations = 8N
resolver calls = 16N
objective evaluations = 16N
```

Persistent H8 fitted state is exactly:

```text
candidate word: 1 byte discriminant
frozen scalar constants: compile-time
```

No learned vector, cluster assignment, residual direction, neural parameter, or feature threshold is persisted.

## Pre-execution red-team audit

### Target leakage

Risk: the first output can contain the correct answer.

Could fake a pass by: manually injecting the target into the intermediate object.

Prevention: only the actual first component output is serialized. The verifier target and evidence text are inaccessible to the continuation builder. The first output is historical operation output and is exactly the information whose compositional use is under test.

### Resolver identity leakage

Risk: the second component could infer the intended word from a supplied label.

Could fake a pass by: writing `reasoning output` or `causal output` into the continuation prompt.

Prevention: the intermediate serialization contains no resolver name or numeric resolver ID.

### Last-resolver overwrite

Risk: `TargetVerifierEnvironment::act` assigns progress from the current action score, so terminal behavior can collapse to the last operation.

Could fake a pass by: calling any difference between `R;C` and `C;R` noncommutativity.

Prevention: the primary gain is stateful minus the exact same-word blind path, and right absorption has explicit rejection gates.

### More state instead of transformed operation content

Risk: carrying three historical scalars may itself perturb a prompt.

Could fake a pass by: celebrating any context-length effect.

Prevention: the scalar-state path receives the same scaffold and scalars but no first output. Stateful execution must beat it by at least `0.03` on future data.

### Rewired-content artifact

Risk: unrelated prior output may trivially confuse the second resolver.

Could fake a pass by: using a weak random control with less compute.

Prevention: the rewired path consumes the exact same bundle multiset, two component calls, and two objective evaluations. Only same-root incidence is destroyed. The control is intentionally hostile; a pass requires a `0.03` future margin.

### Temporal leakage

Risk: future families influence candidate selection.

Prevention: candidate word is selected from the first 48 retained non-memory training observations only. Holdout and future are applied without refitting.

### Hidden emitter reconstruction

Risk: the continuation prompt could contain hidden `EventClass` or family names.

Prevention: serialization is defined only over the root prompt and historically measured intermediate object. Hidden labels exist only for fixture generation and post-hoc cohort reporting.

### Asymmetric eligibility

Risk: excluding anchors separately by word changes the comparison cohort.

Prevention: paired eligibility is the intersection of roots for which both reasoning-first and causal-first leave unresolved state. Every word and control uses the identical eligible anchor set.

### Compute mismatch

Risk: stateful paths receive more calls than controls.

Prevention: every admitted path executes exactly two resolver calls and two objective evaluations. Any path that violates this is an infrastructure error and the experiment terminates without a scientific verdict.

### Prompt-length cost omission

Risk: current component compute accounting may not charge proportionally for continuation prompt length.

Prevention: H8 does not compare two-step composites against one-step production efficiency. All primary comparisons are two-step against two-step. Serialized prompt byte counts are reported diagnostically; no production-cost claim is made.

### Post-selection inference

Risk: two candidate words are inspected and the better is chosen.

Prevention: the proposal budget is exactly two, deterministic tie-breaking is frozen, a separate promotion holdout is mandatory, and the same frozen candidate must survive four later family windows.

## Execution discipline

Implementation begins only after this document is committed.

The executable must be:

```text
cargo run -p star --example h8_transformed_action_order_diamond --locked
```

The first complete run must emit one structured JSON report containing:

```text
git-visible frozen constants
cohort counts
paired eligibility counts
resolver-call and objective-evaluation counts
candidate word
training metrics for both words
holdout metrics for both words
future aggregate metrics for both words
four future-window metrics
scalar-state control metrics
common-root-rewired control metrics
right-absorption rates
prompt byte diagnostics
gates
terminal classification
```

The CI artifact filename is frozen as:

```text
h8-transformed-action-order-diamond-report.json
```

After the first complete primary result:

```text
no threshold changes
no added features
no new candidate words
no prompt scaffold revision
no control substitution
no rescue iteration
```

Compile or serialization defects may be repaired only when the numerical contract and mathematical operation remain unchanged.
