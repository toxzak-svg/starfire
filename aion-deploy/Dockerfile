FROM debian:bookworm-slim

ENV STAR_API_URL="http://localhost:8080" \
    RUST_LOG="aion_core=info,aion_cli=info"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Binaries
COPY bin/ /usr/local/bin/
RUN chmod +x /usr/local/bin/aion /usr/local/bin/star

# Seed data baked into image
COPY star_data/ /app/star_data_init/

EXPOSE 8080

# Run everything as root — Railway volume mounts are root-owned anyway
ENTRYPOINT ["/bin/sh", "-c"]
CMD ["mkdir -p /app/star_data && cp -r /app/star_data_init/* /app/star_data/ 2>/dev/null || true && star --data-dir /app/star_data api --host 0.0.0.0 --port 8080 & echo 'Waiting for Star API...' && for i in $(seq 1 30); do curl -sf http://localhost:8080/health && break || true; sleep 1; done && echo 'Star API ready. Starting Aion/StarMind...' && aion start-star --api-url http://localhost:8080 && wait"]
