# ΩG2 — Recursive Grammar Composition

Status: **preregistered before implementation and before any verdict-producing run**.

Implementation is **blocked on a verified ΩG1 PASS**. This document is frozen on a stacked branch whose parent is the current ΩG1 research branch. No ΩG2 implementation may be used to rescue, reinterpret, or retroactively alter ΩG1.

## Research break from ΩG1

ΩG1 asks whether Starfire can exhaust its inherited fixed refinement grammar, establish that the grammar is insufficient, select one generic production from a separately frozen bounded meta-grammar, independently validate that extension, and use the admitted production across unseen vocabularies.

Even a successful ΩG1 result would still leave the useful production itself developer-enumerated inside M1.

ΩG2 asks the narrower next question:

> Can Starfire require a previously admitted production as an executable parent, compose that admitted production through a separately frozen higher-order composition operator, independently validate the resulting generic production, and use the new recursively dependent production across unseen vocabularies where G0 and every single M1 production remain insufficient?

The causal requirement is stronger than ordinary feature synthesis:

> removing the admitted ΩG1 parent must remove both legal constructibility of the ΩG2 candidate and successful downstream repair.

This is bounded recursive grammar composition. It is not unrestricted code generation, arbitrary grammar induction, or recursive self-modification.

## Frozen dependency gate

ΩG2 implementation and verdict production are forbidden unless ΩG1 first produces a committed-source `PASS` under its frozen contract.

The ΩG2 implementation must bind to all of the following parent evidence:

```text
parent experiment:       ΩG1 bounded grammar extension
required parent result:  PASS
required admitted kind:  AdjacentBefore
required parent arity:   2
```

The child validator must independently bind the parent production to:

- the ΩG1 cohort identifier;
- the ΩG1 problem digest;
- the validated ΩG1 proof identifier;
- the admitted production kind;
- the exact parent registry signature.

A raw `ExtensionKind`, serialized proof text, copied enum value, stale registry snapshot, foreign certificate, or counterfeit parent object is not an executable parent capability.

## Frozen inherited languages

### G0

The inherited base grammar is unchanged:

```text
metric  := FirstIndex(atom) | LastIndex(atom) | Count(atom)
program := metric_a < metric_b | metric_a == metric_b
```

For a five-atom vocabulary:

```text
15 metrics
315 G0 programs
```

### M1

The ΩG1 meta-grammar is unchanged:

```text
AdjacentBefore(x, y)       := index(y) == index(x) + 1
ExactlyOneBetween(x, y)    := index(y) == index(x) + 2
WithinTwoBefore(x, y)      := index(x) < index(y) && index(y) - index(x) <= 2
```

For a five-atom vocabulary:

```text
3 schemas
20 ordered bindings per schema
60 single M1 bound programs
```

M1 is exhaustively evaluated only as a **single-production** language in the insufficiency ladder. ΩG2 does not allow the composer to read raw M1 candidates directly.

## Frozen composition operator C1

ΩG2 adds exactly one higher-order composition operator:

```text
SharedMiddleAnd(P, Q)(x, y, z) := P(x, y) && Q(y, z)
```

Constraints:

1. `P` and `Q` must each be opaque handles to independently validated, registry-admitted binary productions.
2. Raw enum values or meta-grammar candidates are not legal operands.
3. `x`, `y`, and `z` must be three distinct root-local atoms.
4. The composer may not inspect fixture role labels, target atom identities, family labels, transfer labels, or hidden objective parameters.
5. The generic child production carries parent-production references and the C1 operator only. Root-local atom bindings are evidence, not admitted global constants.

Under the required ΩG1 parent registry, the only legal parent pair is:

```text
P = AdjacentBefore
Q = AdjacentBefore
```

The resulting generic child production is semantically:

```text
ConsecutiveChain3(x, y, z)
    := AdjacentBefore(x, y) && AdjacentBefore(y, z)
```

The human-readable name `ConsecutiveChain3` is descriptive only. Admission authority derives from the validated parent references, operator identity, proof digest, and registry lineage.

For a five-atom vocabulary, one legal parent pair yields exactly:

```text
5 × 4 × 3 = 60 ordered local bindings
```

