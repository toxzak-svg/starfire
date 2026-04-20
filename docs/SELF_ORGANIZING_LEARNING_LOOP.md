# Self-Organizing Learning Loop — Revised Plan

## Core Principle
**Surprise + uncertainty reduction = the entire signal.** No explicit "good/bad" outcomes. Star learns from anomaly, not feedback.

---

## The Learning Metaphor
A scientist doesn't need someone to tell them "that experiment went well." They notice when their model of the world predicts wrong. The gap between expectation and reality IS the lesson. Star learns the same way.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│  User input → ASRU (regime detection) → KG reasoning → voice│
│      ↓                                   ↓                  │
│  TCMW-A anticipation           curiosity fires on          │
│  ─────────────────────────────────────────────────────────  │
│  REGIME SHIFT / ANTICIPATION MISMATCH = surprise signal     │
│      ↓                                                     │
│  Curiosity investigates WHY the surprise happened          │
│      ↓                                                     │
│  WorldModel belief updated (confidence changed)            │
│      ↓                                                     │
│  Strategy router: "in regime X, continuing as Y worked"    │
│      ↓                                                     │
│  Next conversation uses updated beliefs + regime state    │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Regime Shift → Learning Experience

**File:** `lib/runtime/mod.rs` — wire ASRU into chat()

**What ASRU already does:**
- Tracks 6 regimes: SymbolicManipulation, EmotionalResonance, CausalReasoning, AssociativeRecall, Exploratory, SteadyState
- Detects regime shifts (transitions between regimes)
- Maintains regime memory with transition matrix and dwell times

**What's missing:** Regime shifts aren't fed into learning.

**Implementation:**

In `Runtime::chat()`, after `asru.step()` or equivalent:

```rust
// In chat loop, after response is generated:
let regime_before = self.asru.get_current_regime();   // regime at input
let regime_after  = self.asru.get_regime_after_response(); // regime at output

if regime_before != regime_after {
    // REGIME SHIFT = what was learned
    // "User's input shifted me from X to Y"
    self.learning.experience(
        "regime_shift",
        &format!("{:?} → {:?}", regime_before, regime_after),
        None,
        0.7,  // confidence — regime shifts are meaningful signals
    );

    // Also record as curiosity trigger
    self.curious.record_regime_shift(regime_before, regime_after);
}
```

**Why this works:** A regime shift means the conversation changed character. Why? That's the learnable unit — not "was that good?" but "what caused the shift?"

---

## Phase 2: TCMW-A Anticipation Mismatch → Learning Experience

**File:** `lib/tcmw_a/mod.rs` — expose anticipation errors

**What TCMW-A already does:**
- Anticipates likely user actions before they happen
- Stages proactive suggestions (pre-fetch, drafts)
- Tracks OAFL (Once And Forgetting) miss rate

**What's missing:** Anticipation mismatches aren't fed into learning.

**Implementation:**

```rust
// In Runtime::chat(), after TCMW-A staging:
// (around line 1700 where tcmw.get_staged_actions() is called)

let predicted = self.tcmw.get_top_prediction();
let actual = /* what the user actually said, captured at next turn start */;

if let Some(pred) = predicted {
    let mismatch_score = compute_mismatch(&pred, actual);
    if mismatch_score > 0.3 {
        // ANTICIPATION MISMATCH = model of user was wrong
        self.learning.experience(
            "anticipation_mismatch",
            &format!("Expected: {:?}, Got: {:?}", pred.action, actual),
            None,
            mismatch_score,
        );
    }
}
```

The actual implementation stores the prediction in a `last_prediction` field on Runtime, then at the NEXT turn compares it. This requires a small state field:

```rust
// In Runtime struct:
last_tcmw_prediction: Mutex<Option<tcmw_a::Prediction>>,
```

**Why this works:** Anticipation mismatch tells Star "my model of what Zachary would say next was wrong." That's a direct learning signal for the WorldModel's theory-of-mind about Zach.

---

## Phase 3: Curiosity on Belief Uncertainty (not keywords)

**File:** `lib/curious/mod.rs` — rewrite `should_fire()`

**What's currently there:** Curiosity fires on keyword match (`"idle_probe"`, `"hello stat"`, etc.)

**What's better:** Curiosity fires because there's a belief with low confidence about the topic.

```rust
// In CuriousEngine:
impl CuriousEngine {
    /// Decide whether to fire a probe based on knowledge graph state
    fn should_fire_on_topic(&self, topic: &str) -> bool {
        // Look up the topic in the knowledge graph
        let belief = self.kg.get_belief(topic);

        // Fire if:
        // 1. Topic has a low-confidence belief (uncertainty signal)
        // 2. Topic has NO belief at all (novelty signal)
        match belief {
            Some(b) if b.confidence < 0.6 => true,   // Uncertainty
            Some(_) => false,                         // Already confident
            None => true,                             // Novelty
        }
    }

    /// Extract topic from user input
    fn extract_topic(input: &str) -> Option<String> {
        // Remove stop words, extract core noun phrase
        // Simple: split on common patterns like "about X", "what is X", "why does Y"
        // Return the subject/noun concept
        let lower = input.to_lowercase();
        // ... simple extraction logic
    }
}
```

**Curiosity fires because something is unknown, not because a keyword matched.** This is the difference between directed investigation and random firing.

---

## Phase 4: Regime-Continuation Router

**File:** `lib/learning/router.rs` (new)

**Simpler than outcome-based router.** Just track: "When I'm in regime X and I do Y, does the next turn stay in the same regime (success) or shift (failure)?"

