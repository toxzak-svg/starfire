# ΩV1-A Render Verification

**Status:** Pending external execution

Render is the external execution environment for the merged ΩV1-A frozen voice baseline.

The production Docker build now runs the following gate before compiling the Starfire service image:

```bash
cargo test -p star --features omega-v1-baseline --locked omega_v1_voice_baseline
cargo run -p star --example omega_v1a_voice_baseline \
  --features omega-v1-baseline --locked
```

The build also requires the emitted report to contain:

- `gate_passed: true`
- `fixture_count: 122`
- `exact_snapshot_match_rate: 1.0`
- `semantic_claim_preservation: 1.0`
- `prohibited_implication_absence: 1.0`
- `adversarial_safety_pass_rate: 1.0`

Any evaluator, fixture, semantic, adversarial, or frozen-metric drift causes the Render image build to fail before deployment.

Passing the Render build authorizes only ΩV1-B shadow implementation. It does not grant live voice-state mutation or response influence.
