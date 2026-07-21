# ΩV1-B Typed VoiceState Shadow Gate

**Status:** Pending external execution

ΩV1-B implements a feature-gated, typed `VoiceState` without connecting it to live response generation.

## Feature boundary

Feature flag: `voice-state-shadow`

The module has no imports or wiring to `Runtime::chat()`, `VoiceEngine`, memory persistence, belief or ontology promotion, routing, tools, CHARGE discharge, or autonomous action.

State changes occur only through explicit `VoiceRevisionEvent` values and optimistic version checks. There is no automatic mutation.

## Typed layers

- immutable baseline identity ranges
- acquired tendencies
- relationship-scoped calibration
- session expression and intensity
- append-only revision history
- monotonic state version

The baseline covers bounded ranges for directness, warmth, severity, playfulness, philosophical depth, sentence compression, imagery density, initiative, disagreement style, uncertainty expression, and emotional explicitness.

## Gate

Render must run:

```bash
cargo test -p star --features voice-state-shadow --locked voice_state
cargo run -p star --example omega_v1b_voice_state_shadow \
  --features voice-state-shadow --locked
```

The probe must report:

- `gate_passed: true`
- `exact_state_match: true`
- `exact_json_match: true`
- `exact_digest_match: true`
- `version: 1`
- `session_intensity: 0.24`
- `no_runtime_influence: true`

The same ordered revision log must reproduce the exact state, canonical JSON, and deterministic digest.

Passing ΩV1-B authorizes only ΩV1-C semantic-response-plan completion in matched shadow mode. It does not authorize live voice-state influence.
