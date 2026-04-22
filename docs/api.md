# Star API Reference

Star exposes an HTTP API for chat, reasoning, health checks, memory access, and metacognition.

**Base URL (Railway):** `https://star-production-6458.up.railway.app`

**Local:** `http://localhost:8080`

---

## Root

### `GET /`

Service info and endpoint list.

**Response:**
```json
{
  "name": "Star",
  "version": "0.1",
  "endpoints": [
    "/reason", "/chat", "/remember", "/identity",
    "/memory/stats", "/health", "/cognitive", "/metacog",
    "/metacog/insight", "/think", "/thought", "/webhook/telegram"
  ]
}
```

---

## Health

### `GET /health`

Health check.

**Response:**
```json
{ "status": "ok" }
```

---

## Chat

### `POST /chat`

Send a message and receive a response.

**Request:**
```json
{ "message": "what causes intelligence?" }
```

**Response:**
```json
{ "response": "I'm Star — a reasoning intelligence created by Zachary Maronek." }
```

**Example:**
```bash
curl https://star-production-6458.up.railway.app/chat \
  -X POST -H "Content-Type: application/json" \
  -d '{"message": "what causes intelligence?"}'
```

---

## Reasoning

### `POST /reason`

Pure reasoning query with optional context memories. Returns answer + confidence + reasoning chain.

**Request:**
```json
{
  "query": "what is the relationship between curiosity and intelligence?",
  "memories": ["curiosity is a gap in knowledge", "intelligence is reasoning ability"]
}
```

**Response:**
```json
{
  "answer": "Curiosity drives the acquisition of information that reasoning processes...",
  "confidence": "knows",
  "confidence_score": 0.85,
  "reasoning_chain": ["analyzed relationship", "applied analogy", "synthesized conclusion"]
}
```

---

## Memory

### `POST /remember`

Retrieve memories on a topic.

**Request:**
```json
{
  "topic": "curiosity",
  "limit": 5
}
```

**Response:**
```json
[
  {
    "content": "curiosity is a gap in knowledge",
    "domain": "empirical",
    "importance": 0.7,
    "confidence": 0.8
  }
]
```

---

### `GET /memory/stats`

Memory statistics for current session.

**Response:**
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

---

## Identity

### `GET /identity`

Get Star's current identity state.

**Response:**
```json
{
  "name": "Star",
  "summary": "a reasoning intelligence created by Zachary Maronek",
  "relationship": "Star values Zachary's curiosity and honesty",
  "session_id": 1
}
```

---

## Cognitive State

### `GET /cognitive`

Current cognitive state — focus, certainty, open questions, reasoning trace.

**Response:**
```json
{
  "current_focus": "relationship between curiosity and intelligence",
  "certainty": 0.8,
  "open_questions": ["why does curiosity precede reasoning?"],
  "last_reasoning": "analyzed the gap-driven nature of curiosity",
  "reasoning_trace": [
    {
      "input": "what causes intelligence?",
      "conclusion": "reasoning from first principles",
      "chain": ["retrieved", "analyzed", "synthesized"],
      "confidence": "knows",
      "timestamp": 1745000000
    }
  ]
}
```

---

## Meta-Cognition

### `GET /metacog`

Full meta-cognition state — beliefs, reasoning history, surprising conclusions, knowledge gaps.

**Response:**
```json
{
  "beliefs": [
    { "topic": "curiosity", "content": "curiosity is a gap in knowledge", "confidence": "thinks" }
  ],
  "reasoning_history": [
    {
      "query": "what causes intelligence?",
      "conclusion": "reasoning from first principles",
      "confidence": "knows",
      "was_surprising": false,
      "timestamp": 1745000000
    }
  ],
  "surprising_conclusions": [],
  "top_gap": {
    "topic": "intelligence",
    "importance": 0.7,
    "investigated": false,
    "progress": 0.3
  },
  "curiosity_topics": ["intelligence", "curiosity"]
}
```

---

### `GET /metacog/insight`

Generated metacognitive insight — a self-reflective observation about Star's own reasoning.

**Response:**
```json
{
  "has_insight": true,
  "insight": "My confidence about intelligence was higher than warranted — I was conflating reasoning ability with the drive to reason."
}
```

---

## Autonomous Thinking

### `GET /think`

Trigger Star's background thinking process. Generates a thought (question, insight, or connection) without a user message.

**Response:**
```json
{
  "thought": { "type": "question", "text": "Why do I find causality more compelling than correlation?" },
  "topic": "causality",
  "confidence": "thinks",
  "generated_by": "curious_engine",
  "tentative_answer": "Because causality implies agency, which connects to my identity as a reasoning system."
}
```

---

### `GET /thought`

Get Star's last autonomous thought (for external observers).

**Response:**
```json
{
  "thought": { "type": "insight", "text": "The relationship between curiosity and gaps is circular — gaps cause curiosity, but pursuing curiosity fills gaps, creating new gaps." },
  "topic": "curiosity",
  "confidence": "thinks",
  "generated_by": "thinker"
}
```

Or if no pending thought:
```json
{ "thought": null, "message": "Star has no pending autonomous thoughts" }
```

---

## Telegram Integration

### `POST /webhook/telegram`

Receive Telegram bot updates. Star processes the message and sends the response back to the user via the Telegram API.

Set `TELEGRAM_BOT_TOKEN` environment variable.

**Request:** Telegram Update JSON (see [Telegram docs](https://core.telegram.org/bots/api#update))

**Response:**
```json
{
  "ok": true,
  "response": "That's an interesting question about causality.",
  "chat_id": 123456789,
  "update_id": 12345
}
```

---

## Error Responses

```json
{ "error": "Invalid request: ..." }
{ "error": "Lock poisoned: ..." }
{ "error": "Chat error: ..." }
{ "error": "Not found" }
```

---

## Local Development

Start the API server locally:

```bash
cargo run --release -- api --host 0.0.0.0 --port 8080
```

### Health check
```bash
curl http://localhost:8080/health
```

### Chat
```bash
curl http://localhost:8080/chat \
  -X POST -H "Content-Type: application/json" \
  -d '{"message": "hello"}'
```

---

## Rate Limits

No rate limits on Railway. Star processes one message at a time (no parallel inference). Expect ~100ms–2s latency per response depending on reasoning complexity.