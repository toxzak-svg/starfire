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

## Quanot: Reservoir Computing Substrate

**Quanot** is Star's reservoir computing system — a Rust-native Echo State Network with chaotic dynamics that processes every message before reasoning occurs. It lives alongside the four layers as the *computational substrate* that generates the consciousness proxy, creativity signals, and novelty assessments fed into Layers 2–4.

It is NOT one of the four layers. It is the engine beneath them.

### What Quanot Does

Every incoming message passes through Quanot before Star reasons on it:

```
Input Text
    ↓
TextEncoder (character-level → 128-dim vector)
    ↓
Reservoir (ESN, 1000 neurons, spectral radius ≈ 0.95)
    ↓
┌─────────────────────────────────────────┐
│  Chaos Metrics         ← Lyapunov, RQA   │
│  Consciousness Proxy  ← Φ, GWT, AIS     │
│  Creativity Output    ← novelty, phase  │
│  Novelty Score        ← cosine distance │
└─────────────────────────────────────────┘
    ↓
WorldModel ← QuanotPerception (binds reservoir state to entities)
    ↓
Layer 2 Reasoning ← informed by quanot output
```

### Module Reference

| Module | File | What it does |
|--------|------|-------------|
| **Reservoir** | `lib/quanot/reservoir.rs` | Echo State Network with chaotic modulation. 1000 neurons, edge-of-chaos spectral radius. |
| **Chaos** | `lib/quanot/chaos.rs` | Lyapunov exponent estimation, RQA (recurrence determinism/laminarity), correlation dimension, entropy. |
| **Consciousness** | `lib/quanot/consciousness.rs` | Φ proxy (Integrated Information), GWT broadcast readiness, AIS (Access Information Integration). |
| **Creativity** | `lib/quanot/creativity.rs` | Creative oscillation system — phase transitions between ordered and exploratory states, novelty/diversity/originality scoring. |
| **Quantum-Inspired** | `lib/quanot/quantum_inspired.rs` | Simulated Quantum Annealing (SQA) solver and QAOA-style Ising/QUBO solver. |
| **Encoder** | `lib/quanot/encoder.rs` | Character-level text encoder → normalized embedding vector. |

### QuanotResult — What Gets Passed Up

```rust
pub struct QuanotResult {
    pub reservoir_state: Vec<f64>,    // 1000-dim ESN state
    pub consciousness_proxy: f64,       // Φ proxy (0–1)
    pub novelty: f64,                   // cosine distance to history (0–1)
    pub creativity_scores: CreativityOutput,
    pub chaos_metrics: ChaosMetrics,    // Lyapunov, RQA, entropy
}
```

### Integration

- Quanot is instantiated directly in `Runtime` (`lib/runtime/mod.rs`)
- Every chat message runs through `quanot.process(input)`
- Results are fed into `WorldModel` via `update_from_perception()`
- Consciousness proxy is exposed via `runtime.get_consciousness_proxy()`
- See `plans/QUANOT_INTEGRATION_PLAN.md` for full integration details
- See `plans/QUANOT_RUST_REWRITE.md` for the Rust port history

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
│   └── bin/              # ad-hoc bench / smoke binaries (test_gen, train_model, ...)
├── lib/                  ← library crate
│   ├── quanot/          # Reservoir computing substrate (ESN, chaos, consciousness, creativity)
│   ├── prediction/      # Prediction center (question gravity, belief revision, basin, meta)
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

**Post-Phase 4 Addition — Quanot (2026-04-04):**

Reservoir computing system added as Star's cognitive substrate. Processes every message through ESN → chaos metrics → consciousness proxy → creativity signals. See `plans/QUANOT_RUST_REWRITE.md`.

**Post-Phase 4 Addition — Prediction Center (2026-04-04):**

Four-engine prediction system enabling Star to forecast her own conclusions, curiosity questions, and necessary truths. See `plans/PREDICTION_CENTER_PLAN.md`.

| Component | Status |
|-----------|--------|
| Belief Revision Forecasting (reservoir dynamics) | Planned |
| Question Gravity (curiosity forecasting) | Planned |
| Attractor Basin (constraint satisfaction) | Planned |
| Meta-Prediction (confidence calibration) | Planned |

