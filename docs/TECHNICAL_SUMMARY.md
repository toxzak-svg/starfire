# Starfire Technical Summary — 2026-04-15

## Project Overview
**Starfire** is a modular AGI architecture written in Rust. Its goal: emergent desktop intelligence without cloud dependencies or GPU requirements.

**What Ships Today:** 20.6MB binary with 30+ modules, 266 unit tests, Railway-deployed web UI. Reasoning, planning, memory, curiosity — all work. Text generation — **BROKEN** (see below).

> ⚠️ **Critical Status**: Bonsai-8B was removed from Starfire on 2026-04-13. Starfire has **NO local inference** and cannot generate unassisted text without an external LLM API. This is the central problem to solve.

---

## Architecture

### Layer 1 — Persistence
- `persistence/`: SQLite-backed store (memories, beliefs, goals, aspirations)
- `learning/`: Concept formation from observations
- `memory/` domains: Identity, Empirical, Procedural, Episodic

### Layer 2 — Reasoning
- `reasoning/`: Knowledge graph with entities + weighted relations
- `knowledge/`: Seed knowledge injection, fact extraction
- `causal/`: Causal graph building and querying
- `metacog/`: Self-model beliefs, uncertainty quantification
- `curriculum/`: Knowledge gap tracking and learning scheduler

### Layer 3 — Meta-Cognition
- `curiosity/`: CuriousEngine — idle self-probing, curiosity probes every 2min
- `tcmw_a/`: TCMW-A — Anticipatory Temporal Causal Memory Weaving (validity windows, OAFL loop, prefetch, auto-confirm)
- `context/`: ContextFusion — ring-state attractor for temporal context
- `prediction/`: PredictionCenter — foresight engine

### Layer 4 — Emergence
- `runtime/`: Orchestrates all layers, session management, command handling
- `conversation/`: Conversation memory across turns
- `personality/`: Identity-congruent personality expression
- `voice/`: VoiceEngine — shapes tone/style of expression

### Support Modules
- `book/`: Library system — hierarchical knowledge storage
- `crumbs/`: CrumbStore — distributed memory (local + GitHub Gist sync)
- `quanot/`: Reservoir computing (128 input dim, 1000 reservoir)
- `world_model/`: Grounded perceptual representation
- `goals/`: Hierarchical goal memory
- `capabilities/`: FileReader, WebReader, WebSearcher (file system + HTTP)
- `asru/`: **[NEW]** Anticipatory Self-Reshaping Update — regime controller

### Text Generation
- `http_llm.rs`: HTTP client calling external LLM server at `LLM_ENDPOINT` env var (default `127.0.0.1:1234`)
- **No local generation** — requires external API endpoint

---

## What Works
- ✅ Full CLI with 20+ commands (/read, /search, /learn, /think, /goals, etc.)
- ✅ Knowledge graph with 100+ seeded facts
- ✅ Persistent memory across sessions (SQLite)
- ✅ TCMW-A anticipation engine with OAFL feedback loop
- ✅ Curiosity engine (background thinker)
- ✅ Quanot reservoir
- ✅ File reading, web search, web fetch
- ✅ 266 unit tests passing
- ✅ Railway-deployed web UI
- ✅ Self-diagnostic `star doctor` command

---

## The Core Problem

**Starfire cannot generate text without an external LLM API.**

The original design used Bonsai-8B via Candle (local GGUF inference). That was removed on 2026-04-13. The current options to restore text generation:

### Option A: Keep External API (Status Quo)
Use `http_llm.rs` to call Groq/Minimax/OpenAI. This works today but:
- Requires API key + internet
- Cloud dependency
- Cost per token

### Option B: ASRU + HGSEL (The Claimed Path)
Replace LLM dependency with a regime-routed sparse expert system:
- ASRU (Anticipatory Self-Reshaping Update): regime controller
- HGSEL (Hash-based Sparse Expert Layer): 64 tiny FFN experts, k=2 active per token
- ASRU controls HGSEL via **salt parameter** — changes expert routing without retraining

**This is the stated goal. Here is the reality:**

---

## Ambition vs. Bandwidth Gap

### What ASRU+HGSEL Claims to Do
> Replace an LLM with a regime-routed sparse expert pool — no training, no cloud, fully local, runs on any laptop

