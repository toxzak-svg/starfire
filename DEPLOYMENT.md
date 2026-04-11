# Starfire Docker Deployment

## Overview

Builds a self-contained Docker image for Starfire AGI with:
- Starfire (Rust) — the AGI core
- Quanot (Rust) — reservoir computing subsystem
- Bonsai-8B LLM via Candle (in-process)
- Gateway API — HTTP/WebSocket interface

## Building

```bash
docker build -t starfire:latest .
```

## Running

```bash
# With docker-compose
docker compose up -d

# Standalone
docker run -p 8080:8080 starfire:latest
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `STARFIRE_PORT` | 8080 | HTTP API port |
| `STARFIRE_DATA` | /data | Persistent data directory |
| `STARFIRE_LOG` | info | Log level: trace, debug, info, warn, error |
| `LLM_ENDPOINT` | (in-process) | URL of llm-server (optional, for two-container) |

## Ports

- `8080` — HTTP API
- `8081` — LLM inference server (two-container mode)

## Volumes

- `/data` — persistent memory and state

## Health Check

```bash
curl http://localhost:8080/health
```

---

## Railway Deployment

### Prerequisites

1. [Railway CLI](https://docs.railway.app/reference/cli) installed and authenticated
2. GitHub repo connected to Railway project

### Deploy Steps

```bash
# Via GitHub (recommended)
# Push to GitHub → connect repo to Railway → deploy

# Or via CLI
railway up
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RAILWAY_PORT` | 8080 | Port Railway assigns (auto-set) |
| `STARFIRE_DATA` | /data | Persistent data directory |
| `STARFIRE_LOG` | info | Log level |

### Verify Deployment

```bash
# Health check
curl https://your-project-name.up.railway.app/health

# API endpoint
curl https://your-project-name.up.railway.app/
```

### Viewing Logs

```bash
railway logs
```

### Two-Container Deployment (with separate LLM server)

For deployment with Bonsai LLM as a separate container, see [DEPLOYMENT_TWO_CONTAINER.md](./DEPLOYMENT_TWO_CONTAINER.md).

### Persistent Volume

Star's memory lives at `/data`. On Railway, add a persistent volume:
1. Railway dashboard → Star service → Settings → Volumes
2. Add volume mounted at `/data`

This preserves memory across deployments.

---

## Docker Compose (Local Development)

```bash
# Build and run all services
docker compose up --build

# Run only starfire core (uses in-process LLM)
docker compose up starfire

# Run with external llm-server
docker compose up --build llm && docker compose up starfire
```

See [`docker-compose.yml`](docker-compose.yml) for full service configuration.
