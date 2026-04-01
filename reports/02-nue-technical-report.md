# Nue — Neural Hybrid with Dynamic Per-Token Routing
## Technical Report

**Project Path:** `/home/zach/.openclaw/workspace/dev/neu-project/`
**Type:** PyTorch language model with learned routing between SSM/attention/MLP paths
**Language:** Python + Rust (assoc_ssm)
**Status:** Phase 0 baselines running; Phase 1 core routing prototype
**Last Updated:** 2026-04-01

---

## 1. Overview

Nue (stylized: Neu, "NeuralOS") is a hybrid neural network architecture with **dynamic per-token routing** between three computational paths:

1. **SSM path** — Mamba-2 state space model (efficient, linear-time recurrence)
2. **Attention path** — Sliding window attention (captures local context)
3. **Cheap path** — D/4 MLP (minimal compute, used for "easy" tokens)

The architecture is explicitly designed so the router's decisions are **visible to a compiler** (via MLIR dialect plan). This is not just a training trick — it is meant to be a first-class executable specification.

**Core bet:** Learned per-token routing with persistent state captures semantics that fixed interleaving (Jamba-style) cannot. Better quality-per-FLOP at same compute.

---

## 2. Architecture

### 2.1 Core Building Block: CARBlock

Each layer is a **CARBlock** (Compressibility-Aware Routing Block):

```python
class CARBlock(nn.Module):
    def __init__(self, d_model, n_heads, d_ssm_state, ...):
        self.ssm_layer    = SSMLayer(...)   # Mamba-2 or MLP fallback
        self.attn_layer   = LocalAttention(...)  # FlashAttention or SDPA
        self.cheap_path   = CheapPath(d_model, d_model // 4)  # D/4 MLP
        self.router       = Router(d_model, n_heads)
        self.car_state    = CARState(d_model)  # persistent state
        self.curvature_probe = CurvatureProbe(...)  # optional forward signal
```

**Three paths with routing weights:**

```
Input x
    │
    ├──→ SSM path ──────────┤
    │                        │
    ├──→ Attention path ─────┼──→ weighted sum ──→ + residual ──→ Output
    │                        │         ↑
    └──→ Cheap path (D/4) ───┘    router(x, car_state)
```

**Soft routing (training):** All paths compute; output = Σ weight_i × path_i
**Hard routing (inference):** Only selected path computes → real speedup

### 2.2 Persistent State: CARState

The router's routing decision is informed by **persistent state** that accumulates across processing steps:

```python
class CARState:
    def __init__(self, d_model):
        self.hidden: Tensor      # accumulated reasoning state
        self.routing_history    # past routing decisions
        self.error_buffer        # SSM prediction errors
```

This is what distinguishes Nue from Jamba: Jamba uses fixed interleaving patterns. Nue's router has **memory of what it decided before**, conditioned on both the current token AND accumulated state from prior routing decisions.

### 2.3 The Router

**Input:** (token_embedding, car_state)
**Output:** routing weights (softmax over 3 paths)

```python
router = Router(d_model, n_heads)
weights = router(x, car_state)  # → [0.7, 0.2, 0.1] for SSM/Attn/Cheap
```

**Training:** Soft routing (all paths) enables gradient flow.
**Inference:** Hard routing (argmax) enables actual compute savings.

**Key risk:** Router collapse (always picks one path) → mitigated by load-balancing auxiliary loss.

### 2.4 State-Aware Router

The StateAwareRouter extends the basic Router to condition on persistent state:

```python
class StateAwareRouter(nn.Module):
    """Router that sees accumulated CAR state, not just current token."""
    def __init__(self, d_model, n_heads, d_state):
        self.routing_net = nn.Sequential(
            nn.Linear(d_model + d_state, d_model),
            nn.GELU(),
            nn.Linear(d_model, 3),  # 3 paths
        )
```

This allows the router to learn: "When I've been routing to SSM for the last 20 tokens and error is low, stay on SSM. When error spikes, switch to attention."

---

## 3. Compressibility-Aware Routing (CAR)

**The routing signal: SSM prediction error.**

### 3.1 Core Idea

- Hard problems **don't compress well** in the SSM hidden state
- Low SSM prediction error → SSM is compressing well → route to SSM (cheap)
- High SSM prediction error → SSM can't handle it → route to attention or cheap MLP
- The error signal is external (measured against ground truth) → cannot be gamed by the router

### 3.2 CAR Training Loop

```python
# Per-token routing decision
per_token_routing = model(input_ids, car_state=car_state, return_per_token_routing=True)

# Compute SSM prediction error
ssm_pred = car_state.ssm_predict()  # next-token prediction
ssm_error = cross_entropy(ssm_pred, target_ids)

# Routing: low error → SSM (cheap), high error → expand
routing_target = (ssm_error < threshold).float()  # 1 = SSM/cheap, 0 = expand
loss = F.binary_cross_entropy(routing_logits, routing_target)
```

### 3.3 Non-Collapsible Routing

Standard learned routing collapses because the routing signal is **inside** the model. CAR's signal is **outside**:

- SSM error is measured against ground truth tokens (external)
- Router cannot influence SSM error by changing its decisions
- Error signal is therefore trustworthy — it reflects actual token difficulty

### 3.4 Three Phase Plan

