# TCMW-A: Temporal Causal Memory Weaving + Anticipation

**Generated:** 2026-04-10
**Status:** 🔜 IN PROGRESS
**Stack:** Pure Rust, CPU-only, no external ML services

---

## What This Is

TCMW-A gives Starfire a **learned model of Zach's behavioral grammar** — not just memory of what happened, but foresight into what he'll likely do next. It runs on pure Rust ML (Markov chains, k-means, EMA updates), no GPU, no Ollama.

The mutually reinforcing loop:
- Anticipation improves continuity (CEF/BGE get better signal)
- Continuity improves anticipation (OAFL closes the loop)

---

## Five Layers

### Layer 1 — Causal Event Fabric (CEF)
**Extends existing `lib/causal/`** — adds outcome tags + half-life decay to the causal graph.

New on top of CausalGraph:
- `CausalEvent` — action node with causal parent pointer, outcome tag, half-life weight
- Events carry WHY (causal parent), not just WHAT (action name)
- Half-life decay: `weight(t) = weight₀ × 0.5^(t / half_life)`
- On each causal event, CEF emits to BGE

### Layer 2 — Behavioral Grammar Encoder (BGE)
**New: `lib/tcmw_a/bge.rs`**

Encodes Zach's action sequences into a learnable grammar:
- **Session archetypes** — cluster of recurring action patterns (e.g., "pipeline build", "debugging loop", "research sweep")
- **Markov transition matrix** — P(archetype_j | archetype_i) from observed transitions
- **Intent Resumption Token** — on restart, the last CEF event provides context to resume

Archetype detection: simple k-means on action frequency vectors (pure Rust, no external deps).

### Layer 3 — Anticipatory Intent Horizon (AIH)
**New: `lib/tcmw_a/aih.rs`**

Rolling probability cone, re-evaluated on every causal event (not a clock):
```
P(intent_{t+N}) = BGE(history) × e^(-λ × N) × CEF_weight
```
- λ (horizon decay constant): higher = shorter cone, lower = longer-range predictions
- Cone depth N: configurable (default 5)
- Outputs ranked list of `IntentPrediction` items

### Layer 4 — Proactive Staging Engine (PSE)
**New: `lib/tcmw_a/pse.rs`**

Takes AIH's top predictions and pre-stages resources/actions above threshold θ:
- Pre-warm: file contents, repo state, embeddings cache
- Draft: suggested next action in natural language
- All staging is **reversible and zero-side-effect** until user confirms or causal event fires
- Threshold θ: only stage when P > θ (avoids noisy low-confidence pre-staging)

### Layer 5 — Online Anticipation Feedback Loop (OAFL)
**New: `lib/tcmw_a/oafl.rs`**

Closes the learning loop:
- ✅ Match at rank 1 → strengthen causal edge, decrease λ for this archetype
- ⚠️ Match at rank 3–5 → mild reinforcement, widen cone
- ❌ Miss → penalize grammar, trigger k-means re-clustering pass
- Uses exponential moving average (EMA) for weight updates

---

## Module Structure

```
lib/tcmw_a/
├── mod.rs              — TCMWEngine orchestrates all 5 layers
├── cef.rs              — CausalEvent + CEF half-life decay (extends causal/)
├── bge.rs              — Behavioral Grammar: archetypes + Markov chain
├── aih.rs              — Anticipatory Intent Horizon: probability cone
├── pse.rs              — Proactive Staging Engine: pre-staging task pool
└── oafl.rs             — Online Anticipation Feedback Loop: EMA updates
```

---

## Key Data Types

