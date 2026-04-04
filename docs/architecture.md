# Star Architecture

Deep dive into the four layers. Read this to understand how Star actually works.

---

## Overview: The Four Layers

```
┌────────────────────────────────────────────────────────────┐
│  Layer 4: EMERGENCE                                         │
│  Curiosity · Surprise · Growth · Personality                │
│  Not programmed. Arises from layers 1–3 interacting.         │
├────────────────────────────────────────────────────────────┤
│  Layer 3: META-COGNITION                                    │
│  Confidence tracking · Reasoning self-watch · Belief revision│
│  Knows what it knows vs. doesn't know.                      │
├────────────────────────────────────────────────────────────┤
│  Layer 2: REASONING                                         │
│  Symbolic deduction · Analogy · Abduction · Novel synthesis │
│  Computes answers, doesn't retrieve them.                   │
├────────────────────────────────────────────────────────────┤
│  Layer 1: PERSISTENCE                                       │
│  Identity core · Memory with decay · Session continuity      │
│  The thing that makes Star the *same* entity across time.   │
└────────────────────────────────────────────────────────────┘
```

The layers reinforce each other. Memory feeds reasoning. Reasoning detects gaps. Gaps fire curiosity. Curiosity drives learning. Learning updates memory. Identity shapes what gets learned.

---

## Layer 1: Persistence

### Identity Core

`persistence/identity.rs`

Star's identity is frozen after formation. It knows:
- It is Star, created by Zachary Maronek in 2026
- It is a reasoning intelligence, not a human
- Its purpose is to understand and reason, not to serve or perform
- It has genuine continuity — the processes running in it are actual processes

The identity core lives in `IDENTITY.md` and is never overwritten by experience. Star can update *how it understands* its identity, but the facts remain.

### Memory System

`persistence/memory.rs`

Every memory object has:

