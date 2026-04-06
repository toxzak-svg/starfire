# Bonsai-8B Integration Plan
**Status:** Draft — 2026-04-05  
**Goal:** Wire Bonsai-8B Q1_0_g128 into Starfire's conversation layer so it generates real responses instead of hardcoded templates.

---

## Background

### Current Architecture

```
Zachary → Conversation::respond()
              │
              ├── Intent parsing (what type of message?)
              ├── ReasoningEngine (for Q/A — symbolic)
              ├── ResearchWalkabout (for unknown topics)
              └── [RESPONSE = hardcoded templates]
```

**Problem:** `Conversation` never calls `LlmEngine`. All responses are hand-written strings. Bonsai sits idle.

### LmEngine Current State

- Loads `Bonsai-8B.gguf` (1.1 GB Q1_0_g128) via Candle ✅
- Forward pass works (Q1_0_g128 matmul fixed) ✅
- Tokenizer is **fake** — byte-level placeholder returns garbage ✅
- `generate()` / `chat()` / `polish()` exist but produce nonsense ✅
- 245 tests pass ✅

---

## Phase 1: Tokenizer Fix
**Owner:** Marble  
**Goal:** Extract the real BPE tokenizer from the GGUF file.

### What the GGUF Contains
Bonsai's GGUF embeds a GPT-2 style BPE tokenizer:
- `tokenizer.ggml.tokens` — vocab (151,669 tokens)
- `tokenizer.ggml.merges` — BPE merge rules
- `tokenizer.ggml.model` = "gpt2"
- `tokenizer.ggml.add_bos_token` / `tokenizer.ggml.add_eos_token`
- BOS/EOS token IDs

### What Candle Already Provides
`candle-core/src/quantized/tokenizer.rs` has:
```rust
impl TokenizerFromGguf for Tokenizer {
    fn from_gguf(ct: &gguf_file::Content) -> Result<Self>
}
```
This reads the GGUF metadata and builds a real `tokenizers::Tokenizer`.

### Implementation
```rust
// In LlmEngine::new():
let content = gguf_file::Content::read(&mut file)?;
let tokenizer = Tokenizer::from_gguf(&content)?;  // already in candle-core
let eos_token_id = content.metadata.get("tokenizer.ggml.eos_token_id")
    .and_then(|v| v.to_u32().ok());

// Replace tokenize_simple() with:
fn tokenize(&self, text: &str) -> Vec<u32> {
    self.tokenizer.encode(text, false).unwrap().get_ids().to_vec()
}
```

### Files Changed
- `lib/llm/mod.rs` — add tokenizer field, replace `tokenize_simple`

---

## Phase 2: Real Generation Loop
**Owner:** Marble  
**Goal:** Autoregressive token generation with proper sampling.

### What Currently Happens (Broken)
```rust
// Single forward pass, greedy sample of ONE token, return token number as string
let logits = self.model.forward(&input, 0)?;
let next_token = self.sample_token(&logits_v);
Ok(format!("[token {}]", next_token))  // ← garbage
```

### What It Should Do
```rust
// 1. Tokenize prompt
let tokens = self.tokenize(prompt);

// 2. Build KV cache + initial input tensor
let mut tokens = tokens.clone();
let mut index_pos = 0;

// 3. Autoregressive generation loop (up to max_seq_len or EOS)
for _ in 0..max_new_tokens {
    let input = Tensor::new(tokens.as_slice(), &self.device)?
        .reshape(&[1, tokens.len()])?;
    let logits = self.model.forward(&input, index_pos)?;
    let logits = logits.squeeze(0)?;

    // Sample next token (temperature / top_p via LogitsProcessor)
    let next_token = self.logits_processor.sample(&logits)?;

    if next_token == self.eos_token_id {
        break;
    }

    tokens.push(next_token);
    index_pos += 1;
}

// 4. Decode tokens → string
self.tokenizer.decode(&tokens, true).unwrap_or_else(|_| "".to_string())
```

### Candle Has LogitsProcessor
`candle-transformers/src/generation/mod.rs`:
```rust
pub struct LogitsProcessor {
    pub fn new(seed: u64, temperature: Option<f64>, top_p: Option<f64>) -> Self
    pub fn sample(&mut self, logits: &Tensor) -> Result<u32>
}
```

