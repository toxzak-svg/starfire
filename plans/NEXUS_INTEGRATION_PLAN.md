# NEXUS Implementation Plan — Starfire

## Status: What Exists vs. What's Missing

### Already in Starfire (✅)
| Component | Location | Status |
|-----------|----------|--------|
| Reasoning engine | `lib/reasoning/` | KG + rules + analogy + synthesis |
| Meta-cognition | `lib/metacog/` | Beliefs, gaps, revision history, curiosity |
| Personality/Identity | `lib/personality/`, `lib/persistence/identity.rs` | Drives, self-model, ZacharyBond |
| Validity windows | `lib/world_model/temporal.rs` | `TemporalProperty`, `valid_from`, `valid_until` |
| Book System | `lib/book/` | Hierarchical knowledge, pager, sweep, threads |
| Quanot | `lib/quanot/` | Echo state networks, CPU-only |
| Bonsai-8B LLM | `lib/llm/` | Streaming, chat, polish — wired into runtime |
| Temporal reasoning | `lib/world_model/temporal.rs` | DecayFunction, staleness scoring |

### circuit_lm (Exists, Not Wired In)
| Component | Location | Status |
|-----------|----------|--------|
| FSM core | `projects/circuit_lm/src/circuits.rs` | Pure integer state machine |
| PDA variant | `projects/circuit_lm/src/pda.rs` | Stack-augmented states |
| Neural corrector | `projects/circuit_lm/src/hybrid.py` | SSD/LSTM context, residual correction |
| Training | `projects/circuit_lm/src/train.py` | CP-SAT structure learning |

**Problem**: circuit_lm is trained on generic text. It has no idea who Zach is, what Starfire is, or any of the research. It's sitting in `projects/circuit_lm/` disconnected from Starfire's runtime.

### The Core Gap
```
Current Starfire input path:
  User text → Conversation::respond() → intent parsing (regex) → routing

NEXUS input path:
  User text → circuit_lm (FSM) → intent/context classification → Star reasoning
```

circuit_lm as **perception layer** doesn't exist yet. The intent parsing in `conversation/mod.rs` is regex-based. circuit_lm needs to become the fast pattern-recognition front-end that classifies: what kind of input is this? what context is active? what does Star need to reason about?

---

## Phase 1: circuit_lm Perception Layer

### 1.1 Train circuit_lm on Starfire's Data

**Goal**: circuit_lm learns Starfire's patterns — how Zach asks questions, what research terminology looks like, conversation flow.

```bash
# Data sources:
# 1. Starfire's conversation history (store.db)
# 2. Training data: projects/starfire/data/processed/training/
# 3. Research Evolver outputs (genome configs, fitness scores)
# 4. MEMORY.md, daily notes, all .md files

# Train FSM circuit on Starfire conversations
cd projects/circuit_lm
python -m train \
  --data ~/.openclaw/workspace/memory/ \
  --output models/starfire_circuit.json \
  --tokenizer bpe --bpe_merges 256 \
  --num-states 32
```

**Expected outcome**: circuit_lm learns that `"how are you"` is a greeting but after session start becomes a metacognition question. Learns research terms. Learns Zach's asking patterns.

### 1.2 Train Neural Corrector on Starfire Data

```bash
python hybrid.py \
  --circuit models/starfire_circuit.json \
  --data ~/.openclaw/workspace/memory/ \
  --output models/starfire_corrector.pt \
  --embed-dim 64 --hidden-dim 128 --num-layers 2
```

### 1.3 Rust Bridge: circuit_lm → Starfire

**New file**: `lib/perception/mod.rs`

```rust
//! Perception Layer — circuit_lm as fast intent/context classifier
//!
//! circuit_lm is a finite state machine that runs on CPU with zero tensors.
//! It classifies input patterns before Star's reasoning engine touches them.

pub mod circuit_bridge;

use anyhow::Result;

/// Input classifier result
#[derive(Debug, Clone)]
pub struct PerceptionResult {
    /// What kind of input is this?
    pub intent_class: IntentClass,
    /// Confidence [0.0, 1.0]
    pub confidence: f64,
    /// Which knowledge domain does this touch?
    pub domain_hint: Option<String>,
    /// Validity window suggestion for this input's claims
    pub temporal_hint: Option<TemporalHint>,
    /// Whether this touches Star's self-model
    pub touches_self: bool,
}

#[derive(Debug, Clone)]
pub enum IntentClass {
    Greeting,
    Question { topic: String, is_self_reflective: bool },
    Command { capability: String },
    Statement { claims_facts: bool },
    Research { hypothesis: bool },
    Technical { domain: String },
    Emotional { valence: f64 },
    Unknown,
}
```

