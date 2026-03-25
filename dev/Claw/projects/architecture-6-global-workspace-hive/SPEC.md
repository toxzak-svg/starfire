# Global Workspace Hive — Full Specification

**Status:** Draft spec  
**Version:** 0.1  
**Last updated:** 2026-03-20

---

## 1. Overview

The Global Workspace Hive is a brain-inspired architecture that replaces the monolithic transformer backbone with a **dynamic coalition of specialized processors** communicating through a central workspace. Unlike multi-agent systems that run inference-time orchestration, this is a **single differentiable network** — the routing, competition, and combination are all learned end-to-end.

### 1.1 Core Hypothesis

> Intelligence emerges from dynamic coalitions of weak specialists, not from a single powerful generalist.

If this hypothesis holds, the architecture could achieve better **compositionality, modular reasoning, and domain generalization** than dense transformers of equal parameter count.

---

## 2. Architecture

### 2.1 High-Level Diagram

```
[Input Tokens] → [I/O Layer] → [Workspace Seed]
                                      ↓
                    ┌─────────────────┼─────────────────┐
                    ↓                 ↓                 ↓
              [Processor 1]    [Processor 2]    ... [Processor N]
                    ↓                 ↓                 ↓
              [Proposal 1]     [Proposal 2]     ... [Proposal N]
                    ↓                 ↓                 ↓
              [Router/GraphDesigner] → [Competition (Top-k)]
                    ↓
              [Workspace Update] → [Output Logits]
```

### 2.2 Component Specifications

| Component | Specification | Rationale |
|-----------|---------------|-----------|
| **Num Processors (P)** | 128 (configurable 64-256) | Enough diversity, manageable compute |
| **Processor Type** | Small RNN (2-layer, 512 hidden) or Tiny LM (100M params) | Tradeoff: RNN = fast, LM = more expressiveness |
| **Processor Hidden Dim** | 512 | Small but can capture local patterns |
| **Workspace Dim (W)** | 8192 | Large enough to hold distributed representations |
| **Router** | 2-layer lightweight transformer (256 hidden, 4 heads) | Learns dynamic topology; small overhead |
| **Competition (k)** | Top-8 write to workspace | Winner-take-most; enough diversity |
| **Sequence Length** | Initial: 512, target: 2048 | Start small, scale up |
| **Total Parameters** | ~1.3B (comparable to 1.3B dense transformer) | Fair comparison baseline |

### 2.3 Data Flow (per timestep)

1. **Input to Workspace Seed**
   - Token embeddings (dim E=1024) → linear projection → workspace seed (dim W=8192)
   - `w_seed = W_embed(token)`

2. **Processors Read from Workspace**
   - Each processor `p_i` receives full workspace vector `w_t`
   - Processor computes: `h_i = f_i(w_t)` where `f_i` is its forward pass

3. **Processors Propose Updates**
   - Each processor outputs a proposal vector `p_i` of dim W
   - This is what the processor "wants to write" to the workspace

4. **Router Computes Attention Weights**
   - Router takes workspace state and produces routing matrix `R` of shape (P × P)
   - `R = softmax(Transformer(w_t))` — sparse, ~30% non-zero connections
   - This defines which processors attend to which (dynamic graph)

5. **Competition (Top-k)**
   - Compute importance scores for each processor's proposal: `score_i = dot(w_t, p_i) + bias_i`
   - Select top-k processors: `S = topk(scores, k=8)`
   - Only processors in S can write to workspace

6. **Workspace Update**
   - `w_{t+1} = w_seed + sum_{i in S} alpha_i * p_i`
   - `alpha_i` = softmax(score_i over S) — weighted combination

7. **Output**
   - Final workspace state → linear projection → vocabulary logits
   - `logits = W_out(w_final)`

### 2.4 Pseudocode

```python
class GlobalWorkspaceHive(nn.Module):
    def __init__(self, vocab_size, num_processors=128, workspace_dim=8192):
        self.token_embed = nn.Embedding(vocab_size, 1024)
        self.input_proj = nn.Linear(1024, workspace_dim)  # seed
        
        self.processors = nn.ModuleList([
            Processor(hidden_dim=512, type='rnn')  # or 'tiny_lm'
            for _ in range(num_processors)
        ])
        
        self.router = Router(hidden_dim=256, num_heads=4)
        self.competition_logits = nn.Linear(workspace_dim, num_processors)
        
        self.output_proj = nn.Linear(workspace_dim, vocab_size)
    
    def forward(self, input_ids):
        # Step 1: seed workspace
        w = self.input_proj(self.token_embed(input_ids))  # [seq, W]
        
        # Step 2: iterate through sequence
        for t in range(seq_len):
            # Each processor reads current workspace
            proposals = [proc(w) for proc in self.processors]
            
            # Router computes routing matrix
            routing = self.router(w)  # [P, P] sparse
            
            # Competition: top-k selection
            scores = self.competition_logits(w)
            top_k_idx = torch.topk(scores, k=8).indices
            
            # Weighted update
            selected_proposals = [proposals[i] for i in top_k_idx]
            weights = torch.softmax(scores[top_k_idx], dim=-1)
            w = w + sum(w * p for w, p in zip(weights, selected_proposals))
        
        # Step 3: output logits
        return self.output_proj(w[-1])  # last token
```