### What's Actually Built
| Component | Status |
|---|---|
| ASRU regime classifier | Built (Rust) — heuristic rules, not learned |
| ASRU fragility estimator | Built (Rust) — Lyapunov + RQA metrics |
| ASRU engine (two-timescale loop) | Built (Rust) — compiles, not integrated |
| HGSEL expert bank (Python) | Built — 64-expert sparse dispatch |
| HGSEL multi-hash router (Python) | Built — deterministic, no learned params |
| car_small SSM (840K) | In lib/ — DonkeyCar control, not text gen |
| Regime → salt mapping | Designed on paper — unvalidated |
| Regime classifier (learned) | Not built |
| Generation head | Not built |

### Specific Gaps to Close

**1. No generation head exists.**
car_small (840K params) is in the codebase but is a DonkeyCar controller, not a text generator. It needs either:
- Retraining on text data for text generation, OR
- Swapping in a pretrained SSM (Mamba-130M, RWKV-100M)

**2. HGSEL is Python, not Rust.**
The HGSEL expert bank + multi-hash router exists in `hgsel-moe/` as a Python package. A Rust port would be needed to integrate into starfire's binary — or the architecture needs a Python subprocess bridge.

**3. ASRU is not wired into Runtime.**
ASRU engine compiles but has zero calls from Runtime. The regime classifier + fragility estimator + salt computation pipeline is untested in the actual system.

**4. Salt → expert routing mapping is unvalidated.**
The theoretical mapping from AFI + regime + viscosity → salt → expert activation exists on paper only. No experiments confirm this produces coherent text.

**5. TCMW-A predictions not wired to ASRU.**
TCMW-A predicts user actions (regime transitions) before they happen. This anticipatory signal should feed into ASRU's slow loop — but the wiring doesn't exist yet.

---

## What ASRU+HGSEL Would Require to Build

### Phase 1: Generation Head (1-2 weeks)
- Integrate pretrained SSM as text generator (RWKV-100M or Mamba-130M via candle)
- OR retrain car_small on text generation data
- Validate: can it produce coherent short responses?

### Phase 2: HGSEL Integration (2-3 weeks)
- Port HGSEL expert bank to Rust, OR
- Build Python subprocess bridge for HGSEL + call from starfire runtime
- Integrate regime classifier output → HGSEL salt parameter
- Validate: salt changes produce different expert activation patterns?

### Phase 3: ASRU Wired into Pipeline (1-2 weeks)
- Hook ASRU engine into Runtime.chat()
- Connect TCMW-A predictions → ASRU anticipatory signals
- Validate: regime shifts produce measurable plasticity changes

### Phase 4: End-to-End (1 week)
- Full generation: input → regime classify → AFI → salt → HGSEL → output
- Benchmark against external API quality
- Tune salt mapping via hill climbing

**Estimated total: 6-8 weeks for a first working prototype.**

---

## Realistic Expectations

**What a first ASRU+HGSEL prototype can achieve:**
- Short responses (1-3 sentences) in narrow domains
- Regime-appropriate tone (emotional vs. analytical vs. factual)
- No cloud, no GPU, CPU-only
- Quality will be below a 1B-parameter transformer initially

**What it won't achieve (without sustained iteration):**
- General-purpose conversation at ChatGPT level
- Long coherent documents
- Complex reasoning requiring multi-hop chains

**The value proposition:** Fully local, no API cost, regime-adaptive routing. If the quality target is "useful on laptop without internet" — achievable. If the target is "beat GPT-4" — not realistic on this timeline.

---

## Orphaned Modules (Cleaned Up 2026-04-15)

**Removed:**
- `grammar_corrector/` — broken ONNX, no LLM feature
- `fabqrc/` — archived (FABQ-RC was dead end)
- `llm-server/` — Bonsai-only, orphaned
- `models/bonsai-8b/` — Bonsai deleted
- `http_llm` feature — removed from default (wasn't wired)

**Still needs investigation:**
- `reflex/` — unknown if used
- `research/` — unknown if used
- `multimodal/` — stub, no implementation
- `fact_lock/` — unknown if used
- `input_normalizer/` — large but possibly legacy

---

## Key Contacts
- **Zachary Maronek (Zach)**: GitHub toxzak-svg
- **Starfire**: `projects/starfire/` (Rust, Apache 2.0)
- **HGSEL-MoE**: `projects/hgsel-moe/` (Python, Apache 2.0)
- **Research Evolver**: `projects/research_evolver/` (Python)

---

## Open Questions for Third Party

1. Which pretrained SSM to use as generation head — RWKV-100M or Mamba-130M?
2. Build HGSEL as Rust module or Python subprocess bridge?
3. What's the quality bar for "success"? (Short responses? Full conversation?)
4. Timeline expectations — 6-8 weeks is the honest estimate. Is that acceptable?
5. Who owns training/fine-tuning of the generation head?
