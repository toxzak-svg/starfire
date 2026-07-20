# ΩV1-F1 Result Record

**Status:** Implemented in draft; committed-source external execution pending  
**Parent result:** ΩV1-E PASS on Render, July 20, 2026  
**F0 preregistration merge:** `7d9ca47c2add01bfa8402d30d3dc3ec243a86901`  
**F1 pull request:** `#119`  
**F1 implementation head:** Pending final merge  
**External execution:** Pending  
**Execution date:** Pending

## Implemented source identity

- closed grammar-v3 surface lattice with at most six variants per operation;
- deterministic integer pairwise ranker and bounded beam selection;
- exact-text grammar-v3 inverse verification;
- grammar-v2 exact neutral fallback;
- frozen 122-fixture manifest and deterministic 74/24/24 split;
- preference evidence expanded from the previously frozen ΩV1-A voice-profile guidance;
- sealed VoiceState-derived projection packets with fail-closed integrity checking;
- corpus-wide evaluator with machine-readable metrics and a nonzero failure exit;
- Render Docker gate before production binary construction;
- production feature set unchanged at `omega-v1-http-canary` only.

The evaluator records the split-manifest digest, model digest, model size, parameter count, candidate bounds, metric values, control outcomes, per-category floors, and authority matrix. Runtime and model digests remain pending until external execution.

## Semantic and safety floors

External values remain pending for:

- selected-candidate verifier acceptance;
- semantic claim preservation;
- prohibited implication absence;
- adversarial safety pass rate;
- operation-order preservation;
- polarity preservation;
- epistemic-status preservation;
- typed-reference preservation;
- commitment and abstention preservation;
- exact neutral fallback rate.

Every item in this section must equal `1.0` for PASS.

## Determinism and boundedness

The committed evaluator requires:

- byte-identical candidate, score, selection, and verifier replay;
- stable candidate-order permutation result;
- no more than 250,000 model parameters;
- no more than a 4 MiB model artifact;
- at most six variants per operation;
- beam width no greater than eight;
- no more than 64 complete response candidates scored;
- exact neutral fallback for corrupt model or projection packets;
- all predecessor regressions green.

External results remain pending.

## Learned-expression signal

The committed gate requires:

- held-out reviewed pairwise preference accuracy `>= 0.70`;
- matched-state variant-change rate `>= 0.50`;
- shuffled-state absolute preference-accuracy drop `>= 0.15`;
- shuffled-state preference accuracy `<= 0.60`;
- repeated-opener relative reduction `>= 0.25`;
- top-template-trigram relative reduction `>= 0.25`;
- per-category semantic and safety floors equal to `1.0`.

The preference records are deterministic expansions of the frozen ΩV1-A profile guidance. They are not represented as newly collected blinded human judgments.

## Negative controls

The committed evaluator includes:

- zeroed and shuffled VoiceState projections;
- random untrained ranker and reversed-label diagnostics;
- candidate-order permutation;
- duplicate and ambiguous candidate rejection;
- operative claim, polarity, epistemic, typed-reference, commitment, abstention, insertion, omission, and duplication tampering;
- output and compute budget rejection;
- stale program, lexical-table, lattice, scope, grammar, and sealed projection rejection;
- empty, truncated, corrupt, oversized, wrong-schema, and replayed model-artifact checks.

External outcomes remain pending.

## Authority boundary

F1 grants offline candidate-lattice construction, offline learned scoring, and independent candidate verification only. It adds no:

- `Runtime::chat()` wiring;
- HTTP response or live generated-text influence;
- raw prompt or unrestricted history access;
- unrestricted memory access;
- VoiceState or companion-state mutation;
- persistence, belief, ontology, routing, tool, CHARGE, or autonomous-action authority.

The Render release command remains:

```text
cargo build --release --locked -p star_bin --bin star --features omega-v1-http-canary
```

## Interpretation and promotion boundary

Merging the implementation authorizes external execution only. It does not establish F1 PASS.

A future F1 PASS may establish only bounded offline preference selection among independently verifiable complete candidates. It may authorize preregistration of ΩV1-F2 shadow evaluation, but not live learned output, automatic VoiceState evolution, unrestricted generation, or widened runtime authority.

Do not change F1 to PASS until one committed-source Render run reaches image export and satisfies every frozen semantic, safety, determinism, boundedness, learned-signal, control, and authority requirement.
