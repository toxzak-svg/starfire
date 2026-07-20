# ΩG2 Recursive Grammar Composition — Terminal Result

Status: **PASS**

Local date: **2026-07-19 America/Detroit**  
UTC date: **2026-07-20**

## Frozen provenance

- Preregistration: `docs/experiments/OMEGAG2_RECURSIVE_GRAMMAR_COMPOSITION.md`
- Frozen preregistration commit: `168fd9246864a005fb4691062c11112ab36c72f6`
- Required parent experiment: ΩG1 bounded grammar extension
- Required executable parent: `AdjacentBefore`
- Frozen higher-order operator: `SharedMiddleAnd`
- Admitted child: `ConsecutiveChain3`

The experiment contract was frozen before implementation. The fixture, search spaces, budgets, negative controls, success gates, claim boundary, and authority boundary were not loosened after execution began.

## Exact-source execution

The terminal Rust verdict executed from GitHub-verified commit:

```text
6f9c97af9763c60b893538fe592041abeffd651e
```

Vercel deployment:

```text
dpl_4MxTWgvZHjffkB9UvvE6Pt9Y7prX
https://vercel.com/autostackdev/star/4MxTWgvZHjffkB9UvvE6Pt9Y7prX
```

The verifier executed, in order:

```text
cargo check -p star --all-targets --locked
cargo test -p star recursive_grammar_composition --locked
cargo run -p star --example omega1_endogenous_state_space_genesis --locked
cargo run -p star --example omega_g1_bounded_grammar_extension --locked
cargo run -p star --example omega_g2_recursive_grammar_composition --locked
```

The ΩG2 machine report emitted:

```text
terminal_classification: PASS
```

The scientific source was byte-identical before and after execution:

```text
lib/recursive_grammar_composition.rs
8b480d91958f6d7ad4350a69da4c4865fc5f6e36bf26391e85a9827284aedb9b

lib/examples/omega_g2_recursive_grammar_composition.rs
53c1a1b2695054ab7d99639f4650e0db0057ae8447e054161e3e581c675d287d
```

Machine-report digest:

```text
97c9295eaf55f2b43acc6bb21a94d81eb7bc20a4832c19696aa1ca17798ac039
```

The first terminal run completed every Rust scientific gate and wrote the PASS report before a later, unrelated temporary UI packaging step failed because a redundant `npm ci` removed an installed PostCSS development dependency. That packaging defect occurred after the machine report, did not modify scientific source, and did not alter the terminal ΩG2 verdict. The temporary verifier was then corrected and is removed before mainline merge.

## Frozen expressibility ladder

Every development root reproduced the preregistered ladder exactly:

| Language | Candidates | Canonical partitions | Best repaired defects |
|---|---:|---:|---:|
| G0 | 315 | 26 | 204 / 368 |
| Single M1 | 60 | 60 | 328 / 368 |
| C1 recursive composition | 60 | 60 | 368 / 368 |

C1 produced exactly one complete local repair per development root.

## Parent dependence

The child was constructed only from an independently revalidated, registry-admitted ΩG1 `AdjacentBefore` handle. The validator rebound the exact ΩG1 cohort, problem digest, proof identifier, admitted kind, and registry lineage.

A raw schema value, copied proof text, stale registry, foreign certificate, counterfeit proof, or missing parent handle could not authorize child construction.

When the executable ΩG1 parent was ablated:

```text
legal C1 candidates: 0
successful transfer predictions: 0
```

This satisfies the frozen causal requirement that removing the admitted parent removes both legal child constructibility and downstream success.

## Transfer result

### Holdout

```text
perfect roots: 8 / 8
correct transfer predictions: 192 / 192
```

### Future vocabulary

```text
perfect roots: 24 / 24
correct transfer predictions: 576 / 576
```

Each future family independently passed:

| Future family | Perfect roots | Maximum control predictions |
|---|---:|---:|
| `future_thermal` | 8 / 8 | 0 |
| `future_software` | 8 / 8 | 0 |
| `future_watershed` | 8 / 8 | 0 |

Every admitted root produced `24 / 24` correct transfer predictions.

## Negative controls

Each transfer-evaluated control produced zero successful predictions per root:

```text
base_g0_only: 0
m1_single_only: 0
parent_proof_text_only: 0
parent_ablated: 0
delayed_parent_admission: 0
```

Integrity controls also passed:

- duplicate child admission rejected atomically;
- foreign parent certificate rejected atomically;
- counterfeit parent proof rejected;
- stale parent registry rejected;
- raw schema injection rejected;
- foreign child certificate rejected atomically;
- counterfeit child-certificate construction rejected;
- problem-digest mismatch rejected;
- shuffled-development proof rejected.

## Budget and replay gates

Proposal and independent validation consumed identical frozen budgets. Each development root executed:

```text
96 vocabulary-history scans
4,560 unordered history-pair evaluations
315 G0 candidates
30,240 G0 program-history executions
60 single-M1 candidates
5,760 single-M1 program-history executions
60 C1 candidates
5,760 C1 program-history executions
```

Each holdout and future local synthesis and validation executed:

```text
96 vocabulary-history scans
4,560 unordered history-pair evaluations
60 child bindings
5,760 child program-history executions
```

Fresh-state replay was byte-exact.

## Authority boundary

All authority flags remained false:

- no `Runtime::chat()` wiring;
- no response influence;
- no routing authority;
- no persistence mutation;
- no belief or ontology promotion;
- no PECS or CHARGE mutation;
- no tool or capability selection;
- no external side effects;
- no autonomous action;
- no automatic source modification.

ΩG2 remains offline, deterministic, and shadow-only.

## Supported claim

The PASS supports only this bounded statement:

> Under the frozen five-atom permutation regime, Starfire required a previously validated and admitted ΩG1 `AdjacentBefore` production as an executable parent, exhaustively established that G0 and every single M1 production were insufficient for the target distinction, composed the admitted parent through the frozen `SharedMiddleAnd` operator into a generic arity-3 production, independently validated and admitted that recursively dependent production, and used it to synthesize independently validated local refinements across unseen vocabularies. Ablating the admitted parent removed legal child construction and downstream success.

## Claims not established

This result does not establish unrestricted grammar invention, arbitrary composition depth, unbounded recursive metalanguage growth, natural-language grammar acquisition, open-world ontology learning, automatic promotion readiness, safe live self-modification, general intelligence, consciousness, or human-level cognition.

## Next scientific step

The frozen preregistration names ΩG3 as multi-step abstraction and reuse: independently discovered composed expressions must expose a deeper shared structure that can be factored into a parameterized production, then beat matched re-synthesis from lower-level primitives while preserving proof-carrying lineage through every abstraction layer.
