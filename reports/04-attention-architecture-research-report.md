# Attention Architecture Research Project
## Technical Report

**Project Path:** `/home/zach/.openclaw/workspace/dev/research/attention/`
**Type:** PyTorch research — structural separation of perception/memory/routing
**Language:** Python
**Status:** 107/107 foundation tests passing; experiment infrastructure ready
**Last Updated:** 2026-04-01

---

## 1. Overview

This project tests a fundamental architectural hypothesis:

> **Transformers collapse three distinct functions (perception, memory, routing) into a single self-attention operation. This homogeneity limits interference resistance under high distractor density.**

**Architecture B** implements explicit structural separation:
- **GRU recurrent controller** for routing decisions
- **External differentiable hash table** for structured memory
- **Learned sparse routing policy** with hard budget constraints
- **Rolling-window attention budget enforcer**

The experiment: Controlled interference under varying distractor density/type. Does explicit structure provide a protective noise floor that homogeneous attention cannot maintain?

**Status:** 107/107 foundation tests passing. Ready for full experiment sweep.

---

## 2. Core Hypothesis

### 2.1 The Homogeneity Problem

In a transformer, self-attention does three things simultaneously:

| Function | Attention Mechanism | Limitation |
|----------|---------------------|------------|
| **Perception** | Q/K/V projection of current token | No separation from memory |
| **Memory** | All prior tokens in KV cache | No write policy, no eviction |
| **Routing** | Softmax over all positions | Never truly eliminates traces |

The KV cache is treated as memory, but it has:
- **No write policy** — all tokens enter indiscriminately
- **No eviction** — only recency or fixed-size window
- **No competitive dynamics** — softmax globally renormalizes

### 2.2 Architecture B Hypothesis

Explicit structure provides interference resistance:

```
Input Token
     │
     ▼
[Perception Encoder]  ← Shared embedding (what to encode)
     │
     ▼
[GRU Controller]      ← Persistent hidden state, decides actions
     │
     ▼
[Routing Policy]       ← Learned {PERCEIVE, READ, WRITE, SKIP}
     │
     ├─→ PERCEIVE  → Pass through (no memory access)
     ├─→ READ       → External Memory (cosine lookup)
     ├─→ WRITE      → External Memory (salience-gated)
     └─→ SKIP       → No operation
     │
     ▼
[Output Head]          → Predict next token
```

The key insight: **Controller hidden state is not the same as memory.**

---

## 3. Architecture Components

### 3.1 GRU Controller

```python
class GRUController(nn.Module):
    def __init__(self, embed_dim, hidden_dim):
        self.gru = nn.GRUCell(embed_dim, hidden_dim)
        self.hidden_dim = hidden_dim
    
    def forward(self, token_emb, prev_hidden):
        hidden = self.gru(token_emb, prev_hidden)
        return hidden  # shape: (B, hidden_dim)
```

Why GRU instead of Mamba? Debugging clarity. Mamba's SSM dynamics add architectural unknowns. GRU's discrete gate mechanism is fully interpretable.

### 3.2 Differentiable Hash Table

```python
class HashTable(nn.Module):
    def __init__(self, embed_dim, n_slots, key_dim):
        self.keys = nn.Parameter(torch.randn(n_slots, key_dim))
        self.values = nn.Parameter(torch.randn(n_slots, embed_dim))
    
    def forward(self, query, top_k=1):
        # Cosine similarity → straight-through top-k
        similarities = cosine(query, self.keys)  # no softmax!
        top_idx = gumbel_topk(similarities, top_k)  # straight-through
        return top_idx, self.values[top_idx]
```

**Key design choices:**
- **Top-1 retrieval** — competitive dynamics; winner takes all
- **Straight-through estimator** — hard retrieval but gradient flows
- **No softmax** — softmax softens competition, undermining the point
- **Salience-gated writes** — only high-confidence inputs overwrite slots

### 3.3 Routing Policy

```python
class RoutingPolicy(nn.Module):
    def __init__(self, hidden_dim, n_actions=4):
        self.net = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.GELU(),
            nn.Linear(hidden_dim, n_actions),  # PERCEIVE, READ, WRITE, SKIP
        )
    
    def forward(self, hidden, temperature=1.0):
        logits = self.net(hidden)
        if temperature < 0.1:
            actions = gumbel_hard_max(logits)
        else:
            actions = F.gumbel_softmax(logits, tau=temperature)
        return actions
```

**Training phases:**
1. **Supervised warm-start** (Gumbel τ=1.0 → 0.1): Learn basic routing
2. **RL fine-tuning** (hard argmax): REINFORCE with sparsity penalty

### 3.4 Budget Enforcer

```python
class BudgetEnforcer:
    """
    Rolling-window budget: B accesses per T timesteps.
    Prevents front-loading. Continuous pressure.
    """
    def __init__(self, B=10, T=100):
        self.B = B
        self.T = T
        self.access_log = deque(maxlen=T)
    
    def can_access(self):
        return sum(self.access_log) < self.B
    
    def record_access(self, n=1):
        self.access_log.append(n)
```

Default: **B=10, T=100** — 10% access rate forced throughout episode.

---

## 4. The Experiment

### 4.1 Task: Associative Recall Under Interference

```
Phase 1 (Encode):    100 tokens  (50 key-value pairs)
Phase 2 (Delay):      50 tokens  (filler, no queries)
Phase 3 (Query):      20 tokens  (genuine keys, predict values)
──────────────────────────────
Total:               170 tokens
```

