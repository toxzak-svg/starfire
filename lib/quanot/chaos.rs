//! Chaos — Chaos metrics: Lyapunov exponents, RQA, attractors
//!
//! Ported from Python `chaos.py`

use serde::{Deserialize, Serialize};

/// Chaos metrics computed from trajectory
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChaosMetrics {
    /// Maximal Lyapunov exponent (positive = chaotic)
    pub lyapunov_exponent: f64,
    /// Correlation dimension
    pub correlation_dimension: f64,
    /// Entropy of the trajectory
    pub entropy: f64,
    /// REC (recurrence rate) from RQA
    pub recurrence: f64,
    /// DET (determinism) from RQA
    pub determinism: f64,
    /// RQA laminarity
    pub laminarity: f64,
}

impl ChaosMetrics {
    /// Compute metrics from a trajectory
    pub fn from_trajectory(trajectory: &[Vec<f64>]) -> Self {
        if trajectory.len() < 20 {
            return Self::default();
        }

        // Project to 1D for simplicity (use first component)
        let series: Vec<f64> = trajectory.iter().map(|v| v[0]).collect();

        let lyapunov = maximal_lyapunov_estimate(&series);
        let corr_dim = correlation_dimension_estimate(&series);
        let entropy = entropy_estimate(&series);
        let rqa = rqa_metrics(&series);

        Self {
            lyapunov_exponent: lyapunov,
            correlation_dimension: corr_dim,
            entropy,
            recurrence: rqa.recurrence,
            determinism: rqa.determinism,
            laminarity: rqa.laminarity,
        }
    }

    /// Get regime description
    pub fn regime(&self) -> &'static str {
        if self.lyapunov_exponent < -0.1 {
            "stable"
        } else if self.lyapunov_exponent > 0.1 {
            "chaotic"
        } else {
            "edge_of_chaos"
        }
    }
}

/// RQA results
#[derive(Debug, Clone, Default)]
pub struct RQAResults {
    pub recurrence: f64,
    pub determinism: f64,
    pub laminarity: f64,
    pub trapping_time: f64,
    pub max_line_length: usize,
}

/// Compute RQA metrics from a 1D series
fn rqa_metrics(series: &[f64]) -> RQAResults {
    let n = series.len();
    if n < 20 {
        return RQAResults::default();
    }

    // Threshold for recurrence
    let threshold = compute_threshold(series);
    let threshold = threshold * 0.1; // Use 10th percentile

    // Build recurrence matrix
    let mut rec_matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            let dist = (series[i] - series[j]).abs();
            if dist < threshold {
                rec_matrix[i][j] = 1.0;
            }
        }
    }

    // Compute RQA metrics
    let mut n_recurrent = 0;
    let mut diagonal_sum = 0;
    let mut vertical_sum = 0;
    let mut diag_lengths = Vec::new();
    let mut vert_lengths = Vec::new();

    // Count recurrent points (excluding main diagonal)
    for i in 0..n {
        for j in (i + 1)..n {
            if rec_matrix[i][j] > 0.5 {
                n_recurrent += 2; // Count symmetric

                // Trace diagonal lines
                let mut len = 1;
                let mut k = 1;
                while i + k < n && j + k < n && rec_matrix[i + k][j + k] > 0.5 {
                    len += 1;
                    k += 1;
                }
                if len > 1 {
                    diagonal_sum += len;
                    diag_lengths.push(len);
                }

                // Trace vertical lines
                let mut vlen = 1;
                let mut k = 1;
                while i + k < n && rec_matrix[i + k][j] > 0.5 {
                    vlen += 1;
                    k += 1;
                }
                if vlen > 1 {
                    vertical_sum += vlen;
                    vert_lengths.push(vlen);
                }
            }
        }
    }

    let total_pairs = (n * (n - 1)) as f64;
    let recurrence = (n_recurrent as f64 / total_pairs).min(1.0);
    let determinism = if diagonal_sum > 0 { diagonal_sum as f64 / n_recurrent as f64 } else { 0.0 };
    let laminarity = if vertical_sum > 0 { vertical_sum as f64 / n_recurrent as f64 } else { 0.0 };
    let trapping_time = if vert_lengths.is_empty() { 0.0 } else { vertical_sum as f64 / vert_lengths.len() as f64 };
    let max_line_length = diag_lengths.iter().cloned().max().unwrap_or(0);

    RQAResults {
        recurrence,
        determinism,
        laminarity,
        trapping_time,
        max_line_length,
    }
}

/// Estimate threshold for recurrence
fn compute_threshold(series: &[f64]) -> f64 {
    // Standard deviation as threshold
    let mean = series.iter().sum::<f64>() / series.len() as f64;
    let variance = series.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / series.len() as f64;
    variance.sqrt()
}

/// Estimate maximal Lyapunov exponent via divergence method
fn maximal_lyapunov_estimate(series: &[f64]) -> f64 {
    let n = series.len();
    if n < 100 {
        return 0.0;
    }

    // Simplified: compute divergence from neighboring points
    let window = 50.min(n / 2);
    let mut divergences = Vec::with_capacity(window);

    for i in window..n {
        // Find nearest previous point
        let mut min_dist = f64::INFINITY;
        for j in (i - window)..i {
            let dist = (series[i] - series[j]).abs();
            if dist > 1e-8 && dist < min_dist {
                min_dist = dist;
            }
        }

        if min_dist < f64::INFINITY {
            divergences.push((i, min_dist));
        }
    }

    if divergences.len() < 10 {
        return 0.0;
    }

    // Linear fit: log(divergence) vs index
    // Use first half for slope estimation
    let half = divergences.len() / 2;
    let (slope, _) = linear_fit(
        &divergences[..half].iter().map(|(i, _)| *i as f64).collect::<Vec<_>>(),
        &divergences[..half].iter().map(|(_, d)| (*d).max(1e-10).ln()).collect::<Vec<_>>(),
    );

    // Slope is the Lyapunov exponent estimate
    slope.max(-5.0).min(5.0)
}

