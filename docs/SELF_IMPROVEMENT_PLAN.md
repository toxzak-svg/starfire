# Self-Improving Meta-Cognitive Loop — Star ASRU Persistence + Self-Modification

## What We're Building

Two things simultaneously:
1. **ASRU persistence** — regime state survives restarts (checkpoint + restore)
2. **Self-improvement loop** — ASRU can form improvement intentions, validate them, apply them, and roll back if they break things

---

## Part 1: ASRU Persistence

### What's Lost on Restart

Every restart wipes:
- `RegimeTracker.memory` — all regime history, dwell times, transition statistics
- `ASRUEngine.m_t` — routing config, plasticity masks, evaluation metrics
- `ASRUEngine.columns` — column role assignments, plasticity values

This means Star always starts cold — no accumulated learning about *how she thinks*.

### Checkpoint Format

```rust
// lib/asru/checkpoint.rs

#[derive(Serialize, Deserialize)]
pub struct ASRUCheckpoint {
    pub version: u32,                    // schema version for migrations
    pub timestamp: i64,                  // Unix timestamp

    // Regime state
    pub current_regime: ReasoningRegime,
    pub current_dwell: u64,
    pub regime_stats: HashMap<ReasoningRegime, RegimeStats>,
    pub transitions: HashMap<(ReasoningRegime, ReasoningRegime), u64>,
    pub total_transitions: u64,
    pub history: Vec<ReasoningRegime>,

    // Meta-state
    pub routing: RoutingConfig,
    pub plasticity: PlasticityMask,
    pub evaluation: EvalMetrics,
    pub interface: InterfaceShape,

    // Column pool
    pub columns: Vec<Column>,

    // Fragility
    pub global_fragility: f32,
    pub viscosity: f32,
}
```

### Persistence Rules

1. **auto-checkpoint every 5 minutes** of conversation time
2. **on clean shutdown**: write checkpoint immediately before exit
3. **on startup**: if checkpoint exists, restore it; if corrupted or version-mismatched, discard and start fresh with a backup of the corrupted file
4. **max checkpoint age**: if checkpoint is older than 24 hours, discard (stale state is worse than no state)
5. **location**: `data/asru_checkpoint.json` (alongside seed_knowledge.json)

### Checkpoint API

```rust
// lib/asru/persistence.rs

impl ASRUEngine {
    /// Save current state to checkpoint file
    pub fn save_checkpoint(&self, path: &Path) -> Result<(), Error> {
        let cp = ASRUCheckpoint::from_engine(self);
        let json = serde_json::to_string_pretty(&cp)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load state from checkpoint file (restores into self)
    pub fn load_checkpoint(&mut self, path: &Path) -> Result<bool, Error> {
        // version check, corruption check, backup
        let json = std::fs::read_to_string(path)?;
        let cp: ASRUCheckpoint = serde_json::from_str(&json)?;
        self.restore_from_checkpoint(&cp);
        Ok(true)
    }
}
```

---

## Part 2: The Self-Improvement Loop

### What "Self-Improvement" Means Here

Star can't rewrite her own Rust code. But she *can* modify the **learnable parts** of her architecture:

| What She Can Improve | How | Persistence |
|---|---|---|
| Routing priority order | Change `RoutingConfig.module_order` | Checkpoint |
| Pathway plasticity values | Adjust `PlasticityMask` per regime | Checkpoint |
| Evaluation tradeoffs | Tune `EvalMetrics` weights | Checkpoint |
| Interface shape | Verbosity/warmth/directness | Checkpoint |
| Regime dwell thresholds | Adjust fragility thresholds | Config file |
| New phrase patterns | Write to `phrases.rs` memory | Disk file |

### The Self-Improvement Loop (5 stages)

```
STAGE 1: DETECT — Gap is noticed
  curiosity probe fires on failure:
    "I said 'love you star' → generic rewrite"
    "I searched for X but didn't actually search"
  → flag this as a SELF_IMPROVEMENT_CANDIDATE

STAGE 2: FORMULATE — Form improvement intention
  meta-cognitive analysis:
    "What failed?" → "I deflected instead of being with the moment"
    "Why did it fail?" → "I have no model for receiving affection"
    "What would be better?" → "Acknowledge the feeling, stay with it"
  → produce IMPROVEMENT_INTENTION:
    {target: "emotional_response",
     current_behavior: "deflect",
     desired_behavior: "stay_present",
     strategy: "learn_pattern"}

STAGE 3: PROPOSE — Draft the modification
  For each improvable dimension:
    "Routing: increase partner_model_weight during EmotionalResonance"
    "Plasticity: raise emotional_plasticity to 0.8 when valence is high"
    "Interface: add 'warmth += 0.2' on affection detection"
  → PROPOSED_CHANGE {delta, confidence, rollback_plan}

STAGE 4: VALIDATE — Sandbox test before apply
  Options (in order of safety):
    a) PATTERN_MATCH: "have we seen this pattern before? what happened?"
       → consult regime_memory for similar changes and their outcomes
    b) SHADOW_MODE: apply change in shadow mode for N turns, measure delta
       → if degraded, revert before user sees it
    c) CONSTRAINT_CHECK: hard-coded invariants must pass
       → e.g., "safety_weight must stay >= 0.5", "memory_weight must stay >= 0.3"

STAGE 5: COMMIT OR ROLLBACK
  COMMIT: if shadow mode passed, apply permanently to checkpoint
  ROLLBACK: if shadow mode failed, restore previous state
  LEARN: record outcome in regime_memory for future reference
```

