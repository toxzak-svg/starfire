# COG-Research — Critical Dynamics & Self-Model Learning
## Technical Report

**Project Path:** `/home/zach/.openclaw/workspace/dev/COG-research/`
**Type:** PyTorch research — critical dynamics, stability, and imagination-based learning
**Language:** Python
**Status:** Active experimentation; multiple dynamics models tested
**Last Updated:** 2026-04-01

---

## 1. Overview

COG-research (Cognitive Dynamics Research) explores a fundamental question:

> **What dynamics enable a system to learn a model of itself?**

This is distinct from learning a world model — self-model learning requires the system to:
1. Separate self-generated patterns from externally-driven patterns
2. Predict the consequences of its own actions
3. Detect when its self-model fails (imagination vs reality mismatch)

The research spans multiple experimental axes:
- **Critical dynamics** — operating at the edge of chaos for maximum information processing
- **Deterministic data** — controlled settings where ground truth is known
- **Imagination-first learning** — predicting outcomes before they happen
- **Timeseries Pile integration** — real-world temporal data at scale

---

## 2. Critical Dynamics

### 2.1 What Are Critical Dynamics?

Many complex systems exhibit **criticality** — a balance point between order and chaos:

| Regime | Example | Information Capacity |
|--------|---------|---------------------|
| Subcritical | Damped pendulum | Low — quickly settles |
| **Critical** | Sandpile avalanche | **Maximum — long-range correlations** |
| Supercritical | Chaos | High but unpredictable |

Neural networks trained near criticality show enhanced:
- Information transmission
- Dynamic range (response to input magnitude)
- Computational capacity (long-range dependencies)

### 2.2 How It's Implemented

```python
# Critical dynamics via eigenvalue constraint
class CriticalLinear(nn.Module):
    def __init__(self, d_model):
        self.weight = nn.Parameter(torch.randn(d_model, d_model))
    
    def forward(self, x):
        # Spectral radius of weight matrix ≈ 1.0
        eigenvalues = torch.linalg.eigvals(self.weight)
        spectral_radius = eigenvalues.abs().max()
        
        # Penalize deviation from criticality
        critical_loss = (spectral_radius - 1.0).abs()
        
        return x @ self.weight, critical_loss
```

### 2.3 Experiments (critical_dynamics_*.py)

| File | Focus | Status |
|------|-------|--------|
| `critical_dynamics_v2.py` | Core dynamics at scale | Active |
| `critical_dynamics_v3.py` | Ablation of criticality constraint | Active |
| `critical_dynamics_v4.py` | Generalization out of distribution | Active |
| `critical_simple.py` | Minimal proof-of-concept | Complete |
| `critical_strong.py` | Strong criticality signal | Active |
| `critical_constrained.py` | Parameter constraints | Active |
| `critical_critical.py` | Phase transition detection | Active |
| `critical_lstm.py` | LSTM baseline comparison | Active |

---

## 3. Imagination-First Learning

### 3. Concept

Traditional reinforcement learning: act → observe → update.
**Imagination-first:** predict outcome of action before acting → act if prediction is favorable → update predictor.

```
Current State
     │
     ▼
[Predict Self-Action Outcome]    ← Imagination
     │
     ├── Prediction: good outcome  ──→ Act
     │                                     │
     │                                     ▼
     │                              [Observe Reality]
     │                                     │
     └── Prediction: bad outcome  ──→ Don't act
                                          │
                                          ▼
                                   [Update Predictor]
```

