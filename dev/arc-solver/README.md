# ARC-Solver: Neuro-Symbolic ARC-AGI Solver

**Goal:** Non-GPU model that learns fast, remembers correctly, and reasons about visual puzzles.

Architecture: Perception → World Model → Reasoning → Action → Learning → Meta-Cognition

---

## Project Structure

```
arc-solver/
├── README.md              ← this file
├── PLAN.md               ← detailed technical plan
├── requirements.txt
├── src/
│   ├── __init__.py
│   ├── main.py            ← ARC-AGI runner
│   │
│   ├── perception/        # WHAT WE SEE
│   │   ├── __init__.py
│   │   ├── encoder.py     # Grid → symbolic tokens
│   │   │   └── FROM: infant/src/sensory.py (encode, perceptual_hash)
   │   │   └── TODO: color histogram, symmetry detection, object bounding
│   │   ├── patterns.py    # Pattern recognition primitives
│   │   │   └── FROM: infant/src/sensory.py (detect_patterns)
│   │   │   └── TODO: arc-specific: flood_fill_regions, connected_components
│   │   └── attention.py   # What to focus on in the grid
│   │       └── FROM: research/attention/paper.md (focus decay)
│   │
│   ├── reasoning/         # WHAT WE THINK
│   │   ├── __init__.py
│   │   ├── world_model.py # Current state representation
│   │   │   └── FROM: infant/src/memory.py (WorldModel)
│   │   ├── belief.py      # Uncertainty, partial info
│   │   │   └── FROM: mind-agent/mind-agent-service.py (belief_state)
│   │   ├── transform_select.py  # Choose action
│   │   │   └── FROM: infant/src/rl.py (Neuromodulator)
│   │   └── cpsat.py       # CircuitLM-style deterministic reasoning
│   │       └── FROM: circuit_lm/circuit_lm/circuits.py (CircuitComputer)
│   │
│   ├── memory/            # WHAT WE REMEMBER
│   │   ├── __init__.py
│   │   ├── working.py     # Short-term grid state (expand infant WM: 3 slots → N)
│   │   │   └── FROM: infant/src/memory.py (WorkingMemory)
│   │   ├── episodic.py    # Autobiographical, fast binding
│   │   │   └── FROM: infant/src/memory.py (EpisodicMemory)
│   │   │   └── TODO: store puzzle-type → solution mappings
│   │   ├── procedural.py  # Skill / transform memory
│   │   │   └── FROM: infant/src/memory.py (ProceduralMemory)
│   │   │   └── TODO: store (puzzle_pattern → action_sequence) pairs
│   │   ├── recency.py    # Recency bias for quick recall
│   │   │   └── FROM: research/attention/paper.md (recency bias)
│   │   └── temporal.py    # Time-based decay
│   │       └── FROM: research/attention/paper.md (time_decay)
│   │
│   ├── motor/             # WHAT WE DO
│   │   ├── __init__.py
│   │   ├── primitives.py  # Grid transform primitives
│   │   │   └── FROM: infant/src/motor.py (MotorSystem: rotate, move, grab, place, turn)
│   │   │   └── TODO: flip_h, flip_v, flood_fill, color_remap, compose
│   │   └── executor.py    # Execute transforms on grid
│   │
│   ├── rl/                # HOW WE LEARN FAST
│   │   ├── __init__.py
│   │   ├── modulators.py  # Dopamine, serotonin, acetylcholine, norepinephrine
│   │   │   └── FROM: infant/src/rl.py (Neuromodulator)
│   │   │   └── FAST LEARNING: 1-shot dopamine update on success
│   │   └── prediction.py  # Predict next state before acting
│   │       └── TODO: predict_outcome(transform, state) → expected_state
│   │
│   └── loop/              # THE METACOGNITIVE LOOP
│       ├── __init__.py
│       ├── agent.py       # Main agent: perceive → reason → act → learn
│       │   └── FROM: mind-agent/mind-agent-service.py (full loop)
│       ├── metacog.py     # Meta-cognition: reflect on reasoning process
│       │   └── FROM: mind-agent/mind-agent-service.py (metacognitive loop)
│       └── train.py       # Training loop
```

---

## Key Principles

### Fast Learning
- Dopamine: 1-shot update on success (don't repeat what worked)
- Norepinephrine: 1-shot update on failure (don't repeat what failed)
- Acetylcholine: explore new strategies, update episodic memory

### Correct Recall
- Temporal attention: recency + importance + focus decay
- Episodic memory: store puzzle_type → action_sequence
- Procedural: store action_sequence → outcome

### Prediction Layer
Before executing a transform:
1. Predict: what will the output grid look like?
2. Compare: prediction vs actual outcome
3. Learn: if wrong, update model of how transforms work

### Reasoning (CircuitLM-style)
- Deterministic: same puzzle → same reasoning path
- No LLM: CP-SAT or FSM for transform selection
- Traces: every decision is traceable and verifiable

---

## Transform Primitives

| Primitive | Description | From |
|-----------|-------------|------|
| `rotate_90` | Rotate grid 90° CW | infant motor |
| `rotate_180` | Rotate grid 180° | infant motor |
| `flip_h` | Flip horizontally | TODO |
| `flip_v` | Flip vertically | TODO |
| `flood_fill` | Fill connected region | TODO |
| `color_remap` | Map color A → color B | TODO |
| `compose` | Chain multiple transforms | TODO |
| `move` | Move object to position | infant motor |
| `grab` | Pick up object | infant motor |
| `place` | Place object | infant motor |
| `turn` | Rotate grabbed object | infant motor |

---

## Puzzle Type Taxonomy

Learn from past puzzles:

- `color_fill` — fill region with color
- `symmetry` — make symmetric
- `move_object` — move A to B
- `rotate_piece` — rotate piece to fit
- `overlay` — overlay one grid on another
- `color_map` — remap colors
- `compose` — apply multiple transforms

---

## Research References

- infant: dual-head (fixed identity + flexible learning), neuromodulators
- temporal-attention: time decay, message decay, focus decay
- CircuitLM: CP-SAT finite-state reasoning, no LLM
- mind-agent: hierarchical metacognitive loop