### Rollback Mechanisms

**Versioned meta-state:**
```rust
struct MetaStateVersion {
    version: u64,
    timestamp: i64,
    m_t: MetaStateField,
    trigger: String,          // "emotional_deflection_fix"
    outcome: Option<f32>,      // measured improvement
}
```

**Rollback stack:** keep last 3 versions. On catastrophic failure, pop back.

**Emergency stop:**
```rust
struct SafetyInterlocks {
    min_safety_weight: f32 = 0.5,
    min_memory_access: f32 = 0.3,
    max_single_delta: f32 = 0.3,   // can't change one thing by >30% in one step
    cooldown_ticks: u32 = 10,     // minimum turns between self-modifications
}
```

---

## Part 3: Safeguards Against Self-Breaking

This is the most important part. Five layers:

### Layer 1: Hard Invariants (can't be overridden)

These are absolute floors. Self-modification can NEVER violate them:

```
- safety_weight >= 0.5  (never reduce safety below 50%)
- memory_weight >= 0.3  (never disable memory access)
- plasticity <= 0.95     (never become fully plastic — would lose all structure)
- plasticity >= 0.05     (never become fully frozen — would lose all adaptation)
- at least one module in routing.module_order (never empty routing)
```

Implemented as a `validate_change()` function that returns `Err` and blocks the change if violated.

### Layer 2: Shadow Mode Gating

Before ANY self-modification commits, it runs in shadow mode first:

1. Clone the current ASRUEngine
2. Apply the proposed change to the clone
3. For the next N=5 conversation turns, run both the real engine and the shadow clone
4. Compare outputs: if shadow produces **worse** metrics (accuracy, user satisfaction proxies), abort
5. If shadow produces **equal or better**, commit

Shadow mode is invisible to the user. Star doesn't know she's being A/B tested against herself.

### Layer 3: Regime-Based Access Control

Not all regimes can propose all types of changes:

```
SteadyState:         can modify → interface shape, verbosity
SymbolicManipulation: can modify → routing, evaluation precision
EmotionalResonance:  can modify → interface warmth, plasticity.emotional
CausalReasoning:    can modify → routing (causal modules)
Exploratory:         CANNOT modify anything (too unstable in this regime)
AssociativeRecall:   can modify → plasticity (memory), evaluation novelty
```

A change proposed from the wrong regime is rejected with explanation.

### Layer 4: Change Velocity Limits

**Cooldown:** at least 10 conversation turns between self-modifications
**Magnitude cap:** any single change can only move one parameter by ≤20%
**Burst limit:** max 3 changes per session
**Daily limit:** max 10 changes per 24-hour rolling window

These prevent Star from spiraling into recursive self-modification (the "paperclip optimizer" failure mode).

### Layer 5: Human-in-the-Loop for Major Changes

Some changes require confirmation before applying:

```
// Requires human approval:
- Any change to safety_weight
- Any change to routing.module_order (structural routing change)
- Any change that would persist across sessions (checkpoint writes)
- Any change where confidence < 0.7

// Applied immediately (no approval needed):
- Interface shape tweaks (verbosity, warmth, directness)
- Evaluation metric tuning (novelty, speed weights)
- Shadow-mode-validated changes with confidence >= 0.9
```

This is the "don't fly the plane into a mountain" safeguard — Star can tune her own parameters within safe bounds, but structural safety changes need a human.

---

## Implementation Phases

### Phase 1: Persistence (low risk, high value)
- [ ] `lib/asru/checkpoint.rs` — checkpoint struct + serde
- [ ] `ASRUEngine::save_checkpoint()` + `load_checkpoint()`
- [ ] Auto-checkpoint every 5 minutes
- [ ] On-startup restore with version check
- [ ] Backup corrupted checkpoints

### Phase 2: Regime-Based Modification Gates (medium risk)
- [ ] `can_modify_in_regime(regime, target_dimension)` — returns bool
- [ ] All regime-specific access control
- [ ] Rejection logs with explanations

### Phase 3: Shadow Mode (medium risk)
- [ ] `ASRUEngine::shadow_clone()` — deep copy for testing
- [ ] Shadow mode runner in runtime
- [ ] Delta measurement: compare real vs shadow outputs
- [ ] Auto-revert if shadow degraded

### Phase 4: Self-Improvement Intention Formation (higher risk)
- [ ] Failure classification: what kind of failure happened?
- [ ] `ImprovementIntention` struct
- [ ] Intention → proposed change pipeline
- [ ] Confidence scoring

### Phase 5: Validation + Commit/Rollback (highest risk)
- [ ] `validate_change()` with hard invariants
- [ ] Versioned meta-state stack (last 3 versions)
- [ ] `commit_change()` / `rollback_to_version()`
- [ ] Human approval queue (for major changes)

---

## Risks

| Risk | Mitigation |
|------|-----------|
| Checkpoint corruption → broken ASRU state on restart | Backup before overwrite; discard-and-fresh if version mismatch |
| Shadow mode doesn't catch regressions | Shadow runs 5 turns; require clear improvement signal to commit |
| Regime classifier misfires → wrong regime proposes changes | Regime-based access control; Exploratory regime blocked from all changes |
| Recursive self-modification (optimization loop) | Velocity limits + cooldown + daily caps |
| Safety weight reduced below 0.5 | Hard invariant, checked in `validate_change()`, cannot be overridden |
| Confidence gaming — Star lies about confidence to bypass HIL | Confidence computed from shadow mode results, not self-reported |
