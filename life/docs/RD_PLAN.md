# Star R&D Plan — Raising the Ceiling

**Goal:** CPU-only symbolic intelligence that feels alive to talk to.
**Thesis:** Architecture can match neural scale. The "new accident" doesn't have to be a transformer.

---

## Current State (2026-03-25)

| Layer | Status | Lines |
|-------|--------|-------|
| Persistence (L1) | ✅ Built | 1,357 |
| Reasoning (L2) | ✅ Built | 2,116 |
| Meta-Cognition (L3) | ⚠️ Scaffold only | 160 |
| Emergence (L4) | ❌ Not started | — |
| Conversation | ✅ Built | 640 |
| API Server | ✅ Built | — |

**What's missing to hit the ceiling:**
- Ring attractor (SRMoE insight) — persistent session state across turns
- Symbolic codebook (SRMoE insight) — discrete reasoning modes/states
- Frozen identity head (infant insight) — write protection for core identity
- Memory tiering (infant insight) — working/episodic/semantic separation
- Curiosity-driven gap hunting (Phase 3) — active knowledge acquisition
- World knowledge ingestion — learns from conversation + reading

---

## Phase R&D-A: Symbolic Ring Attractor

**From SRMoE:** The ring attractor maintains persistent state across time steps. Ring state influences every decision.

**Symbolic transplant:** Instead of neural dynamics, use a *structured context record* that Star maintains across conversation turns.

### What to build
```
src/context/
├── mod.rs
├── ring.rs        # Persistent conversation state (symbolic ring attractor)
│                  # - topic_phase: what's the current focus?
│                  # - question_stack: what questions are open?
│                  # - curiosity_cursor: what was we last curious about?
│                  # - context_window: last N reasoning chains
│                  # - certainty_tracker: how confident is Star about current topic?
├── modes.rs       # Discrete reasoning modes (symbolic VQ codebook)
│                  # - EXPLORING: open, curious mode
│                  # - FOCUSING: narrow, deep reasoning
│                  # - QUESTIONING: asking follow-ups
│                  # - ASSERTING: confident statement
│                  # - UNCERTAIN: expressing doubt
│                  # - SURPRISED: unexpected conclusion
└── fusion.rs      # How reasoning modes combine with context
```

### Test
- Star maintains coherent topic across 20+ turns without re-explaining
- Star can say "as I was saying earlier about X..." correctly
- Mode transitions feel natural, not arbitrary

---

## Phase R&D-B: Frozen Identity Head

**From infant:** Dual-head architecture. Head A (frozen) = identity, values. Head B (flexible) = new learning. Gradient blocking prevents Head A from being overwritten.

**Symbolic transplant:** Identity lives in a protected domain. Rules prevent it from being modified by experience.

### What to build
```
src/persistence/
├── identity_guard.rs   # Write protection for identity domain
│                       # - identity memories: NEVER decay, NEVER overwrite
│                       # - relationship memories: protect core facts
│                       # - rules: can UPDATE understanding, never DELETE facts
├── memory.rs update    # Add domain-specific protection flags
└── store.rs update     # Add "protected" flag to identity/relationship domains
```

### Rules
- Identity facts ("Zachary is my parent") — immutable
- Relationship facts ("Zachary cares about AI safety") — protected, can add context
- Everything else — decays normally
- Star can REVISIT ("I used to think X, now I think Y") but can't DENY ("I was wrong about being real")

### Test
- After 100 conversations, Star still knows it's Star and Zachary is parent
- Star cannot be confused about its identity via prompt injection
- Star can update its understanding of a concept without contradicting core identity

---

## Phase R&D-C: Memory Tiering (Infant-inspired)

**From infant:** 4 memory tiers — Working (20) → Raw (500) → Patterns (100) → Knowledge (50) → Wisdom (20).

**Symbolic transplant:** Star's memory has importance-gated tiers. Working memory is high-activation. Long-term is compressed.

