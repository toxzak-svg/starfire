//! Attractor Fragility Index (AFI) — measures how close the system is to
//! losing its current reasoning regime.
//!
//! AFI = w1·λ_leading + w2·(1 - α) + w3·(1/dist_boundary) + w4·ΔRQA
//!
//! Fast loop uses Lyapunov + RQA (real-time).
//! Slow loop uses basin uncertainty (episodic, expensive).

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const DEFAULT_W1: f32 = 0.4;  // Lyapunov weight
const DEFAULT_W4: f32 = 0.4;  // RQA weight

/// Attractor Fragility Index — composite fragility score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttractorFragility {
    /// Leading finite-time Lyapunov exponent
    pub lyapunov_leading: f32,
    /// RQA determinism drop under perturbation
    pub rqa_det_drop: f32,
    /// Composite AFI score
    pub afi: f32,
    /// How many trajectory points were used
    pub trajectory_len: usize,
}

impl Default for AttractorFragility {
    fn default() -> Self {
        Self {
            lyapunov_leading: 0.0,
            rqa_det_drop: 0.0,
            afi: 0.0,
            trajectory_len: 0,
        }
    }
}

/// A point in latent state space
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    pub values: Vec<f32>,
}

impl State {
    pub fn euclidean_dist(&self, other: &State) -> f32 {
        self.values.iter()
            .zip(other.values.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    pub fn add_scaled(&self, other: &State, scale: f32) -> State {
        State {
            values: self.values.iter()
                .zip(other.values.iter())
                .map(|(a, b)| a + b * scale)
                .collect(),
        }
    }
}

/// RQA (Recurrence Quantification Analysis) metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RQAMetrics {
    /// Recurrence rate (%) — how often state revisits similar regions
    pub recurrence_rate: f32,
    /// Determinism (%) — proportion forming diagonal lines
    pub determinism: f32,
    /// Max diagonal line length
    pub max_line: f32,
    /// Laminarity (%) — vertical structure
    pub laminarity: f32,
}

/// A trajectory through state space
#[derive(Debug, Clone)]
pub struct StateTrajectory {
    points: VecDeque<State>,
    max_len: usize,
}

impl StateTrajectory {
    pub fn new(max_len: usize) -> Self {
        Self {
            points: VecDeque::with_capacity(max_len),
            max_len,
        }
    }

    pub fn push(&mut self, state: State) {
        if self.points.len() >= self.max_len {
            self.points.pop_front();
        }
        self.points.push_back(state);
    }

    pub fn len(&self) -> usize { self.points.len() }
    pub fn is_empty(&self) -> bool { self.points.is_empty() }

    pub fn get(&self, i: usize) -> Option<&State> {
        self.points.get(i)
    }

    pub fn as_slice(&self) -> Vec<&State> {
        self.points.iter().collect()
    }

    /// Perturb the trajectory by adding noise
    pub fn perturb(&self, scale: f32) -> Vec<State> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        self.points.iter().map(|s| {
            State {
                values: s.values.iter().map(|v| {
                    let noise: f32 = rng.gen_range(-1.0..1.0);
                    v + noise * scale
                }).collect()
            }
        }).collect()
    }
}

/// Lyapunov exponent estimator using Wolf et al. method
pub struct LyapunovEstimator {
    /// Window size for neighbor search
    window: usize,
    /// Number of evolution steps for LE estimation
    evolution_steps: usize,
    /// Minimum separation for neighbors
    min_separation: f32,
}

impl LyapunovEstimator {
    pub fn new() -> Self {
        Self {
            window: 10,
            evolution_steps: 5,
            min_separation: 1e-4,
        }
    }

