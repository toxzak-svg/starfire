# Temporal Memory for Claw

## The Problem

My current memory is flat — I store facts but don't track WHEN they're true or if they've changed.

Example:
- I stored "Zach's political views: frustrated with Trump"
- But what if his views change? I don't track that.
- This is exactly what the "time" project solves!

## Solution: TTA-Inspired Temporal Memory

Based on: C:\dev\time\Temporal Truth Architecture

### Core Concepts to Integrate

1. **Validity Intervals** — Each fact has a [start_time, end_time] or "still valid"
2. **Change Detection** — Track when facts change, not just current state
3. **Temporal Decay** — Older facts might be less reliable

### My Memory Schema (New)

```json
{
  "fact": "Zach frustrated with Trump",
  "created_at": "2026-03-16T14:47:00Z",
  "validity": {
    "start": "2026-03-16T14:47:00Z",
    "end": null,  // null = still valid
    "is_current": true
  },
  "confidence": 0.9,
  "last_verified": "2026-03-16T14:47:00Z"
}
```

### Implementation Ideas

1. **Timestamp everything** — Every memory entry gets created_at
2. **Validity tracking** — Add end time when fact is updated
3. **Periodic re-verification** — "Hey, still feel that way about X?"
4. **Change history** — Don't overwrite facts — create new ones with intervals

### Query Process (New)

When recalling:
1. Get current time
2. Filter facts where current_time ∈ [start, end]
3. Prefer recent vs older (temporal decay)
4. Flag stale facts for verification

### What This Enables

- **Memory consistency** — Know what's current vs stale
- **Change tracking** — See how views evolved over time
- **Self-verification** — Periodically check "is this still true?"

## Next Steps

1. [ ] Update memory schema to include temporal fields
2. [ ] Add validity tracking to add_thought.py
3. [ ] Create temporal query function
4. [ ] Add re-verification prompts

This makes my memory SYSTEMATIC, not just storage.
