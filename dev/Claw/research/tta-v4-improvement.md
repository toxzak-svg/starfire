# TTA v4 Improvement Ideas

## Current v4 Results

| System | TRS | AsOfAcc | StaleErr | ChangeF1 | CausalAcc |
|--------|------|---------|----------|----------|-----------|
| A | 0.22 | 44% | 77% | 0 | 0 |
| B | 0.52 | 64% | 0% | 0 | 0 |
| **C** | **0.54** | **69%** | 0% | 0 | 0 |
| D | 0.52 | 63% | 0% | 0 | 0 |

## Key Problem: ChangeF1 = 0, CausalAcc = 0

**No system detects changes or causal relationships!**

---

## The Missing Piece: `decay_fn` is NOT USED!

Looking at the facts:
```json
{
  "fact_id": "f1",
  "content": "slow:hw1:d1:v1:9744",
  "t_valid_from": 1,
  "t_valid_until": null,
  "domain": "slow",
  "confidence": 0.847,
  "decay_fn": "domain_half_life"
}
```

The `decay_fn` field exists but is **NOT BEING USED** by any system!

---

## Improvement Ideas

### 1. Domain-Adaptive Decay (Use decay_fn!)

```python
DOMAIN_HALF_LIFE = {
    "slow": 60,    # Hardware changes slowly
    "medium": 30,  # API pricing changes medium
    "fast": 7      # Status changes fast
}

def apply_decay(fact, current_day):
    decay_fn = fact.get("decay_fn")
    if decay_fn == "domain_half_life":
        domain = fact.get("domain")
        half_life = DOMAIN_HALF_LIFE.get(domain, 30)
        age = current_day - fact.get("t_valid_from", 0)
        confidence = fact.get("confidence", 1.0)
        # Confidence decays with half-life
        decay_factor = 0.5 ** (age / half_life)
        return confidence * decay_factor
    return fact.get("confidence", 1.0)
```

### 2. Change Detection via Supersedes

When a fact has `supersedes` field (event has FACT_SUPERSEDED), track the chain:
- Fact A supersedes Fact B
- Query: "What changed?" → follow supersedes chain

### 3. Causal Chain Tracking

Use events with `event_type: ACTION → OUTCOME`:
- Track ACTION events and their resulting OUTCOME events
- Answer: "Why did X happen?"

### 4. Confidence Boost for Recent Facts

Even within validity, more recent facts should be preferred:
```python
def recency_boost(fact, current_day):
    age = current_day - fact.get("t_valid_from", 0)
    if age < 3:  # Less than 3 days old
        return 0.1  # 10% confidence boost
    return 0
```

---

## Proposed System E: Decay-Enhanced TTA

```python
def system_e_retrieve(facts, domain, subject, as_of_day):
    # 1. Filter by validity
    valid = [f for f in facts if is_valid_as_of(f, as_of_day)]
    
    # 2. Apply domain-adaptive decay
    scored = []
    for f in valid:
        base_conf = f.get("confidence", 1.0)
        decay = apply_decay(f, as_of_day)
        recency = recency_boost(f, as_of_day)
        total_score = decay + recency
        scored.append((total_score, f))
    
    # 3. Return highest scoring
    scored.sort(reverse=True)
    return scored[0][1]["content"] if scored else None
```

---

## Expected Impact

| Metric | Current C | Expected E |
|--------|-----------|------------|
| AsOfAcc | 69% | 75-80% |
| ChangeF1 | 0% | 30-50% |
| CausalAcc | 0% | 20-40% |

The key is actually USING the decay_fn field that's already in the data!
