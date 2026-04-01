# =============================================================================
# Star API — Railway Deployment
# Build context: repo root (toxzak-svg/star)
# Dockerfile: life/Dockerfile
# Railway build root: life/
# =============================================================================

FROM rust:1.77-slim AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy from repo root → build/ (build context is repo root)
# life/ subdir paths work because we COPY the whole life/ subtree
COPY life/Cargo.toml life/Cargo.lock /build/life/
COPY life/life/src /build/life/life/src/
COPY life/life/Cargo.toml /build/life/life/
COPY life/life/Cargo.lock /build/life/life/ 2>/dev/null || true

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

# Seed DBs — create empty placeholders (DB created fresh on Railway if files don't exist)
RUN mkdir -p /app/star_init && touch /app/star_init/star.db /app/star_init/training.db

USER nonroot
EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=5s --start-period=8s --retries=5 \
    CMD curl -sf http://localhost:${PORT}/health || exit 1

ENTRYPOINT []
CMD ["sh", "-c", "\
    mkdir -p $STAR_DATA_DIR && \
    star api --data-dir $STAR_DATA_DIR --host 0.0.0.0 --port $PORT && wait"]
