# ΩV1: Starfire Cognitive-to-Voice Bridge

**Status:** Active implementation program  
**Current stage:** ΩV1-B typed `VoiceState` shadow gate

## Central hypothesis

Star's voice remains static because the response layer receives finished prose or shallow style hints instead of a structured account of what Star concluded, how certain she is, what changed in her understanding, and how she relates to the conversation.

ΩV1 tests whether a bounded semantic response program, persistent voice state, and independent realization verification can make Star's language evolve detectably without allowing expression to invent beliefs, claims, memories, tools, or actions.

## Authority boundary

The renderer controls expression only. It does not control factual conclusions, confidence calculations, memory truth, belief or ontology promotion, routing, CHARGE discharge, tool selection, or autonomous action.

## Promotion ladder

1. ΩV1-A: frozen corpus, current outputs, metrics, and promotion criteria — **PASS**
2. ΩV1-B: typed persistent `VoiceState` in shadow mode — **current gate**
3. ΩV1-C: complete typed semantic-response-plan migration
4. ΩV1-D: bounded deterministic live bridge with neutral fallback
5. ΩV1-E: independent language verifier
6. ΩV1-F: optional learned expression renderer
7. ΩV1-G: replayable, earned voice evolution
8. ΩV1-H: validated companion-policy projection

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

ΩV1-A passed its externally executed Render gate on July 20, 2026.

ΩV1-B adds typed `VoiceState` with deterministic serialization, optimistic versioning, exact replay, explicit default state, bounded dimensions, debug projection, no automatic mutation, and no live response influence. Its feature flag is `voice-state-shadow`.

ΩV1-C remains blocked until the same ordered ΩV1-B event log reproduces the exact state, canonical serialization, and digest under external execution.