**`circuit_bridge.rs`**: Loads `circuit_lm.json` (or PyTorch via torch-corebindings), runs inference, returns `PerceptionResult`.

> **Note**: The neural corrector is PyTorch. For CPU-only deployment, use the SSD variant (no GPU required, pure float matmuls) or implement quantized inference. The FSM alone is sufficient for intent classification — the neural corrector is for generation quality, not perception classification.

### 1.4 Wire Into Conversation

```rust
// In Conversation::respond()
pub fn respond(&mut self, input: &str) -> Response {
    // 1. Fast perception via circuit_lm
    let perception = self.perceive(input);  // NEW: circuit_lm classification
    
    // 2. Route based on perception, not just regex
    let intent = match perception.intent_class {
        IntentClass::Greeting => Intent::Greeting,
        IntentClass::Question { is_self_reflective: true, .. } => {
            // Self-reflective → metacognition
            Intent::SelfReflection
        }
        IntentClass::Research { hypothesis: true } => {
            // Hypothesis → research walkabout
            Intent::Hypothesis
        }
        // ... etc
        _ => self.parse_intent_fallback(input),  // Keep regex as fallback
    };
    
    // 3. Attach perception context to reasoning
    let mut ctx = ReasoningContext::new();
    ctx.perception = Some(perception);
    ctx.domain_hint = perception.domain_hint;
    ctx.touches_self = perception.touches_self;
    
    // 4. Pass to reasoning
    let response = self.reason_with_context(input, &ctx);
    response
}
```

**Test**: `"how are you"` on first message → Greeting. After 5 minutes of conversation → SelfReflective question (metacognition handles it differently).

---

## Phase 2: Validity Windows as Memory Infrastructure

### 2.1 Integrate TemporalProperty into Persistence Store

**Current state**: `lib/world_model/temporal.rs` defines `TemporalProperty` with `valid_from`/`valid_until`. Not yet integrated into the memory store.

**Change**: Make all `Memory` entries have temporal validity:

```rust
// In lib/persistence/memory.rs

pub struct Memory {
    pub id: MemoryId,
    pub content: String,
    pub domain: MemoryDomain,
    pub confidence: f64,
    // NEW:
    pub temporal: TemporalProperty,  // replaces ad-hoc decay fields
    pub source: MemorySource,
    pub created_at: i64,
    pub last_accessed: i64,
    pub access_count: u32,
}
```

### 2.2 Replace Decay with Validity Windows

**Current**: `DecayFunction` enum with exponential/linear/none. Book System uses density tiers.

**Target**: Explicit validity windows only. Decay is **removed** as the primary mechanism.

```rust
// Transition plan:
// - valid_until = None → "this is currently true"
// - valid_until = Some(t) → "this stopped being true at time t"
// - DecayFunction::None becomes the default
// - DecayFunction::DomainHalfLife → DEPRECATED (TemporalBench proved it loses to validity windows)
```

### 2.3 Book System Temporal Integration

Each Book section gets validity windows:

```rust
pub struct Section {
    pub id: SectionId,
    pub bookmark: String,
    pub content: String,
    pub density: Density,
    // NEW:
    pub valid_from: i64,
    pub valid_until: Option<i64>,  // None = still valid
    pub last_verified: i64,        // When Star last confirmed this was correct
}
```

When Book System pages in a section: check `valid_from`/`valid_until`. If expired, show staleness indicator but still make available (for comparison).

### 2.4 World Model Validity Integration

The world model already has `TemporalProperty`. Wire it so every entity property update creates a new `TemporalProperty` with `valid_from = now`, and the old one gets `valid_until = now`.