This separates **imagination** (internal model of consequences) from **motor** (actual action), enabling:
- Sample-efficient learning (don't waste episodes on failed actions)
- Risk aversion without explicit reward shaping
- Self-model accuracy monitoring

### 3.2 Directory

**Path:** `imagination_first_learning/` — contains experimental implementations.

---

## 4. Timeseries Pile Integration

**Timeseries Pile** = a large-scale dataset of temporal sequences (time series from multiple domains).

### 4.1 Why Timeseries?

Timeseries data is particularly revealing for self-model learning because:
1. **Temporal coherence** — patterns persist and can be predicted
2. **Self-generated vs observed** — agent's own actions create observable effects
3. **Ground truth exists** — unlike text, physical timeseries has ground truth

### 4.2 Directory

**Path:** `timeseries_pile_data/` — raw dataset
**Path:** `scripts/` — data processing

### 4.3 Timeseries Pile Integration Doc

See: `TIMESERIES_PILE_INTEGRATION.md` for full integration plan.

---

## 5. Stability Analysis

**Path:** `stability_analysis/` — formal stability analysis of trained models.

Key question: Does the learned self-model remain stable under:
- Distribution shift (novel inputs)
- Action consequences that deviate from prediction
- Cascading prediction errors (imagination collapse)

### 5.1 Deterministic Data

**Path:** `deterministic_data/` — controlled datasets where ground truth is known.

When ground truth is known:
- Prediction error can be decomposed: model error vs inherent stochasticity
- Self-model accuracy can be precisely measured
- Failure modes can be categorized

---

## 6. VAE World Model

**File:** `VAE_WORLD_MODEL_DESIGN.md`

A variational autoencoder world model that:
1. Encodes observations into latent state
2. Predicts next latent state given action
3. Decodes latent state back to observation

The self-model is the **transition model** (step 2) — predicting what happens given self-generated actions.

---

## 7. Verification & Replication

The project maintains rigorous verification:

| Document | Purpose |
|----------|---------|
| `VERIFICATION_EXECUTION_PLAN.md` | Planned verification experiments |
| `VERIFICATION_EXECUTION_GUIDE.md` | How to run verification |
| `VERIFICATION_RESULTS.md` | Results of verification runs |
| `VERIFICATION_REPLICATION.md` | Replication protocol |
| `DYNAMICS_TRANSFER_ANALYSIS.md` | Cross-domain dynamics transfer |

---

## 8. Key Research Questions

1. **Can critical dynamics be learned or must they be initialized?**
   - If learnable: self-organizes toward criticality
   - If must be initialized: architectural constraint is fundamental

2. **Is imagination-first learning stable?**
   - Risk: imagination collapse (predictor stops producing useful variance)
   - Solution: diversity-enforcing regularization

3. **Does self-model accuracy transfer across domains?**
   - If yes: general self-awareness mechanism
   - If no: domain-specific only

---

## 9. Architecture

Multiple model types are tested as backbones:

| Model | File | Purpose |
|-------|------|---------|
| Linear (critical) | `critical_simple.py` | Minimal dynamics |
| MLP | `critical_strong.py` | Nonlinear dynamics |
| LSTM | `critical_lstm.py` | Recurrent baseline |
| Transformer | (in neu-project) | Long-range dependencies |

The **fair_comparison** files ensure these are fairly compared (same parameter count, same data).

---

## 10. Integration with Nue/CAR

COG-research and Nue share a core concept — **persistent state**:

- **COG:** Critical dynamics + self-model to maintain stable self-representation
- **Nue/CAR:** Persistent routing state that conditions future routing decisions

Both face the same fundamental challenge: **when does accumulated state become a liability (catastrophic forgetting) vs an asset (learning)**?

The answer in both projects involves:
- Error signal magnitude (COG: prediction error; CAR: SSM error)
- Temporal dynamics of error (drift vs ceiling)
- Diversity pressure to prevent state collapse

---

## 11. Key Files

| File | Purpose |
|------|---------|
| `critical_dynamics_experiment.py` | Core experiment runner |
| `fair_comparison.py` / `fair_comparison_v2.py` | Fair model comparisons |
| `analyze_benchmarks.py` | Benchmark analysis |
| `TIMESERIES_PILE_INTEGRATION.md` | Timeseries data plan |
| `INTELLIGENCE_METRICS.md` | Evaluation metrics design |
| `STATUS.md` | Current project status |
| `RESEARCH_EXPANSION_PLAN.md` | Future directions |
