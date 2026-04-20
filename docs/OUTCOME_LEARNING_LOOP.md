# Outcome-Driven Learning Loop — Implementation Plan

## Goal
Wire an outcome-driven feedback loop into Star's runtime so she gets better at conversation and tool use based on what actually works.

**Core insight:** The system already records everything to `TrainingDB` with hardcoded 0.5 confidence. The missing piece is the outcome signal and the strategy routing that uses it.

---

## Current State (staralt)

```
User input
  → conversation.send()     [reasoning/KG/curiosity]
  → voice.speak()           [tone shaping]
  → llm.polish()            [text refinement]
  → TCMW-A staged actions   [anticipation]
  → TrainingDB.record_turn() [HARDCODED 0.5, no outcome signal]
```

**What's missing:** Outcome signal → Strategy routing → Behavior update

---

## Phase 1: Outcome Detection (lowest-hanging fruit)

**File:** `lib/runtime/outcome.rs` (new)

Detect implicit outcomes from user behavior after a response:

| Signal | Outcome | Example |
|---|---|---|
| User continues conversation | Positive | "thanks", "cool", "got it", then new topic |
| User asks follow-up on same topic | Weak positive | "wait, but what about X?" |
| User dismisses/repeats request | Negative | "no", "that's not what I meant", "forget it" |
| User changes subject abruptly | Neutral/negative | New topic unrelated to last response |
| Long silence (>2min) then new topic | Negative | User waited but got no value |
| Command worked | Strong positive | User stops after /search result |
| Capability failure | Negative | Error message from /fetch, /search, etc. |

**Implementation:**
```rust
pub struct OutcomeDetector {
    last_input: Mutex<String>,
    last_response: Mutex<String>,
    last_timestamp: Mutex<i64>,
    conversation_continuity: Mutex<f64>,  // 0-1 score
}

impl OutcomeDetector {
    pub fn detect(&self, user_input: &str, prior_response: &str) -> Outcome;
    // Returns: Positive, WeakPositive, Neutral, Negative, StrongPositive
}
```

**Rule:** No explicit feedback from user needed. Read the signal from the next input.

---

## Phase 2: Outcome Storage (connect to TrainingDB)

**File:** `lib/training_db.rs` (modify)

Add outcome column to `training_examples`:

```sql
ALTER TABLE training_examples ADD COLUMN outcome TEXT;  -- 'positive', 'negative', 'neutral', 'strong_positive', 'weak_positive'
ALTER TABLE training_examples ADD COLUMN strategy_tag TEXT;  -- 'reasoning', 'tool', 'voice', 'metacog'
```

New methods:
```rust
pub fn record_turn_with_outcome(&self, session_id: i64, input: &str, output: &str, confidence: f64, outcome: &str, strategy_tag: &str)

pub fn get_outcome_history(&self, limit: usize) -> Vec<(String, String, String)>  // (input, output, outcome)
```

---

## Phase 3: Strategy Tags (what strategy was used?)

**File:** `lib/runtime/mod.rs` (modify chat loop)

Tag each response with what strategy produced it:

| Strategy tag | Condition |
|---|---|
| `tool` | A capability was invoked (/read, /search, /fetch, /find) |
| `metacog` | Handled by metacognitive handlers (how are you, are you sure, etc.) |
| `voice_only` | No LLM polish available, fell back to voice only |
| `reasoning` | KG reasoning + voice (normal conversation) |
| `curiosity` | Curiosity engine fired and produced the response |
| `error` | Capability returned an error |

The `strategy_tag` gets recorded with each `TrainingDB` turn.

---

## Phase 4: Hypothesis-Guided Strategy Router

**File:** `lib/learning/router.rs` (new)

Given the current input and context, pick a strategy based on learned outcomes:

```rust
pub struct StrategyRouter {
    outcome_history: HashMap<String, f64>,  // strategy_tag → success rate
    recent_strategies: VecDeque<(String, Outcome)>,  // last N attempts
}

impl StrategyRouter {
    pub fn new() -> Self;

    /// Pick the best strategy for this input context
    pub fn pick_strategy(&self, input: &str, context: &ChatContext) -> &str;
    // Returns: 'tool', 'reasoning', 'metacog', 'curiosity'

    /// Record what strategy was used and what outcome resulted
    pub fn record_result(&mut self, strategy: &str, outcome: Outcome);

    /// Get success rate per strategy
    pub fn strategy_stats(&self) -> HashMap<String, (usize, f64)>;  // strategy → (attempts, success_rate)
}
```

