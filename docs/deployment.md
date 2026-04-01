# Deployment Guide

Star deploys to Railway. This guide covers setup, environment variables, and troubleshooting.

---

## Prerequisites

- [Railway CLI](https://docs.railway.app/reference/cli) installed and authenticated
- GitHub repo forked from `toxzak-svg/star`

---

## One-Command Deploy

```bash
cd life
railway link --project <project-id>
railway up
```

Railway auto-detects the `Dockerfile` and starts the API server. No environment configuration required.

---

## Project ID

Find your project ID from the Railway dashboard or:

```bash
railway list --json | python3 -c "
import sys, json
d = json.load(sys.stdin)
for p in d['projects']:
    print(p['id'], p['name'])
"
```

The Star Claw project ID is: `a24765ef-9885-4da8-849c-5a525f4a22fb`

---

## What Gets Deployed

```
toxzak-svg/star
└── Dockerfile          → builds Rust binary
    └── CMD ["star"]    → Railway overrides, runs binary
                          (auto-starts API because RAILWAY_PUBLIC_DOMAIN is set)
```

Star auto-detects Railway via `RAILWAY_PUBLIC_DOMAIN` env var and starts the API server on port 8080 instead of the chat CLI.

---

## Environment Variables

Star works out of the box on Railway. No env vars required.

| Variable | Default | Description |
|----------|---------|-------------|
| `STAR_DATA_DIR` | `/data/star` | SQLite + memory files |
| `PORT` | `8080` | HTTP server port |
| `USE_LLM` | `false` | Ollama for text generation (not needed) |
| `OLLAMA_BASE_URL` | — | Ollama server URL |
| `USE_TELEGNOSTR` | `false` | Telegram bridge mode |
| `TELEGRAM_BOT_TOKEN` | — | Bot token (Aion only) |
| `STAR_API_URL` | — | Star's public URL (Aion only) |

---

## Adding a Persistent Volume

Star's memory lives at `/data/star`. On Railway, add a persistent volume:

1. Railway dashboard → Star service → Settings → Volumes
2. Add volume mounted at `/data/star`

This preserves memory across deployments.

---

## Railway Architecture

```
GitHub (toxzak-svg/star, layer4 branch)
    └── railway up
            └── Railway build
                    ├── rust:1.77-slim (builder)
                    │       └── cargo build --release
                    └── debian:bookworm-slim (runtime)
                            └── star binary
                                    └── star api (auto, RAILWAY_PUBLIC_DOMAIN set)
                                            └── port 8080
                                                    └── Railway proxy (HTTPS)
                                                            └── https://star-production-6458.up.railway.app
```

---

## Troubleshooting

### 502 Bad Gateway

Star's container is restarting. Check the build logs:

```bash
railway logs --service star
```

If the logs show Chat mode ("Type /quit to end the conversation"), the deployment is using an older build. Push the latest and redeploy:

```bash
cd life
git push origin layer4
railway up
```

### Health check fails

The Dockerfile includes a health check:

```dockerfile
HEALTHCHECK --interval=10s --timeout=5s --start-period=8s --retries=5 \
    CMD curl -sf http://localhost:${PORT}/health || exit 1
```

If it keeps failing, check that the API server is actually starting:

```bash
railway logs --service star | grep "API"
```

Should see: `Starting Star API server at http://0.0.0.0:8080`

### Star not responding to chat

The chat endpoint requires a POST with JSON:

```bash
curl https://star-production-6458.up.railway.app/chat \
  -X POST -H "Content-Type: application/json" \
  -d '{"message": "hello"}'
```

A GET request returns 405 Method Not Allowed.

### Memory not persisting

Star needs a persistent volume at `/data/star`. Without it, memory resets on every redeploy.

1. Railway dashboard → Star service → Volumes
2. Add volume named `star-data`
3. Mount at `/data/star`

---

## Branch Strategy

- **layer4** — the active development branch
- **main** — stable (merge from layer4 after testing)

Push to layer4 to trigger a deploy:

```bash
git push origin layer4
railway up
```

---

## Aion Deployment

Aion is a separate Railway service that polls Telegram and forwards messages to Star.

See `aion/RAILWAY_DEPLOY.md` for Aion-specific setup.

Environment variables for Aion:
- `STAR_API_URL=https://star-production-6458.up.railway.app`
- `TELEGRAM_BOT_TOKEN=<your bot token>`
