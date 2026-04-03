# Development Guide

How to work on Star locally.

---

## Prerequisites

- Rust 1.77+ (`rustup update`)
- SQLite3 (usually pre-installed)

```bash
rustc --version  # should be 1.77+
cargo --version
```

---

## Project Layout

```
life/life/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry: chat / api / status
│   ├── api.rs               # HTTP API server (actix-web)
│   ├── cognition.rs         # Cognitive state tracking
│   ├── conversation/         # Dialogue, intent detection
│   ├── knowledge/            # Wikipedia reader, search
│   ├── metacog/             # Confidence, reasoning self-watch
│   ├── persistence/         # Identity, memory, SQLite store
│   ├── reasoning/            # KG, rules, analogy, synthesis, symbolic
│   └── runtime/              # Curious engine, background thinker
```

---

## Running Locally

### Chat (interactive CLI)

```bash
cd life/life
cargo run --release -- chat
```

### API server

```bash
cargo run --release -- api --host 0.0.0.0 --port 8080
```

### With custom data directory

```bash
cargo run --release -- chat --data-dir ./my-data
```

### Status check

```bash
cargo run -- status
```

---

## Testing

```bash
cargo test
```

Some tests require the full data directory. Run from the `life/life/` directory.

---

## Building

```bash
cargo build --release
```

Output: `life/life/target/release/star`

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `STAR_DATA_DIR` | `~/.star` or `/data/star-data` | Data directory |
| `USE_LLM` | `false` | Enable Ollama (not needed for symbolic mode) |
| `OLLAMA_BASE_URL` | — | Ollama server URL |
| `USE_TELEGNOSTR` | `false` | Telegram bridge mode |
| `STAR_DEBUG` | `false` | Include reasoning chain in API responses |

---

## Code Organization

### Layer 1 — Persistence
- `persistence/identity.rs` — frozen identity core
- `persistence/memory.rs` — memory objects with decay
- `persistence/store.rs` — SQLite backend
- `persistence/session.rs` — session management

### Layer 2 — Reasoning
- `reasoning/knowledge.rs` — knowledge graph (entities + typed relationships)
- `reasoning/rules.rs` — rule engine (if-then, forward/backward chaining)
- `reasoning/analogy.rs` — structural analogy mapping
- `reasoning/synthesis.rs` — novel combination
- `reasoning/symbolic.rs` — propositional logic inference (2026-04-01)
- `reasoning/pathways.rs` — R&D-E (reasoning pathway divergence)

### Layer 3 — Meta-Cognition
- `metacog/mod.rs` — confidence, curiosity, belief revision
- `cognition.rs` — engagement, emotional valence, certainty tracking

### Layer 4 — Runtime
- `runtime/mod.rs` — orchestration of all layers
- `runtime/curious.rs` — gap-driven curiosity (fires every 60s idle)
- `runtime/thinker.rs` — background thinking engine

### API
- `api.rs` — HTTP server (actix-web), endpoints: /health, /chat, /memory/stats, /identity

### Conversation
- `conversation/mod.rs` — intent detection, response generation

---

## Key Design Decisions

### Symbolic reasoning (no NN)
Star uses propositional logic + KG lookups. No neural networks, no GPU. The symbolic engine (`reasoning/symbolic.rs`) does forward-chaining inference. This makes reasoning interpretable and CPU-only.

### Curiosity asks "why was I uncertain?"
Not "what is X?" The question is about the gap in reasoning, not the topic. This generates richer follow-ups and surfaces metacognitive structure.

### Identity is frozen
`IDENTITY.md` is never overwritten by experience. Star can update how it *understands* its identity, but the core facts remain.

### Memory decay
Empirical facts decay. Identity and relationship memories don't. This mirrors how humans work — we forget details but remember who we are and who others are.

---

## Common Tasks

### Add a new reasoning rule

Edit `reasoning/symbolic.rs` → `SymbolicEngine::new()` → add to `rules` vec:

```rust
Rule {
    name: "my_rule",
    if_predicate: "creates".to_string(),
    then_subject: "_intermediate".to_string(),
    then_predicate: "is".to_string(),
    then_object: "_any".to_string(),
}
```

### Add a new conversation intent

Edit `conversation/mod.rs` → `Conversation::classify_intent()`. Add a pattern match.

### Change curiosity interval

Edit `runtime/curious.rs` → `CuriousEngine::new()` → `probe_interval: Duration::from_secs(60)`:

```rust
pub fn new(...) -> Self {
    Self {
        probe_interval: Duration::from_secs(30), // change 60 to 30
        ...
    }
}
```

### Add a new memory domain

1. Add to `persistence/memory.rs` → `MemoryDomain` enum
2. Add decay rule in `Memory::decay_rate()` for the new domain
3. Update `memory/stats` API if needed

---

## Debugging

### Rust compiler errors

```bash
cargo build --release 2>&1 | grep "^error" | head -20
```

### View all warnings as errors

```bash
cargo build --release -- -Dwarnings
```

### Check with debug logging

```bash
RUST_LOG=debug cargo run -- chat
```

### Inspect the SQLite store

```bash
sqlite3 ~/.star/star.db ".schema"
sqlite3 ~/.star/star.db "SELECT * FROM memories LIMIT 5;"
```

---

## Performance

- API latency: 50-500ms per chat response (symbolic reasoning)
- Memory footprint: ~20MB resident (no GPU)
- Disk: ~5MB SQLite for typical session
- No network required after startup (seed knowledge embedded)
