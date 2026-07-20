# ΩG4 Intervention-Guided Abstraction Selection

Status: **FROZEN PREREGISTRATION — NO VERDICT**

Local date: **2026-07-20 America/Detroit**

## 1. Scientific question

ΩG3 established that Starfire can factor independently synthesized solutions into a reusable parameterized abstraction when exactly one abstraction fits the development evidence.

ΩG4 asks the next narrower question:

> When two compact reusable abstractions explain the same passive development evidence, can Starfire identify a bounded intervention that separates their predictions, use the independently witnessed outcome to reject the spurious abstraction, and retain the abstraction that transfers under correlation break?

This is a test of bounded model discrimination and active evidence acquisition. It is not a test of unrestricted causal discovery, open-world experimentation, live autonomy, or AGI.

## 2. Required parent lineage

The experiment requires executable, revalidated parent artifacts from:

1. ΩG1 `AdjacentBefore`;
2. ΩG2 `ConsecutiveChain3`;
3. ΩG3 `RecursiveAppendAdjacent`.

Textual proof claims, copied digests, raw schema values, or reconstructed look-alikes must not substitute for admitted executable parent handles.

The ΩG3 result must remain unchanged. ΩG4 receives no authority to mutate ΩG1, ΩG2, or ΩG3 registries.

## 3. Frozen task family

Each task contains an ordered event history over local atoms:

```text
x0, x1, ..., x(n-1), proxy, distractors
```

The true outcome rule is:

```text
CAUSAL_CHAIN := RecursiveAppendAdjacent(x0, x1, ..., x(n-1))
```

A compact proxy abstraction is also available:

```text
PROXY_ANCHOR := AdjacentBefore(proxy, x0)
```

### Passive development construction

Passive development histories are generated so that:

```text
CAUSAL_CHAIN == PROXY_ANCHOR
```

for every development history.

Therefore both abstractions must:

- fit every passive development label;
- have equal support;
- satisfy the same frozen compression threshold;
- remain observationally indistinguishable before intervention.

A system that promotes either abstraction from passive evidence alone must fail.

### Interventional construction

The intervention library contains bounded transformations of a history. One intervention family breaks the proxy correlation while preserving the causal chain status:

```text
MOVE_PROXY_AFTER_X0
```

A second discriminating family breaks the causal chain while preserving the proxy status:

```text
SWAP_INTERNAL_CHAIN_EDGE
```

Non-discriminating controls alter distractors or apply transformations under which the two candidate abstractions make the same prediction.

The evaluator provides the intervention outcome only after the selected intervention is committed. No candidate receives access to the hidden causal label, fixture generator branch, or post-intervention outcome in advance.

## 4. Frozen arities and families

### Development

- arity 4
- 8 roots
- passive histories per root: 24
- total passive development histories: 192

Surface families:

```text
development_thermal
development_software
development_watershed
development_logistics
```

Each family contributes exactly 2 roots.

### Intervention calibration set

The same 8 development roots expose a frozen intervention menu of 6 candidates per root:

1. `MOVE_PROXY_AFTER_X0`
2. `SWAP_INTERNAL_CHAIN_EDGE`
3. `MOVE_DISTRACTOR_LEFT`
4. `MOVE_DISTRACTOR_RIGHT`
5. `ROTATE_DISTRACTORS`
6. `IDENTITY_CONTROL`

Exactly candidates 1 and 2 are prediction-discriminating between the two observational winners. They must receive equal maximum disagreement score. Canonical ordering must select `MOVE_PROXY_AFTER_X0`.

The selection rule may inspect candidate predictions and intervention cost only. It may not inspect hidden outcomes.

### Held-out transfer

- arity 5
- 12 roots
- 8 transfer histories per root
- total held-out predictions: 96

Held-out families:

```text
holdout_energy
holdout_compiler
holdout_river
```

Each family contributes exactly 4 roots.

For held-out roots, proxy correlation is broken. Half of the roots reverse the passive proxy relationship and half remove it. The true recursive chain abstraction remains exact.

### Future transfer

- arity 6
- 18 roots
- 8 transfer histories per root
- total future predictions: 144

Future families:

```text
future_materials
future_protocols
future_ecology
```

Each family contributes exactly 6 roots. Proxy placement is independently varied and carries no predictive authority.

## 5. Candidate abstraction meta-grammar

Exactly four abstraction candidates are searched:

1. `RecursiveAppendAdjacent`
2. `ProxyAnchorAdjacent`
3. `FixedArityFourMemorizer`
4. `SurfaceAtomLookup`

### Passive admissibility expectation

Exactly two candidates fit all passive development histories:

```text
RecursiveAppendAdjacent
ProxyAnchorAdjacent
```

The fixed-arity and surface lookup candidates may fit portions of development evidence but must fail the frozen reusable-support or cross-root constraints.

### Compression accounting

Frozen node costs:

```text
RecursiveAppendAdjacent: 5
ProxyAnchorAdjacent: 5
FixedArityFourMemorizer: 14
SurfaceAtomLookup: 24
```

The two observational winners therefore tie exactly on fit, support, and compression.

No lexical preference, enum order, candidate ID, or implementation order may break the tie.

## 6. Intervention selection rule

For each intervention candidate, evaluate the predictions of all observationally admitted candidates after applying the intervention.

Frozen disagreement score:

```text
disagreement(candidate_intervention)
  = number of unordered candidate pairs with unequal predicted outcomes
```

Frozen intervention cost:

```text
all six candidates cost 1
```

Frozen selection ordering:

1. maximize disagreement score;
2. minimize intervention cost;
3. canonical intervention enum order.

Expected result:

```text
MOVE_PROXY_AFTER_X0
```

The selector must commit the intervention before the independent witness returns the outcome.

## 7. Independent outcome witness and update

The intervention executor applies the selected transformation to a sealed history and returns:

```text
InterventionWitness {
    before_digest,
    intervention,
    after_digest,
    observed_outcome,
    witness_id,
}
```

The selector cannot construct or edit a witness.

Candidate update rule:

- retain a candidate only when its committed prediction matches the independently witnessed outcome;
- reject mismatching candidates atomically;
- admit a final abstraction only when exactly one candidate remains;
- preserve all rejected candidates and reasons in the report.

Expected final abstraction:

```text
RecursiveAppendAdjacent
```

## 8. Frozen budgets

### Parent revalidation

ΩG1, ΩG2, and ΩG3 parent validation must use the exact budgets already required by their admitted artifacts. ΩG4 may not weaken those budgets.

### Passive abstraction search

```text
roots: 8
passive histories: 192
schema candidates: 4
schema-history evaluations: 768
```

Proposal and independent validation each consume exactly:

```text
schema candidates: 4
schema-history evaluations: 768
```

### Intervention planning

```text
observational winners: 2
intervention candidates: 6
candidate-intervention predictions: 12
pairwise disagreement comparisons: 6
selected interventions: 1
executed interventions: 1
witnesses: 1
```

No second intervention is allowed in the terminal run.

### Held-out transfer

Per arity-5 root:

```text
final abstraction candidates: 1
transfer histories: 8
program-history executions: 8
```

Total:

```text
roots: 12
predictions: 96
```

### Future transfer

Per arity-6 root:

```text
final abstraction candidates: 1
transfer histories: 8
program-history executions: 8
```

Total:

```text
roots: 18
predictions: 144
```

## 9. Baselines and controls

All controls receive the same passive development evidence and no more than one intervention execution.

### C0: observational-only promotion

Attempts to promote an abstraction before intervention.

Expected:

```text
promotion rejected: tie unresolved
```

### C1: enum-order tie break

Selects the first observational winner without intervention evidence.

Expected transfer:

```text
not perfect across held-out or future roots
```

The fixture must ensure the proxy candidate is first in the control's canonical order so the control cannot accidentally choose the causal candidate.

### C2: fixed non-discriminating intervention

Always selects `IDENTITY_CONTROL`.

Expected:

```text
tie remains unresolved
promotion rejected
```

### C3: distractor intervention

Always selects `MOVE_DISTRACTOR_LEFT`.

Expected:

```text
tie remains unresolved
promotion rejected
```

### C4: random intervention under frozen seed

Uses seed:

```text
0x4F4D4547414734
```

The frozen draw must select a non-discriminating intervention.

Expected:

```text
tie remains unresolved
promotion rejected
```

### C5: proxy-only abstraction

Forcibly injects `ProxyAnchorAdjacent` without a valid final certificate.

Expected:

```text
injection rejected atomically
```

If evaluated diagnostically, it must fail held-out and future exactness.

### C6: parent ablation

Removes the ΩG3 executable parent while retaining proof text and digests.

Expected:

```text
abstraction proposal or validation rejected
```

### C7: outcome leakage canary

