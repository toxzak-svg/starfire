# =============================================================================
# Star — Railway Deployment
# Build context: repo root (toxzak-svg/star)
# Railway build root: life/ → Docker context = repo root, WORKDIR = life/
# =============================================================================

FROM rust:1.77-slim AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock /build/
COPY src /build/src/
COPY lib /build/lib/

WORKDIR /build
RUN cargo build --release --manifest-path Cargo.toml && \
    mv target/release/star /build/star_bin && \
    mkdir -p /build/bin && \
    mv /build/star_bin /build/bin/star

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
COPY --from=builder /build/bin/star /usr/local/bin/star
RUN chmod +x /usr/local/bin/star

USER nonroot
EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=5s --start-period=8s --retries=5 \
    CMD curl -sf http://localhost:${PORT}/health

CMD ["/usr/local/bin/star", "api", "--data-dir", "/data/star", "--host", "0.0.0.0", "--port", "8080"]
