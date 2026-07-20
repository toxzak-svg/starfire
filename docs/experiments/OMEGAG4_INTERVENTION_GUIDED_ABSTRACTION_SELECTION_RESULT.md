# ΩG4 Intervention-Guided Abstraction Selection Result

Status: **PASS**

Local date: **2026-07-20 America/Detroit**

## Frozen provenance

Preregistration commit:

```text
d6778cf29db725775c0d6815a6d23d6398c74010
```

Exact-source admission commit:

```text
e004330c1913c5812f1427b1e20871d02f6fb3f0
```

Vercel deployment:

```text
dpl_54TFq5qkjTBcsXDMD19atMPfqvNK
```

Deployment state:

```text
READY
```

Scientific source hashes:

```text
eaf09bddd7f16c1018db0e8b5fea524100a9f20e2a028f9dea375dd17fe4e45b  lib/intervention_guided_abstraction_selection.rs
f5f2adfd1c49e8ea6a0d47695c4fd04d96e20ec3a8084b6391ca7295f5c6ca6c  lib/examples/omega_g4_intervention_guided_abstraction_selection.rs
```

The hashes were identical before and after execution.

Machine-report digest:

```text
557501f0db4faec6773550cfd67b54c9b29eb95e3de34c04e7f9a98fb1215795
```

## Validation lane

The admission deployment completed:

```text
cargo check -p star --all-targets --locked
cargo test -p star --test omega_g4_selection --locked
cargo run -p star --example omega_g4_intervention_guided_abstraction_selection --locked
npm --prefix ui run build:app
```

The public integration tests independently verified both correlation-break directions:

1. breaking the proxy while preserving the recursive chain;
2. breaking the recursive chain while preserving the proxy anchor.

The full frozen verdict probe then reran every preregistered gate.

## Parent lineage

The executable lineage revalidated through:

```text
ΩG1 AdjacentBefore
ΩG2 ConsecutiveChain3
ΩG3 RecursiveAppendAdjacent
```

Proposal, validation, and revalidation budgets matched where required.

## Passive ambiguity result

The frozen passive set contained:

```text
roots: 8
histories: 192
schema candidates: 4
schema-history evaluations: 768
```

Exactly two reusable candidates fit all passive evidence:

```text
ProxyAnchorAdjacent
RecursiveAppendAdjacent
```

Their node costs tied exactly:

```text
5
5
```

Pre-intervention promotion was rejected. No enum order, lexical preference, or copied proof text resolved the tie.

## Intervention result

The planner scored all six frozen interventions without outcome access:

```text
intervention candidates: 6
candidate-intervention predictions: 12
pairwise disagreement comparisons: 6
selected interventions: 1
executed interventions: 1
```

Two interventions were discriminating and four were non-discriminating.

The frozen ordering selected:

```text
MoveProxyAfterX0
```

The independent digest-bound witness returned a positive outcome. This result:

- retained `RecursiveAppendAdjacent`;
- rejected `ProxyAnchorAdjacent`;
- produced exactly one final admitted abstraction.

The leakage canary remained absent from the planner trace. Counterfeit passive proofs, plans, and witnesses were rejected.

## Held-out transfer

Arity-5 correlation-break transfer:

```text
roots: 12 / 12 perfect
predictions: 96 / 96 correct
proxy control: 24 / 96 correct
proxy perfect roots: 0 / 12
```

Each held-out family passed independently:

```text
holdout_energy:   4 / 4 roots, 32 / 32 predictions
holdout_compiler: 4 / 4 roots, 32 / 32 predictions
holdout_river:    4 / 4 roots, 32 / 32 predictions
```

## Future transfer

Arity-6 future transfer:

```text
roots: 18 / 18 perfect
predictions: 144 / 144 correct
proxy control: 36 / 144 correct
proxy perfect roots: 0 / 18
```

Each future family passed independently:

```text
future_materials: 6 / 6 roots, 48 / 48 predictions
future_protocols: 6 / 6 roots, 48 / 48 predictions
future_ecology:   6 / 6 roots, 48 / 48 predictions
```

## Controls

All frozen controls behaved as preregistered:

- observational-only promotion rejected;
- enum-order control selected the proxy and failed perfect transfer;
- identity intervention left the tie unresolved;
- distractor intervention left the tie unresolved;
- frozen-seed random control selected `RotateDistractors` and left the tie unresolved;
- raw proxy injection rejected;
- ΩG3 parent ablation rejected;
- proof text alone granted no authority;
- counterfeit passive proof rejected;
- counterfeit intervention plan rejected;
- counterfeit witness rejected;
- duplicate final admission rejected atomically;
- foreign final certificate rejected;
- foreign transfer vocabulary rejected;
- fresh-state replay byte-exact.

## Authority boundary

All authority flags remained false:

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

The intervention occurred only inside the deterministic frozen fixture.

## Supported claim

ΩG4 supports the following bounded claim:

> Under the frozen fixture, Starfire preserved ambiguity when two equally compact reusable abstractions fit passive evidence, selected a one-step intervention by prediction disagreement without outcome access, used an independently witnessed result to reject the spurious abstraction, retained the ΩG3-derived recursive abstraction, and transferred it exactly under held-out proxy-correlation break.

## What it does not establish

ΩG4 does not establish:

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

## Program consequence

ΩG1 through ΩG4 now form one coherent capability staircase:

```text
learn a bounded relation
        -> compose admitted relations
        -> abstract repeated solutions
        -> preserve model uncertainty
        -> seek discriminating evidence
        -> revise from an independent outcome
        -> transfer after correlation break
```

The next milestone should compile this successful discrimination path into a reusable cognitive operator and place it inside the persistent closed cognitive cycle, where unresolved disagreement emits typed CHARGE and the router must decide whether intervention is worth its cost.
