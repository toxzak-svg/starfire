# STLM L1 Independent Language Verifier

**Status:** frozen preregistration before implementation  
**Date:** 2026-07-13  
**Parent result:** `STLM_L0C_DETERMINISTIC_RENDERER_RESULT.md`

## 1. Purpose

L1 tests whether Starfire can independently reconstruct a normalized semantic
projection from rendered text and reject semantic drift without trusting the
renderer-provided alignment witness.

The verifier is offline, feature-gated, deterministic, and fail-closed. It has
no `Runtime::chat()` wiring and no authority over response routing, memory,
persistence, companion state, beliefs, ontology, tools, CHARGE, or actions.

## 2. Independence boundary

The verifier may receive:

- one validated `SemanticResponseProgram`;
- one validated, scope-bound `LexicalBindingTable`;
- untrusted rendered text;
- the source program and lexical-table digests needed to bind the check.

The verifier must not consume or trust:

- renderer-supplied `SemanticAlignment` entries;
- renderer-supplied claim or operation coverage counts;
- renderer-supplied semantic-marker judgments;
- renderer hidden state;
- raw conversation history;
- unrestricted memory;
- tools, persistence, routing, or action handles.

Renderer metadata may be retained only as opaque provenance. Acceptance must be
computed from the text, validated program, and validated lexical table.

## 3. L1 implementation slice

L1 implements a deterministic inverse verifier for the frozen L0-C reference
grammar. It reconstructs, from text alone:

- discourse-operation sequence and operation kind;
- claim references;
- observation, missing-variable, and prediction references;
- claim polarity;
- epistemic status;
- abstention reason;
- byte spans discovered by the verifier;
- character, sentence, paragraph, operation, claim, and verification-step cost.

The reconstructed projection is compared with the authorized semantic program.
The verifier does not prove open-ended natural-language understanding. It tests
an independent semantic judgment path for the current bounded grammar.

## 4. Frozen positive gates

The frozen probe must establish all of the following:

1. all nine discourse-operation forms are reconstructed from text;
2. reconstructed operation order exactly matches the semantic program;
3. every reconstructed claim and typed reference is authorized;
4. claim polarity exactly matches the authorized claim;
5. epistemic status exactly matches the authorized claim or operation;
6. abstention reason exactly matches the authorized operation;
7. required-operation coverage is complete;
8. no unsupported claim, reference, or trailing text is accepted;
9. independently recomputed budgets remain within the program limits;
10. repeated verification is byte-identical and digest-identical;
11. deleting, reordering, or forging renderer alignments does not change the
    verifier result because alignments are not an input to semantic judgment;
12. every verifier authority flag remains false.

A PASS requires every positive gate and every negative control to pass in one
committed-source workflow run.

## 5. Frozen negative controls

The probe must reject at least these text mutations:

- required-operation omission;
- duplicated operation;
- operation reorder;
- unsupported inserted sentence;
- trailing unparsed text;
- same-length polarity reversal;
- epistemic certainty inflation;
- epistemic certainty collapse;
- claim substitution;
- observation-reference substitution;
- missing-variable-reference substitution;
- prediction-reference substitution;
- abstention-reason substitution;
- prohibited or forbidden surface form;
- ambiguous lexical surface bindings;
- noncanonical separator or hidden unparsed gap;
- output-budget overflow;
- sentence-count, paragraph-count, operation-count, claim-count, or
  verification-step overflow;
- stale program digest;
- wrong lexical-table digest;
- wrong subject scope.

Any parse ambiguity is rejection, not best-effort selection.

## 6. Determinism and replay

The verifier emits a canonical `LanguageVerificationReport` containing:

- source program digest;
- lexical-table digest;
- normalized reconstructed operations;
- reconstructed spans;
- independently recomputed costs;
- terminal classification;
- deterministic report digest.

Two checks of the same inputs must produce byte-identical canonical reports.
The digest is an integrity and replay checksum, not cryptographic
authentication.

## 7. Authority boundary

The frozen authority matrix is:

```text
Runtime::chat() wiring:           false
live generated-text influence:    false
raw conversation access:          false
unrestricted memory access:       false
persistence authority:            false
routing authority:                false
companion mutation authority:     false
belief-promotion authority:       false
ontology-promotion authority:     false
tool-selection authority:         false
CHARGE-discharge authority:       false
autonomous-action authority:      false
```

## 8. Claim boundary

A PASS can establish only that the L1 verifier independently reconstructs and
checks the frozen deterministic grammar under the preregistered fixtures and
controls.

It cannot establish:

- open-ended semantic understanding;
- learned-verifier reliability;
- correctness of Starfire's semantic program selection;
- natural fluency;
- hallucination resistance for an unrestricted renderer;
- human preference or real-conversation benefit;
- non-wrapper attribution;
- autonomous agency or AGI.

## 9. Required committed-source validation

The dedicated workflow must run, with `pipefail` enabled:

- canonical `rustfmt --check` for touched Rust files;
- locked library and probe compilation;
- scoped Clippy with warnings denied;
- deterministic verifier unit contracts;
- the frozen L0-B semantic-program regression;
- the frozen L0-C deterministic-renderer regression;
- the frozen L1 verifier probe;
- artifact upload containing the complete JSON verdict.

Synthetic fixture success remains mechanics evidence only and cannot unlock
production response influence.