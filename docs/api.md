# Starfire API Reference

> **Base implementation:** `lib/api.rs`  
> **Optional live boundary:** `src/live_api.rs`  
> **Last reviewed:** 2026-07-21

Starfire exposes a small JSON-over-HTTP API for chat, direct reasoning, memory retrieval, identity, cognitive inspection, metacognition, and autonomous-thought probes.

## Base URLs

| Environment | URL |
|---|---|
| Local | `http://localhost:8080` |
| Hosted research API | `https://starfire-cuee.onrender.com` |

The hosted endpoint is a research deployment. It does not currently provide built-in authentication, per-user isolation, or production-grade rate limiting.

## Common behavior

- Request and response bodies use JSON.
- CORS currently allows `*`.
- The service processes access to the shared runtime through a mutex.
- Some protected-API application failures are encoded as `{ "error": "..." }` while the HTTP status remains `200`. Clients should inspect both the status code and the JSON body.
- Unknown routes return `404` with `{ "error": "Not found" }`.
- `OPTIONS` requests receive an empty `204` response.

## Chat response variants

### Protected API

The base API returns:

```json
{
  "response": "Star's completed response"
}
```

### Production live envelope

When the executable is built with `starfire-live`, successful chat responses may include a `live` object:

```json
{
  "response": "Star's rendered response",
  "live": {
    "enabled": true,
    "pipeline": "live-integration-1",
    "trace_id": "live-...",
    "turn": 14,
    "intent": "Reflection",
    "voice_before": {},
    "voice_after": {},
    "semantic_plan": {}
  }
}
```

If the live planning or persistence layer fails, the protected response is preserved and the envelope is labeled:

```json
{
  "response": "Protected response",
  "live": {
    "enabled": false,
    "pipeline": "live-integration-1",
    "failed_open": true,
    "error": "..."
  }
}
```

Clients must treat the `response` field as the stable contract and the `live` object as optional metadata.

