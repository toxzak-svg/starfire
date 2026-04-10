# Starfire Core — Docker Image (no LLM, no candle)
# Multi-stage build for minimal image size
#
# Usage:
#   docker build -t starfire .         # with Docker
#   podman build -t starfire .         # with Podman
#   docker run --rm starfire star api --port 8080
#   podman run --rm starfire star api --port 8080

# ============================================
# Stage 1: Build
# ============================================
FROM debian:bookworm-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build

# Copy manifests first (better layer caching)
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY lib/ ./lib/

# Build with NO LLM feature (candle-free)
# --locked ensures Cargo.lock is respected
RUN cargo build --release --bin star --no-default-features --locked 2>&1

# ============================================
# Stage 2: Runtime
# ============================================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash starfire
WORKDIR /home/starfire

# Copy binary
COPY --from=builder /build/target/release/star /usr/local/bin/
RUN test -f /build/target/release/libstar.so && cp /build/target/release/libstar.so /usr/local/lib/libstar.so || true

# Copy entrypoint
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Data volume
RUN mkdir -p /data && chown -R starfire:starfire /data

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

USER starfire
ENV STARFIRE_HOME=/data
ENV RAILWAY_PORT=8080

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