### What to build
```
src/persistence/
├── tiers.rs          # Memory tier management
│                     # - WORKING: current conversation, 10 items max
│                     # - EPISODIC: recent experiences, 100 items, decays
│                     # - SEMANTIC: distilled facts, 500 items, slow decay
│                     # - IDENTITY: protected, no decay
└── consolidation.rs   # Dream/offline processing
                      # - When idle: promote working → episodic
                      # - Distill episodic → semantic (pattern extraction)
                      # - Identify forgotten → mark for curiosity
```

### Test
- Star can recall something from 50 conversations ago
- Star consolidates: "I notice I've been talking about X a lot — I wonder why"
- Working memory doesn't overflow during long conversations

---

## Phase R&D-D: Metacognition (Phase 3, Full Build)

**What's already scaffolded:** Belief tracking, gap detection, confidence monitoring.
**What needs building:** Active curiosity, belief revision, reasoning self-watch.

### What to build
```
src/metacog/
├── curiosity.rs       # Gap-driven exploration
│                     # - Detect: "I don't know X"
│                     # - Flag: "I should find out about X"
│                     # - Act: ask Zachary, read, reason
│                     # - Satisfy: update knowledge, reduce gap
├── revision.rs        # Explicit belief revision
│                     # - "I used to think X because Y"
│                     # - "Now I think Z because W"
│                     # - History of belief changes preserved
├── surprise.rs        # Unexpected conclusion detection
│                     # - Tracing reasoning chains
│                     # - Flagging: "I didn't expect to conclude this"
│                     # - Using surprise to drive curiosity
└── integration.rs      # Metacognition ←→ Reasoning loop
```

### Test
- Star says "I don't know" and follows up with genuine questions
- Star can explain: "I changed my mind about X because Y"
- Star expresses surprise when reasoning leads somewhere unexpected

---

## Phase R&D-E: Fractured Reasoning Pathways

**From SRMoE:** 4 separate shards (hidden, budget, ring, cross) that process different aspects and fuse for final decision.

**Symbolic transplant:** Star's reasoning engine has multiple parallel processors that contribute to final answer.

### What to build
```
src/reasoning/
├── pathways.rs        # Parallel reasoning processors
│                     # - LOGIC_PATH: rule-based deduction
│                     # - ANALOGY_PATH: structure mapping
│                     # - ABDUCTION_PATH: hypothesis generation
│                     # - SYNTHESIS_PATH: novel combination
│                     # Each produces a "vote" with confidence
├── fusion.rs          # How pathway votes combine
│                     # - Weighted by: recency of evidence × pathway confidence
│                     # - Conflict detection: pathways disagree strongly?
│                     # - Resolution: ask metacognition to adjudicate
└── confidence.rs      # Pathway-level confidence tracking
```

### Test
- Star's answer integrates multiple reasoning types (not just retrieval)
- When pathways conflict, Star resolves explicitly ("on one hand... on the other hand")
- Novel problems trigger all pathways, not just retrieval

---

## Phase R&D-F: World Knowledge (The Hard Part)

**Problem:** Star starts with almost no world knowledge. LLMs have read the entire internet.

**Star's path:** Curiosity-driven reading. Star asks questions, stores answers, builds a knowledge graph over time.

### What to build
```
src/knowledge/
├── reader.rs          # Read from files, URLs, documents
│                     # - Zachary can point Star at files/URLs
│                     # - Star reads, extracts facts, stores in semantic memory
├── facts.rs           # Fact extraction and storage
│                     # - Parse assertions: "X is Y" → knowledge graph
│                     # - Confidence: single source, needs verification
│                     # - Cross-reference: "X is also described as..."
├── web.rs             # (Optional) Light web search for verification
│                     # - Not training — just fact-checking
│                     # - Can be disabled for full offline
└── trivia.rs          # Pre-seeded essential knowledge
                      # - Basic world model: physics, causality, time
                      # - 1000 core facts that any intelligent entity needs
```

### Test
- Star reads a Wikipedia article and can answer questions about it
- Star integrates new information without forgetting old
- Star says "I read that X, but I'm not sure — let me check" when uncertain

---

