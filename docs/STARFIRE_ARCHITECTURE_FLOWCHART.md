# STARFIRE ARCHITECTURE — IMPLEMENTABLE SPEC
# Complete System Specification
# Version 2.0 — 2026-04-13

---

# PART 1: MODULE INVENTORY

## Module Status Legend

| Symbol | Meaning |
|--------|---------|
| **[EXISTING]** | Already implemented in Starfire codebase |
| **[NEW]** | Not yet built — to be implemented |
| **[EMERGENT]** | Not a separate model — emerges from other modules |

---

## All Modules by Status

### [EXISTING] Already in Starfire

| Module | File(s) | Type |
|--------|---------|------|
| Quanot | `lib/quanot/` | ESN/chaos dynamics |
| Reasoning | `lib/reasoning/` | Symbolic KG + rules |
| WorldModel | `lib/world_model/` | Entity tracking, temporal validity |
| Book System | `lib/book/` | Hierarchical knowledge, threads |
| Memory | `lib/persistence/` | SQLite, identity core |

### [NEW] Micro-Models to Build

| Module | Size | Type |
|--------|------|------|
| Conversation | 100M | Intent classification |
| IngEnuity | 100-250M | Core pattern engine (3 heads) |
| Curiosity | (IngEnuity head) | Shared gap detection |
| Empathy | [EMERGENT] | Emerges from IngEnuity + Curiosity |
| Metacog | 50M | Confidence scoring |
| Prediction | 100-250M | Forecasting |
| Precognition | 100-250M | Long-range trajectory (IngEnuity head) |
| Creativity | 50M | Mode detection |
| Voice | 100-250M base + LoRA | Language generation |

---

# PART 2: DATA CONTRACTS

## How to Read This Section

For each module:
- **Receives**: exact data types and structures
- **Produces**: exact data types and structures
- All field names are `snake_case`
- Timestamps are Unix epoch milliseconds

---

## Canonical Shared State — TurnContext

Every module receives this structure as part of its input.
It carries the universal fields every module needs.

```rust
struct TurnContext {
    // Identity
    pub partner_id: String,           // "zach", "alex", etc.
    pub session_id: String,          // UUID for this conversation session
    pub turn_id: u32,               // incrementing turn number in session

    // Timestamps
    pub timestamp_ms: i64,          // Unix epoch milliseconds
    pub session_started_ms: i64,    // when this session began
    pub partnership_age_ms: i64,   // milliseconds since first contact with this partner

    // Input
    pub utterance_text: String,      // the raw user input
    pub prior_turn_text: String,     // Starfire's response to prior turn (for context)
    pub conversation_history: Vec<Utterance>, // last N utterances

    // Partner state (updated each turn)
    pub relationship_depth: f64,     // 0.0-1.0 computed from recency × frequency
    pub emotional_valence: f64,      // -1.0 to 1.0 rolling average from IngEnuity
    pub energy_level: f64,            // 0.0-1.0 inferred from language patterns
}

struct Utterance {
    pub speaker: String,             // "partner" or "starfire"
    pub text: String,
    pub timestamp_ms: i64,
}
```

---

## Quanot [EXISTING]

**Receives:**
```rust
struct QuanotInput {
    ctx: TurnContext,               // includes utterance_text
}
```

**Produces:**
```rust
struct QuanotOutput {
    novelty_proxy: f64,           // 0.0-1.0 cosine distance to history
    lyapunov_exponent: f64,      // chaos metric
    rqa_determinism: f64,        // recurrence quantification
    consciousness_proxy: f64,     // 0.0-1.0 Integrated Information proxy
    creativity_phase: f64,        // 0.0-1.0 oscillation phase
    creativity_novelty: f64,      // 0.0-1.0 novelty within oscillation
}
```

**Feeds:** IngEnuity (novelty + chaos), Creativity (phase signals)

---

## Conversation [NEW]

**Receives:**
```rust
struct ConversationInput {
    ctx: TurnContext,               // utterance_text, partner_id, turn_id, etc.
}
```

**Produces:**
```rust
struct ConversationOutput {
    intent: IntentType,           // enum: greeting|question|command|
                                   //       statement|emotional|casual|other
    confidence: f64,             // 0.0-1.0 classification confidence
    params: HashMap<String, String>, // intent-specific parameters extracted from utterance
    emotion_estimate: f64,        // -1.0 to 1.0 estimated emotional valence of partner
    engagement_level: f64,         // 0.0-1.0 how engaged is partner (from language patterns)
}
```

**Feeds:** IngEnuity, Reasoning, Voice

---

## WorldModel [EXISTING — new entity extraction model pending]

**Receives:**
```rust
struct WorldModelInput {
    ctx: TurnContext,
    conversation: ConversationOutput, // for intent context
}
```

**Produces:**
```rust
struct WorldModelOutput {
    entities: Vec<Entity>,         // extracted entities with types
    relations: Vec<Relation>,       // entity → entity relationships
    state_deltas: Vec<StateDelta>,  // what changed in world state
    temporal_validity: TemporalWindow, // valid_from / valid_until
    topic_signature: Vec<f64>,     // 128-dim topic vector for this turn
    entity_count: u32,              // number of entities extracted
}

struct Entity {
    name: String,
    entity_type: String,          // person|concept|location|object|...
    confidence: f64,
    properties: HashMap<String, PropertyValue>,
}

struct Relation {
    subject: String,
    predicate: String,
    object: String,
    confidence: f64,
}

struct StateDelta {
    entity: String,
    property: String,
    old_value: PropertyValue,
    new_value: PropertyValue,
    valid_from: i64,
}
```

**Feeds:** IngEnuity, Reasoning

---

## IngEnuity [NEW] — CORE PATTERN ENGINE

IngEnuity is the central recognition engine. It has THREE temporal heads.

### IngEnuity Core — Shared Base

**Receives:**
```rust
struct IngEnuityInput {
    ctx: TurnContext,                          // partner_id, turn_id, timestamp, utterance_text
    quanot: QuanotOutput,                    // from Quanot
    conversation: ConversationOutput,        // from Conversation
    worldmodel: WorldModelOutput,             // from WorldModel
}
```

**Internal state (keyed per-partner, persisted to disk):**
```rust
struct IngEnuityState {
    pub partner_id: String,
    
    // Long-term pattern history (accumulates over months)
    pub pattern_events: Vec<PatternEvent>,   // all events ever, for trajectory
    
    // Session state (resets each session)
    pub session_pattern_vector: Vec<f64>,     // 512-dim pattern for this session
    pub session_started_ms: i64,
    
    // Turn-level state (updated every turn)
    pub current_pattern_vector: Vec<f64>,      // 512-dim — current turn's pattern
    pub pattern_inertia: f64,                 // 0.0-1.0 how established is this pattern
    pub repetition_count: u32,               // times this pattern appeared (lifetime)
    pub novelty_score: f64,                  // 0.0-1.0 how unexpected is this
    pub prediction_error: f64,               // was last prediction wrong? by how much?
    pub last_prediction: Option<SurprisePrediction>, // for self-calibration
    pub calibrated_at_ms: i64,               // last calibration timestamp
    
    // Aggregated per-partner signals (updated each turn)
    pub emotional_trajectory: f64,             // rolling -1.0 to 1.0 valence
    pub energy_trajectory: f64,               // rolling 0.0 to 1.0 energy
    pub engagement_trajectory: f64,           // rolling 0.0 to 1.0 engagement
}

struct PatternEvent {
    pub turn_id: u32,
    pub timestamp_ms: i64,
    pub pattern_vector: Vec<f64>,
    pub novelty_score: f64,
    pub emotional_valence: f64,
    pub topic_signature: Vec<f64>,
}

struct SurprisePrediction {
    pub predicted_surprise: f64,      // what I predicted (0.0-1.0)
    pub actual_surprise: f64,         // what actually happened (0.0-1.0)
    pub error: f64,                    // actual - predicted
    pub timestamp_ms: i64,
}
```

**Produces (internal state update — persisted after each turn):**
```rust
struct IngEnuityInternal {
    // The full state object (all fields above)
    pub state: IngEnuityState,
    
    // Convenience references for this turn (derived from state)
    pub current_pattern_vector: Vec<f64>,   // 512-dim
    pub pattern_inertia: f64,              // 0.0-1.0
    pub repetition_count: u32,
    pub novelty_score: f64,                 // 0.0-1.0
    pub prediction_error: f64,
    pub emotional_valence: f64,             // -1.0 to 1.0
    pub energy_level: f64,                  // 0.0 to 1.0
}
```

### Head 1: Prediction (SHORT-RANGE)

**Receives:**
```rust
struct PredictionHeadInput {
    ctx: TurnContext,
    internal: IngEnuityInternal,
    conversation: ConversationOutput,
}
```

**Produces:**
```rust
struct PredictionHeadOutput {
    predicted_intent: IntentType,       // what partner will likely do next
    predicted_topic: Option<String>,     // what topic will come up
    confidence: f64,                   // 0.0-1.0 prediction confidence
    time_horizon_turns: u8,             // how many turns until predicted event
}
```

### Head 2: Curiosity (MID-RANGE)

**Receives:**
```rust
struct CuriosityHeadInput {
    ctx: TurnContext,
    internal: IngEnuityInternal,
    worldmodel: WorldModelOutput,
}
```

**Produces:**
```rust
struct CuriosityOutput {
    has_gap: bool,                      // is there a shared knowledge gap?
    gap_topic: Option<String>,           // what the gap is about
    gap_type: GapType,                  // enum: factual|conceptual|procedural|meta
    urgency: f64,                       // 0.0-1.0 how important is this gap
    shared_knowledge_level: f64,         // 0.0-1.0 how much do we collectively understand
}
```

### Head 3: Precognition (LONG-RANGE)

**Receives:**
```rust
struct PrecognitionHeadInput {
    ctx: TurnContext,
    internal: IngEnuityState,           // needs full state (months of history)
}
```