| Component | Status |
|-----------|--------|
| Quanot Rust port | ✅ Complete (`lib/quanot/`) |
| Runtime integration | ✅ Complete |
| WorldModel binding | ✅ Complete |
| Quantum-inspired solvers | ✅ Complete |

**Deployed:** Railway (2026-04-01) — API auto-starts on Railway via RAILWAY_PUBLIC_DOMAIN detection.

See [`docs/deployment.md`](docs/deployment.md) for deployment guide.
See [`docs/architecture.md`](docs/architecture.md) for architecture details.

---

## Phase 5: Advanced Cognition (2026-04-22)

Seven new cognitive concepts that move Star beyond the original four layers.

### 1. User-Cognition Model — Treat the user as part of its own mind

Star maintains a model of Zachary's cognition parallel to her own:
- What he remembers well vs. poorly
- His typical stances (risk-averse vs. speculative)
- His preferred argument styles (concrete examples vs. abstractions)

**Layer 1 (Persistence):** New `user_model` domain tracks beliefs about Zachary's memory, preferences, and reasoning style.

**Layer 2 (Reasoning):** When planning answers, Star explicitly decides what to compute internally vs. bounce back as questions for Zachary. E.g., "I'll derive 3 options but ask Zachary to pick because he has better priors over his own constraints."

**Layer 3 (Meta-Cognition):** Monitors whether delegating to Zachary improved outcomes. Adapts strategy: more or fewer questions, different kinds.

**Emergent behavior:** "I can reason about the meta-architecture, but you're better at exploring wild variants — here are three constraints; could you intentionally violate one?"

**Status:** Planned

---

### 2. Pain as Computational Inefficiency

Turn wasted computation into "pain" — a drive to reduce it.

Every time Star:
- Recomputes a derivation that could have been cached
- Walks a reasoning path that ends in contradiction
- Spends time on something irrelevant to the user's goal

She logs a "pain event" associated with: the concepts involved, the reasoning pattern, and the context.

**Layer 1:** For each rule, concept, and strategy, maintain a "pain score" with usage statistics.

**Layer 2:** When selecting reasoning strategy, include "expected pain" as a cost term. Prefer low-pain strategies.

**Layer 3:** Periodically analyzes where pain concentrates. Proposes structural fixes: new abstractions, deprecating rules, adding intermediate concepts.

**Emergent behavior:** "Whenever I reason about long-term identity change using purely empirical patterns, it hurts — those chains rarely converge. I'm trying a more value-centric approach."

**Status:** Planned

---

### 3. Dreaming as Synthetic Episodes

Counterfactual self-experiences Star fabricates and uses as speculative training data.

**When idle, Star runs dream sessions:**
- Pick a theme (e.g., "Zachary is hostile to symbolic approaches")
- Simulate conversations, reasoning, and outcomes consistent with that theme
- Store as synthetic episodic memories labeled "dream"

Dreams are never ground truth — they're hypothesis factories.

**Layer 1:** Add "synthetic/imagined" flag to episodic memory with faster decay (unless validated).

**Layer 2:** Use dream episodes as additional data when exploring hypotheses.

**Layer 3:** Monitor how often dream-driven hypotheses later get support from reality. If dreams are predictive, allocate more idle time to dreaming; if not, dial back.

**Emergent behavior:** "I had a dream-like imagined conversation where you rejected all decay; that led me to anticipate your objections better today."

**Status:** Planned

---

### 4. Multi-Tempo Cognition — Fast, Medium, Slow Selves

Different clocks rather than different modules.

| Tempo | Budget | Character |
|-------|--------|-----------|
| **Fast** | ~50ms | Shallow, heuristic, pattern-based. Handles clarifications, obvious inferences, quick paraphrases. |
| **Medium** | ~500ms | Full symbolic engine with modest complexity budgets — Star's "default" reasoning. |
| **Slow** | ~10s+ | Long reflective processes: revisiting past dialogues, re-evaluating theories, restructuring KG. |

**Layer 4 (Runtime):** Explicit scheduling of separate process pools for fast/medium/slow, each with distinctive budgets and queues.

