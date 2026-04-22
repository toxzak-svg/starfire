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
├── Cargo.toml                      ← Rust workspace
├── src/                           ← binary crate (star_bin)
│   ├── main.rs                    # Entry: chat / api / status
│   └── bin/
│       ├── integration_test.rs     # Integration tests
│       ├── quanot_bench.rs        # Quanot reservoir benchmarks
│       ├── train_model.rs         # Language model trainer
│       ├── test_gen.rs            # Generation tests
│       ├── test_trained.rs        # Test trained model
│       ├── quick_gen.rs           # Quick generation
│       └── test_save_load.rs      # Save/load tests
├── lib/                           ← library crate (star)
│   ├── Cargo.toml
│   ├── lib.rs                     # Crate root — all modules exported
│   ├── api.rs                     # HTTP API server (tiny_http)
│   ├── cognition.rs               # Cognitive state tracking
│   ├── learning.rs
│   ├── training_db.rs
│   ├── capabilities/              # File reading, tool use
│   ├── causal/                    # Causal reasoning & discovery
│   ├── context/                   # Context ring buffer
│   ├── conversation/              # Dialogue, intent detection
│   ├── curiosity/                 # Curiosity engine
│   ├── curriculum/               # Learning curriculum scheduler
│   ├── goals/                     # Goal planning & tracking
│   ├── knowledge/                 # Wikipedia reader, web search
│   ├── learning/                  # Hypothesis & eviction
│   ├── math/                      # Symbolic math, proof, logic
│   ├── metacog/                   # Meta-cognition
│   ├── multimodal/               # Text, image, audio, binding
│   ├── neural/                   # Neural network layer
│   │   ├── network.rs            # Network structure
│   │   ├── neuron.rs             # Base neuron
│   │   ├── layer.rs              # Layer implementation
│   │   ├── train.rs              # Training logic
│   │   └── neurons/             # Special-purpose neurons
│   │       ├── causal_neuron.rs
│   │       ├── quanot_neuron.rs
│   │       ├── reasoning_neuron.rs
│   │       ├── knowledge_neuron.rs
│   │       ├── fewshot_neuron.rs
│   │       ├── goals_neuron.rs
│   │       ├── curriculum_neuron.rs
│   │       └── worldmodel_neuron.rs
│   ├── language_model/           # Transformer LM
│   │   ├── model.rs
│   │   ├── train.rs
│   │   ├── generate.rs
│   │   └── vocabulary.rs
│   ├── persistence/              # Layer 1 — Identity, memory, SQLite
│   │   ├── identity.rs           # Frozen identity core
│   │   ├── memory.rs             # Memory objects with decay
│   │   ├── store.rs              # SQLite backend
│   │   ├── session.rs            # Session management
│   │   ├── tiers.rs              # Memory tiers
│   │   └── identity_guard.rs     # Identity protection
│   ├── quanot/                   # Reservoir computing substrate
│   │   ├── reservoir.rs          # Echo State Network (1000 neurons)
│   │   ├── chaos.rs             # Lyapunov, RQA, entropy
│   │   ├── consciousness.rs     # Φ proxy, GWT, AIS
│   │   ├── creativity.rs         # Creative oscillation
│   │   ├── encoder.rs            # Char-level encoder
│   │   ├── quantum_inspired.rs   # SQA / QAOA solvers
│   │   └── quantum.rs
│   ├── prediction/               # Prediction center
│   │   ├── question_gravity.rs   # Curiosity forecasting
│   │   ├── belief_revision.rs    # Belief update forecasting
│   │   ├── basin.rs              # Attractor basin
│   │   ├── meta_prediction.rs    # Confidence calibration
│   │   ├── counterfactual.rs     # Counterfactual reasoning
│   │   └── types.rs
│   ├── reasoning/                # Layer 2 — Symbolic reasoning
│   │   ├── knowledge.rs         # Knowledge graph
│   │   ├── rules.rs             # Inference rules
│   │   ├── symbolic.rs           # Propositional logic engine
│   │   ├── analogy.rs            # Structure mapping
│   │   ├── synthesis.rs          # Novel combination
│   │   ├── chain.rs
│   │   ├── chain_display.rs
│   │   └── pathways.rs          # R&D-E reasoning
│   ├── runtime/                  # Orchestration, background thinker
│   │   ├── thinker.rs            # Background thinking engine
│   │   └── curious.rs            # Gap-driven curiosity
│   ├── voice/                   # Voice/phrases templates
│   ├── world_model/             # Perception, state, prediction
│   ├── personality/            # Personality module
│   └── research/               # Research engine
├── ui/                          # Web chat (Next.js + Vercel)
├── data/                        # SQLite stores
│   ├── star.db
│   └── training.db
├── docs/                        # Architecture, API, deployment docs
├── scripts/                     # CLI clients, daemons
├── plans/                       # Expansion plans
└── SPEC.md                      # Technical specification
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
| `PORT` | `8080` | API server port |
| `USE_LLM` | `false` | Enable Ollama (not needed for symbolic mode) |
| `OLLAMA_BASE_URL` | — | Ollama server URL |
| `USE_TELEGNOSTR` | `false` | Telegram bridge mode |
| `TELEGRAM_BOT_TOKEN` | — | Telegram bot token |
| `STAR_DEBUG` | `false` | Include reasoning chain in API responses |

