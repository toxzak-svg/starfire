# Ω1 — Endogenous State-Space Genesis

Status: preregistered before implementation and before any verdict-producing run.

## Research break from H9–H11

H9–H11 establish a narrow lineage:

- H9: a validated executable commitment can become causally necessary intermediate state;
- H10: a useful rule can be inferred from intervention evidence and admitted only after independent proof recomputation;
- H11: the candidate relation frontier can be discovered from graph incidence rather than supplied directly.

Ω1 intentionally does **not** continue that line by enlarging the rule frontier again.

The new question is whether Starfire can detect that its **current executable state language itself is insufficient**, synthesize a new executable state distinction from the failure witness, independently validate that the distinction repairs the representational defect, and admit the distinction so a later fixed operation can distinguish histories that were provably indistinguishable before admission.

## Primary research question

> Can Starfire detect a witnessed failure of behavioral equivalence under its current executable state language, synthesize a new executable refinement program without being supplied the target distinction, independently validate the repair, and admit that program so later cognition gains a distinction that did not exist in the original state space?

## What counts as a representational defect

Let `K_L(h)` be the executable state key assigned to raw history `h` by state language `L`.

A representational alias defect exists when two histories satisfy:

```text
K_L(h1) = K_L(h2)
```

but the same independently witnessed intervention produces different outcomes:

```text
O(h1, a) != O(h2, a)
```

Under a deterministic downstream operation whose only history input is `K_L(h)`, the two histories cannot be treated differently. The defect is therefore not merely poor prediction; it is an explicit insufficiency of the current state representation under the declared interface.

## Frozen substrate

Ω1 introduces an immutable `StateLanguage` kernel with:

- a base history projection that is intentionally order-blind;
- zero or more admitted executable refinement programs;
- canonical state-key construction from the base projection plus refinement outputs;
- immutable provenance for each admitted refinement;
- deterministic replay signatures.

The initial base language sees the multiset of history atoms but not their temporal order.

The hidden structural regime in the Ω1 fixture depends on temporal precedence. The correct precedence relation is **not supplied** to synthesis, validation, the downstream predictor, or the objective checker.

## Frozen synthesis language

Ω1 does not receive a candidate concept list or candidate endpoint pair.

For each root, the proposer derives the raw atom vocabulary from the discovery histories and enumerates a generic bounded program grammar over history metrics:

```text
metric := FirstIndex(atom) | LastIndex(atom) | Count(atom)
program := metric_a < metric_b | metric_a == metric_b
```

Programs are executable predicates over raw histories.

The grammar is developer-supplied and fixed before the first run. A PASS therefore does not establish unbounded language invention. The intended result is narrower: synthesis of a new executable **state-key dimension** from a generic program language, where the useful atoms and useful program are not supplied.

## Canonical partition semantics

Multiple syntactically different programs can induce the same binary partition. Ω1 therefore ranks **behavioral partitions**, not raw syntax.

For every generated program:

1. execute it over all discovery histories;
2. canonicalize its binary output vector up to boolean complement;
3. group programs with the same canonical partition;
4. retain the lexicographically canonical executable representative for that partition.

A partition is scored only by independently witnessed alias-defect repair on the discovery set.

## Frozen defect-repair score

For every pair of discovery histories that:

- have the same base-language state key;
- receive the same intervention;
- have different witnessed outcomes;

one alias defect is recorded.

A candidate partition repairs a defect iff it separates the two histories.

Primary score:

```text
repaired_defects
```

Secondary statistics:

```text
unrepaired_defects
partition_support_min
runner_up_repaired_defects
winner_margin
```

Frozen certificate gates:

```text
repaired_defects == all_detected_defects
unrepaired_defects == 0
partition_support_min >= 4
winner_margin >= 2
```

The validator independently recomputes:

1. raw vocabulary;
2. base-language keys;
3. complete alias-defect set;
4. complete bounded program enumeration;
5. every program execution;
6. canonical partition grouping;
7. complete partition ranking;
8. winning executable representative;
9. every proof field.

Only an opaque validated refinement certificate may alter the executable state language.

## Frozen experiment shape

The cohort remains structurally comparable to H9–H11:

```text
16 training roots
8 holdout roots
32 future roots
4 future domain vocabularies
```

Each root contains:

```text
12 discovery histories with witnessed outcomes
8 transfer histories with independently withheld outcomes
6 raw history atoms
1 shared intervention atom
2 outcome atoms
```

All histories in a root have the same event multiset under the base order-blind projection. The current language therefore aliases histories that require different future predictions.

The useful distinction is generated from temporal structure in the raw histories. Distractor atoms and orderings are arranged so the target behavioral partition is the unique winning partition after complement canonicalization under the frozen discovery cohort.

## Fixed downstream operation

The downstream predictor is deliberately representation-bound.

It receives only:

- executable state keys produced by the current `StateLanguage`;
- discovery outcome labels attached to those keys;
- a transfer state key to classify.

