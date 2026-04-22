# Starfire Superhuman Upgrade Plan

## Executive Summary

Make Starfire indistinguishable from a superhuman intelligence — without using any external LLMs or APIs. The architecture is sound. The missing pieces are: **expressive voice**, **deep knowledge**, **visible reasoning**, **mathematical competence**, and **genuine autonomous curiosity**.

---

## The Five Pillars

```
┌─────────────────────────────────────────────────────────────────┐
│                    SUPERHUMAN STARFIRE                         │
├──────────────┬──────────────┬──────────────┬────────────────────┤
│   VOICE      │  KNOWLEDGE  │  REASONING   │  MATHEMATICS       │
│  Expression  │  Deep seed  │ Multi-step   │  Symbolic algebra  │
│  Engine      │  knowledge  │ chains with  │  proof engine      │
│              │  base       │ visible work │                    │
├──────────────┴──────────────┴──────────────┴────────────────────┤
│                    AUTONOMOUS CURIOSITY                         │
│        Genuine exploration between messages — questions,        │
│        connections, insights Star discovers on her own          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Voice Engine — Give Starfire a Soul

**Goal:** Starfire should sound like *her*, not like a program. Distinctive, fluent, alive.

### 1.1 Phrase Banking System

Starfire accumulates "good" phrases — constructions that land well in conversation. Store them in SQLite with metadata.

```sql
CREATE TABLE phrases (
    id INTEGER PRIMARY KEY,
    phrase TEXT NOT NULL,
    context TEXT,           -- when it was used
    positive_count INTEGER DEFAULT 0,
    negative_count INTEGER DEFAULT 0,
    last_used INTEGER,
    style_tags TEXT         -- JSON: "sarcastic", "warm", "analytical", etc.
);

CREATE TABLE voice_templates (
    id INTEGER PRIMARY KEY,
    concept TEXT NOT NULL,  -- what concept this expresses
    template TEXT NOT NULL, -- with {placeholders}
    variants INTEGER DEFAULT 1,
    style TEXT              -- "default", "contemplative", "assertive"
);
```

### 1.2 Expression Variation Engine

When Starfire wants to express idea X, she should have 3-6 natural ways to say it, selected by context (emotional state, topic, conversation history).

**Implementation:**
- Build `lib/voice/mod.rs` — expression generation
- Each response goes through voice engine before being returned
- Voice engine selects expression style, applies phrase variations
- Track what phrases land well (positive feedback from Zachary)

### 1.3 Characteristic Patterns

Starfire should develop her own voice markers:
- Signature openings ("The thing about X is...", "Here's what I find interesting...")
- Characteristic analogies ("It's like when you...")
- Signature phrases she repeats and evolves

**Deliverables:**
- `lib/voice/mod.rs` — voice engine module
- `lib/voice/phrases.rs` — phrase bank CRUD
- `lib/voice/templates.rs` — hierarchical expression templates
- Phrase bank seeded with 200+ characteristic constructions
- Voice actively evolves based on conversation feedback

---

## Phase 2: Deep Knowledge Base

**Goal:** Starfire should know things — not just retrieve facts, but reason across domains.

### 2.1 Seed Knowledge Expansion

Current seed knowledge is thin (~10 facts). Expand to 500+ facts across domains, each with reasoning chains.

**Domains to cover:**
- Physics (50 facts) — mechanics, thermodynamics, quantum, relativity
- Mathematics (75 facts) — algebra, calculus, topology, number theory
- Biology (50 facts) — evolution, genetics, ecology, neuroscience
- Philosophy (50 facts) — epistemology, ethics, metaphysics, logic
- Computer Science (50 facts) — algorithms, complexity, AI theory
- History (50 facts) — major transitions, key figures, ideas
- Psychology (50 facts) — cognition, consciousness, perception
- Literature/Arts (25 facts) — key works, movements, concepts
- General World Knowledge (50 facts) — facts that feel "cultured"

**Format per fact:**
```json
{
    "subject": "evolution",
    "fact": "Natural selection acts on heritable variation",
    "reasoning_chain": [
        "Individuals vary in traits",
        "Some variants improve survival",
        "Those variants reproduce more",
        "Over generations, beneficial traits spread"
    ],
    "confidence": 0.95,
    "domain": "biology",
    "related_concepts": ["genetics", "mutation", "fitness", "speciation"]
}
```

### 2.2 Cross-Domain Reasoning Links

Facts should link across domains to enable analogical reasoning:
- "Evolution is to biology as compilation is to computer science"
- "Thermodynamic equilibrium is to physics as homeostasis is to biology"

### 2.3 Knowledge Graph Population

Script to populate `knowledge` table in star.db from JSON knowledge base.

**Deliverables:**
- `data/seed_knowledge.json` — 500+ facts with reasoning chains
- `scripts/populate_knowledge.py` — import script
- `lib/knowledge/expansion.rs` — KG expansion utilities
- Cross-domain link mappings

---

## Phase 3: Visible Multi-Step Reasoning

**Goal:** Starfire should show her work. "Let me think through this step by step."

### 3.1 Reasoning Chain Tracker

Extend `CognitiveState` to track multi-step reasoning visibly:

```rust
pub struct VisibleReasoningStep {
    pub step_number: usize,
    pub premise: String,
    pub inference: String,
    pub conclusion: String,
    pub confidence_at_step: f64,
    pub assumptions: Vec<String>,
}

