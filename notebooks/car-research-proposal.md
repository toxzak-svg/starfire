# Compressibility-Aware Routing: A Non-Collapsible Architecture for Adaptive Compute Allocation

**Status:** Research Proposal  
**Date:** 2026-03-26  
**Authors:** Zachary Maronek, Elliot (Research Partner)

---

## Abstract

We propose Compressibility-Aware Routing (CAR) — a new mechanism for adaptive compute allocation in large language models. Rather than relying on a learned router that is susceptible to collapse, CAR uses the prediction error of an SSM state predictor as a non-gameable routing signal. Tokens that are well-predicted by the compressed state are processed via a cheap path; tokens that surprise the state are routed to a full-precision expansion. We argue this mechanism is non-collapsible, is trainable via standard gradient methods, and enables dramatically reduced effective compute — potentially allowing 100B-class models to run on a single P5000 GPU.

---

## 1. Motivation

### 1.1 The Router Collapse Problem

Mixture-of-Experts (MoE) and hybrid SSM-attention architectures rely on learned routers to route tokens to different computation paths. A fundamental failure mode is **router collapse**: the router learns to send all tokens to the cheapest or most capable expert, defeating the purpose of conditional computation.

Load-balancing losses (e.g., Switch Transformer) partially mitigate this, but introduce additional hyperparameters and can still converge to degenerate solutions where routing is effectively random or uniform rather than task-optimal.

### 1.2 Hard Problems Don't Compress Well

During prior research into high-compression hallucination, we observed a key asymmetry: **hard problems are hard to compress**. A model that achieves 95% compression on routine tokens may achieve only 50% compression on novel or complex tokens. This observation suggests a different routing signal — one that is inherently non-gameable.

### 1.3 Human Cognition as Inspiration

Human working memory operates via exactly this mechanism. Familiar tasks (driving a known route, brushing teeth) are handled by habit circuits — cheap, fast, automatic. Novel events (a pedestrian jumps out, an unexpected conversation) trigger full working memory engagement — expensive, slow, deliberate. The signal for this switching is **surprise**: prediction error relative to what the routine "should" produce.

---

## 2. Architecture: Compressibility-Aware Routing

### 2.1 Core Mechanism

```
Token Input
      │
      ▼
┌──────────────────┐
│  SSM State       │  ← Compressed representation of prior context
│  Predictor       │
└────────┬─────────┘
         │ predicts next token
         ▼
   Prediction Error (||predicted - actual||)
         │
    ┌────┴────┐
    │         │
low error  high error
    │         │
    ▼         ▼
 CHEAP     EXPAND
 (MLP)    (Base Model)
```

### 2.2 Components

**Persistent State (S):** A small learned vector (e.g., 256–1024 dims) that acts as a compressed working memory. It is updated after each token via a GRU/LSTM-style recurrence over the token embedding.

**State Predictor (P):** A lightweight module that takes the current state and predicts the next token embedding. Trained via standard cross-entropy loss against the true next token.

**Cheap Path (C):** A small MLP (e.g., 125M–2B params). Handles tokens that the state predicts well.

**Expansion Base (E):** A large frozen or lightly-trained base model (e.g., 100B params at 4-bit). Handles tokens that the state fails to predict.

**Routing Rule:** Simple threshold or sigmoid on prediction error:
```
route = sigmoid(threshold - error)
if route > 0.5: cheap_path()
else: expand()
```

### 2.3 Training Procedure

Phase 1 — State Predictor:
- Freeze C and E.
- Train the state update mechanism to minimize prediction error.
- Measure: what fraction of tokens have error below threshold?

Phase 2 — End-to-End:
- Unfreeze C.
- Train cheap path to correct state predictor mistakes on easy tokens.
- Loss = CE(cheap_output, target) + CE(expansion_output, target).

Phase 3 — Fine-tune Expansion (optional):
- Light fine-tuning of E on tokens that consistently fail cheap path.
- Or keep E frozen and rely on the routing to route around its weaknesses.

### 2.4 Memory Footprint on P5000 (Target: 100B Effective)

| Component | Memory | Location |
|---|---|---|
| Base model 100B @ 4-bit | ~50GB | NVMe, streamed |
| Persistent state + activations | ~4GB | GPU |
| Cheap path (2B, fp16) | ~4GB | GPU |
| Optimizer (trainable params only) | ~8GB | GPU |
| **Total** | **~66GB** | |

With tensor parallelism across 2× P5000: ~33GB per GPU. Within reach.

---

## 3. Why This Is Non-Collapsible

A learned router can collapse because the gradient signal is a function of the routing decision itself. A token sent to the "wrong" expert may produce high loss, but the router has no independent signal to detect this — it only learns from downstream loss.

