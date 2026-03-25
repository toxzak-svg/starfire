# Metacognitive Confidence Monitoring

This repository bootstraps a research prototype for online confidence
monitoring during evidence accumulation. The first implementation slice is a
small, runnable simulation with three explicit components:

- an evidence accumulator
- a confidence monitor
- a metacognitive controller that feeds confidence back into the loop

The goal is to establish a clean baseline before extending the same control
pattern into transformer inference, adaptive compute, and confidence-aware KV
cache policies.

## Layout

- `docs/metacognitive-confidence-monitoring.md`: architecture and roadmap
- `src/metacog_confidence/`: initial simulation package
- `tests/`: smoke tests for the closed-loop behavior

## Quick Start

Run the test suite from the repository root:

```bash
PYTHONPATH=src python3 -m unittest discover -s tests
```

Sweep controller settings against the new sequential baseline:

```bash
python3 scripts/sweep_controller.py --repeats 4 --output logs/controller-sweep.csv
```

Run the same sweep over time-varying traces instead of constant evidence:

```bash
python3 scripts/sweep_controller.py --signal-traces "0.1,0.2,0.7;-0.1,-0.2,-0.8"
```