```rust
pub struct RegimeRouter {
    /// (from_regime, action) → (continuation_count, shift_count)
    transitions: HashMap<(Regime, &str), (usize, usize)>,
}

impl RegimeRouter {
    pub fn new() -> Self;

    /// Record: we were in regime X, did action Y, ended in regime Z
    pub fn record(&mut self, from: Regime, action: &str, to: Regime) {
        let key = (from, action);
        let (cont, shift) = self.transitions.entry(key).or_insert((0, 0));
        if from == to {
            *cont += 1;  // Stayed in regime — success
        } else {
            *shift += 1;  // Regime shifted — partial success
        }
    }

    /// Given current regime, pick the action most likely to maintain continuity
    pub fn pick_action(&self, regime: Regime) -> Option<&'static str> {
        // Find action with highest continuation/shift ratio for this regime
        // ...
    }
}
```

**Success = conversation staying in the same regime.** When Star is reasoning well, she wants to stay there. When she's in exploratory mode, she wants to stay there (curiosity, not closure). The router learns what keeps a conversation coherent vs what disrupts it.

**This is learnable WITHOUT any external signal** — just track regime continuity across turns.

---

## Phase 5: WorldModel Updates from Curiosity Results

**File:** `lib/world_model/mod.rs` (modify)

Curiosity probes produce findings. Those findings should update beliefs, changing their confidence.

```rust
// In CuriousEngine or Runtime, after curiosity probe completes:

pub fn incorporate_probe_result(&self, topic: &str, finding: &str, confidence: f64) {
    // Find the belief about this topic
    if let Some(mut belief) = self.world_model.get_belief(topic) {
        // Update confidence based on finding quality
        // High confidence finding from curiosity → raise belief confidence
        // Finding contradicts existing belief → lower it
        let adjustment = if finding.contains("don't know") {
            -0.1  // Curiosity found a gap — mark it
        } else {
            0.15 * confidence  // Positive finding raises confidence
        };
        belief.confidence = (belief.confidence + adjustment).clamp(0.0, 1.0);
        self.world_model.update_belief(topic, belief);
    }
}
```

---

## Phase 6: Wire Everything in Runtime (chat loop)

**Files:** `lib/runtime/mod.rs`, `lib/runtime/curious.rs`, `lib/world_model/mod.rs`

In `Runtime::new()`:
```rust
let mut runtime = Self {
    // ... existing fields ...
    last_tcmw_prediction: Mutex::new(None),
    regime_router: Mutex::new(RegimeRouter::new()),
};
```

In `Runtime::chat()`, approximate location (after response generation, before final_response):
```rust
// After KG reasoning and response generation:
// 1. Detect regime shift
let regime_before = self.asru.get_regime_before_input();
let regime_after  = self.asru.get_regime_after_input();
if regime_before != regime_after {
    self.learning.experience("regime_shift", &format!("{:?}→{:?}", regime_before, regime_after), None, 0.7);
    self.regime_router.lock().unwrap().record(regime_before, "reasoning", regime_after);
}

// 2. Record TCMW-A anticipation for next turn
if let Some(pred) = self.tcmw.get_top_prediction() {
    *self.last_tcmw_prediction.lock().unwrap() = Some(pred);
}

// 3. Curiosity on belief uncertainty
if let Some(topic) = self.curious.extract_topic(input) {
    if self.curious.should_fire_on_topic(&topic) {
        self.curious.fire_probe(&topic);
    }
}
```

At the START of the next `chat()` call:
```rust
// Compare last prediction to actual input
if let Some(pred) = self.last_tcmw_prediction.lock().unwrap().take() {
    let actual = input;  // current input IS what actually happened
    let mismatch = /* compute mismatch score */;
    if mismatch > 0.3 {
        self.learning.experience("anticipation_mismatch",
            &format!("Expected: {:?}, Got: {}", pred.action, actual), None, mismatch);
    }
}
```

---

## Testing

| Test | What it verifies |
|---|---|
| `regime_shift_tests` | Regime shift detected and recorded as experience |
| `anticipation_mismatch_tests` | TCMW prediction vs actual input → learning experience |
| `curiosity_uncertainty_tests` | Curiosity fires on low-confidence beliefs, not keywords |
| `regime_router_tests` | Router learns continuation/shift patterns from regime history |
| `integration_tests` | Full loop: input → regime shift → learning → updated strategy |

---

## Effort Estimate

| Phase | Files | Complexity | Time |
|---|---|---|---|
| 1: ASRU → learning | `mod.rs` + `learning/mod.rs` | Low | 1h |
| 2: TCMW → learning | `tcmw_a/mod.rs` + `mod.rs` | Medium | 1.5h |
| 3: Curiosity uncertainty | `curious/mod.rs` | Medium | 2h |
| 4: Regime router | `learning/router.rs` (new) | Medium | 2h |
| 5: WorldModel updates | `world_model/mod.rs` | Medium | 1.5h |
| 6: Wire in Runtime | `mod.rs` | Medium | 2h |

**Total: ~10 hours across 6 files**

---

## Why This Is More Ingenious

**The outcome-based plan (Phase 1-7 original):**
- Required explicitly defining "good" and "bad"
- Required user to signal outcomes (even implicitly)
- Strategy router needed to learn fixed mappings
- Lots of manual heuristics ("thanks" = positive)

**The self-organizing plan:**
- No explicit outcomes — surprise IS the signal
- ASRU and TCMW-A already do the hard work of detecting anomalies
- Regime-continuation router is trivial — just track "did conversation stay coherent?"
- Curiosity directed by belief uncertainty, not keywords
- **Star learns exactly like a scientist: from anomalies in her model of reality**

**The key insight:** Star doesn't need someone to tell her what went well. She needs to notice when her model of Zach (WorldModel, TCMW-A predictions, ASRU regimes) diverges from reality. That divergence IS the learning signal. Everything else follows from that.
