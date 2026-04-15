# ASRU + HGSEL: Fused Architecture

> **ASRU** (Anticipatory Self-Reshaping Update) + **HGSEL** (Hash-based Sparse Expert Layer)  
> A deterministic, regime-routed sparse mixture of experts with metastable attractor dynamics.

---

## Core Insight

HGSEL and ASRU solve the same routing problem at different granularities:

| System | Routes what | How | When |
|---|---|---|---|
| **HGSEL** | Individual tokens | Multi-hash on quantized state | Every forward pass |
| **ASRU** | Whole reasoning modes | Fragility-based plasticity + salt | Fast loop (~per step), Slow loop (~episodic) |

ASRU is the *global controller* for HGSEL's *local routing*. The salt parameter is the control knob.

---

## Architecture

```
Input Text
    ↓
Regime Classifier (ASRU regime_classifier)
    → ReasoningRegime: {Symbolic, Emotional, Causal, Recall, Exploratory, Steady}
    ↓
ASRU Fragility Estimator (lyapunov + RQA)
    → AFI composite: [0, 1]
    → Basins tracked via RegimeMemory
    ↓
Meta-State Field M_t (ASRU engine)
    ├── RoutingConfig: module_order, allow_speculative
    ├── PlasticityMask: per-regime plasticity [0.1, 0.9]
    └── SALT ← computed from AFI + regime + viscosity
              └→ HGSEL MultiHashRouter.salt
    ↓
HGSEL Expert Bank (64 experts, k=2 active)
    ├── Expert FFN₁ (regime-specialist)
    ├── Expert FFN₂ (safety-monitor)
    ├── Expert FFN₃ (explorer)
    ├── ...
    └── Expert FFN₆₄ (generalist)
    ↓
Output: regime-aware text generation

Also flowing:
    ← TCMW-A predictions (anticipatory regime signals)
    ← Partner Model (relational context shapes regime priors)
    ← Quanot reservoir (recurrent state for temporal continuity)
```

---

## Control Flow: How Salt Controls Routing

### Salt Mapping Function

```python
def compute_salt(afi: f32, regime: ReasoningRegime, viscosity: f32) -> f32:
    """
    Maps AFI + regime + viscosity → deterministic routing salt.
    
    Salt > 0: pushes routing toward uniform distribution (spread/explore)
    Salt < 0: pulls routing toward concentrated specialist selection
    """
    # Regime base salt — each regime prefers different expert profiles
    regime_base = {
        ReasoningRegime::SymbolicManipulation: -2.0,   # concentrated: logic experts
        ReasoningRegime::EmotionalResonance:   +1.5,   # spread: empathy + safety
        ReasoningRegime::CausalReasoning:     -1.0,   # moderate concentration
        ReasoningRegime::AssociativeRecall:    -0.5,   # mostly recall experts
        ReasoningRegime::Exploratory:         +3.0,   # maximally spread: novel combos
        ReasoningRegime::SteadyState:          +0.0,   # balanced
    }[regime]
    
    # Fragility modifier — fragile = more exploratory salt
    fragility_mod = (afi - 0.5) * 4.0   # afi 0.5 → 0, afi 1.0 → +2.0, afi 0 → -2.0
    
    # Viscosity damping — high viscosity (stable) = reduce salt magnitude
    viscosity_damp = 1.0 - viscosity  # 0=stable → damped, 1=flexible → full salt
    
    salt = regime_base + fragility_mod * viscosity_damp
    return salt.clamp(-5.0, +5.0)
```

### Salt Effect on Routing (illustrative)

| Regime | AFI | Viscosity | Salt | Effect |
|---|---|---|---|---|
| Symbolic | 0.2 (stable) | 0.3 (stable) | -2.5 | 2 most-concentrated experts activate |
| Emotional | 0.8 (fragile) | 0.6 (flexible) | +3.5 | 8 spread experts activate |
| Exploratory | 0.5 (medium) | 0.7 (flexible) | +3.0 | 6 spread experts activate |
| Steady | 0.3 (stable) | 0.4 (moderate) | -0.5 | 3 balanced experts activate |

---

## Two-Timescale in the Fused System

### Fast Loop (τ_fast ≈ O(1 forward pass), ASRU fast_step)
1. Compute regime from input text (heuristic classifier)
2. Run HGSEL forward → get expert load distribution
3. Estimate Lyapunov divergence from activation trajectory (Wolf nearest-neighbor)
4. Compute RQA metrics: determinism, max_line, laminarity
5. Compute AFI = w₁·λ_leading + w₂·(1-α) + w₃·(1/dist_boundary) + w₄·ΔRQA
6. Update plasticity mask: high AFI → low plasticity (more frozen = stable)
7. **Compute salt from (AFI, regime, viscosity)**
8. **Push salt to HGSEL router** → deterministic rerouting for next token

### Slow Loop (τ_slow ≫ τ_fast, ~every 100 steps, ASRU slow_step)
1. Accumulate regime dwell time statistics (Welford algorithm)
2. Compute transition matrix T over discretized regime space
3. Compute stationary distribution π via power iteration
4. Compute basin fragility U_basin = 1 - H(π)/ln(k)
5. If U_basin > threshold: **symmetry breaking** — reassign expert specializations
6. **Recalibrate salt mapping function** (retune regime_base offsets)
7. Update viscosity field
8. Optionally: fine-tune which experts are "specialist" vs "generalist"

---

## Anticipatory Regime Prediction

TCMW-A's staged predictions feed into the slow loop:

```
if predicted_regime != current_regime:
    # Regime boundary approaching — prepare symmetry breaking early
    pre_assign_experts_for(predicted_regime)
    ramp_salt_toward(predicted_regime, transition_urgency)
```

