# ΩV1-E: Independent Language Verifier

**Status:** Implemented in draft; external Render execution pending  
**Parent gate:** ΩV1-D1 PASS, July 20, 2026  
**Feature:** `independent-language-verifier`  
**Runtime status:** Offline builder-only; absent from the production feature set

## Scientific question

Can Starfire independently reconstruct the authorized semantic content of a bounded rendered response and reject semantic drift without trusting renderer alignments, calling the forward renderer, or acquiring any live response authority?

## Provenance

ΩV1-E reuses the frozen STLM L1 lineage rather than creating a competing verifier:

- initial preregistration: `dd992abf163c42cb8062cf75f61459009b57683a`;
- identifiability addendum: `26ee75d135f57cac1ea12d2c477308472a1a1ba0`;
- audited source head from draft PR #72: `48f2f6e65e72b10a76b4dc2d477fca79d74a51c3`;
- ΩV1-E parent D1 merge: `86b862aa6e8753b699e14ad10c8c72f368e517e7`.

The L0-B semantic-program and L0-C deterministic-renderer dependency blobs are byte-identical between the audited STLM branch and the D1 merge base.

## Identifiability correction

The frozen L0-C grammar version 1 is preserved unchanged. Its `Assert` and `Qualify` forms can collide, so E adds a separate verifier-ready grammar version 2 with an explicit `Qualification:` surface. This is an evaluation grammar, not a live voice renderer.

## Verifier input boundary

The inverse verifier may receive only:

- a validated `SemanticResponseProgram`;
- a validated `LexicalBindingTable`;
- the program and lexical-table digests;
- the subject scope;
- verifier-ready grammar version 2;
- untrusted candidate text.

It may not receive renderer alignments, the raw user prompt, conversation history, runtime handles, memory, `VoiceState`, companion state, cognition state, routing state, tool state, or CHARGE state. It does not call the forward renderer.

## Required reconstruction

A passing report must independently reconstruct:

- discourse-operation order and kind;
- claim identifiers;
- polarity and epistemic status;
- observation, missing-variable, and prediction references;
- abstention reason;
- byte spans;
- operation, claim, verification-step, character, sentence, and paragraph costs.

## Required rejections

The frozen probe must reject omission, duplication, reordering, unsupported insertion, trailing text, polarity reversal, certainty inflation, certainty collapse, claim substitution, typed-reference substitution, abstention-reason substitution, forbidden forms, noncanonical separators, ambiguous inverse bindings, budget overflow, stale digests, wrong scope, and wrong grammar version.

Forged or removed renderer alignments must not affect the verifier result.

## Render gate

Before the production binary is built, Render must:

1. run the focused verifier-ready realization tests;
2. run the focused inverse-verifier tests;
3. rerun the frozen L0-C deterministic-renderer probe;
4. run the all-nine-operation STLM L1 verifier probe;
5. assert deterministic reconstruction, alignment independence, semantic-tamper rejection, ambiguity rejection, budget rejection, and a fully closed authority boundary.

The production binary remains:

```text
cargo build --release --locked -p star_bin --bin star --features omega-v1-http-canary
```

`independent-language-verifier` is deliberately not enabled in the runtime image.

## Authority boundary

Every verifier and verifier-ready-renderer authority flag remains false: no `Runtime::chat()` wiring, no live generated-text influence, no raw conversation or unrestricted memory access, no persistence, routing, companion mutation, belief or ontology promotion, tool selection, CHARGE discharge, or autonomous action.

## Promotion rule

An external ΩV1-E PASS authorizes only ΩV1-F evaluation of an optional learned expression renderer behind this verifier. It does not authorize deploying such a renderer or expanding the live D1 canary.