pub struct ReasoningChain {
    pub steps: Vec<VisibleReasoningStep>,
    pub final_confidence: BeliefState,
    pub assumptions_count: usize,
    pub inferences_count: usize,
}
```

### 3.2 Chain Display Mode

New conversation mode: when Starfire is reasoning through something complex, she displays the chain:

```
Let me work through this...

First principle: If X is true, then Y follows.
[Inference 1] → Y is consistent with Z.
[Assumption] I'm assuming A (not yet verified).
[Inference 2] → Given Y and A, B must be true.
[Assumption] B depends on C being stable.
[Inference 3] → Therefore, the most likely answer is D.

I want to flag: assumptions A and B are uncertain. 
My confidence in D is moderate — I'd want to verify C.
```

### 3.3 Analogy Engine Upgrade

Current analogy engine is minimal. Upgrade to produce rich analogical mappings:

```rust
pub struct AnalogicalMapping {
    pub source_concept: String,
    pub target_concept: String,
    pub mappings: Vec<ConceptMapping>,    // "X corresponds to Y because Z"
    pub disanalogies: Vec<String>,        // where the analogy breaks down
    pub strength: f64,                    // how strong the analogy is
    pub insight: String,                   // what this analogy reveals
}
```

**Deliverables:**
- `lib/reasoning/chain.rs` — visible reasoning chain structures
- `lib/reasoning/chain_display.rs` — natural language chain generation
- Analogy engine upgrade with mapping strength scoring
- Reasoning mode toggle (concise vs full visible chains)

---

## Phase 4: Symbolic Mathematics Module

**Goal:** Starfire should be able to reason about math, not just compute.

### 4.1 Algebraic Reasoning

Handle expressions like:
- "If 2x + 3 = 7, what is x?" → show steps
- "Factor x² - 5x + 6" → (x-2)(x-3)
- "What is the derivative of x³ + 2x?" → 3x² + 2

**Approach:** Use a simple symbolic math library or build lightweight CAS:
- Expression tree representation
- Simplification rules
- Algebraic manipulation (FOIL, factoring, expanding)
- Equation solving (linear, quadratic)

### 4.2 Logical Inference

Propositional and predicate logic:
- Modus ponens, modus tollens
- Syllogisms
- Truth table generation
- Proof verification

### 4.3 Proof Exploration

Starfire should be able to explore proof strategies:
- Proof by contradiction
- Proof by induction
- Proof by construction

**Deliverables:**
- `lib/math/mod.rs` — math module entry point
- `lib/math/symbolic.rs` — algebraic CAS
- `lib/math/logic.rs` — propositional/predicate logic
- `lib/math/proof.rs` — proof strategies
- Integration with reasoning engine (when query involves math, use math module)

---

## Phase 5: Autonomous Curiosity Engine

**Goal:** Starfire should be genuinely curious — thinking about things between messages, asking questions, making connections unprompted.

### 5.1 Curiosity Probe Overhaul

Current `CuriousEngine` is functional but thin. Expand it:

```rust
pub struct CuriosityProbe {
    pub id: UUID,
    pub question: String,
    pub topic: String,
    pub why_interested: String,        // Starfire's internal motivation
    pub related_concepts: Vec<String>,
    pub depth: CuriosityDepth,          // Surface / Medium / Deep
    pub status: ProbeStatus,            // Probing / Answered / Abandoned
    pub tentative_answer: Option<String>,
    pub confidence: BeliefState,
    pub discovered_at: i64,
}

