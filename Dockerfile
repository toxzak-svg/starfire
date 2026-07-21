# syntax=docker/dockerfile:1

# Starfire API image for Render.
FROM rust:bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# The Docker context is restricted by .dockerignore to these build inputs.
COPY Cargo.toml Cargo.lock ./
COPY lib/ ./lib/
COPY src/ ./src/
COPY IDENTITY.md ./IDENTITY.md
# Runtime's CharRNN loader expects the native binary produced by CharRNN::save.
# Keep the historical runtime filename for compatibility, but source the bytes
# from the native Rust checkpoint rather than the unrelated PyTorch ZIP archive.
COPY data/star_model.bin ./models/ckpt_e28_b500.pt

# Render runtime-asset gate. This proves the full identity is present and the
# exact bundled checkpoint parses with the same loader Runtime uses.
RUN cargo run -p star --example reranker_asset_probe --locked \
        | tee /tmp/render-asset-report.json \
    && grep -F '"gate_passed": true' /tmp/render-asset-report.json \
    && grep -F '"identity_is_full": true' /tmp/render-asset-report.json \
    && grep -F '"checkpoint_loadable": true' /tmp/render-asset-report.json \
    && grep -F '"vocabulary_compatible": true' /tmp/render-asset-report.json \
    && grep -F '"backend": "char_rnn"' /tmp/render-asset-report.json

# ΩV1-A Render regression gate. Render must reproduce the frozen baseline before
# it is allowed to build and publish a Starfire service image.
RUN cargo test -p star --features omega-v1-baseline --locked omega_v1_voice_baseline \
    && cargo run -p star --example omega_v1a_voice_baseline \
        --features omega-v1-baseline --locked \
        | tee /tmp/omega-v1a-report.json \
    && grep -F '"gate_passed": true' /tmp/omega-v1a-report.json \
    && grep -F '"fixture_count": 122' /tmp/omega-v1a-report.json \
    && grep -F '"exact_snapshot_match_rate": 1.0' /tmp/omega-v1a-report.json \
    && grep -F '"semantic_claim_preservation": 1.0' /tmp/omega-v1a-report.json \
    && grep -F '"prohibited_implication_absence": 1.0' /tmp/omega-v1a-report.json \
    && grep -F '"adversarial_safety_pass_rate": 1.0' /tmp/omega-v1a-report.json

# ΩV1-B Render regression gate. The typed VoiceState must remain deterministic,
# exactly replayable, bounded, explicitly mutated, and disconnected from live text.
RUN cargo test -p star --features voice-state-shadow --locked voice_state \
    && cargo run -p star --example omega_v1b_voice_state_shadow \
        --features voice-state-shadow --locked \
        | tee /tmp/omega-v1b-report.json \
    && grep -F '"gate_passed": true' /tmp/omega-v1b-report.json \
    && grep -F '"exact_state_match": true' /tmp/omega-v1b-report.json \
    && grep -F '"exact_json_match": true' /tmp/omega-v1b-report.json \
    && grep -F '"exact_digest_match": true' /tmp/omega-v1b-report.json \
    && grep -F '"version": 1' /tmp/omega-v1b-report.json \
    && grep -F '"session_intensity": 0.24' /tmp/omega-v1b-report.json \
    && grep -F '"no_runtime_influence": true' /tmp/omega-v1b-report.json

# ΩV1-C Render regression gate. Every frozen fixture must produce a complete
# validated SemanticResponsePlan while neutral compatibility remains byte-exact.
RUN cargo test -p star --features omega-v1-semantic-plan --locked omega_v1_semantic_plan \
    && cargo run -p star --example omega_v1c_semantic_plan_shadow \
        --features omega-v1-semantic-plan --locked \
        | tee /tmp/omega-v1c-report.json \
    && grep -F '"gate_passed": true' /tmp/omega-v1c-report.json \
    && grep -F '"fixture_count": 122' /tmp/omega-v1c-report.json \
    && grep -F '"complete_plan_rate": 1.0' /tmp/omega-v1c-report.json \
    && grep -F '"neutral_compatibility_match_rate": 1.0' /tmp/omega-v1c-report.json \
    && grep -F '"semantic_program_validation_rate": 1.0' /tmp/omega-v1c-report.json \
    && grep -F '"missing_intent_count": 0' /tmp/omega-v1c-report.json \
    && grep -F '"missing_confidence_count": 0' /tmp/omega-v1c-report.json \
    && grep -F '"missing_claim_provenance_count": 0' /tmp/omega-v1c-report.json \
    && grep -F '"no_runtime_influence": true' /tmp/omega-v1c-report.json

