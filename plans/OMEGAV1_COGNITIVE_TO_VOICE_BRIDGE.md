# ΩV1: Starfire Cognitive-to-Voice Bridge

**Status:** Active implementation program  
**Current stage:** ΩV1-E independent language verifier external gate

## Central hypothesis

Star's voice remains static because the response layer receives finished prose or shallow style hints instead of a structured account of what Star concluded, how certain she is, what changed in her understanding, and how she relates to the conversation.

ΩV1 tests whether a bounded semantic response program, persistent voice state, and independent realization verification can make Star's language evolve detectably without allowing expression to invent beliefs, claims, memories, tools, or actions.

## Authority boundary

The renderer controls expression only. It does not control factual conclusions, confidence calculations, memory truth, belief or ontology promotion, routing, CHARGE discharge, tool selection, or autonomous action.

## Promotion ladder

1. ΩV1-A: frozen corpus, current outputs, metrics, and promotion criteria — **PASS**
2. ΩV1-B: typed persistent `VoiceState` in shadow mode — **PASS**
3. ΩV1-C: complete typed semantic-response-plan migration — **PASS**
4. ΩV1-D0: bounded deterministic bridge kernel with exact neutral fallback — **PASS**
5. ΩV1-D1: bounded HTTP chat canary wiring — **PASS**
6. ΩV1-E: independent language verifier — **implemented in draft; external PASS pending**
7. ΩV1-F: optional learned expression renderer
8. ΩV1-G: replayable, earned voice evolution
9. ΩV1-H: validated companion-policy projection

No stage skips its predecessor.

## Core invariants

- Every durable voice revision is versioned, attributable, replayable, and reversible.
- Baseline identity, acquired tendencies, relationship calibration, and session expression remain separate.
- Voice changes cannot promote factual beliefs or ontology.
- Relationship-specific calibration cannot silently rewrite global identity.
- Renderer failure returns the exact neutral realization.
- Style variation cannot change claim set, polarity, confidence, commitments, abstentions, or prohibited implications.
- Companion policy remains subordinate to cognition and truth.

## Current implementation target

ΩV1-A through ΩV1-D1 passed externally executed Render gates on July 20, 2026. D1 remains the only live voice-path authority: the successful HTTP `POST /chat` response may pass through the unchanged separator-only D0 kernel. Prompt access, CLI, Telegram, non-chat routes, state mutation, and cognition-side authority remain closed.

ΩV1-E converges with the existing STLM L1 independent-language-verifier lineage from draft PR #72. The frozen preregistration commit is `dd992abf163c42cb8062cf75f61459009b57683a`; the identifiability addendum is `26ee75d135f57cac1ea12d2c477308472a1a1ba0`.

E preserves the frozen L0-C renderer and adds a separate verifier-ready grammar version 2 because the v1 surfaces for `Assert` and `Qualify` were not independently identifiable. The v2 forward surface exists only to create an invertible controlled evaluation target. It is not wired into the live D1 response.

The independent inverse verifier receives a validated semantic program, a validated lexical binding table, their digests and subject scope, a grammar version, and untrusted text. It does not receive renderer alignments and does not call the forward renderer. It reconstructs operation order and type, claims, polarity, epistemic status, typed references, abstention reason, spans, and independently recomputed costs. Ambiguous, malformed, semantically altered, over-budget, stale, or scope-mismatched text fails closed.

The Render builder reruns the frozen L0-C regression, focused verifier-ready and inverse-verifier tests, and the all-nine-operation negative-control probe. The production executable continues to build with only `omega-v1-http-canary`; `independent-language-verifier` is absent from the runtime feature set.

An ΩV1-E PASS authorizes only ΩV1-F evaluation of an optional learned expression renderer behind the independent verifier. It does not authorize live learned rendering, automatic `VoiceState` mutation, belief or ontology changes, routing, tools, CHARGE discharge, persistence, companion-state authority, or autonomous action.
