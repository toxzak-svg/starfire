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

ENTRYPOINT ["/usr/local/bin/aion"]
CMD ["start-star"]