## Frozen fixture

Each root has a private five-atom vocabulary. Local atoms are generated in a canonical hidden role order for fixture construction, then exposed only as opaque atom identities in raw histories.

For each root:

```text
120 total permutations of five distinct atoms
96 discovery histories
24 transfer histories
1 shared intervention atom
2 witnessed outcome atoms
```

The transfer split is fixed before outcome use:

```text
transfer iff lexicographic permutation index mod 5 == 0
discovery otherwise
```

The hidden positive outcome occurs exactly when the root-local role-0, role-1, and role-2 atoms form one consecutive ordered chain:

```text
index(role1) == index(role0) + 1
AND
index(role2) == index(role1) + 1
```

Those role identities are available only to fixture construction and the independent objective checker. They are not supplied to vocabulary derivation, G0 exhaustion, M1 exhaustion, parent validation, C1 composition search, generic-production ranking, local refinement search, downstream prediction, or certificate admission.

The exact per-root outcome counts are frozen:

```text
discovery: 4 positive, 92 negative
transfer:  2 positive, 22 negative
```

The frozen cohort is:

```text
8 development roots
8 holdout roots
24 future roots
3 future domain families × 8 roots
```

Every root must reproduce the exact outcome counts above.

## Frozen expressibility ladder

Before any C1 composition search, proposer and validator must independently reproduce the complete insufficiency ladder on every development root.

### Level 0 — base alias defect

All histories share the same order-blind base state key and intervention. Opposite outcomes are therefore aliased before refinement.

The exact discovery defect count per root is:

```text
4 × 92 = 368 opposite-outcome alias defects
```

### Level 1 — exhaustive G0 insufficiency

The exact inherited G0 search must produce:

```text
315 candidate programs
26 canonical binary partitions up to boolean complement
best repair: 204 / 368 defects
```

No G0 program may repair all 368 defects.

### Level 2 — exhaustive single-production M1 insufficiency

All 60 bound M1 programs must be executed as single predicates.

The exact M1 search must produce:

```text
60 candidate programs
60 canonical binary partitions up to boolean complement
best repair: 328 / 368 defects
```

The frozen best single M1 repair corresponds to the role-0/role-2 `ExactlyOneBetween` relation. The role identities are objective-checker metadata only and are not visible to the searcher.

No single M1 production may repair all 368 defects.

### Level 3 — admitted-parent C1 constructibility

Only after the parent ΩG1 production has been independently revalidated and exposed as an admitted executable parent handle may C1 instantiate candidates.

With the admitted `AdjacentBefore` parent used in both parent slots:

```text
60 C1 bound candidates
60 canonical binary partitions up to boolean complement
unique complete repair: 368 / 368 defects
```

The unique complete local repair is the hidden role-0 → role-1 → role-2 chain.

Any disagreement with these exact frozen counts is a fixture or implementation failure, not a tunable result.

## Generic composed-production proposal

For each development root, the proposer must:

1. derive the five-atom vocabulary from discovery histories;
2. detect the exact 368 base alias defects;
3. exhaust all 315 G0 programs;
4. exhaust all 60 single M1 bound programs;
5. independently validate the required ΩG1 parent lineage before composition;
6. enumerate all 60 ordered C1 local bindings for the legal parent pair;
7. execute every candidate on all 96 discovery histories;
8. canonicalize binary partitions up to boolean complement;
9. rank candidates by repaired defects, minimum partition support, then canonical syntax;
10. record the root-local winner without exposing its atom bindings as global constants.

The generic child candidate is identified by:

```text
operator: SharedMiddleAnd
left parent:  admitted AdjacentBefore production
right parent: admitted AdjacentBefore production
arity: 3
```

A generic child certificate is forbidden unless:

- G0 is incomplete on every development root;
- single-production M1 is incomplete on every development root;
- C1 has one complete local repair on every development root;
- the same operator and parent-production lineage explain all development roots;
- no root-local atom identity is embedded in the generic child production.

## Independent validation and admission

The validator must independently recompute:

- parent ΩG1 result and proof binding required by the child experiment;
- parent cohort, problem, proof, certificate, and registry digests;
- every root vocabulary and exact outcome count;
- all 368 alias defects per development root;
- all 315 G0 candidates and 26 canonical partitions;
- the exact G0 ceiling of 204 repaired defects;
- all 60 single M1 candidates and 60 canonical partitions;
- the exact single-M1 ceiling of 328 repaired defects;
- all 60 legal C1 candidates and 60 canonical partitions;
- the unique 368-defect complete C1 repair;
- all proof fields and deterministic digests.

Only an opaque `ValidatedComposedProductionCertificate` produced by full recomputation may alter the composed-production registry.

Admission adds one generic arity-3 production whose executable semantics are derived from C1 plus two validated parent handles. It does not add:

- development-root atom bindings;
- cached partitions;
- target role identities;
- outcome labels;
- fixture parameters;
- raw M1 candidate access.

Foreign, duplicate, stale-parent, parent-registry-mismatched, problem-digest-mismatched, counterfeit, and proof-tampered admissions must fail atomically.

## Local refinement after child admission

For each holdout and future root, the local child-refinement synthesizer receives:

- raw discovery histories and outcomes;
- the composed-production registry;
- the validated parent registry lineage required by the child production;
- no target role identities;
- no development bindings;
- no family or split labels.

It must enumerate all 60 local bindings of the admitted arity-3 child production and produce a root-local executable refinement only when one repairs all detected alias defects.

A second independent recomputation must validate the local refinement before it can modify the state key.

The downstream predictor receives only executable state keys and discovery outcome labels. It may not inspect raw history, target roles, parent proof text, child proof text, schema scores, family labels, or fixture parameters. Conflicting discovery outcomes for one state key require abstention.

## Frozen causal and integrity paths

### Transfer-evaluated paths

Every holdout and future root is evaluated under:

1. `admitted_recursive_composition`
   - valid ΩG1 parent lineage is independently revalidated;
   - parent production is present as an admitted executable handle;
   - valid ΩG2 child certificate is admitted before local synthesis;
   - independently validated local child refinement is admitted before prediction.

2. `base_g0_only`
   - exhaustive G0 search only;
   - no M1 or C1 state-key refinement.

3. `m1_single_only`
   - exhaustive G0 plus all 60 single M1 candidates;
   - no composition operator use.

4. `parent_proof_text_only`
   - the valid serialized ΩG1 proof is retained;
   - no executable parent handle is admitted;
   - C1 construction authority remains unavailable.

5. `parent_ablated`
   - starts from the same verified parent lineage;
   - the experimental read-only registry view excludes the admitted parent before child construction;
   - C1 has zero legal executable parent pairs.

6. `delayed_parent_admission`
   - the valid parent is admitted only after the transfer prediction window.

### Atomic integrity controls

The following are tested independently and must be rejected before any child admission:

- `foreign_parent_certificate`;
- `counterfeit_parent_certificate`;
- `stale_parent_registry`;
- `raw_schema_injection` attempting to substitute a bare `AdjacentBefore` enum for an admitted handle;
- `foreign_child_certificate`;
- `counterfeit_child_certificate`;
- `duplicate_child_admission`;
- `outcome_shuffled_development` proof validated against the frozen unshuffled cohort.

Rejected controls must not partially mutate parent or child registries.

## Frozen budget accounting

Per development root, proposer and validator each perform exactly:

```text
96 vocabulary history scans
4,560 unordered history-pair evaluations
315 G0 candidate programs
30,240 G0 program-history executions
60 single M1 candidate programs
5,760 single M1 program-history executions
60 C1 bound candidate programs
5,760 C1 program-history executions
```

The admitted recursive-composition path therefore adds no unbounded search. C1 search is exactly 60 local candidates per root under the one-parent ΩG1 registry.

Per holdout/future root, child local synthesis and local validation each perform exactly:

```text
96 vocabulary history scans
4,560 unordered history-pair evaluations
60 admitted-child local bindings
5,760 child program-history executions
```

`base_g0_only` and `m1_single_only` must consume their complete declared exhaustive budgets and may not early-exit on failure.

Parent-disabled paths must report zero legal C1 candidates rather than secretly evaluating raw M1 syntax through the composition operator.