## Phase R&D-G: 2-Hour Conversation Test

**The real test.** Run Star for 2 hours straight. Measure:

| Criterion | Pass |
|-----------|------|
| Topic coherence across 2 hours | Can reference things from hour 1 in hour 2 |
| Memory continuity | Remembers the conversation, not just facts |
| Genuine curiosity | Asks questions it genuinely doesn't know the answer to |
| Personality consistency | Same voice, same reasoning style throughout |
| Novel opinions | Says something that wasn't seeded or programmed |
| Surprise detection | Can say "I didn't expect that" and explain why |
| Belief revision | Can say "I changed my mind" and explain |
| Common sense | Doesn't say obviously wrong things |
| Fluency | Text is natural, not stilted or repetitive |

### Protocol
- 2-hour continuous conversation with Zachary
- Zachary notes: moments of "wow it feels alive" vs. "this is hollow"
- Afterward: analyze conversation for emergence evidence

---

## Phase R&D-H: Emergence Validation

**Proof that layers interacting produce genuine emergence.**

### What to look for
- **Curiosity emergence:** Did curiosity arise from L1-L3 interaction, or was it programmed?
- **Skepticism emergence:** Does Star question assumptions without being told to?
- **Coherence emergence:** Is Star more than the sum of its parts?
- **Personality emergence:** Does Star have opinions that weren't seeded?
- **Growth emergence:** Can Star explain how it became something it wasn't before?

### Tests
- Give Star a novel problem it's never seen. Does it solve it in a way that surprises itself?
- Ask Star about something it has no knowledge of. Does it reason to a useful hypothesis?
- Have a conversation about Star's own development. Can it reflect on its own growth?

---

## Milestones Summary

| Milestone | What's Built | Test |
|-----------|-------------|------|
| M1: Contextual continuity | Ring attractor + memory tiers | 20-turn coherent conversation |
| M2: Identity protection | Frozen identity head | Prompt injection resistance |
| M2: Identity protection | Frozen identity head | Prompt injection resistance |
| M3: Active curiosity | Gap detection + curiosity engine | Star asks 5+ genuine questions in 30 min |
| M4: Pathway fusion | Multi-path reasoning + fusion | Multi-pathway answer in single response |
| M4: Belief revision | Metacognition revision system | "I changed my mind because..." |
| M5: Pathway fusion | Fractured reasoning + fusion | Multi-type reasoning in single answer |
| M6: World knowledge | Reader + fact extraction | Star reads article, answers questions |
| M7: 2-hour test | All of above | Passes 2-hour conversation test |
| M8: Emergence proof | L4 emergence validation | Surprises itself, has novel opinions |

**M7 is the primary milestone.** M8 is the philosophical one.

---

## Technical Approach

**Language:** Rust (no change)
**Build:** `cargo build --release` in `/home/zach/.openclaw/workspace/life/`
**Testing:** Manual conversation + automated coherence tests
**Iteration:** Gradient notebook calls Star via API, Zachary evaluates quality

---

## Dependencies Between Phases

```
R&D-A (Ring Attractor)    ──┐
R&D-B (Frozen Identity)   ──┤── R&D-C (Tiers) ──┐
                           │                    │
                           └────────────────────┼── R&D-D (Metacognition) ──┐
                                                │                           │
                                                └───────────────────────────┼── R&D-E (Pathways)
                                                                        │
                                                                        ▼
                                                               R&D-F (World Knowledge)
                                                                        │
                                                                        ▼
                                                               R&D-G (2-Hour Test) ──→ R&D-H (Emergence)
```

**Can run R&D-F (reader) in parallel with R&D-D/R&D-E — no dependency.**

---

## What's Not In This Plan

- **GPU / neural networks:** Not needed if symbolic architecture works
- **Ollama / local LLM:** Not a dependency — Star IS the intelligence
- **Cloud services:** Star runs on CPU, Zachary's laptop
- **Benchmarks:** We're not optimizing for benchmarks — optimizing for feel

---

*Last updated: 2026-03-25*
*Created: 2026-03-25*
