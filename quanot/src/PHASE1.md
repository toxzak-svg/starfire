# QuaNot Phase 1: Quantum-Inspired Foundations
## Status: Implementation

**Started:** 2026-04-03
**Goal:** Implement core quantum-inspired and chaotic reservoir components

---

## Goals for Phase 1

1. [ ] Set up Python environment (conda + all dependencies)
2. [ ] Baseline Echo State Network (ESN) with chaotic modulation
3. [ ] Lyapunov exponent calculator (real-time monitoring)
4. [ ] RQA consciousness proxy metrics
5. [ ] Simulated Quantum Annealing (SQA) Ising/QUBO solver
6. [ ] Tensor network compression module
7. [ ] Integrate with starfire API (optional Day 15 stretch goal)

---

## Implementation Notes

### ESN Baseline
- Target: N=1000 reservoir, spectral radius = 0.95
- Test: NARMA-10 task (standard ESN benchmark)
- Add chaotic modulation: ε * tanh(state) perturbation

### Lyapunov Monitor
- Algorithm: Benettin (tangent vector propagation)
- Online: sliding window, N=100 iterations per window
- Control: if MLE > threshold, reduce spectral radius

### RQA Metrics
- REC (recurrence), DET (determinism), LAM (laminarity)
- Use scipy.spatial.distance.pdist for pairwise distances
- Threshold: 0.1 (cosine distance)

### SQA Solver
- Trotter number P=10-20
- N up to 500 (Ising/QUBO)
- Parallel tempering across replicas
- Use NumPy vectorization

---

## Code Structure (src/)

```
src/
├── __init__.py
├── reservoir.py      # ChaoticReservoir (ESN)
├── chaos.py          # Lyapunov, fractal dimension
├── consciousness.py  # RQA, GWT simulation
├── quantum_inspired.py  # SQA, tensor networks
└── main.py          # Phase 1 demo
```

---

## Running the Demo

```bash
cd projects/quanot
conda activate quanot
python src/main.py
```

---

## Dependencies

All in `environment.yml`:
- python=3.11
- numpy, scipy, matplotlib, networkx
- pytorch (cpuonly)
- pyphi (for small-system Φ only)
- cma (CMA-ES optimization)
- deap (genetic algorithms)

---