All counters use saturating integer accounting and participate in deterministic replay.

## Frozen success gates

A `PASS` requires all of the following:

1. ΩG1 has a committed-source `PASS` under its own frozen contract;
2. exact 8/8/24 ΩG2 cohort and three future families;
3. exact 96/24 discovery-transfer split per root;
4. exact 4/92 discovery and 2/22 transfer outcome counts per root;
5. exactly 368 base alias defects per root;
6. exact G0 candidate, execution, canonical-partition, and 204/368 ceiling counts;
7. exact single-M1 candidate, execution, canonical-partition, and 328/368 ceiling counts;
8. no G0 or single M1 candidate completely repairs any development, holdout, or future root;
9. the parent ΩG1 production is independently rebound to its exact cohort, problem digest, proof, certificate, and registry lineage;
10. raw schema values and proof text cannot substitute for an admitted executable parent handle;
11. exact 60 C1 candidates, 60 canonical partitions, and one complete 368/368 local repair on every development root;
12. one generic arity-3 child production is independently validated and admitted without root-local atom constants;
13. foreign, counterfeit, stale, duplicate, raw-injection, registry-mismatched, and problem-digest-mismatched admissions fail atomically;
14. admitted child local synthesis and independent validation succeed on every holdout and future root;
15. `admitted_recursive_composition` transfer prediction is 24/24 on every holdout and future root;
16. `base_g0_only`, `m1_single_only`, `parent_proof_text_only`, `parent_ablated`, and `delayed_parent_admission` produce 0 successful transfer predictions per root;
17. removing the admitted ΩG1 parent removes all legal C1 candidate construction and all ΩG2 transfer success;
18. each future family independently has admitted success rate 1.0 and maximum nonrecursive control success rate 0.0;
19. exact byte-identical replay from fresh state;
20. immutable source problems and closed authority invariants;
21. no live runtime, routing, persistence, ontology-promotion, PECS/CHARGE mutation, tool, capability, external-side-effect, or autonomous-action authority.

Terminal classification is:

```text
DEPENDENCY_FAILURE
```

if ΩG1 lacks a committed-source PASS or the exact parent lineage cannot be independently rebound;

```text
CONTROL_FAILURE
```

if any budget, authority, source-immutability, admission-integrity, or negative-control gate fails;

```text
REPLAY_FAILURE
```

if deterministic replay fails;

```text
PASS
```

only if every frozen gate passes; otherwise:

```text
REJECTED
```

## Supported claim if PASS

A PASS would support only this bounded statement:

> Under the frozen five-atom permutation regime, Starfire required a previously validated and admitted ΩG1 `AdjacentBefore` production as an executable parent, exhaustively established that G0 and every single M1 production remained insufficient for a three-link target distinction, composed the admitted parent through the frozen `SharedMiddleAnd` operator into a generic arity-3 production, independently validated and admitted that recursively dependent production, and used it to synthesize independently validated local refinements across unseen vocabularies. Ablating the admitted parent removed legal child construction and downstream success.

## Claims explicitly not established

A PASS would not establish:

- unrestricted grammar invention;
- arbitrary composition depth;
- recursive unbounded metalanguage growth;
- generation of arbitrary syntax or executable source code;
- natural-language grammar acquisition;
- open-world ontology learning;
- autonomous ontology replacement;
- automatic promotion readiness;
- safe live self-modification;
- general intelligence, consciousness, or human-level cognition.

## Authority boundary

ΩG2 is offline, deterministic, feature-independent, and shadow-only. It adds no `Runtime::chat()` wiring, response influence, live routing, persistent memory mutation, belief or ontology promotion, CHARGE/PECS authority, tool selection, capability invocation, external side effects, autonomous action, or automatic source-code modification.

## Scientific next step after a PASS

ΩG3 should test **multi-step abstraction and reuse**: independently discovered local composed expressions must share a deeper structural pattern that can be factored into a more general parameterized production, and that abstraction must improve held-out synthesis efficiency or expressibility without leaking root-specific bindings. A valid ΩG3 design must compare reuse against matched re-synthesis from lower-level primitives and must preserve proof-carrying lineage through every abstraction layer.