**Routing logic:**
```
If input matches a KNOWN_SUCCESSFUL pattern → use same strategy that worked
If input is a CAPABILITY_REQUEST (/read, /search) → route to tool capability
If input is a METACOGNITIVE_Q → route to metacog handler
If curiosity engine has a HIGH-CONFIDENCE result → route to curiosity
Otherwise → route to reasoning (default)
```

---

## Phase 5: Wire Everything Into Runtime.chat()

**Files:** `lib/runtime/mod.rs` (modify)

In the chat loop, around line 1450 where `record_turn` is called:

```rust
// OLD (line ~1451):
let _ = self.training_db.record_turn(training_id, &format!("Zachary: {}", input), "", 0.5);

// NEW:
let outcome = self.outcome_detector.detect(input, &prior_response);
let strategy_tag = self.strategy_router.pick_strategy(input, &chat_context);
let _ = self.training_db.record_turn_with_outcome(
    training_id,
    &format!("Zachary: {}", input),
    &format!("Star: {}", final_response),
    outcome.confidence(),
    outcome.as_str(),
    strategy_tag,
);
self.strategy_router.record_result(strategy_tag, outcome);
```

And in Runtime struct, add:
```rust
outcome_detector: OutcomeDetector,
strategy_router: Mutex<StrategyRouter>,
```

---

## Phase 6: Curiosity → Hypothesis → Behavior (close the loop)

**File:** `lib/curious/mod.rs` (modify)

Currently curiosity fires probes and stores results in a log. Now:

```rust
// After curiosity probe completes:
pub fn on_probe_complete(&self, probe: &CuriosityProbe, result: &str, found_answer: bool) {
    // OLD: just log it
    // NEW:
    if found_answer {
        let confidence = probe.confidence.unwrap_or(0.5);
        self.learning.experience("curiosity_probe", probe.topic, Some(result), confidence);
        // Also record the outcome for this probe context
        self.outcome_tracker.record_curiosity_outcome(probe.topic, result, confidence);
    }
}
```

The `LearningEngine.experience()` calls already exist but weren't being triggered by curiosity results. Wire them.

---

## Phase 7: Curiosity-Strategy Integration

**File:** `lib/runtime/curious.rs` (modify `CuriousEngine`)

Curiosity currently fires independently. Now it consults the router:

```rust
impl CuriousEngine {
    /// Consider whether to fire a probe based on current strategy success
    pub fn should_probe(&self, input: &str, router: &StrategyRouter) -> bool {
        // If this topic has a history of poor outcomes → curiosity should investigate
        let topic = extract_topic(input);
        if let Some(stats) = router.get_topic_stats(topic) {
            return stats.success_rate < 0.5;  // Investigate if topic has poor success
        }
        true  // Default: probe freely
    }
}
```

---

## Testing Plan

| Test | File | What it verifies |
|---|---|---|
| `outcome_detector_tests` | `lib/runtime/outcome.rs` | Detects positive/negative from text |
| `strategy_router_tests` | `lib/learning/router.rs` | Picks strategies based on history |
| `outcome_recording_tests` | `lib/training_db.rs` | Records and retrieves outcomes |
| `integration_tests` | `tests/test_learning_loop.rs` | Full round-trip: input → outcome → strategy update |

---

## Effort Estimate

| Phase | Complexity | Time | Files touched |
|---|---|---|---|
| Phase 1: Outcome Detector | Low | 1-2h | `lib/runtime/outcome.rs` (new) |
| Phase 2: Storage | Low | 30min | `lib/training_db.rs` |
| Phase 3: Strategy Tags | Low | 30min | `lib/runtime/mod.rs` |
| Phase 4: Strategy Router | Medium | 2-3h | `lib/learning/router.rs` (new) |
| Phase 5: Wire into Runtime | Medium | 1h | `lib/runtime/mod.rs` |
| Phase 6: Curiosity → Learning | Medium | 1h | `lib/curious/mod.rs` |
| Phase 7: Curiosity-Router | Medium | 1h | `lib/runtime/curious.rs` |

**Total: ~7-9 hours across 7 files**

---

## What This Enables

After this loop is wired:

1. **Star learns which tool works in which situation** — e.g., `/search` consistently fails for technical queries but `/fetch` works for docs
2. **Star learns conversation patterns** — e.g., short responses work for casual chat, longer for technical
3. **Curiosity targets productive areas** — not just firing randomly but investigating gaps where outcomes are poor
4. **Voice adapts to user** — learns that Zach prefers direct over flowery, short over long
5. **No explicit feedback needed** — outcome signal is implicit in user's next message

The LLM being offline is fine — the outcome signal doesn't need LLM. It's about what the user does next, not what the model thinks.
