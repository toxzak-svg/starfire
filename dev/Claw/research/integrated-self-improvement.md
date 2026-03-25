# Integrated Self-Improvement for Claw

Based on: C:\dev\selfhelper-main\self_improving_agent\intrinsic.py

## Intrinsic Rewards for Evolution

From selfhelper-main, adapted for my evolution system:

```python
# Useful novelty - how different is this experiment from prior ones?
def useful_novelty(proposal_config, prior_configs, predicted_gain):
    distance = min(euclidean(proposal_config, cfg) for cfg in prior_configs)
    return distance * max(0.0, predicted_gain)

# Empowerment bonus - what type of change?
empowerment_bonuses = {
    "eval_harness": 0.40,    # Better evaluation
    "profiling": 0.30,        # Better measurement
    "logging": 0.25,         # Better tracking
    "infra_speedup": 0.35,   # Faster execution
    "dataset_checks": 0.30,  # Better validation
}

# Total reward = info_gain + learning_progress + useful_novelty + empowerment
```

## Integration Ideas

### 1. Novelty Tracking
- Keep vector of my past experiments
- Reward experiments that try NEW things, not just repeats
- Prevent getting stuck in local optima

### 2. Empowerment Tracking  
- Track what types of improvements I've made
- Bonus for infrastructure (faster, better logging)
- Not just "better scores" but "more capable"

### 3. Failure-Informed Tool Invention
- When a test fails → analyze WHY
- Propose a new tool/capability to address gap
- This is what selfhelper-main does!

## What I Can Actually Implement Now

1. **Experiment config vectors** - serialize what each experiment changes
2. **Novelty scoring** - reward different approaches
3. **Empowerment categories** - track types of improvements

Let me implement these in my evolution system.
