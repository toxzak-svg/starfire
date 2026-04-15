# AFI Over Markov Chains: Metastable Partition Formalism

> **Attractor = metastable set in a Markov chain.** No asymptotic fixed points required.

---

## 1. The Metastable Partition

Let `S = {s₁, s₂, ..., sₖ}` be the finite set of identified metastable reasoning modes:
- `s₁` = SymbolicManipulation
- `s₂` = EmotionalResonance
- `s₃` = CausalReasoning
- `s₄` = AssociativeRecall
- `s₅` = Exploratory
- `s₆` = SteadyState

The system is always in exactly one mode at any step `t`. Its state is a probability vector:

```
p(t) ∈ Δᵏ  (simplex of dimension k)
p_i(t) = Pr[X_t = s_i]
```

Let `T(t,t+τ)` be the transition matrix for lag τ:
```
[T(t,t+τ)]_{ij} = Pr[X_{t+τ} = s_j | X_t = s_i]
```

Under stationarity (after warmup), `T(t,t+τ) ≈ T(τ)` is approximately time-homogeneous for a given regime.

---

## 2. Metastability Criteria

A subset `M ⊂ S` is **metastable** if trajectories starting in `M` remain in `M` for long enough to satisfy:

```
P(exit from M before τ_task) < ε_exit
```

where `τ_task` is the task-relevant timescale (e.g., 5-20 tokens for a single reasoning step).

This is operationalized via **mean first passage time (MFPT)**:

```
MFPT(i→j) = expected steps to reach s_j starting from s_i, first hitting j before returning to i
```

The **metastable set M** around mode `s_i` is:
```
M_i = { s_j | MFPT(i→j) > τ_task }
```

i.e., modes that are "sticky" relative to the task horizon.

---

## 3. AFI Over Markov Chains

Define the **Metastable Attractor Fragility Index** as a weighted combination:

```
AFI = w_λ · AFI_λ   +   w_μ · AFI_μ   +   w_κ · AFI_κ
```

### AFI_λ: Spectral Fragility (Mixing Rate)

From the transition matrix `T`, compute eigenvalues:
```
|λ₁| ≥ |λ₂| ≥ ... ≥ |λ_k|
```

`λ₁ = 1` (stationary state). The second eigenvalue `|λ₂|` governs mixing rate:
- `|λ₂| ≈ 1` → slow mixing → system gets trapped in metastable states
- `|λ₂| ≈ 0` → fast mixing → system escapes metastable states quickly

Define mixing time:
```
τ_mix = -1 / ln(|λ₂|)
```

Then:
```
AFI_λ = 1 / (τ_mix + 1)        (high when mixing is fast = fragile)
```
Alternative:
```
AFI_λ = (1 - |λ₂|) · κ_dyn     (scaled by fraction of dynamism)
```

### AFI_μ: Occupancy Imbalance (Mode Rigidity)

Deviation from uniform stationary distribution `π`:
```
π_i = stationary probability of mode s_i
π_uniform = 1/k
```

```
AFI_μ = (1/k) · Σ_i |π_i - 1/k|      (Gini coefficient of occupancy)
```
- `AFI_μ = 0` → uniform occupancy (system explores all modes freely)
- `AFI_μ = 1` → all mass on one mode (system is rigid / trapped)

### AFI_κ: Condition Number of Transition Graph

Condition number of `T` as a linear operator:
```
κ(T) = ‖T‖ · ‖T⁻¹‖
```
(in practice: ratio of largest to smallest non-trivial singular value)

High condition number → some directions in mode-space are stiff (hard to perturb),
other directions are complaint → the basin is anisotropic (shaped like a ridge or valley).

```
AFI_κ = log10(κ(T)) / log10(κ_max)      (normalized)
```

### Weights

```
w_λ + w_μ + w_κ = 1
```

Recommended: `w_λ = 0.4, w_μ = 0.3, w_κ = 0.3`

---

## 4. Perturbation Analysis

A **perturbation** `δp` to the state `p` produces a new state:
```
p' = p + δp
```

After one step:
```
p(1) = T · p
p'(1) = T · p'
```

The **fragility to perturbation** is:
```
‖p(1) - p'(1)‖ = ‖T · δp‖
```

The **worst-case perturbation** (that maximizes divergence) is the singular vector
of `T` corresponding to the largest singular value — same as the leading eigenvector
for the transpose. This gives an *actionable* perturbation direction: nudge the system
toward or away from a specific metastable mode.

---

## 5. Regime-conditional AFI

Let `AFI(s_i)` be the AFI computed *conditioned on being in metastable state s_i*:

```
AFI(s_i) = w_λ · λ_2(s_i) + w_μ · (1 - π_i) + w_κ · κ(T_{s_i})
```

Where:
- `λ_2(s_i)` = second eigenvalue of the **local transition matrix** restricted to
  states reachable from `s_i` within the metastable set
- `π_i` = stationary probability of `s_i`
- `κ(T_{s_i})` = condition number of the local transition subgraph around `s_i`

This gives a per-mode fragility map. ASRU can use this to decide where to freeze vs plastic.

---

## 6. Two-Timescale Update Law (Metastable Version)

### Fast Loop (step-level, continuous)
```
Δp = -η_fast · ∇_p AFI(p)
p ← softmax(log(p) + Δp)      (projected back to simplex)
```

Where `AFI(p) = Σ_i p_i · AFI(s_i)` is the occupation-expectation of AFI.

