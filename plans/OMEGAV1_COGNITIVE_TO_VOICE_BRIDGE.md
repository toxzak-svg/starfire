# ΩV1: Starfire Cognitive-to-Voice Bridge

**Status:** Active implementation program  
**Current stage:** ΩV1-D1 bounded HTTP chat canary external gate

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
5. ΩV1-D1: bounded HTTP chat canary wiring — **implemented in draft; external PASS pending**
6. ΩV1-E: independent language verifier
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

ΩV1-A, ΩV1-B, ΩV1-C, and ΩV1-D0 passed externally executed Render gates on July 20, 2026.

D0 remains the unchanged separator-only kernel. It receives only a completed neutral response, preserves the protected body byte-for-byte, and returns exact neutral text for every ineligible or invariant-breaking input. Its own authority declaration remains shadow-only when compiled without D1.

ΩV1-D1 adds a distinct `omega-v1-http-canary` feature layered over `omega-v1-live-bridge`. The successful HTTP `POST /chat` result is passed through a pure `finalize_chat_response(String) -> String` helper after `Runtime::chat()` completes and before JSON serialization. The helper signature cannot accept the prompt, request body, runtime, memory, state, route metadata, or conversation history.

The D1 Docker gate must prove deterministic replay, exact protected-body preservation, exact ineligible passthrough, unchanged single-field JSON shape, confinement to the frozen separator table, one-byte maximum growth, continued D0 shadow authority, and a D1 authority matrix in which only HTTP chat wiring and bounded returned-text influence are true.

The production binary enables D1 explicitly. CLI chat, Telegram, all non-chat HTTP routes, `Runtime::chat()`, cognition, reranking, `VoiceEngine`, memory, beliefs, ontology, routing, tools, CHARGE, persistence, companion state, and `VoiceState` remain unchanged.

A D1 PASS authorizes only ΩV1-E, the independent language verifier. It does not authorize broader rewriting, learned rendering, automatic `VoiceState` mutation, belief or ontology changes, routing, tools, CHARGE discharge, or autonomous action.
