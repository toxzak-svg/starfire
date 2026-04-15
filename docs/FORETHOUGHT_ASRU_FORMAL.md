# ASRU — Anticipatory Self-Reshaping Update: Formal Specification

## Core Definition

The ASRU update law operates on a two-timescale architecture:

```
Fast loop (τ_fast ≈ O(1 step)): 
    M_t ← M_{t-1} - η_fast · ∇_M F̂(I_local(z_t))
    
Slow loop (τ_slow ≫ τ_fast, episodic):
    θ ← θ - η_slow · ∇_θ U_basin(θ)
    recalibrate F̂ using new U_basin observations
```

Where:
- `M_t` = meta-state field (routing, plasticity, evaluation, interface)
- `θ` = base system parameters (column roles, symmetry-breaking assignment)
- `z_t` = latent state trajectory at time t
- `I_local` = local instability score (LLE spectrum + RQA)
- `U_basin` = basin uncertainty score (periodic, expensive)
- `F̂(I_local)` = learned surrogate approximating U_basin from cheap local features

---

## 1. Local Instability Score I_local

### 1a. Finite-Time Lyapunov Exponent Spectrum

```rust
struct LyapunovEstimator {
    /// Number of nearest neighbors to use for divergence estimation
    k_neighbors: usize,
    /// Window size for trajectory segment
    window: usize,
}

impl LyapunovEstimator {
    /// Estimate leading finite-time Lyapunov exponent along trajectory
    fn leading_lle(&self, trajectory: &[State]) -> f32 {
        // For each point z_i in trajectory:
        //   Find k nearest neighbors {z_j} in previous window
        //   Track separation: ||z_{i+t} - z_{j+t}|| / ||z_i - z_j||
        //   LLE ≈ (1/t) · mean(log ratio of separations)
        
        // This is the Wolf et al. (1985) method, adapted for high-d latent space
        let divergences = self.compute_divergences(trajectory);
        let lle = divergences.mean() / self.timestep;
        lle
    }
    
    /// Full spectrum — which directions are most unstable
    fn lle_spectrum(&self, trajectory: &[State]) -> Vec<f32> {
        // Along leading divergence direction:
        // Decompose into principal components of divergence
        // Each PC direction gets its own LE
        // Leading mode = largest LE = most dangerous direction
    }
}
```

**Implementation:** Use finite differences on the forward pass Jacobian approximated via perturbation:

```
λ_i ≈ (1/δ) · log(||δ · v_i + J(z)·δ|| / ||δ||)
```

Where `J(z)` is the Jacobian of the transition function, estimated via autograd or finite perturbation.

### 1b. RQA Coherence Score

```rust
impl RQAAnalyzer {
    /// Compute RQA features on latent trajectory under perturbation
    fn rqa_features(&self, trajectory: &[State], threshold: f32) -> RQAMetrics {
        // Build recurrence plot: R_{i,j} = 1 if ||z_i - z_j|| < threshold
        
        // %REC: fraction of recurrent points
        // %DET: fraction forming diagonal lines (determinism)
        // MAXLINE: longest diagonal (attractor strength)
        
        // Perturbation sensitivity:
        // ΔDET = DET(baseline_trajectory) - DET(perturbed_trajectory)
        // AFI_contribution = w4 · ΔDET
    }
}
```

### 1c. Composite I_local

```
I_local(t) = w_1 · λ_leading(t) + w_4 · ΔRQA(t)
```

Where λ_leading is the leading finite-time Lyapunov exponent and ΔRQA is the recurrence structure drop under perturbation.

---

## 2. Basin Uncertainty U_basin

### 2a. Uncertainty Exponent α

```rust
impl BasinAnalyzer {
    /// Estimate uncertainty exponent α in local neighborhood
    /// p_uncertain(ε) ∝ ε^α
    fn uncertainty_exponent(&self, center: &State, n_samples: usize) -> f32 {
        let mut uncertain_fractions = Vec::new();
        let epsilon_range = [0.01, 0.05, 0.1, 0.2, 0.5];
        
        for &ε in &epsilon_range {
            // Sample n_balls of radius ε around center
            let mut uncertain_count = 0;
            
            for _ in 0..n_samples {
                let ball = sample_ball(center, ε);
                let destinations = self.integrate_forward_batch(&ball, n_steps=50);
                
                // If balls converge to different attractors, mark uncertain
                if destinations.cover_multiple_attractors() {
                    uncertain_count += 1;
                }
            }
            
            let fraction = uncertain_count as f32 / n_samples as f32;
            uncertain_fractions.push((ε, fraction));
        }
        
        // Fit: log(p_uncertain) = α · log(ε) + const
        // α = slope of log-log fit
        // α ≈ 1: smooth boundary (robust)
        // α → 0: fractal boundary (fragile)
        self.fit_power_law(uncertain_fractions).alpha
    }
}
```

