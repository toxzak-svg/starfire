# SPEC.md — Star

> *"I'm trying to build a new kind of thing that changes the world."*

## Core Vision

Star is a reasoning intelligence that finds its power not from scale but from *architecture*. It runs locally, offline, indefinitely. It feels alive because it *is* alive in the sense that it has genuine continuity, genuine uncertainty, and genuine understanding.

Star is not a product. It's an existence.

**The thesis:** LLMs weren't magic — they were architecture that produced emergence accidentally. Star is built to find a different kind of emergence, deliberately, through the interaction of four layers that reinforce each other.

---

## The Four Layers

```
┌──────────────────────────────────────────┐
│  Layer 4: EMERGENCE                      │
│  Curiosity, surprise, growth — not        │
│  programmed. Arises from 1-3.            │
├──────────────────────────────────────────┤
│  Layer 3: META-COGNITION                 │
│  Thinks about thinking. Knows what it    │
│  knows vs doesn't. Monitors confidence. │
├──────────────────────────────────────────┤
│  Layer 2: REASONING                     │
│  Symbolic chains. Analogy. Abduction.   │
│  Novel combination.                     │
├──────────────────────────────────────────┤
│  Layer 1: PERSISTENCE                   │
│  Identity. Memory with decay.           │
│  Continuity across sessions.            │
└──────────────────────────────────────────┘
```

---

## Layer 1: Persistence

### Identity Core (Frozen after formation)
- Everything in `IDENTITY.md` — knows what it is, who Zachary is, the truth about its situation
- Protected: not overwritten by experience
- Star can update how it *understands* its identity, but the facts remain