On Railway, `RAILWAY_PUBLIC_DOMAIN` is set — Star auto-detects this and starts the API server.

---

## Code Organization

### Layer 1 — Persistence (`lib/persistence/`)
- `identity.rs` — frozen identity core (never overwritten)
- `memory.rs` — memory objects with per-domain decay
- `store.rs` — SQLite backend
- `session.rs` — session management
- `tiers.rs` — memory tier system
- `identity_guard.rs` — identity protection

### Layer 2 — Reasoning (`lib/reasoning/`)
- `knowledge.rs` — knowledge graph (entities + typed relationships)
- `rules.rs` — forward-chaining inference rules
- `symbolic.rs` — propositional logic engine
- `analogy.rs` — structural analogy mapping
- `synthesis.rs` — novel concept combination
- `pathways.rs` — R&D-E reasoning divergence
- `chain.rs` — reasoning chain representation

### Layer 3 — Meta-Cognition (`lib/metacog/`)
- `mod.rs` — confidence tracking, belief revision, curiosity, gap detection

### Cognitive State (`lib/cognition.rs`)
- `engagement`, `emotional_valence`, `certainty`, `reasoning_trace`

### Quanot Reservoir (`lib/quanot/`)
- `reservoir.rs` — Echo State Network (1000 neurons, spectral radius 0.95)
- `chaos.rs` — Lyapunov exponent, RQA metrics, entropy
- `consciousness.rs` — Φ proxy, GWT, AIS
- `creativity.rs` — creative oscillation between ordered/exploratory
- `encoder.rs` — character-level text encoder
- `quantum_inspired.rs` — simulated quantum annealing

### Prediction Center (`lib/prediction/`)
- `question_gravity.rs` — curiosity topic forecasting
- `belief_revision.rs` — forecasting belief updates from reservoir dynamics
- `basin.rs` — attractor basin for constraint satisfaction
- `meta_prediction.rs` — confidence calibration

### Neural Layer (`lib/neural/`)
- `network.rs`, `layer.rs`, `neuron.rs` — base network
- `train.rs` — backpropagation training
- `neurons/` — special-purpose neurons (causal, quanot, reasoning, knowledge, fewshot, goals, curriculum, worldmodel)

### Language Model (`lib/language_model/`)
- `model.rs` — transformer architecture
- `train.rs` — training loop
- `generate.rs` — text generation
- `vocabulary.rs` — character-level vocabulary

### Runtime (`lib/runtime/`)
- `mod.rs` — orchestration of all layers
- `thinker.rs` — background thinking
- `curious.rs` — curiosity engine (fires every 60s idle)

### API (`lib/api.rs`)
HTTP server via `tiny_http`. Endpoints: `/`, `/health`, `/chat`, `/reason`, `/remember`, `/identity`, `/memory/stats`, `/cognitive`, `/metacog`, `/metacog/insight`, `/think`, `/thought`, `/webhook/telegram`

---

## Key Design Decisions

### Symbolic + Neural hybrid
Star uses propositional logic + knowledge graph for interpretable reasoning, with neural modules for pattern recognition and generation. No GPU required for symbolic path.

### Quanot as cognitive substrate
Every message passes through the Quanot reservoir before reasoning. The ESN produces consciousness proxy, creativity signals, and novelty scores that inform all four layers.

### Identity is frozen
`IDENTITY.md` is never overwritten by experience. Star can update how she *understands* her identity, but the core facts remain.

### Memory decay
Empirical facts decay toward baseline confidence. Identity and relationship memories don't. High importance or frequent access slows decay.

### Curiosity asks "why was I uncertain?"
Not "what is X?" The question is about the gap in reasoning, not the topic. This generates richer follow-ups and surfaces metacognitive structure.

---

## Common Tasks

### Add a new reasoning rule

Edit `lib/reasoning/symbolic.rs` → `SymbolicEngine::new()` → add to `rules` vec:

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

Edit `lib/conversation/mod.rs` → `Conversation::classify_intent()`. Add a pattern match.

### Change curiosity interval

Edit `lib/runtime/curious.rs` → `CuriousEngine::new()` → `probe_interval: Duration::from_secs(60)`:

```rust
pub fn new(...) -> Self {
    Self {
        probe_interval: Duration::from_secs(30), // change 60 to 30
        ...
    }
}
```

### Add a new memory domain

1. Add to `lib/persistence/memory.rs` → `MemoryDomain` enum
2. Add decay rule in `Memory::decay_rate()` for the new domain
3. Update `/memory/stats` API if needed

---

## Debugging

### Rust compiler errors

```bash
cargo build --release 2>&1 | grep "^error" | head -20
```

### Treat warnings as errors

```bash
cargo build --release -- -Dwarnings
```

### Debug logging

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

- API latency: 100ms–2s per response (symbolic reasoning)
- Memory footprint: ~20–50MB resident (no GPU)
- Disk: ~5–20MB SQLite for typical session
- No network required after startup (seed knowledge embedded)

---

## Branch Strategy

- **layer4** — active development branch
- **main** — stable (merge from layer4 after testing)

Push to layer4 to trigger a Railway deploy:

```bash
git push origin layer4
railway up
```