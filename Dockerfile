# =============================================================================
# Star — Railway Deployment
# =============================================================================

FROM rust:1.89-slim AS builder

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
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /usr/local/bin/star /entrypoint.sh

# No USER directive — entrypoint runs as root, which then execs as nonroot
EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=5s --start-period=8s --retries=5 \
    CMD curl -sf http://localhost:${PORT}/health

ENTRYPOINT ["/entrypoint.sh"]
