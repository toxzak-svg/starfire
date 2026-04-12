# Starfire + Model Architecture Research Notes

**Date:** 2026-04-11
**Author:** Marble

---

## Current Situation

### Hardware Constraints
- AMD Radeon integrated GPU (8GB VRAM shared with system RAM)
- No dedicated VRAM — everything is shared memory
- KV cache at large context sizes = OOM crash
- Bonsai Q1_0 at 65k context: KV cache alone is 4.6GB → OOM → CPU fallback → ~0.5 tok/s (unusable)

### What's Working Right Now
- **Dolphin3.0-Qwen2.5-3b** loaded in LM Studio at port 1234
- Context capped at 2048 (fits in iGPU VRAM)
- Speed: ~5 tok/s on generation, ~15s for 100 tokens (acceptable)
- Uncensored, coherent, Q4_K_M quantization

### LM Studio Status
- Server running on port 1234
- Model loaded: `dolphin3.0-qwen2.5-3b`
- Context length: 2048
- Device: iGPU VRAM (not CPU)

---

## The Core Problem

**Bonsai Q1_0 is fundamentally wrong for this hardware.**
- It's a 1-bit model (BPW = 1.13), extremely low quality ceiling
- At large context it blows the KV cache and falls back to CPU
- Even at small context, Q1_0 quality is poor

**Starfire's http_llm path is pointed at the wrong port (8081 instead of 1234).**
- http_llm.rs defaults to localhost:8081
- LM Studio runs on localhost:1234
- Model name hardcoded to "bonsai-8b" instead of "dolphin3.0-qwen2.5-3b"

---

## Architecture Paths

### Path A: Quick Fix (LM Studio + Starfire http_llm)
- Rebuild Starfire with `http_llm` feature enabled (not `llm`)
- Wire to LM Studio at port 1234 with Dolphin3.0
- Works tonight, no native integration needed

**Pros:** Works immediately, fully local, fast enough for demos
**Cons:** LM Studio is still a middleman