```rust
// CEF — Layer 1
pub struct CausalEvent {
    pub id: EventId,
    pub parent: Option<EventId>,        // causal parent
    pub action: String,                // what happened
    pub outcome: Outcome,              // success/partial/failure/abandoned
    pub weight: f64,                   // half-life weighted
    pub timestamp: i64,
    pub archetype_id: Option<ArchetypeId>,
}

pub enum Outcome { Success, Partial, Failed, Abandoned }

// BGE — Layer 2
pub struct SessionArchetype {
    pub id: ArchetypeId,
    pub label: String,                 // "pipeline_build", "debug_loop"
    pub action_signature: Vec<String>,  // characteristic actions
    pub frequency: usize,
}

pub struct MarkovChain {
    transitions: HashMap<(ArchetypeId, ArchetypeId), usize>, // count
    totals: HashMap<ArchetypeId, usize>,
}

// AIH — Layer 3
pub struct IntentPrediction {
    pub rank: usize,
    pub action: String,
    pub probability: f64,
    pub horizon: usize,              // steps ahead
    pub causal_parents: Vec<EventId>,
    pub archetype_id: Option<ArchetypeId>,
}

// PSE — Layer 4
pub struct StagedAction {
    pub id: StagedId,
    pub prediction: IntentPrediction,
    pub action_type: StagedActionType,
    pub reversible: bool,
    pub status: StagedStatus,
    pub created_at: i64,
}

pub enum StagedActionType {
    PreFetch { path: String },
    PreLoad { resource: String },
    Draft { text: String },
}

// OAFL — Layer 5
pub struct PredictionDelta {
    pub predicted: IntentPrediction,
    pub actual_action: String,
    pub delta_magnitude: f64,
    pub archetype_id: Option<ArchetypeId>,
    pub timestamp: i64,
}

pub enum MatchQuality {
    Perfect,     // rank 1 match
    Partial,     // rank 3-5 match
    Miss,        // no match
}
```

---

## Data Flow

```
User Action
    │
    ▼
[Layer 1: CEF] ──emit event──►
    │ causal parent, outcome tag, half-life weight
    ▼
[Layer 2: BGE] ──archetype + Markov ──►
    │ session archetype, transition probabilities
    ▼
[Layer 3: AIH] ──probability cone──►
    │ ranked predictions P(intent_{t+N})
    ▼
[Layer 4: PSE] ──if P > θ: pre-stage──►
    │ reversible actions, suggestions
    ▼
[Layer 5: OAFL] ──was prediction right?──►
    │ EMA updates, grammar revision
    └──────────────────────────────────► back to Layer 2
```

---

## Implementation Phases

### Phase 1: CEF (Causal Event Fabric)
- [x] `CausalEvent` struct with outcome tags
- [x] Half-life decay function
- [x] `CEF::record(event)` → stores and emits to BGE
- [x] Extend `lib/causal/mod.rs` integration

### Phase 2: BGE (Behavioral Grammar Encoder)
- [x] `SessionArchetype` + archetype detection (k-means on action vectors)
- [x] `MarkovChain::update(arch_from, arch_to)` — increment transition count
- [x] `MarkovChain::probability(arch_from, arch_to)` —贝叶斯 smoothed
- [x] `BGE::current_archetype()` — guess from recent action sequence
- [x] `BGE::next_archetype()` — from Markov chain

### Phase 3: AIH (Anticipatory Intent Horizon)
- [x] `IntentPrediction::build(cone_depth, λ, bge_state, cef_weight)`
- [x] Rolling cone re-evaluated on each CEF event
- [x] `AIH::top_predictions(k)` — ranked list

### Phase 4: PSE (Proactive Staging Engine)
- [x] `StagedAction::new(prediction, action_type)`
- [x] Threshold θ gating: only stage if P > θ
- [x] Reversibility: all PSE actions can be cancelled
- [x] `StagedActionType` variants: PreFetch, PreLoad, Draft
- [x] `PSE::propose()` — surface top prediction as draft suggestion

### Phase 5: OAFL (Online Anticipation Feedback Loop)
- [x] `OAFL::record_delta(prediction, actual_action)` → MatchQuality
- [x] EMA weight updates: strengthen or penalize
- [x] Grammar revision trigger: re-run k-means if miss rate exceeds threshold
- [x] λ adaptation: decrease λ for accurate archetypes, increase for noisy ones

---

## Stack

- Pure Rust (no external services)
- No `ndarray` or ML crates — simple k-means on `Vec<f64>`
- Markov chain: `HashMap<(ArchetypeId, ArchetypeId), usize>` + Laplace smoothing
- EMA: `α × new + (1-α) × old`
- Serialization: `serde` (already in Starfire's deps)

---

## Testing

- `cargo test -p star --lib tcmw_a` — all 5 layer tests
- `cargo test -p star --lib` — full Starfire suite
- No network, no files required for unit tests

---

## Integration Points

- `lib/causal/mod.rs` — CEF builds on CausalGraph
- `lib/runtime/mod.rs` — Runtime calls TCMWEngine on each user action
- `lib/learning/hypothesis.rs` — OAFL can trigger hypothesis revision
- `lib/curriculum/mod.rs` — AIH predictions inform curriculum gap discovery
