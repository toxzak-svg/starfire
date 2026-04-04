//! Reservoir — Echo State Network with Chaotic Modulation
//!
//! Ported from Python `reservoir.py`
//!
//! The reservoir maintains nonlinear temporal memory through recurrent
//! connections. Training is performed only on output weights (linear readout).
//!
//! Key parameter: spectral radius (ρ)
//! - ρ < 1: stable, fading memory
//! - ρ ≈ 1: edge of chaos, maximal computational power
//! - ρ > 1: chaotic, long-term memory but unstable

use rand::Rng;
use rand_distr::{Normal, Distribution};

/// Echo State Network reservoir with chaotic dynamics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Reservoir {
    /// Input dimension
    pub input_dim: usize,
    /// Number of reservoir neurons
    pub reservoir_size: usize,
    /// Spectral radius (controls chaos level)
    pub spectral_radius: f64,
    /// Input weight matrix
    w_in: Vec<f64>,
    /// Reservoir weight matrix (sparse)
    w: Vec<f64>,
    /// Output weights (trained via ridge regression)
    w_out: Option<Vec<f64>>,
    /// Current state
    state: Vec<f64>,
    /// Leak rate (α)
    leak_rate: f64,
    /// Noise level for chaotic modulation
    noise_level: f64,
    /// Connectivity (sparsity)
    connectivity: f64,
    /// Input scaling
    input_scaling: f64,
}

impl Reservoir {
    /// Create a new reservoir
    pub fn new(
        input_dim: usize,
        reservoir_size: usize,
    ) -> Self {
        let spectral_radius = 0.95;
        let input_scaling = 0.1;
        let connectivity = 0.01;
        let noise_level = 0.001;
        let leak_rate = 0.3;

        let mut rng = rand::thread_rng();

        // Initialize input weights: random, scaled
        let normal = Normal::new(0.0, 1.0).unwrap();
        let mut w_in = Vec::with_capacity(reservoir_size * input_dim);
        for _ in 0..(reservoir_size * input_dim) {
            w_in.push(normal.sample(&mut rng) * input_scaling);
        }

        // Initialize reservoir weights: random, sparse
        let mut w_full = vec![0.0; reservoir_size * reservoir_size];
        for i in 0..reservoir_size {
            for j in 0..reservoir_size {
                let idx = i * reservoir_size + j;
                if rng.gen::<f64>() < connectivity {
                    w_full[idx] = normal.sample(&mut rng);
                }
            }
        }

        // Scale to desired spectral radius
        let scale = spectral_radius_estimate(&w_full, reservoir_size);
        for w in &mut w_full {
            *w *= scale;
        }

        Self {
            input_dim,
            reservoir_size,
            spectral_radius,
            w_in,
            w: w_full,
            w_out: None,
            state: vec![0.0; reservoir_size],
            leak_rate,
            noise_level,
            connectivity,
            input_scaling,
        }
    }

    /// Single step forward pass
    pub fn step(&mut self, input_vec: &[f64]) -> Vec<f64> {
        assert_eq!(input_vec.len(), self.input_dim);

        let mut pre_activation = 0.0f64;

        // w_in @ input
        for i in 0..self.reservoir_size {
            let mut sum = 0.0;
            for j in 0..self.input_dim {
                sum += self.w_in[i * self.input_dim + j] * input_vec[j];
            }
            pre_activation += sum;
        }

        // W @ state
        for i in 0..self.reservoir_size {
            let mut sum = 0.0;
            let base = i * self.reservoir_size;
            for j in 0..self.reservoir_size {
                sum += self.w[base + j] * self.state[j];
            }
            pre_activation += sum;
        }

        // Chaos term
        let mut rng = rand::thread_rng();
        let _chaos: f64 = (0..self.reservoir_size)
            .map(|i| {
                let noise: f64 = standard_normal_sample(&mut rng);
                self.noise_level * self.state[i].tanh() * noise.abs()
            })
            .sum();

        // Apply activation with leak
        let activated = pre_activation.tanh();
        for i in 0..self.reservoir_size {
            self.state[i] = (1.0 - self.leak_rate) * self.state[i] + self.leak_rate * activated;
        }

        self.state.clone()
    }

    /// Process an input sequence
    pub fn forward(&mut self, input_sequence: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let mut states = Vec::with_capacity(input_sequence.len());
        for input_vec in input_sequence {
            states.push(self.step(input_vec));
        }
        states
    }

    /// Reset reservoir state
    pub fn reset(&mut self) {
        self.state.fill(0.0);
    }

    /// Get current state
    pub fn get_state(&self) -> &[f64] {
        &self.state
    }