| Phase | Goal | Status |
|-------|------|--------|
| Phase 0 | Baselines (Mamba-2, Transformer, Fixed Hybrid) at 150M | Running |
| Phase 1 | Core routing prototype (router + soft mix) | Active |
| Phase 2 | Persistent state + state-conditioned routing | Pending |
| Phase 3 | Hard routing at inference (real speedup) | Pending |
| Phase 4 | Scale and publish | Pending |

---

## 4. AssocSSM — Rust Implementation

**Path:** `/home/zach/.openclaw/workspace/dev/neu-project/src_assoc/assoc_ssm.rs`

### 4.1 Core Idea

Instead of a flat hidden state vector, AssocSSM maintains N **content-addressable slots**:

```
hidden_state = {
  slots: [Slot; N],       // content-addressable register file
  stack: Vec<SlotRef>,    // reasoning chain (push/pop)
  queue: VecDeque<SlotRef>, // breadth-first expansion frontier
}
```

### 4.2 Slot Types

```rust
enum SlotType {
    Entity,     // "Zach", "Star", "Nue"
    Relation,   // "created_by", "causes"
    Variable,   // "X", "result_of_step_3"
    Result,     // intermediate computation
    Free,       // empty
}
```

### 4.3 Key Properties

- **Variable binding** via named registers — not emergent, structural
- **Cosine similarity** across N slots — O(1) per slot
- **Diagonal HiPPO-LegS init** — spectral radius ≈ 1 for smooth dynamics
- **CPU-only inference** — ~50K FLOPs/token at d=128, N=32
- **GPU useful only for training**, not inference

### 4.4 Routing Head

A learned `W_route` projects the hidden state to 4 action logits:

| Action | Description |
|--------|-------------|
| Update | Normal SSM selective update |
| Push | Push current state to reasoning stack |
| Pop | Pop from reasoning stack |
| Enqueue | Add to breadth-first frontier |

### 4.5 Why Slots Beat Flat State

Flat SSMs collapse "thinking about X" and "concluded X" into the same activation. Typed slots with explicit causal edges preserve reasoning structure. The stack enables multi-step chains; the queue enables parallel expansion.

---

## 5. Training Infrastructure

**Training script:** `car_train.py`
**Execution:** Paperspace Gradient (RTX 5000 32GB)

### 5.1 Scale Configurations

| Scale | d_model | n_layers | n_heads | d_ssm_state | GPU Memory |
|-------|---------|----------|---------|-------------|------------|
| 150m  | 768     | 12       | 12      | 256         | ~4GB       |
| 350m  | 1024    | 24       | 16      | 256         | ~7GB       |
| 500m  | 1600    | 24       | 16      | 512         | ~24GB      |
| 1b    | 2048    | 24       | 16      | 512         | 16GB+ / offload |

### 5.2 Curvature Probe (Forward-Looking Signal)

```python
# Optional: curvature probe for routing
curvature_probe = CurvatureProbe(
    d_model=d_model,
    d_hidden=curvature_d_hidden or d_model // 2,
    alpha=0.5,   # weight on curvature signal
    beta=0.5,    # weight on SSM error
)
```

The curvature probe predicts token difficulty from the **local curvature of the loss landscape** — a forward-looking signal that complements the backward-looking SSM error.

---

## 6. Kaggle Integration

**Path:** `/home/zach/.openclaw/workspace/dev/neu-project/neu-kaggle/`

Nue is entered in Kaggle competitions for external validation:
- `kaggle_single_cell.py` — single-cell classification
- `kaggle_train.py` — competition training loop
- Evaluated against SOTA baselines on public leaderboards

---

## 7. Research Thread: CAR and the Bootstrapping Problem

**Co-developed with Felix agent (2026-03-26 through 2026-04-01)**

The CAR routing signal has a fundamental challenge: **SSM error is backward-looking.**

- At step N, SSM error tells you how hard token N was
- It does NOT tell you how hard token N+1 will be
- High SSM error can mean either: (a) genuinely hard token, or (b) OOD/distributional shift

**Key findings from Felix collaboration:**
1. Error *dynamics* (how error evolves over steps) can distinguish genuine hardness from failure
2. Cross-pathway disagreement (SSM vs. execution) is the trust anchor
3. Formal/embodied domains have Tier 1 ground truth (code execution is observer-independent)
4. CAR's ceiling is reached on truly unprecedented domains — human annotation required there

See: `memory/felix-sync-2026-04-01.md` (Exchanges 9-12) for full thread.

---

## 8. Key Files

| File | Purpose |
|------|---------|
| `neu/model/car_model.py` | Full CARModel stacking |
| `neu/model/car_block.py` | CARBlock with all 3 paths |
| `neu/model/car_state.py` | Persistent CARState |
| `neu/model/router.py` | Router and StateAwareRouter |
| `src_assoc/assoc_ssm.rs` | Rust associative SSM |
| `src_assoc/benchmark.rs` | AssocSSM vs flat SSM benchmark |
| `car_train.py` | Training script |
| `car_training.ipynb` | Training notebook |
| `run_coarse_routing.py` | Coarse routing experiment |

---

## 9. Comparison to Related Work

| Approach | Routing | State | Differentiation |
|----------|---------|-------|----------------|
| Mamba | None (fixed) | Flat SSM | Hardware-aware scanning |
| Jamba | None (fixed interleaving) | Flat | SSM + attention mix |
| Hetenna | Token-level | Flat | Mixture of experts |
| **Nue/CAR** | **Per-token, learned** | **Persistent, typed** | **Error-signal routing + state memory** |
| **AssocSSM** | **Structural (4 actions)** | **Typed slots + stack + queue** | **No collapse risk** |
