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

# ΩV1-C Render gate. Every frozen fixture must produce a complete validated
# SemanticResponsePlan while the neutral compatibility renderer remains byte-exact.
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

# Build the exact executable Render runs. Do not pipe through tail: preserving
# Cargo's exit status makes failures visible in Render's build logs.
RUN cargo build --release --locked -p star_bin --bin star

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash starfire
WORKDIR /home/starfire

COPY --from=builder /build/target/release/star /usr/local/bin/star
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh \
    && mkdir -p /data \
    && chown -R starfire:starfire /data

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -f "http://localhost:${STARFIRE_PORT:-8080}/health" || exit 1

USER starfire
ENV STARFIRE_HOME=/data
ENV STARFIRE_DATA=/data

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