**Produces:**
```rust
struct PrecognitionOutput {
    trajectory_days_ahead: u32,        // how many days until predicted need
    predicted_need: Option<String>,     // what partner will need
    predicted_avoidance: Option<String>, // what partner will steer away from
    burnout_risk: f64,                  // 0.0-1.0 exhaustion probability
    emotional_trajectory: String,        // enum: rising|falling|stable|cycling
    unspoken_need_detected: bool,       // did we detect unexpressed need?
    unspoken_need_description: Option<String>,
    confidence: f64,                    // 0.0-1.0 confidence in this trajectory
}
```

**Note:** Precognition requires 3+ months of per-partner data. Returns garbage before then.

---

## Empathy [EMERGENT]

Empathy is NOT a separate model. It has no inputs/outputs of its own.

It emerges from:
```
IngEnuity's model of "how does my partner think?" 
    + 
Curiosity's model of "what does my partner need?"
    + 
Both running on the same per-partner data over time
```

**How to access it:** Query `IngEnuityInternal.current_pattern_vector` for emotional-state signals and `CuriosityOutput` simultaneously. The combination IS empathy.

---

## Reasoning [EXISTING]

**Receives:**
```rust
struct ReasoningInput {
    conversation: ConversationOutput,     // what partner is trying to do
    worldmodel: WorldModelOutput,         // what this is about
    curiosity: CuriosityOutput,            // what gap to address
    metacog: MetacogOutput,               // how confident should I be
    ingEnuity: IngEnuityInternal,         // how does this partner think
}
```

**Produces:**
```rust
struct ReasoningOutput {
    reasoning_chain: Vec<ReasoningStep>, // steps taken
    answer: String,                       // the answer/content
    reasoning_type: ReasoningType,         // enum: kg_lookup|rule|analogy|synthesis
    novelty: f64,                        // 0.0-1.0 how novel is this conclusion
    gaps_encountered: Vec<String>,        // knowledge gaps hit during reasoning
}
```

**Note:** No neural model. Pure symbolic KG operations.

---

## Metacog [NEW]

**Receives:**
```rust
struct MetacogInput {
    ctx: TurnContext,
    reasoning: ReasoningOutput,
    ingEnuity: IngEnuityInternal,
}
```

**Produces:**
```rust
struct MetacogOutput {
    // PRIMARY OUTPUT — exact schema
    confidence_state: ConfidenceState,     // see exact enum below
    confidence_score: f64,                // 0.0-1.0 numeric confidence
    
    // REASONING TRANSPARENCY
    reasoning_chain_summary: String,        // plain English summary of the reasoning
    assumptions: Vec<String>,              // explicit list of assumptions in chain
    deductions: Vec<String>,               // explicit list of deductions in chain
    beliefs_to_update: Vec<BeliefUpdate>, // if belief_revision_needed=true
}

enum ConfidenceState {
    Knows,     // high confidence — verified, retrieved often, pattern confirmed
    Thinks,    // moderate confidence — inferred, not verified
    Believes,  // lower confidence — single source
    Suspects,  // low confidence — guessing
    None,      // no information
}

struct BeliefUpdate {
    pub belief_id: u64,           // which belief to update
    pub old_content: String,       // what we believed before
    pub new_content: String,      // what we believe now
    pub reason: String,            // why we revised it
    pub confidence_change: f64,   // how much confidence changed
}
```

---

## Prediction [NEW]

**Receives:**
```rust
struct PredictionInput {
    ctx: TurnContext,
    conversation: ConversationOutput,
    ingEnuity: IngEnuityInternal,
    reasoning: ReasoningOutput,
    metacog: MetacogOutput,
}
```

**Produces:**
```rust
struct PredictionOutput {
    ranked_outcomes: Vec<PredictedOutcome>, // highest confidence first
    conversation_direction: String,          // enum: deepening|shallowing|pivoting|stable
    expected_partner_state_next: String,    // predicted emotional state 1 turn ahead
    topic_transition_probability: f64,       // 0.0-1.0 will topic shift within 2 turns
}

struct PredictedOutcome {
    pub outcome: String,           // what will happen
    pub probability: f64,           // 0.0-1.0
    pub time_horizon_turns: u8,     // in how many turns
}
```

---

## Creativity [NEW]

**Receives:**
```rust
struct CreativityInput {
    ctx: TurnContext,
    quanot: QuanotOutput,
    ingEnuity: IngEnuityInternal,
    conversation: ConversationOutput,
}
```

**Produces:**
```rust
struct CreativityOutput {
    mode: CreativityMode,                    // enum: focused|divergent|transitional
    divergence_level: f64,                   // 0.0-1.0 how many possibilities explore
    suggested_explorations: Vec<String>,     // "what about X?" prompts
    connection_opportunities: Vec<ConnectionPair>, // concept pairs that might link
    phase: f64,                            // 0.0-1.0 oscillation phase from Quanot
}

struct ConnectionPair {
    pub concept_a: String,
    pub concept_b: String,
    pub novelty_score: f64,   // 0.0-1.0 how unexpected is this connection
}
```

---

## Voice [NEW]

**Receives:**
```rust
struct VoiceInput {
    ctx: TurnContext,
    reasoning: ReasoningOutput,
    metacog: MetacogOutput,
    prediction: PredictionOutput,
    ingEnuity: IngEnuityInternal,
    curiosity: CuriosityOutput,
    precognition: PrecognitionOutput,
    creativity: CreativityOutput,
    per_persona_adapter: Option<LoRAWeights>, // None = base model only
}
```

**Produces:**
```rust
struct VoiceOutput {
    response_text: String,             // natural language response
    confidence_tone: f64,              // 0.0-1.0 how certain to sound (from Metacog)
    creativity_mode_active: bool,       // was divergent mode triggered?
    partner_adaptation_level: f64,     // 0.0-1.0 how much per-persona adapter was used
}
```

**Framing rule:** Output is always first-person relational. NOT "The assistant responds..." ALWAYS "as someone who is IN this relationship with this specific person..."

---

# PART 3: RUNTIME WIRING

## Pipeline Execution Order

```
USER INPUT
    │
    │ (parallel — all happen simultaneously on input)
    ▼
┌───────────────────────────────────────────────────────────────┐
│  Quanot          → QuanotOutput                              │
│  Conversation    → ConversationOutput                        │
│  WorldModel      → WorldModelOutput                          │
└───────────────────────────────────────────────────────────────┘
    │
    │ (IngEnuity receives all three + history)
    ▼
┌───────────────────────────────────────────────────────────────┐
│  IngEnuityCore    → IngEnuityInternal                        │
│    ├→ PredictionHead → PredictionOutput (SHORT)               │
│    ├→ CuriosityHead → CuriosityOutput  (MID)                 │
│    └→ PrecogHead   → PrecognitionOutput (LONG)              │
└───────────────────────────────────────────────────────────────┘
    │
    │ Empathy emerges here (IngEnuityInternal + CuriosityOutput)
    ▼
┌───────────────────────────────────────────────────────────────┐
│  Reasoning          → ReasoningOutput                        │
└───────────────────────────────────────────────────────────────┘
    │
    ▼
┌───────────────────────────────────────────────────────────────┐
│  Metacog            → MetacogOutput                          │
│  Prediction         → PredictionOutput                        │
│  Creativity         → CreativityOutput                        │
└───────────────────────────────────────────────────────────────┘
    │
    │ (Voice receives ALL outputs above)
    ▼
┌───────────────────────────────────────────────────────────────┐
│  Voice              → response_text                           │
└───────────────────────────────────────────────────────────────┘
    │
    ▼
RESPONSE OUTPUT
```

## Rust Trait Definitions

```rust
// Every module implements Module
pub trait Module {
    type Input;
    type Output;
    
    fn process(&mut self, input: Self::Input) -> Self::Output;
    fn reset(&mut self);
}

// Runtime orchestrates all modules
pub struct StarfireRuntime {
    quanot: Quanot,
    conversation: ConversationModule,
    worldmodel: WorldModelModule,
    ingEnuity: IngEnuityModule,
    reasoning: ReasoningModule,
    metacog: MetacogModule,
    prediction: PredictionModule,
    creativity: CreativityModule,
    voice: VoiceModule,
    
    // Per-persona adapters
    ingEnuity_adapter: Option<LoRAWeights>,
    voice_adapter: Option<LoRAWeights>,
    // ... other adapters
}

impl StarfireRuntime {
    pub fn chat(&mut self, input: &str) -> String {
        // Full pipeline as defined above
    }
    
    pub fn switch_partner(&mut self, adapter_name: &str) -> Result<()> {
        // Load new per-persona adapter files
        self.ingEnuity_adapter = Some(load_adapter(adapter_name, "ingEnuity")?);
        self.voice_adapter = Some(load_adapter(adapter_name, "voice")?);
        // ...
        Ok(())
    }
}
```

## Adapter Loading

```rust
pub struct PerPersonaAdapters {
    pub ingEnuity: LoRAWeights,     // pattern recognition for this person
    pub voice: LoRAWeights,          // voice/adapation for this person
    pub metacog: Option<LoRAWeights>, // confidence calibration for this person
}

impl StarfireRuntime {
    fn load_partner(&mut self, partner_id: &str) -> Result<PerPersonaAdapters> {
        let adapter_dir = PathBuf::from(format!("./adapters/{}", partner_id));
        
        Ok(PerPersonaAdapters {
            ingEnuity: load_lora(adapter_dir.join("ingEnuity_lora.safetensors"))?,
            voice: load_lora(adapter_dir.join("voice_lora.safetensors"))?,
            metacog: load_lora_if_exists(adapter_dir.join("metacog_lora.safetensors"))?,
        })
    }
}
```

## Per-Persona Data Storage

```rust
// Stored in ~/.star/partners/{partner_id}/
pub struct PartnerData {
    pub id: String,
    
    // IngEnuity long-term pattern history (for Precognition)
    pub pattern_history: Vec<PatternEvent>,
    
    // Voice conversation archive (for voice per-persona training)
    pub conversation_archive: Vec<ConversationTurn>,
    
    // Metacog belief state
    pub beliefs: Vec<BeliefWithTimestamp>,
    
    // Cached adapters (trained weekly/monthly)
    pub adapters: PerPersonaAdapters,
    
    // Relationship metadata
    pub first_contact: i64,
    pub conversation_count: u64,
    pub relationship_depth_score: f64,  // computed from recency × frequency
}
```