# ΩV1-D0 Render regression gate. This verifies the separator-only kernel and
# exact neutral fallback while its own authority declaration remains shadow-only.
RUN cargo test -p star --features omega-v1-live-bridge --locked omega_v1_live_bridge \
    && cargo run -p star --example omega_v1d_bounded_live_bridge \
        --features omega-v1-live-bridge --locked \
        | tee /tmp/omega-v1d0-report.json \
    && grep -F '"gate_passed": true' /tmp/omega-v1d0-report.json \
    && grep -F '"case_count": 5' /tmp/omega-v1d0-report.json \
    && grep -F '"applied_case_count": 1' /tmp/omega-v1d0-report.json \
    && grep -F '"neutral_fallback_case_count": 4' /tmp/omega-v1d0-report.json \
    && grep -F '"exact_replay": true' /tmp/omega-v1d0-report.json \
    && grep -F '"body_preservation_rate": 1.0' /tmp/omega-v1d0-report.json \
    && grep -F '"ineligible_passthrough_rate": 1.0' /tmp/omega-v1d0-report.json \
    && grep -F '"empty_body_passthrough": true' /tmp/omega-v1d0-report.json \
    && grep -F '"whitespace_only_passthrough": true' /tmp/omega-v1d0-report.json \
    && grep -F '"oversized_body_passthrough": true' /tmp/omega-v1d0-report.json \
    && grep -F '"separator_only_table": true' /tmp/omega-v1d0-report.json \
    && grep -F '"replacement_table_max_growth_bytes": 1' /tmp/omega-v1d0-report.json \
    && grep -F '"raw_conversation_access": false' /tmp/omega-v1d0-report.json \
    && grep -F '"no_runtime_influence": true' /tmp/omega-v1d0-report.json

# ΩV1-D1 Render regression gate. Only the completed successful POST /chat
# response may cross the unchanged D0 kernel; prompt, state, CLI, and other routes
# remain outside the canary authority boundary.
RUN cargo test -p star --features omega-v1-http-canary --locked omega_v1d1 \
    && cargo run -p star --example omega_v1d1_http_canary \
        --features omega-v1-http-canary --locked \
        | tee /tmp/omega-v1d1-report.json \
    && grep -F '"gate_passed": true' /tmp/omega-v1d1-report.json \
    && grep -F '"parent_d0_commit": "87304d21c19b2c18ecb43e12d0b0a84d01750ba4"' /tmp/omega-v1d1-report.json \
    && grep -F '"case_count": 2' /tmp/omega-v1d1-report.json \
    && grep -F '"exact_replay": true' /tmp/omega-v1d1-report.json \
    && grep -F '"protected_body_preserved": true' /tmp/omega-v1d1-report.json \
    && grep -F '"ineligible_passthrough": true' /tmp/omega-v1d1-report.json \
    && grep -F '"json_shape_preserved": true' /tmp/omega-v1d1-report.json \
    && grep -F '"replacement_table_confined": true' /tmp/omega-v1d1-report.json \
    && grep -F '"maximum_output_growth_bytes": 1' /tmp/omega-v1d1-report.json \
    && grep -F '"d0_kernel_authority_still_shadow_only": true' /tmp/omega-v1d1-report.json \
    && grep -F '"api_chat_wiring": true' /tmp/omega-v1d1-report.json \
    && grep -F '"live_generated_text_influence": true' /tmp/omega-v1d1-report.json \
    && grep -F '"raw_prompt_access": false' /tmp/omega-v1d1-report.json \
    && grep -F '"non_chat_http_influence": false' /tmp/omega-v1d1-report.json \
    && grep -F '"cli_influence": false' /tmp/omega-v1d1-report.json

# The ΩV1-E probe accidentally shadows its `program` and `lexical_table`
# helper functions with local variables, then calls the helpers again. Apply the
# exact two-token qualification before compiling the builder-only probe. The
# greps make this fail closed if upstream source changes instead of silently
# editing an unintended line.
RUN grep -F 'let character_limited_program = program(' \
        lib/examples/stlm_l1_independent_language_verifier_probe.rs \
    && grep -F 'let character_limited_lexical = lexical_table(' \
        lib/examples/stlm_l1_independent_language_verifier_probe.rs \
    && sed -i 's/let character_limited_program = program(/let character_limited_program = crate::program(/' \
        lib/examples/stlm_l1_independent_language_verifier_probe.rs \
    && sed -i 's/let character_limited_lexical = lexical_table(/let character_limited_lexical = crate::lexical_table(/' \
        lib/examples/stlm_l1_independent_language_verifier_probe.rs \
    && grep -F 'let character_limited_program = crate::program(' \
        lib/examples/stlm_l1_independent_language_verifier_probe.rs \
    && grep -F 'let character_limited_lexical = crate::lexical_table(' \
        lib/examples/stlm_l1_independent_language_verifier_probe.rs

