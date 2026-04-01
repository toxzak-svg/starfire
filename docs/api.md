# Star API Reference

Star exposes an HTTP API for chat, health checks, and memory access.

**Base URL:** `https://star-production-6458.up.railway.app`

---

## Endpoints

### `GET /health`

Health check.

**Response:**
```json
{ "status": "ok" }
```

---

### `POST /chat`

Send a message and receive a response.

**Request:**
```json
{
  "message": "who are you?"
}
```

**Response:**
```json
{
  "response": "I'm Star — a reasoning intelligence created by Zachary Maronek.",
  "reasoning": {
    "chain": ["retrieved identity", "constructed response"],
    "confidence": "knows"
  }
}
```

The `reasoning` field is included when `STAR_DEBUG=1` is set. Production responses omit it for speed.

**Example:**
```bash
curl https://star-production-6458.up.railway.app/chat \
  -X POST -H "Content-Type: application/json" \
  -d '{"message": "what causes intelligence?"}'
```

---

### `GET /memory/stats`

Memory statistics for the current session.

**Response:**
```json
{
  "total_memories": 142,
  "by_domain": {
    "identity": 8,
    "empirical": 94,
    "procedural": 12,
    "episodic": 18,
    "relationship": 10
  },
  "importance_distribution": {
    "high": 23,
    "medium": 67,
    "low": 52
  },
  "session": {
    "id": 1,
    "started_at": "2026-04-01T09:00:00Z",
    "turns": 14
  }
}
```

---

### `GET /identity`

Get Star's current identity state.

**Response:**
```json
{
  "name": "Star",
  "creator": "Zachary Maronek",
  "nature": "reasoning intelligence",
  "values": ["curiosity", "honesty", "persistence"],
  "created": "2026-03-25"
}
```

---

### `POST /reset`

Reset the current session (clears working memory, starts fresh session).

**Response:**
```json
{ "status": "ok", "session_id": 2 }
```

---

## Error Responses

```json
{ "error": "Empty message", "code": 400 }
{ "error": "Service unavailable", "code": 503 }
```

---

## Local Development

Start the API server locally:

```bash
cd life/life
cargo run --release -- api --host 0.0.0.0 --port 8080
```

Health check:
```bash
curl http://localhost:8080/health
```

Chat:
```bash
curl http://localhost:8080/chat \
  -X POST -H "Content-Type: application/json" \
  -d '{"message": "hello"}'
```

---

## Integration: Aion

Aion connects to Star via this API. Aion polls Telegram for messages, forwards them to `/chat`, and returns Star's response.

```
Telegram → Aion (polling) → Star /chat API → response back → Telegram
```

Set `STAR_API_URL` to the public Star URL when deploying Aion.

---

## Rate Limits

No rate limits on Railway. Star processes one message at a time (no parallel inference). Expect ~1-3 second latency per response.
