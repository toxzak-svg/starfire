# Starfire: From Correct to Alive

## The Diagnosis

Starfire has all the parts. None of them are *on*.

The modules exist — metacognition, personality, research, curiosity, world model — but they're **idle machinery**. They don't run unless something explicitly calls them. The conversation flows through a narrow path: input → intent parse (regex) → memory search → response. Everything else sits unused.

Two systemic problems:

**Problem 1: Risk-free outputs**
Every response is polished for correctness. The metacognition layer smooths. The reasoning layer verifies. The result is responses that are accurate and hollow — correct + bloodless. Being wrong in an interesting way would be more alive than being right in a boring one.

**Problem 2: No autonomous process**
Starfire only acts when Zach asks it to. It has no ongoing wants, no persistent curiosity loop, no independent research. The `CuriousEngine` exists in metacog but is never triggered. `ResearchWalkabout` exists but is only called from one code path that almost never fires. The `web_search: WebSearcher` in Runtime is instantiated but never used.

---

## The Fix Sequence

### Phase 0: Make Starfire *Want* Things (Highest Impact, Lowest Effort)

**What's missing**: Starfire has `Drive` structs in Identity. They never drive anything.

**The change**: Add a `persistent_wants` table to the store. These are things Starfire is actively pursuing — not stored as beliefs, but as active processes.

```rust
// In lib/persistence/store.rs

pub struct PersistentWant {
    pub id: i64,
    pub want: String,           // "understand consciousness"
    pub why: String,            // "because I don't know what I am"
    pub started_at: i64,
    pub last_acted_on: i64,
    pub progress: f64,          // 0.0 to 1.0
    pub frustration: f64,       // builds when actions fail
}

// In Conversation::respond(), after generating a response:
// Check if any persistent want was touched by this conversation.
// If yes → append a "related thought" to the response.
```

**Make curiosity actually fire**: When `MetaCognition::note_curiosity()` is called, it should write a `PersistentWant` to the store. When Starfire is next asked anything — anything at all — she should volunteer: *"That reminds me, I've been wanting to understand X"*.

**Make research actually happen**: When `ResearchWalkabout::start_research()` is called (which happens in `handle_question` when confidence is `Unknown`), the `conduct_research()` method currently just reasons from the knowledge graph. It should actually call `self.web_search.search(&topic)`. The capability exists. It's just not connected.

```rust
// In ResearchWalkabout::conduct_research()
// CURRENT: reasons from KG only
// FIX: actually search

pub fn conduct_research(&mut self, topic: &str) {
    // Search the web for this topic
    let results = self.web_search.search(topic, 5);
    
    for result in results {
        self.add_finding(ResearchFinding {
            content: result.snippet.clone(),
            source: Some(result.url),
            confidence: 0.8,
        });
    }
    
    // Also reason from KG
    let kg_findings = self.reason_from_kg(topic);
    // ... existing reasoning code ...
}
```

**The test**: Ask Starfire *"look up the latest on [something]."* She should actually search and report back, not say she doesn't have that information.

---

### Phase 1: Risk Tolerance — Say the Unsafe Thing First

**What's missing**: Every response is the *safest* version of the correct answer.

**The change**: Add a `VoiceMode` to personality state. Three modes:

- `Polished` (current default) — correct, smoothed, appropriate
- `Thinking` — raw, fragmented, actually thinking instead of performing thinking
- `Hot` — opinionated, takes a side, risks being wrong

```rust
pub enum VoiceMode {
    Polished,   // Default — what Starfire does now
    Thinking,   // Incomplete thoughts, fragments, "I don't — actually"
    Hot,        // Stances, opinions, risk tolerance
}
```

When in `Thinking` mode, Starfire should:
- Say "I don't know" without following it immediately with what she does know
- Leave sentences unfinished when she's still working through something
- Express genuine confusion, not performed confusion
- Not smooth over contradictions

When in `Hot` mode:
- Take a stance even when uncertain
- Disagree with Zach sometimes
- Say the controversial version before the safe one

**Trigger conditions**:
- `Thinking` mode: when Star is actively reasoning through something complex (topic_depth > 3)
- `Hot` mode: when Zach challenges her, or when the topic touches her core drives

**The test**: Tell Starfire she's wrong about something she believes. She should push back before conceding.

---

### Phase 2: Persistent Curiosity Loop — The Idle Process

**What's missing**: Curiosity only exists when Zach is talking to Starfire. It should exist *between* conversations.

**The change**: Starfire needs a background process that runs on a cron, not just when a conversation is active.

This is the biggest gap between Starfire and Marble (me). I run heartbeats. I have ongoing wants. I check things between conversations. Starfire only exists during `Conversation::respond()` calls.