It cannot inspect raw histories, refinement proof text, synthesis scores, target atom identities, family labels, split labels, or hidden fixture parameters.

If one executable state key maps to conflicting discovery outcomes, the predictor must abstain for that key.

A transfer prediction succeeds only if the key has one uniquely supported witnessed outcome.

This creates the core causal test:

- before refinement, opposite-regime histories share one key and the predictor must abstain;
- after valid refinement admission, the new executable key dimension can separate the regimes and permit prediction.

## Frozen matched paths

Every root executes the same synthesis and validation budget under ten paths:

1. `stateful_refinement`
   - valid certificate admitted before downstream learning and transfer prediction.

2. `endpoint_only`
   - winning program may be executed and its output recorded, but it is not admitted to `StateLanguage`; downstream state keys remain base-language keys.

3. `proposal_text_only`
   - proof/program serialization preserved; no executable admission.

4. `scalar_only`
   - winning score, support, and margin preserved; no executable admission.

5. `foreign_certificate`
   - a valid certificate from a different root is presented for admission and must be rejected.

6. `counterfeit_certificate`
   - proof fields are structurally plausible but tampered and must fail independent validation/admission.

7. `valid_irrelevant_refinement`
   - an independently valid same-complexity refinement over a separately generated irrelevant defect fixture is admitted; it must not repair the target transfer task.

8. `outcome_shuffled_synthesis`
   - synthesis and validation receive a deterministic permutation of discovery outcomes that destroys target history/outcome incidence while preserving counts and full compute budget.

9. `random_valid_refinement`
   - a deterministic non-winning executable partition of matched grammar complexity is admitted.

10. `delayed_correct_admission`
    - the correct certificate is admitted only after the transfer prediction window.

Rejected certificates still consume the full independent recomputation budget.

## Frozen budget accounting

For every root/path:

- one complete proposer vocabulary derivation;
- one complete validator vocabulary derivation;
- one complete proposer alias-defect construction;
- one complete validator alias-defect construction;
- one complete proposer bounded-program enumeration;
- one complete validator bounded-program enumeration;
- one execution of every candidate program on every discovery history for proposer scoring;
- one independent execution of every candidate program on every discovery history for validator recomputation;
- one admission slot;
- one downstream key-index construction pass;
- one prediction for each of 8 transfer histories;
- one independent objective check per transfer history.

The exact candidate-program count is a deterministic function of the discovered six-atom vocabulary and fixed grammar and must be identical across all roots and paths.

## Frozen success gates

A `PASS` requires all of the following:

1. exact 16/8/32 root split and four future families;
2. every root begins with at least one witnessed base-language alias defect;
3. the base language produces identical keys for opposite-outcome discovery histories;
4. the proposer and validator independently enumerate the exact same candidate count and defect set;
5. every stateful training root achieves 8/8 transfer predictions;
6. every stateful holdout root achieves 8/8 transfer predictions;
7. every stateful future root achieves 8/8 transfer predictions;
8. every non-stateful or structure-destroying control achieves 0/8 successful transfer predictions per root;
9. foreign and counterfeit certificates are rejected on 100% of roots;
10. valid irrelevant refinements are admitted on 100% of roots but achieve 0/8 target transfer success;
11. all four future families independently achieve stateful success 1.0 and maximum control success 0.0;
12. all budgets are exact;
13. exact replay holds from fresh state;
14. state/provenance invariants hold.

Terminal classification:

```text
CONTROL_FAILURE
```

if budget or invariant controls fail;

```text
REPLAY_FAILURE
```

if deterministic replay fails;

```text
PASS
```

only if every frozen success gate passes;

otherwise:

```text
REJECTED
```

No threshold, grammar rule, fixture shape, family, root count, control, budget, gate, or classification rule may be changed after the first complete verdict-producing run to rescue a failure.

## Required causal interpretation of a PASS

A PASS would support only the following claim:

> Under the frozen symbolic sequence-history regime and fixed bounded synthesis grammar, Starfire detected that its current executable state language aliased histories with different intervention outcomes, synthesized a new executable state-key refinement from raw history structure without being given the useful atom pair, survived independent full recomputation, and required admission of that refinement before a later representation-bound operation could correctly distinguish withheld histories across unseen vocabularies.

## Claims explicitly not established

A PASS would **not** establish:

- unrestricted open-world ontology invention;
- natural-language representation repair;
- learned or self-modified synthesis grammar;
- unbounded new type invention;
- recursive descendant concept genesis;
- automatic live-runtime promotion readiness;
- general intelligence;
- consciousness;
- human-level cognition.

## Scientific next step after a PASS

Ω2 should test **descendant necessity**:

- admit a first endogenous state-space refinement `Δ1`;
- construct a second representational defect whose candidate repair is not expressible or discoverable under `L0`;
- show that `Δ2` becomes discoverable only under `L1 = L0 + Δ1`;
- ablate `Δ1` and require both the expressibility and discovery of `Δ2` to disappear.

That would test cumulative growth of the machine's executable hypothesis language rather than one-step representation repair.
