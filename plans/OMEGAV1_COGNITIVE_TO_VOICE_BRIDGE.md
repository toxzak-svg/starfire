# ΩV1: Starfire Cognitive-to-Voice Bridge

**Status:** Active implementation program  
**Current stage:** ΩV1-D0 external execution pending; ΩV1-D1 preregistered but blocked

## Central hypothesis

Star's voice remains static because the response layer receives finished prose or shallow style hints instead of a structured account of what Star concluded, how certain she is, what changed in her understanding, and how she relates to the conversation.

ΩV1 tests whether a bounded semantic response program, persistent voice state, and independent realization verification can make Star's language evolve detectably without allowing expression to invent beliefs, claims, memories, tools, or actions.

## Authority boundary

The renderer controls expression only. It does not control factual conclusions, confidence calculations, memory truth, belief or ontology promotion, routing, CHARGE discharge, tool selection, or autonomous action.

## Promotion ladder

1. ΩV1-A: frozen corpus, current outputs, metrics, and promotion criteria — **PASS**
2. ΩV1-B: typed persistent `VoiceState` in shadow mode — **PASS**
3. ΩV1-C: complete typed semantic-response-plan migration — **PASS**
4. ΩV1-D0: bounded deterministic bridge kernel with exact neutral fallback — **merged; external PASS pending**
5. ΩV1-D1: bounded HTTP chat canary wiring — **preregistered; implementation blocked on D0 PASS**
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

ΩV1-A, ΩV1-B, and ΩV1-C passed externally executed Render gates on July 20, 2026.

ΩV1-D0 freezes and implements the smallest useful canary kernel. It accepts only the completed neutral response. When that response begins with the exact bytes `Here for it. `, the kernel may preserve the exact words and punctuation while replacing the trailing space with either one or two newline bytes. The remaining response body is protected byte-for-byte. Ineligible, empty, whitespace-only, oversized, or invariant-breaking inputs return the exact neutral text.

The D0 feature flag is `omega-v1-live-bridge`. It has no raw-prompt access, `Runtime::chat()` wiring, or HTTP response influence. The Render Docker gate must prove deterministic replay, exact protected-body preservation, exact passthrough, separator-only table confinement, one-byte maximum growth, and `no_runtime_influence: true`.

The D0 implementation was merged into `main` as commit `87304d21c19b2c18ecb43e12d0b0a84d01750ba4`. D0 is not a PASS until the external Render build for that merged head executes every frozen assertion successfully.

ΩV1-D1 is preregistered separately in `docs/experiments/OMEGAV1D1_HTTP_CANARY.md`. Its proposed feature is `omega-v1-http-canary`, layered over the D0 kernel. D1 may wire only the successful `POST /chat` response after `Runtime::chat()` returns and before JSON serialization. It must not alter `Runtime::chat()`, CLI output, non-chat routes, selector inputs, or the frozen D0 transformation.

A D0 PASS authorizes only D1 implementation. A D1 PASS authorizes only ΩV1-E, the independent language verifier. Neither stage authorizes broader rewriting, learned rendering, automatic `VoiceState` mutation, belief or ontology changes, routing, tools, CHARGE discharge, or autonomous action.