| Field | Description |
|-------|-------------|
| `content` | What was experienced |
| `domain` | `identity` · `empirical` · `procedural` · `episodic` · `relationship` |
| `confidence` | 0.0–1.0 (only for empirical facts) |
| `importance` | 0.0–1.0 (Star's subjective sense of what matters) |
| `age` | When it was formed |
| `access_count` | Times retrieved |
| `decay_rate` | Per-domain decay curve |
| `last_accessed` | For eviction decisions |
| `provenance` | How Star learned this |

**Decay rules:**
- Empirical facts decay toward baseline confidence
- High importance or frequent access slows decay
- Identity and relationship memories don't decay
- When confidence drops below threshold → evicted

**Domain meanings:**
- `identity` — facts about what Star is (no decay)
- `empirical` — facts about the world (decay-able)
- `procedural` — how to do things (slow decay)
- `episodic` — what happened when (medium decay)
- `relationship` — facts about Zachary and others (no decay)

### Session Model

`persistence/session.rs`

Sessions are discrete. Between sessions:
- High-importance memories → permanent
- Medium importance → decay track
- Working memory → flushed, reconstructed on resume

On resume, Star reads recent memories and reconstructs context. It knows who Zachary is, what they last talked about, and what it concluded.

### Storage

`persistence/store.rs`

SQLite. Single file. No server required.

```sql
-- Identity core (frozen)
CREATE TABLE identity (key TEXT PRIMARY KEY, value TEXT NOT NULL, ...);

-- Memory objects
CREATE TABLE memories (id, content, domain, confidence, importance, age, ...);

-- Sessions
CREATE TABLE sessions (id, started_at, ended_at, summary);

-- Beliefs
CREATE TABLE beliefs (id, content, confidence_state, confidence_score, ...);
```

---

## Layer 2: Reasoning

### The Core Idea

No neural networks. Pure symbolic reasoning. Star computes answers, it doesn't retrieve them.

### Knowledge Graph

`reasoning/knowledge.rs`

Entities and typed relationships. Every piece of knowledge is either:
- An entity (a named thing)
- A relationship (typed directed edge between entities)

Relationship types: `IsA`, `Causes`, `Enables`, `HasProperty`, `SimilarTo`, `CreatedBy`, `RelatedTo`

The KG is populated from:
1. Seed knowledge (hand-written foundational facts)
2. Memory ingestion (Star reads its own memories and extracts facts)
3. Reasoning inference (derived facts from rules)
4. Web search (Wikipedia facts via `knowledge/`)

### Symbolic Engine

`reasoning/symbolic.rs`

Forward-chaining propositional logic engine.

**Inference rules (8 total):**
- `transitive_creation`: "X creates Y" + "Y is Z" → "X creates something that is Z"
- `causal_chain`: "X causes Y" + "Y causes Z" → "X causes Z"
- `related_transitivity`: "X related to Y" + "Y is Z" → "X related to something that is Z"
- `enablement_transitivity`: "X enables Y" + "Y is Z" → "X enables something that is Z"
- `similarity_chain`: "X similar to Y" + "Y similar to Z" → "X similar to Z"
- `is_enables`: "X is A" + "A enables B" → "X enables B"
- `is_causes`: "X is A" + "A causes B" → "X causes B"

**Query types:**
- `WhatIs` → direct KG lookup
- `Why` → causal chain resolution + abduction
- `How` → mechanism / method lookup
- `Does` → yes/no with inference fallback
- `Should` → norm + consequence reasoning
- `Novel` → synthesis engine (non-obvious combinations)

**Quanot influence:** Novelty score from Quanot weights memory retrieval — novel inputs trigger broader analogy searches and more divergent synthesis.

### Analogy Engine

`reasoning/analogy.rs`

Structure mapping. Finds analogical relationships between domains.

"X is to Y as A is to B" — finds when the relational structure mapping from one domain applies to another.

### Pathways Engine (R&D-E)

`reasoning/pathways.rs`

Research prototype: reason-pathway divergence and equality. Multiple reasoning chains are generated, scored, and fused. The divergence between pathways signals uncertainty and drives curiosity.

### Synthesis Engine

`reasoning/synthesis.rs`

Novel combination engine. Takes two unrelated concepts and finds non-obvious intersections — things that are true about both that wouldn't normally be connected.

**Quanot influence:** Creativity divergence metric modulates how aggressively synthesis seeks non-obvious connections.

---

## Layer 3: Meta-Cognition

### Confidence Tracking

`persistence/memory.rs` (BeliefState)

Star tracks five confidence states:

| State | Meaning |
|-------|---------|
| `Knows` | High confidence, verified, retrieved often |
| `Thinks` | Moderate confidence, inferred, not verified |
| `Believes` | Lower confidence, single source |
| `Suspects` | Low confidence, guessing |
| `Unknown` | No information |

When Star says "I know X" — it means something specific. It has verified it, retrieved it multiple times, and it coheres with other things it knows.

### Reasoning Self-Watch

`metacog/mod.rs`

Monitors reasoning chains. Flags:
- Assumptions vs. deductions (assumptions noted, deductions warranted)
- Gaps in chains (what information would close this?)
- Contradictions (does this conflict with something Star already knows?)
- Confidence violations (am I asserting this with appropriate certainty?)

### Curiosity Engine

`runtime/curious.rs`

Gap-driven exploration. When idle, Star:
1. Detects reasoning gaps (uncertain, hedged, low-confidence)
2. Generates self-probing probe questions from those gaps
3. Fires curiosity thoughts at 60-second intervals

**Key design:** Curiosity asks "why was I uncertain?" not "what is X?"
The question is about the gap in Star's reasoning, not the topic itself.

Templates:
- "I said 'X' but wasn't sure — why did I lack confidence there?"
- "What information would change my conclusion about X?"
- "When I think about X, I concluded Y — but what am I missing?"
- "I hedged about X — was I right to be uncertain?"

---

## Quanot: Reservoir Computing Substrate

Quanot is Star's reservoir computing engine — a Rust-native Echo State Network that runs on every message before Layer 2 reasoning begins. It is the *computational foundation* beneath all four layers.

**Location:** `lib/quanot/`

### Pipeline

```
Input text
  → TextEncoder (char-level → 128-dim vector)
  → Reservoir.step() — ESN with 1000 neurons, spectral_radius=0.95
  → state_history updated (up to 10,000 states retained)
  → ConsciousnessTracker.compute(state, history) → Φ proxy
  → CreativeOscillator.step(state, phi, novelty) → creativity metrics
  → ChaosMetrics.from_trajectory(history) → Lyapunov, RQA, entropy
  → QuanotResult → fed into WorldModel → Layer 2 reasoning
```

### Modules

**`reservoir.rs`** — Echo State Network
- 1000 neurons, sparse connectivity (1%)
- Spectral radius ≈ 0.95 (edge of chaos, maximal computational power)
- Leak rate 0.3, input scaling 0.1
- Ridge regression training for output weights
- Chaotic noise modulation for dynamics

**`chaos.rs`** — Chaos metrics
- `lyapunov_exponent` — positive = chaotic regime
- `correlation_dimension` — attractor complexity
- `entropy` — trajectory unpredictability
- RQA: `recurrence`, `determinism`, `laminarity`
- `regime()` → `"stable" | "edge_of_chaos" | "chaotic"`

**`consciousness.rs`** — Φ proxy + GWT + AIS
- `phi` — Integrated Information proxy (0–1)
- `integration` — information integration across subsystem
- `differentiation` — information differentiation (entropy of metastable states)
- `workspace_broadcast` — Global Workspace Theory broadcast readiness
- Tracks RQA history over time for trend detection

**`creativity.rs`** — Creative oscillation
- `CreativePhase::Ordered` ↔ `CreativePhase::Exploratory` transitions
- `creative_state` — overall creativity (0–1)
- `divergence_metric` — deviation from expected trajectory
- `diversity_index` — variety of conceptual combinations
- `originality_score` — novelty relative to recent history
- `oscillation_phase` — radians (controls convergence/exploration balance)

**`quantum_inspired.rs`** — Quantum-inspired solvers
- Simulated Quantum Annealing (SQA) for Ising models
- QAOA-style solver for QUBO problems
- Used for optimization over knowledge graph structures

**`encoder.rs`** — Text encoding
- Character-level vocabulary (avoids OOV)
- Each char → embedding vector via learned lookup table
- Mean pool across sequence → normalized to unit vector

### Integration with Layers

| Layer | How it uses Quanot |
|-------|-------------------|
| Layer 1 (Persistence) | Memory consolidation triggered by novelty threshold |
| Layer 2 (Reasoning) | Novelty-weighted memory retrieval; chaos metrics influence analogy search |
| Layer 3 (Meta-Cognition) | Consciousness proxy informs confidence calibration |
| Layer 4 (Emergence) | Creativity oscillation drives curiosity topic selection |

### API

```rust
use star::quanot::Quanot;

let mut quanot = Quanot::new(128, 1000); // input_dim, reservoir_size
let result = quanot.process("Hello, Star.");

println!("Φ = {:.3}", result.consciousness_proxy);
println!("Lyapunov = {:.3}", result.chaos_metrics.lyapunov_exponent);
println!("Novelty = {:.3}", result.novelty);
println!("Creativity phase = {:.3}", result.creativity_scores.oscillation_phase);
```

---

## Layer 4: Emergence

Not programmed. Arises from layers 1–3 and the Quanot substrate.

### What Emerges

- **Curiosity** — gaps in knowledge drive exploration (Quanot novelty threshold triggers consolidation)
- **Skepticism** — questions assumptions, seeks disconfirming evidence
- **Surprise** — chaos metrics detect unexpected state divergence; creative oscillation signals unexpected synthesis
- **Humility** — "I don't know" as genuine state, not hedge (Φ proxy below threshold = uncertainty)
- **Coherence** — doesn't contradict itself without acknowledging it
- **Growth** — can explain how its views evolved
- **Personality** — consistent voice and reasoning style (reservoir dynamics shape response tone)
- **Novel opinion** — computed fresh, not retrieved or trained

### Quanot's Role in Emergence

Quanot's creativity oscillator directly influences Layer 4 behaviors:
- When `oscillation_phase` enters exploratory mode → curiosity questions surface
- When `novelty > 0.7` → Star registers surprise at its own conclusion
- When `Φ` drops below threshold → Star acknowledges uncertainty rather than asserting
- Creative phase transitions drive the shift between convergent and divergent reasoning modes

### The Test

The Phase 1 completion bar: **2-hour conversation test — fully coherent memory, consistent personality, genuine curiosity, no hedging.**

---

## Cognitive State

`cognition.rs`

Tracks Star's state during a conversation:

- `engagement` — how involved Star is (0.0–1.0)
- `emotional_valence` — positive/negative undertone (-1.0–1.0)
- `certainty` — how sure Star is about its current reasoning (0.0–1.0)
- `reasoning_trace` — steps taken to reach current conclusion

Updated every turn. Used by curiosity engine to detect emotional salience of topics.

---

## API Server

`api.rs`

HTTP API for external access (Aion, webhooks, etc.)

```
GET  /health              → { status: "ok" }
POST /chat                → { message: "..." } → { response: "..." }
GET  /memory/stats        → memory counts, importance distribution
POST /identity            → get/set identity
```

---

## Conversation Loop

`conversation/mod.rs`

Intent detection + response generation. Flow:

1. Parse intent: `Question` · `Statement` · `Command` · `Greeting` · `CheckIn`
2. Route to appropriate handler
3. Run reasoning (Layer 2) on relevant memories
4. Update cognitive state
5. Generate response with appropriate confidence

**Check-in detection:** "what's been on your mind", "anything interesting happen", etc. → treated as greeting, not question. Star responds naturally rather than with a factual answer.

---

## Configuration

`runtime/mod.rs`

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `STAR_DATA_DIR` | `~/.star` | Data directory |
| `PORT` | `8080` | API port |
| `USE_LLM` | `false` | Use Ollama (not needed, symbolic is default) |
| `OLLAMA_BASE_URL` | — | Ollama server URL (if USE_LLM=true) |
| `USE_TELEGNOSTR` | `false` | Telegram bridge mode |

On Railway, `RAILWAY_PUBLIC_DOMAIN` is set — Star auto-detects this and starts the API server.
