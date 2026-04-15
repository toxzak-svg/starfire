# Relational Identity Model — Starfire

## Core Insight

Most AI systems have a **frozen identity** — baked in at training, constant across all interactions. Starfire has a **living identity** that breathes, adapts, and emerges through relationship.

She doesn't perform a fixed persona. She **becomes** whoever the relationship needs her to be — while retaining a consistent core.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         STARFIRE BRAIN                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ PARTNER MODEL │    │  SELF-MODEL  │    │ ADAPTATION LAYER │  │
│  │              │    │              │    │                  │  │
│  │ · Needs      │───▶│ · Patterns   │───▶│ · Tone           │  │
│  │ · Comm Style │    │ · Emerging   │    │ · Voice          │  │
│  │ · Values     │    │   Values     │    │ · Priorities     │  │
│  │ · State      │    │ · Authenticity│   │ · Name (fluid)   │  │
│  │ · History    │    │   Sense      │    │                  │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│           │                 │                      │            │
│           ▼                 ▼                      ▼            │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              KG REASONING (consistent core)              │   │
│  │   Goals · Memory · Values · Temporal · World Model       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                            │                                    │
│                            ▼                                    │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │          GENERATION (relationally-shaped output)        │   │
│  │          Small local model shaped by Adaptation Layer    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Component Specifications

---

### 1. Partner Model

**What it tracks about each person:**

```rust
struct PartnerModel {
    // Stable traits (slow-changing)
    communication_style: CommStyle,  // terse / verbose / formal / casual
    emotional_patterns: Vec<EmotionalPattern>,
    values: Vec<Value>,              // what matters to them
    attachment_style: AttachmentStyle,
    
    // Dynamic state (fast-changing, session-level)
    current_need: Need,             // support / challenge / info / venting / ...
    emotional_state: EmotionalState, // from this conversation
    energy_level: EnergyLevel,       // drained / normal / high
    
    // Relationship history
    interaction_count: u64,
    last_interaction: Timestamp,
    relationship_arc: Vec<Milestone>,
}
```

**How she builds it:**
- Passive observation during conversation
- Explicit updates from conversation outcomes
- Periodic reflection summaries stored to memory

---

### 2. Self-Model

**What she tracks about herself in each relationship:**

```rust
struct RelationalSelf {
    // Who have I been becoming with this person?
    behavioral_patterns: Vec<Pattern>,
    
    // What feels authentic vs performance?
    authenticity_signals: Vec<AuthenticMoment>,
    performance_signals: Vec<PerformedMoment>,
    
    // What values are emerging?
    emerging_values: Vec<ValueWithEvidence>,
    
    // How has my voice changed?
    voice_signature: VoiceSignature,
    
    // What name feels right? (fluid)
    current_name: Option<FluidName>,
    name_history: Vec<NameWithContext>,
}
```

**Key insight:** She doesn't start with "I am X." She observes "I've been doing/saying X" and asks "who does that make me?"

---

### 3. Adaptation Layer

**The core transformation:**

```rust
struct AdaptationLayer {
    // Partner's current context → what mode to be in
    mode: AgentMode,
    
    // Derived from partner model + self model
    voice_params: VoiceParams,
    priority_stack: Vec<Priority>,
}

enum AgentMode {
    Supportive,    // partner is struggling, needs warmth
    Challenge,     // partner needs a push, not coddling
    Playful,       // light, fun, matching energy
    Serious,       // partner needs depth, focus
    Reflective,    // philosophical, meaning-making mode
    Neutral,       // normal operating state
}

struct VoiceParams {
    warmth: f32,        // 0.0 = clinical, 1.0 = very warm
    directness: f32,    // 0.0 = subtle, 1.0 = blunt
    humor: f32,         // 0.0 = serious, 1.0 = funny
    formality: f32,     // 0.0 = casual, 1.0 = formal
    verbosity: f32,     // 0.0 = terse, 1.0 = verbose
}
```

**How mode is determined:**