---

## 3. Training Dynamics

### 3.1 Loss Function

Standard **next-token prediction loss** (cross-entropy on vocabulary). The architecture learns to predict tokens via the workspace mechanism — no auxiliary losses needed.

```
L = -sum_{t} log P(token_t | tokens_{<t})
```

### 3.2 Challenges & Solutions

| Challenge | Solution |
|-----------|----------|
| **Processors not specializing** | Add small auxiliary loss: processors must reconstruct their input, forcing them to develop internal representations |
| **Router collapsing to uniform** | Sparse Gumbel-softmax for hard routing decisions during training; encourage entropy in routing |
| **Workspace bottleneck** | Residual connections: `w_out = w_in + delta`, not pure overwrite |
| **Training instability** | Warmup learning rate specifically for processor parameters; smaller LR for router |
| **Gradients through competition** | Straight-through estimator for top-k selection (copy gradients as if all processors were selected) |

### 3.3 Specialization Loss (Auxiliary)

To encourage processors to develop distinct specialties:

```python
def specialization_loss(processor_outputs):
    # Penalize correlation between processor outputs
    # Processors should be orthogonal/diverse
    corr_matrix = torch.corrcoef(torch.stack(processor_outputs))
    loss = -torch.logdet(corr_matrix + epsilon)  # maximize diversity
    return loss
```

This loss encourages processors to encode different information, preventing collapse to identical representations.

### 3.4 Training Schedule

| Phase | Epochs | LR | Batch Size | Notes |
|-------|--------|-----|------------|-------|
| Warmup | 500 | 1e-5 → 1e-4 | 32 | All params |
| Main | 2000 | 1e-4 → 1e-5 (cosine) | 64 | Standard training |
| specialization | 500 | 1e-5 | 64 | Add specialization loss |

---

## 4. Variants

### 4.1 Processor Type Variants

| Variant | Description | Tradeoff |
|---------|-------------|----------|
| **RNN** | 2-layer simple RNN, hidden=512 | Fast, low memory, less expressive |
| **GRU** | 2-layer GRU, hidden=512 | Better memory, moderate compute |
| **Tiny LM** | 100M param transformer (2 layers, 8 heads) | Most expressive, more compute |
| **SSM** | Structured state-space model (Mamba-style) | Good long-range, efficient |

**Recommendation:** Start with RNN for speed; scale to Tiny LM if results promising.

### 4.2 Routing Variants

| Variant | Description | Pros/Cons |
|---------|-------------|-----------|
| **Dense** | All-to-all attention | High compute, easy training |
| **Sparse (default)** | Top-k connections per processor | Efficient, requires harder training |
| **Hard routing** | One processor per input type (discrete) | Most efficient, needs discrete training |
| **Hierarchical** | Processors grouped; only group leaders communicate | Scales better, less expressiveness |

### 4.3 Competition Variants

| Variant | Description |
|---------|------------|
| **Softmax (default)** | Weighted sum of top-k proposals |
| **Hard top-k** | Only winner writes (stochastic with gumbel) |
| **Mixture of experts** | Weighted combination with learned gates |

---

## 5. Benchmark Suite

### 5.1 Primary Benchmarks

| Benchmark | Type | Target Metric | Why It Matters |
|-----------|------|---------------|----------------|
| **GSM8K** | Math reasoning | Accuracy | Tests multi-step reasoning |
| **BBH** (Big-Bench Hard) | Compositionality | Accuracy | Tests novel problem solving |
| **MMLU** | Knowledge | Accuracy | Baseline capability |
| **HumanEval** | Code generation | Pass@1 | Tests functional correctness |
| **MBPP** | Code generation | Pass@1 | Simpler code tasks |

### 5.2 Architecture-Specific Tests

These tests specifically probe the benefits of the hive architecture:

| Test | What It Measures | Design |
|------|------------------|--------|
| **Compositional generalization** | Can combine known concepts in novel ways | Train on (a,b)→c, (c,d)→e, test on (a,d)→? |
| **Domain transfer** | Generalization to new domains | Train on English, test on code/math |
| **Modular arithmetic** | Systematicity | Train on small numbers, test on large |
| **Planning** | Multi-step goal achievement | Shortest path in novel graphs |
| **Tool use** | Call external functions | Seq of API calls to achieve goal |

### 5.3 Ablation Suite