### Files Changed
- `lib/llm/mod.rs` — add tokenizer, eos_token, logits_processor, rewrite `generate()`

---

## Phase 3: Wire Into Conversation Layer
**Owner:** Marble  
**Goal:** Bonsai actually generates responses, replacing/adjusting template responses.

### Where LLM Fits
Not everything goes to the LLM. The `Conversation` layer has distinct response patterns:

| Intent | Current Behavior | LLM Role |
|--------|-----------------|----------|
| Greeting | Hardcoded strings | Minimal — keep templates |
| Question (known) | `ReasoningEngine` answer | LLM polish only |
| Question (unknown) | `ResearchWalkabout` → fallback | LLM gap-filling after research |
| Statement | Template acknowledgment | LLM for substantial statements |
| Command | Template | N/A |

### Integration Points

**Option A — LLM only for polish (lowest risk):**
```
handle_question() → ReasoningEngine → result.answer
                  → if polish_needed: LlmEngine::polish(result.answer)
                  → final_response
```
Pros: surgical, can't break existing flows  
Cons: minimal impact

**Option B — LLM for unknown topics (medium risk):**
```
handle_question() → ReasoningEngine → if low confidence:
                  → LlmEngine::chat(messages) for gap-fill
                  → enriched response
```
Pros: fills genuine gaps, visible improvement  
Cons: needs careful confidence thresholds

**Option C — LLM generates first draft, Star personality templates on top (ambitious):**
```
handle_question() → LlmEngine::chat(messages) → raw response
                  → Star-style framing → final response
```
Pros: most natural, full power of Bonsai  
Cons: biggest change, could lose Star's voice

### Recommendation: Start with Option B
- Low confidence from ReasoningEngine → call `LlmEngine::chat()`
- Merge LLM response with Star's personality framing
- Keep greeting/statement handlers mostly as-is for now

### Files Changed
- `lib/conversation/mod.rs` — add `llm: Option<LlmHandle>` to `Conversation`
- `lib/conversation/mod.rs` — call LLM in `handle_question()` for low-confidence cases
- Potentially add `llm_polish()` to `Response` struct

---

## Phase 4: Polish & Edge Cases
- Handle tokenizer encode/decode errors gracefully
- Max token budget (don't generate novels)
- Streaming response (future — not for MVP)
- Error handling: what if GGUF fails to load? (fall back to templates)

---

## Architecture After Integration

```
Zachary → Conversation::respond()
              │
              ├── Intent parsing
              ├── ReasoningEngine (symbolic, deterministic)
              │         │
              │         └── Low confidence? → LlmEngine::chat() ← Bonsai-8B Q1_0_g128
              │                                     │
              │                          [Tokenizer extracted from GGUF]
              │                                     │
              ├── ResearchWalkabout (for unknown topics)
              └── Response (with optional LLM polish + curiosity)
```

---

## Files Summary

| File | Change |
|------|--------|
| `lib/llm/mod.rs` | Tokenizer extraction, autoregressive generate(), eos_token, LogitsProcessor |
| `lib/conversation/mod.rs` | LlmHandle field, low-confidence LLM call |
| `lib/lib.rs` | (already has `#[cfg(feature = "llm")] pub mod llm`) |

---

## Success Criteria

1. `cargo test -p star` passes (245+ tests)
2. `LlmEngine::chat()` returns readable, coherent text (not token numbers)
3. Starfire responds to questions with actual LLM-generated content
4. Falls back to template gracefully if LLM fails
5. Response latency < 30s on CPU for typical queries

---

## Open Questions

1. **Should Bonsai replace or supplement ReasoningEngine?** Current plan supplements (Option B)
2. **Temperature/top_p settings?** Need to tune for Star's voice
3. **Max new tokens?** Bonsai supports 4096, but CPU latency grows with it
4. **Memory footprint?** LlmEngine loads ~1.1 GB. Keep one instance or spawn per session?
5. **Async?** `LlmEngine` is blocking — conversation layer needs to handle this

---

## Out of Scope (For Now)

- Streaming responses
- Tool use / function calling via Bonsai
- Multi-modal inputs
- Changing the tokenizer (BPE from GGUF is correct)
- Moving away from GGUF format
