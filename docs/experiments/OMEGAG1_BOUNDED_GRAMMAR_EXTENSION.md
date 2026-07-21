# ΩG1 — Bounded Grammar Extension

Status: **preregistered before implementation and before any verdict-producing run**.

## Research break from Ω1 and R1-A

Ω1 showed that Starfire can search a developer-fixed executable predicate grammar and admit a new state-key dimension. R1-A/H13-C showed that a proof-carrying structural role can transfer through larger and altered synthetic graphs. Neither result shows that Starfire can detect that its **representation-synthesis grammar is itself insufficient** and add a new executable production.

ΩG1 asks the narrower next question:

> Can Starfire exhaust its current bounded refinement grammar, prove that no existing production repairs a witnessed alias defect, select one production schema from a separately frozen meta-grammar, independently validate that extension, admit it through an opaque certificate, and then use the newly admitted production to synthesize local state refinements across unseen vocabularies?

This is a bounded grammar-growth experiment. It is not unrestricted language invention.

## Frozen base grammar G0

The inherited Ω1 grammar is frozen unchanged:

```text
metric  := FirstIndex(atom) | LastIndex(atom) | Count(atom)
program := metric_a < metric_b | metric_a == metric_b
```

For a five-atom vocabulary, exhaustive G0 enumeration contains exactly:

```text
15 metrics
315 programs
```

Every ΩG1 discovery history contains each local atom exactly once. The hidden regime depends on an ordered local adjacency relation. No single G0 comparison can express that partition over the frozen discovery cohort.

The proposer and validator must independently enumerate and execute all 315 G0 programs for every root. A grammar-extension certificate is forbidden unless the best G0 partition repairs strictly fewer than all detected defects on every development root.

## Frozen bounded meta-grammar M1

The meta-grammar contains exactly three generic binary production schemas:

```text
AdjacentBefore(x, y)       := index(y) == index(x) + 1
ExactlyOneBetween(x, y)    := index(y) == index(x) + 2
WithinTwoBefore(x, y)      := index(x) < index(y) && index(y) - index(x) <= 2
```

For a five-atom vocabulary, each schema has 20 ordered bindings, giving exactly:

```text
3 schemas
60 bound candidate programs per root
```

The schema set, semantics, ranking, tie-breaking, cohort, budgets, controls, and gates are frozen by this document. They may not be changed after the first complete verdict-producing run to rescue a failure.

## Frozen fixture

Each root has a private five-atom vocabulary. Atom names, family names, and root identifiers are unavailable to the grammar ranker except through the raw histories it receives.

For each root:

```text
120 total permutations of five distinct atoms
96 discovery histories
24 transfer histories
1 shared intervention atom
2 witnessed outcome atoms
```

The transfer split is selected before outcome use by the frozen permutation-index rule:

```text
transfer iff lexicographic permutation index mod 5 == 0
discovery otherwise
```

The hidden outcome regime is positive exactly when the root-local role-0 atom occurs immediately before the root-local role-1 atom. Those target identities are used only by fixture construction and the independent objective checker. They are not supplied to vocabulary derivation, G0 exhaustion, schema ranking, local binding search, downstream prediction, or certificate validation.

The frozen cohort is:

```text
8 development roots
8 holdout roots
24 future roots
3 future domain families × 8 roots
```

Every root must contain both outcomes in discovery and transfer.

## What counts as a grammar insufficiency witness

For each root, all histories have the same order-blind base state key and the same intervention. Opposite-outcome histories are therefore aliased before refinement.

A valid insufficiency witness requires:

1. at least one witnessed alias defect;
2. exhaustive G0 enumeration;
3. no G0 partition repairs all defects;
4. the best G0 score and canonical representative are recorded;
5. the proposer and validator independently reproduce the same G0 ceiling.

No extension search may begin from a hand-selected atom pair or target schema.

## Schema ranking

For every development root and every schema:

1. derive the local vocabulary from discovery histories;
2. enumerate all 20 ordered bindings;
3. execute every bound program on all 96 discovery histories;
4. canonicalize binary partitions up to boolean complement;
5. retain the lexicographically canonical program per partition;
6. rank bound programs by repaired defects, minimum partition support, then canonical syntax;
7. record the best local binding for that schema.

Schemas are ranked across all eight development roots by:

1. number of roots with complete defect repair;
2. aggregate repaired defects;
3. aggregate minimum support;
4. canonical schema name.

The winning schema must be uniquely superior under the frozen gates. The schema certificate carries the generic production only; development-root atom bindings are evidence, not admitted global constants.

## Independent validation and admission

The validator independently recomputes:

- cohort and problem digests;
- every root vocabulary and alias-defect set;
- all 315 G0 programs per root;
- all 60 M1 bound programs per root;
- all executions and canonical partitions;
- the G0 ceiling;
- every schema-local winner;
- aggregate schema ranking;
- every proof field and digest.

Only an opaque `ValidatedGrammarExtensionCertificate` produced by full recomputation may alter the `GrammarRegistry`.

