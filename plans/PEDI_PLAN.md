# PEDI — Predictive Emergent Desktop Intelligence

## Overview

PEDI is Starfire's predictive layer. It watches what you do, learns your patterns, and pre-computes what you're likely to need before you ask. The goal: eliminate latency by moving computation to idle time.

Built on top of TCMW-A's temporal infrastructure. Extends it with user identity, knowledge state, and speculative execution.

## Core Loop

```
User Input → User Emulator → Prediction Queue → Speculative Execution Engine
                                      ↓
                           Memory Store (attention patterns + KV compressed states)
                                      ↓
                         Decompress only when wrong
                                      ↓
                         Self-Improving Loop (failure → adaptation)
```

## Architecture Phases

### Phase 1 — Temporal Gap Classification + User Recognition
**Goal:** Detect behavioral context from timing alone. Know when user fell asleep, returned, shifted contexts.

**Files:**
- `tcmw_a/cef.rs` — add `GapEvent` type with classification
- `tcmw_a/aih.rs` — factor gap classification into prediction cone
- `tcmw_a/bge.rs` — add per-user Markov chain (session ID keyed)

**Gap classification rules:**
```
gap_duration + hour_of_day → gap_type
0-5min                   → short_break
5-30min                  → afk
30min-4hr                → session_end
4-12hr + 22-10hr         → sleep (hour-adjusted)
4-12hr + 10-22hr         → likely_sleep (ambiguous)
12hr-3days               → away
3+days                   → return
```

**Output:** Gap event injected into CEF, shifts next AIH prediction cone.

**Test:** Simulate 7am resume after 6hr gap → prediction cone shifts toward "morning check-in", not "deep work".

---

### Phase 2 — Per-User Knowledge State (CrumbStore extension)
**Goal:** Know what each user knows. Don't re-explain things they've already learned.

**New CrumbStore fields:**
```rust
struct Crumb {
    content: String,
    density: Density,
    known_by: HashSet<UserId>,      // NEW: who has been told this
    confidence: f64,                // NEW: how confident they know it
    valid_from: Timestamp,
    valid_until: Timestamp,         // NEW: expiry
}
```

**API:**
- `mark_known(content_id, user_id)` — mark a crumb as known by user
- `check_known(content_id, user_id)` → bool — has user seen this?
- `get_unknown_for_user(user_id)` → Vec<Crumb> — what does user NOT know
- `expiry_sweep()` — clean up expired validity windows

**Test:** Mark a fact as "known" by Zach. Query → returns false until re-marked.

---

### Phase 3 — User Emulator
**Goal:** Predict WHAT computation a user will need, not just what they'll say.

**Components:**
```
UserEmulator
├── BehavioralProfile { user_id, markov_chain, archetype, last_seen, topic_history }
├── ArchetypeClassifier — classify user into behavioral archetype
│   ├── deep_worker (long sessions, complex queries)
│   ├── casual_checker (short sessions, simple queries)
│   ├── morning_routine (consistent time patterns)
│   └── night_owl (late night activity)
├── MarkovChain — next_topic = predict_next_topic(history) based on P(topic_i | topic_j)
└── IntentPredictor — given gap_type + archetype + time → likely computation need
```

**Predictor output:** `Prediction { topic, urgency, attention_signature, precompute_hint }`

**Training:** OAFL loop — when user asks something predictable, decrease λ (longer cone). When unpredictable, increase λ.

---

### Phase 4 — Prediction Queue + Speculative Execution Engine
**Goal:** Consume predictions, execute pre-computation before user asks.

**Prediction Queue:** priority queue of `PredictedComputation` sorted by urgency × confidence.

**Speculative Execution Engine actions:**
1. **Pre-fetch memory** — retrieve compressed attention traces for predicted topics
2. **Merge tokens proactively** — if two semantically similar tokens were seen within the prediction window, merge state before needed
3. **Route representation** — factual → retrieval-augmented, procedural → symbolic, creative → neural
4. **Bidirectional pre-scan** — on long context, pre-scan with bidirectional attention for key entities
5. **Draft from failure history** — draft model trained on verifier's specific failure tokens

---

### Phase 5 — Compressed KV State + Pattern Memory
**Goal:** Memory store that holds compressed attention patterns, not full KV pairs.

**Compression forward:** store `AttentionPattern { query_signature, key_positions, compression_fn }` instead of raw KV.

**Reconstruction:** given query, reconstruct KV state from pattern + correct from actual values if error > threshold.

**Memory store structure:**
```
MemoryStore
├── episode_id → Vec<AttentionPattern>  // per conversation episode
├── pattern_index → Vec<AttentionPattern> // globally indexed by semantic hash
└── user_index → Vec<episode_id>        // which episodes belong to which user
```

---

### Phase 6 — Self-Improving Loop
**Goal:** Every failure teaches the system. No human labels needed.

**Failure tracking:**
```rust
struct FailureTrace {
    prediction: Prediction,
    actual_topic: String,
    error_magnitude: f64,
    computation_trace: Vec<ComputationStep>,
}
```

**After each session:**
1. Analyze failures — where did predictions miss?
2. Update Markov chain weights — increase P(actual | given_history)
3. Update emulator archetype if behavioral shift detected
4. If pattern of same failure → flag for corrector network training

**No gradient updates to main model. Only emulator + predictor adapt.**

---

## Dependencies

```
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6
```

Phase 1 is standalone. Phase 2 depends on Phase 1 (gap events inform knowledge state expiry). Phase 3 depends on Phase 1+2. And so on.

## File Locations

```
projects/starfire/src/tcmw_a/       — temporal layer (Phase 1, 3)
projects/starfire/src/crumb/        — CrumbStore extensions (Phase 2)
projects/starfire/src/user_emulator/ — Phase 3
projects/starfire/src/predictor/    — Phase 4
projects/starfire/src/memory/       — Phase 5
projects/starfire/src/self_improve/ — Phase 6
```

## Testing Strategy

Each phase has unit tests. Integration tests connect phases. Full system test: simulate 1 week of Zach's message patterns, measure prediction accuracy and latency reduction.

## Priority

**Phase 1 first** — most self-contained, immediately useful for temporal awareness, no model changes needed.
