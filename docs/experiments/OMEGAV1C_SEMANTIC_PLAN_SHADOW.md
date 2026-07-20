# ΩV1-C Semantic Response-Plan Shadow Migration

**Status:** Pending external Render execution

## Purpose

Complete the unfinished `ResponseIntent` migration without changing live language. Every frozen ΩV1-A fixture must produce a complete typed `SemanticResponsePlan` before the legacy response is returned by the neutral compatibility renderer.

## Typed plan requirements

Every plan carries:

- explicit intent
- ordered semantic operations
- grounded claims
- epistemic confidence attached before rendering
- response stance
- emotional position
- initiative level
- dialogue policy
- detail budget
- prohibited implications
- reference bindings
- claim provenance
- exact neutral compatibility text

Curiosity, revision, surprise, and acknowledgment are represented as typed operations rather than being inferred only from finished prose.

## Matched shadow controls

- Feature flag: `omega-v1-semantic-plan`
- The legacy expected output remains byte-exact through `neutral_compatibility_render`.
- Every plan is also converted into the existing STLM semantic authorization payload and validated independently.
- No `Runtime::chat()` or `VoiceEngine` wiring is added.
- No plan can mutate `VoiceState`, memory, beliefs, ontology, routing, tools, CHARGE, or autonomous actions.
- No old response path is deleted.

## Render gate

Render must establish all of the following before publishing the service image:

- `fixture_count: 122`
- `complete_plan_rate: 1.0`
- `neutral_compatibility_match_rate: 1.0`
- `semantic_program_validation_rate: 1.0`
- `missing_intent_count: 0`
- `missing_confidence_count: 0`
- `missing_claim_provenance_count: 0`
- nonzero typed curiosity, revision, surprise, and acknowledgment coverage
- `no_runtime_influence: true`
- `gate_passed: true`

A PASS authorizes only ΩV1-D implementation. It does not authorize live semantic-plan rendering.
