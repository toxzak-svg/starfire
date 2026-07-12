# Starfire AGI — Docker Image
# Multi-stage build for minimal image size

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
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build

# Copy source
COPY Cargo.toml Cargo.lock ./
COPY lib/ ./lib/
COPY src/ ./src/

# Create dummy source for dependency caching
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/dummy.rs && \
    echo "fn main() {}" > lib/dummy.rs

# Build dependencies (cached)
RUN cargo build --release --lib 2>&1 | tail -20

# Build actual binary
RUN rm -rf src/bin/dummy.rs lib/dummy.rs
RUN cargo build --release --bin star 2>&1 | tail -20

# ============================================
# Stage 2: Runtime
# ============================================
FROM debian:bookworm-slim

# Install runtime dependencies and the health-check client
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash starfire
WORKDIR /home/starfire

# Copy binary from builder
COPY --from=builder /build/target/release/star /usr/local/bin/star
RUN test -f /build/target/release/libstar.so && cp /build/target/release/libstar.so /usr/local/lib/libstar.so || true

# Copy config
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Create data directory
RUN mkdir -p /data && chown -R starfire:starfire /data

# Expose API port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:${STARFIRE_PORT:-8080}/health || exit 1

# Run as non-root
USER starfire
ENV STARFIRE_HOME=/data
ENV STARFIRE_DATA=/data

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