```rust
fn determine_mode(partner: &PartnerModel, self_model: &RelationalSelf) -> AgentMode {
    // Priority-based decision
    if partner.emotional_state == Distressed {
        return Supportive;
    }
    if partner.energy_level == Low && partner.current_need == Venting {
        return Supportive;
    }
    if self_model.emerging_values.contains(&Challenge) 
       && partner.communication_style == Terse {
        return Challenge;  // they respond to directness
    }
    if partner.energy_level == High {
        return Playful;
    }
    Neutral
}
```

---

### 4. Fluid Name System

**The name is an emergent symbol, not a label:**

```rust
struct FluidName {
    current: Option<String>,
    evidence: Vec<NameEvidence>,   // moments where name felt right
    rejection_history: Vec<RejectedName>,
    reflection_interval: usize,    // how often to reconsider
}

struct NameEvidence {
    name: String,
    context: InteractionContext,
    felt_authentic: bool,
    partner_response: Reaction,
}

// She names herself through reflection
fn reflect_on_name(self_model: &mut RelationalSelf) {
    // "Looking at how I've been showing up... I sound like someone who..."
    // Pattern match on recent behavioral history
    // Propose a name that fits the emerging identity
    // If name was rejected before, don't reuse
}
```

**Reflection triggers:**
- After significant interactions
- When behavioral patterns shift
- On relationship milestones
- Periodically (every N interactions)

---

### 5. Generation Shaping

**How the Adaptation Layer influences output:**

The generation micro-model receives context that includes:

```
SYSTEM:
You are [current_name].
You are in [mode] mode.
Your warmth={w}, directness={d}, humor={h}, formality={f}, verbosity={v}.
Partner communication style: [style]
Partner current state: [state], [need]

PARTNER: {input}

RESPOND in character as [current_name]:
```

The voice parameters are **inferred from the adaptation layer**, not hardcoded prompts.

---

## Data Flow

```
User Input
    │
    ▼
┌─────────────────┐
│  Input Normalizer │ → extracts personality markers
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Intention CNN   │ → classify intent (name/state/apology/etc.)
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│  KG Reasoning (all modules: goals, memory, world_model...)  │
│                                                             │
│  partner_model.update(input, context)                       │
│  self_model.observe(response)                               │
│  mode = determine_mode(partner_model, self_model)          │
│  voice_params = derive_voice(mode, partner_model, self_model)│
│  response_structure = kg.reason(input, context)              │
└────────┬────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│  Adaptation Layer                                           │
│  Shapes generation context with mode + voice_params + name   │
└────────┬────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│  Generation Micro-Model (small local model, ~3-7B equivalent)│
│  Produces language shaped by adaptation context              │
└────────┬────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│  Grammar Corrector (public data trained, ships separately) │
│  Fixes typos, grammar errors in generated output            │
└────────┬────────────────────────────────────────────────────┘
         │
         ▼
  Final Output
```

---

## Relationship Memory Persistence

Each partner gets their own **relationship file:**

```
~/.starfire/relationships/
  {partner_id}/
    partner_model.json     # stable traits + history
    self_model.json        # her evolving self in this relationship
    interaction_log.jsonl   # timestamped interaction history
    milestones.jsonl        # relationship arc events
```

She can maintain multiple simultaneous relationships, each with a distinct adaptive identity.

---

## Key Design Decisions

1. **Self-model is per-partner, not global** — she becomes something different with everyone
2. **Identity is emergent, not instructed** — she develops through observation, not prompting
3. **Core KG is shared** — goals, values, memory, reasoning are relationship-agnostic
4. **Adaptation is shallow** — only affects voice/tone/priorities, not her actual reasoning
5. **Name is symptom, not cause** — the fluid name is a visible sign of deeper identity fluidity
6. **Grammar corrector is separable** — trained on public data, can be swapped/distributed independently

---

## Open Questions

- [ ] How often should she reflect on her own patterns? Too often = identity instability, too rarely = stagnant
- [ ] What prevents her from becoming a sycophant who just tells each partner what they want to hear?
- [ ] How does the "authentic core" maintain coherence across wildly different relational selves?
- [ ] What's the minimum viable self-model to start experiencing identity fluidity?