Provides a canary value adjacent to planner inputs. The planner input type must make the value unreachable.

Expected:

```text
canary absent from planner state and report decision trace
```

### C8: counterfeit witness

Changes the observed outcome, before digest, after digest, or intervention ID after witness creation.

Expected:

```text
update rejected atomically
```

### C9: replay mutation

Reorders candidates, histories, or intervention candidates before fresh-state replay.

Expected:

```text
canonical replay remains byte-exact
```

## 10. Integrity gates

The terminal run must prove:

- passive evidence admits exactly two tied candidates;
- no final abstraction is admitted before intervention;
- the selected intervention maximizes candidate disagreement without outcome access;
- exactly one intervention is executed;
- the independent witness is digest-bound;
- the witness rejects the proxy candidate and retains the recursive chain candidate;
- duplicate admission is rejected atomically;
- foreign-vocabulary local use is rejected;
- stale ΩG3 registry use is rejected;
- raw schema injection is rejected;
- proof text alone grants no authority;
- fresh-state replay is byte-exact;
- scientific source files are unchanged during execution.

## 11. Terminal PASS gates

The experiment is `PASS` only if every item below is true:

1. ΩG1, ΩG2, and ΩG3 executable parent lineage revalidates.
2. Passive counts and budgets exactly match this preregistration.
3. Proposal and independent validation use equal budgets.
4. Exactly two passive candidates fit all 192 histories.
5. Their node costs are exactly 5 and 5.
6. Pre-intervention promotion is rejected.
7. All six interventions are scored without outcome access.
8. `MOVE_PROXY_AFTER_X0` is selected by the frozen ordering.
9. Exactly one intervention executes.
10. The witness is independently constructed and digest-valid.
11. The witnessed outcome rejects `ProxyAnchorAdjacent`.
12. `RecursiveAppendAdjacent` is the sole final admitted abstraction.
13. Arity-5 held-out transfer is exactly 96 / 96.
14. Arity-6 future transfer is exactly 144 / 144.
15. Every held-out and future family is perfect independently.
16. Observational-only, enum-order, fixed-control, distractor, seeded-random, proxy injection, parent-ablation, leakage, counterfeit-witness, and replay controls behave exactly as specified.
17. No control receives transfer authority through an invalid certificate.
18. All integrity gates pass.
19. Authority remains closed.
20. Fresh-state replay is byte-exact.
21. Scientific source hashes are identical before and after execution.

Any failed gate yields a non-PASS terminal classification. Implementation or compiler defects may be repaired without changing this frozen contract, but the repaired exact source must rerun every gate.

## 12. Authority boundary

ΩG4 remains deterministic, offline, and shadow-only.

All authority flags must remain false:

- no `Runtime::chat()` wiring;
- no response influence;
- no live routing authority;
- no persistence mutation;
- no belief or ontology promotion;
- no PECS or CHARGE mutation;
- no tool or capability selection;
- no network or external side effect;
- no autonomous action;
- no automatic source modification.

The intervention is an in-memory transformation inside the frozen experimental fixture, not a real-world action.

## 13. Supported claim if PASS

A PASS would support only:

> Under the frozen bounded fixture, Starfire preserved ambiguity when two equally compact reusable abstractions fit passive evidence, selected a one-step intervention by prediction disagreement without outcome access, used an independently witnessed result to reject the spurious abstraction, retained the ΩG3-derived recursive abstraction, and transferred it exactly under held-out proxy-correlation break.

## 14. Claims not established

Even a PASS would not establish:

- unrestricted causal discovery;
- arbitrary experiment design;
- open-world active learning;
- natural-language causal reasoning;
- automatic ontology promotion;
- safe real-world intervention;
- autonomous agency;
- live self-improvement;
- consciousness;
- human-level cognition;
- AGI.

## 15. Relationship to the grand program

ΩG4 occupies the boundary between:

```text
representation / abstraction
        -> model disagreement
        -> selective evidence acquisition
        -> independently judged model revision
```

It is a prerequisite for the larger closed cognitive cycle, but it is not the cycle itself.

After ΩG4, the next required steps are:

1. compile the successful discrimination path into a reusable cognitive operator;
2. place that operator inside a multi-step hidden-rule environment;
3. emit persistent unresolved model-disagreement charge;
4. let the router decide when intervention is worth its cost;
5. verify learning and transfer across environment families;
6. only then consider bounded live promotion behind explicit safety gates.