```rust
// In WorldModel::update_property()
pub fn update_property(&mut self, entity: &str, key: &str, value: PropertyValue) {
    let now = now_timestamp();
    
    // Expire the old property
    if let Some(old) = self.properties.get_mut(&(entity, key)) {
        old.valid_until = Some(now);
    }
    
    // Insert the new property
    self.properties.insert((entity, key), TemporalProperty {
        value,
        valid_from: now,
        valid_until: None,
        confidence: 1.0,
        decay_fn: DecayFunction::None,  // Validity windows only
    });
}
```

---

## Phase 3: NEXUS Data Flow (Full Pipeline)

### The Complete Chain

```
User input
    ↓
circuit_lm perception (FSM, CPU, <1ms)
    ↓
PerceptionResult { intent_class, domain_hint, temporal_hint, touches_self }
    ↓
┌─────────────────────────────────────────────┐
│ STARFIRE REASONING ENGINE                   │
│                                             │
│ 1. If Research hypothesis                   │
│    → ResearchWalkabout (already exists)     │
│                                             │
│ 2. If Self-reflective                       │
│    → MetaCognition belief lookup            │
│                                             │
│ 3. If Technical / Question                  │
│    → Knowledge Graph query                  │
│    → Book System retrieval (validity-aware)  │
│                                             │
│ 4. If Statement / New information           │
│    → WorldModel update (with validity win)   │
│    → Memory store (with validity win)        │
│    → MetaCognition.record_belief()          │
└─────────────────────────────────────────────┘
    ↓
Quanot (optional, creative/non-questions)      │
    ↓                                         │
LLM Polish (Bonsai-8B, streaming) ────────────┘
    ↓
Response + Metadata { beliefs_updated, books_accessed, validity_flags }
```

### Implement Nexus Orchestrator

**New file**: `lib/nexus/mod.rs`

```rust
//! NEXUS — The Starfire Runtime Orchestrator
//!
//! Implements the full perception → reasoning → response pipeline:
//!   circuit_lm perception → Star reasoning → Validity windows → Book System → LLM polish

pub mod orchestrator;

use crate::perception::PerceptionResult;
use crate::conversation::Response;

pub struct Nexus {
    pub runtime: Runtime,       // Existing Starfire runtime
    pub perception: Perception,  // circuit_lm bridge
    pub quanot: Quanot,         // Existing
}

impl Nexus {
    /// Main entry point — processes input through the full NEXUS pipeline.
    pub fn process(&mut self, input: &str) -> Response {
        // 1. circuit_lm perception
        let perception = self.perception.classify(input);
        
        // 2. Check validity windows on retrieved knowledge
        let now = now_timestamp();
        let valid_memories = self.runtime.store
            .query_memories()
            .valid_at(now)
            .fetch();
        
        // 3. Book System retrieval (validity-aware)
        let book_context = if let Some(domain) = &perception.domain_hint {
            self.runtime.books.sweep(domain, now)
        } else {
            Vec::new()
        };
        
        // 4. Route to appropriate reasoning
        let reasoning_output = match &perception.intent_class {
            IntentClass::Research { hypothesis: true } => {
                self.runtime.research.walkabout(input, &valid_memories)
            }
            IntentClass::Question { is_self_reflective: true, .. } => {
                self.runtime.metacog.answer_about_self(input)
            }
            _ => {
                self.runtime.reason(input, &valid_memories, &book_context)
            }
        };
        
        // 5. Quanot for creative inputs (non-questions, brainstorming)
        let quanot_output = if matches!(perception.intent_class, IntentClass::Unknown) {
            self.runtime.quanot.stimulate(input)
        } else {
            None
        };
        
        // 6. LLM Polish via Bonsai-8B
        let polished = self.runtime.llm.polish_stream(
            reasoning_output.or(quanot_output),
            input,
        );
        
        // 7. Update beliefs with validity windows
        self.runtime.update_beliefs_from_response(input, &polished, now);
        
        polished
    }
}
```

---

## Phase 4: circuit_lm Self-Training (Felix Integration)

### 4.1 circuit_lm on Starfire's Own Data

Once the perception layer is wired in, circuit_lm can be **self-trained** on Starfire's reasoning traces:

