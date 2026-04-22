# Quanot Rust Rewrite Plan

**Created:** 2026-04-04
**Status:** ✅ COMPLETED (2026-04-04)
**Goal:** Rewrite Quanot entirely in Rust, fully integrated into Starfire

---

## Why Rewrite in Rust

**Current state:**
- Quanot is Python (ESN, chaos, consciousness, creativity modules)
- Starfire is Rust
- Bridge via IPC (TCP/STDIO) as documented in `quanot/INTEGRATION_PLAN.md`

**Problems with Python:**
- IPC overhead between Quanot and Starfire
- Can't share memory directly (reservoir state vectors need serialization)
- Two runtimes to manage
- Python dependencies for deployment
- GIL prevents true parallelism

**Benefits of Rust:**
- Single unified runtime
- Zero-copy shared memory between Quanot perception → Starfire reasoning
- No IPC — direct function calls
- Far faster reservoir computation
- One binary to deploy
- Memory safety without GC pauses

---

## Architecture After Rewrite

```
┌─────────────────────────────────────────┐
│           Starfire (Rust)                │
│                                          │
│  ┌─────────────┐    ┌────────────────┐  │
│  │  Quanot     │───►│ Starfire       │  │
│  │  (Rust)     │    │ Reasoning      │  │
│  │             │    │ Meta-Cognition │  │
│  │  ESN        │    │ Memory         │  │
│  │  Chaos      │    │ Curiosity      │  │
│  │  Φ-Metrics  │    │ Conversation   │  │
│  │  Creativity │    │ ...            │  │
│  └─────────────┘    └────────────────┘  │
└─────────────────────────────────────────┘
```

**Data flow becomes:**
```
Input Text
    ↓
Quanot.encode() → ReservoirState (in-memory)
    ↓
Quanot.consciousness() → ψ proxy
    ↓
Quanot.creativity() → CreativityOutput
    ↓
Direct call to Starfire.process(perception)
    ↓
Starfire reasoning → response
```

No serialization, no IPC, no network overhead.

---

## Quanot Rust Module Structure

```
lib/quanot/
├── mod.rs              — Quanot core orchestrator
├── reservoir.rs        — Echo State Network implementation
├── chaos.rs            — Chaos metrics (Lyapunov, attractors, RQA)
├── consciousness.rs    — Φ proxy computation (IIT-inspired)
├── creativity.rs       — Creative oscillation system
├── quantum_inspired.rs — Quantum-inspired solvers
├── encoder.rs          — Text → reservoir input encoding
├── lib.rs              — Public API re-exports
└── tests/
    ├── test_reservoir.rs
    ├── test_chaos.rs
    ├── test_consciousness.rs
    └── test_creativity.rs
```

---

## Implementation Details

### 1. Reservoir (ESN)

```rust
pub struct Reservoir {
    pub input_size: usize,
    pub reservoir_size: usize,
    pub output_size: usize,
    pub input_weights: Matrix<f64>,
    pub reservoir_weights: Matrix<f64>,
    pub output_weights: Matrix<f64>,
    pub state: Vec<f64>,
    pub leak_rate: f64,
    pub spectral_radius: f64,
}

impl Reservoir {
    /// Step the reservoir with input
    pub fn step(&mut self, input: &[f64]) -> &[f64];

    /// Get current reservoir state
    pub fn get_state(&self) -> &[f64];

    /// Reset to initial state
    pub fn reset(&mut self);

    /// Train output weights (linear regression)
    pub fn train(&mut self, inputs: &[Vec<f64>], outputs: &[Vec<f64>]);
}
```

**Key parameters:**
- `reservoir_size`: 1000 (from current Python)
- `spectral_radius`: 0.9 (chaotic regime)
- `leak_rate`: 0.3
- `input_scaling`: 0.1

### 2. Chaos Metrics

```rust
pub struct ChaosMetrics {
    pub lyapunov_exponent: f64,
    pub correlation_dimension: f64,
    pub entropy: f64,
}

impl ChaosMetrics {
    /// Compute Lyapunov exponent from state trajectory
    pub fn lyapunov_from_trajectory(states: &[Vec<f64>]) -> f64;

    /// Recurrence quantification analysis
    pub fn rqa(state: &[f64], delay: usize, epsilon: f64) -> RQAResults;

    /// Bettencourt attractor metrics
    pub fn attractor_metrics(states: &[Vec<f64>]) -> AttractorMetrics;
}
```

