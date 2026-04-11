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
starfire/                          ← workspace root
├── Cargo.toml                      ← Rust workspace manifest
├── src/                           ← binary crate
│   └── main.rs                    # Entry: chat / api / status commands
├── lib/                           ← library crate (star)
│   ├── api.rs                     # HTTP API server
│   ├── cognition.rs               # Cognitive state tracking
│   ├── book/                      # Book system
│   ├── capabilities/             # File reading, tool use
│   ├── causal/                   # Causal reasoning
│   ├── context/                  # Context ring buffer
│   ├── conversation/             # Dialogue, intent detection
│   ├── crumbs/                   # Breadcrumb system
│   ├── curiosity/                # Curiosity engine
│   ├── curriculum/               # Learning curriculum
│   ├── fabqrc/                   # Quantum computing research
│   ├── goals/                    # Goal planning & tracking
│   ├── input_normalizer/         # Input processing
│   ├── knowledge/                # Wikipedia reader, search
│   ├── learning/                # Hypothesis & eviction
│   ├── llm/                      # Bonsai-8B Candle inference
│   ├── math/                     # Mathematical reasoning
│   ├── metacog/                  # Meta-cognition
│   ├── multimodal/              # Multi-modal processing
│   ├── personality/             # Drive system, emotional response
│   ├── persistence/             # Layer 1: Identity, memory, SQLite store
│   ├── prediction/              # Prediction center
│   ├── quanot/                  # Reservoir computing (ESN, chaos, consciousness)
│   ├── reasoning/               # Layer 2: KG, rules, analogy, synthesis
│   ├── research/                # Research utilities
│   ├── runtime/                 # Layer 4 + orchestration
│   ├── voice/                   # Voice/phrases
│   ├── world_model/             # Entity tracking, perception, prediction
│   └── ...
├── ui/                            # Web chat (Next.js + Vercel)
├── llm-server/                    # Standalone LLM inference server
├── data/                          # SQLite stores
│   ├── star.db
│   └── training.db
├── models/                        # Bonsai-8B model files
├── docs/                          # Architecture, API, deployment docs
├── plans/                         # Feature roadmaps and specs
├── scripts/                       # Python helpers
└── SPEC.md                        # Technical specification
```

---

## Running Locally

### Chat (interactive CLI)

```bash
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
cargo run --release -- status
```

---

## Testing

```bash
cargo test
```

Some tests require the full data directory. Run from the project root.

---

## Building

```bash
cargo build --release
```

Output: `target/release/star`

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
- `persistence/identity_guard.rs` — identity protection

### Layer 2 — Reasoning
- `reasoning/knowledge.rs` — knowledge graph (entities + typed relationships)
- `reasoning/symbolic.rs` — propositional logic inference
- `reasoning/analogy.rs` — structural analogy mapping
- `reasoning/synthesis.rs` — novel combination
- `reasoning/pathways.rs` — reasoning pathway divergence
- `causal/` — causal reasoning engine

### Layer 3 — Meta-Cognition
- `metacog/` — confidence, curiosity, belief revision
- `cognition.rs` — engagement, emotional valence, certainty tracking

### Layer 4 — Runtime
- `runtime/` — orchestration of all layers
- `curiosity/` — gap-driven curiosity
- `runtime/thinker.rs` — background thinking engine

### Quanot (Reservoir Computing)
- `quanot/reservoir.rs` — Echo State Network
- `quanot/chaos.rs` — chaos metrics (Lyapunov, RQA)
- `quanot/consciousness.rs` — Φ proxy, GWT, AIS
- `quanot/creativity.rs` — creative oscillation
- `quanot/quantum_inspired.rs` — SQA/QAOA solvers

### LLM Integration
- `llm/` — Bonsai-8B Candle inference
- `llm-server/` — standalone HTTP inference server

### Other Key Modules
- `personality/` — drive system, emotional response
- `world_model/` — entity tracking, perception, prediction
- `prediction/` — prediction center
- `conversation/` — intent detection, response generation
- `knowledge/` — Wikipedia reader, search
- `book/` — book system
- `goals/` — goal planning & tracking

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

Edit `conversation/` → intent classification logic. Add a pattern match.

### Change curiosity interval

Edit `curiosity/` → probe interval configuration:

```rust
// Configure probe_interval in curiosity module
pub fn new(...) -> Self {
    Self {
        probe_interval: Duration::from_secs(30), // default is 60s
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