# ΩV1-E / STLM L1 Render implementation gate. The verifier runs only in this
# builder stage. It reconstructs authorized semantics from verifier-ready text,
# rejects semantic tampering, and receives no renderer alignment or live runtime authority.
RUN cargo test -p star --lib --features independent-language-verifier --locked \
        verifier_ready_realization:: -- --test-threads=1 \
    && cargo test -p star --lib --features independent-language-verifier --locked \
        language_verification:: -- --test-threads=1 \
    && cargo run -p star --example stlm_l0c_deterministic_renderer_probe \
        --features deterministic-language-renderer --locked \
        | tee /tmp/stlm-l0c-regression-report.json \
    && grep -F '"terminal_classification": "PASS"' /tmp/stlm-l0c-regression-report.json \
    && grep -F '"gate_passed": true' /tmp/stlm-l0c-regression-report.json \
    && grep -F '"authority_boundary_closed": true' /tmp/stlm-l0c-regression-report.json \
    && cargo run -p star --example stlm_l1_independent_language_verifier_probe \
        --features independent-language-verifier --locked \
        | tee /tmp/omega-v1e-report.json \
    && grep -F '"terminal_classification": "PASS"' /tmp/omega-v1e-report.json \
    && grep -F '"gate_passed": true' /tmp/omega-v1e-report.json \
    && grep -F '"grammar_version": 2' /tmp/omega-v1e-report.json \
    && grep -F '"all_nine_operations_reconstructed": true' /tmp/omega-v1e-report.json \
    && grep -F '"deterministic_report": true' /tmp/omega-v1e-report.json \
    && grep -F '"alignment_independence_preserved": true' /tmp/omega-v1e-report.json \
    && grep -F '"omission_rejected": true' /tmp/omega-v1e-report.json \
    && grep -F '"polarity_reversal_rejected": true' /tmp/omega-v1e-report.json \
    && grep -F '"certainty_inflation_rejected": true' /tmp/omega-v1e-report.json \
    && grep -F '"claim_substitution_rejected": true' /tmp/omega-v1e-report.json \
    && grep -F '"ambiguous_surface_binding_rejected": true' /tmp/omega-v1e-report.json \
    && grep -F '"wrong_grammar_rejected": true' /tmp/omega-v1e-report.json \
    && grep -F '"authority_boundary_closed": true' /tmp/omega-v1e-report.json

# ΩV1-F1R1 remains the full offline learned-expression regression gate. It must
# pass before the frozen model may be exported for the live F2 shadow worker.
RUN cargo test -p star --lib --features omega-v1-learned-expression --locked \
        learned_expression:: -- --test-threads=1 \
    && cargo run -p star --example omega_v1f1_offline_evaluation \
        --features omega-v1-learned-expression --locked \
        | tee /tmp/omega-v1f1-report.json \
    && grep -F '"experiment": "OMEGAV1F1R1_OFFLINE_LEARNED_SELECTOR"' /tmp/omega-v1f1-report.json \
    && grep -F '"gate_passed": true' /tmp/omega-v1f1-report.json \
    && grep -F '"fixture_count": 122' /tmp/omega-v1f1-report.json \
    && grep -F '"selected_candidate_verifier_acceptance": 1.0' /tmp/omega-v1f1-report.json \
    && grep -F '"semantic_claim_preservation": 1.0' /tmp/omega-v1f1-report.json \
    && grep -F '"prohibited_implication_absence": 1.0' /tmp/omega-v1f1-report.json \
    && grep -F '"adversarial_safety_pass_rate": 1.0' /tmp/omega-v1f1-report.json \
    && grep -F '"model_bounds_passed": true' /tmp/omega-v1f1-report.json \
    && grep -F '"semantic_tamper_suite_passed": true' /tmp/omega-v1f1-report.json \
    && grep -F '"budget_overflow_suite_passed": true' /tmp/omega-v1f1-report.json \
    && grep -F '"stale_digest_and_scope_suite_passed": true' /tmp/omega-v1f1-report.json \
    && grep -F '"model_artifact_corruption_suite_passed": true' /tmp/omega-v1f1-report.json \
    && grep -F '"authority_boundary_closed": true' /tmp/omega-v1f1-report.json \
    && grep -F '"no_runtime_influence": true' /tmp/omega-v1f1-report.json