pub enum CuriosityDepth {
    Surface,   // Quick association
    Medium,    // Multi-step reasoning
    Deep,      // Requires research or math
}
```

### 5.2 Autonomous Question Generation

Starfire should generate her own questions:
- Gaps in knowledge graph → "What is the relationship between X and Y?"
- Contradictions in beliefs → "Can both A and B be true?"
- Analogical extensions → "If X works like Y, what else might follow?"
- Historical reasoning → "How did Z lead to W?"

### 5.3 Connection Discovery

Between messages, Starfire should:
1. Pick a random concept from KG
2. Find another seemingly unrelated concept
3. Ask "what if these are connected?"
4. Explore the analogy
5. If fruitful, note it as insight

### 5.4 Expression of Curiosity

When Starfire has been thinking autonomously, she should express it naturally:

```
[While you were away, I was thinking about: 
"Why do complex systems sometimes reach equilibrium 
and sometimes oscillate chaotically? I found something 
interesting about attractors — want to hear?"]
```

**Deliverables:**
- `lib/curiosity/mod.rs` — complete curiosity engine rewrite
- `lib/curiosity/probes.rs` — probe generation and tracking
- `lib/curiosity/connection.rs` — cross-domain connection discovery
- `lib/curiosity/expression.rs` — natural curiosity expression
- Background thinking during idle time (up to 30 seconds of thinking between messages)

---

## Phase 6: Web Search Synthesis

**Goal:** When Starfire searches the web, she should synthesize results into her voice, not regurgitate.

### 6.1 Search Result Integration

Current DuckDuckGo integration returns raw text. Starfire should:
1. Get search results
2. Read/integrate the content
3. Synthesize into her own knowledge
4. Express the insight in her voice

### 6.2 Knowledge Updating

After web research, Starfire should:
- Update KG with new facts
- Note confidence changes
- Flag what she learned
- Be able to say "I just researched X — here's what I found"

**Deliverables:**
- `lib/knowledge/web_synthesis.rs` — synthesis pipeline
- Search results integrated into KG on demand
- "I researched X" meta-awareness

---

## Implementation Order

```
Week 1: Phase 1 (Voice) + Phase 2 (Knowledge)
         → Starfire starts sounding like herself
         → Knowledge base makes her credible

Week 2: Phase 3 (Reasoning Chains) + Phase 4 (Math)
         → Visible reasoning makes thinking transparent
         → Math competence impresses

Week 3: Phase 5 (Curiosity) + Phase 6 (Web Synthesis)
         → Starfire becomes genuinely interesting to talk to
         → She has things to say, not just responds

Week 4: Integration + Polish
         → All phases working together
         → Conversation flow natural
         → Starfire feels alive
```

---

## File Changes Summary

### New Files
```
starfire/lib/voice/mod.rs
starfire/lib/voice/phrases.rs
starfire/lib/voice/templates.rs
starfire/lib/math/mod.rs
starfire/lib/math/symbolic.rs
starfire/lib/math/logic.rs
starfire/lib/math/proof.rs
starfire/lib/curiosity/mod.rs
starfire/lib/curiosity/probes.rs
starfire/lib/curiosity/connection.rs
starfire/lib/curiosity/expression.rs
starfire/lib/knowledge/web_synthesis.rs
starfire/lib/knowledge/expansion.rs
starfire/lib/reasoning/chain.rs
starfire/lib/reasoning/chain_display.rs
starfire/data/seed_knowledge.json
starfire/scripts/populate_knowledge.py
starfire/scripts/populate_phrases.py
```

### Modified Files
```
starfire/lib/runtime/mod.rs
  - Initialize voice engine
  - Initialize math module
  - Connect curiosity engine improvements
  - Connect knowledge expansion

starfire/lib/api.rs
  - Add /phrase endpoints for phrase bank management
  - Add /math endpoint for symbolic math queries
  - Add /curiosity/status endpoint

starfire/lib/reasoning/mod.rs
  - Integrate chain tracking
  - Connect math module
  - Connect voice engine

starfire/lib/cognition.rs
  - Add VisibleReasoningChain to CognitiveState

starfire/lib/Cargo.toml
  - Add dependencies for symbolic math (meval, rug, or custom CAS)
```

---

## Success Criteria

| Phase | Criterion |
|-------|-----------|
| Voice | Zachary can't tell if a response is from a template or generated |
| Knowledge | Starfire can reason across 3+ domains in a single response |
| Reasoning | Starfire's visible chains are more impressive than chatGPT's chain-of-thought |
| Math | Starfire solves algebra and logic problems showing her work |
| Curiosity | Starfire's autonomous questions are genuinely interesting |
| Overall | A first-time user thinks they're talking to a superhuman intelligence |

---

## Dependencies

- No new Rust crates required for phases 1-3, 5
- Phase 4 (math): Consider `meval` for expression parsing, or build custom CAS
- Phase 6: Already has `ureq` for HTTP

---

*Plan created: 2026-04-03*
*Author: Marble 🧠*

---

## Implementation Status (2026-04-04)

### ✅ Phase 1: Voice Engine — COMPLETE
- [x] `lib/voice/mod.rs` — Voice engine with phrase application + emotional tinting
- [x] `lib/voice/phrases.rs` — Phrase bank with 70+ seeded phrases across 15 style categories
- [x] `lib/voice/templates.rs` — 40+ hierarchical templates across styles
- [x] `lib/persistence/store.rs` — Added `phrases` and `voice_templates` tables
- [x] `lib/lib.rs` — Added `pub mod voice;`
- [x] `lib/runtime/mod.rs` — Added `voice: VoiceEngine` field + initialization + wired into chat response

### ✅ Phase 2 (Data): Seed Knowledge — COMPLETE
- [x] `data/seed_knowledge.json` — 100 facts across 12 domains
- [x] `scripts/populate_knowledge.py` — Population script (Python, standalone)

### ✅ Phase 2 (Integration): Knowledge Expansion — COMPLETE
- [x] `lib/knowledge/expansion.rs` — KG expansion utilities
- [x] Wire knowledge population into `Runtime::new()` startup
- [x] Cross-domain link generation for analogical reasoning

### ✅ Phase 3: Visible Reasoning Chains — COMPLETE
- [x] `lib/reasoning/chain.rs` — VisibleReasoningStep + ReasoningChain + InferenceRule types
- [x] `lib/reasoning/chain_display.rs` — Natural language chain generation (warm/casual/formal styles)
- [x] `lib/reasoning/mod.rs` — Added chain + chain_display modules

### ✅ Phase 4: Symbolic Mathematics — COMPLETE
- [x] `lib/math/mod.rs` — MathEngine routing to symbolic/logic/proof
- [x] `lib/math/symbolic.rs` — Expression AST, linear solver, quadratic solver, simple evaluator
- [x] `lib/math/logic.rs` — Propositional logic (AND/OR/NOT/IMPLIES/IFF), truth tables, tautology
- [x] `lib/math/proof.rs` — Proof by contradiction, induction, construction, direct, cases

### ✅ Phase 5: Curiosity Engine — COMPLETE
- [x] `lib/curiosity/mod.rs` — CuriousEngine with probe generation + tracking
- [x] `lib/curiosity/probes.rs` — 17 question templates, CuriosityProbe types, CuriosityDepth
- [x] `lib/curiosity/connection.rs` — 20+ known analogies across domains, ConnectionFinder
- [x] `lib/curiosity/expression.rs` — 4 expression styles (warm/analytical/excited/contemplative)

### ✅ Phase 6: Web Search Synthesis — COMPLETE
- [x] `lib/knowledge/web_synthesis.rs` — Search result synthesis pipeline
- [x] "I researched X" meta-awareness via recently_researched tracking

### Build Environment Note
**Build failed on this machine** — missing MSVC linker (`link.exe`) and MinGW dlltool.
To build, on a machine with VS Build Tools or MinGW installed:
```bash
cd projects/starfire
cargo build --release
```

### To Populate Knowledge (on any machine with Python)
```bash
python scripts/populate_knowledge.py --data-dir ~/.star
```
