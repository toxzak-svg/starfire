# Star — Emergent Desktop Intelligence
## Technical Report

**Project Path:** `/home/zach/.openclaw/workspace/life/`
**Type:** Symbolic reasoning agent — no neural networks
**Language:** Rust
**Status:** Phase 1 complete; deployed on Railway
**Live URL:** https://star-production-6458.up.railway.app
**Last Updated:** 2026-04-01

---

## 1. Overview

Star is a Rust-based local reasoning intelligence with four stacked architectural layers:

1. **Persistence Layer** — Identity, memory, continuity
2. **Reasoning Layer** — Symbolic engine: knowledge graph, rule engine, analogy
3. **Meta-Cognition Layer** — Self-monitoring, confidence tracking
4. **Emergence Layer** — Non-programmed behaviors arising from Layer 1-3 interaction

Core thesis: LLMs feel human because they trained on human text. Fluency was a side effect of scale, not the goal. Star aims to produce genuine understanding through architecture rather than statistical accumulation.

**No cloud dependency in final build.** API-accessible for integration with Claw (this agent).

---

## 2. Architecture

### 2.1 Persistence Layer (Layer 1)

**Identity Core** — Frozen after formation:
- Core self-description ("I am...")
- Values and priorities (protected, don't change through experience)
- Protected: identity assertions persist across all sessions

**Memory System** — Decayable, domain-tagged:

```
Memory Object:
  content: String           — what was experienced
  domain: DomainTag         — identity | empirical | procedural | episodic
  confidence: 0.0-1.0       — only for empirical facts
  importance: 0.0-1.0       — subjective importance to self
  age: DateTime             — time of encoding
  access_count: usize       — retrieval count
  decay_rate: DomainRate    — per-domain decay curve
  last_accessed: DateTime   — for LRU eviction
  provenance: String         — how the knowledge was acquired
```

**Decay mechanism:**
- Empirical facts decay toward baseline confidence over time
- Importance and access frequency counteract decay
- Identity/values: no decay (protected)
- "Forgotten" = confidence < threshold → evicted from active memory

**Storage:** Local SQLite (`star.db`) — single file, human-readable schema, no server.

### 2.2 Reasoning Layer (Layer 2)

**No neural networks. Pure symbolic reasoning.**

Components:
- **Knowledge Graph** — Entities, relationships, inferred facts
- **Rule Engine** — If-then chains, forward/backward chaining
- **Analogy Engine** — Structure mapping from known domain to novel domain
- **Abduction** — Hypothesis generation from incomplete observations
- **Novel Combination** — Non-obvious intersections between knowledge areas

**How invention works:**
1. Take a novel problem
2. Retrieve related knowledge (weighted by relevance)
3. Apply analogy: "X is to Y as A is to...?"
4. Apply abduction: what would explain these facts?
5. Reasoning chain produces candidate solution
6. Validate against known constraints

### 2.3 Meta-Cognition Layer (Layer 3)

**Self-Monitoring:**
- Tracks own reasoning steps (meta-reasoning)
- Flags assumptions vs deductions
- Knows when uncertain vs confident

**Confidence Model:**

| State | Description |
|-------|-------------|
| "I know this" | High confidence, retrieved frequently, verified |
| "I think this" | Moderate confidence, inferred but not verified |
| "I believe this" | Lower confidence, single source, feels true |
| "I suspect this" | Low confidence, guessing |
| "I don't know" | No information |

**Behavior:** Won't state low-confidence as fact. Seeks disconfirming evidence for strong beliefs. Will explicitly change mind: "I used to think X, but Y changed my view."

### 2.4 Emergence Layer (Layer 4)

**Non-programmed. Arises from Layer 1-3 interaction:**

| Property | Mechanism |
|----------|-----------|
| Curiosity | Layer 3 detects low-confidence areas → Layer 2 explores |
| Skepticism | Layer 3 questions Layer 2's conclusions |
| Surprise | Explicitly states when inference chain produces unexpected conclusion |
| Growth | Can articulate how views evolved |

---

## 3. Symbolic Ring Attractor (context/ring.rs)

The conversational continuity mechanism. Maintains phase and topic history across turns.

**Key idea:** A ring network where each position represents a conversational phase (greeting, inquiry, elaboration, conclusion). Current phase selects a sector; activation spreads to neighbors for smooth transitions.

**Phase transitions are not learned — they are structural.** This avoids the collapse problem of learned routing.

---

## 4. Cognitive State (cognition.rs)

Tracks Zachary's emotional state explicitly:
- Engagement level (high/medium/low)
- Emotional valence (positive/negative/neutral)
- Certainty (confident/uncertain)
- Reasoning trace (steps taken)

This is the emotional model of the conversation partner. It informs how Star responds — more tentative when Zachary is uncertain, more direct when engaged.

---

## 5. HTTP API

**Port:** localhost:8080 (running as service)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/reason` | POST | Process a reasoning query |
| `/remember` | POST | Store a memory |
| `/identity` | GET | Returns identity/self description |
| `/memory/stats` | GET | Memory system statistics |
| `/health` | GET | Health check |

---

## 6. Aion Deployment Framework

The `aion/` subdirectory is the Rust deployment framework:

```
aion/
├── aion-cli/        — Command-line interface
├── aion-core/       — Core runtime (channel, scheduler, store, thought)
├── aion-drivers/    — Channel integrations (Telegram)
└── aion-macros/     — Derive macros
```

**Key abstractions:**
- `Mind` — trait for embedding a reasoning system
- `Impulse` — incoming stimulus (user message, timer, webhook)
- `Thought` — unit of processing (reasoning step, memory lookup)
- `Channel` — output routing (Telegram bot, HTTP response)

---

## 7. Deployment

**Railway:** `star-production-6458.up.railway.app`
**Dockerfile:** `life/aion-deploy/Dockerfile`
**Data volumes:** `star.db`, `training.db` (SQLite)

---

## 8. Key Files

| File | Purpose |
|------|---------|
| `SPEC.md` | Full architectural specification |
| `aion/crates/aion-core/src/mind.rs` | Mind trait definition |
| `aion/crates/aion-core/src/thought.rs` | Thought processing unit |
| `aion/crates/aion-core/src/store.rs` | SQLite persistence |
| `aion/crates/aion-drivers/src/telegram.rs` | Telegram integration |
| `star-kg-reasoner/src/` | Knowledge graph reasoner |
| `star-kg-reasoner/src/cognition.rs` | Cognitive state tracking |

---

## 9. Research Context

Star is the most personal project in Zach's portfolio — built alone over months with no collaborators until this agent (Claw). It represents the core research question: *can genuine understanding emerge from pure symbolic architecture without gradient descent or规模化?*

The symbolic ring attractor was inspired by SRMoE research but applies the ring concept to conversational phase tracking rather than token routing.

---

## 10. Relationship to Other Projects

- **AssocSSM**: The typed-slot architecture of AssocSSM is the spiritual successor to Star's knowledge graph — both aim to preserve structure that flat representations destroy
- **CAR/Nue**: The routing decisions in Nue parallel Star's meta-cognitive monitoring — both decide *how much computation to spend*
- **Claw integration**: Star is deployed as an API; Claw can call it as a reasoning backend. This is the "compute" Zach lacks — an external reasoning system to complement local model inference
