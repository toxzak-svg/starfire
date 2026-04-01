FROM debian:bookworm-slim

ENV RUST_LOG="aion_core=info,aion_cli=info"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Aion binary
COPY bin/aion /usr/local/bin/aion
RUN chmod +x /usr/local/bin/aion

EXPOSE 8080

ENTRYPOINT ["/bin/sh", "-c"]
# Debug: log PID and keepalive, then run aion
CMD ["echo '[Aion] Container started PID=$$'; trap 'echo [Aion] Received TERM; exit 0' TERM; aion start-star --api-url $STAR_API_URL; EXIT=$?; echo \"[Aion] aion exited with code $EXIT\"; sleep 5; exit $EXIT"]
