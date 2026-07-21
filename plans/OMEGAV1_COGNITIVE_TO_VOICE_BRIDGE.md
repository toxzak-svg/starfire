# ΩV1: Starfire Cognitive-to-Voice Bridge

**Status:** Active implementation program  
**Current stage:** ΩV1-F2 shadow implementation candidate

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
7. ΩV1-F0: learned selector preregistration — **PASS**
8. ΩV1-F1: original offline learned selector — **FAIL, preserved**
9. ΩV1-F1R1: bounded template-collapse remediation — **PASS**
10. ΩV1-F2: live shadow evaluation — **current implementation candidate**
11. ΩV1-F3: separately preregistered verified canary
12. ΩV1-G: replayable earned voice evolution
13. ΩV1-H: validated companion-policy projection

No stage skips its predecessor.

## Current implementation target

ΩV1-A through ΩV1-E passed externally executed Render gates on July 20, 2026. The original F1 run preserved semantics but failed its frozen anti-template signal and remains permanently classified as `FAIL`.

The separately preregistered F1R1 remediation expanded the bounded candidate lattice, retained nested independent verification, corrected the held-out comparison denominator, passed its external Render gate, and still granted no live learned-text authority.

F2 now tests whether that bounded selector can run beside successful live `POST /chat` requests after the D1 response bytes are frozen. The candidate is discarded and only bounded metadata may be recorded. The runtime switch defaults to disabled, and every shadow-system failure must leave the production response unchanged.

The frozen external verdict requires at least 200 eligible completed chat events across seven UTC days, at least 50 ineligible events, perfect response isolation and verifier floors, deterministic replay, fixed artifact and candidate bounds, p95 selector-and-verifier time no greater than 75 ms, a 250 ms hard maximum, and all forced-failure controls.

F2 implementation and collection do not authorize returning learned text. A successful F2 result may authorize only a separately preregistered F3 verified canary. The complete contract is frozen in `docs/experiments/OMEGAV1F2_SHADOW_EVALUATION_PREREGISTRATION.md`.
