# STLM L0 — Semantic Response Program Mechanics

Status: **PREREGISTERED — no result yet**

## Purpose

L0-B tests whether Starfire can represent a response decision as a typed,
version-bound, replayable semantic program before any surface renderer is
allowed to generate prose.

This experiment validates the contract and its failure behavior only. It does
not test conversational fluency, semantic understanding, renderer quality,
real-world user benefit, or learned cognition.

## Frozen hypothesis

A valid `SemanticResponseProgram` can be committed and replayed exactly from a
typed event while malformed, stale, scope-mismatched, disclosure-violating, or
digest-tampered programs are rejected without mutating the registry.

## Required positive behavior

The frozen fixture must establish:

1. a complete typed program containing required, optional, and prohibited
   claims;
2. ordered discourse operations covering `Assert`, `Qualify`, `Contrast`,
   `Correct`, `Explain`, `Acknowledge`, `RequestEvidence`, `Commit`, and
   `Abstain`;
3. exact binding to the current cognitive-state version, optional companion
   version, and subject scope;
4. deterministic canonical serialization;
5. byte-identical canonical bytes and digest across repeated validation;
6. exact replay into a fresh registry;
7. optimistic registry-version enforcement;
8. no `Runtime::chat()` integration or runtime authority.

## Frozen negative controls

The probe must reject all of the following atomically:

- zero or duplicate program, operation, claim, or referenced IDs;
- noncanonical claim or epistemic ordering;
- operation IDs that are duplicated, reordered, or noncontiguous;
- operation references to unknown or prohibited claims;
- required/prohibited overlap by ID or semantic key;
- duplicate semantic keys across authorized claims;
- confidence outside `0..=10_000` basis points;
- epistemic status inconsistent with claim confidence;
- a qualifier inconsistent with the claim's frozen epistemic status;
- zero or excessive output and compute budgets;
- stale cognitive-state or companion-state versions;
- subject-scope mismatch;
- sensitive claims disclosed above the authorized sensitivity level or outside
  the authorized disclosure scope;
- duplicate program commit;
- stale optimistic registry version;
- tampered canonical digest during replay;
- reordered or deleted replay events.

## Canonicalization contract

Canonical bytes are the UTF-8 `serde_json` serialization of the validated
program payload excluding its digest. Every vector whose order is not semantic
must already be strictly sorted by its typed ID. Discourse operation order is
semantic and therefore must use contiguous IDs beginning at one.

The digest is deterministic FNV-1a 64-bit over a domain separator followed by
canonical bytes. It is an accidental-tamper and replay-integrity checksum, not
cryptographic authentication.

## Frozen verdict

`PASS` requires every positive behavior and every negative control above to
succeed, exact repeated replay, zero partial mutation on rejected operations,
and all authority flags remaining false.

`FAIL` is mandatory if any malformed program commits, any stale or mismatched
program commits, any replay tampering is accepted, any rejected operation
partially mutates state, or any runtime authority is added.

`INFRASTRUCTURE_FAILURE` is reserved for failure to compile or execute the
frozen probe. It may not be converted into PASS.

## Authority boundary

L0-B may add only typed semantic-program contracts, validation, deterministic
digests, an in-memory replay registry, tests, a frozen probe, and scoped CI. It
adds no renderer, generated-text influence, persistence authority,
`Runtime::chat()` wiring, companion mutation, belief or ontology promotion,
tool selection, routing, CHARGE discharge, or autonomous action.

## Claim boundary

A PASS would establish only that the typed semantic boundary is mechanically
well-formed and replayable under the frozen synthetic fixture. It would not
establish fluent language, correct semantic planning, generalization, safety of
a learned renderer, absence of wrapper behavior, autonomous agency, or AGI.
