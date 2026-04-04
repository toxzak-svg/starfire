"""
Simulated Quantum Annealing (SQA)
===============================
Quantum-inspired optimizer using Path Integral Monte Carlo.

SQA simulates quantum annealing by mapping the problem onto a "path integral" 
representation with multiple "replicas" (Trotter slices). Quantum fluctuations
(tunneling) allow the system to escape local minima that classical simulated
annealing cannot.

CPU-optimized implementation using NumPy.

References:
- Santoro et al. (2002) - Theory of Quantum Annealing
- Martonak et al. (2002) - Path Integral Monte Carlo for optimization
- Troyer et al. (2005) - Performance of SQA vs classical SA
"""

import numpy as np
from typing import Optional, Tuple, Callable


class SQAOptimizer:
    """
    Simulated Quantum Annealing Optimizer.
    
    Solves QUBO (Quadratic Unconstrained Binary Optimization) and Ising problems
    using Path Integral Monte Carlo (PIMC).
    
    The algorithm maintains N_trotters parallel "replicas" of the system,
    coupled together to simulate quantum tunneling. This allows solutions
    to tunnel through energy barriers instead of climbing over them.
    
    Parameters
    ----------
    n_spins : int
        Number of spins (variables)
    n_trotters : int
        Number of Trotter slices (replicas). Higher = more quantum behavior,
        but slower. Typical: 8-20.
    coupling : float
        Inter-trotter coupling strength (J_perp). Controls tunneling rate.
        Typical: 0.5-2.0.
    """
    
    def __init__(
        self,
        n_spins: int,
        n_trotters: int = 10,
        coupling: float = 1.0,
        seed: Optional[int] = None
    ):
        self.n_spins = n_spins
        self.n_trotters = n_trotters
        self.coupling = coupling
        
        if seed is not None:
            np.random.seed(seed)
        
        # Path: (n_trotters x n_spins) spin configuration
        # Initialize random configuration
        self.path = np.random.choice([-1, 1], size=(n_trotters, n_spins))
        
        # Best solution found
        self.best_path = None
        self.best_energy = np.inf
        
        # History
        self.energy_history = []
        self.accept_history = []
    
    def set_qubo(self, qubo_matrix: np.ndarray):
        """
        Set the QUBO problem matrix.
        
        QUBO: minimize x^T @ Q @ x
        
        Parameters
        ----------
        qubo_matrix : np.ndarray
            QUBO matrix of shape (n_spins, n_spins)
        """
        if qubo_matrix.shape != (self.n_spins, self.n_spins):
            raise ValueError(f"Expected {self.n_spins}x{self.n_spins} matrix, got {qubo_matrix.shape}")
        self.qubo_matrix = qubo_matrix
    
    def set_ising(
        self,
        h: np.ndarray,
        J: np.ndarray
    ):
        """
        Set the Ising problem.
        
        Ising: H = -sum_i h_i*s_i - sum_{i<j} J_ij * s_i * s_j
        
        Parameters
        ----------
        h : np.ndarray
            Local fields (n_spins,)
        J : np.ndarray
            Couplings (n_spins, n_spins) - upper triangular
        """
        self.h = h
        self.J = J
        
        # Convert to QUBO for uniform handling
        # QUBO: x_i = (s_i + 1) / 2  (map from spin {-1,1} to bit {0,1})
        n = self.n_spins
        
        # QUBO matrix from Ising
        Q = np.zeros((n, n))
        for i in range(n):
            for j in range(n):
                if i != j:
                    Q[i, j] = J[i, j] / 4.0
                else:
                    Q[i, i] = -h[i] / 2.0
        
        self.qubo_matrix = Q
    
    def _compute_energy(self, spins: np.ndarray) -> float:
        """Compute QUBO energy for a single spin configuration."""
        return float(spins @ self.qubo_matrix @ spins)
    
    def _compute_path_energy(self) -> float:
        """Compute total energy of the path (sum over trotters + coupling)."""
        # Classical energy for each trotters
        classical = sum(self._compute_energy(self.path[t]) for t in range(self.n_trotters))
        
        # Tunneling term: -J_perp * sum_{t} s_t @ s_{t+1}
        tunnel = 0.0
        for t in range(self.n_trotters):
            t_next = (t + 1) % self.n_trotters
            # - coupling encourages alignment (tunneling)
            tunnel += self.coupling * np.sum(self.path[t] * self.path[t_next])
        
        return classical - tunnel
    
    def _metropolis_step(
        self,
        temp: float,
        gamma: float
    ) -> int:
        """
        Perform one PIMC Metropolis step.
        
        Parameters
        ----------
        temp : float
            Temperature
        gamma : float
            Quantum field strength (0 = classical, 1 = full quantum)
            
        Returns
        -------
        int
            Number of accepted moves
        """
        n_accepts = 0
        
        for t in range(self.n_trotters):
            # Pick random spin to flip
            i = np.random.randint(self.n_spins)
            
            # Compute local energy change
            delta_E = 0.0
            
            # QUBO contribution
            for j in range(self.n_spins):
                delta_E += 2 * self.qubo_matrix[i, j] * self.path[t, i] * self.path[t, j]
            
            # Quantum tunneling: coupling to neighboring trotters
            t_prev = (t - 1) % self.n_trotters
            t_next = (t + 1) % self.n_trotters
            
            # The change in tunneling energy when spin i is flipped
            delta_tunnel = self.coupling * (
                self.path[t_prev, i] + self.path[t_next, i]
            ) * (-2 * self.path[t, i])  # Flip changes s_i to -s_i
            
            delta_E += delta_tunnel * gamma
            
            # Acceptance: flip if energy decreases or with probability exp(-ΔE/T)
            if delta_E < 0 or np.random.rand() < np.exp(-delta_E / temp):
                self.path[t, i] *= -1
                n_accepts += 1
        
        return n_accepts
    
    def optimize(
        self,
        n_steps: int = 10000,
        T_init: float = 5.0,
        T_final: float = 0.01,
        annealing_schedule: str = 'geometric',
        verbose: bool = True
    ) -> Tuple[np.ndarray, float]:
        """
        Run SQA optimization.
        
        Parameters
        ----------
        n_steps : int
            Number of Monte Carlo steps
        T_init : float
            Initial temperature
        T_final : float
            Final temperature  
        annealing_schedule : str
            'geometric' or 'linear' cooling
        verbose : bool
            Print progress
            
        Returns
        -------
        Tuple[np.ndarray, float]
            (best_solution, best_energy)
        """
        self.energy_history = []
        self.accept_history = []
        
        for step in range(n_steps):
            # Temperature schedule
            if annealing_schedule == 'geometric':
                frac = step / n_steps
                T = T_init * (T_final / T_init) ** frac
                gamma = 1.0 - frac  # Quantum field strength
            else:
                T = T_init - (T_init - T_final) * step / n_steps
                gamma = 1.0 - step / n_steps
            
            # Metropolis step
            n_accepts = self._metropolis_step(T, gamma)
            
            # Record energy
            current_energy = self._compute_path_energy() / self.n_trotters
            self.energy_history.append(current_energy)
            self.accept_history.append(n_accepts)
            
            # Track best
            if current_energy < self.best_energy:
                self.best_energy = current_energy
                # Use majority vote across trotters
                self.best_path = np.sign(np.sum(self.path, axis=0))
            
            if verbose and step % (n_steps // 10) == 0:
                print(f"Step {step}/{n_steps}: T={T:.4f}, gamma={gamma:.4f}, "
                      f"E={current_energy:.4f}, best={self.best_energy:.4f}")
        
        # Return best solution (majority vote)
        best_solution = np.sign(np.sum(self.path, axis=0))
        best_solution[best_solution == 0] = 1  # Handle ties
        
        return best_solution, self.best_energy
    
    def get_solution(self, method: str = 'majority') -> np.ndarray:
        """
        Get the current solution.
        
        Parameters
        ----------
        method : str
            'majority' (vote across trotters) or 'best' (stored best)
            
        Returns
        -------
        np.ndarray
            Spin configuration
        """
        if method == 'best' and self.best_path is not None:
            return self.best_path.copy()
        
        # Majority vote
        return np.sign(np.sum(self.path, axis=0))
    
    def get_statistics(self) -> dict:
        """Get optimization statistics."""
        if not self.energy_history:
            return {}
        
        return {
            'best_energy': self.best_energy,
            'final_energy': self.energy_history[-1],
            'mean_accepts': np.mean(self.accept_history),
            'n_improvements': sum(
                1 for i in range(1, len(self.energy_history))
                if self.energy_history[i] < self.energy_history[i-1]
            )
        }


def solve_qubo_sqa(
    qubo_matrix: np.ndarray,
    n_steps: int = 10000,
    n_trotters: int = 10,
    seed: Optional[int] = None
) -> Tuple[np.ndarray, float]:
    """
    Convenience function to solve a QUBO problem using SQA.
    
    Parameters
    ----------
    qubo_matrix : np.ndarray
        QUBO matrix
    n_steps : int
        Number of optimization steps
    n_trotters : int
        Number of Trotter slices
    seed : int
        Random seed
        
    Returns
    -------
    Tuple[np.ndarray, float]
        (solution, energy)
    """
    n = qubo_matrix.shape[0]
    optimizer = SQAOptimizer(n, n_trotters=n_trotters, seed=seed)
    optimizer.set_qubo(qubo_matrix)
    return optimizer.optimize(n_steps=n_steps)


def solve_ising_sqa(
    h: np.ndarray,
    J: np.ndarray,
    n_steps: int = 10000,
    n_trotters: int = 10,
    seed: Optional[int] = None
) -> Tuple[np.ndarray, float]:
    """
    Convenience function to solve an Ising problem using SQA.
    
    Parameters
    ----------
    h : np.ndarray
        Local fields
    J : np.ndarray  
        Couplings
    n_steps : int
        Number of optimization steps
    n_trotters : int
        Number of Trotter slices
    seed : int
        Random seed
        
    Returns
    -------
    Tuple[np.ndarray, float]
        (solution, energy)
    """
    n = len(h)
    optimizer = SQAOptimizer(n, n_trotters=n_trotters, seed=seed)
    optimizer.set_ising(h, J)
    return optimizer.optimize(n_steps=n_steps)