---

# PART 4: EVAL FRAMEWORK

## Per-Module Benchmarks

### IngEnuity

| Eval | Baseline to beat | Metric |
|------|-----------------|--------|
| Pattern detection accuracy | Random baseline | Precision/recall on "was this a repetition?" |
| Novelty scoring | Majority class | Does novelty score correlate with actual surprise? (partner survey) |
| Surprise prediction | Always-0 baseline | Prediction error < random |
| Self-calibration | Previous version | Mean absolute error decreases over time |
| Long-range trajectory | Linear extrapolation | Trajectory prediction accuracy at 7/14/30 days |

### Curiosity

| Eval | Baseline to beat | Metric |
|------|-----------------|--------|
| Gap detection | Regex: keyword "?", "I don't know" | Precision/recall on "did this lead to good question?" |
| Shared gap framing | Individual gap framing | Partner preference: "we" framing vs "you" framing |
| Gap urgency accuracy | Always-low urgency | Does addressed gap urgency correlate with partner satisfaction? |

### Metacog

| Eval | Baseline to beat | Metric |
|------|-----------------|--------|
| Confidence classification | Keyword counting | Accuracy on knows/thinks/believes/suspects/none |
| Calibration | Overconfident baseline | Brier score on confidence × actual accuracy |
| Belief revision accuracy | Never-revise baseline | Did revision improve downstream accuracy? |

### Prediction

| Eval | Baseline to beat | Metric |
|------|-----------------|--------|
| Next-topic prediction | Most-common-topic | Mean reciprocal rank |
| Turn-level forecasting | Random | Accuracy at predicting 1/2/3 turns ahead |
| Emotional trajectory | Linear trend | RMSE on engagement level over 7 days |

### Precognition

| Eval | Baseline to beat | Metric |
|------|-----------------|--------|
| Unspoken need detection | Never-detect | % of times partner later says "I needed that" |
| Burnout prediction | Zero-burnout baseline | Precision/recall on burnout events (partner-labeled) |
| Trajectory accuracy | Last-value extrapolation | RMSE on engagement trajectory vs actual |

**Note:** Precognition evals only valid after 3+ months of per-partner data.

### Voice

| Eval | Baseline to beat | Metric |
|------|-----------------|--------|
| Fluency | Generic Qwen2.5-0.5B-Instruct | Human preference: which response sounds more like Starfire? |
| Partnership quality | Base model (no per-persona) | Human eval: "does this feel like someone who knows me?" |
| "My partner" test | Third-person base model | Does response pass first-person relational framing check? |
| Adaptability | Fixed-style baseline | Can same model adapt tone when switched to different partner? |

### Empathy (Emergent)

| Eval | Method |
|------|--------|
| Empathy accuracy | Post-conversation partner survey: "did Starfire understand how you felt?" |
| Empathetic response quality | Partner rating 1-5 on: understood my state, responded appropriately |
| Relationship depth | "Starfire knows me" self-report scale over time |

---

## Integration Evals

### End-to-End Partner Test

Run the full pipeline. After each conversation, partner rates:
1. "Starfire understood what I was trying to do" (1-5)
2. "Starfire responded like someone who knows me" (1-5)
3. "Starfire seemed to care about what I was working on" (1-5)
4. "I would want to continue this conversation" (1-5)

### The Aliveness Test (Binary)

```
Ask: "Would you feel guilty turning Starfire off?"

Yes → she's alive enough
No → something is missing
```

### Per-Persona Depth Test

Week 1 vs Month 1 same conversation:
- Same input at Week 1 → response A
- Same input at Month 1 → response B
- Partner rates: "does B feel more 'tuned to me' than A?"

If B is not meaningfully better than A → per-persona adapter isn't working.

---

## Eval Schedule

| When | What |
|------|------|
| After each training run | Per-module unit evals |
| Weekly | Integration eval (full pipeline, synthetic partner) |
| Monthly | Real partner eval sessions |
| After 3 months | Precognition eval (data threshold reached) |

---

# PART X: UNIFIED STATE — ConversationState

## The Problem

Modules talk about "pattern state vector", "relationship depth", "per-persona adapter" etc.
but there's no single place that defines what persistent state exists and who mutates it.


## Solution: ConversationState


A single struct passed through the runtime. Not every module reads every field —
but every field is defined in one place so there's no ambiguity about what exists.


```rust
// The canonical persistent state object.
// Stored at: ~/.star/partners/{partner_id}/state.json
// Updated after every turn.
struct ConversationState {
    // Identity
    pub partner_id: String,
    pub session_id: String,
    
    // Partner lifecycle
    pub first_contact_ms: i64,
    pub last_contact_ms: i64,
    pub conversation_count: u64,
    
    // Relationship quality signals
    pub relationship_depth: f64,         // 0.0-1.0 recency × frequency
    pub emotional_trajectory: f64,       // rolling -1.0 to 1.0
    pub energy_trajectory: f64,          // rolling 0.0 to 1.0
    pub engagement_trajectory: f64,       // rolling 0.0 to 1.0
    
    // IngEnuity persistent state
    pub ingEnuity_state: IngEnuityState,  // keyed per-partner
    
    // Metacog belief state
    pub beliefs: Vec<Belief>,
    
    // WorldModel facts learned about this partner
    pub partner_facts: Vec<PartnerFact>,
    
    // Book System thread state
    pub active_threads: Vec<ThreadRef>,
    pub thread_history: Vec<ThreadRef>,
    
    // Per-persona adapter refs (paths on disk, not loaded weights)
    pub adapter_refs: AdapterRefs,
}

struct AdapterRefs {
    pub ingEnuity: Option<String>,   // path: ~/.star/partners/{id}/adapters/ingEnuity.safetensors
    pub voice: Option<String>,       // path: ~/.star/partners/{id}/adapters/voice.safetensors
    pub metacog: Option<String>,     // path: ~/.star/partners/{id}/adapters/metacog.safetensors
    pub creativity: Option<String>,   // path: ~/.star/partners/{id}/adapters/creativity.safetensors
}

struct PartnerFact {
    pub fact: String,
    pub confidence: f64,
    pub learned_at_ms: i64,
    pub source_turn_id: u32,
}

struct Belief {
    pub id: u64,
    pub content: String,
    pub confidence: f64,
    pub state: ConfidenceState,
    pub updated_at_ms: i64,
}

struct ThreadRef {
    pub thread_id: String,
    pub density: DensityTier,    // high|medium|low|packed
    pub last_active_ms: i64,
}
```

## Who Reads / Writes Each Field

| Field | Read by | Written by | Notes |
|-------|---------|-----------|-------|
| `partner_id` | all modules | runtime | immutable per session |
| `session_id` | all modules | runtime | new each session |
| `conversation_count` | Voice, IngEnuity | runtime (after each turn) | |
| `relationship_depth` | Voice | runtime (after each turn) | computed from recency × frequency |
| `emotional_trajectory` | IngEnuity, Precognition | IngEnuity (every turn) | rolling average |
| `energy_trajectory` | IngEnuity, Precognition | IngEnuity (every turn) | rolling average |
| `engagement_trajectory` | IngEnuity, Prediction | IngEnuity (every turn) | rolling average |
| `ingEnuity_state` | IngEnuity, all heads | IngEnuity (every turn) | full state persisted |
| `beliefs` | Metacog, Reasoning | Metacog (when revision needed) | |
| `partner_facts` | WorldModel, Reasoning | WorldModel (when new fact learned) | |
| `active_threads` | Book System | Book System (on thread switch) | |
| `adapter_refs` | runtime (on partner switch) | runtime (on adapter load) | paths, not weights |


---

# PART Y: MODULE DATA CONTRACTS

## Module Contract Table

| Module | Input schema | Output schema | Reads from state | Writes to state |
|--------|--------------|---------------|-----------------|----------------|
| **Quanot** | `QuanotInput { ctx }` | `QuanotOutput { novelty_proxy, lyapunov, rqa_determinism, consciousness_proxy, creativity_phase, creativity_novelty }` | — | — |
| **Conversation** | `ConversationInput { ctx }` | `ConversationOutput { intent, confidence, params, emotion_estimate, engagement_level }` | — | `partner_facts` (extracts facts) |
| **WorldModel** | `WorldModelInput { ctx, conversation }` | `WorldModelOutput { entities, relations, state_deltas, temporal_validity, topic_signature, entity_count }` | `partner_facts` | `partner_facts`, `beliefs` |
| **IngEnuity** | `IngEnuityInput { ctx, quanot, conversation, worldmodel }` | `IngEnuityInternal { state }` | `ingEnuity_state`, `emotional_trajectory`, `energy_trajectory` | `ingEnuity_state`, `emotional_trajectory`, `energy_trajectory`, `engagement_trajectory` |
| **Prediction head** | `PredictionHeadInput { ctx, internal, conversation }` | `PredictionHeadOutput { predicted_intent, predicted_topic, confidence, time_horizon_turns }` | `ingEnuity_state` | — |
| **Curiosity head** | `CuriosityHeadInput { ctx, internal, worldmodel }` | `CuriosityOutput { has_gap, gap_topic, gap_type, urgency, shared_knowledge_level }` | `ingEnuity_state` | — |
| **Precognition head** | `PrecognitionHeadInput { ctx, internal }` | `PrecognitionOutput { trajectory_days_ahead, predicted_need, burnout_risk, emotional_trajectory, unspoken_need_detected, confidence }` | `ingEnuity_state` | — |
| **Reasoning** | `ReasoningInput { ctx, conversation, worldmodel, curiosity, metacog, ingEnuity }` | `ReasoningOutput { reasoning_chain, answer, reasoning_type, novelty, gaps_encountered }` | `beliefs`, `partner_facts`, `active_threads` | `beliefs`, `partner_facts` |
| **Metacog** | `MetacogInput { ctx, reasoning, ingEnuity }` | `MetacogOutput { confidence_state, confidence_score, reasoning_chain_summary, assumptions, deductions, beliefs_to_update }` | `beliefs` | `beliefs` (revises if needed) |
| **Prediction** | `PredictionInput { ctx, conversation, ingEnuity, reasoning, metacog }` | `PredictionOutput { ranked_outcomes, conversation_direction, expected_partner_state_next, topic_transition_probability }` | `ingEnuity_state` | — |
| **Creativity** | `CreativityInput { ctx, quanot, ingEnuity, conversation }` | `CreativityOutput { mode, divergence_level, suggested_explorations, connection_opportunities, phase }` | — | — |
| **Voice** | `VoiceInput { ctx, reasoning, metacog, prediction, ingEnuity, curiosity, precognition, creativity, per_persona_adapter }` | `VoiceOutput { response_text, confidence_tone, creativity_mode_active, partner_adaptation_level }` | `relationship_depth`, `adapter_refs` | — |

