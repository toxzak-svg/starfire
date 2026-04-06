# Starfire Doctor — Health Check & Diagnostic CLI

## Overview

`starfire doctor` is a self-diagnostic tool for Starfire — one command that checks the health of all subsystems, reports issues, and optionally applies repairs. Modeled after `openclaw doctor` but scoped to Starfire's internal architecture.

## Usage

```bash
starfire doctor              # Interactive health report
starfire doctor --repair     # Apply recommended fixes
starfire doctor --fix        # Alias for --repair
starfire doctor --non-interactive  # CI mode (skip interactive prompts)
starfire doctor --deep       # Extended checks (clippy, integration tests)
```

## Check Categories

### 1. Build & Tests
- [ ] `cargo build --lib` — compiles without errors
- [ ] `cargo test --lib` — all tests pass
- [ ] `clippy` — no new warnings (baseline: 43 pre-existing)

### 2. File Integrity
- [ ] `star.db` — exists, readable, not empty
- [ ] `training.db` — exists, readable
- [ ] `voice.db` — exists, readable
- [ ] `Bonsai-8B.gguf` — exists, ~1.1 GB, readable

### 3. Identity Seeds
- [ ] `IDENTITY.md` — exists, parses correctly
- [ ] `SOUL.md` — exists
- [ ] Knowledge Graph entity count — above minimum threshold (indicates seed knowledge injected)

### 4. Runtime Health (lightweight initialization test)
- [ ] `Runtime::new()` — completes without panic
- [ ] Store loads — memory count > 0
- [ ] Seed memories — identity, goal, constraint memories present
- [ ] Reasoning engine — KG initialized, seed knowledge injected
- [ ] Metacognition — bootstrapped with self-model beliefs

### 5. API Server (if running on port 8080)
- [ ] `GET /health` → 200 + `{"status":"ok"}`
- [ ] `GET /identity` → valid JSON with name/summary/relationship
- [ ] `GET /memory/stats` → memory_count > 0
- [ ] `GET /cognitive` → valid cognitive state JSON
- [ ] `GET /metacog` → valid metacognition JSON
- [ ] `POST /chat` → valid response from reasoning layer

### 6. LLM Layer (Bonsai-8B GGUF)
- [ ] GGUF file readable — no I/O errors
- [ ] Q1_0_g128 tensor detection — 254 tensors found
- [ ] Model loads via Candle — no quantization format errors
- [ ] Forward pass — model responds to test token batch
- [ ] Tokenizer — proper tokenizer loaded (not byte-level fallback)
- [ ] Text generation — produces readable output (not `[token N]`)

### 7. Subsystem Stats (read-only diagnostics)
- [ ] Curiosity engine — last_probe timestamp, idle_for_secs, pending probes
- [ ] Knowledge Graph — entity count, relationship count
- [ ] Memory domains — breakdown by type (identity, episodic, procedural, etc.)
- [ ] Training DB — conversation count, turns, facts, corrections
- [ ] Quanot — reservoir size, current activity level
- [ ] World Model — entity count, last perception timestamp
- [ ] Goals/Aspirations — count of active goals, long-term aspirations
- [ ] Prediction center — active predictions count

### 8. Autonomy State
- [ ] Goals — persisted goals load correctly
- [ ] Aspirations — persisted aspirations load correctly
- [ ] Curiosity probes — persisted probes load into CuriousEngine

## Auto-Repair Map

| Check | Problem | Repair |
|-------|---------|--------|
| Build | compilation fails | Report error, suggest `cargo build 2>&1` |
| Tests | test failure | Report test name + output |
| star.db | missing | `Runtime::new()` recreates on startup |
| star.db | corrupt | Delete + `Runtime::new()` re-initializes |
| KG empty | entity count = 0 | Re-run `inject_seed_knowledge()` + `inject_self_knowledge_into_kg()` |
| API | not running | Print `starfire api --host 127.0.0.1 --port 8080` |
| LLM | model fails to load | Report GGUF path issue, check candle-core version |

## Output Format

```
🦀 Starfire Doctor — YYYY-MM-DD

[1/8] Build & Tests
  ✅ cargo build --lib     — compiled in 12.3s
  ✅ cargo test --lib      — 253 passed, 0 failed
  ⚠️  clippy               — 43 warnings (pre-existing)

[2/8] File Integrity
  ✅ star.db               — 20 KB
  ✅ training.db           — 32 KB
  ✅ voice.db              — 4 KB
  ✅ Bonsai-8B.gguf        — 1.1 GB (Q1_0_g128)

[3/8] Identity Seeds
  ✅ IDENTITY.md           — exists, 1.2 KB
  ✅ SOUL.md               — exists, 8.6 KB
  ✅ KG entity count       — 47 entities (seeded)

[4/8] Runtime Health
  ✅ Store loads           — 847 memories, 12 beliefs
  ✅ Seed memories         — identity/goal/constraint injected
  ✅ Reasoning engine      — KG initialized with seed knowledge
  ✅ Metacognition         — bootstrapped

[5/8] API Server (port 8080)
  ❌ not running           → starfire api --host 127.0.0.1 --port 8080

[6/8] LLM Layer
  ✅ GGUF readable         — 254 Q1_0_g128 tensors detected
  ✅ Tensor loading        — all 254 tensors loaded correctly

[7/8] Subsystem Stats
  ✅ Curiosity             — last_probe: 4m ago, idle_for: 12s
  ✅ KG                    — 47 entities, 89 relationships
  ✅ Memory                — 847 total
  ✅ Training DB           — 23 convos, 156 turns, 89 facts
  ✅ Quanot               — reservoir: 1000 units, activity: 0.73

[8/8] Autonomy State
  ✅ Goals                 — 2 active
  ✅ Aspirations           — 1 set
  ✅ Curiosity probes      — 4 persisted

─────────────────────────────────────────
Summary: 22/23 checks passed
⚠️  1 warning (API server not running)
✅ Starfire is healthy and ready to run
```

## Exit Codes

- `0` — all checks passed
- `1` — one or more checks failed
- `2` — doctor itself encountered an error (not a Starfire issue)

## Implementation

```
src/
  main.rs              — add `Doctor` variant to `Commands` enum
  bin/
    doctor.rs          — NEW: all diagnostic logic
```

- ~300 lines of Rust
- Uses `tracing` for structured output
- `--repair` writes backups before any mutation
- `--non-interactive` skips all prompts (for cron/CI)
- Runs in < 5 seconds without API server; ~10s with API probes

## Relationship to `openclaw doctor`

`starfire doctor` is **orthogonal** to `openclaw doctor`:
- `openclaw doctor` checks OpenClaw gateway, channels (WhatsApp/Telegram), agents
- `starfire doctor` checks Starfire's internal subsystems (KG, metacog, LLM, Quanot, etc.)

They can run independently. `starfire doctor` does NOT call `openclaw doctor`.

## Status

**Not yet implemented.** Implementation candidate for Starfire v0.2.
