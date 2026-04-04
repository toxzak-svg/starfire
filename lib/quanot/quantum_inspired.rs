//! Quantum-Inspired Algorithms — Simulated Quantum Annealing (SQA)
//!
//! Ported from Python `quantum_inspired.py` / `sqa.py`
//!
//! Simulates quantum tunneling effects using Trotter decomposition.
//! Used for solving Ising/QUBO optimization problems.

use rand::Rng;

/// Simulated Quantum Annealing solver
#[derive(Debug, Clone)]
pub struct SimulatedQuantumAnnealing {
    /// Number of spin variables
    n_spins: usize,
    /// Number of Trotter replicas
    n_trotters: usize,
    /// Number of annealing steps
    n_steps: usize,
    /// Initial temperature
    t_init: f64,
    /// Final temperature
    t_final: f64,
    /// Initial transverse field
    gamma_init: f64,
    /// Current path configuration: [trotter][spin]
    path: Vec<Vec<i8>>,
    /// Best energy found
    best_energy: f64,
    /// Best spin configuration
    best_spins: Option<Vec<i8>>,
    /// Energy history
    energy_history: Vec<f64>,
}

impl Default for SimulatedQuantumAnnealing {
    fn default() -> Self {
        Self::new(100, 10, 5000)
    }
}

impl SimulatedQuantumAnnealing {
    /// Create a new SQA solver
    pub fn new(n_spins: usize, n_trotters: usize, n_steps: usize) -> Self {
        let mut rng = rand::thread_rng();

        let path: Vec<Vec<i8>> = (0..n_trotters)
            .map(|_| {
                (0..n_spins)
                    .map(|_| if rng.gen_bool(0.5) { 1 } else { -1 })
                    .collect()
            })
            .collect();

        Self {
            n_spins,
            n_trotters,
            n_steps,
            t_init: 5.0,
            t_final: 0.01,
            gamma_init: 2.0,
            path,
            best_energy: f64::INFINITY,
            best_spins: None,
            energy_history: Vec::with_capacity(n_steps),
        }
    }

    /// Solve an Ising problem
    ///
    /// J: coupling matrix (upper triangular)
    /// h: local fields
    pub fn solve(&mut self, j: &[Vec<f64>], h: &[f64]) -> SQAResult {
        let mut rng = rand::thread_rng();

        // Linear schedule
        let t_start = self.t_init;
        let t_end = self.t_final;
        let gamma_start = self.gamma_init;
        let gamma_end = 0.01;

        for step in 0..self.n_steps {
            let progress = step as f64 / self.n_steps as f64;

            // Annealing schedule
            let t = t_start * (t_end / t_start).powf(progress);
            let gamma = gamma_start * (gamma_end / gamma_start).powf(progress);

            // Compute transverse field term
            let j_perp = gamma * (self.n_trotters as f64).recip();

            // For each replica, compute energy and do Metropolis update
            for p in 0..self.n_trotters {
                let prev = if p == 0 { self.n_trotters - 1 } else { p - 1 };
                let next = if p == self.n_trotters - 1 { 0 } else { p + 1 };

                for i in 0..self.n_spins {
                    // Delta energy from flipping spin i in replica p
                    let mut delta = 0.0;

                    // Classical term
                    for k in (i + 1)..self.n_spins {
                        delta -= 2.0 * j[i][k] * self.path[p][i] as f64 * self.path[p][k] as f64;
                    }
                    delta -= 2.0 * h[i] * self.path[p][i] as f64;

                    // Quantum term (coupling to neighboring replicas)
                    if self.path[p][i] == self.path[prev][i] {
                        delta += j_perp;
                    } else {
                        delta -= j_perp;
                    }
                    if self.path[p][i] == self.path[next][i] {
                        delta += j_perp;
                    } else {
                        delta -= j_perp;
                    }

                    // Metropolis acceptance
                    if delta < 0.0 || rng.gen_bool((-delta / t).exp().min(1.0)) {
                        self.path[p][i] *= -1;
                    }
                }
            }

            // Record energy of first replica
            let e = self.energy(&self.path[0], j, h);
            self.energy_history.push(e);

            // Track best
            if e < self.best_energy {
                self.best_energy = e;
                self.best_spins = Some(self.path[0].clone());
            }
        }

        SQAResult {
            best_energy: self.best_energy,
            best_spins: self.best_spins.clone().unwrap_or_else(|| vec![1; self.n_spins]),
            energy_history: self.energy_history.clone(),
        }
    }

    /// Compute Ising energy
    fn energy(&self, spins: &[i8], j: &[Vec<f64>], h: &[f64]) -> f64 {
        let mut e = 0.0;

        // Interaction term
        for i in 0..self.n_spins {
            for k in (i + 1)..self.n_spins {
                e -= j[i][k] * spins[i] as f64 * spins[k] as f64;
            }
        }

        // Field term
        for i in 0..self.n_spins {
            e -= h[i] * spins[i] as f64;
        }

        e
    }
}

/// Result of SQA optimization
#[derive(Debug, Clone)]
pub struct SQAResult {
    /// Best energy found
    pub best_energy: f64,
    /// Best spin configuration
    pub best_spins: Vec<i8>,
    /// Energy during annealing
    pub energy_history: Vec<f64>,
}

/// Solve a QUBO problem (minimize x^T Q x)
///
/// Returns spin values (-1 or 1)
pub fn solve_qubo(q: &[Vec<f64>], n_trotters: usize) -> Vec<i8> {
    let n = q.len();

    // Convert QUBO to Ising
    let mut j = vec![vec![0.0; n]; n];
    let mut h = vec![0.0; n];

    for i in 0..n {
        for k in 0..n {
            if i < k {
                j[i][k] = 2.0 * q[i][k];
            } else if i == k {
                h[i] += 2.0 * q[i][k];
            }
        }
    }

    let mut sqa = SimulatedQuantumAnnealing::new(n, n_trotters, 1000);
    let result = sqa.solve(&j, &h);
    result.best_spins
}

/// Tensor network operations (simplified)

/// Compute tensor contraction along indices
pub fn tensor_contract(a: &[f64], dims_a: &[usize], b: &[f64], dims_b: &[usize], _contract_idx: usize) -> Vec<f64> {
    // Simplified: assumes 2D tensors
    let (m, k1) = (dims_a[0], dims_a[1]);
    let (k2, n) = (dims_b[0], dims_b[1]);

    if k1 != k2 {
        return vec![];
    }

    let k = k1;
    let mut result = vec![0.0; m * n];

    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0f64;
            for l in 0..k {
                sum += a[i * k + l] * b[l * n + j];
            }
            result[i * n + j] = sum;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqa_simple() {
        // Simple anti-ferromagnetic problem
        let n = 10;
        let J = vec![vec![0.0; n]; n];
        let h = vec![0.0; n];

        let mut sqa = SimulatedQuantumAnnealing::new(n, 5, 100);
        let result = sqa.solve(&J, &h);

        assert_eq!(result.best_spins.len(), n);
        assert!(result.best_energy.is_finite());
    }

    #[test]
    fn test_tensor_contract() {
        let a = vec![1.0, 2.0, 3.0, 4.0]; // 2x2
        let dims_a = vec![2, 2];
        let b = vec![1.0, 0.0, 0.0, 1.0]; // 2x2 identity
        let dims_b = vec![2, 2];

        let result = tensor_contract(&a, &dims_a, &b, &dims_b, 1);
        // Should be identity * a = a (contracting on inner dimension)
        assert_eq!(result, a);
    }
}
