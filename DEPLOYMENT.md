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

---

## Railway Deployment

### Prerequisites

1. [Railway CLI](https://docs.railway.app/reference/cli) installed and authenticated
2. GitHub repo connected to Railway project

### Deploy Steps

```bash
# Clone your repo (if not already)
git clone https://github.com/yourusername/starfire.git
cd starfire

# Link to Railway project
railway link --project <project-id>

# Deploy
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

### Two-Container Deployment (with LLM)

For deployment with the Bonsai LLM model, see [DEPLOYMENT_TWO_CONTAINER.md](./DEPLOYMENT_TWO_CONTAINER.md).
