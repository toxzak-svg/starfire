# LLM Inference Server — standalone Candle + GGUF model
# Builds in ~10-15 min, produces ~1.5GB image
FROM rust:1.77-slim as builder

WORKDIR /build

# Install build deps
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first (for dependency caching)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source
COPY src/ src/

# Build with cleanup
RUN cargo build --release --bin llm-server 2>&1 | tail -5

# === runtime stage ===
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/llm-server /usr/local/bin/

# Copy model (add more COPY lines for additional models)
COPY models/ /models/

ENV HOST=0.0.0.0
ENV PORT=8081
ENV GGUF_PATH=/models/bonzai-8b/Bonsai-8B.gguf

EXPOSE 8081

ENTRYPOINT ["llm-server"]