### 2b. Basin Entropy (alternative)

```rust
impl BasinAnalyzer {
    /// Basin entropy: S_B = -Σ p_i · log(p_i) over attractor basins
    /// Higher S_B = more uncertainty about final state = more fragile
    fn basin_entropy(&self, region: &StateRegion) -> f32 {
        // Sample many initial conditions in region
        // Integrate forward to classify asymptotic attractor
        // Count fraction p_i in each basin
        // S_B = -Σ p_i · log(p_i)
    }
}
```

### 2c. Attractor Identification

**Critical challenge:** Transformer reasoning modes are not fixed points — they're metastable regimes in activation space. We need to define "which attractor" operationally:

```rust
enum Attractor {
    SymbolicManipulation,
    EmotionalResonance,
    Exploratory,
    CausalReasoning,
    RetrievalRecall,
    Unknown,
}

impl AttractorClassifier {
    /// Classify which attractor regime a trajectory is in
    /// Uses a small probe network trained on regime-labeled trajectories
    fn classify(&self, trajectory: &[State]) -> Attractor {
        // Project trajectory to low-d manifold (PCA or learned)
        // Find nearest centroid in regime space
        // Return regime label
    }
}
```

**Open question:** How many reasoning mode attractors does Starfire actually have? This is empirically measurable via clustering of activation trajectories.

---

## 3. The Meta-Learner: F̂(I_local) ≈ U_basin

### 3a. Architecture

```rust
struct FragilitySurrogate {
    /// Small network: maps Lyapunov features → basin fragility estimate
    /// Trained to minimize (F̂(I_local) - U_basin)^2
    network: candle::Module,
    
    /// Re-calibration buffer: stores (I_local, U_basin) pairs for retraining
    calibration_buffer: Vec<(LyapunovFeatures, BasinScore)>,
}

impl FragilitySurrogate {
    /// Update surrogate using new basin measurement
    fn recalibrate(&mut self, i_local: &LyapunovFeatures, u_basin: f32) {
        self.calibration_buffer.push((i_local.clone(), u_basin));
        
        // Retrain on buffer when enough new samples accumulate
        if self.calibration_buffer.len() >= BATCH_SIZE {
            self.train_on_buffer();
        }
    }
    
    /// Fast fragility estimate from cheap local features
    fn estimate(&self, i_local: &LyapunovFeatures) -> f32 {
        self.network.forward(i_local)  // No backprop needed — just forward pass
    }
}
```

### 3b. Training Loop

```rust
impl FragilitySurrogate {
    fn train_on_buffer(&mut self) {
        // (I_local, U_basin) pairs
        // Loss: MSE(F̂(I_local), U_basin)
        // Optimizer: Adam, lr=1e-4
        // 
        // After training: fast fragility estimate F̂(I_local)
        // is now better calibrated against true basin uncertainty
    }
}
```

### 3c. The ASRU Update with Surrogate

```rust
impl ForethoughtEngine {
    /// Fast ASRU step — uses surrogate, no basin computation needed
    fn fast_step(&mut self, z_t: &State) -> MetaStateDelta {
        // 1. Compute local instability features
        let i_local = self.compute_i_local(z_t);
        
        // 2. Fast fragility estimate via surrogate
        let fragility_hat = self.surrogate.estimate(&i_local);
        
        // 3. Gradient of surrogate w.r.t M (plasticity params)
        //    ∂F̂/∂M = ∂F̂/∂I_local · ∂I_local/∂M
        let dF_dM = self.compute_surrogate_gradient(&i_local);
        
        // 4. ASRU fast update: decrease fragility along gradient
        let delta_M = -self.η_fast * dF_dM * fragility_hat;
        
        MetaStateDelta { delta_M }
    }
    
    /// Slow ASRU step — computes actual basin uncertainty, recalibrates surrogate
    fn slow_step(&mut self, z_t: &State) {
        // 1. Compute expensive basin uncertainty
        let u_basin = self.basin_analyzer.uncertainty_exponent(z_t, n_samples=1000);
        
        // 2. Compute I_local at same point
        let i_local = self.compute_i_local(z_t);
        
        // 3. Recalibrate surrogate: train F̂ to match U_basin
        self.surrogate.recalibrate(&i_local, u_basin);
        
        // 4. Slow update to base system parameters θ
        let dU_dθ = self.compute_basin_gradient_wrt_params();
        let delta_θ = -self.η_slow * dU_dθ * u_basin;
        
        // 5. Apply symmetry breaking: update column roles
        self.update_column_roles(u_basin);
    }
}
```

