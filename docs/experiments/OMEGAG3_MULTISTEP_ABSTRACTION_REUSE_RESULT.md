# ΩG3 Multi-Step Abstraction and Reuse — Terminal Result

Status: **PASS**

Local date: **2026-07-19 America/Detroit**  
UTC date: **2026-07-20**

## Frozen provenance

- Preregistration: `docs/experiments/OMEGAG3_MULTISTEP_ABSTRACTION_REUSE.md`
- Frozen preregistration commit: `723f1233db6573d117212e064c2d6a113640c855`
- Required parent result: ΩG2 recursive grammar composition `PASS`
- Required ΩG2 parent: `ConsecutiveChain3`
- Selected abstraction: `RecursiveAppendAdjacent`

The fixture, search spaces, budgets, controls, gates, authority boundary, and claim boundary were frozen before implementation.

## Exact-source execution

The terminal verdict executed from GitHub-verified commit:

```text
17632ba96542cbc970d9090f5d29806ed3989f69
```

Vercel deployment:

```text
dpl_HjsRZJU3szqZ4gQuv4QLPrNtFPGP
https://vercel.com/autostackdev/star/HjsRZJU3szqZ4gQuv4QLPrNtFPGP
```

The deployment reached `READY`. It compiled all default-feature targets, ran the scoped ΩG3 kernel tests, ran the frozen ΩG3 probe, verified the machine report, and completed the normal Next.js build.

Scientific source hashes were identical before and after execution:

```text
lib/multistep_abstraction_reuse.rs
d057cf74a2ef9173043539d0abdc06d6baf4af86918e8b1adb6fe274eea6e125

lib/examples/omega_g3_multistep_abstraction_reuse.rs
1059e38deda2bada6c3560ff7c6fdd8d6604c28bcb9dc3535d4eb503b9b4624e
```

Machine-report digest:

```text
abb53baf439a4d21991fe94d7b9ca8a738cf818a515f8b39f16b5fd51cda931e
```

The machine report emitted:

```text
terminal_classification: PASS
```

## Development synthesis and abstraction

The development set contained twelve independently synthesized and validated concrete solutions:

```text
4 roots at arity 3
4 roots at arity 4
4 roots at arity 5
```

The frozen concrete search totals were reproduced exactly:

```text
candidate programs: 1,080
program-history executions: 6,336
exact lower-level candidates: 16
```

Proposal and independent validation consumed identical budgets.

The abstraction search evaluated:

```text
4 schema candidates
12 validated examples
48 schema-example evaluations
```

Exactly one schema fit every validated example:

```text
RecursiveAppendAdjacent
```

The frozen compression gate passed:

```text
concrete node cost: 36
schema node cost: 5
compression advantage: 31
```

## Holdout arity-6 reuse

Across eight unseen arity-6 roots:

```text
perfect lower-level re-synthesis roots: 8 / 8
perfect abstraction-reuse roots: 8 / 8
lower-level correct transfer predictions: 56 / 56
abstraction correct transfer predictions: 56 / 56
```

Per root, matched search cost was:

```text
lower-level candidates: 3,600
abstraction candidates: 720
lower-level executions: 25,200
abstraction executions: 5,040
reduction factor: 5
```

## Future arity-7 reuse

Across twenty-four unseen arity-7 roots:

```text
perfect lower-level re-synthesis roots: 24 / 24
perfect abstraction-reuse roots: 24 / 24
lower-level correct transfer predictions: 192 / 192
abstraction correct transfer predictions: 192 / 192
```

Per root, matched search cost was:

```text
lower-level candidates: 70,560
abstraction candidates: 5,040
lower-level executions: 564,480
abstraction executions: 40,320
reduction factor: 14
```

Each future family independently passed:

| Future family | Perfect reuse roots | Correct transfer predictions | Maximum control predictions |
|---|---:|---:|---:|
| `future_thermal` | 8 / 8 | 64 / 64 | 0 |
| `future_software` | 8 / 8 | 64 / 64 | 0 |
| `future_watershed` | 8 / 8 | 64 / 64 | 0 |

## Causal and integrity controls

All frozen controls passed:

- ΩG2 parent dependence preserved;
- parent-proof text alone could not authorize abstraction;
- single-example support rejected;
- single-arity support rejected;
- fixed-arity memorization produced zero transfer authority;
- concrete atom memorization produced zero transfer authority;
- shuffled outcomes rejected;
- counterfeit concrete proof rejected;
- counterfeit abstraction proof rejected;
- foreign abstraction certificate rejected atomically;
- stale ΩG2 registry rejected;
- raw schema injection rejected atomically;
- duplicate abstraction admission rejected atomically;
- duplicate local reuse admission rejected atomically;
- foreign-vocabulary local admission rejected;
- problem-digest mismatch rejected;
- fresh-state replay was byte-exact.

All transfer-evaluated controls produced zero successful predictions.

## Authority boundary

All authority flags remained false:

- no `Runtime::chat()` wiring;
- no generated-response influence;
- no routing authority;
- no persistence mutation;
- no belief or ontology promotion;
- no PECS or CHARGE mutation;
- no tool or capability selection;
- no external side effects;
- no autonomous action;
- no automatic source modification.

ΩG3 remains offline, deterministic, and shadow-only.

## Supported claim

The PASS supports only this bounded statement:

> Under the frozen chain fixture, Starfire factored independently synthesized and validated arity-3, arity-4, and arity-5 lower-level expressions into one proof-carrying recursive parameterized production, independently validated and admitted that abstraction, and reused it on unseen arity-6 and arity-7 tasks with exact transfer and the preregistered matched reduction in candidate search and program-history executions. The abstraction required the admitted ΩG2 executable parent, while lower-level re-synthesis remained correct but more expensive.

## Claims not established

This result does not establish unrestricted program synthesis, arbitrary recursive abstraction, general concept formation, open-world learning, natural-language grammar induction, automatic ontology promotion, safe live self-modification, autonomous agency, consciousness, human-level cognition, or AGI.

## Next scientific step

The next experiment should test abstraction selection under competing reusable structures. More than one compression hypothesis should fit development evidence, and held-out intervention should be required to select the causally useful abstraction.