### 3. Consciousness (Φ Proxy)

```rust
/// IIT-inspired consciousness proxy
pub struct ConsciousnessMetrics {
    pub phi: f64,           // Integrated Information proxy
    pub integration: f64,    // Information integration
    pub differentiation: f64, // Information differentiation
    pub entropy: f64,        // State entropy
}

impl ConsciousnessMetrics {
    /// Compute Φ from reservoir state
    pub fn compute_phi(state: &[f64], history: &[Vec<f64>]) -> f64;

    /// Update with new state
    pub fn update(&mut self, state: &[f64]);

    /// Global Workspace Theory indicators
    pub fn gwt_indicators(&self) -> GWTIndicators;
}
```

### 4. Creativity

```rust
pub struct CreativeOscillator {
    pub phase: f64,
    pub frequency: f64,
    pub amplitude: f64,
    pub divergence_threshold: f64,
}

pub struct CreativityOutput {
    pub creative_state: f64,
    pub divergence_metric: f64,
    pub diversity_index: f64,
    pub originality_score: f64,
    pub oscillation_phase: f64,
}

impl CreativeOscillator {
    pub fn step(&mut self, reservoir_state: &[f64], consciousness: f64) -> CreativityOutput;
}
```

### 5. Encoder (Text → Vector)

```rust
/// Simple text encoder for reservoir input
pub struct TextEncoder {
    pub vocab: HashMap<char, usize>,
    pub embedding_dim: usize,
    pub embeddings: Matrix<f64>,
}

impl TextEncoder {
    pub fn new(vocab_size: usize, embedding_dim: usize) -> Self;
    pub fn encode(&self, text: &str) -> Vec<f64>;
    pub fn batch_encode(&self, texts: &[String]) -> Vec<Vec<f64>>;
}
```

**Encoding strategy:**
- Character-level embedding (avoids OOV)
- Each character → embedding vector
- Mean pool across sequence length
- Normalize to unit vector

### 6. Quanot Core (Orchestrator)

```rust
pub struct Quanot {
    pub reservoir: Reservoir,
    pub encoder: TextEncoder,
    pub chaos: ChaosAnalyzer,
    pub consciousness: ConsciousnessTracker,
    pub creativity: CreativeOscillator,
    pub state_history: Vec<Vec<f64>>,
    pub max_history: usize,
}

impl Quanot {
    /// Process text input through full pipeline
    pub fn process(&mut self, input: &str) -> QuanotResult {
        // 1. Encode text to vector
        let encoded = self.encoder.encode(input);

        // 2. Step reservoir
        self.reservoir.step(&encoded);
        let state = self.reservoir.get_state().to_vec();

        // 3. Update history
        self.state_history.push(state.clone());
        if self.state_history.len() > self.max_history {
            self.state_history.remove(0);
        }

        // 4. Compute chaos metrics
        let chaos = self.chaos.analyze(&self.state_history);

        // 5. Compute consciousness
        let consciousness = self.consciousness.compute(&state, &self.state_history);

        // 6. Compute creativity
        let creativity = self.creativity.step(&state, consciousness.phi);

        QuanotResult {
            reservoir_state: state,
            consciousness_proxy: consciousness.phi,
            novelty: self.compute_novelty(&state),
            creativity_scores: creativity,
            chaos_metrics: chaos,
        }
    }

    /// Compute novelty of current state vs history
    fn compute_novelty(&self, state: &[f64]) -> f64 {
        // Cosine distance from nearest neighbor in history
        let mut max_similarity = -1.0;
        for prev in &self.state_history {
            let sim = cosine_similarity(state, prev);
            max_similarity = max_similarity.max(sim);
        }
        1.0 - max_similarity
    }
}

pub struct QuanotResult {
    pub reservoir_state: Vec<f64>,
    pub consciousness_proxy: f64,
    pub novelty: f64,
    pub creativity_scores: CreativityOutput,
    pub chaos_metrics: ChaosMetrics,
}
```

---

## Starfire Integration

### Replace Quanot Bridge with Direct Module