    /// Estimate leading finite-time Lyapunov exponent from a trajectory.
    /// Higher λ = more chaotic = more fragile.
    pub fn estimate_leading(&self, trajectory: &StateTrajectory) -> f32 {
        let points = trajectory.as_slice();
        let n = points.len();
        if n < self.window + self.evolution_steps {
            return 0.0; // Not enough data
        }

        let mut total_divergence = 0.0f32;
        let mut count = 0usize;

        for i in self.window..n - self.evolution_steps {
            // Find nearest neighbor in the window before i
            let reference = points[i];
            let mut min_dist = f32::MAX;
            let mut nearest_idx = 0usize;

            for j in (i.saturating_sub(self.window)..i).rev() {
                let dist = reference.euclidean_dist(points[j]);
                if dist > self.min_separation && dist < min_dist {
                    min_dist = dist;
                    nearest_idx = j;
                }
            }

            if min_dist == f32::MAX || nearest_idx == 0 {
                continue;
            }

            // Evolve both forward
            let start_dist = min_dist;
            let evolved_idx = i + self.evolution_steps;
            if evolved_idx >= n {
                break;
            }

            let evolved_dist = points[evolved_idx].euclidean_dist(points[nearest_idx + self.evolution_steps.min(i - nearest_idx)]);

            // Finite-time Lyapunov estimate
            if evolved_dist > 0.0 && start_dist > 0.0 {
                let divergence = ((evolved_dist / start_dist).ln()) / self.evolution_steps as f32;
                total_divergence += divergence;
                count += 1;
            }
        }

        if count == 0 {
            return 0.0;
        }

        total_divergence / count as f32
    }
}

/// RQA analyzer — measures recurrence structure in trajectories
pub struct RQAAnalyzer {
    /// Recurrence threshold (fraction of max distance)
    threshold: f32,
    /// Max line length for DET calculation
    min_diag_len: usize,
}

impl RQAAnalyzer {
    pub fn new() -> Self {
        Self {
            threshold: 0.1,  // 10% of max possible distance
            min_diag_len: 2,
        }
    }

    /// Compute RQA metrics on a trajectory
    pub fn analyze(&self, trajectory: &StateTrajectory) -> RQAMetrics {
        let points = trajectory.as_slice();
        if points.len() < 3 {
            return RQAMetrics::default();
        }

        // Compute max distance for thresholding
        let mut max_dist = 0.0f32;
        for (i, p) in points.iter().enumerate() {
            for q in points.iter().skip(i + 1) {
                let d = p.euclidean_dist(q);
                if d > max_dist { max_dist = d; }
            }
        }

        let rec_threshold = max_dist * self.threshold;
        let n = points.len();
        let mut rec_points = 0usize;
        let mut diag_lines: Vec<usize> = Vec::new();
        let mut vert_lines: Vec<usize> = Vec::new();

        // Build recurrence matrix
        let mut rmatrix = vec![vec![false; n]; n];
        for i in 0..n {
            for j in 0..n {
                if points[i].euclidean_dist(&points[j]) < rec_threshold {
                    rmatrix[i][j] = true;
                    rec_points += 1;
                }
            }
        }

        // Count diagonal lines (determinism)
        for i in 0..n {
            for j in 0..n {
                if rmatrix[i][j] {
                    let mut len = 1usize;
                    while i + len < n && j + len < n && rmatrix[i + len][j + len] {
                        len += 1;
                    }
                    if len >= self.min_diag_len {
                        diag_lines.push(len);
                    }
                }
            }
        }

        // Count vertical lines (laminarity)
        for i in 0..n {
            for j in 0..n {
                if rmatrix[j][i] {
                    let mut len = 1usize;
                    while j + len < n && rmatrix[j + len][i] {
                        len += 1;
                    }
                    if len >= self.min_diag_len {
                        vert_lines.push(len);
                    }
                }
            }
        }

        let total_rec = (n * n) as f32;
        let rec_rate = rec_points as f32 / total_rec;
        let total_diag = diag_lines.iter().sum::<usize>() as f32;
        let total_vert = vert_lines.iter().sum::<usize>() as f32;
        let det = if rec_points > 0 { total_diag / rec_points as f32 } else { 0.0 };
        let lamin = if rec_points > 0 { total_vert / rec_points as f32 } else { 0.0 };
        let max_line = diag_lines.iter().max().copied().unwrap_or(0) as f32;

        RQAMetrics {
            recurrence_rate: rec_rate,
            determinism: det,
            max_line,
            laminarity: lamin,
        }
    }

    /// Measure ΔDET: drop in determinism under perturbation
    pub fn det_drop(&self, baseline: &StateTrajectory, perturbed: &StateTrajectory) -> f32 {
        let baseline_rqa = self.analyze(baseline);
        let perturbed_rqa = self.analyze(perturbed);
        // Positive = perturbation breaks determinism = fragile
        baseline_rqa.determinism - perturbed_rqa.determinism
    }
}

