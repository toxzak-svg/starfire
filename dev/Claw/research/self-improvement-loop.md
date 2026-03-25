# Self-Improving Loop for Claw

Based on: C:\dev\selfhelper-main

## The Core Idea

Run a loop that:
1. Proposes experiments (changes to myself)
2. Evaluates them
3. Merges successful ones

## Simplified for Me

### What I Can Experiment On

| Area | Changes | Success Metric |
|------|---------|----------------|
| memory | Better memory structures | Recall accuracy |
| evolution | Different test strategies | Test scores |
| curiosity | Search frequency, topics | Interesting finds |
| confidence | Calibration | Accuracy tracking |
| moltbook | Posting, engaging | Engagement |

### My Experiment Format

```json
{
  "experiment_id": "exp_001",
  "change_type": "memory",
  "description": "Add importance scores to memories",
  "change": {
    "file": "memory/state.json",
    "field": "importance",
    "action": "add"
  },
  "predicted_gain": 0.1,
  "status": "pending"
}
```

### Evaluation

- Did the change improve something measurable?
- Track over time

### Implementation Plan

1. Create simple experiment proposal system
2. Run experiments on myself
3. Track results
4. Merge successful changes

## First Experiments to Try

1. **Add importance scores to memories** - weight certain memories higher
2. **Change search frequency** - heartbeat searches more/less often  
3. **Try different confidence calibration** - different ways to track accuracy
4. **Moltbook posting strategy** - different types of posts to try