Example episode:
```
[KEY_A=5, VAL_A=12, KEY_B=7, VAL_B=3, ..., KEY_Z=2, VAL_Z=19]  ← Encode
[filler_1, filler_2, ..., filler_50]                           ← Delay
[KEY_A, ?, KEY_B, ?, ..., KEY_Z, ?]                             ← Query
```

### 4.2 Distractor Types

| Type | Description | Threat Level |
|------|-------------|--------------|
| **N (noise)** | Random tokens, no relationship to keys | Medium — fills memory |
| **K (key-like)** | Cosine similarity 0.65-0.75 to genuine keys | High — confuses routing |

Key-like distractors are the core threat: they look relevant, trigger memory access, but evict or overwrite genuine key-value pairs.

### 4.3 Sweep Matrix

| Axis | Values |
|------|--------|
| Distractor density | 0%, 20%, 40%, 60%, 80% |
| Distractor type | N, K |
| Budget ratio B/T | 0.05, 0.10, 0.20 |

**30 cells** × 200 episodes = 6,000 evaluation episodes.

### 4.4 Decision Criteria

**H₀ (Null):** No significant difference in degradation slope.

**H₁ (Alternative):** Architecture B's degradation slope is ≥2× shallower than Architecture A's, with ≥10 pp accuracy gap at 60% key-like distractors.

**Requires ALL three:**
1. B_full ≥ A + 10 pp at 60% Type K distractors
2. B_rand significantly worse than B_full (routing matters)
3. B_zero_h significantly worse than B_full (memory matters)

---

## 5. Ablations

| Ablation | Description | What It Tests |
|----------|-------------|---------------|
| B_full | Complete Architecture B | Full system |
| B_no_mem | Memory disabled (GRU-only) | Does memory actually help? |
| B_rand | Random routing | Does routing policy matter? |
| B_sched | Fixed write-every-5th | Is learned routing better than schedule? |
| B_zero_h | Hidden state zeroed at query | Does GRU state persistence matter? |
| B_no_sal | Salience gating disabled | Does gating actually help? |

---

## 6. Salience Signal

**Critical design decision: What signal drives salience?**

Prediction error would preferentially write distractors (they have high error!). Instead, salience = **entropy of routing distribution** — controller uncertainty about what to do:

```python
def salience(hidden, logits):
    probs = F.softmax(logits, dim=-1)
    entropy = -(probs * probs.log()).sum(dim=-1)  # high entropy = uncertain
    return entropy
```

High entropy → uncertain controller → write to memory (might need it later).
Low entropy → confident controller → skip (knows what to do).

---

## 7. Test Suite (107 Tests)

| Test File | Count | Coverage |
|-----------|-------|----------|
| `test_memory.py` | 15 | Read/write roundtrip, salience gating, eviction |
| `test_controller.py` | 9 | Hidden state persistence, gradient flow |
| `test_routing.py` | 16 | Temperature annealing, hard mode, budget signal |
| `test_budget.py` | 21 | Rolling window, access rate, exhaustion |
| `test_reinforce.py` | 14 | REINFORCE signal, sparsity penalty |
| `test_architecture.py` | 15 | End-to-end forward, budget enforcement |
| `test_persistent_state.py` | 17 | Cross-episode hidden state |

---

## 8. Project Structure

```
attention/
├── src/
│   ├── architecture.py          # ArchitectureB integration
│   ├── architecture.py
│   ├── arch/
│   │   ├── arch_b.py            # Architecture B implementation
│   │   ├── budget.py            # Budget utilities
│   │   ├── memory.py           # Hash table + eviction
│   │   └── routing.py          # Routing policy
│   ├── baseline/
│   │   └── transformer.py      # Transformer (Architecture A)
│   ├── budget/
│   │   └── enforcer.py         # Rolling-window budget
│   ├── controller/
│   │   └── gru_controller.py  # GRU cell
│   ├── experiment/
│   │   ├── ablations.py        # Ablation definitions
│   │   ├── distractors.py      # Distractor construction
│   │   ├── metrics.py          # Evaluation metrics
│   │   ├── sweep.py            # Sweep runner
│   │   └── task.py             # Associative recall task
│   ├── memory/
│   │   └── hash_table.py       # Differentiable hash table
│   ├── routing/
│   │   └── policy.py           # Learned routing
│   ├── training/
│   │   ├── reinforce.py        # REINFORCE training
│   │   └── supervised_routing.py  # Warm-start
│   └── utils/
│       └── io.py              # I/O utilities
├── tests/                      # 107 tests
├── results/                    # Experiment results
├── run_experiment.py           # Experiment entry point
├── notebooks/                  # Analysis notebooks
└── draft.md, foundation.md, experiment.md, status.md  # Research docs
```

---

## 9. Key Research Question

**Does structural separation provide interference resistance that homogeneous attention cannot achieve?**

This is an architectural question, not a scaling question. The answer matters for understanding what transformers are actually doing — and what alternative architectures might do differently.

---

## 10. Relationship to Nue/CAR

Architecture B's **GRU controller** is conceptually similar to Nue's **router**:
- Both decide what to do with incoming information
- Both have persistent state that informs decisions

The difference:
- Architecture B: Discrete actions (PERCEIVE/READ/WRITE/SKIP)
- Nue/CAR: Soft routing weights (3 continuous paths)

The budget enforcer is analogous to the CAR error threshold — both enforce computational constraints that force genuine prioritization.
