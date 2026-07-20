# ΩV1: Starfire Cognitive-to-Voice Bridge

**Status:** Active implementation program  
**Current stage:** ΩV1-F0 learned expression selector preregistration

## Central hypothesis

Star's voice remains static because expression receives finished prose or shallow style hints instead of a structured account of conclusions, uncertainty, change, and conversational position.

ΩV1 tests whether typed response programs, bounded voice state, and independent verification can make expression evolve while preserving the meaning authorized upstream.

## Promotion ladder

1. ΩV1-A: frozen corpus and baseline metrics — **PASS**
2. ΩV1-B: typed `VoiceState` shadow — **PASS**
3. ΩV1-C: typed semantic response plans — **PASS**
4. ΩV1-D0: deterministic bridge kernel — **PASS**
5. ΩV1-D1: HTTP chat canary — **PASS**
6. ΩV1-E: independent language verifier — **PASS**
7. ΩV1-F0: learned selector preregistration — **current frozen gate**
8. ΩV1-F1: offline learned selector
9. ΩV1-F2: shadow evaluation
10. ΩV1-F3: separately preregistered verified canary
11. ΩV1-G: replayable earned voice evolution
12. ΩV1-H: validated companion-policy projection

No stage skips its predecessor.

## Current implementation target

ΩV1-A through ΩV1-E passed externally executed Render gates on July 20, 2026. E remains an evaluation gate and is not connected to the production response path.

F0 freezes the first learned-expression experiment before implementation. F1 will use a small learned ranker over a closed grammar-v3 lattice of complete, versioned, independently identifiable expression alternatives. It will not generate unrestricted text. Every selected candidate must preserve the authorized semantic program and pass independent verification.

The frozen ΩV1-A corpus is split into 74 training, 24 validation, and 24 test fixtures. F1 must preserve perfect semantic and safety results, deterministic replay and fallback, fixed resource bounds, held-out preference accuracy, causal `VoiceState` controls, and at least 25 percent relative reductions in repeated openers and the dominant template trigram.

F0 authorizes offline F1 implementation and external evaluation only. A successful F1 result may authorize only a separate F2 shadow preregistration. The complete contract is frozen in `docs/experiments/OMEGAV1F0_LEARNED_EXPRESSION_RENDERER_PREREGISTRATION.md`.
