# Two-Container Deployment on Railway

Starfire ships as two separate containers that communicate over HTTP:

```
┌──────────────────────────────────────────────────────────────┐
│  Railway Project                                             │
│                                                              │
│  ┌────────────────────┐         ┌────────────────────────┐ │
│  │  starfire-core      │ HTTP   │  llm-inference          │ │
│  │  star bin           │◄──────►│  Bonsai-8B via Candle   │ │
│  │  Port 8080          │:8081   │  Port 8081               │ │
│  └────────────────────┘         └────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

## Files Changed/Created

### New files
- `llm-server/` — standalone HTTP inference server crate
  - `Cargo.toml` — uses `candle-core = "0.6"` + `candle-transformers = "0.6"` from crates.io
  - `src/main.rs` — tiny_http server, OpenAI-compatible API (`/v1/chat/completions`, `/v1/completions`, `/health`)
  - `Dockerfile` — builds image with model baked in
- `lib/llm/http_llm.rs` — Rust client that calls llm-server from star-core over HTTP
- `docker-compose.yml` — local dev with both containers linked

### Modified files
- `lib/lib.rs` — added `pub mod http_llm;` (always available, no candle dep)
- `Dockerfile` — simplified, removed candle build steps

---

## Setup Steps

### 1. Copy Bonsai GGUF to the instance

```bash
# On local machine — find the model
ls ~/.openclaw/workspace/projects/starfire/models/bonsai-8b/

# Upload to instance (adjust user/host)
scp -r models/bonsai-8b/ user@your-instance:/home/user/starfire/models/
```

Or download on the instance:
```bash
mkdir -p models/bonsai-8b
# ... (use whatever download method the instance supports)
```

### 2. Set up Git on the instance

```bash
git clone https://github.com/yourusername/starfire.git
cd starfire
git pull  # get latest code with llm-server
```

### 3. Build both Docker images on the instance

```bash
# Build star-core (no candle, fast ~3 min)
docker build -t starfire:latest -f Dockerfile .

# Build llm-inference (with candle, ~10-15 min first time)
docker build -t llm-inference:latest -f llm-server/Dockerfile .
```

### 4. Run locally with Docker Compose (before Railway)

```bash
# Copy model to the models/ directory first
docker compose up --build
```

Test: `curl http://localhost:8080/health`

### 5. Deploy to Railway

**Option A — Two separate Railway services:**
1. Create a new Railway project
2. Add a service for `starfire-core`: point to the repo, use `Dockerfile`
3. Add a service for `llm-inference`: point to the repo, use `llm-server/Dockerfile`
4. In `starfire-core` service settings, add environment variable:
   ```
   LLM_ENDPOINT=http://llm-inference:8081
   ```
5. Link the services in Railway (they'll share internal DNS)

**Option B — Docker Compose on a Railway dedicated server:**
Deploy the compose file to a Railway VPS with Docker support.

### 6. Verify

```bash
# Health check on starfire
curl https://your-starfire.railway.app/health

# Health check on llm-inference
curl https://your-llm.railway.app/health
```

---

## Environment Variables

### starfire-core
| Variable | Default | Description |
|---|---|---|
| `LLM_ENDPOINT` | `http://127.0.0.1:8081` | URL of the llm-inference service |
| `RAILWAY_PUBLIC_DOMAIN` | — | Set automatically by Railway |
| `STARFIRE_PORT` | `8080` | Port star listens on |

### llm-inference
| Variable | Default | Description |
|---|---|---|
| `GGUF_PATH` | `/models/bonsai-8b/Bonsai-8B.gguf` | Path to model in container |
| `HOST` | `0.0.0.0` | Bind host |
| `PORT` | `8081` | HTTP server port |

---

## Local Development

```bash
# Terminal 1 — start llm-server
cd llm-server
cargo run --release

# Terminal 2 — start star
cargo run --bin star -- chat
```

Or with Docker Compose:
```bash
docker compose up --build llm
# then in another terminal:
cargo run --bin star -- chat
```

---

## Troubleshooting

**llm-server fails to load model:**
```bash
# Check model file exists in container
docker run --rm -it llm-inference ls /models/bonsai-8b/
```

**starfire returns "model not loaded":**
```bash
# Check llm-server is healthy
curl http://llm-inference:8081/health

# Check starfire can reach it
docker compose exec starfire curl http://llm:8081/health
```

**Railway linking not working:**
Make sure both services are in the same Railway project. Then use the exact service name as hostname: `http://llm-inference:8081` (not `localhost`).