## Endpoint summary

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/` | Service name, version, and endpoint list |
| `GET` | `/health` | Health check |
| `POST` | `/chat` | Process a conversational message |
| `POST` | `/reason` | Run a reasoning query with optional supplied memories |
| `POST` | `/remember` | Retrieve memories by topic |
| `GET` | `/identity` | Inspect identity and session information |
| `GET` | `/memory/stats` | Inspect persistence counts |
| `GET` | `/cognitive` | Inspect current cognitive state |
| `GET` | `/metacog` | Inspect beliefs, gaps, and reasoning history |
| `GET` | `/metacog/insight` | Generate a metacognitive insight |
| `GET` | `/think` | Trigger one autonomous-thought cycle |
| `GET` | `/thought` | Read the most recent autonomous thought |
| `POST` | `/webhook/telegram` | Receive a Telegram update |
| `GET` | `/live/status` | Inspect Live Integration 1 state; live wrapper only |

## Root

### `GET /`

Returns service metadata and the protected API route list.

```bash
curl http://localhost:8080/
```

Representative response:

```json
{
  "name": "Star",
  "version": "0.1",
  "endpoints": [
    "/reason",
    "/chat",
    "/remember",
    "/identity",
    "/memory/stats",
    "/health",
    "/cognitive",
    "/metacog",
    "/metacog/insight",
    "/think",
    "/thought",
    "/webhook/telegram"
  ]
}
```

`/live/status` is implemented by the outer live server and is therefore not listed by the protected root response.

## Health

### `GET /health`

```bash
curl http://localhost:8080/health
```

```json
{
  "status": "ok"
}
```

The Docker health check uses this endpoint.

## Chat

### `POST /chat`

Processes one message through the persistent runtime.

Request:

```json
{
  "message": "What are you uncertain about right now?"
}
```

Example:

```bash
curl http://localhost:8080/chat \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"message":"What are you uncertain about right now?"}'
```

The web UI currently also sends a `history` field. The protected Rust handler ignores unknown fields and deserializes only `message`; runtime persistence and session state remain authoritative.

Error body examples:

```json
{
  "error": "Invalid request: missing field `message`"
}
```

```json
{
  "error": "Chat error: ..."
}
```

## Reasoning

### `POST /reason`

Runs the runtime reasoning engine against a query and optional caller-supplied memory strings.

Request:

```json
{
  "query": "How are curiosity and uncertainty related?",
  "memories": [
    "Curiosity can be driven by an unresolved gap.",
    "Uncertainty represents incomplete support for a conclusion."
  ]
}
```

Response shape:

```json
{
  "answer": "...",
  "confidence": "thinks",
  "confidence_score": 0.72,
  "reasoning_chain": [
    "..."
  ]
}
```

The supplied strings are converted into temporary episodic `Memory` values for this request. The endpoint does not itself persist them.

## Memory

### `POST /remember`

Retrieves memories matching a topic.

Request:

```json
{
  "topic": "curiosity",
  "limit": 5
}
```

Response:

```json
[
  {
    "content": "...",
    "domain": "empirical",
    "importance": 0.7,
    "confidence": 0.8
  }
]
```

`limit` is optional and defaults to `5`.

### `GET /memory/stats`

Returns counts from the persistence snapshot.

```json
{
  "memory_count": 142,
  "beliefs_count": 38,
  "sessions_count": 12,
  "domain_breakdown": {
    "identity": 8,
    "empirical": 94,
    "procedural": 12,
    "episodic": 18,
    "relationship": 10
  }
}
```

## Identity

### `GET /identity`

Returns the current identity summary, relationship summary, and session identifier.

```json
{
  "name": "Star",
  "summary": "...",
  "relationship": "...",
  "session_id": 14
}
```

This endpoint is read-only. Older documentation describing `POST /identity` does not match the current route table.

## Cognitive state

### `GET /cognitive`

Returns the current focus, certainty, open questions, last reasoning summary, and structured reasoning trace.

```json
{
  "current_focus": "...",
  "certainty": 0.74,
  "open_questions": [
    "..."
  ],
  "last_reasoning": "...",
  "reasoning_trace": [
    {
      "input": "...",
      "conclusion": "...",
      "chain": ["..."],
      "confidence": "thinks",
      "timestamp": 1784640000
    }
  ]
}
```

## Metacognition

### `GET /metacog`

Returns current beliefs, recent reasoning history, surprising conclusions, the top tracked gap, and curiosity topics.

```json
{
  "beliefs": [
    {
      "topic": "...",
      "content": "...",
      "confidence": "thinks"
    }
  ],
  "reasoning_history": [],
  "surprising_conclusions": [],
  "top_gap": null,
  "curiosity_topics": []
}
```

### `GET /metacog/insight`

Generates one structured metacognitive insight when available.

```json
{
  "has_insight": true,
  "kind": "ConfidenceCalibration",
  "topic": "...",
  "insight": "..."
}
```

When none is available, the structured fields may be `null`.

## Autonomous thought

### `GET /think`

Triggers one runtime thought cycle.

```json
{
  "thought": {
    "type": "question",
    "text": "..."
  },
  "topic": "...",
  "confidence": "thinks",
  "generated_by": "curious_engine",
  "tentative_answer": "..."
}
```

This is an explicit request-triggered action. It should not be interpreted as proof that the hosted process is continuously thinking between requests.

### `GET /thought`

Returns the latest autonomous thought when one exists.

```json
{
  "thought": {
    "type": "insight",
    "text": "..."
  },
  "topic": "...",
  "confidence": "thinks",
  "generated_by": "thinker"
}
```

No pending thought:

```json
{
  "thought": null,
  "message": "Star has no pending autonomous thoughts"
}
```

## Live status

### `GET /live/status`

Available only when the outer `starfire-live` server is active.

Representative response:

```json
{
  "enabled": true,
  "pipeline": "live-integration-1",
  "turn": 14,
  "voice_state": {},
  "last_trace": {}
}
```

The response may include the last raw and rendered response inside `last_trace`. Treat this endpoint and its backing trace file as potentially sensitive.

## Telegram webhook

### `POST /webhook/telegram`

Accepts a Telegram Update JSON object. Text messages are passed to `Runtime::chat`. When `TELEGRAM_BOT_TOKEN` is configured, Starfire sends a reply through Telegram’s Bot API on a spawned thread.

Representative response:

```json
{
  "ok": true,
  "response": "...",
  "chat_id": 123456789,
  "update_id": 12345
}
```

The route does not currently implement Telegram signature verification or an independent shared secret.

## Client guidance

A resilient client should:

1. use the `response` field as the chat contract;
2. treat `live` as optional;
3. check for an `error` field even after HTTP 200;
4. use timeouts and retry only transport failures;
5. avoid exposing `/identity`, `/memory/*`, `/cognitive`, `/metacog`, or `/live/status` publicly without access control;
6. not assume browser-provided history is consumed by the Rust handler.

## Local server command

```bash
cargo run --release -p star_bin --bin star -- \
  --data-dir ./data/dev \
  api --host 0.0.0.0 --port 8080
```

## Related documents

- [Architecture](architecture.md)
- [Deployment](deployment.md)
- [Current status](CURRENT_STATUS.md)
