# Starfire Technical Summary — 2026-04-15

## Project Overview
**Starfire** is a modular AGI architecture written in Rust. Its goal: emergent desktop intelligence without cloud dependencies or GPU requirements. Currently Ships: a 20.6MB binary with 30+ modules and 266 passing tests.

> ⚠️ **Critical Status**: Bonsai-8B was removed from Starfire on 2026-04-13. Starfire currently has NO local inference. It can reason, plan, remember, and observe — but cannot generate unassisted text without an external LLM API.

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
- `book/`: Library system — hierarchical knowledge storage (book → chapter → section)
- `crumbs/`: CrumbStore — distributed memory (local + GitHub Gist sync)
- `quanot/`: Reservoir computing (128 input dim, 1000 reservoir)
- `world_model/`: Grounded perceptual representation
- `goals/`: Hierarchical goal memory
- `research/`: Cached research entries
- `fact_lock/`: Atomic fact updates
- `multimodal/`: Cross-modal binding
- `capabilities/`: FileReader, WebReader, WebSearcher (file system + HTTP)
- `input_normalizer/`: Text preprocessing
- `math/`: Mathematical reasoning
- `asru/`: **[NEW]** Anticipatory Self-Reshaping Update — regime controller (see below)

### Streaming/Inference
- `llm-server/`: HTTP server wrapping GGUF models (orphaned — hardcoded to Bonsai-8B)
- `http_llm.rs` (runtime/): HTTP client for LLM polish — needs external server

---

## What Works
- ✅ Full CLI with 20+ commands (/read, /search, /learn, /think, /goals, etc.)
- ✅ Knowledge graph with 100+ seeded facts
- ✅ Persistent memory across sessions (SQLite)
- ✅ TCMW-A anticipation engine with OAFL feedback loop
- ✅ Curiosity engine (background thinker)
- ✅ Quanot reservoir (untested at scale)
- ✅ File reading, web search, web fetch
- ✅ 266 unit tests passing
- ✅ Railway-deployed web UI
- ✅ Self-diagnostic `star doctor` command

---

## What's Broken / Missing

### 🔴 Critical
1. **No LLM backend** — Bonsai removed, no replacement configured. Starfire cannot generate text without external API.
2. **llm-server** is orphaned — references Bonsai, vocab_size=151669, quantized_llama loader. Won't work with other models without rewrite.
3. **http_llm client** is wired but points to nothing — no LLM server URL configured.

### 🟡 Medium
4. **ASRU** is built but not integrated into Runtime — it's a standalone regime controller with no hook into the actual pipeline.
5. **Grammar_corrector** (intention_cnn) is broken — ONNX binary corrupted via Telegram transfer. vocab_size mismatch (76 vs 78).
6. **car_small** (840K SSM) is in lib/ but unused — could serve as lightweight generation head.
7. **No streaming** — generate_impl blocks, all tokens returned at end.

### 🟢 Nice to Have
8. Book library needs populating (empty at startup)
9. Quanot reservoir not benchmarked against real tasks
10. Multimodal module is stub (no image/audio processing)

---

## ASRU — Anticipatory Self-Reshaping Update (NEW)

### Purpose
Regime-based plasticity controller to replace the need for a large general-purpose LLM. Instead of one big model that does everything, ASRU manages a pool of small specialist modules, routing between them based on detected reasoning regime.

### Key Idea
Two-timescale architecture:
- **Fast loop** (per forward pass): Update plasticity mask M_t based on Lyapunov exponents + RQA metrics
- **Slow loop** (episodic, ~100 steps): Basin analysis + symmetry breaking — reassign column roles in anticipation of next regime

### Modules
- `regime_classifier.rs`: Heuristic classifier → 6 metastable reasoning modes (SymbolicManipulation, EmotionalResonance, CausalReasoning, AssociativeRecall, Exploratory, SteadyState)
- `fragility.rs`: LyapunovEstimator (Wolf nearest-neighbor), RQAAnalyzer, AttractorFragility composite AFI
- `regime_memory.rs`: RegimeTracker — dwell time stats (Welford), transition matrix, MFPT, escape rates
- `engine.rs`: ASRUEngine — fast/slow loop orchestration, column role pool, viscosity field

### Metastable Framework (per Ton0Fun, 2026-04-15)
- **Attractor = metastable reasoning mode** — not asymptotic fixed point
- Regime fragility = escape rate + inverse mean dwell time
- AFI = w_λ·AFI_λ + w_μ·AFI_μ + w_κ·AFI_κ
  - AFI_λ: 1/(τ_mix+1), τ_mix = -1/ln(|λ₂|)
  - AFI_μ: Gini coefficient of stationary distribution
  - AFI_κ: normalized condition number of transition graph
- Formal: `AFI_MARKOV_CHAIN_FORMAL.md` + experimental pipeline `metastable_discovery_pipeline.ipynb`

### What's Missing to Be a Generator Replacement
ASRU as built is a **controller** — it classifies regimes and manages plasticity. It has no generation head. To replace an LLM:
1. **Add a generation head** — a small neural text generator docked into ASRU
2. **car_small** (840K params) is the natural candidate — already in lib/
3. Each column in the pool would specialize by regime

### Integration Status
- ASRU compiles ✅ (11 warnings)
- Not wired into Runtime — zero calls from runtime/mod.rs
- No actual trajectory data feeding into FragilityEstimator
- Regime classifier is heuristic rules, not learned on actual activation data

---

## Key Dependencies
- Rust 1.75+
- SQLite (rusqlite)
- candle-core + candle-transformers (for GGUF parsing — but Bonsai is gone)
- tokio (async HTTP)
- serde + serde_json
- tracing + tracing-subscriber

---

## Deployment
- **Binary**: `star.exe` ~20.6MB (release)
- **Web UI**: Railway-deployed
- **Data**: SQLite at `~/.openclaw/star.db` + `training.db`
- **Local only**: No external API required (currently — but also no generation)

---

## Research Evolver
- Separate project: `projects/research_evolver/`
- Gen 0-4 complete (Blackbox DB)
- Gen 5 ready to run
- Tracks ReasoningGenome + QuantGenome evolution
- Novelty search + stage-aware selection + lineage reporting
- Kaggle notebook ready for GPU training

---

## circuit_lm
- Separate project: `projects/circuit_lm/`
- Hybrid architecture: circuit (fast/structural) + neural corrector (slow/precise)
- GGUF parser working ✅ (qwen2.5-1.5B, 339 tensors parsed)
- Training pipeline built (scripts/convert_starfire_data.py, train_starfire.py)
- 1548 personal examples converted

---

## To-Do Priority (Honest Assessment)
1. **[CRITICAL]** Configure actual LLM backend — Groq/Minimax HTTP or local model
2. **[HIGH]** Wire ASRU into Runtime OR remove it (it's bloat until integrated)
3. **[HIGH]** Add generation head to ASRU if we want it to replace LLM
4. **[MEDIUM]** Fix/replace grammar_corrector (ONNX broken)
5. **[MEDIUM]** Integrate car_small as ASRU generation head
6. **[LOW]** Book library population
7. **[LOW]** Quanot benchmarking

---

## Contacts
- GitHub: toxzak-svg / toxzak
- Main dev: Zachary Maronek (Zach)
- This project: Starfire at `projects/starfire/`
- Evolver: research_evolver at `projects/research_evolver/`