### Slow Loop (episodic, every N steps)
```
η_slow = η_slow_base · exp(-β · U_basin)

θ ← θ - η_slow · ∇_θ U_basin(θ)
```
Where `θ` are the parameters governing the transition matrix structure
(e.g., bias terms that shape `T`).

### Basin Uncertainty `U_basin`

Approximated from the stationary distribution entropy:
```
H(π) = -Σ_i π_i · ln(π_i)
U_basin = 1 - H(π) / ln(k)           (normalized to [0,1])
```
- `U_basin ≈ 0` → uniform spread (basin is uncertain, many modes accessible)
- `U_basin ≈ 1` → concentrated on few modes (basin is well-defined, few pathways)

**Connection to Lyapunov:** The mixing time `τ_mix` serves as the Lyapunov timescale.
Fast mixing (`τ_mix` small) ↔ positive leading Lyapunov exponent (chaotic divergence).
This bridges the continuous and discrete formalisms.

---

## 7. Identifiability of Metastable Modes

The modes `S` must be **discovered**, not hand-specified. Use:

### Method 1: Spectral Clustering on Activation Trajectories
1. Collect activation trajectories `{a_t}` from LLM layers during diverse reasoning tasks
2. Project to low-dim via PCA/UMAP (top 10-20 components)
3. Build transition matrix from discretized states (bin the embedding space)
4. Run spectral clustering on `T + T^T` (symmetrized) to recover metastable sets
5. Validate via MFPT statistics and within-cluster coherence

### Method 2: HMM on Behavioral Traces
1. Define behavioral observable `b_t = f(activation_t)` (e.g., which tool was called,
   which reasoning pattern fired, log-prob of next token)
2. Fit Hidden Markov Model with K states (K = hypothesized number of metastable modes)
3. HMM states ≈ metastable reasoning modes
4. Use Viterbi decoding to assign each timestep to a mode

### Method 3: Transfer Operator / Perron-Frobenius Operator
1. Build a kernel `K(x,y)` from activation similarities (e.g., Gaussian kernel on embeddings)
2. Compute the top eigenvectors of the associated transfer operator (Ulam method)
3. Eigenfunctions partition the state space — the partition gives metastable sets
4. This is the rigorous dynamical systems approach (Schütte et al.)

### Validation
- **MFPT consistency**: modes should have long within-mode MFPT
- **Cross-seed stability**: modes should be similar across different random seeds
- **Task alignment**: modes should correspond to interpretable reasoning strategies
- **Causal isolation**: intervening on one mode should affect only its neighbors

---

## 8. Computing AFI in Practice (Algorithm)

```
INPUT: sequence of mode assignments {s[t]} for t = 1...T, or activation trajectories {a[t]}
OUTPUT: AFI scores per mode

1. BUILD TRANSITION MATRIX
   For each adjacent pair (s[t], s[t+τ]):
     T[s[t], s[t+τ]] += 1
   T ← T / row_sums (normalize)

2. COMPUTE STATIONARY DISTRIBUTION π
   Power iteration: π ← π · T until ‖π·T - π‖ < ε
   Or: dominant eigenvector of T^T

3. SPECTRAL ANALYSIS
   λ = eig(T)           // eigenvalues
   λ₂ = second largest eigenvalue by magnitude
   τ_mix = -1 / ln(|λ₂|)

4. METASTABLE SETS
   For each mode s_i:
     MFPT(i→j) for all j
     M_i = { j | MFPT(i→j) > τ_task }

5. AFI SCORES
   AFI_λ = 1 / (τ_mix + 1)
   AFI_μ = (1/k) · Σ_i |π_i - 1/k|
   AFI_κ = log10(κ(T)) / log10(κ_max)
   AFI = w_λ·AFI_λ + w_μ·AFI_μ + w_κ·AFI_κ

   AFI(s_i) per-mode = w_λ·(1-|λ₂(i)|) + w_μ·(1-π_i) + w_κ·κ_i
```

---

## 9. Relation to Original Continuous AFI

| Continuous AFI | Markov Chain AFI |
|---|---|
| Leading Lyapunov exponent λ₁ | Spectral radius `|λ₂|` (mixing rate) |
| Finite-time Lyapunov divergence | Mean first passage time (MFPT) |
| Basin uncertainty α | Stationary distribution entropy `H(π)` |
| RQA metrics (det, max_line, laminarity) | Graph-theoretic equivalents on `T` |
| Perturbation field `δx` | Perturbation vector `δp` (simplex) |
| Attractor as invariant set | Metastable set as absorbing subgraph |

The two formalisms agree in the limit where the continuous system is discretized finely enough
that trajectories are well-approximated by a Markov chain on a partition of state space
(molecular dynamics literature: Chodera et al., Proc. Natl. Acad. Sci. 2007).

---

## 10. Key Insight: What This Enables

Once AFI is defined over Markov chains:
- **Regime routing**: ASRU routes based on current metastable mode occupation vector `p(t)`
- **Anticipatory symmetry breaking**: Pre-assign column roles based on predicted next mode
  (computed from `T^n` — n-step-ahead distribution)
- **Tool-triggered freezing**: When a tool fires, `T` is perturbed — AFI spike triggers
  temporary plasticity freeze to stabilize the new metastable configuration
- **Basin shaping**: Slow loop modifies bias terms to reshape `T` toward desired metastable structure

The pipeline from behavioral trace → Markov chain → AFI → ASRU update is fully
operationalizable and makes the architecture empirically testable.
