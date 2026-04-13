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

## Quanot [EXISTING]

**Receives:**
```
string input_text
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
```
string user_input_text
```

**Produces:**
```rust
struct ConversationOutput {
    intent: IntentType,           // enum: greeting|question|command|
                                   //       statement|emotional|casual|other
    confidence: f64,             // 0.0-1.0 classification confidence
    params: HashMap<String, String>, // intent-specific parameters
}
```

**Feeds:** IngEnuity, Reasoning, Voice

---

## WorldModel [EXISTING — new entity extraction model pending]

**Receives:**
```
string user_input_text
```

**Produces:**
```rust
struct WorldModelOutput {
    entities: Vec<Entity>,         // extracted entities with types
    relations: Vec<Relation>,     // entity → entity relationships
    state_deltas: Vec<StateDelta>, // what changed in world state
    temporal_validity: TemporalWindow, // valid_from / valid_until
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

IngEnuity is the central recognition engine. It has THREE temporal heads:

### IngEnuity Core — Shared Base

**Receives:**
```rust
struct IngEnuityInput {
    quanot_novelty: f64,          // from Quanot
    conversation_context: ConversationOutput,
    worldmodel_state: WorldModelOutput,
    partner_long_term_history: Vec<PatternEvent>, // pattern events over months
    short_term_buffer: Vec<RecentEvent>,            // last N turns
}
```

**Produces (internal state):**
```rust
struct IngEnuityInternal {
    current_pattern_vector: Vec<f64>,   // 512-dim pattern state
    pattern_inertia: f64,              // 0.0-1.0 how established is this pattern
    repetition_count: u32,              // times this pattern has appeared
    novelty_score: f64,                 // 0.0-1.0 how unexpected is this
    prediction_error: f64,              // was last prediction wrong? by how much?
    calibrated_at: i64,                // last calibration timestamp
}
```

### Head 1: Prediction (SHORT-RANGE)

**Receives:** `IngEnuityInternal` + `short_term_buffer`

**Produces:**
```rust
struct PredictionOutput {
    predicted_intent: IntentType,       // what partner will likely do next
    predicted_topic: Option<String>,     // what topic will come up
    confidence: f64,                   // prediction confidence
    time_horizon_turns: u8,             // how many turns until predicted event
}
```

### Head 2: Curiosity (MID-RANGE)

**Receives:** `IngEnuityInternal` + `worldmodel_state`

**Produces:**
```rust
struct CuriosityOutput {
    has_gap: bool,                      // is there a shared knowledge gap?
    gap_topic: Option<String>,          // what the gap is about
    gap_type: GapType,                  // enum: factual|conceptual|procedural|meta
    urgency: f64,                       // 0.0-1.0 how important is this gap
    shared_knowledge_level: f64,         // 0.0-1.0 how much do we collectively understand
}
```

### Head 3: Precognition (LONG-RANGE)

**Receives:** `IngEnuityInternal` + `partner_long_term_history`

**Produces:**
```rust
struct PrecognitionOutput {
    trajectory_days_ahead: u32,         // how many days until predicted need
    predicted_need: String,             // what partner will need
    predicted_avoidance: String,        // what partner will steer away from
    burnout_risk: f64,                  // 0.0-1.0 exhaustion probability
    emotional_trajectory: String,        // rising|falling|stable|cycling
    unspoken_need_detected: bool,       // did we detect unexpressed need?
    unspoken_need_description: Option<String>,
    confidence: f64,
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
    reasoning_output: ReasoningOutput,
    ingEnuity: IngEnuityInternal,         // partner's thinking patterns
}
```

**Produces:**
```rust
struct MetacogOutput {
    confidence_state: ConfidenceState,     // enum: knows|thinks|believes|suspects|none
    confidence_score: f64,                // 0.0-1.0 numeric confidence
    certainty_signals: Vec<String>,        // specific phrases indicating confidence
    belief_revision_needed: bool,          // should I update a prior belief?
    prior_belief_id: Option<u64>,         // which belief to revise
    metacognitive_awareness: f64,          // 0.0-1.0 how aware am I of my own reasoning
}
```

---

## Prediction [NEW]

**Receives:**
```rust
struct PredictionInput {
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
    conversation_direction: String,          // where is this conversation headed
    expected_partner_state: String,          // likely partner emotional state next
    topic_transition_probability: f64,       // 0.0-1.0 will topic shift soon
}

struct PredictedOutcome {
    outcome: String,
    probability: f64,                        // 0.0-1.0
    time_horizon_turns: u8,
}
```

---

## Creativity [NEW]

**Receives:**
```rust
struct CreativityInput {
    quanot: QuanotOutput,                    // phase + novelty signals
    ingEnuity: IngEnuityInternal,           // unexpected connections detected
    conversation: ConversationOutput,        // is this a brainstorming context?
}
```

**Produces:**
```rust
struct CreativityOutput {
    mode: CreativityMode,                    // enum: focused|divergent|transitional
    divergence_level: f64,                   // 0.0-1.0 how many possibilities explore
    suggested_explorations: Vec<String>,     // "what about X?" style prompts
    connection_opportunities: Vec<(String, String)>, // pairs of concepts that might link
    phase: f64,                              // oscillation phase (from quanot)
}
```

---

## Voice [NEW]

**Receives:**
```rust
struct VoiceInput {
    reasoning: ReasoningOutput,              // what to say (content)
    metacog: MetacogOutput,                 // how confident to sound
    prediction: PredictionOutput,            // what's coming next (anticipate)
    ingEnuity: IngEnuityInternal,           // how this partner thinks
    curiosity: CuriosityOutput,              // what this partner needs
    precognition: PrecognitionOutput,        // what they'll need later
    creativity: CreativityOutput,           // creative vs focused mode
    relationship_depth: f64,                // 0.0-1.0 how long have we been partnered
    per_persona_adapter: LoRAWeights,       // "my partner" weights (None = base)
}
```

**Produces:**
```
string response_text  // natural language output
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
