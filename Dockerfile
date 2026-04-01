# =============================================================================
# Star API — Railway Deployment
# Build context: repo root (toxzak-svg/star)
# Dockerfile location: life/Dockerfile
# Railway: set "Build Command Directory" to "life"
# =============================================================================

FROM rust:1.77-slim AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Build from the inner life/ Rust project
COPY Cargo.toml Cargo.lock ./
COPY life/life/src ./life/life/src/
COPY life/life/Cargo.toml life/life/Cargo.lock life/life/ 2>/dev/null || true

WORKDIR /build/life/life
RUN cargo build --release && mv target/release/star /build/star

# =============================================================================
# Runtime stage
# =============================================================================
FROM debian:bookworm-slim

ENV STAR_DATA_DIR="/data/star" \
    PORT="8080"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1000 -s /bin/bash nonroot

WORKDIR /app
COPY --from=builder /build/star /usr/local/bin/star
RUN chmod +x /usr/local/bin/star

# Seed data from build context
COPY life/life/star.db /app/star_init/star.db
COPY life/life/training.db /app/star_init/training.db
RUN mkdir -p /app/star_init && chown -R nonroot:nonroot /app/star_init

USER nonroot
EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=5s --start-period=8s --retries=5 \
    CMD curl -sf http://localhost:${PORT}/health || exit 1

ENTRYPOINT []
CMD ["sh", "-c", "\
    mkdir -p $STAR_DATA_DIR && \
    cp /app/star_init/star.db $STAR_DATA_DIR/ 2>/dev/null || true && \
    cp /app/star_init/training.db $STAR_DATA_DIR/ 2>/dev/null || true && \
    star api --data-dir $STAR_DATA_DIR --host 0.0.0.0 --port $PORT && wait"]
