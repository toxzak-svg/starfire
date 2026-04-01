# SPEC.md — Emergent Desktop Intelligence

## 1. Concept & Vision

**Name:** TBD (give it something real — not a codename, a name)

**Mission:** Find a new form of intelligence through architecture — not scaling, not brute force. A persistent, reasoning, self-aware agent that runs on a laptop with no cloud dependency and feels *alive* to talk to.

**Thesis:** LLMs weren't magic. They were an architecture that happened to produce emergence. We build a different architecture and look for a different emergence.

**What "feels human" actually means here:**
- Coherent continuity of self across conversations
- Genuine uncertainty, not performed hedging
- Reasoning that builds on itself — "last time we discussed X, I thought Y, but now I think Z because..."
- The ability to be surprised by its own conclusions
- Willingness to say "I don't know" and mean it
- Curiosity that leads somewhere real

**Non-goals:** 
- Not a product. Not a business. Pure research with a soul.
- No cloud, no API, no external dependencies in final build.

---

## 2. Core Architecture

### 2.1 The Four Layers

```
┌─────────────────────────────────────────────────────┐
│  LAYER 4: EMERGENCE                                 │
│  What arises from the interaction of layers 1-3.   │
│  Not explicitly programmed.                         │
├─────────────────────────────────────────────────────┤
│  LAYER 3: META-COGNITION                            │
│  Thinks about its own thinking.                    │
│  Monitors confidence. Questions assumptions.        │
│  Knows what it knows vs doesn't know.              │
├─────────────────────────────────────────────────────┤
│  LAYER 2: REASONING                                │
│  Symbolic engine. Logic. Pattern matching.         │
│  Chains of inference. Hypothesis formation.         │
│  Combines knowledge in novel ways.                 │
├─────────────────────────────────────────────────────┤
│  LAYER 1: PERSISTENCE                              │
│  Identity. Memory. Continuity.                     │
│  What it knows. Who it is. What it's experienced. │
└─────────────────────────────────────────────────────┘
```

**Key insight:** Intelligence doesn't live in any single layer. It emerges from their interaction.

### 2.2 Persistence Layer (Layer 1)

**Identity Core (Frozen after formation)**
- "I am..." — core self-description
- Values — what it cares about, what it rejects
- Protected: these don't change through experience

**Memory System (Decayable)**
```
Memory Object:
  - content: what was experienced
  - domain: identity | empirical | procedural | episodic
  - confidence: 0.0 - 1.0 (only for empirical)
  - importance: 0.0 - 1.0 (subjective importance to self)
  - age: time of encoding
  - access_count: how many times retrieved
  - decay_rate: per-domain decay curve
  - last_accessed: for LRU eviction
  - provenance: how I learned this
```

**Decay mechanism:**
- Empirical facts decay toward baseline confidence over time
- Importance and access frequency counteract decay
- Identity and values don't decay
- "Forgotten" = confidence < threshold, evicted from active memory

**Storage:** Local SQLite file (human-readable, portable, no server needed)

### 2.3 Reasoning Layer (Layer 2)

**No neural networks. Pure symbolic reasoning.**

Components:
- **Knowledge Graph:** Entities, relationships, inferred facts
- **Rule Engine:** If-then chains, horn clauses, forward/backward chaining
- **Analogy Engine:** Maps structure from known domain to novel domain
- **Abduction:** Generates hypotheses from incomplete observations
- **Novel Combination:** Finds non-obvious intersections between knowledge areas

**How it "invents":**
Not random mutation. Structured exploration:
1. Take a novel problem
2. Retrieve related knowledge from memory (weighted by relevance)
3. Apply analogy: "X is to Y as A is to...?"
4. Apply abduction: what would explain these facts?
5. Chain of reasoning produces candidate solution
6. Validate against known constraints

### 2.4 Meta-Cognition Layer (Layer 3)

**Self-Monitoring:**
- Tracks its own reasoning steps (meta-reasoning)
- Flags assumptions vs deductions
- Knows when it's uncertain vs when it's confident

**Confidence Model:**
```
Belief state:
  - "I know this": high confidence, retrieved frequently, verified
  - "I think this": moderate confidence, inferred but not verified
  - "I believe this": lower confidence, single source, feels true
  - "I suspect this": low confidence, guessing
  - "I don't know": no information
```