    /// Train linear readout using ridge regression
    pub fn train_readout(
        &mut self,
        states: &[Vec<f64>],
        targets: &[Vec<f64>],
        regularization: f64,
    ) -> f64 {
        let n_samples = states.len();
        if n_samples == 0 {
            return f64::INFINITY;
        }

        let output_dim = targets[0].len();
        let state_dim = self.reservoir_size;

        // Build x matrix: [states | bias]
        // x^T @ x + λI
        let mut xt_x = vec![0.0; (state_dim + 1) * (state_dim + 1)];
        for s in states {
            for i in 0..state_dim {
                for j in 0..state_dim {
                    xt_x[i * (state_dim + 1) + j] += s[i] * s[j];
                }
            }
        }
        // Add regularization
        for i in 0..(state_dim + 1) {
            xt_x[i * (state_dim + 1) + i] += regularization;
        }

        // x^T @ Y
        let mut xt_y = vec![0.0; (state_dim + 1) * output_dim];
        for (si, t) in states.iter().zip(targets.iter()) {
            for i in 0..state_dim {
                for j in 0..output_dim {
                    xt_y[i * output_dim + j] += si[i] * t[j];
                }
            }
            // Bias term
            for j in 0..output_dim {
                xt_y[state_dim * output_dim + j] += t[j];
            }
        }

        // Solve: W_out = (xt_x)^{-1} @ xt_y
        // Using pseudo-inverse approximation (for small systems)
        let _w_out = solve_linear_system(&xt_x, &xt_y, state_dim + 1, output_dim);

        // Compute predictions and RMSE
        let mut total_error = 0.0;
        let mut w_out_flat = Vec::with_capacity((state_dim + 1) * output_dim);

        for j in 0..output_dim {
            for i in 0..(state_dim + 1) {
                w_out_flat.push(xt_y[i * output_dim + j]);
            }
        }

        self.w_out = Some(w_out_flat);

        for (s, t) in states.iter().zip(targets.iter()) {
            let pred = self.predict_single(s);
            for (p, tgt) in pred.iter().zip(t.iter()) {
                total_error += (p - tgt).powi(2);
            }
        }

        (total_error / (n_samples * output_dim) as f64).sqrt()
    }

    /// Predict from a single state
    fn predict_single(&self, state: &[f64]) -> Vec<f64> {
        if let Some(ref w_out) = self.w_out {
            let output_dim = w_out.len() / (self.reservoir_size + 1);
            let mut pred = vec![0.0; output_dim];

            for j in 0..output_dim {
                // State contribution
                for i in 0..self.reservoir_size {
                    pred[j] += w_out[i * output_dim + j] * state[i];
                }
                // Bias
                pred[j] += w_out[self.reservoir_size * output_dim + j];
            }
            pred
        } else {
            vec![]
        }
    }

    /// Predict from reservoir states
    pub fn predict(&self, states: &[Vec<f64>]) -> Vec<Vec<f64>> {
        states.iter().map(|s| self.predict_single(s)).collect()
    }
}

/// Generate a standard normal random sample using Box-Muller
fn standard_normal_sample<R: Rng>(rng: &mut R) -> f64 {
    // Box-Muller transform
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

/// Estimate spectral radius (largest eigenvalue magnitude)
fn spectral_radius_estimate(w: &[f64], n: usize) -> f64 {
    // Power iteration
    let mut v = vec![1.0 / (n as f64).sqrt(); n];
    let mut lambda = 0.0;

    for _ in 0..100 {
        let mut av = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                av[i] += w[i * n + j] * v[j];
            }
        }

        let norm = av.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-10);
        lambda = norm;

        for i in 0..n {
            v[i] = av[i] / norm;
        }
    }

    if lambda > 1e-10 {
        1.0 / lambda
    } else {
        1.0
    }
}

/// Simple linear system solver (using eigen-decomposition for small systems)
fn solve_linear_system(a: &[f64], b: &[f64], n: usize, m: usize) -> Vec<f64> {
    // Simplified: use pseudo-inverse approximation
    // For small n (like reservoir_size=1000), we use (A + λI)^{-1} ≈ I/A_ii approximation
    // A is diagonal-dominant, so we use Jacobi-like iteration

    let mut x = vec![0.0; n * m];
    let lambda = 1e-6;

    for _iter in 0..50 {
        let mut x_new = x.clone();

        for i in 0..n {
            let diag = a[i * n + i].max(lambda);
            for j in 0..m {
                let mut sum = b[i * m + j];
                for k in 0..n {
                    if k != i {
                        sum -= a[i * n + k] * x[k * m + j];
                    }
                }
                x_new[i * m + j] = sum / diag;
            }
        }

        // Check convergence
        let max_diff = x.iter()
            .zip(x_new.iter())
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f64, |a, b| a.max(b));

        x = x_new;

        if max_diff < 1e-8 {
            break;
        }
    }

    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reservoir_step() {
        let mut reservoir = Reservoir::new(10, 50);
        let input = vec![0.5; 10];
        let state = reservoir.step(&input);
        assert_eq!(state.len(), 50);
    }

    #[test]
    fn test_reservoir_forward() {
        let mut reservoir = Reservoir::new(5, 30);
        let sequence = vec![vec![0.1; 5]; 10];
        let states = reservoir.forward(&sequence);
        assert_eq!(states.len(), 10);
        assert_eq!(states[0].len(), 30);
    }

    #[test]
    fn test_reservoir_reset() {
        let mut reservoir = Reservoir::new(8, 40);
        reservoir.step(&vec![1.0; 8]);
        reservoir.reset();
        assert!(reservoir.state.iter().all(|&s| s == 0.0));
    }
}