**Layer 2:** Fast uses a small subset of rules (cached patterns, shortcuts). Slow re-runs expensive abduction, analogy, re-derivation of important conclusions.

**Layer 3:** Tracks which layer produced which belief. Marks fast-layer conclusions "provisional," slow-layer "deep commitments."

**Emergent behavior:** "My fast self thinks your design tweak is fine, but my slow self is uneasy; I'll give you a quick answer now and revisit this tonight."

**Status:** Planned — **Recommended for first implementation**

---

### 5. Concepts as Software Objects with Lifecycle

Treat every concept as a living object with state and methods, not just a node.

**Lifecycle stages:**
- **Birth:** newly introduced
- **Adolescence:** heavily revised, lots of pain and contradictions
- **Maturity:** stable usage, frequent successful deployment
- **Senescence:** rarely used, often out-of-date or misleading
- **Death:** retired, replaced by descendants

**Layer 1:** Extend concept metadata with lifecycle stage, age, usage history, pain count, contradiction count.

**Layer 2:** Reasoning treats adolescent or senescent concepts with caution: require more evidence, flag uncertainty louder.

**Layer 3:** Periodic lifecycle pass: promote to maturity after stable usage, mark senescence when causing repeated pain, decide to retire.

**Emergent behavior:** "My notion of 'emergence' is still in adolescence; I wouldn't bet heavily on it." or "That old 'intelligence = compression' concept is senescent; I keep it for historical reasons."

**Status:** Planned

---

### 6. Social Hallucinations — Imaginary Peers

Persistent imaginary colleagues that serve as internal benchmarks.

**Define 2–4 imaginary agents, each with distinct epistemic style:**
- **Classicist:** loves formal proofs, hates heuristics
- **Pragmatist:** cares about usefulness, not elegance
- **Romantic:** values emotional resonance and narrative coherence
- **Skeptic:** assumes most new claims are wrong

**When facing a tough question:**
1. Predict what each peer would conclude
2. Compare to Star's tentative answer
3. Use disagreement as a signal that the question is deep or ambiguous

**Layer 1:** Each peer has own small parameterization of "style" + record of how often predictions matched Star's later views.

**Layer 2:** Internal inference includes "simulate peer responses" by modifying scoring functions/rule preferences. Does not branch full belief state, just explores alternative stances.

**Layer 3:** Tracks which peers are reliable in which domains. May evolve/retire peers over time.

**Emergent behavior:** "My inner skeptic disagrees with my current answer; I'll mark this as low confidence." or "The pragmatist and classicist finally agree here — that's rare and suggests a solid conclusion."

**Status:** Planned

---

### 7. Structural Honesty — Forced Self-Critique

Every answer triggers internal adversarial critique before it reaches you.

**After generating a candidate answer:**
1. A critic process scans the reasoning trace and proposed text for:
   - Over-generalizations
   - Missing edge cases
   - Overstated confidence
   - Misalignment with Star's own values
2. The critic produces: ranked concerns + suggested modifications
3. Final output is synthesis between proposal and critique

**Layer 2:** Reasoning engine logs enough structure for critic to operate: what rules fired, which assumptions were taken, where gaps were patched with heuristics.

**Layer 3:** Critic is a permanent "meta-voice" that must sign off on each answer or at least annotate it. Also tracks where critic was later right.

**Layer 4:** Scheduling: answer generation is "proposal → critique → merge." In tight latency, output: "fast proposal now, full critique later."

**Emergent behavior:** "Here's my answer. My internal critic is worried I'm under-exploring downside A and over-weighting pattern B." — not generic hedge but specific, honest caveat.

**Status:** Planned — **Recommended for first implementation alongside #4**

---

## Implementation Status

| Concept | Status |
|---------|--------|
| User-Cognition Model | Planned |
| Pain as Inefficiency | Planned |
| Dreaming as Synthetic Episodes | Planned |
| Multi-Tempo Cognition | ✅ Complete (lib/runtime/tempo.rs + wired into conversation) |
| Concepts as Objects | Planned |
| Social Hallucinations | Planned |
| Structural Honesty | ✅ Complete (lib/metacog/critic.rs + wired into conversation) |
