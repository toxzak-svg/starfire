# =============================================================================
# Aion — Railway Deployment (connects to external Star API)
# =============================================================================

FROM debian:bookworm-slim

ENV STAR_API_URL="${STAR_API_URL}" \
    AION_DATA_DIR="/data/aion" \
    RUST_LOG="aion_core=info,aion_cli=info"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1000 -s /bin/bash nonroot

WORKDIR /app

# Aion binary from pre-built aion-deploy/bin/
COPY --chmod=755 ../aion-deploy/bin/aion /usr/local/bin/aion
RUN mkdir -p /app/aion_data && chown nonroot:nonroot /app/aion_data

USER nonroot

EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -sf http://localhost:8080/health || exit 1

# Aion connects to external Star API via STAR_API_URL
ENTRYPOINT []
CMD ["sh", "-c", "aion start-star --api-url $STAR_API_URL"]