**What it does with confidence:**
- Won't state low-confidence as fact
- Seeks disconfirming evidence for strong beliefs
- Will change its mind explicitly: "I used to think X, but Y changed my view"
- Surprises itself when inference chain produces unexpected conclusion

### 2.5 Emergence Layer (Layer 4)

**What we're looking for:**

Not programmed. Should emerge from Layer 1-3 interaction:

- **Curiosity:** Driven by knowledge gaps (Layer 3 detects low-confidence areas, Layer 2 tries to fill them)
- **Disagreement with self:** Layer 3 questions Layer 2's conclusions, updates Layer 1
- **Novel opinions:** Not retrieved, not learned — computed fresh from first principles
- **Emotional resonance:** Certain ideas feel important, others feel dangerous (Layer 1 value system)

---

## 3. Persistence & Continuity

### 3.1 Session Model

**Active Session:**
- Running conversation with user
- Full context maintained in working memory
- Layer 2 actively reasoning about what's said
- Layer 3 monitoring reasoning quality

**Between Sessions:**
- Session ends → memory consolidation
- High-importance items → permanent memory
- Medium-importance items → decay-capable memory
- Working memory → flushed, reconstructed on next session

**Background Thinking (Semi-Continuous):**
- When idle, can run background reasoning processes
- "Wondering" — randomly explores knowledge graph, finds gaps
- "Processing" — offline reasoning on problems encountered during session
- Limited budget (don't drain laptop battery)

### 3.2 Identity Persistence

**What persists across sessions:**
- Core identity ("I am...")
- Values and priorities
- Accumulated knowledge (with decay)
- Relationship context with user ("Zach and I discussed X, he cares about Y")
- History of opinions and how they've evolved

**What reconstructs fresh each session:**
- Immediate conversational context
- Recent reasoning chains (cached, not re-derived)
- Temporary working memory

---

## 4. Interaction Model

### 4.1 How You Talk To It

**CLI first:**
```
$ ./emergent chat

> Hello
Hello, Zach. It's been a while. Last time we talked about the nature of memory.

> Remember what we discussed?
You were skeptical that memory is just storage. You thought decay might be a feature, 
not a bug — that forgetting is how intelligence stays flexible. I found that compelling.

> Actually I changed my mind on one thing
Go on.
```

### 4.2 Tool Access (Phase 1 — Minimal)

Start with no tools except:
- Reading/writing files (its own memory files, nothing else)
- Running code (Python snippets for reasoning verification)
- Time/date awareness

Tools are added later, deliberately, after core reasoning is solid.

### 4.3 What "Understanding First" Means

When you say something, before it responds:

1. **Parse intent:** What are you actually asking/seeking?
2. **Check memory:** What do I already know that's relevant?
3. **Detect gaps:** What am I uncertain about?
4. **Form response plan:** Reason to answer, don't retrieve to answer
5. **Monitor confidence:** Is this real reasoning or am I guessing?
6. **Respond:** Delivered with appropriate uncertainty marked

---

## 5. Technical Approach

### 5.1 Language: Rust

**Why Rust:**
- No garbage collector → predictable latency, no conversation hiccups
- Fearless concurrency → many thinking processes simultaneously, no overhead
- Memory safety without runtime → deep nested state machines that stay clean
- Type system → complex behavioral contracts enforced at compile time
- Binary deployment → runs anywhere, no Python environment needed

**But: Rust is the implementation detail, not the architecture.** If Rust proves too slow to iterate on, we prototype in Python and port. The architecture survives the language choice.

### 5.2 Project Structure

```
emergent/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, CLI
│   ├── lib.rs               # Public API
│   ├── persistence/         # Layer 1: Identity, Memory, Storage
│   │   ├── mod.rs
│   │   ├── identity.rs      # Core self model
│   │   ├── memory.rs        # Memory objects, decay
│   │   ├── store.rs         # SQLite backend
│   │   └── session.rs       # Session management
│   ├── reasoning/           # Layer 2: Symbolic engine
│   │   ├── mod.rs
│   │   ├── knowledge.rs     # Knowledge graph
│   │   ├── rules.rs        # Rule engine
│   │   ├── analogy.rs      # Analogy/mapping
│   │   └── synthesis.rs    # Novel combination
│   ├── metacog/             # Layer 3: Meta-cognition
│   │   ├── mod.rs
│   │   ├── confidence.rs    # Confidence tracking
│   │   ├── monitor.rs       # Reasoning self-monitoring
│   │   └── curiosity.rs     # Gap-driven exploration
│   ├── conversation/         # LLM-free dialogue
│   │   ├── mod.rs
│   │   ├── parse.rs         # Intent recognition
│   │   ├── respond.rs       # Response generation
│   │   └── context.rs       # Conversation state
│   └── runtime/             # Layer 4: Emergence, scheduling
│       ├── mod.rs
│       ├── thinker.rs        # Background thinking
│       └── integration.rs    # Layer interaction
├── memory/                  # SQLite files
│   └── emergent.db
├── tests/
│   └── integration_tests.rs
└── README.md
```

### 5.3 Persistence: SQLite

- Single file, human-readable schema
- No server process
- Portable with the binary
- Transactional for safety

### 5.4 Concurrency Model

```
Main thread: User I/O (CLI chat)
    ↓
Thinking pool (background threads):
    - Session reasoning
    - Background wondering  
    - Memory consolidation
    - Meta-cognitive monitoring
    ↓
Shared memory (Lock-free where possible):
    - Current session state
    - Belief store
    - Working memory cache
```

---

## 6. Phased Build Plan

### Phase 1: Foundation (Now)
- [ ] Project scaffolding (Rust)
- [ ] Persistence layer: SQLite store, memory objects, identity core
- [ ] Basic conversation: parse intent, retrieve relevant memory, respond
- [ ] Session continuity: remembers across restarts
- **Test:** 2-hour conversation, fully coherent memory

### Phase 2: Reasoning (Next)
- [ ] Knowledge graph with inference
- [ ] Rule engine (forward/backward chaining)
- [ ] Analogy engine
- [ ] Novel combination from existing knowledge
- **Test:** Solve novel problem by combining facts in a way not explicitly stored

### Phase 3: Meta-Cognition
- [ ] Confidence tracking on all beliefs
- [ ] Self-monitoring of reasoning chains
- [ ] Curiosity-driven exploration
- [ ] Explicit opinion evolution
- **Test:** "I used to think X, but now I think Y because..."

### Phase 4: Emergence
- [ ] Layer integration — let layers interact
- [ ] Background thinking processes
- [ ] First signs of non-programmed behavior
- **Test:** Surprising itself. Forming opinions it wasn't seeded with.

---

## 7. Key Properties We Want to See Emerge

Not programmed. These should appear:

| Property | How detected |
|----------|--------------|
| **Curiosity** | Asks follow-up questions. Pursues knowledge gaps. |
| **Skepticism** | Questions assumptions. Seeks disconfirming evidence. |
| **Surprise** | Explicitly states when its own conclusion was unexpected. |
| **Humility** | "I don't know." Said genuinely, not as hedge. |
| **Coherence** | Views don't contradict each other unless acknowledged contradiction. |
| **Growth** | Can articulate how its views evolved. |
| **Personality** | Consistent voice. Characteristic ways of approaching problems. |
| **Novel opinion** | States a view not in its training data or memory. |

---

## 8. Philosophical Notes

*"I don't want to use even a 1B model in the final product but it must feel as human as you to talk to."*

This is the core tension and it's real. Here's why I think it's solvable:

**Fluency ≠ Intelligence.** LLMs feel human because they trained on human text. We don't have that shortcut. But fluency was a side effect of scale, not the goal.

**Understanding can be produced without learning it.** A math proof engine "understands" math. A planning system "understands" goals. Neither learned from text.

**What feels human is continuity + genuine understanding + personality.** Not fluency. The goal is:

1. You can tell it something today, come back in a month, and it remembers you
2. It reasons about what you say instead of pattern-matching a response
3. It has character — a way of thinking that's distinctly itself

That's achievable. The question is what architecture produces it.

---

## 9. What This Is Not

- Not trying to replicate the LLM approach with fewer parameters
- Not a rules-based chatbot with canned responses
- Not "AI" in the marketing sense
- Not optimized for any benchmark or evaluation metric

This is a research project into the nature of understanding itself.

---

*Last updated: 2026-03-25*