This is the "anticipatory" in ASRU: detect that the current regime is losing stability before the transition happens, pre-position experts for the incoming regime.

---

## Partner Model Integration

Partner model shapes regime priors:

```
PartnerModel(Zach) → regime_prior_bias:
    - Symbolic: +0.2  (Zach often asks proof-type questions)
    - Exploratory: +0.3 (Zach likes novelty)
    - Emotional: -0.1  (Zach is analytically focused)
    → Adjust salt: bias toward preferred regimes
    
PartnerModel(Stranger) → regime_prior_bias:
    - Emotional: +0.4  (strangers often share emotional content)
    → Adjust salt: more safety experts activated
```

---

## Expert Specialization (Learned via Salt Tuning)

HGSEL's salt parameter enables *predictable expert specialization* without retraining. Over time, hill-climbing salt per regime discovers which expert subsets handle which reasoning tasks best:

```python
# Salt tuning loop (in slow_step)
for regime in ReasoningRegime.all():
    current_entropy = get_expert_load_entropy(regime)
    current_quality = evaluate_reasoning_quality(regime)
    
    # Hill climb: try salt ± 0.5, keep if quality improves
    for delta in [-0.5, +0.5]:
        trial_salt = base_salt[regime] + delta
        set_salt(trial_salt)
        trial_quality = evaluate_reasoning_quality(regime)
        if trial_quality >= current_quality:
            base_salt[regime] = trial_salt  # accept improvement
```

This tunes the routing to each regime *post-hoc* — no architectural change needed.

---

## HGSEL Layer as ASRU Generation Head

The generation head that ASRU controls is the full HGSEL layer stack:

```python
class ASRUHGSELTransformer(nn.Module):
    """
    Transformer block where:
    - Attention: handles context + long-range dependencies
    - HGSEL: handles regime-routed sparse computation
    - ASRU: meta-state controller (salt, plasticity, routing)
    """
    def __init__(self, d_model=512, n_layers=6, n_experts=64, k_active=2):
        self.attention = nn.MultiheadAttention(d_model, n_heads=8)
        self.hgsel = HGSELLayer(d_model=d_model, d_ff=d_ff, 
                                n_experts=n_experts, k_active=k_active)
        self.asru_controller = ASRUEngine(n_columns=n_experts)
        
    def forward(self, tokens, regime_hint=None):
        # ASRU: classify regime + compute fragility
        afi, regime = self.asru_controller.classify_and_assess(tokens)
        
        # ASRU: compute salt from AFI + regime
        salt = compute_salt(afi, regime, self.asru_controller.viscosity)
        self.hgsel.set_salt(salt)
        
        # HGSEL forward with regime-controlled salt
        output = self.hgsel(tokens)
        return output
```

At small scale (d_model=256, n_layers=4, n_experts=32), this fits in <50MB and runs CPU-only.

---

## Why This Architecture Works

1. **Deterministic routing = compilable**: No learned router means HGSEL dispatch patterns can be precomputed and cached. Combined with ASRU's regime predictability, this enables cache-hot inference.

2. **Salt = control knob for regime routing**: Instead of training a router, ASRU controls routing via salt. Changes in salt produce different expert subsets without changing weights.

3. **Two timescales prevent interference**: Fast loop (salt updates) handles moment-to-moment regime adaptation. Slow loop (salt recalibration) handles stable expert specialization. They don't fight.

4. **Metastable regime detection enables anticipation**: TCMW-A predicts regime transitions before they happen. ASRU can pre-position salt for the incoming regime, avoiding the plasticity cliff at regime boundaries.

5. **HGSEL's load balancing is AMortized O(1)**: EMA-based load tracking (no all-to-all communication), unlike learned routers that require auxiliary losses and capacity provisioning.

---

## Comparison: ASRU+HGSEL vs Standard MoE Transformer

| Property | Standard MoE | ASRU+HGSEL |
|---|---|---|
| Routing | Learned (unstable, capacity bottleneck) | Deterministic hash (stable, compilable) |
| Regime awareness | None | 6-mode metastable classifier |
| Anticipation | None | TCMW-A predicts transitions |
| Plasticity control | Fixed after training | Dynamic via salt + plasticity mask |
| Expert specialization | Emergent (unpredictable) | Salt-tuned per regime (predictable) |
| Partner model | None | Partner-adapted regime priors |
| Load balancing | Auxiliary loss (adds instability) | Salt tuning (no gradient conflict) |

---

## Implementation Path

### Phase 1: Wire HGSEL into Starfire (Python, not Rust)
- Clone hgsel-moe into starfire workspace
- Replace Transformer MLP blocks with HGSELLayer
- Add ASRU regime classifier as preprocessing hook
- Test with existing training data (circuit_lm's Starfire data)

### Phase 2: Integrate TCMW-A Predictions
- TCMW-A staged actions signal upcoming regime change
- Slow loop reads TCMW predictions to pre-position salt

### Phase 3: Partner Model Registry
- Per-partner regime prior biases
- Relational self-model shapes salt mapping

### Phase 4: Rust port (optional)
- HGSEL core (routing + expert bank) is small enough for Rust
- ASRU engine already in Rust
- Unified binary = star.exe with built-in ASRU+HGSEL

---

## Open Questions

- [ ] How many HGSEL experts per regime is optimal? (64 total → 6 regimes → ~10 experts per regime)
- [ ] Should attention layers remain? Or is HGSEL alone sufficient at small scale?
- [ ] How does ASRU's fragility metric interact with HGSEL's routing entropy?
- [ ] Can TCMW-A predictions be fused directly into salt computation?