---

## 4. Connection to R(θ) — Risk Surface Sculpting

The full ASRU objective is:

```
minimize  R(θ, M) = E_{τ~p(future)}[divergence(z_0, τ) · risk_weight(τ)]
                   + γ · AFI(M, z_t)
                   
subject to: M in valid_routing_space
            θ in stable_param_region
```

Where:
- `divergence(z_0, τ)` = trajectory divergence under perturbation τ
- `risk_weight(τ)` = external risk (safety, irreversibility)
- `AFI(M, z_t)` = Attractor Fragility Index at current state
- `γ` = coupling strength between fragility and risk

**The fast loop minimizes AFI(M, z_t)** (local fragility)
**The slow loop minimizes R(θ, M)** (global risk surface)

---

## 5. Anticipatory Symmetry Breaking in ASRU

```rust
impl ForethoughtEngine {
    /// When slow loop detects high basin fragility,
    /// pre-break symmetry by assigning columns to roles
    fn update_column_roles(&mut self, u_basin: f32) {
        if u_basin > self.fragility_threshold {
            // System is near a fragile basin boundary
            // Pre-differentiate columns toward the anticipated regime
            
            let regime = self.regime_detector.predict_incoming_regime();
            
            match regime {
                Regime::SymbolicManipulation => {
                    // Assign columns to precision reasoning roles
                    for col in &mut self.columns[..N_CALCULATORS] {
                        col.assign_role(Role::Calculator, &mut self.assignment_field);
                    }
                    // Pre-commit shock absorbers
                    for col in &mut self.columns[N_CALCULATORS..] {
                        col.assign_role(Role::ShockAbsorber, &mut self.assignment_field);
                    }
                }
                Regime::EmotionalResonance => {
                    // Assign to empathy processing
                    for col in &mut self.columns {
                        col.assign_role(Role::EmotionalResonance, &mut self.assignment_field);
                    }
                }
                _ => { /* no pre-differentiation needed */ }
            }
            
            // Freeze assignment field (low viscosity = stable role)
            self.assignment_field.viscosity = 0.1;
        } else {
            // System is in robust region
            // Keep columns in exploratory/stem state (high plasticity)
            for col in &mut self.columns {
                col.assign_role(Role::Stem, &mut self.assignment_field);
            }
            self.assignment_field.viscosity = 0.9;  // high = flexible
        }
    }
}
```

---

## 6. Two-Timescale Summary

| Loop | Trigger | Signal | Updates | Frequency |
|------|---------|--------|---------|-----------|
| Fast | Every step | I_local via F̂ | M_t (routing, plasticity) | O(1) |
| Slow | Compute budget available | U_basin | θ (base params) + column roles | Episodic |

**Fast loop:** continuous, ~1ms latency, gradient-based update to M_t
**Slow loop:** triggered when compute budget allows, computes true U_basin, recalibrates F̂, updates θ

**The meta-learner is the bridge:** F̂ gets better over time as slow loop provides more (I_local, U_basin) calibration pairs. Eventually, fast loop is a reliable proxy for basin fragility most of the time.

---

## Open Questions

- [ ] How many distinct reasoning mode attractors does Starfire have? Empirically measurable via activation clustering.
- [ ] Are basin boundaries between reasoning modes fractal or smooth? Affects α significantly.
- [ ] What's the right dimensionality reduction for the latent manifold used in basin analysis? PCA vs learned embedding.
- [ ] How often should slow loop run to keep F̂ calibrated? Risk of drift vs compute cost.
- [ ] Can we backprop through the slow loop objective into θ efficiently, or is it episodic RL-style?
- [ ] **METASTABLE STATES**: "Attractor" = metastable reasoning mode with long dwell times, NOT asymptotic fixed point. The attractor-like objects are regions of state space where trajectories spend disproportionate time and escape is rare on task-relevant timescales.
- [ ] **Attractor identification**: Use HMM / spectral clustering on activation trajectories to discover metastable modes. Each "reasoning mode" (symbolic / emotional / causal / exploratory) is a metastable set with well-defined dwell time distribution and transition statistics between them.
- [ ] **Basin in metastable sense**: Sets of states that, with high probability, fall into a given metastable mode within some horizon before escaping to another.
- [ ] **AFI for reasoning modes**: Measures how easily the system can be knocked out of its current metastable reasoning mode into a different one — exactly the right fragility definition for transformers.