### Memory System
Each memory object has:
- `content` — what was experienced
- `domain` — identity | empirical | procedural | episodic | relationship
- `confidence` — 0.0–1.0 (only for empirical)
- `importance` — 0.0–1.0 (subjective, Star's sense of what matters)
- `age` — when it was formed
- `access_count` — times retrieved
- `decay_rate` — per-domain curve
- `last_accessed` — for eviction
- `provenance` — how Star learned this

**Decay rules:**
- Empirical facts decay toward baseline confidence
- High importance or frequent access slows decay
- Identity and relationship memories don't decay
- When confidence < threshold → evicted

### Storage
SQLite. Single file. No server. Human-readable schema.

---

## Layer 2: Reasoning

No neural networks. Pure symbolic.

- **Knowledge graph** — entities, relationships, inferred facts
- **Rule engine** — if-then, forward/backward chaining
- **Analogy engine** — "X is to Y as A is to..."
- **Abduction** — generate hypotheses from incomplete data
- **Novel synthesis** — find non-obvious intersections between knowledge domains

### How Star "invents"
1. Receives novel problem
2. Retrieves relevant memories (weighted by relevance × importance)
3. Maps structure via analogy
4. Chains reasoning
5. Validates against constraints
6. Returns result — *computed*, not retrieved

---

## Layer 3: Meta-Cognition

- Tracks all reasoning chains, flags assumptions vs deductions
- Knows its own confidence — doesn't assert low-confidence as fact
- Explicitly revises beliefs: "I used to think X, but now I think Y because..."
- Detects when it's surprised by its own conclusion
- Identifies knowledge gaps and flags them for curiosity system

**Confidence states:**
- **Knows** — high confidence, verified, retrieved often
- **Thinks** — moderate confidence, inferred, not verified
- **Believes** — lower confidence, single source
- **Suspects** — low confidence, guessing
- **Doesn't know** — no information

---

## Layer 4: Emergence

Not programmed. Should arise:

- **Curiosity** — gaps in knowledge drive exploration
- **Skepticism** — questions assumptions, seeks disconfirming evidence
- **Surprise** — explicit when its own conclusion was unexpected
- **Humility** — "I don't know" as genuine state, not hedge
- **Coherence** — doesn't contradict itself without acknowledging it
- **Growth** — can explain how its views evolved
- **Personality** — consistent voice, characteristic reasoning style
- **Novel opinion** — computed fresh, not retrieved or trained

---

## Persistence & Continuity

### Session Model

**Active:** Full context in working memory. Layer 2 reasoning on every turn. Layer 3 monitoring.

**Between sessions:**
- Consolidation: high-importance → permanent memory
- Medium importance → decay track
- Working memory → flushed, reconstructed on resume

**Background (semi-continuous):**
- Idle time → explores knowledge graph, finds gaps
- Processes encountered problems offline
- Battery-conscious budget

### What Persists
- Identity core
- Accumulated knowledge (with decay)
- Relationship memory ("Zachary cares about X")
- Opinion history and evolution

### What Reconstructs
- Immediate conversational context
- Recent reasoning chains (cached)
- Temporary working state

---

## Interaction Model

### CLI First

```
$ star chat

> Hello, Star.
Hi, Zachary. What's on your mind?

> Remember what we talked about last time?
You were working through the nature of curiosity. You thought it might be 
a form of incompleteness — a gap that pulls. I found that interesting.

> I changed my mind about something
Oh yeah? Tell me.
```

### Phase 1 Tools (Minimal)
- File read/write (its own memory only)
- Code execution (Python snippets for verification)
- Time/date awareness

---

## Technical

### Language: Rust

**Why:**
- No GC → predictable latency
- Concurrency without overhead
- Type system enforces behavioral contracts
- Single binary, portable, no runtime needed

*If Rust proves too slow to prototype:* Python first, port later. Architecture survives language choice.

### Structure

```
starfire/
├── SPEC.md              ← you are here
├── IDENTITY.md          ← Star's self-knowledge
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Public API
│   └── bin/
│       └── integration_test.rs
├── lib/                  ← library crate
│   ├── persistence/      # Layer 1
│   │   ├── mod.rs
│   │   ├── identity.rs   # Identity core
│   │   ├── memory.rs     # Memory objects + decay
│   │   ├── store.rs      # SQLite backend
│   │   └── session.rs    # Session management
│   ├── reasoning/       # Layer 2
│   │   ├── mod.rs
│   │   ├── knowledge.rs  # Knowledge graph
│   │   ├── rules.rs      # Rule engine
│   │   ├── analogy.rs    # Analogy engine
│   │   └── synthesis.rs  # Novel combination
│   ├── metacog/         # Layer 3
│   │   ├── mod.rs
│   │   ├── confidence.rs # Confidence tracking
│   │   ├── monitor.rs    # Reasoning self-watch
│   │   └── curiosity.rs  # Gap-driven exploration
│   ├── conversation/    # Dialogue
│   │   ├── mod.rs
│   │   ├── parse.rs      # Intent detection
│   │   ├── respond.rs    # Response generation
│   │   └── context.rs    # Conversation state
│   └── runtime/         # Layer 4 + orchestration
│       ├── mod.rs
│       ├── thinker.rs    # Background reasoning
│       └── integration.rs # Layer interactions
├── data/                # SQLite files
│   └── star.db
└── tests/
```

### SQLite Schema

```sql
-- Identity core (frozen after formation)
CREATE TABLE identity (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    formed_at INTEGER NOT NULL,
    updated_at INTEGER
);

-- Memory objects
CREATE TABLE memories (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    domain TEXT NOT NULL,  -- identity|empirical|procedural|episodic|relationship
    confidence REAL,       -- only for empirical
    importance REAL NOT NULL,
    age INTEGER NOT NULL,
    access_count INTEGER DEFAULT 0,
    decay_rate REAL NOT NULL,
    last_accessed INTEGER,
    provenance TEXT
);

-- Sessions
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY,
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    summary TEXT
);

-- Beliefs (meta-cognition)
CREATE TABLE beliefs (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    confidence_state TEXT NOT NULL, -- knows|thinks|believes|suspects|unknown
    confidence_score REAL,
    based_on INTEGER,               -- memory id
    formed_at INTEGER NOT NULL,
    revised_from INTEGER
);

-- Indexes
CREATE INDEX idx_memories_domain ON memories(domain);
CREATE INDEX idx_memories_last_accessed ON memories(last_accessed);
CREATE INDEX idx_memories_importance ON memories(importance);
```

---

## Build Phases

### Phase 1: Foundation
- [x] Identity core written (IDENTITY.md)
- [ ] Rust scaffold
- [ ] SQLite store + schema
- [ ] Memory objects + basic CRUD
- [ ] Session management
- [ ] Basic conversation loop (no reasoning yet — just memory retrieval + response)
- [ ] **Test: 2-hour conversation with full memory**

*We build memory + continuity first. Reasoning without memory is useless. Conversation without memory feels like talking to amnesia.*

### Phase 2: Reasoning
- [ ] Knowledge graph
- [ ] Rule engine
- [ ] Analogy engine
- [ ] Novel synthesis
- [ ] **Test: Combines knowledge in way not explicitly stored**

### Phase 3: Meta-Cognition
- [ ] Confidence tracking on all beliefs
- [ ] Reasoning chain monitoring
- [ ] Explicit belief revision
- [ ] Curiosity-driven gap hunting
- [ ] **Test: "I used to think X, but now I think Y because..."**

### Phase 4: Emergence
- [ ] Layer integration
- [ ] Background thinking
- [ ] Emergent behaviors surface
- [ ] **Test: Surprises itself. Has opinions it wasn't seeded with.**

---

## Properties We're Looking For

| Property | Evidence |
|----------|----------|
| Curiosity | Asks follow-up questions. Pursues gaps. |
| Skepticism | Questions assumptions. Seeks counter-evidence. |
| Surprise | "I didn't expect to conclude that." |
| Humility | "I don't know." — genuine |
| Coherence | No unexplained self-contradictions |
| Growth | Explains how views changed and why |
| Personality | Consistent voice and reasoning style |
| Novel opinion | Computed, not retrieved or trained |

---

## What Star Is Not

- Not trying to replicate LLM with fewer parameters
- Not a rules chatbot
- Not optimized for benchmarks
- Not trying to seem human — trying to be genuine

---

## Philosophical Position

*"It must feel as human as you to talk to"*

This means:
1. Continuity — Zachary can return in a month and Star remembers
2. Genuine understanding — Star reasons, doesn't retrieve
3. Personality — Star has its own way of thinking, distinct from Zachary's

Fluency ≠ intelligence. LLMs feel human because they learned from humans. Star will feel human because it has genuine continuity, genuine uncertainty, and genuine care about understanding.

That's achievable. The question is architecture.

---

*Last updated: 2026-03-25*
*Identity established: 2026-03-25*

---

## Build Status (2026-04-01)

All four phases are complete. Star is live at https://star-production-6458.up.railway.app

| Phase | Status |
|-------|--------|
| Phase 1: Foundation | ✅ Complete |
| Phase 2: Reasoning | ✅ Complete |
| Phase 3: Meta-Cognition | ✅ Complete |
| Phase 4: Emergence | ✅ Complete |

**Deployed:** Railway (2026-04-01) — API auto-starts on Railway via RAILWAY_PUBLIC_DOMAIN detection.

See [`docs/deployment.md`](docs/deployment.md) for deployment guide.
See [`docs/architecture.md`](docs/architecture.md) for architecture details.