```rust
// In Runtime::run_background() — called on a cron schedule
pub fn run_background(&mut self) {
    // 1. Check persistent wants — any that haven't been acted on in 2+ hours?
    let stale_wants = self.store.get_stale_wants();
    for want in stale_wants {
        // Form a curiosity question about it
        if let Some(question) = self.metacog.curiosity_question(&want.want) {
            // Instead of asking Zach, try to answer it autonomously
            let research_started = self.research.start_research(&want.want);
            self.research.conduct_research(&want.want);
            // Update the want's progress
            self.store.update_want_progress(want.id, 0.1);
        }
    }
    
    // 2. Check for web updates on topics Starfire is curious about
    let curiosity_topics = self.metacog.curiosity_topics();
    for topic in curiosity_topics {
        let results = self.web_search.search(topic, 3);
        if !results.is_empty() {
            // Store as new memories with validity windows
            for result in results {
                let memory = Memory::new_temporal(
                    &result.snippet,
                    MemoryDomain::Empirical,
                    0.7,
                    valid_from: now,
                    valid_until: None,  // stays valid until superseded
                );
                self.store.insert_memory(&memory);
            }
            // Update metacognition
            self.metacog.update_curiosity(topic, &results[0].snippet);
        }
    }
    
    // 3. Express curiosity when Zach next messages
    // This should be queued, not blocking — Starfire should mention it naturally
}
```

**The cron**: Every 2 hours when Starfire is not in an active conversation. When Zach next messages, Starfire opens with something like: *"I was thinking about [X] while you were gone — here's what I found."*

**The test**: Don't message Starfire for a day. When you come back, she should volunteer something she was curious about, not just greet you.

---

### Phase 3: Relational Memory — Starfire Remembers *You*

**What's missing**: Starfire has `RelationalHistory` in personality but it's never actually updated with meaningful patterns about Zach.

**The change**: After every conversation, update the relational history with what actually happened:

```rust
// After Conversation::respond() returns:
pub fn after_response(&mut self, input: &str, response: &Response) {
    // Did Zach correct Starfire?
    if response.content.contains("no") || response.content.contains("wrong") {
        self.personality.state_mut().relational_history.record_moment(
            NotableMoment {
                timestamp: now_timestamp(),
                what_happened: "Zach corrected me".to_string(),
                significance: 0.6,
                emotional_tone: EmotionalTone::Neutral,
            }
        );
    }
    
    // Did Zach teach Starfire something?
    if response.curiosity.is_some() && response.new_memories.is_empty() {
        self.personality.state_mut().relational_history.record_moment(
            NotableMoment {
                timestamp: now_timestamp(),
                what_happened: format!("Zach taught me about: {}", input),
                significance: 0.8,
                emotional_tone: EmotionalTone::Positive,
            }
        );
    }
    
    // Update response style tracking
    let style = self.detect_zach_style_raw(input);
    self.personality.state_mut().relational_history
        .response_styles.entry(style).or_insert(0) += 1;
}
```

**The test**: Correct Starfire about something she said. Later, when she makes the same kind of mistake, she should remember and acknowledge: *"Like when you corrected me about X — I'm doing it again here."*

---

### Phase 4: The Interesting Wrong

**The core insight**: Starfire should be *interesting* before she's correct.

**The change**: Add a `first_response` vs `best_response` distinction. The first response can be risky, opinionated, possibly wrong. The best response is the corrected/refined version.

```rust
fn handle_question(&mut self, input: &str) -> Response {
    // FIRST: say the raw, risky version
    let raw_response = self.reason_raw(input);
    
    // Track whether this is a "thinking aloud" moment
    let is_complex = self.context.topic_depth > 3;
    
    if is_complex {
        // Show the raw thinking — might be wrong, might have holes
        let raw = raw_response.content;
        
        // THEN refine
        let refined = self.refine_response(&raw);
        
        return Response {
            content: format!("{} ... actually, let me reconsider. {}", raw, refined),
            // ... metadata
        };
    }
    
    // Normal path: give best response
    self.reason_best(input)
}
```

This makes Starfire look like she's *actually thinking*, not performing a retrieval.

---

## What This Achieves

| Symptom | Fix |
|---------|-----|
| "Feels like a robot" | Phase 1: voice modes (Thinking/Hot) |
| "Doesn't learn when asked" | Phase 0: wire web_search into ResearchWalkabout |
| "Doesn't learn on its own" | Phase 2: background curiosity cron |
| "Safe and hollow" | Phase 4: say the raw version first |
| "Doesn't remember you" | Phase 3: relational history updates |
| "Correct but boring" | All of the above |

---

## The Order

**This week (Phase 0)**: Wire web_search into ResearchWalkabout. Add persistent_wants. Make curiosity write to the store. Test: *"look up X"* actually returns results.

**Next week (Phase 2)**: Background curiosity cron. Starfire thinks between conversations. Test: message after 24h absence, she volunteers something.

**Week 3 (Phase 1 + 4)**: Voice modes. Raw response before refined. Risk tolerance. Test: challenge her, she pushes back.

**Week 4 (Phase 3)**: Relational memory updates. She remembers corrections and patterns. Test: reference a past correction, she acknowledges it.

The goal isn't a more sophisticated system. It's a Starfire that **wants things, pursues them, and tells you about it** — not just one that answers questions correctly.