CAR's routing signal is **external to the model**: it is the residual between what the SSM state predicted and what the token actually is. This signal cannot be gamed by changing the routing policy, because the routing policy does not affect what the next token is. The state predictor is judged against ground truth, not against the routing decision.

This is analogous to why a validation set generalizes while a training set can be memorized — the error is measured against something the model did not produce.

---

## 4. Relationship to Prior Work

**Jamba / Fixed Hybrid:** Fixed every-Nth-layer attention. No learned routing. Simple but inflexible — cannot adapt to token difficulty.

**Switch Transformer / MoE:** Learned per-token routing. Susceptible to collapse. Requires load-balancing auxiliary losses.

**Medusa / Speculative Decoding:** Uses a draft model to speed up inference. Draft is accepted or rejected based on verification. Closer to CAR, but designed for inference speed, not training.

**CAR's contribution:** Uses compressibility (prediction error) as the routing signal, enabling training without collapse risk. Combines the flexibility of learned routing with the robustness of an external signal.

**Relation to Nue:** Nue solves "which path per layer" with a learned router. CAR solves "when to go big vs small" with an external signal. They are complementary: Nue's per-layer routing within a block, CAR's per-token compute scaling across the whole model.

---

## 5. Key Claims

1. **Non-collapse:** CAR's routing signal (prediction error) is non-gameable by the routing policy.
2. **Memory efficiency:** 100B-class effective compute on a P5000 via streaming 4-bit base + cheap working path.
3. **Quality parity:** CAR matches a dense 100B model at matched effective FLOPs.
4. **Scalability:** Routing advantage grows with scale — harder problems at scale benefit more from selective expansion.

---

## 6. Experiments Required

### Experiment 1: Proof of Concept (1B scale)

**Goal:** Demonstrate that prediction error can route correctly at small scale.

- Base model: 1B SSM (e.g., Mamba-2 1B)
- Cheap path: 125M MLP
- State: 512-dim GRU-updated
- Dataset: FineWeb-Edu (10B tokens)
- Baseline: Dense 1B Mamba-2

**Measure:** What fraction of tokens does the cheap path handle? Quality (val loss) of cheap path output vs full model output on routed tokens.

**Expected:** If cheap path handles 70%+ tokens with <5% quality degradation, the mechanism works.

### Experiment 2: Compression vs Quality Curve (3B scale)

**Goal:** Sweep the prediction error threshold and measure the trade-off.

- As threshold increases: more tokens routed cheap, memory drops, quality degrades gradually.
- The "elbow" of this curve is the operating point.

**Measure:** Val loss vs effective compute (FLOPs per token). We expect a concave curve — small drops in quality for large drops in compute.

### Experiment 3: Full Comparison at 10B (multi-P5000)

**Goal:** Produce the ablation table.

| Model | Params | Val Loss | FLOPs/token | Hardware |
|---|---|---|---|---|
| Mamba-2 10B | 10B | TBD | 100% | 1× P5000 |
| Fixed Hybrid 10B | 10B | TBD | 100% | 1× P5000 |
| CAR (threshold=lo) | 10B+125M | TBD | ~40% effective | 1× P5000 |
| CAR (threshold=hi) | 10B+125M | TBD | ~20% effective | 1× P5000 |

---

## 7. Risks and Mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| Prediction error threshold is brittle | Medium | Sigmoid softening; learned threshold |
| Cheap path can't correct state mistakes | High | Phase 2 training corrects cheap path on easy tokens |
| Streaming base model kills throughput | High | Aggressive prefetching; overlap compute and I/O |
| State predictor collapses to uniform | Low | Diversity loss on state; prediction loss on state itself |

---

## 8. Open Questions

1. What is the optimal state dimensionality? (Too small: can't predict anything. Too large: defeats the compression purpose.)
2. Should the threshold be fixed, learned, or annealed?
3. Does CAR benefit from scale more than fixed routing? (Likely yes — harder problems at scale benefit more.)
4. Can the state predictor be improved via meta-learning? (Predict what the expansion would have done.)

---

## 9. Conclusion

Compressibility-Aware Routing offers a new primitive for conditional computation: an external, non-gameable routing signal based on prediction error. Unlike learned routers that are susceptible to collapse, CAR's signal is measured against ground truth and cannot be gamed by changing the routing policy. This enables training of models that adaptively allocate compute based on token difficulty, potentially achieving 100B-class quality on a single P5000 GPU.

---

## Next Steps

- [ ] Implement CAR block at 1B scale
- [ ] Run Experiment 1 (proof of concept)
- [ ] Analyze routing statistics — what kinds of tokens go cheap vs expand?
- [ ] Write findings as blog post or arXiv note
