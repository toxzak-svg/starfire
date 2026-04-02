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
life/                          ← workspace root
├── Cargo.toml                 ← Rust workspace
├── src/                       ← binary crate (star_bin)
│   ├── Cargo.toml
│   └── main.rs
├── lib/                       ← library crate (star)
│   ├── Cargo.toml
│   ├── lib.rs
│   ├── api.rs
│   ├── cognition.rs
│   ├── learning.rs
│   ├── training_db.rs
│   ├── capabilities/
│   ├── context/
│   ├── conversation/
│   ├── knowledge/
│   ├── metacog/
│   ├── persistence/
│   ├── reasoning/
│   └── runtime/
├── ui/                        ← web chat (Next.js + Vercel)
│   ├── src/app/
│   │   ├── layout.js
│   │   ├── page.js
│   │   └── globals.css
│   ├── lib/api.js
│   ├── package.json
│   └── vercel.json
├── data/                      ← SQLite stores
│   ├── star.db
│   └── training.db
├── docs/                      ← architecture, API, deployment docs
├── scripts/                   ← CLI clients, daemons
│   ├── chat_star.py
│   ├── star_learn.py
│   ├── think_engine.py
│   ├── curiosity_daemon.py
│   └── webhook_bridge.py
├── notebooks/                 ← research notebooks + session logs
│   ├── self_model_benchmark.ipynb
│   └── memory/
├── SPEC.md                    ← technical specification
├── IDENTITY.md                ← Star's self-knowledge
├── Dockerfile                 ← Railway deployment
└── railway.json               ← Railway config
```

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
