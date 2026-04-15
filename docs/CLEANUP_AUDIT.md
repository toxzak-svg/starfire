# Starfire Cleanup Audit — 2026-04-15

## Goal
Remove dead/orphaned code and consolidate to a clean, coherent architecture ready for ASRU+HGSEL integration.

---

## Orphan Audit: What's Orphaned or Dead

### 🔴 CRITICAL — Orphaned (no references from Runtime)

| Module | Status | Evidence |
|---|---|---|
| `grammar_corrector/` | **ORPHANED** | In lib.rs behind `#[cfg(feature = "llm")]`, feature gate never enabled. ONNX binary corrupted. Not referenced from Runtime. |
| `car_small/` | **ORPHANED** | In lib/, never imported anywhere. Files: car_small.rs, PLAN.md. Could serve as ASRU generation head. |
| `fabqrc/` | **ORPHANED** | ARCHIVED per MEMORY.md. All 155 TinyLlama layers chose uniform blocksize — dead end. Not imported in Runtime. |
| `http_llm.rs` | **ORPHANED** | In lib/, but Runtime uses `runtime::http_llm::HttpLlmClient` which is missing (file doesn't exist). No actual LLM backend configured. |
| `llm-server/` | **ORPHANED** | Entire directory. Hardcoded to Bonsai-8B (removed). Won't work without Bonsai. `Cargo.toml` exists but unused. |
| `models/bonsai-8b/` | **ORPHANED** | Bonsai model directory — Bonsai deleted from Starfire 2026-04-13. Folder still exists but empty? |
| `multimodal/` | **STUB** | Empty module, no real implementation. Runtime includes it but has no function. |
| `fact_lock/` | **UNKNOWN** | In lib.rs but may not be wired into Runtime. Need to check if used. |

### 🟡 MEDIUM — Potentially Orphaned

| Module | Status | Evidence |
|---|---|---|
| `reflex/` | **UNKNOWN** | In lib.rs. Check if wired into Runtime or just a research module. |
| `research/` | **UNKNOWN** | Same. |
| `input_normalizer/` | Large (865 lines). Check if Runtime actually uses it or is it legacy? | |
| `math/` | Check if math reasoning is actually used or stub. | |
| `capabilities/` | Has FileReader, WebReader, WebSearcher — actually used in Runtime `/read`, `/search`, `/fetch`. | ✅ NEEDED |
| `prediction/` | PredictionCenter in Runtime. Check if actually called. | |

### 🟢 CLEAN — Needed

| Module | Status |
|---|---|
| `persistence/` | ✅ Core — SQLite store, memories, beliefs |
| `reasoning/` | ✅ Core — KG, causal graph |
| `knowledge/` | ✅ Seed knowledge injection |
| `conversation/` | ✅ Conversation memory |
| `metacog/` | ✅ Self-model, metacognition |
| `context/` | ✅ ContextFuser, ring state |
| `curiosity/` | ✅ CuriousEngine, background thinker |
| `tcmw_a/` | ✅ TCMW-A, OAFL loop, auto-confirm |
| `runtime/` | ✅ Core orchestrator |
| `voice/` | ✅ VoiceEngine |
| `quanot/` | ✅ Reservoir computing |
| `world_model/` | ✅ World model |
| `learning/` | ✅ Concept formation |
| `book/` | ✅ Library system |
| `goals/` | ✅ Goal tracking |
| `prediction/` | ✅ Prediction center |
| `curriculum/` | ✅ Knowledge gap scheduler |
| `cognition/` | ✅ Cognitive state |
| `personality/` | ✅ Identity-congruent expression |
| `asru/` | ✅ NEW — regime controller (needs integration) |
| `crumbs/` | ✅ CrumbStore (local + gist sync) |
| `capabilities/` | ✅ FileReader, WebReader, WebSearcher |

---

## Cleanup Plan

### Phase 1: Remove Clear Orphans (No Breaking Changes)

```
REMOVE:
  - grammar_corrector/         (behind feature gate, broken ONNX)
  - llm-server/               (Bonsai-only, broken)
  - models/bonsai-8b/         (empty dir, Bonsai deleted)
  - http_llm.rs               (references missing runtime/http_llm)
  - fabqrc/                   (archived, dead end)
```

Changes to make:
- `lib.rs`: Remove `#[cfg(feature = "llm")] pub mod grammar_corrector;`
- `lib.rs`: Remove `pub mod http_llm;`
- `Cargo.toml`: Remove candle dependencies used only by llm-server
- Delete: `llm-server/` directory
- Delete: `models/bonsai-8b/` directory
- Delete: `grammar_corrector/` directory (unless we fix intention_cnn)
- Delete: `fabqrc/` directory

### Phase 2: Investigate Unknowns

Check which of these are actually used:
- [ ] `input_normalizer/` — check if Runtime.chat() calls it
- [ ] `math/` — check if any module calls it
- [ ] `reflex/` — check if used anywhere
- [ ] `research/` — check if used in Runtime or just curiosity
- [ ] `multimodal/` — confirm it's a stub

### Phase 3: Consolidate

After removals:
- [ ] Run `cargo check -p star --lib` — must pass clean
- [ ] Run `cargo test -p star --lib` — must pass
- [ ] Update docs/TECHNICAL_SUMMARY.md
- [ ] Update lib.rs comments showing clean architecture

### Phase 4: Add HGSEL (New)

After cleanup, add HGSEL integration:
- Clone hgsel-moe into starfire as `hgsel/` Python subproject
- Wire ASRU regime classifier → salt parameter
- Test regime → expert routing mapping

---

## Files to Delete (Confirm)

```
CLEAR_ORPHANS = [
    "lib/grammar_corrector/",
    "lib/fabqrc/",
    "llm-server/",
    "models/bonsai-8b/",
    "lib/http_llm.rs",
]
```

## Files to Investigate Before Deleting

```
UNKNOWN_STATUS = [
    "lib/multimodal/",
    "lib/reflex/",
    "lib/research/",
    "lib/math/",
    "lib/input_normalizer/",
]
```

---

## After Cleanup: Architecture

```
lib/
├── asru/          ✅ Regime controller (NEW)
├── book/          ✅ Library system
├── capabilities/  ✅ FileReader, WebSearcher, WebReader
├── car_small/     ✅ Available (ASRU generation head candidate)
├── causal/        ✅ Causal graph
├── context/       ✅ ContextFuser + ring state
├── conversation/  ✅ Conversation memory
├── crumbs/        ✅ CrumbStore (local + gist)
├── curiosity/     ✅ CuriousEngine
├── curriculum/    ✅ Knowledge gap scheduler
├── cognition/     ✅ Cognitive state
├── goals/         ✅ Goal tracking
├── knowledge/     ✅ Seed knowledge
├── learning/      ✅ Concept formation
├── metacog/       ✅ Metacognition + self-model
├── book/          ✅ Library system
├── personality/   ✅ Identity expression
├── persistence/   ✅ SQLite store
├── prediction/    ✅ PredictionCenter
├── quanot/        ✅ Reservoir computing
├── reasoning/     ✅ Knowledge graph
├── runtime/       ✅ Orchestrator
├── tcmw_a/        ✅ TCMW-A engine
├── voice/         ✅ VoiceEngine
├── world_model/   ✅ World model
└── reflex/        ⚠️ UNKNOWN (check)

Removed:
  ✗ grammar_corrector   (broken ONNX, no LLM feature)
  ✗ http_llm            (references missing module)
  ✗ llm-server/         (Bonsai-only)
  ✗ fabqrc/             (dead end)
  ✗ bonsai-8b/           (deleted 2026-04-13)
```

---

## Next Action
Wait for `cargo check --tests` to finish (running in background calm-shell).
Then: execute Phase 1 deletions.