### Path B: Native Candle Path (Real Fix)
- Fix the broken `llm-server` (starfire's own server):
  - Broken tokenizer (uses byte-level approximation, not real tokenizer)
  - Hardcoded `Device::Cpu` (no GPU acceleration)
  - No sampling (argmax only)
- Then Starfire can own the whole stack

**Pros:** No middleman, direct hardware control, fully self-owned
**Cons:** Has real bugs to fix, more work

---

## Zach's Goals

1. **Make Starfire work NOW** with available hardware
2. **Understand Starfire's architecture** (document all of it)
3. **Build a purpose-built model** that is:
   - Uncensored
   - Fast on integrated GPU
   - High quality
   - GPU-free / laptop-capable
   - Personal (trained on his data, not internet)

---

## Personal Benchmark

**Not formally defined yet.** Current informal tests:
- Can it answer 2+2 = 4?
- Can it explain the sky coherently?
- Does it know about Sky (cat)?
- Does it sound like Starfire?

**Recommended:** Define a proper benchmark before validating anything:
- "10-minute coherent conversation about my projects"
- "Answer code questions better than ChatGPT baseline"
- "Remember context from session start to end"

---

## Training Own Model

### Reality Check
Training a 3B model from scratch on a laptop — even with LoRA — will take days and bottleneck on CPU. With integrated GPU: ~5-10 tok/s if lucky. Not practical as a primary approach.

### Smarter Approach: Build Small, Quantize Aggressive

```
Step 1: Start with a tiny base (SmolLM3-360M or Qwen2.5-0.5B)
Step 2: LoRA fine-tune on YOUR data (ChatGPT + GitHub + Starfire memories)
        → Fits in RAM, runs on CPU, done in hours
Step 3: Aggressive quantization (Q1_0 or custom binary)
Step 4: Validate with personal benchmark
Step 5: If you need bigger — stack models, not scale them
```

### Why This Might Beat Anything Out There
- Every big model (ChatGPT, Claude, Gemini) is trained on generic internet data
- Your data is personal, contextual, specific to YOUR life
- A 360M model trained deeply on your world >>> a 70B model trained on everything
- circuit_lm's corrector network idea is perfect for this — fix the small model's mistakes rather than scaling up

### What's Actually Cheaper Than Thought Possible
- Training on your data with LoRA: ~$0 if you use your own machine
- Quantizing with your own approach: free
- The real cost is time + validation

---

## Model Source Opinions

Zach hates relying on someone else's base model. Reality:
- Everyone starts from a base (even big labs use architecture insights + scaled pretraining)
- The question isn't "whether to start from a base" — it's "what you do AFTER"

### Options That Respect the Hate

1. **Train from scratch**
   - Generate your own pretraining data or use public datasets (The Stack, SlimPajama)
   - Compute-intensive — not recommended on laptop

2. **Start minimal, modify heavily**
   - Take SmolLM3-135M (smallest viable base)
   - Apply LoRA fine-tuning on your data
   - Apply aggressive novel quantization (custom binary approach)
   - At each step you're transforming it

3. **circuit_lm approach**
   - Stack multiple small specialized models with corrector networks
   - Novel architecture, not just weights
   - Probably Zach's sweet spot

### Best Source Right Now
**SmolLM3-360M** from HuggingFace — fully public, good quality, tiny enough to run on CPU

---

## Gemma4 Hackathon

**Context:** $200K prize pool, deadline May 18 2026, offline AI deployment focus

**Reality check:** It's for discovering new methods, not just using existing ones. Zach's assessment: worth entering if he discovers something novel in his approach, not as a primary goal.

---

## What Starfire Does (Current)

Starfire's chat pipeline:
```
KG Reasoning → VoiceEngine → LLM Polish → Response
```

- KG reasoning generates "rough" response
- VoiceEngine shapes it into Star's voice
- LLM Polish (either native Candle or remote HTTP) turns rough → natural fluent text
- Only runs if polish backend is available

The polish step is where LM Studio/Dolphin would come in once wired up.

---

## Next Steps (Priority Order)

1. **Make Starfire work with LM Studio** (tonight)
   - Rebuild with `http_llm` feature
   - Fix default URL from 8081 → 1234
   - Fix model name from "bonsai-8b" → "dolphin3.0-qwen2.5-3b"
   - Test Starfire chat with Dolphin3.0

2. **Document Starfire architecture fully**
   - How KG reasoning works
   - How TCMW-A temporal memory works
   - How voice shaping works
   - How the LLM integration fits in

3. **Define personal benchmark**
   - Write it down formally
   - Use it to validate every model change

4. **Explore circuit_lm + novel quantization approach**
   - This is the long-term architecture play
   - Stack small models with corrector networks

5. **Consider SmolLM3-360M as starting point**
   - Fully public, tiny, CPU-capable
   - Transform it with LoRA + custom quantization

---

## File Locations

- Starfire repo: `C:\Users\Zwmar\.openclaw\workspace\projects\starfire\`
- LM Studio models: `C:\Users\Zwmar\.lmstudio\models\`
- LM Studio server: port 1234
- Bonsai GGUF: `C:\Users\Zwmar\.star\models\bonsai-8b\Bonsai-8B.gguf` (1.1 GB, Q1_0)
- Dolphin3.0 GGUF: `C:\Users\Zwmar\.lmstudio\models\itlwas\Dolphin3.0-Qwen2.5-3b-Q4_K_M-GGUF\`

---

## Key Files

- `lib/http_llm.rs` — HTTP LLM client (currently pointing at wrong port 8081)
- `lib/llm/polish.rs` — Polish layer (has `#[cfg(feature = "llm")]` path that bypasses http_llm)
- `lib/runtime/mod.rs` — Runtime with `llm: Option<LlmHandle>` and `llm_engine: Mutex<Option<LlmEngine>>`
- `lib/llm/mod.rs` — Native Candle LLM engine (has bugs, not used when http_llm enabled)
- `plans/PEDI_PLAN.md` — PEDI (Predictive Emergent Desktop Intelligence) architecture
- `plans/QUANT_EVOLVER_PLAN.md` — Quantization evolver plan

---

## Quantization Reference

| Format | BPW | Quality |
|--------|-----|---------|
| Q1_0 | 1.13 | Terrible |
| Q2_K | ~2 | Bad |
| Q4_K_M | 4.5 | Good |
| Q5_K_M | 5.5 | Very good |
| Q8_0 | 8.5 | Near-lossless |

Target for laptop: Q4_K_M or lower