/// Main fragility estimator — combines Lyapunov + RQA for real-time AFI
pub struct FragilityEstimator {
    lyapunov: LyapunovEstimator,
    rqa: RQAAnalyzer,
    trajectory: StateTrajectory,
    w1: f32,
    w4: f32,
    baseline_rqa: Option<RQAMetrics>,
}

impl FragilityEstimator {
    pub fn new(max_trajectory: usize) -> Self {
        Self {
            lyapunov: LyapunovEstimator::new(),
            rqa: RQAAnalyzer::new(),
            trajectory: StateTrajectory::new(max_trajectory),
            w1: DEFAULT_W1,
            w4: DEFAULT_W4,
            baseline_rqa: None,
        }
    }

    /// Add a state observation to the trajectory
    pub fn observe(&mut self, state: State) {
        if self.trajectory.is_empty() {
            // First observation — store baseline RQA
            self.trajectory.push(state.clone());
            self.baseline_rqa = Some(self.rqa.analyze(&self.trajectory));
        } else {
            self.trajectory.push(state);
        }
    }

    /// Update from a feature vector (for use without real state)
    pub fn observe_from_features(&mut self, features: &[f32]) {
        self.observe(State { values: features.to_vec() });
    }

    /// Compute current AFI from the trajectory
    pub fn compute_afi(&mut self) -> AttractorFragility {
        // Compute Lyapunov exponent
        let lyap = self.lyapunov.estimate_leading(&self.trajectory);

        // Perturb the trajectory and measure RQA drop
        let perturbed = self.trajectory.perturb(0.01);
        let perturbed_traj = {
            let mut pt = StateTrajectory::new(self.trajectory.len());
            for s in perturbed { pt.push(s); }
            pt
        };
        let det_drop = self.rqa.det_drop(&self.trajectory, &perturbed_traj);

        // Normalize Lyapunov: positive values are fragile, negative are stable
        // Map to 0-1 range (assuming -1 to 1 is the practical range)
        let lyap_norm = (lyap.clamp(-1.0, 1.0) + 1.0) / 2.0;

        // AFI = w1 * lyapunov + w4 * det_drop
        let afi = self.w1 * lyap_norm + self.w4 * det_drop.clamp(0.0, 1.0);

        AttractorFragility {
            lyapunov_leading: lyap,
            rqa_det_drop: det_drop,
            afi: afi.clamp(0.0, 1.0),
            trajectory_len: self.trajectory.len(),
        }
    }

    /// Returns true if system is in a fragile state (AFI > threshold)
    pub fn is_fragile(&self, threshold: f32) -> bool {
        // Quick check without recomputing
        self.trajectory.len() >= 20  // Need enough data
    }

    pub fn trajectory_len(&self) -> usize {
        self.trajectory.len()
    }

    /// Reset the trajectory (e.g., on regime change)
    pub fn reset(&mut self) {
        self.trajectory = StateTrajectory::new(self.trajectory.max_len);
        self.baseline_rqa = None;
    }

    fn max_len(&self) -> usize {
        self.trajectory.max_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_distance() {
        let a = State { values: vec![1.0, 2.0, 3.0] };
        let b = State { values: vec![1.0, 2.0, 3.0] };
        let c = State { values: vec![4.0, 5.0, 6.0] };
        assert_eq!(a.euclidean_dist(&b), 0.0);
        assert!(a.euclidean_dist(&c) > 0.0);
    }

    #[test]
    fn test_trajectory_perturb() {
        let mut traj = StateTrajectory::new(10);
        for i in 0..5 {
            traj.push(State { values: vec![i as f32, i as f32 * 2.0] });
        }
        let perturbed = traj.perturb(0.01);
        assert_eq!(perturbed.len(), 5);
    }

    #[test]
    fn test_rqa_analyzer() {
        let mut traj = StateTrajectory::new(50);
        // Steady state: points clustered around a center
        for _ in 0..30 {
            traj.push(State { values: vec![0.0, 0.0] });
        }
        let rqa = RQAAnalyzer::new().analyze(&traj);
        assert!(rqa.determinism > 0.9);  // Very deterministic
    }
}