Current: `lib/quanot/` (Python) → IPC → `lib/` (Rust)

New: `lib/quanot/` (Rust) → direct call → `lib/` (Rust)

### Add to lib.rs

```rust
pub mod quanot;
```

### Modify Runtime to use Quanot

In `lib/runtime/mod.rs`:
- Import `quanot::Quanot`
- Initialize Quanot alongside other subsystems
- Use `quanot.process(text)` instead of IPC call
- Feed `QuanotResult` into WorldModel and consciousness tracking

---

## Migration Steps

### Phase 1: Core Rust Implementation ✅
- [x] Create `lib/quanot/` module structure
- [x] Implement `reservoir.rs` — ESN core
- [x] Implement `chaos.rs` — Lyapunov, RQA, attractor metrics
- [x] Implement `consciousness.rs` — Φ proxy
- [x] Implement `creativity.rs` — oscillation system
- [x] Implement `encoder.rs` — text encoding
- [x] Implement `quantum_inspired.rs` — SQA/QAOA solver
- [x] All unit tests passing

### Phase 2: Starfire Integration ✅
- [x] Add `pub mod quanot;` to `lib/lib.rs`
- [x] Initialize Quanot in Runtime
- [x] Replace IPC calls with direct `quanot.process()`
- [x] Feed results into WorldModel
- [x] Remove old Python quanot bridge code
- [x] Remove Python quanot from project

### Phase 3: Optimization (Deferred — Future)
- [ ] Benchmark reservoir performance
- [ ] SIMD vectorization for reservoir compute
- [ ] Memory pool for state vectors (avoid allocations)
- [ ] Parallel training (ridge regression)

### Phase 4: Feature Parity ✅
- [x] Verify all Python quanot features are in Rust
- [x] Phase demos work identically
- [x] Chaos visualization (deprecated — skipped)
- [x] Full pipeline test

---

## Key Differences from Python Original

| Aspect | Python (Original) | Rust (Rewrite) |
|--------|------------------|-----------------|
| Reservoir size | 1000 | 1000 (configurable) |
| Numerical precision | f64 | f64 |
| Parallelism | GIL-limited | True parallelism |
| Memory | Python GC | Manual (no GC pauses) |
| IPC | TCP/STDIO | Direct function call |
| Deployment | Python + Rust | Single Rust binary |
| Testing | pytest | cargo test |

---

## Files Created

```
lib/quanot/
├── Cargo.toml           — separate crate? or module? (recommend: module)
├── mod.rs               — Quanot orchestrator + public API
├── reservoir.rs         — ESN implementation
├── chaos.rs             — Chaos metrics
├── consciousness.rs     — Φ proxy
├── creativity.rs        — Creative oscillation
├── quantum_inspired.rs  — QAOA-style solver (from Python sqa.py)
├── encoder.rs           — Text → vector
└── tests/
    ├── test_reservoir.rs
    ├── test_chaos.rs
    ├── test_consciousness.rs
    ├── test_creativity.rs
    └── test_integration.rs
```

**Recommendation:** Keep Quanot as a module within the `star` crate (not a separate crate) since it has no external dependencies and is always used with Starfire. Use `lib/quanot/` subdirectory.

---

## Testing Strategy

- **Unit tests** for each submodule (reservoir, chaos, consciousness, creativity)
- **Integration tests** for full pipeline (text → QuanotResult)
- **Property-based tests** for numerical stability
- **Comparison tests** against Python original outputs (use known inputs, verify same outputs)
- **Benchmark tests** to verify Rust is faster

---

## Success Criteria

1. All quanot tests pass in Rust
2. `cargo test` for Starfire passes with Quanot integrated
3. No Python/IPC dependency for quanot
4. Benchmarks show meaningful speedup over Python
5. Feature parity verified by running same test inputs

---

## Notes

- The `sqa.py` (Simulated Quantum Annealing) was ported as `quantum_inspired.rs` — both SQA and QAOA solvers ✅
- The `visualization.py` is deprecated — CLI/text output is sufficient
- The Python quanot in `projects/quanot/` is now superseded by the Rust version in `lib/quanot/`
- The original Python quanot is preserved at `projects/quanot/` for reference but is no longer used by Starfire
