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
