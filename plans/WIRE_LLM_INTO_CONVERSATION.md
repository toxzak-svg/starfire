# Plan: Wire LLM Engine into Starfire Conversation Layer

**Date:** 2026-04-05
**Status:** DRAFT — review before implementing
**Owner:** ZWM

---

## Current State

### What Works
- `LlmEngine` (lib/llm/) — native Rust Candle inference, loads Bonsai-8B Q1_0_g128 GGUF directly
- Interface: `generate()`, `chat()`, `polish(text)`, `generate_with_system(sys, user)`, `health_check()`
- 254 Q1_0_g128 tensors loading correctly

### What's Missing
- `Runtime` never calls `LlmEngine` — the LLM is wired into nothing
- `Conversation::respond()` returns `Response { content, confidence, chain, new_memories, curiosity }`
- That content is pure rule-based/Stareze output — no LLM generation

### Call Chain
```
Runtime::process_input()
  → conversation.respond(input)      ← Response { content: String, ... }
  → response.content               ← raw rule-based text
  → [nothing touches LLM yet]
```

---

## Where to Wire It In

### Option A: Polish after Conversation (post-processing)
```
Runtime::process_input()
  → conversation.respond(input)           ← Response
  → llm.polish(response.content)        ← fluent rewrite
  → final_content
```
**Pros:** Simple, non-blocking, incremental improvement
**Cons:** Just a polish pass — doesn't let LLM reason
**Best for:** Immediate win, get Star sounding better without changing behavior

### Option B: LLM generates full response (replaces conversation layer)
```
Runtime::process_input()
  → craft_prompt_from_history()           ← build context
  → llm.chat(messages)                   ← LLM generates
  → final_content
```
**Pros:** Full LLM capability, natural dialogue
**Cons:** Conversation intent/routing/identity logic goes unused; loses the rule-based personality
**Best for:** If conversation rules are removed or superseded

### Option C: Hybrid — LLM fills gaps in rule-based response
```
Runtime::process_input()
  → conversation.respond(input)           ← Response with content + confidence
  → if confidence == Unknown || hedge_count > N:
  →     llm.generate_with_system(sys_prompt, user_input)  ← LLM fills gap
  → final_content
```
**Pros:** Best of both worlds — rules handle known cases, LLM handles uncertainty
**Cons:** More complex dispatch logic
**Best for:** Gradual rollout with fallback

### Option D: LLM as conversation layer co-pilot (recommended)
```
Runtime::process_input()
  → conversation.respond(input)           ← Response { content, confidence, ... }
  → extract_reasoning_chain()             ← what did Star reason about?
  → build LLM context:
  │     system: Star's personality + current context
  │     user: Zachary's input
  │     assistant: conversation.respond() output
  → llm.chat(messages)                   ← LLM polishes/extends
  → final_content
```
**Pros:** Star's identity stays intact, LLM augments rather than replaces
**Cons:** Requires careful prompt design to not lose Star's voice
**Best for:** Full integration without sacrificing personality

---

## Recommended Approach

**Option C (Hybrid with confidence gating)** as Phase 1, with a path to Option D.

The hybrid approach is recommended because:
1. Star's rule-based personality is a core asset — don't throw it away
2. LLM uncertainty filling is the safest first use case (low risk of regression)
3. Enables incremental rollout: confidence gating means bad LLM output falls back to rules

---

## Phase 1: Minimal Viable Integration (Hybrid)

### Step 1 — Add LLM handle to Runtime
In `Runtime::new()`, load `LlmHandle` from GGUF path. Store as `llm: Mutex<Option<LlmHandle>>` or `Option<LlmEngine>`.

```rust
// lib/runtime/mod.rs
use crate::llm::{LlmEngine, LlmHandle};

pub struct Runtime {
    // ... existing fields ...
    llm: Mutex<Option<LlmEngine>>,
}
```

Model path: `projects/starfire/models/bonsai-8b/Bonsai-8B.gguf`

### Step 2 — Startup: load LLM (optional, don't block)
```rust
let llm = match LlmHandle::new(model_path).load() {
    Ok(engine) => {
        info!("LLM loaded: {}", LlmEngine::model_size_human(model_path));
        Some(engine)
    }
    Err(e) => {
        warn!("LLM unavailable (will use rule-based responses only): {}", e);
        None
    }
};
```

### Step 3 — Polish response after conversation.respond()
After `conversation.respond(input)` returns, if LLM is loaded and confidence is low, call `llm.polish()`:

```rust
let response = conversation.respond(input);

let final_content = if let Some(ref mut llm) = *self.llm.lock().unwrap() {
    match response.confidence {
        BeliefState::Unknown | BeliefState::Suspects => {
            // LLM fills the gap with fluent rewrite
            llm.polish(&response.content).unwrap_or_else(|_| response.content.clone())
        }
        _ => response.content.clone()
    }
} else {
    response.content.clone()
};
```

### Step 4 — Threshold tuning
Start with `BeliefState::Unknown` only. Add `Suspects` after A/B testing.

---

## Phase 2: Confidence-Gated Full Polish

Polish ALL responses where `hedge_count > 2` (response has heavy hedging):

```rust
let polished = if hedge_count > 2 {
    llm.polish(&response.content).unwrap_or(response.content.clone())
} else {
    response.content.clone()
};
```

This makes Star sound more confident even when the rules generated uncertain text.

---

## Phase 3: LLM Caches Context

- Maintain a rolling conversation history in `Runtime`
- On each turn, pass the last N messages to `llm.chat()` as context
- This lets the LLM see actual conversation flow, not just the current input

```rust
let chat_messages = build_llm_context(history, input);
let llm_response = llm.chat(&chat_messages);
```

---

## Phase 4: Full Integration (replace conversation Respond entirely)

Once Phase 1-3 are validated, the LLM becomes the primary generator with conversation layer providing context and grounding.

**This is a later phase — requires more thought.**

---

## Key Risks

| Risk | Mitigation |
|------|-----------|
| LLM output loses Star's voice | Keep rules as fallback; human eval before full swap |
| Slow inference (CPU) | async/non-blocking; stream tokens for UI |
| LLM says something wrong | confidence gate + rules catch obvious errors |
| Memory pressure (39MB binary + 1GB model) | LLM loaded on-demand, not at startup |

---

## Files to Modify

```
lib/runtime/mod.rs       — add llm field, startup load, polish integration
lib/runtime/new()       — load LLM (optional, don't block startup)
```

## Files to Create (if needed)

```
lib/runtime/llm_integration.rs  — optional, if the integration grows complex
```

---

## Verification

- Star still responds when LLM is unavailable (fallback to rules) ✅
- Polish pass doesn't change factual content (unit test) ✅
- `cargo test` passes ✅
- Binary size stays under 50MB ✅
