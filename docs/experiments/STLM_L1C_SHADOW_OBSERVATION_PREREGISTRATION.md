# STLM L1-C Verified Improvisation Shadow Observation Preregistration

## Status

Preregistered before the first complete L1-C implementation run.

## Parent evidence

L1-C is unlocked only by the passing STLM L1-B held-out result:

- 10 held-out conversational scenarios
- 160 of 160 independently verified selections
- 160 of 160 exact replay matches
- zero fallback
- zero legacy remediation leads
- complete response to recent-language and microstate treatments
- closed runtime and HTTP authority

L1-B did not authorize improvised text to be returned. It authorized this bounded shadow-observation proposal only.

## Hypothesis

The verified-improvisation selector can execute beside finalized `POST /chat` responses, produce independently verified comparison metadata, respond to bounded recent-language pressure, and preserve the returned response bytes exactly.

## Frozen implementation boundary

The L1-C observer may receive only:

- the existing ΩV1-F2 typed semantic bundle
- bounded lexical bindings
- typed intent and sensitivity labels
- a frozen response-byte fingerprint
- a deterministic entropy seed derived from sealed metadata
- an ephemeral fingerprint-only recent-language trace

The observer may:

- render the grammar-v2 neutral control
- execute the L1-A verified-improvisation selector
- independently verify the selected candidate
- compare fingerprints, lengths, dispositions, and digests
- append bounded metadata to a dedicated JSONL ledger

The observer may not:

- receive the raw prompt or unrestricted conversation
- receive raw live response text
- return or persist candidate text
- alter `Runtime::chat()` or HTTP response bytes
- mutate VoiceState, companion state, beliefs, ontology, routing, tools, CHARGE, or autonomous actions
- obtain general persistence authority

## Frozen probe cases

The first complete probe must demonstrate:

1. an eligible typed event produces comparison metadata
2. the selected candidate survives an independent verifier invocation
3. identical input, seed, microstate, and trace replay exactly
4. the candidate diverges from the grammar-v2 neutral control
5. a recent-language fingerprint treatment changes the selected surface
6. finalized response bytes remain identical
7. candidate and lexical text are absent from serialized ledger records
8. an ineligible event remains isolated
9. the full authority boundary remains closed

## Decision rule

`PASS` requires every frozen probe case to pass. Any timeout, panic, ledger failure, trace failure, or selection failure must be isolated from the live response path.

A pass permits only a separately preregistered L1-D longitudinal shadow-evidence phase. It does not authorize returning improvised text from `Runtime::chat()`, HTTP responses, Telegram, CLI, or any other live surface.
