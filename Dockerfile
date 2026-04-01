# =============================================================================
# Star API — Railway Deployment
# Build context: repo root (toxzak-svg/star)
# Railway build root: life/ → Docker context = repo root, WORKDIR = life/
# So paths like "Cargo.toml" = repo-root/Cargo.toml (the outer project)
# And "life/src/..." = repo-root/life/src/... (source inside life/)
# =============================================================================

FROM rust:1.77-slim AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Cargo.toml for the workspace wrapper is at repo root
# Cargo.toml for the actual star project is at life/life/Cargo.toml
# Since WORKDIR=/build/life and we're copying from repo-root context:
COPY Cargo.toml Cargo.lock /build/life/                           # repo-root/Cargo.toml → /build/life/
COPY life/life/src /build/life/life/src/                          # repo-root/life/life/src → /build/life/life/src
COPY life/life/Cargo.toml /build/life/life/                       # repo-root/life/life/Cargo.toml → /build/life/life/
COPY life/life/Cargo.lock /build/life/life/   # same for lock

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

USER nonroot
EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=5s --start-period=8s --retries=5 \
    CMD curl -sf http://localhost:${PORT}/health

CMD ["/usr/local/bin/star", "api", "--data-dir", "/data/star", "--host", "0.0.0.0", "--port", "8080"]