/// Simple linear regression
fn linear_fit(x: &[f64], y: &[f64]) -> (f64, f64) {
    let n = x.len().min(y.len()) as f64;
    if n < 2.0 {
        return (0.0, 0.0);
    }

    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    let sum_x2: f64 = x.iter().map(|a| a * a).sum();

    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-10 {
        return (0.0, sum_y / n);
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;

    (slope, intercept)
}

/// Estimate correlation dimension (simplified Grassberger-Procaccia)
fn correlation_dimension_estimate(series: &[f64]) -> f64 {
    let n = series.len();
    if n < 20 {
        return 0.0;
    }

    // Use embedding dimension of 2
    let emb_dim = 2;
    let m = n - emb_dim + 1;
    if m < 10 {
        return 0.0;
    }

    // Build embedding vectors
    let emb: Vec<_> = (0..m)
        .map(|i| vec![series[i], series[i + 1]])
        .collect();

    // Compute pairwise distances
    let mut all_dists: Vec<f64> = Vec::with_capacity(m * m);
    for i in 0..m {
        for j in 0..m {
            if i != j {
                let d = (0..emb_dim)
                    .map(|k| (emb[i][k] - emb[j][k]).powi(2))
                    .sum::<f64>()
                    .sqrt();
                all_dists.push(d);
            }
        }
    }

    all_dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n_dists = all_dists.len();

    // Use 10% to 80% of distances for scaling region
    let start = (n_dists as f64 * 0.1) as usize;
    let end = (n_dists as f64 * 0.8) as usize;

    if end - start < 3 {
        return 0.0;
    }

    // Log-log slope = correlation dimension
    let radii: Vec<_> = all_dists[start..end].to_vec();
    let counts: Vec<_> = (start..end).map(|i| (i + 1) as f64).collect();

    let (slope, _) = linear_fit(
        &radii.iter().map(|r| r.max(1e-10).ln()).collect::<Vec<_>>(),
        &counts.iter().map(|c| c.max(1.0).ln()).collect::<Vec<_>>(),
    );

    slope.max(0.0).min(10.0)
}

/// Estimate entropy of a series (simplified)
fn entropy_estimate(series: &[f64]) -> f64 {
    // Discretize into bins
    let n_bins = 20.min(series.len() / 5);
    if n_bins < 5 {
        return 0.0;
    }

    let min_val = series.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = series.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max_val - min_val).max(1e-10);

    let mut counts = vec![0usize; n_bins];
    for v in series {
        let bin = ((*v - min_val) / range * n_bins as f64) as usize;
        let bin = bin.min(n_bins - 1);
        counts[bin] += 1;
    }

    // Shannon entropy
    let n = series.len() as f64;
    let mut entropy = 0.0f64;
    for &c in &counts {
        if c > 0 {
            let p = c as f64 / n;
            entropy -= p * p.ln();
        }
    }

    entropy
}

/// Generate Lorenz attractor trajectory
pub fn lorenz_attractor(
    steps: usize,
    dt: f64,
    sigma: f64,
    rho: f64,
    beta: f64,
) -> Vec<Vec<f64>> {
    let mut trajectory = Vec::with_capacity(steps);
    let mut x = 0.1;
    let mut y = 0.0;
    let mut z = 0.0;

    for _ in 0..steps {
        let dx = sigma * (y - x) * dt;
        let dy = (x * (rho - z) - y) * dt;
        let dz = (x * y - beta * z) * dt;
        x += dx;
        y += dy;
        z += dz;
        trajectory.push(vec![x, y, z]);
    }

    trajectory
}

/// Generate Rössler attractor trajectory
pub fn rossler_attractor(
    steps: usize,
    dt: f64,
    a: f64,
    b: f64,
    c: f64,
) -> Vec<Vec<f64>> {
    let mut trajectory = Vec::with_capacity(steps);
    let mut x = 0.1;
    let mut y = 0.1;
    let mut z = 0.1;

    for _ in 0..steps {
        let dx = (-y - z) * dt;
        let dy = (x + a * y) * dt;
        let dz = (b + z * (x - c)) * dt;
        x += dx;
        y += dy;
        z += dz;
        trajectory.push(vec![x, y, z]);
    }

    trajectory
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lorenz_attractor() {
        let traj = lorenz_attractor(1000, 0.01, 10.0, 28.0, 8.0 / 3.0);
        assert_eq!(traj.len(), 1000);
        assert_eq!(traj[0].len(), 3);
    }

    #[test]
    fn test_rossler_attractor() {
        let traj = rossler_attractor(1000, 0.01, 0.2, 0.2, 5.7);
        assert_eq!(traj.len(), 1000);
    }

    #[test]
    fn test_chaos_metrics() {
        let traj = lorenz_attractor(500, 0.01, 10.0, 28.0, 8.0 / 3.0);
        let metrics = ChaosMetrics::from_trajectory(&traj);
        assert!(metrics.lyapunov_exponent > 0.0); // Lorenz is chaotic
    }
}