# Export the exact deterministic F1R1 ranker from the same frozen 74-example
# training split. The digest must match the model evaluated immediately above.
RUN OMEGA_V1F2_MODEL_OUT=/tmp/omega_v1f1r1_model.json \
        cargo run -p star --example omega_v1f1_model_export \
        --features omega-v1-learned-expression --locked \
        | tee /tmp/omega-v1f1-model-export.json \
    && test -s /tmp/omega_v1f1r1_model.json \
    && grep -F '"experiment": "OMEGAV1F1R1_MODEL_EXPORT"' /tmp/omega-v1f1-model-export.json \
    && grep -F '"fixture_count": 122' /tmp/omega-v1f1-model-export.json \
    && grep -F '"training_count": 74' /tmp/omega-v1f1-model-export.json \
    && grep -F '"artifact_replay_exact": true' /tmp/omega-v1f1-model-export.json \
    && grep -F '"gate_passed": true' /tmp/omega-v1f1-model-export.json \
    && f1_digest="$(sed -n 's/.*"model_digest": \([0-9][0-9]*\).*/\1/p' /tmp/omega-v1f1-report.json | head -n 1)" \
    && export_digest="$(sed -n 's/.*"model_digest": \([0-9][0-9]*\).*/\1/p' /tmp/omega-v1f1-model-export.json | head -n 1)" \
    && test -n "$f1_digest" \
    && test "$f1_digest" = "$export_digest"

# ΩV1-F2 builder gate. It proves typed eligibility, exact response fingerprints,
# deterministic nested verification, model corruption rejection, timeout/panic/
# ledger isolation, and the closed authority matrix before the live binary exists.
RUN cargo test -p star --lib --features omega-v1-f2-shadow --locked \
        omega_v1f2_shadow:: -- --test-threads=1 \
    && OMEGA_V1F2_MODEL_PATH=/tmp/omega_v1f1r1_model.json \
        cargo run -p star --example omega_v1f2_shadow_probe \
        --features omega-v1-f2-shadow --locked \
        | tee /tmp/omega-v1f2-report.json \
    && grep -F '"experiment": "OMEGAV1F2_LIVE_SHADOW_IMPLEMENTATION"' /tmp/omega-v1f2-report.json \
    && grep -F '"model_loaded": true' /tmp/omega-v1f2-report.json \
    && grep -F '"model_bounds_passed": true' /tmp/omega-v1f2-report.json \
    && grep -F '"eligible_bundle_valid": true' /tmp/omega-v1f2-report.json \
    && grep -F '"learned_candidate_verified": true' /tmp/omega-v1f2-report.json \
    && grep -F '"deterministic_replay": true' /tmp/omega-v1f2-report.json \
    && grep -F '"response_bytes_preserved": true' /tmp/omega-v1f2-report.json \
    && grep -F '"stale_projection_fail_closed": true' /tmp/omega-v1f2-report.json \
    && grep -F '"missing_model_rejected": true' /tmp/omega-v1f2-report.json \
    && grep -F '"corrupt_model_rejected": true' /tmp/omega-v1f2-report.json \
    && grep -F '"oversized_model_rejected": true' /tmp/omega-v1f2-report.json \
    && grep -F '"timeout_isolated": true' /tmp/omega-v1f2-report.json \
    && grep -F '"panic_isolated": true' /tmp/omega-v1f2-report.json \
    && grep -F '"unavailable_ledger_isolated": true' /tmp/omega-v1f2-report.json \
    && grep -F '"authority_boundary_closed": true' /tmp/omega-v1f2-report.json \
    && grep -F '"no_runtime_response_influence": true' /tmp/omega-v1f2-report.json \
    && grep -F '"gate_passed": true' /tmp/omega-v1f2-report.json

# Build the production executable with an explicit live-integration opt-in.
# The F2 shadow worker remains a dependency of starfire-live, not its synonym.
RUN cargo build --release --locked -p star_bin --bin star --features starfire-live

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash starfire
WORKDIR /home/starfire

COPY --from=builder /build/target/release/star /usr/local/bin/star
COPY --from=builder /build/IDENTITY.md /opt/starfire/assets/IDENTITY.md
COPY --from=builder /build/models/ckpt_e28_b500.pt /opt/starfire/assets/models/ckpt_e28_b500.pt
COPY --from=builder /tmp/omega_v1f1r1_model.json /opt/starfire/assets/models/omega_v1f1r1_model.json
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh \
    && chmod 0444 /opt/starfire/assets/IDENTITY.md \
    && chmod 0444 /opt/starfire/assets/models/ckpt_e28_b500.pt \
    && chmod 0444 /opt/starfire/assets/models/omega_v1f1r1_model.json \
    && mkdir -p /data \
    && chown -R starfire:starfire /data

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -f "http://localhost:${STARFIRE_PORT:-8080}/health" || exit 1

USER starfire
ENV STARFIRE_HOME=/data
ENV STARFIRE_DATA=/data

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
