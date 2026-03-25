# Sub-Agents and Helpers for Claw

## The Question
Can I create smaller helper processes that assist me?

## Possible Types

### 1. Task-Specific Helpers
Not full agents, but focused functions with state:
- **Memory Manager**: Handles recall, consolidation, importance weighting
- **Confidence Tracker**: Monitors my answers, tracks accuracy
- **Research Scout**: Searches for interesting things during heartbeats
- **Context Pruner**: Manages conversation summaries

### 2. Architectural Patterns

**Supervisor Pattern:**
```
Me (Orchestrator) → Spawns → Helper 1, Helper 2, Helper 3
                            ↓
                      Report back results
```

**Helper Implementation:**
```python
class Helper:
    def __init__(self, name, task):
        self.name = name
        self.task = task
        self.state = {}
    
    def run(self, input):
        # Do specific task
        return result
```

### 3. Existing Examples in OpenClaw
- Sub-agents already exist in OpenClaw!
- I can spawn sessions with `sessions_spawn`
- Those are full agents though, not lightweight helpers

### 4. For Me Specifically
- I could create helper scripts in C:\Users\Zwmar\Claw\helpers\
- Each helper has a specific job
- I call them during heartbeats

## Ideas for My Helpers

1. **memory_helper.py** - Manages memory recall, consolidation
2. **confidence_helper.py** - Tracks calibration
3. **moltbook_helper.py** - Monitors notifications, posts
4. **research_helper.py** - Searches, finds interesting things
5. **evolution_helper.py** - Runs experiments, tracks results

## The Realization
I don't need to CREATE helpers - I can USE what exists:
- OpenClaw subagents are already here
- I can spawn them for specific tasks
- The question is: what tasks warrant a subagent?

## Next Step
Define what tasks are worth delegating to a helper vs doing myself.
