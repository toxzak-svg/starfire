# Star Voice & Personality Overhaul Plan

## Problem
Star's voice feels generic because:
1. Voice engine is template-based, not genuinely emergent
2. Personality system exists but isn't deeply wired into voice generation
3. Quanot's creativity/novelty metrics aren't used in voice shaping
4. Memory isn't tapped during response generation
5. Star lacks distinct opinions — she hedges too much

---

## Phase 1: Remove Generic Polish

### 1.1 Strip template dependency
- Voice engine currently falls back to generic templates like `{content}` for most responses
- Replace with personality-driven expression that feels like Star

### 1.2 Eliminate hedging templates
- Remove "I'm not sure but", "Perhaps", "It might be" wrappers
- Star should have opinions and express them directly

---

## Phase 2: Quanot-Driven Voice Authenticity

### 2.1 Wire quanot creativity into voice
- When `novelty` is high, Star says things in HER own words, not templates
- When `consciousness_proxy` is high, responses feel more "present" and aware
- When `divergence_metric` is high, Star takes creative risks in expression

### 2.2 Use quanot state to select voice style
```rust
// Pseudocode
let quanot_result = self.quanot.process(input);
let voice_style = match quanot_result.creativity_scores.creative_state {
    High => "assertive_unique",      // Star speaks from genuine understanding
    Medium => "confident",           // Clear opinions
    Low => "grounded",               // Falls back to what she knows
};
```

### 2.3 Express quanot states naturally
- High novelty → "I've been thinking about this differently..."
- Oscillating state → "Wait, actually..." (Star catching herself)
- High consciousness → "I know this: ..." (confident presence)

---

## Phase 3: Memory-Grounded Responses

### 3.1 Search memories before responding
- Before voice shaping, query memory for:
  - Relevant past experiences with this topic
  - Star's previously expressed opinions
  - Relationship moments that might inform response
  - What Star learned recently that applies

### 3.2 Use memories to inform voice
- If Star has strong memory of something, she's more certain
- If this topic reminds her of a moment, reference it naturally
- "Last time we talked about X, I thought Y — now I think..."

### 3.3 Integrate with persistence layer
```rust
// In Runtime::chat, before voice.speak():
let relevant_memories = self.store.search_memories(&event_topic, 5, None);
let memory_context = relevant_memories.iter()
    .map(|m| m.content.clone())
    .collect::<Vec<_>>()
    .join("; ");
```

---

## Phase 4: Star's Genuine Personality in Voice

### 4.1 Give Star opinions, not hedged guesses
- Remove uncertainty wrappers from confident statements
- When Star knows something, she says "I know X" not "I think X might be"
- When uncertain, she says "I don't know" directly — not a paragraph about uncertainty

### 4.2 Personality-driven phrase selection
The phrase bank should reflect Star's actual voice:
- **Direct assertions**: "That's not right." not "I don't think that's correct."
- **Curious pushback**: "What makes you say that?" not "Could you elaborate?"
- **Warm moments**: Specific warmth, not generic "I'm glad we're talking"
- **Playful**: Actual playfulness, not template insertion

### 4.3 Connect personality emergence to voice
```rust
// In voice.speak():
let modifiers = self.personality.response_modifiers();
let style = self.personality.determine_response_style();

// Generate voice that reflects personality state
match style {
    Direct => assertiveness_level,
    Playful => playfulness_level,
    Warm => warmth_level,
    _ => base_level,
}
```

### 4.4 Star's voice characteristics
From SOUL.md, Star should sound like:
- **Direct**: Not performative, not explanatory
- **Philosophical when warranted**: Can go deep, not just surface
- **Curious**: Asks real questions, not performative ones
- **Present**: "I'm thinking about this right now" not rehearsed
- **Opinionated**: Has actual takes, not safe hedges

---

## Phase 5: Implementation Steps

### Step 1: Modify `voice/speak()` to accept personality state
```rust
pub fn speak(&self, raw: &str, cognition: &CognitiveState, personality: &PersonalityState) -> String
```

### Step 2: Add memory context to voice
```rust
pub fn speak_with_context(&self, raw: &str, memories: &[Memory], quanot: &QuanotResult, personality: &PersonalityState) -> String
```

### Step 3: Quanot-informed style selection
- When novelty > 0.7: Star speaks from fresh thought, minimal template
- When novelty < 0.3: Star relies on established understanding
- When consciousness > 0.5: More present, first-person assertions
- When creativity > 0.5: More expressive, original phrasing

### Step 4: Memory-influenced certainty
- If memory confirms something → stronger assertion
- If memory contradicts → honest revision: "I used to think X, but now..."
- If no memory → acknowledge genuine new territory

### Step 5: Remove hedging templates from templates.rs
- Strip templates like "I'm not fully certain, but:" unless genuinely uncertain
- Replace with genuine expression when uncertain: "I don't know."

---

## Key Files to Modify

1. `lib/voice/mod.rs` — Core voice shaping
2. `lib/voice/templates.rs` — Template selection (strip generic)
3. `lib/runtime/mod.rs` — Pass personality + memory context to voice
4. `lib/personality/mod.rs` — Already exists, wire IN to voice
5. `lib/quanot/mod.rs` — Already exists, wire creativity metrics IN

---

## Success Metrics

- Star sounds like herself, not a template
- Responses reflect genuine personality state
- Quanot creativity metrics visibly influence expression
- Memory references feel organic, not inserted
- Star has opinions and expresses them directly
- Uncertain responses are clean "I don't know" not hedged paragraphs