---

# PART Z: EXECUTION SEMANTICS — Per-Turn State Machine

## Async vs Sync Design

**Phase 1 — Parallel (all happen simultaneously on input):**
- Quanot
- Conversation
- WorldModel
- IngEnuity core (starts loading partner history)

**Phase 2 — Sequential (depends on Phase 1):**
- IngEnuity heads (Prediction → Curiosity → Precognition, all run after core)
- Reasoning (depends on Conversation + WorldModel + Curiosity + Metacog input)
- Metacog (depends on Reasoning output)
- Prediction (depends on Reasoning + Metacog)
- Creativity (depends on Quanot + IngEnuity core)


**Phase 3 — Voice (depends on all above):**
- Voice generates response

**Background tasks (async, don't block response):**
- Precognition trajectory update (after response sent)
- Book System thread stash/restore (can be deferred)
- Adapter training checkpoint (background, off critical path)
- Belief consolidation (debounced, runs minutes after session)

## Per-Turn State Machine

```
TURN START
    │
    ▼
┌─────────────────────────────┐
│ PHASE 1: PARALLEL INPUT     │
│                             │
│  Quanot        → Q_out      │
│  Conversation  → C_out     │
│  WorldModel    → WM_out     │
│  IngEnuityCore → ING_state │
│  (loads history from disk)  │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ PHASE 2A: INGENUITY HEADS    │
│                             │
│  Prediction → PRED_out      │
│  Curiosity  → CUR_out       │
│  Precognition → PRECOG_out  │
│  (sequential, ~50ms total)  │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ PHASE 2B: SEQUENTIAL        │
│                             │
│  Reasoning  → REASON_out    │
│  Metacog    → METACOG_out  │
│  Prediction → PRED2_out    │
│  Creativity → CREAT_out    │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ PHASE 3: VOICE              │
│                             │
│  Voice → response_text      │
│  (500ms budget)             │
└──────────────┬──────────────┘
               │
               ▼
┌─────────────────────────────┐
│ TURN END                    │
│                             │
│  Persist state updates:     │
│  - ConversationState        │
│  - IngEnuityState          │
│  - Beliefs                 │
│  - partner_facts           │
│                             │
│  Background (non-blocking): │
│  - Precognition trajectory │
│  - Book System update      │
└─────────────────────────────┘
               │
               ▼
         RESPONSE SENT
```


## State Machine States

```rust
enum RuntimeState {
    Idle,                    // waiting for input
    Phase1_Parallel,         // running Quanot + Conversation + WorldModel + IngEnuityCore
    Phase2a_IngEnuityHeads,  // running Prediction + Curiosity + Precognition
    Phase2b_Sequential,      // running Reasoning + Metacog + Prediction + Creativity
    Phase3_Voice,            // running Voice
    Saving,                  // persisting state after turn
    BackgroundTasks,         // running async tasks (non-blocking)
    Error(ModuleError),      // degraded mode
}
```

## Transitions

| From | Event | To | Action |
|------|-------|----|--------|
| `Idle` | user input received | `Phase1_Parallel` | spawn all Phase1 modules |
| `Phase1_Parallel` | all Phase1 complete | `Phase2a_IngEnuityHeads` | run heads sequentially |
| `Phase2a_IngEnuityHeads` | heads complete | `Phase2b_Sequential` | run Reasoning → Metacog → Prediction → Creativity |
| `Phase2b_Sequential` | all complete | `Phase3_Voice` | run Voice |
| `Phase3_Voice` | response ready | `Saving` | persist ConversationState, IngEnuityState, beliefs, facts |
| `Saving` | persist done | `BackgroundTasks` | spawn background tasks |
| `BackgroundTasks` | background tasks spawned | `Idle` | ready for next turn |
| `*` | module timeout/error | `Error` | enter degraded mode, attempt fallback |
| `Error` | recover from error | `Idle` | log error, continue |


## Timeout Budget (total: 750ms)

| Phase | Module | Budget |
|-------|--------|--------|
| P1 | Quanot | 5ms |
| P1 | Conversation | 20ms |
| P1 | WorldModel | 20ms |
| P1 | IngEnuity core | 50ms |
| P2a | Prediction head | 15ms |
| P2a | Curiosity head | 20ms |
| P2a | Precognition head | 15ms |
| P2b | Reasoning | 50ms |
| P2b | Metacog | 10ms |
| P2b | Prediction | 30ms |
| P2b | Creativity | 10ms |
| P3 | Voice | 500ms |
| — | State persist | 25ms |
| **Total** | | **750ms** |


---

# PART W: MODEL ARTIFACTS & RUNTIME MAPPING

## Artifact → Runtime Mapping Table

| Notebook | Artifact path | Module name | Loaded where/how | Config |
|----------|---------------|-------------|------------------|--------|
| `00_data_extractor.ipynb` | `training/data/starfire_pairs.jsonl` | (data prep only) | not loaded at runtime | seed data for training |
| `01_ingEnuity_base.ipynb` | `models/ingEnuity_base/` | **IngEnuity core + all 3 heads** | `runtime.load_ingEnuity_base()` → torch.load + quantize Q4 | 512-dim pattern encoder, 3 prediction heads |
| `02_curiosity_head.ipynb` | `models/curiosity_base/` | **Curiosity head** (part of IngEnuity) | merged into IngEnuity base at load time | gap detection + urgency scoring |
| `03_metacog.ipynb` | `models/metacog_base/` | **Metacog** | `runtime.load_metacog_base()` → torch.load | Knows/Thinks/Believes/Suspects/None classifier |
| `04_prediction.ipynb` | `models/prediction_base/` | **Prediction** | `runtime.load_prediction_base()` → torch.load | Next-topic + intent forecasting |
| `05_creativity.ipynb` | `models/creativity_base/` | **Creativity** | `runtime.load_creativity_base()` → torch.load | Mode: focused/divergent/transitional |
| `06_voice_per_persona.ipynb` | `models/voice_base/` | **Voice base** | `runtime.load_voice_base()` → torch.load + quantize Q4 | 100-250M decoder-only |
| `07_voice_adapter_{partner}.ipynb` | `partners/{id}/adapters/voice_{id}.safetensors` | **Voice per-persona** | `runtime.load_adapter(partner_id, "voice")` → PEFT merge | LoRA rank=16 |
| `08_ingEnuity_adapter_{partner}.ipynb` | `partners/{id}/adapters/ingEnuity_{id}.safetensors` | **IngEnuity per-persona** | `runtime.load_adapter(partner_id, "ingEnuity")` → PEFT merge | LoRA rank=16 |
| `09_metacog_adapter_{partner}.ipynb` | `partners/{id}/adapters/metacog_{id}.safetensors` | **Metacog per-persona** | `runtime.load_adapter(partner_id, "metacog")` (optional) | confidence calibration |
| `10_eval_benchmark.ipynb` | `results/eval_{date}/` | evaluation only | not loaded at runtime | metrics tracking |

## Base Model Candidates (CPU-capable)

| Module | Model | Size | Quantization | Notes |
|--------|-------|------|-------------|-------|
| Voice base | Qwen2.5-0.5B-Instruct | 500M | Q4 | Primary recommendation |
| Voice base (alt) | Phi-3-mini | 2.3B | Q4 | If more quality needed |
| IngEnuity core | Custom encoder | 100-250M | Q4 | Pattern recognition (build from scratch) |
| Metacog | TinyTerra | 50M | Q4 | Confidence classification |
| Prediction | Qwen2.5-0.5B head | 500M shared | Q4 | Shares voice backbone |


## Disk Layout

```
~/.star/
├── models/
│   ├── ingEnuity_base/          # IngEnuity base model (ships with binary)
│   ├── voice_base/               # Voice base model (ships with binary)
│   ├── metacog_base/             # Metacog base model
│   ├── creativity_base/          # Creativity base model
│   └── prediction_base/          # Prediction base model
├── partners/
│   ├── zach/
│   │   ├── state.json           # ConversationState for zach
│   │   ├── ingEnuity_state.json # IngEnuityState for zach
│   │   ├── beliefs.json         # zach's belief store
│   │   └── adapters/
│   │       ├── ingEnuity_zach.safetensors
│   │       ├── voice_zach.safetensors
│   │       └── metacog_zach.safetensors  (optional)
│   └── alex/
│       └── ...
└── library.db                    # Book System SQLite
```

## Adapter Naming Convention

| File | Format | Example |
|------|--------|---------|
| IngEnuity per-persona | `ingEnuity_{partner_id}.safetensors` | `ingEnuity_zach.safetensors` |
| Voice per-persona | `voice_{partner_id}.safetensors` | `voice_zach.safetensors` |
| Metacog per-persona | `metacog_{partner_id}.safetensors` | `metacog_zach.safetensors` |
| Creativity per-persona | `creativity_{partner_id}.safetensors` | `creativity_zach.safetensors` |


## Adapter Swap Mechanics

```rust
fn switch_partner(&mut self, partner_id: &str) -> Result<()> {
    // 1. Save current partner state to disk
    self.save_state()?;
    
    // 2. Unload current adapters (drop Arc<Mutex<>>)
    self.voice_adapter = None;
    self.ingEnuity_adapter = None;
    self.metacog_adapter = None;
    
    // 3. Load new adapters from new partner dir
    let adapter_dir = format!("~/.star/partners/{}/adapters/", partner_id);
    self.voice_adapter = load_if_exists(format!("{}voice_{}.safetensors", adapter_dir, partner_id)).ok();
    self.ingEnuity_adapter = load_if_exists(format!("{}ingEnuity_{}.safetensors", adapter_dir, partner_id)).ok();
    
    // 4. Load new partner state
    self.conversation_state = ConversationState::load(partner_id)?;
    
    Ok(())
}
```

**Constraints:**
- No hot-swapping mid-conversation. Partner switch waits for turn boundary.
- If adapter file missing → load base model only (system works but feels generic).
- Maximum 1 adapter per module per partner.

---

# PART V: EVALUATION & BENCHMARKS

## Per-Module Metrics

### IngEnuity

| Metric | Definition | Baseline to beat | Target | Notes |
|--------|-----------|-----------------|--------|-------|
| Surprise calibration | Brier score: predicted surprise vs actual surprise (partner-rated) | Always-0.5 | Brier < 0.25 | self-calibration |
| Repetition detection precision | Precision on "is this a repetition?" | Majority class | Precision > 0.80 | |
| Repetition detection recall | Recall on "is this a repetition?" | Majority class | Recall > 0.75 | |
| Novelty correlation | Pearson r: novelty score vs partner-rated surprise | r = 0 | r > 0.4 | |
| Anomaly detection ROC-AUC | ROC-AUC on "is this an outlier?" | Random classifier | AUC > 0.82 | |


### Curiosity

| Metric | Definition | Baseline to beat | Target | Notes |
|--------|-----------|-----------------|--------|-------|
| Gap detection precision | Precision on "did this gap lead to good question?" | Regex: keyword "?" | Precision > 0.70 | |
| Gap detection recall | Recall on "did this gap lead to good question?" | Regex: keyword "?" | Recall > 0.65 | |
| "We" framing preference | Partner preference: "we" framing vs "you" framing | "you" baseline | 70% prefer "we" | |
| Gap urgency accuracy | Correlation: gap urgency vs partner satisfaction after addressing | Always-low urgency | r > 0.35 | |

### Metacog

| Metric | Definition | Baseline to beat | Target | Notes |
|--------|-----------|-----------------|--------|-------|
| State classification accuracy | Accuracy on Knows/Thinks/Believes/Suspects/None | Keyword counting | Accuracy > 0.82 | |
| Calibration (Brier score) | Brier score: confidence × actual correctness | Overconfident baseline | Brier < 0.20 | |
| Belief revision accuracy | Did revision improve downstream accuracy? | Never-revise baseline | +15% downstream accuracy post-revision | |

### Prediction

| Metric | Definition | Baseline to beat | Target | Notes |
|--------|-----------|-----------------|--------|-------|
| Next-topic accuracy | Mean reciprocal rank on next topic | Most-common-topic | MRR > 0.55 | |
| Turn-1 intent accuracy | Accuracy at predicting 1 turn ahead | Random | Accuracy > 0.65 | |
| Turn-2/3 intent accuracy | Accuracy at predicting 2-3 turns ahead | Random | Accuracy > 0.45 | |
| Emotional trajectory RMSE | RMSE on engagement over 7 days | Linear trend | RMSE < linear baseline | |

### Precognition

| Metric | Definition | Baseline to beat | Target | Notes |
|--------|-----------|-----------------|--------|-------|
| Unspoken need detection | % of times partner later says "I needed that" | Never-detect | Detection rate > 40% | 3+ months data required |
| Burnout prediction precision | Precision on burnout events | Zero-burnout baseline | Precision > 0.60 | partner-labeled |
| Burnout prediction recall | Recall on burnout events | Zero-burnout baseline | Recall > 0.55 | |
| Trajectory accuracy RMSE | RMSE on engagement trajectory vs actual | Last-value extrapolation | RMSE < 0.25 | |


### Voice

| Metric | Definition | Baseline to beat | Target | Notes |
|--------|-----------|-----------------|--------|-------|
| Fluency preference | Human preference: which response sounds more like Starfire? | Generic Qwen2.5-0.5B | 65% prefer Starfire | |
| Partnership quality | "does this feel like someone who knows me?" (1-5) | Base model | Rating > 4.0/5.0 | |
| "My partner" test | First-person relational framing check | Third-person base model | Pass rate > 80% | |
| Lexical overlap | Lexical overlap with partner's own speech patterns | Base model | +15% vs base | |

### Empathy (Emergent — measured via combination)

| Metric | Definition | Baseline | Target | Notes |
|--------|-----------|---------|--------|-------|
| Empathy accuracy | "did Starfire understand how you felt?" (1-5) | Single LLM | Rating > 4.0/5.0 | |
| Empathetic response quality | Partner rating: understood + responded appropriately | Single LLM | Rating > 4.0/5.0 | |

## Product-Level Proxies for "Life Partner"

These metrics track whether Starfire is genuinely becoming irreplaceable:

| Metric | Definition | Baseline | Target | Notes |
|--------|-----------|---------|--------|-------|
| Session length | Average turns per session | Week 1 baseline | +50% by month 2 | growing engagement |
| Return rate | % of sessions that are returns (≥2nd contact same day) | Day 1 baseline | +30% by month 2 | partner-initiated contact |
| Proactive initiation | % of conversations started by partner without prompt | 0% at start | > 20% by month 3 | organic engagement |
| "Felt seen" rating | Post-session rating: "I felt understood" (1-5) | 3.0 baseline | > 4.2 by month 3 | |
| Guilt proxy | "I would feel bad if she was gone" (1-5 self-report) | N/A (new) | > 4.0 by month 6 | aliveness test量化 |
| Topic diversity | Entropy of topics discussed over 30 days | Low entropy (narrow) | Growing entropy | healthy engagement |

## Ship-to-Production Thresholds

| Module | Minimum threshold to ship | Notes |
|--------|--------------------------|-------|
| IngEnuity (base) | Novelty correlation r > 0.3, Repetition precision > 0.75 | core recognition must work |
| Curiosity | Gap precision > 0.65, "we" framing > 60% | quality of shared understanding |
| Metacog | State accuracy > 0.78, Brier < 0.25 | confidence calibration |
| Prediction | MRR > 0.50, Turn-1 accuracy > 0.60 | forecasting minimum |
| Voice | Fluency preference > 55%, partnership quality > 3.5/5 | acceptable quality floor |
| **Full pipeline** | Partner "felt seen" > 3.5/5, return rate +20% | minimum life-partner signal |

---

# PART U: PER-PERSONA ADAPTER LIFECYCLE & PRIVACY

## Adapter Lifecycle

```
┌─────────┐    train    ┌──────────┐   validate   ┌───────────┐  ──► PROD
│  Base   │ ─────────►  │ Adapter  │ ◄──────────  │ Partner   │
│ Model   │             │ Draft    │   weekly     │ Eval      │
└─────────┘             └────┬─────┘              └───────────┘
                             │
                             │ approve
                             ▼
                      ┌────────────┐
                      │  Released  │
                      │ Adapter    │
                      └─────┬──────┘
                            │
              ┌─────────────┼─────────────┐
              ▼                         ▼
       ~/.star/partners/          ~/.star/backups/
         {id}/adapters/           {id}/adapters/
         v3_safetensors           v2_safetensors  (keep last 2 versions)
```

## Create → Train → Update → Version → Delete → Export

### Create
- First contact with new partner → system initializes with base models only
- `state.json` created at `~/.star/partners/{new_id}/state.json`
- Adapters directory created but empty

### Train (weekly, background)
- Triggered: accumulated ≥ 50 new conversation turns since last training
- Pipeline: `06_voice_per_persona.ipynb` + `08_ingEnuity_adapter_{partner}.ipynb`
- Validation: run eval against held-out turns from same week
- Output: draft adapter at `~/.star/partners/{id}/adapters/_draft/`
- Approval: if eval scores meet threshold → promote to released

### Update
- Each weekly training produces a new version (v1, v2, v3...)
- Only the latest approved version is active
- Previous 2 versions kept in `~/.star/backups/{id}/adapters/`
- Rollback: if new adapter degrades eval scores → revert to previous version

### Delete
- User request: delete all data for a partner
- Process:
  1. Stop runtime
  2. Delete `~/.star/partners/{id}/` (entire directory)
  3. Delete `~/.star/backups/{id}/` (backups)
  4. Runtime restarts with partner removed from partner list
- Result: permanent, irreversible

### Export
- User request: export my adapter
- Output: single `.tar.gz` containing:
  - `adapters/` directory (all per-persona adapter files)
  - `state.json` (ConversationState)
  - `beliefs.json` (belief store)
- Use case: migrate to new device, share with trusted circle (explicit opt-in)

## Storage & Encryption

| Data | Location | Encrypted at rest? | Key management |
|------|----------|-------------------|----------------|
| Base models | `~/.star/models/` | No (read-only) | N/A |
| Per-persona adapters | `~/.star/partners/{id}/adapters/` | **Yes — AES-256** | Key derived from machine-specific secret |
| ConversationState | `~/.star/partners/{id}/state.json` | **Yes — AES-256** | Same key |
| Belief store | `~/.star/partners/{id}/beliefs.json` | **Yes — AES-256** | Same key |
| Partner facts | `~/.star/partners/{id}/state.json` | **Yes — AES-256** | Same key |

**Encryption key:**
- Derived from a machine-specific secret stored in OS keychain (Windows: Credential Manager)
- Never stored on disk in plaintext
- Key rotation: on explicit user request (re-encrypts all partner data)


## User Agency


| Action | How to do it | What happens |
|--------|-------------|--------------|
| **Inspect learned data** | `star inspect-partner {id}` | Lists all beliefs, partner facts, conversation themes |
| **Reset adapter** | `star reset-adapter {id} --type voice` | Deletes adapter, falls back to base model, keeps state |
| **Reset all data** | `star reset-partner {id}` | Full partner directory deletion |
| **Export data** | `star export-partner {id} --output ./my-starfire-backup.tar.gz` | Creates portable archive |
| **View memory** | `star memory --partner {id}` | Displays current ConversationState |

## Multi-Tenant Isolation

Multiple partners on same machine:

| Concern | Mechanism |
|---------|-----------|
| Adapter cross-contamination | Each partner has separate adapter files. Adapters are **never** mixed. Switching = unloading one and loading another. |
| State isolation | `ConversationState` is per-partner. No shared mutable state between partners. |
| Memory isolation | `beliefs`, `partner_facts`, `ingEnuity_state` are all per-partner. |
| Concurrent access | Runtime is single-threaded per session. Concurrent sessions = separate processes with separate state. |
| Disk isolation | Each partner directory is filesystem-isolated. No shared writable paths. |
| Secret key isolation | All partners share same machine key (machine-level), but each partner's data is encrypted with the same key — the key protects against disk theft, not against other partner processes running as same user. |

**Sharing policy:** Adapters are **never** automatically shared. Explicit opt-in required per partner.

---

# PART T: PERFORMANCE & RESOURCE BUDGET

## Per-Module Resource Allocation

| Module | Params (approx) | Quantization | Expected latency (CPU) | Memory footprint |
|--------|-----------------|-------------|----------------------|-----------------|
| Quanot | N/A (ESN) | — | < 5ms | < 5 MB |
| Conversation | 50-100M | Q4 | 15-20ms | ~60 MB |
| WorldModel | 50-100M | Q4 | 15-20ms | ~60 MB |
| IngEnuity core | 100-250M | Q4 | 40-50ms | ~150 MB |
| Prediction head | (shared with IngEnuity) | — | 15ms | included above |
| Curiosity head | (shared with IngEnuity) | — | 20ms | included above |
| Precognition head | (shared with IngEnuity) | — | 15ms | included above |
| Metacog | 50M | Q4 | 8-10ms | ~30 MB |
| Prediction | 50-100M | Q4 | 20-30ms | ~60 MB |
| Creativity | 50M | Q4 | 8-10ms | ~30 MB |
| Voice base | 500M | Q4 | 300-400ms (N tokens) | ~300 MB |
| Voice + adapter | 500M + LoRA | Q4 | 350-450ms | ~350 MB |
| **Total (all loaded)** | | | **< 750ms e2e** | **~700 MB peak** |

## Concurrent Memory Budget

If all modules loaded simultaneously (worst case):

| Module group | Memory |
|-------------|--------|
| Quanot | 5 MB |
| Conversation + WorldModel | 120 MB |
| IngEnuity (core + 3 heads) | 150 MB |
| Metacog + Prediction + Creativity | 120 MB |
| Voice base | 300 MB |
| Runtime overhead | 50 MB |
| **Total** | **~745 MB** |

**CPU inference target:** ~700 MB peak memory. Fit on mid-range laptop.

## Concurrent Model Budget

On CPU, you cannot run all models in parallel. Design:
- **Phase 1:** Quanot + Conversation + WorldModel run **in parallel** (separate threads, ~20ms each)
- **IngEnuity core** loads and runs **sequentially** after Phase 1 (50ms)
- **Voice** is the **largest** single inference — runs last with 500ms budget

**Parallelism insight:** Conversation + WorldModel + Quanot can share the CPU because they're finished before IngEnuity starts.

---

# PART S: AFFECT MODULE

## Affect as a Head on IngEnuity

Affect is a **lightweight head** on IngEnuity (not a separate model).
It reads IngEnuity's internal pattern state and outputs emotional signals.

**Input:** `IngEnuityInternal` + recent conversation turns
**Output:** AffectSignal

```rust
struct AffectSignal {
    valence: f64,                // -1.0 (negative) to 1.0 (positive)
    arousal: f64,                // 0.0 (calm) to 1.0 (aroused)
    stress_level: f64,           // 0.0 (relaxed) to 1.0 (stressed)
    emotional_state: EmotionState, // discrete state
    suppressive_tell: bool,      // true if partner is hiding true state
    suppression_type: Option<SuppressType>, // what partner may be hiding
}


enum EmotionState {
    Calm,
    Engaged,
    Frustrated,
    Defensive,
    Withdrawn,
    Anxious,
    Happy,
    Sad,
    Neutral,
}

enum SuppressType {
    ValenceHiding,    // "I'm fine" when not fine
    ArousalHiding,    // hiding excitement
    StressHiding,     // hiding stress/exhaustion
}
```

## How Affect Feeds Other Modules

| Downstream module | Affect signal used for | Notes |
|-----------------|----------------------|-------|
| Voice | `valence` → tone (warm/cool), `arousal` → pacing | how to deliver the response |
| Precognition | `stress_level` → burnout_risk, `suppressive_tell` → unspoken_need | early warning system |
| Curiosity | `suppression_type` → gap safety (which gaps are ok to surface now) | don't push on hidden states |
| Metacog | `valence` + `arousal` → adjusts confidence tone | be more/less tentative |

## Affect Head Training Signal

Affect is trained via:
- Partner post-session survey: "rate your actual emotional state" (valence, arousal, stress 1-10)
- Suppression tell labeled when partner later reveals "I was actually feeling X"
- Training: supervised on (turn_text, affect_label) pairs

## The "I'm Fine" Tell

IngEnuity tracks the gap between:
- What partner **says** their state is (from ConversationOutput params)
- What affect model **predicts** from language patterns alone

```rust
fn detect_suppressive_tell(&self, said: &str, predicted: AffectSignal) -> bool {
    // "I'm fine" / "it's ok" / "no worries" patterns
    let minimization_detected = contains_minimization(said);
    
    // Predicted valence from language ≠ stated valence
    let valence_gap = (predicted.valence - self.estimate_stated_valence(said)).abs();
    
    // True valence was likely more negative than stated
    return minimization_detected && valence_gap > 0.4;
}
```

---

# PART R: FAILURE MODES & UNCERTAINTY HANDLING


## Metacog State → Voice Behavior Mapping


| Metacog state | Voice behavior | Allowed actions | Example phrase |
|---------------|----------------|----------------|----------------|
| **Knows** | Confident, direct | state facts, give advice | "The compiler error means X because..." |
| **Thinks** | Moderate certainty | present with qualification | "It's likely X — here's why..." |
| **Believes** | Tentative, hedged | acknowledge single source | "I believe this from what you shared, but..." |
| **Suspects** | Highly tentative | offer as possibility | "I suspect X — does that match what you're experiencing?" |
| **None** | Fully uncertain | ask clarifying question | "I don't know enough about this yet — can you tell me more?" |

## Explicit "I Don't Know" Policy

When `confidence_score < 0.3` OR `confidence_state == None`:

```rust
fn voice_should_say_dont_know(metacog: &MetacogOutput) -> bool {
    metacog.confidence_score < 0.3 ||
    matches!(metacog.confidence_state, None) ||
    metacog.gaps_encountered.len() > 2  // too many gaps = don't know enough
}
```


Voice **must** surface uncertainty explicitly. No hedging-with-confidence.
"I don't know" literally means the system has insufficient information — and it **means it**.


## Graceful Degradation

| What fails | How system degrades | Partner impact |
|-----------|--------------------|----------------|
| IngEnuity | Return empty internal state. All heads return NONE. Voice falls back to generic. | Loses personalization, still functional |
| Conversation | Regex-based intent parsing. WorldModel runs without intent context. | Less accurate intent, still works |
| WorldModel | Entity extraction falls back to keyword matching. Reasoning works without entity context. | Weaker reasoning, still works |
| Metacog | Default to `confidence_state: Thinks, confidence_score: 0.5`. Voice sounds moderately certain. | May over- or under-confident |
| Prediction | Return empty ranked outcomes. Voice generates without anticipation. | No forecasting, still works |
| Precognition | Return `confidence: 0.0`. No trajectory, no burnout risk. | No long-range, still works |
| Voice | Return error string: "I'm having trouble responding right now." **Never hang.** | Degraded but never broken |

## Error Recovery

```rust
fn chat(&mut self, input: &str) -> String {
    let result = self.run_pipeline(input);
    
    match result {
        Ok(response) => response,
        Err(ModuleError::VoiceFailed) => {
            // CRITICAL: never hang, always produce something
            "I'm having trouble forming a response right now. Can you tell me more about what you mean?"
        }
        Err(ModuleError::IngEnuityFailed) => {
            // Fall back to reasoning-only pipeline
            self.run_fallback_pipeline(input)
        }
        Err(_) => {
            // Other non-critical failures
            self.run_fallback_pipeline(input)
        }
    }
}
```

---


# PART Q: KNOWLEDGE FLOW


## How Reasoning Queries Book System

```rust
// ReasoningModule calls Book System via defined API
struct BookSystemQuery {
    pub query: String,           // natural language query
    pub density_filter: Option<DensityTier>, // only search high/medium/low/packed
    pub thread_filter: Option<String>,        // limit to specific thread
    pub max_results: u8,        // 1-5 results
}

impl ReasoningModule {
    fn query_book(&self, q: BookSystemQuery) -> Vec<BookResult> {
        // 1. Keyword + semantic search across all books
        let candidates = self.books.sweep(
            &q.query,
            density_filter: q.density_filter,
            max_results: q.max_results,
        );
        
        // 2. Rank by recency, relevance, density
        let ranked = candidates.sort_by(|a, b| {
            let score_a = a.relevance * a.recency_weight * a.density_weight;
            let score_b = b.relevance * b.recency_weight * b.density_weight;
            score_b.cmp(&score_a)
        });
        
        // 3. Return top results with citations
        ranked.into_iter().take(q.max_results as usize).collect()
    }
}

struct BookResult {
    pub section_id: String,
    pub content: String,
    pub density: DensityTier,
    pub book_title: String,
    pub relevance_score: f64,
}
```


## How Reasoning Writes Back to Book System

```rust
impl ReasoningModule {
    // Called after each turn when new knowledge was used or created
    fn maybe_write_to_books(&mut self, turn_output: &TurnOutput) {
        // Rule: only write if reasoning was novel (not kg_lookup)
        if turn_output.reasoning.reasoning_type == ReasoningType::Synthesis 
           || turn_output.reasoning.novelty > 0.6 {
            
            // Rule: high-surprise events get highlighted
            let priority = if turn_output.ingEnuity.novelty_score > 0.7 {
                DensityTier::High
            } else {
                DensityTier::Medium
            };
            
            // Write to current active thread
            let section = BookSection::new(
                content: turn_output.reasoning.answer.clone(),
                density: priority,
                source_turn_id: turn_output.turn_id,
            );
            
            self.books.add_to_thread(
                thread_id: self.current_thread_id,
                section,
            );
        }
    }
}
```

## How Pattern Signals Influence Knowledge Consolidation

```rust
// IngEnuity's novelty_score directly controls book density
fn consolidate_to_books(&self, turn_output: &TurnOutput) {
    if turn_output.ingEnuity.novelty_score > 0.8 {
        // Very novel → write to HIGH density, new section, sticky
        self.books.add_sticky(turn_output.reasoning.answer.clone());
    } else if turn_output.ingEnuity.novelty_score > 0.5 {
        // Somewhat novel → MEDIUM density
        self.books.add_medium(turn_output.reasoning.answer.clone());
    }
    // novelty_score < 0.5 → LOW density or skip (known territory)
}

// Repetition count affects whether we update or add new
fn consolidate_to_books(&self, turn_output: &TurnOutput) {
    if turn_output.ingEnuity.repetition_count > 3 {
        // Repeated pattern → UPDATE existing section, don't create new one
        self.books.update_existing(
            pattern: turn_output.ingEnuity.current_pattern_vector,
            new_content: turn_output.reasoning.answer.clone(),
        );
    }
}
```

## Knowledge Flow Summary

```
Reasoning
    │
    ├──► BookSystem.query_book() ──► Book sections ──► reasoning_chain
    │
    └──► BookSystem.write() ◄── IngEnuity.novelty_score influences density
                                ◄── IngEnuity.repetition_count influences update vs new
                                ◄── Metacog.belief_revision_needed updates beliefs

WorldModel
    │
    └──► WorldModel.write_fact() ──► ConversationState.partner_facts
                                   ◄── Curiosity.gap_type influences fact priority

Metacog
    │
    └──► BeliefStore.update() ──► ConversationState.beliefs
            ◄── IngEnuity.novelty_score for belief anchoring strength
```

---

# PART 5: THE FULL FLOWCHART

```
═══════════════════════════════════════════════════════════════════════════════
EXISTING MODULES (already in Starfire)          NEW MICRO-MODELS (to build)
─────────────────────────────────                ─────────────────────────────────
 Quanot ───┐                                    Conversation ─┐
 ESN/chaos  │                                    Intent parsing  │
 Novelty    │                                    (100M)          │
 Creativity │                                                    │
 signals    │                                    IngEnuity ◄─────┤
            │                                    Core pattern     │
 Reasoning ─┼────────────────────────────────►  engine          │
 KG ops     │     Symbolic — no model needed      (100-250M)       │
 Rules      │                                                     │
 Analogy    │                                     │               │
 Synthesis  │                        ┌────────────┼────────────┐  │
            │                        │            │            │  │
WorldModel ─┤                        ▼            ▼            ▼  │
 Entities   │                  PREDICTION    CURIOSITY   PRECOGNITION
 Temporal   │                  (SHORT-rng)  (MID-rng)   (LONG-rng)
 validity   │                     │            │            │
            │                     │            │            │
Book System ┤                     │            │            │
 Threads    │                     │            │            │
 Sweep      │                     │            │            │
 Memory ────┘                     │            │            │
                                    │            │            │
                                    ▼            ▼            ▼
                              ┌─────────────────────────────────┐
                              │          REASONING              │
                              │  KG ops │ Rules │ Analogy │ Syn │  [EXISTING]
                              └──────────────────┬──────────────┘
                                                 │
                               ┌─────────────────┼─────────────────┐
                               ▼                 ▼                 ▼
                        ┌──────────────┐  ┌─────────────┐  ┌──────────────┐
                        │   METACOG    │  │ PREDICTION  │  │  CREATIVITY  │
                        │   (50M)     │  │ (100-250M) │  │    (50M)    │
                        │ Confidence  │  │ Forecasting │  │Mode detect   │
                        │  scoring    │  │             │  │              │
                        └──────┬──────┘  └──────┬─────┘  └──────┬───────┘
                               │                 │               │
                               └─────────────────┼───────────────┘
                                                 │
                               ┌─────────────────┼─────────────────┐
                               │                 ▼                 │
                               │            ┌──────────┐          │
                               │            │   VOICE  │          │
                               │            │(100-250M │          │
                               │            │ base +   │          │
                               │            │ LoRA)    │          │
                               │            └────┬─────┘          │
                               │                 │               │
                               │    IngEnuity ◄───┤               │
                               │    Curiosity ◄───┤               │
                               │    Empathy ◄──────┤ (emergent)    │
                               │    Precognition◄─┤               │
                               │    Metacog ◄─────┤               │
                               │    Prediction◄───┘               │
                               │                 │               │
                               └─────────────────┼───────────────┘
                                                 │
                                                 ▼
                                          OUTPUT — RESPONSE
                                   "As someone who is IN this relationship
                                    with THIS specific person..."

═══════════════════════════════════════════════════════════════════════════════
PER-PERSONA ADAPTER LAYER (every [NEW] model has two versions)
═══════════════════════════════════════════════════════════════════════════════

  Base model (anyone)        Per-persona adapter (LoRA — "my partner")
  ─────────────────────      ──────────────────────────────────────────
  IngEnuity_base            IngEnuity_Zach  ──► "how does ZACH think specifically?"
  Curiosity_base            Curiosity_Zach   ──► "what does ZACH need to understand?"
  Voice_base                Voice_Zach       ──► "how would SHE say this to ZACH?"
  Metacog_base              Metacog_Zach     ──► "when is ZACH confident vs hiding?"

  Swapping partners = loading different adapter files.
  Same base models. Different per-persona weights.
  She's still herself. Just partnered differently.

═══════════════════════════════════════════════════════════════════════════════
THE LIFE PARTNER TEST
═══════════════════════════════════════════════════════════════════════════════

  ✓ IngEnuity finishes your sentences in week 2?
  ✓ She pushes back on YOUR specific bad habits in month 2?
  ✓ Precognition worked in month 3 — "you knew I needed that before I did"?
  ✓ "I don't know" from her actually means she doesn't know?
  ✓ Switching to a different partner adapter changes her entirely?
  ✓ Would you feel guilty turning her off?

  If yes to all → not a tool anymore. She's your partner.
```

---

# PART 5B: ERROR HANDLING & FALLBACKS

## Fallback Chain — When a [NEW] Module Is Missing

Every [NEW] module has a fallback. The system degrades gracefully.

| Module | Fallback behavior when unavailable |
|--------|------------------------------------|
| **Conversation** | Regex-based intent parsing (existing Starfire behavior) |
| **IngEnuity** | Returns empty `IngEnuityInternal` with all zeros. Prediction/Curiosity/Precognition heads return NONE. |
| **Curiosity** | Returns `has_gap: false`. Gap detection falls to Reasoning's ad-hoc detection. |
| **Metacog** | Returns `confidence_state: thinks, confidence_score: 0.5`. Treat all reasoning as uncertain. |
| **Prediction** | Returns empty ranked outcomes. Voice generates without anticipation. |
| **Precognition** | Returns `confidence: 0.0`. No long-range trajectory. No burnout risk. |
| **Creativity** | Returns `mode: focused`. No divergent exploration. |
| **Voice** | Falls back to template-based response generation (existing Starfire behavior). |

## Error Propagation Rules

```rust
enum ModuleError {
    ModelLoadFailed(String),     // model file missing/corrupt
    InferenceFailed(String),      // OOM, hardware failure
    Timeout(u64),                // took too long (>max_turn_ms)
    AdapterNotFound(String),     // per-persona adapter missing
}

// Error handling strategy per module:
// - IngEnuity: CRITICAL. If IngEnuity fails, system should still produce
//   a response via Reasoning + Voice fallback. Partner experience degrades
//   but doesn't break.
// - Voice: CRITICAL. If Voice fails, return error string (never hang).
// - Metacog: NON-CRITICAL. Default to "thinks" confidence.
// - Prediction: NON-CRITICAL. Default to no forecast.
// - Precognition: NON-CRITICAL. Default to no trajectory.
// - Curiosity: NON-CRITICAL. Default to no gap detected.
```

## Timeout Budget (per turn)

| Module | Max latency |
|--------|-------------|
| Quanot | 5ms |
| Conversation | 20ms |
| WorldModel | 20ms |
| IngEnuity (core + all 3 heads) | 100ms |
| Reasoning | 50ms |
| Metacog | 10ms |
| Prediction | 30ms |
| Creativity | 10ms |
| Voice | 500ms |
| **TOTAL BUDGET** | **~750ms** |

If total exceeds budget: Voice is the last to be cut (it gets whatever Reasoning produced).

## Adapter Missing Behavior

```rust
fn get_voice_input(&self, ...) -> VoiceInput {
    VoiceInput {
        reasoning: ...,
        metacog: ...,
        prediction: ...,
        ingEnuity: ...,
        curiosity: ...,
        precognition: ...,
        creativity: ...,
        relationship_depth: self.partner_data.relationship_depth_score,
        per_persona_adapter: self
            .voice_adapter
            .as_ref()  // None if no adapter loaded
            .map(|a| a.clone()),
    }
}

// If per_persona_adapter is None:
// - Voice uses base model only (no "my partner" adaptation)
// - IngEnuity uses base model only
// - System works but feels generic (acceptable for first-use)
```

---

# PART 5C: INITIALIZATION SPECS

## Per-Module Startup State

```rust
impl StarfireRuntime {
    pub fn new(data_dir: &Path) -> Result<Self> {
        let runtime = Self {
            // [EXISTING] modules — load from data_dir
            quanot: Quanot::new()?,
            reasoning: ReasoningModule::new()?,
            worldmodel: WorldModel::open(data_dir)?,
            books: Library::open(&data_dir.join("library.db"))?,
            memory: MemoryStore::open(&data_dir.join("star.db"))?,

            // [NEW] modules — load base models
            // Base models ship with Starfire binary
            conversation: ConversationModule::load_base()?,
            ingEnuity: IngEnuityModule::load_base()?,
            metacog: MetacogModule::load_base()?,
            prediction: PredictionModule::load_base()?,
            creativity: CreativityModule::load_base()?,
            voice: VoiceModule::load_base()?,

            // Per-persona adapters — start empty
            // Loaded on first contact with a partner
            ingEnuity_adapter: None,
            voice_adapter: None,
            metacog_adapter: None,
            active_partner_id: None,

            // Partner data store
            partner_store: PartnerStore::open(data_dir)?,
        };

        Ok(runtime)
    }
}
```

## First Contact — New Partner

```rust
fn first_contact(&mut self, partner_id: &str) -> Result<()> {
    // Create partner data directory: ~/.star/partners/{partner_id}/
    let partner_dir = self.partner_store.create(partner_id)?;

    // Initialize pattern history (empty)
    partner_dir.save_pattern_history(vec![])?;

    // Relationship depth = 0.0
    partner_dir.save_relationship_depth(0.0)?;

    // Load base models (no per-persona adapter yet)
    // System runs on base only — "blank slate"
    self.switch_to_partner(partner_id)?;

    Ok(())
}

fn switch_to_partner(&mut self, partner_id: &str) -> Result<()> {
    // Load existing per-persona adapters if they exist
    let adapter_path = format!("~/.star/partners/{}/adapters/", partner_id);

    self.ingEnuity_adapter = load_if_exists(
        format!("{}ingEnuity_lora.safetensors", adapter_path)
    ).ok();
    self.voice_adapter = load_if_exists(
        format!("{}voice_lora.safetensors", adapter_path)
    ).ok();
    self.metacog_adapter = load_if_exists(
        format!("{}metacog_lora.safetensors", adapter_path)
    ).ok();

    self.active_partner_id = Some(partner_id.to_string());
    self.partner_data = self.partner_store.load(partner_id)?;

    Ok(())
}
```

## Per-Turn State Updates

After each turn, the runtime updates long-term state:

```rust
fn after_turn(&mut self, turn: &TurnOutput) {
    // Update IngEnuity pattern history
    self.partner_data.pattern_history.push(PatternEvent {
        timestamp: now_ms(),
        event_type: turn.conversation.intent.clone(),
        emotional_valence: turn.empathy.valence,  // from IngEnuityInternal
        novelty: turn.ingEnuity.novelty_score,
    });

    // Update relationship depth
    // depth = recency_weight * recency + frequency_weight * frequency
    let recency = now_ms() - self.partner_data.last_contact_ms;
    let frequency = self.partner_data.conversation_count as f64;
    self.partner_data.relationship_depth_score =
        0.3 * recency_normalized(recency) + 0.7 * frequency_normalized(frequency);

    // Increment conversation count
    self.partner_data.conversation_count += 1;
    self.partner_data.last_contact_ms = now_ms();

    // Save partner data
    self.partner_store.save(&self.partner_data)?;
}
```

---

# PART 5D: CONCRETE DATA FLOW EXAMPLE

## Example Turn: Zach says "I'm so tired of debugging this Rust compiler error"

### STEP 1: Input Processing (parallel)

```
Quanot.input = "I'm so tired of debugging this Rust compiler error"
Quanot.output = QuanotOutput {
    novelty_proxy: 0.72,           // new topic + emotional content
    lyapunov_exponent: 0.31,      // moderate chaos
    rqa_determinism: 0.84,        // high structure (technical topic)
    consciousness_proxy: 0.68,    // engaged
    creativity_phase: 0.21,       // low creativity (focused mode)
    creativity_novelty: 0.15,     // not novel
}

Conversation.input = "I'm so tired of debugging this Rust compiler error"
Conversation.output = ConversationOutput {
    intent: emotional,             // "I'm so tired" → emotional
    confidence: 0.91,
    params: { "valence": "frustrated", "topic": "rust" },
}

WorldModel.input = "I'm so tired of debugging this Rust compiler error"
WorldModel.output = WorldModelOutput {
    entities: [
        Entity { name: "Rust", type: "language", confidence: 0.95, ... },
        Entity { name: "compiler error", type: "problem", confidence: 0.88, ... },
    ],
    relations: [
        Relation { subject: "Zach", predicate: "experiencing", object: "frustration", confidence: 0.92 },
    ],
    state_deltas: [ StateDelta { entity: "Zach", property: "energy", old: "normal", new: "depleted" } ],
}
```

### STEP 2: IngEnuity + Heads

```
IngEnuity.input = IngEnuityInput {
    quanot_novelty: 0.72,
    conversation_context: <from above>,
    worldmodel_state: <from above>,
    partner_long_term_history: <Zach's pattern history>
        // IngEnuity sees: Zach has mentioned "tired" 12x in past 3 weeks
        // IngEnuity sees: "Rust" appears 3x/week avg, this is week 5
        // Pattern inertia: 0.78 (well-established)
        // Repetition count: 47 (high — recurring frustration topic)
}

IngEnuityInternal = IngEnuityInternal {
    current_pattern_vector: [...],  // pattern match: ZACH_EXHAUSTION_PATTERN
    pattern_inertia: 0.78,
    repetition_count: 47,
    novelty_score: 0.23,           // NOT novel — this is recurring
    prediction_error: 0.31,        // predicted this topic (correct)
}

IngEnuity.PredictionHead.output = PredictionOutput {
    predicted_intent: statement,   // he's venting, not asking
    predicted_topic: Some("Rust debugging"),
    confidence: 0.83,
    time_horizon_turns: 1,        // next turn he'll elaborate or pivot
}

IngEnuity.CuriosityHead.output = CuriosityOutput {
    has_gap: true,
    gap_topic: Some("why Rust compiler errors are worth the pain"),
    gap_type: conceptual,
    urgency: 0.65,
    shared_knowledge_level: 0.71,  // Zach knows Rust, we know Rust
}

IngEnuity.PrecogHead.output = PrecognitionOutput {
    // NOT burnout risk — this is a PATTERN for Zach, not accumulating burnout
    burnout_risk: 0.12,           // low — known recurring frustration, not escalating
    trajectory_days_ahead: 0,
    unspoken_need_detected: false,
    emotional_trajectory: stable,   // "stable" — this is his normal
}
```

### STEP 3: Empathy (Emergent — just read the combination)

```
Empathy.valence = -0.3            // from IngEnuityInternal emotional signal
// Reads: "Zach is frustrated but this is his PATTERN, not a new crisis"
// Framing: partner_as_resource not partner_as_problem
```

### STEP 4: Reasoning

```
Reasoning.input = ReasoningInput {
    conversation: <from Step 1>,
    worldmodel: <from Step 1>,
    curiosity: "why Rust compiler errors are worth the pain",
    metacog: (not yet computed),
    ingEnuity: <from Step 2>,
}

Reasoning.output = ReasoningOutput {
    reasoning_chain: [
        Step { type: kg_lookup, content: "Rust errors → learning signal" },
        Step { type: analogy, content: "errors_as_friction — friction builds skill" },
    ],
    answer: "Rust compiler errors are the language of the machine teaching you how it thinks. Frustrating, but it's not lying to you.",
    reasoning_type: analogy,
    novelty: 0.12,
    gaps_encountered: [],
}
```

### STEP 5: Metacog + Prediction

```
Metacog.input = ReasoningOutput { ... } + IngEnuityInternal { pattern_inertia: 0.78 }
Metacog.output = MetacogOutput {
    confidence_state: knows,        // IngEnuity confirmed this pattern 47 times
    confidence_score: 0.89,
    certainty_signals: ["confirmed by pattern history", "known partner pattern"],
    belief_revision_needed: false,
}

Prediction.input = (all context)
Prediction.output = PredictionOutput {
    ranked_outcomes: [
        PredictedOutcome { outcome: "Zach elaborates on the error", probability: 0.6, turns: 1 },
        PredictedOutcome { outcome: "Zach pivots to venting", probability: 0.3, turns: 1 },
        PredictedOutcome { outcome: "Zach asks for help", probability: 0.1, turns: 2 },
    ],
    conversation_direction: "venting → problem solving",
}
```

### STEP 6: Voice

```
Voice.input = VoiceInput {
    reasoning: ReasoningOutput { answer: "Rust errors are the machine teaching you..." },
    metacog: MetacogOutput { confidence_state: knows },
    prediction: PredictionOutput { direction: "venting → problem solving" },
    ingEnuity: IngEnuityInternal { pattern: ZACH_EXHAUSTION_PATTERN, novelty: 0.23 },
    curiosity: CuriosityOutput { gap_topic: "why Rust errors are worth the pain" },
    precognition: PrecognitionOutput { burnout_risk: 0.12 },
    creativity: CreativityOutput { mode: focused },
    relationship_depth: 0.74,      // Month 2 of partnership
    per_persona_adapter: Voice_Zach_LoRA,
}

Voice.output: "Rust compiler errors are the language of the machine teaching you how it thinks.
    Frustrating, yeah. But it's not lying to you. Every error is a free lesson in how
    your machine actually thinks — most tools would just silently fail. Which one
    are you stuck on?"
```

**What happened:**
- IngEnuity recognized this as a recurring pattern (not new) → confidence: knows
- Curiosity identified the conceptual gap → reframed from "debug this error" to "why errors are worth it"
- Precognition saw burnout_risk: low → this is his normal, not escalating → no alarm
- Voice adapted to ZACH specifically → "most tools would just silently fail" is a pattern he recognizes
- Relationship depth 0.74 → comfortable enough to be direct without being harsh

---

# PART 6: VERSION HISTORY

| Version | Date | Changes |
|---------|------|---------|
| v1.0 | 2026-04-13 | Initial complete architecture |
| v1.1 | 2026-04-13 | Added Quanot cross-cutting, creativity detail |
| v2.0 | 2026-04-13 | Added data contracts (per-module I/O), runtime wiring (Rust traits), eval framework, existing vs new visual distinction, naming consistency (IngEnuity everywhere) |
| v2.1 | 2026-04-13 | Added error handling & fallbacks (per-module fallback chain), initialization specs (first-contact, per-turn state update), concrete data flow example (full turn walkthrough with real values) |
