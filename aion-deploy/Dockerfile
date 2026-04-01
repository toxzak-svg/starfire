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

# Aion connects to Star API, polls Telegram, forwards messages to Star
# Required env vars: STAR_API_URL, TELEGRAM_BOT_TOKEN
ENTRYPOINT ["/bin/sh", "-c"]
# Use shell so $STAR_API_URL gets expanded at runtime
CMD ["aion start-star --api-url $STAR_API_URL"]