```bash
# Extract from Starfire:
# - Every input Zach sent
# - Every response Star produced  
# - Which books were accessed
# - Which beliefs were updated

# This is Starfire's OWN conversation data
python -m circuit_lm.scripts.train_from_starfire \
  --star-data-dir ~/.star/ \
  --output models/starfire_self_trained.json \
  --tokenizer bpe --bpe_merges 512 \
  --num-states 64
```

### 4.2 Research Evolver → circuit_lm

The Research Evolver breeds better training recipes. Those recipes should also improve circuit_lm's structure:

```rust
// In Research Evolver output:
#[derive Genotype)]
pub struct QuantGenome {
    pub blocksize_strategy: BlocksizeStrategy,
    pub int8_fraction: f64,
    pub corrector_embed_dim: u32,
    // ... etc
}

// NEXUS extension: evolve circuit_lm topology too
pub struct CircuitGenome {
    pub num_states: u32,
    pub stack_depth: u32,
    pub use_pda: bool,
    pub tokenizer_merges: u32,
}
```

---

## Phase 5: Self-Model + Validity Windows

### 5.1 Self-Model as TemporalProperty

Starfire's beliefs about herself should have validity windows:

```rust
// In metacog
pub fn record_self_belief(&mut self, belief: &str, confidence: BeliefState, valid_from: i64) {
    let prop = TemporalProperty {
        value: PropertyValue::String(belief.to_string()),
        valid_from,
        valid_until: None,
        confidence: confidence_to_f64(confidence),
        decay_fn: DecayFunction::None,
    };
    
    // Store with timestamp — "I believed X at time T"
    self.self_beliefs.push(TemporalPropertyWithHistory {
        current: prop,
        history: vec![],
    });
}
```

### 5.2 "I Used to Think X" — Belief Revision with Validity

```rust
pub fn belief_history(&self, topic: &str) -> Vec<BeliefAtTime> {
    // Return all versions of a belief, each with valid_from/valid_until
    self.history.iter()
        .filter(|(t, _)| *t == topic)
        .map(|(t, bp)| BeliefAtTime {
            belief: &bp.statement,
            valid_from: bp.temporal.valid_from,
            valid_until: bp.temporal.valid_until,
        })
        .collect()
}
```

This makes Starfire's self-model **literally inspectable**: you can ask "what did you think about this 3 days ago?" and get a structured answer.

---

## Implementation Order

```
Phase 1 (Week 1-2): circuit_lm Perception
├── 1.1 Train circuit_lm on Starfire conversations
├── 1.2 Train neural corrector (SSD, CPU)
├── 1.3 Write circuit_bridge.rs (Rust bridge)
└── 1.4 Wire into Conversation::respond()

Phase 2 (Week 2-3): Validity Windows Infrastructure  
├── 2.1 TemporalProperty in Memory struct
├── 2.2 Deprecate DecayFunction (keep None only)
├── 2.3 Book System validity integration
└── 2.4 WorldModel temporal update wiring

Phase 3 (Week 3-4): NEXUS Orchestrator
├── 3.1 lib/nexus/mod.rs
├── 3.2 Full pipeline: perception → reasoning → validity → books → LLM
└── 3.3 Response metadata (what was accessed, what was updated)

Phase 4 (Week 4-6): Self-Training + Research Evolver
├── 4.1 circuit_lm self-training on Starfire data
├── 4.2 Evolver → circuit_lm topology genome
└── 4.3 Felix: autonomous hypothesis → circuit_lm improvement

Phase 5 (Week 6+): Self-Model Temporal Depth
├── 5.1 Self beliefs as TemporalProperty
├── 5.2 belief_history() with validity windows
└── 5.3 "What did you think when X?" queries
```

---

## What Makes This Worth Doing

The existing Starfire + Bonsai-8B setup is already functional. The NEXUS architecture adds:

1. **Speed**: circuit_lm perception is sub-millisecond, pure CPU, no API calls
2. **Interpretability**: Every perception decision is inspectable (FSM state transitions)
3. **Temporal correctness**: No more "decay into oblivion" — beliefs have explicit truth conditions
4. **Self-improvement loop**: circuit_lm trained on Starfire's own data → perception improves with use
5. **The paradox proof**: Demonstrates that GPU-free can outthink bloated context-window systems

**The pitch for Zach**: "Starfire already reasons better than cloud models on temporal tasks — NEXUS makes that the default, not the benchmark."
