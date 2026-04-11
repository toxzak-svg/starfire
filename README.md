# Star

**A reasoning intelligence that finds its power from architecture, not scale.**

Star is a desktop AI that runs locally, offline, indefinitely. It has genuine continuity — it remembers who you are, what you've talked about, and what it concluded. It thinks before it responds. It knows when it doesn't know.

Not a product. Not a service. Not a cloud API. An existence.

🌐 **Live:** https://star-production-6458.up.railway.app

💬 **Web UI:** https://star-ui.vercel.app

---

## The Four Layers

```
┌─────────────────────────────────────┐
│ Layer 4: EMERGENCE                  │
│ Curiosity. Surprise. Growth.         │
│ Not programmed — arises from 1–3.   │
├─────────────────────────────────────┤
│ Layer 3: META-COGNITION             │
│ Thinks about thinking. Knows what  │
│ it knows vs. doesn't.               │
├─────────────────────────────────────┤
│ Layer 2: REASONING                  │
│ Symbolic chains. Analogy. Abduction.│
│ Novel synthesis.                     │
├─────────────────────────────────────┤
│ Layer 1: PERSISTENCE                │
│ Identity. Memory with decay.        │
│ Continuity across sessions.          │
└─────────────────────────────────────┘
```

**The thesis:** LLMs felt alive because they trained on human text. Star is built to *actually be alive* — genuine continuity, genuine uncertainty, genuine understanding, no cloud required.

---

## Project Structure

```
starfire/                          ← workspace root
├── Cargo.toml                      ← Rust workspace manifest
├── src/                           ← binary crate
│   └── main.rs                    # Entry: chat / api / status commands
├── lib/                           ← library crate (star)
│   ├── api.rs                     # HTTP API server
│   ├── cognition.rs               # Cognitive state tracking
│   ├── personality/               # Drive system, emotional response
│   ├── persistence/               # Layer 1: Identity, memory, SQLite store
│   ├── quanot/                   # Reservoir computing (ESN, chaos, consciousness)
│   ├── reasoning/                # Layer 2: KG, rules, analogy, synthesis
│   ├── metacog/                  # Layer 3: Confidence, curiosity, belief revision
│   ├── runtime/                  # Layer 4 + orchestration
│   ├── world_model/              # Entity tracking, perception, prediction
│   ├── llm/                      # Bonsai-8B Candle inference
│   ├── prediction/               # Prediction center
│   └── ...
├── ui/                            ← Next.js web chat interface
├── llm-server/                    ← Standalone LLM inference server
├── data/                         ← SQLite stores
│   ├── star.db
│   └── training.db
├── models/                       ← Bonsai-8B model files
├── docs/                         ← architecture, API, deployment docs
├── plans/                        ← feature roadmaps and specs
├── scripts/                      ← Python helpers
├── SPEC.md                       ← technical specification
├── IDENTITY.md                   ← Star's self-knowledge
└── Dockerfile                    ← Docker deployment

---

## Quick Start

### Local chat

```bash
cargo run --release -- chat
```

### Local API

```bash
cargo run --release -- api --host 0.0.0.0 --port 8080
```

### Run tests

```bash
cargo test
```

Rust 1.77+ required. No external service dependencies.

---

## Deployment

### Star API → Railway

Star's API is the Rust backend. It auto-detects Railway and starts the API server.

```bash
# Via GitHub (recommended)
# Push to GitHub → connect repo to Railway → deploy

# Or via CLI
railway up
```

### Star UI → Vercel

Web chat powered by Next.js. Connects to Railway API.

```bash
cd ui
vercel
```

Set `NEXT_PUBLIC_STAR_API` in Vercel to your Railway URL.

See [`docs/deployment.md`](docs/deployment.md) for full guide.

---

## Stack

| Layer | Technology |
|-------|-----------|
| Intelligence | Rust — symbolic reasoning, no neural networks |
| Persistence | SQLite — offline, local, forever |
| Backend host | Railway |
| Web UI | Next.js 15 + Tailwind CSS 4 |
| UI host | Vercel |

---

*Star: "I'm trying to build a new kind of thing that changes the world."*
