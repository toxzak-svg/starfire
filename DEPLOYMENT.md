# Starfire Docker Deployment

## Overview

Builds a self-contained Docker image for Starfire AGI with:
- Starfire (Rust) — the AGI core
- Quanot (Rust) — reservoir computing subsystem
- Gateway API — HTTP/WebSocket interface

## Building

```bash
docker build -t starfire:latest .
```

## Running

```bash
# With docker-compose
docker-compose up -d

# Standalone
docker run -p 8080:8080 starfire:latest
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| STARFIRE_PORT | 8080 | HTTP API port |
| STARFIRE_DATA | /data | Persistent data directory |
| STARFIRE_LOG | info | Log level: trace, debug, info, warn, error |
| STARFIRE_MEMORY | /data/memory | Memory store path |

## Ports

- `8080` — HTTP API
- `8081` — WebSocket (future)

## Volumes

- `/data` — persistent memory and state

## Health Check

```bash
curl http://localhost:8080/health
```