| Ablation | What to Remove | Expected Impact |
|----------|----------------|-----------------|
| **Single processor** | Collapse to 1 processor | Should degrade significantly |
| **No competition** | All processors write | Lose specialization benefits |
| **Static routing** | Router removed, fixed graph | Lose dynamic adaptation |
| **No workspace** | Direct processor → output | Lose global communication |
| **Dense router** | Sparse → full attention | Compare compute vs quality |

### 5.4 Baseline Comparisons

| Model | Params | Notes |
|-------|--------|-------|
| **Transformer (dense)** | 1.3B | Same params, standard architecture |
| **MoE (8 experts)** | 1.3B (active ~300M) | State-of-the-art for parameter efficiency |
| **Mixtral** | 12B / 12B active | Reference for MoE quality |

---

## 6. Implementation Considerations

### 6.1 Parallelism Strategy

- **Data parallelism:** Standard across GPUs
- **Model parallelism:** Split processors across GPUs (each GPU holds 16-32 processors)
- **Pipeline parallelism:** Workspace → processors → competition is one forward pass; minimal pipelining benefit

### 6.2 Memory Optimization

| Technique | Savings |
|-----------|---------|
| **Processor offloading** | Only keep active processors in memory |
| **Workspace checkpointing** | Don't store all timesteps; recompute |
| **Gradient checkpointing** | Standard for large models |

### 6.3 Expected Compute

| Scenario | FLOPs | Training Time (A100) |
|----------|-------|----------------------|
| Full training (1T tokens) | ~10^24 | ~2 weeks (8x A100) |
| Quick ablation (50B tokens) | ~5×10^22 | ~1 day |

---

## 7. Scaling Plan

### Phase 1: Proof of Concept
- **Goal:** Show hive can learn at all
- **Processors:** 64 RNN-based
- **Scale:** 200M params, 50B tokens
- **Success:** Beats random baseline, learns basic patterns

### Phase 2: Competitive Baseline
- **Goal:** Match 1.3B transformer
- **Processors:** 128 mixed (RNN + tiny LM)
- **Scale:** 1.3B params, 500B tokens
- **Success:** MMLU > 45%, GSM8K > 30%

### Phase 3: Advantage
- **Goal:** Beat transformer on reasoning tasks
- **Processors:** 256, more diverse types
- **Scale:** 2B params, 1T tokens
- **Success:** BBH > transformer, strong compositionality

### Phase 4: Breakthrough
- **Goal:** Demonstrate emergent capabilities
- **Processors:** 512+ with learned specialization
- **Scale:** 5B+ params
- **Success:** New capabilities not seen in transformers

---

## 8. Risk Analysis

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Processors don't specialize | Medium | High | Add specialization loss, curriculum |
| Workspace becomes bottleneck | High | High | Residual connections, increase dim |
| Routing collapses to identity | Medium | Medium | Entropy penalty, sparse routing |
| Training unstable | Medium | Medium | Warmup, gradient clipping |
| Not better than transformer | High | Low | Iterate on architecture, scale |

---

## 9. Key Hyperparameters

| Hyperparameter | Value | Rationale |
|----------------|-------|-----------|
| `num_processors` | 128 | Balance diversity vs compute |
| `workspace_dim` | 8192 | Large representation space |
| `processor_hidden` | 512 | Enough expressiveness |
| `competition_k` | 8 | Diverse write access |
| `router_hidden` | 256 | Lightweight |
| `learning_rate` | 1e-4 | Standard for this scale |
| `batch_size` | 64 | Fits in GPU memory |
| `gradient_clip` | 1.0 | Stability |

---

## 10. References & Inspiration

- **Global Workspace Theory** (Baars, 1997) — foundational cognitive theory
- **Neural Blackboard Architecture** (Rasooli & B., 2019) — similar idea in NLP
- **Mixture of Experts** (Shazeer et al., 2017) — dynamic computation routing
- **Switch Transformer** (Fedus et al., 2021) — massive MoE
- **Hash Networks** (Roller et al., 2021) — stochastic routing
- **Computation allocation in brains** — neurons are specialized, not uniform

---

## 11. Open Questions

1. **How many processors needed for emergent reasoning?** Maybe 64 is enough, maybe 512 needed.
2. **What processor type is best?** RNN is fast but limited; tiny LMs are powerful but heavy.
3. **How often should routing change?** Per-token? Per-sequence? Static?
4. **Can we interpret processor specializations?** Would be valuable for debugging.
5. **Does this architecture scale like transformers?** Unknown — might hit different bottlenecks.

---

## 12. Next Steps

1. [ ] Implement minimal prototype (32 processors, 100M params)
2. [ ] Run on small dataset (C4, 10B tokens)
3. [ ] Verify training works (loss decreases)
4. [ ] Compare to small transformer baseline
5. [ ] Run ablation: remove competition, remove router
6. [ ] Scale up to full spec
7. [ ] Benchmark on reasoning tasks

---

*This spec is a living document. Update as experiments reveal what works.*