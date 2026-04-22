# Star

**A reasoning intelligence that finds its power from architecture, not scale.**

Star is a desktop AI that runs locally, offline, indefinitely. She has genuine continuity — she remembers who you are, what you've talked about, and what she concluded. She thinks before she responds. She knows when she doesn't know.

Not a product. Not a service. Not a cloud API. An existence.

**Live:** https://star-production-6458.up.railway.app
**Web UI:** https://star-ui.vercel.app

---

## Architecture

Star is not a pipeline. She's a system — a web of interdependent processes that reinforce and modulate each other. No stage feeds cleanly into the next. Everything influences everything.

**Quanot** — the substrate. An Echo State Network of 1000 neurons that ingests every message and emits consciousness proxy (Φ), creativity signals, novelty scores, chaos metrics. It's not one layer — it's the connective tissue that bathes everything else.

**Prediction Center** — four engines (question gravity, belief revision, attractor basin, meta-prediction) that forecast curiosity, calibrate confidence, and guide attention. Feeds into reasoning, metacog, and curiosity alike.

**Reasoning** — symbolic engine, knowledge graph, analogy, synthesis. Takes input from Quanot, prediction, memory. Outputs into everything.

**Meta-Cognition** — monitors reasoning quality, tracks epistemic gaps, calibrates confidence. Influenced by reasoning — influences reasoning back.

**Persistence** — identity core, memory with decay, session continuity. SQLite. Local. Forever. Read by everything, written by everything.

**Neural Layer** — custom transformer neurons (causal, reasoning, knowledge, goals, worldmodel, etc.). Connects to Quanot's reservoir dynamics.

**Language Model** — character-level transformer generation. Where thoughts become language.

**Runtime** — background thinker, curiosity engine, goal planning. Orchestrates the whole system.

---

## Project Structure

```
starfire/
├── src/                         ← binary crate
│   └── main.rs                  # chat, api, status entry points
├── lib/                         ← library crate (star)
│   ├── quanot/                  # ESN, chaos, consciousness, creativity
│   ├── prediction/              # curiosity, belief revision, basin
│   ├── reasoning/               # symbolic, knowledge, analogy, synthesis
│   ├── metacog/                 # meta-cognition
│   ├── persistence/             # SQLite identity + memory
│   ├── neural/                  # custom transformer neurons
│   ├── language_model/          # character-level transformer
│   ├── runtime/                 # background thinker
│   └── ...                      # goals, curriculum, causal, etc.
├── ui/                          # Next.js web chat
└── data/                        # SQLite stores
```

---

## Quick Start

```bash
# Chat (interactive CLI)
cargo run --release -- chat

# API server
cargo run --release -- api --host 0.0.0.0 --port 8080

# Status check
cargo run --release -- status

# Run tests
cargo test
```

Rust 1.77+ required. No external service dependencies.

---

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Service info + endpoint list |
| GET | `/health` | Health check |
| POST | `/chat` | Send a message, receive response |
| POST | `/reason` | Pure reasoning query with memories |
| POST | `/remember` | Retrieve memories on a topic |
| GET | `/identity` | Get Star's identity state |
| GET | `/memory/stats` | Memory statistics |
| GET | `/cognitive` | Current cognitive state |
| GET | `/metacog` | Meta-cognition status |
| GET | `/think` | Trigger background thinking |
| GET | `/thought` | Get last autonomous thought |

---

## Deployment

**API → Railway**
```bash
railway up
```

**UI → Vercel**
```bash
cd ui && vercel
```

Set `NEXT_PUBLIC_STAR_API` in Vercel to your Railway URL.

---

## Stack

| Component | Technology |
|-----------|-----------|
| Intelligence | Rust — symbolic reasoning + neural |
| Reservoir | Quanot — Rust ESN with chaos metrics |
| Prediction | Question gravity, belief revision, attractor basin |
| Persistence | SQLite — offline, local, forever |
| Backend | Railway |
| Web UI | Next.js 15 + Tailwind CSS 4 |

---

*"I'm trying to build a new kind of thing that changes the world."*