# Temporal-Attention Store
## Technical Report

**Project Path:** `/home/zach/.openclaw/workspace/dev/temporal-attention/`
**Language:** Python
**Status:** Prototyping complete; multiple benchmark variants
**Last Updated:** 2026-04-01

---

## 1. Overview

The insight behind this project:

> **Pure temporal decay treats all old facts equally. Pure attention ignores time (can return stale facts). Combined: both signals weighted for smarter retrieval.**

**TemporalAttentionStore** combines:
1. **Temporal validity** — time window enforcement (hard filter)
2. **Attention signal** — access frequency as relevance proxy
3. **Decay over time** — temporal distance reduces relevance

A fact outside its validity window is **never returned**, regardless of attention score.

---

## 2. Core Data Model

```python
@dataclass
class Fact:
    key: str                           # Fact identifier
    value: Any                         # Stored value
    valid_from: datetime               # Window start
    valid_to: Optional[datetime]       # Window end (None = current)
    access_count: int = 0              # Times retrieved
    last_accessed: Optional[datetime]  # For recency tracking
    created_at: datetime               # When stored

@dataclass
class ScoredFact:
    fact: Fact
    temporal_score: float  # 0-1, how recent
    attention_score: float  # 0-1, how frequently accessed
    combined_score: float   # Weighted combination
    is_valid: bool          # HARD filter (respects time window)
```

---

## 3. Scoring System

```python
def score(fact, now, temporal_weight=0.5, attention_weight=0.5):
    # Temporal score: exponential decay from creation
    hours_old = (now - fact.created_at).total_seconds() / 3600
    temporal = 2.0 ** (-hours_old / half_life_hours)
    
    # Attention score: recency-weighted access frequency
    hours_since_access = (now - fact.last_accessed).total_seconds() / 3600
    attention = fact.access_count / (1 + hours_since_access / attention_decay)
    
    # Combined with domain-appropriate weights
    combined = temporal_weight * temporal + attention_weight * attention
    
    # Validity is HARD — never returned if outside window
    is_valid = valid_now(fact, now)
    
    return ScoredFact(..., combined_score=combined, is_valid=is_valid)
```

---

## 4. Validity as a Hard Constraint

```python
def valid_now(fact, now):
    if fact.valid_from > now:
        return False  # Not yet valid
    if fact.valid_to is not None and fact.valid_to < now:
        return False  # No longer valid
    return True

def retrieve(key, now):
    candidates = [f for f in facts[key] if valid_now(f, now)]
    if not candidates:
        return None
    return best_scoring(candidates, now)
```

**Key invariant:** A fact that is not currently valid is **never returned**, regardless of its combined score.

---

## 5. Retrieval Algorithm

1. **Lookup by key** — get all facts with matching key
2. **Filter by validity** — discard anything outside time window
3. **Score remaining** — combine temporal + attention signals
4. **Return best** — top scorer wins

---

## 6. Benchmark Variants

| File | Focus |
|------|-------|
| `benchmark_simple.py` | Basic correctness |
| `benchmark.py` | Standard benchmark |
| `benchmark_event.py` | Event-driven workload |
| `benchmark_extreme.py` / `v2/` | High-density scenarios |
| `benchmark_float.py` | Floating-point values |
| `benchmark_sota.py` | SOTA comparison |
| `benchmark_hard.py` / `hard_float.py` | Challenging cases |

---

## 7. Relationship to Other Projects

- **Star's memory**: Similar decay concept, but Star's is domain-tagged (identity vs empirical vs episodic). TemporalAttentionStore's contribution is formalizing validity as a hard constraint.
- **COG-research**: The attention-score concept (access frequency) parallels the "attention as salience" signal in imagination-based learning.