Admission adds one generic production schema. It does not add the development target atoms, a cached partition, outcome labels, or a root-specific refinement.

Foreign, duplicate, counterfeit, cohort-mismatched, and proof-tampered admissions must fail atomically.

## Local refinement after schema admission

For each holdout and future root, the local refinement synthesizer receives:

- raw discovery histories and outcomes;
- the current grammar registry;
- no target atom identities;
- no development bindings;
- no family or split labels.

It must independently enumerate all bindings allowed by admitted schemas and produce a root-local executable refinement only when one repairs every detected alias defect. A second independent recomputation must validate the local refinement before it can be used in the state language.

The downstream predictor receives only executable state keys and discovery outcome labels. It cannot inspect raw history, target atoms, schema scores, proof text, family labels, or fixture parameters. A conflicting key requires abstention.

## Frozen matched paths

Every holdout and future root is evaluated under these paths:

1. `admitted_extension`
   - valid ΩG1 schema certificate admitted before local synthesis;
   - independently validated local refinement admitted before prediction.

2. `base_grammar_only`
   - exhaustive G0 search only; no schema admission.

3. `proof_text_only`
   - valid serialized proof retained; registry unchanged.

4. `wrong_schema`
   - a nonwinning M1 schema is present with matched local search budget.

5. `delayed_admission`
   - correct schema admitted only after the transfer prediction window.

6. `foreign_certificate`
   - valid certificate from a different cohort is presented and must be rejected.

7. `counterfeit_certificate`
   - structurally plausible proof fields are tampered and must fail independent validation.

8. `outcome_shuffled_development`
   - development outcomes are deterministically permuted before proposal while counts and compute are preserved; its proof must not validate against the frozen cohort.

Rejected paths still consume their declared full recomputation budget. No path may inspect the hidden target pair.

## Frozen budget accounting

Per development root, proposer and validator each perform exactly:

```text
96 vocabulary history scans
4,560 unordered history-pair evaluations
315 G0 candidate programs
30,240 G0 program-history executions
60 M1 bound candidate programs
5,760 M1 program-history executions
```

Per holdout/future root and per matched path, local synthesis and validation perform the exact candidate and execution counts implied by the registry. The admitted-extension path must evaluate 20 bindings for the one admitted schema in both proposal and validation. Base-only and control paths must execute their complete declared search rather than early-exit on failure.

All counters use saturating integer accounting and are included in deterministic replay.

## Frozen success gates

A `PASS` requires all of the following:

1. exact 8/8/24 cohort and three future families;
2. exact 96/24 discovery-transfer split per root;
3. both outcomes present in every discovery and transfer set;
4. exact G0 and M1 candidate counts and execution budgets;
5. proposer and validator reproduce every alias-defect set and G0 ceiling;
6. no G0 candidate completely repairs any development, holdout, or future root;
7. one M1 schema uniquely repairs every development root;
8. foreign, counterfeit, duplicate, and cohort-mismatched admissions are rejected atomically;
9. admitted-extension local synthesis and validation succeed on every holdout and future root;
10. admitted-extension transfer prediction is 24/24 on every holdout and future root;
11. every non-admitted, wrong-schema, delayed, or structure-destroying control produces 0 successful transfer predictions per root;
12. each future family independently has admitted success rate 1.0 and maximum control success rate 0.0;
13. exact byte-identical replay from fresh state;
14. immutable source problems and closed authority invariants;
15. no live runtime, routing, persistence, ontology-promotion, PECS-mutation, tool, capability, or autonomous-action authority.

Terminal classification is:

```text
CONTROL_FAILURE
```

if any budget, authority, source-immutability, or negative-control gate fails;

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

> Under the frozen five-atom permutation regime and three-schema meta-grammar, Starfire exhaustively established that its inherited single-comparison refinement grammar could not repair the witnessed state alias, selected and independently validated one generic executable production schema, admitted that schema through an opaque certificate, and then used it to synthesize independently validated local refinements across unseen vocabularies where non-admitted and wrong-schema controls failed.

## Claims explicitly not established

A PASS would not establish:

- unrestricted grammar invention;
- generation of arbitrary syntax or executable code;
- natural-language grammar acquisition;
- open-world ontology learning;
- recursive or unbounded metalanguage growth;
- automatic promotion readiness;
- safe live self-modification;
- general intelligence, consciousness, or human-level cognition.

## Authority boundary

ΩG1 is feature-independent, offline, deterministic, and shadow-only. It adds no `Runtime::chat()` wiring, response influence, live routing, persistent memory mutation, belief or ontology promotion, CHARGE/PECS authority, tool selection, capability invocation, external side effects, or autonomous action.

## Scientific next step after a PASS

ΩG2 should test **recursive grammar dependence**: a second useful production must be inexpressible under G0 and undiscoverable under M1 alone, but become constructible from the admitted ΩG1 production plus a frozen composition operator. Ablating the ΩG1 production must remove both expressibility and discovery of the ΩG2 repair.