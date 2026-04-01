# GWL — Global Workspace Theory Implementation
## Technical Report

**Project Path:** `/home/zach/.openclaw/workspace/gwl/`
**Type:** PyTorch — Global Workspace Theory (Baars/Dehaane)
**Language:** Python
**Status:** Prototype implementation
**Last Updated:** 2026-04-01

---

## 1. Overview

GWL implements **Global Workspace Theory** — a cognitive architecture from consciousness studies — as a neural network.

**Core concept:** Multiple specialized **processors** (perception, memory, action planning, etc.) each have limited access to each other. Access is mediated by a **global workspace** — a bottleneck where information competes for broadcast. What wins the competition becomes available to all processors simultaneously.

```
Processor A ──┐
Processor B ──┼──→ [Global Workspace] ──→ All Processors
Processor C ──┘         ↑
                    Competition
```

**Why it matters for intelligence:**
- Selective attention at the architectural level (not just attention layers)
- Information integration — diverse perspectives compete, best wins
- Consciousness-like dynamics — what "enters awareness" is what gets broadcast

---

## 2. Components

### 2.1 Processor

A processor reads from and writes to the workspace. Each has a unique perspective on input:

```python
class Processor(nn.Module):
    # MLP, LSTM, or Transformer core
    # Input: embedding
    # Output: workspace query + value
```

### 2.2 Competition

Processors compete via learned or fixed competition function:

```python
def competition(processor_outputs):
    # winner-take-all, softmax, or stochastic
    # winner's information gets broadcast
```

### 2.3 Global Workspace

The broadcast mechanism:

```python
def global_workspace(winner_output):
    # Broadcast to all processors
    # All processors receive the same information simultaneously
    return broadcast
```

---

## 3. Relationship to Other Projects

| Project | Relationship |
|---------|-------------|
| **Attention Architecture Research** | Both explore selective attention as core mechanism. GWL is theory-driven (Global Workspace), Architecture B is experiment-driven. |
| **Nue/CAR** | GWL's competition = CAR's router. GWL broadcasts = CAR's routing weights. |
| **Star** | Star's architecture (4 layers) parallels GWL's processor/workspace split. |

---

## 4. Files

| File | Purpose |
|------|---------|
| `model.py` | Core GWL model (Processors + Competition + Workspace) |
| `config.py` | Configuration dataclass |
| `trainer.py` | Training loop |

---

## 5. Theoretical Grounding

**References:**
- Baars, B. J. — Global Workspace Theory of consciousness
- Dehaene, S. — Neural correlates of conscious access
- Shanahan, M. — Global Workspace and the "theater" model

The Global Workspace is a theory from cognitive science that posits consciousness arises from information being broadcast globally in the brain. GWL tests whether this principle, implemented as a neural architecture, produces useful intelligence properties.
