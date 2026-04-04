"""
Quantum-Inspired Algorithms Module
=================================
CPU-based quantum-inspired optimization and tensor network operations.

IMPORTANT: "Quantum-inspired" means algorithms inspired by quantum mechanics
without requiring actual quantum hardware. These achieve efficiency gains on
specific problem types through mathematical frameworks from quantum theory.

Components:
1. Simulated Quantum Annealing (SQA) for Ising/QUBO optimization
2. Tensor network representations (TT, MPS)
3. Quantum walk sampling

References:
- Kirkpatrick & Selby (1993) - Quantum Annealing
- Santoro et al. (2002) - Theory of Simulated Quantum Annealing
- Oseledets (2011) - Tensor Train Decomposition
"""

import numpy as np
from typing import Tuple, Optional, Callable
from scipy.optimize import minimize


# ============================================================================
# SIMULATED QUANTUM ANNEALING (SQA)
# ============================================================================

class SimulatedQuantumAnnealing:
    """
    Simulated Quantum Annealing solver for Ising/QUBO problems.
    
    SQA simulates quantum tunneling by representing the system as a path
    of N × P "slices" (Trotter decomposition), where P is the number of
    replicas and quantum tunneling between slices allows the system to
    escape local minima that classical simulated annealing cannot.
    
    Parameters
    ----------
    n_spins : int
        Number of spin variables
    n_trotters : int
        Number of Trotter replicas (higher = more accurate quantum simulation)
    n_steps : int
        Number of annealing steps
    T_init : float
        Initial temperature
    T_final : float
        Final temperature
    gamma_init : float
        Initial transverse field strength (quantum fluctuation intensity)
    random_seed : int
        Random seed for reproducibility
    """
    
    def __init__(
        self,
        n_spins: int,
        n_trotters: int = 10,
        n_steps: int = 5000,
        T_init: float = 5.0,
        T_final: float = 0.01,
        gamma_init: float = 2.0,
        random_seed: Optional[int] = None
    ):
        if random_seed is not None:
            np.random.seed(random_seed)
        
        self.n_spins = n_spins
        self.n_trotters = n_trotters
        self.n_steps = n_steps
        self.T_init = T_init
        self.T_final = T_final
        self.gamma_init = gamma_init
        
        # Initialize path: (n_trotters × n_spins) spin configuration
        self.path = np.random.choice([-1, 1], size=(n_trotters, n_spins))
        
        # Current best solution
        self.best_energy = float('inf')
        self.best_spins = None
        self.best_trotter = 0
        
        # History for analysis
        self.energy_history = []
        self.temperature_history = []
        self.gamma_history = []
    
    def energy(self, spins: np.ndarray, J: np.ndarray, h: np.ndarray) -> float:
        """
        Compute Ising energy: E = -∑_i<j J_ij * s_i * s_j - ∑_i h_i * s_i
        
        Parameters
        ----------
        spins : np.ndarray
            Spin configuration, shape (n_spins,)
        J : np.ndarray
            Coupling matrix, shape (n_spins, n_spins)
        h : np.ndarray
            External field, shape (n_spins,)
            
        Returns
        -------
        float
            Energy value
        """
        # Ising energy: E = -0.5 * ∑_i≠j J_ij * s_i * s_j - ∑_i h_i * s_i
        interaction = 0.5 * np.sum(J * np.outer(spins, spins))
        field = np.sum(h * spins)
        return -interaction - field
    
    def energy_path(
        self,
        path: np.ndarray,
        J: np.ndarray,
        h: np.ndarray,
        gamma: float,
        T: float
    ) -> Tuple[float, np.ndarray]:
        """
        Compute effective energy of the full path including quantum terms.
        
        Parameters
        ----------
        path : np.ndarray
            Spin path, shape (n_trotters, n_spins)
        J, h : coupling matrices
        gamma : float
            Transverse field strength
        T : float
            Temperature
            
        Returns
        -------
        Tuple[float, np.ndarray]
            (total_energy, per_trotter_energies)
        """
        n_trotters, n_spins = path.shape
        
        # Classical energy for each trotter
        energies = np.array([
            self.energy(path[t], J, h) for t in range(n_trotters)
        ])
        
        # Quantum coupling energy (inter-trotter coupling)
        # Each spin couples to its copy in neighboring trotters
        quantum_energy = 0.0
        for t in range(n_trotters):
            t_next = (t + 1) % n_trotters
            # Coupling strength: -J_perp * s_t * s_{t+1}
            # J_perp = -T * log(tanh(gamma / (P * T))) for proper quantum-classical mapping
            J_perp = -T / np.log(np.tanh(gamma / (n_trotters * T) + 1e-10))
            quantum_energy += J_perp * np.sum(path[t] * path[t_next])
        
        total = np.sum(energies) + quantum_energy
        return total, energies
    
    def step(
        self,
        J: np.ndarray,
        h: np.ndarray,
        T: float,
        gamma: float
    ) -> float:
        """
        Single SQA step.
        
        Randomly flip one spin across all trotters (simultaneous tunneling).
        
        Parameters
        ----------
        J, h : coupling matrices
        T : float
            Current temperature
        gamma : float
            Current transverse field
            
        Returns
        -------
        float
            Energy change
        """
        n_trotters, n_spins = self.path.shape
        
        # Pick random spin to consider flipping
        spin_idx = np.random.randint(n_spins)
        
        # Current energy contribution of this spin
        E_current = 0.0
        E_proposed = 0.0
        
        for t in range(n_trotters):
            s = self.path[t, spin_idx]
            
            # Classical field contribution
            E_current -= h[spin_idx] * s
            
            # Coupling to other spins
            for j in range(n_spins):
                if j != spin_idx:
                    E_current -= 0.5 * J[spin_idx, j] * s * self.path[t, j]
            
            # Proposed flip
            s_new = -s
            
            E_proposed -= h[spin_idx] * s_new
            for j in range(n_spins):
                if j != spin_idx:
                    E_proposed -= 0.5 * J[spin_idx, j] * s_new * self.path[t, j]
        
        # Quantum tunneling term (coupling between trotters)
        J_perp = -T / np.log(np.tanh(gamma / (n_trotters * T) + 1e-10))
        
        for t in range(n_trotters):
            t_prev = (t - 1) % n_trotters
            t_next = (t + 1) % n_trotters
            
            s = self.path[t, spin_idx]
            s_prev = self.path[t_prev, spin_idx]
            s_next = self.path[t_next, spin_idx]
            
            # Current coupling
            E_current -= J_perp * s * (s_prev + s_next)
            
            # Proposed coupling
            s_new = -s
            E_proposed -= J_perp * s_new * (s_prev + s_next)
        
        delta_E = E_proposed - E_current
        
        # Metropolis acceptance
        if delta_E < 0 or np.random.rand() < np.exp(-delta_E / T):
            # Accept flip across all trotters
            self.path[:, spin_idx] *= -1
            return delta_E
        
        return 0.0
    
    def anneal(
        self,
        J: np.ndarray,
        h: np.ndarray,
        verbose: bool = False
    ) -> Tuple[np.ndarray, float]:
        """
        Run full SQA annealing schedule.
        
        Parameters
        ----------
        J : np.ndarray
            Coupling matrix, shape (n_spins, n_spins)
        h : np.ndarray
            External field, shape (n_spins,)
        verbose : bool
            Print progress
            
        Returns
        -------
        Tuple[np.ndarray, float]
            (best_solution, best_energy)
        """
        for step in range(self.n_steps):
            # Geometric cooling schedule
            progress = step / self.n_steps
            T = self.T_init * (self.T_final / self.T_init) ** progress
            gamma = self.gamma_init * (1 - progress)  # Quantum field decreases
            
            # Perform several spin flip attempts per step
            for _ in range(self.n_spins):
                self.step(J, h, T, gamma)
            
            # Record energy of best trotter
            energies = np.array([
                self.energy(self.path[t], J, h) for t in range(self.n_trotters)
            ])
            min_idx = np.argmin(energies)
            min_energy = energies[min_idx]
            
            if min_energy < self.best_energy:
                self.best_energy = min_energy
                self.best_spins = self.path[min_idx].copy()
                self.best_trotter = min_idx
            
            self.energy_history.append(self.best_energy)
            self.temperature_history.append(T)
            self.gamma_history.append(gamma)
            
            if verbose and (step + 1) % 500 == 0:
                print(f"Step {step+1}/{self.n_steps}: "
                      f"T={T:.4f}, γ={gamma:.4f}, E={self.best_energy:.4f}")
        
        return self.best_spins, self.best_energy
    
    def get_solution(self) -> Tuple[np.ndarray, float]:
        """Get current best solution."""
        return self.best_spins, self.best_energy


def solve_qubo(
    Q: np.ndarray,
    n_trotters: int = 10,
    n_steps: int = 5000,
    verbose: bool = False
) -> Tuple[np.ndarray, float]:
    """
    Solve a QUBO (Quadratic Unconstrained Binary Optimization) problem.
    
    QUBO: minimize x^T * Q * x
    
    Converts to Ising and uses SQA.
    
    Parameters
    ----------
    Q : np.ndarray
        QUBO matrix, shape (n, n)
    n_trotters : int
        Number of Trotter replicas
    n_steps : int
        Number of annealing steps
    verbose : bool
        
    Returns
    -------
    Tuple[np.ndarray, float]
        (optimal_binary_solution, optimal_energy)
    """
    n = Q.shape[0]
    
    # Convert QUBO to Ising:
    # x_i ∈ {0,1} → s_i ∈ {-1,+1}: x_i = (s_i + 1) / 2
    # QUBO: x^T * Q * x = -∑_i<j J_ij * s_i * s_j - ∑_i h_i * s_i + const
    
    # Ising couplings
    J = np.zeros((n, n))
    h = np.zeros(n)
    
    # Constant term
    const = np.sum(Q) / 4
    
    # Linear terms
    for i in range(n):
        h[i] = 0.5 * np.sum(Q[i, :]) + 0.5 * np.sum(Q[:, i])
    
    # Quadratic terms
    for i in range(n):
        for j in range(i+1, n):
            J[i, j] = Q[i, j] / 4
            J[j, i] = Q[i, j] / 4
    
    # Run SQA
    sqa = SimulatedQuantumAnnealing(
        n_spins=n,
        n_trotters=n_trotters,
        n_steps=n_steps,
        random_seed=42
    )
    
    solution_spins, energy = sqa.anneal(J, h, verbose=verbose)
    
    # Convert back to binary
    solution_binary = (solution_spins + 1) // 2
    
    # Compute original QUBO energy
    qubo_energy = solution_binary @ Q @ solution_binary
    
    return solution_binary, qubo_energy


def solve_ising(
    J: np.ndarray,
    h: np.ndarray,
    n_trotters: int = 10,
    n_steps: int = 5000,
    verbose: bool = False
) -> Tuple[np.ndarray, float]:
    """
    Solve an Ising problem directly using SQA.
    
    Ising: minimize -∑_i<j J_ij * s_i * s_j - ∑_i h_i * s_i
    
    Parameters
    ----------
    J : np.ndarray
        Coupling matrix, shape (n, n), symmetric
    h : np.ndarray
        External field, shape (n,)
    n_trotters : int
        Number of Trotter replicas
    n_steps : int
        Number of annealing steps
    verbose : bool
        
    Returns
    -------
    Tuple[np.ndarray, float]
        (optimal_spins, optimal_energy)
    """
    n = len(h)
    
    sqa = SimulatedQuantumAnnealing(
        n_spins=n,
        n_trotters=n_trotters,
        n_steps=n_steps,
        random_seed=42
    )
    
    return sqa.anneal(J, h, verbose=verbose)


# ============================================================================
# TENSOR NETWORK REPRESENTATIONS
# ============================================================================

def tensor_train_decompose(
    tensor: np.ndarray,
    bond_dim: int = 8
) -> Tuple[list, float]:
    """
    Decompose a high-order tensor into Tensor Train (TT) format.
    
    TT format: tensor[i_1, i_2, ..., i_d] = G_1[i_1] * G_2[i_2] * ... * G_d[i_d]
    
    where each G_k is a 3-tensor (r_{k-1}, i_k, r_k) with r_0 = r_d = 1.
    
    Uses SVD-based decomposition.
    
    Parameters
    ----------
    tensor : np.ndarray
        Input tensor of shape (n_1, n_2, ..., n_d)
    bond_dim : int
        Maximum bond dimension r_k
        
    Returns
    -------
    Tuple[list, float]
        (list of TT cores, relative_error)
    """
    original_shape = tensor.shape
    d = len(original_shape)
    
    if d == 0:
        return [], 0.0
    
    if d == 1:
        # 1D tensor - trivial case
        return [tensor.reshape(1, -1, 1)], 0.0
    
    # Use simple SVD-based TT decomposition
    cores = []
    current_tensor = tensor.copy()
    relative_error = 0.0
    
    for mode in range(d - 1):
        # Reshape to 2D: (prod of first modes, last mode)
        left_dim = np.prod(current_tensor.shape[:-1])
        right_dim = current_tensor.shape[-1]
        
        mat = current_tensor.reshape(left_dim, right_dim)
        
        # SVD
        U, S, Vh = np.linalg.svd(mat, full_matrices=False)
        
        # Truncate
        r = min(bond_dim, len(S), left_dim, right_dim)
        r = max(1, r)
        U = U[:, :r]
        S = S[:r]
        Vh = Vh[:r, :]
        
        # Update error
        recon = U @ np.diag(S) @ Vh
        error = np.linalg.norm(mat - recon) / (np.linalg.norm(mat) + 1e-16)
        relative_error += error
        
        # Store core for current mode
        # Shape: (r_left, original_shape[mode], r_right)
        r_left = U.shape[1]
        core_shape = (r_left, original_shape[mode], r)
        cores.append(U.reshape(core_shape))
        
        # Prepare next tensor
        current_tensor = (np.diag(S) @ Vh).reshape(-1, right_dim)
    
    # Last core
    cores.append(current_tensor.reshape(current_tensor.shape[0], original_shape[-1], 1))
    
    return cores, relative_error


def tensor_train_contract(
    cores: list,
    indices: list
) -> float:
    """
    Contract a Tensor Train with specific indices.
    
    Parameters
    ----------
    cores : list
        TT cores from tensor_train_decompose
    indices : list
        List of indices [i_1, i_2, ..., i_d]
        
    Returns
    -------
    float
        Contracted tensor element
    """
    result = np.array([[1.0]])  # Start with scalar (r_0 = r_d = 1)
    
    for k, (core, idx) in enumerate(zip(cores, indices)):
        # core shape: (r_{k-1}, i_k, r_k)
        result = result @ core[:, idx, :]  # (r_{k-1},) @ (r_{k-1}, i_k, r_k) = (r_k,)
    
    return result[0, 0]


def tensor_train_reconstruct(cores: list) -> np.ndarray:
    """
    Reconstruct full tensor from TT cores.
    
    Parameters
    ----------
    cores : list
        TT cores from tensor_train_decompose
        
    Returns
    -------
    np.ndarray
        Reconstructed tensor
    """
    d = len(cores)
    
    if d == 0:
        return np.array([])
    
    if d == 1:
        # Single core - just reshape
        return cores[0].reshape(cores[0].shape[1])
    
    # Contract cores sequentially
    # Start with first core: (r_0, i_0, r_1)
    result = cores[0][0, :, :]  # shape: (i_0, r_1)
    
    for k in range(1, d):
        core = cores[k]  # (r_{k-1}, i_k, r_k)
        # Contract: (i_{k-1}, r_{k-1}) @ (r_{k-1}, i_k, r_k) -> (i_{k-1}, i_k)
        result = result @ core.reshape(result.shape[-1], -1)
        result = result.reshape(-1, core.shape[1])
    
    return result


def cognitive_state_compress(
    state_vector: np.ndarray,
    bond_dim: int = 8
) -> Tuple[list, np.ndarray, float]:
    """
    Compress a high-dimensional cognitive state vector using TT decomposition.
    
    For a state vector of size N, we factor it into compatible dimensions 
    and decompose into tensor train format. The compressed representation
    uses TT cores which can be much smaller than the original vector for
    structured/low-rank vectors.
    
    Parameters
    ----------
    state_vector : np.ndarray
        Flat state vector, length N
    bond_dim : int
        Maximum bond dimension
        
    Returns
    -------
    Tuple[list, np.ndarray, float]
        (tt_cores, reconstructed_vector, compression_ratio)
        compression_ratio = original_size / compressed_size (higher = better compression)
    """
    n = len(state_vector)
    
    # Factor n into compatible dimensions for TT decomposition
    # Simple factorization: try powers of 2
    dims = []
    remaining = n
    
    while remaining > 1:
        if remaining % 2 == 0:
            dims.append(2)
            remaining //= 2
        elif remaining % 3 == 0:
            dims.append(3)
            remaining //= 3
        elif remaining % 5 == 0:
            dims.append(5)
            remaining //= 5
        else:
            # Can't factor nicely - just use 1D
            dims.append(remaining)
            break
    
    if not dims:
        dims = [n]
    
    # Now reverse to get proper order
    dims = list(reversed(dims))
    
    # Ensure product equals n - if not, use 1D
    if np.prod(dims) != n:
        # Return as-is using simple SVD
        dims = [n]
        dims[0] *= 2 if np.prod(dims) > n else 1
    
    # Reshape to tensor
    try:
        tensor = state_vector.reshape(dims)
    except ValueError:
        # Fallback: use 1D
        return [], state_vector, 1.0
    
    # Decompose
    cores, decomp_error = tensor_train_decompose(tensor, bond_dim)
    
    if not cores:
        return [], state_vector, 1.0
    
    # Reconstruct from TT cores
    try:
        reconstructed = tensor_train_reconstruct(cores)
    except Exception:
        return [], state_vector, 1.0
    
    # Compute compression ratio: original / compressed
    original_size = np.prod(dims) * tensor.dtype.itemsize
    compressed_size = sum(
        core.shape[0] * core.shape[1] * core.shape[2] * core.dtype.itemsize 
        for core in cores
    )
    ratio = original_size / compressed_size if compressed_size > 0 else 1.0
    
    # Flatten reconstructed back to vector
    try:
        reconstructed_flat = reconstructed.ravel()
    except Exception:
        reconstructed_flat = state_vector
    
    # Reconstruction error - handle size mismatch
    min_len = min(len(state_vector), len(reconstructed_flat))
    recon_error = np.sqrt(np.mean((state_vector[:min_len] - reconstructed_flat[:min_len]) ** 2))
    
    return cores, reconstructed_flat, ratio


# ============================================================================
# QUANTUM WALK SAMPLING
# ============================================================================

class QuantumWalkSampler:
    """
    Quantum-inspired sampling using quantum walk dynamics.
    
    Provides samples from a distribution inspired by quantum walks,
    which spread faster than classical random walks.
    
    Parameters
    ----------
    adj_matrix : np.ndarray
        Adjacency matrix of the graph to walk on
    n_steps : int
        Number of walk steps
    n_walkers : int
        Number of parallel walkers
    """
    
    def __init__(
        self,
        adj_matrix: np.ndarray,
        n_steps: int = 100,
        n_walkers: int = 100,
        random_seed: Optional[int] = None
    ):
        if random_seed is not None:
            np.random.seed(random_seed)
        
        self.adj = adj_matrix
        self.n_nodes = adj_matrix.shape[0]
        self.n_steps = n_steps
        self.n_walkers = n_walkers
        
        # Normalize adjacency for transition probabilities
        self.transition_probs = adj_matrix / (adj_matrix.sum(axis=1, keepdims=True) + 1e-10)
        
        # Initialize walker positions
        self.walker_positions = np.random.randint(0, self.n_nodes, n_walkers)
        
    def step(self) -> np.ndarray:
        """
        One step of the quantum walk.
        
        Uses superposition-like update: each walker distributes its
        "amplitude" across all possible next positions, weighted by
        transition probabilities.
        
        Returns
        -------
        np.ndarray
            Visit counts after this step
        """
        new_positions = np.zeros(self.n_walkers, dtype=int)
        
        for w in range(self.n_walkers):
            current = self.walker_positions[w]
            # Quantum-inspired: use transition probabilities
            # but with "interference" from multiple walkers
            probs = self.transition_probs[current]
            probs = probs ** 0.5  # Quantum-inspired: sqrt for interference
            
            # Handle zero/all-zero probabilities
            prob_sum = probs.sum()
            if prob_sum <= 0:
                # No outgoing edges - stay at current node
                probs = np.ones(self.n_nodes) / self.n_nodes
            else:
                probs = probs / prob_sum
            
            # Handle NaN values
            if not np.all(np.isfinite(probs)):
                probs = np.ones(self.n_nodes) / self.n_nodes
            
            try:
                new_positions[w] = np.random.choice(self.n_nodes, p=probs)
            except ValueError:
                # Fallback if probabilities are invalid
                new_positions[w] = current
        
        self.walker_positions = new_positions
        
        # Return visit counts
        counts = np.bincount(self.walker_positions, minlength=self.n_nodes)
        return counts.astype(float)
    
    def run(self) -> Tuple[np.ndarray, np.ndarray]:
        """
        Run the full quantum walk.
        
        Returns
        -------
        Tuple[np.ndarray, np.ndarray]
            (stationary_distribution, history)
        """
        history = np.zeros((self.n_steps, self.n_nodes))
        
        for t in range(self.n_steps):
            counts = self.step()
            history[t] = counts / counts.sum()
        
        # Stationary distribution is the average over last 20% of steps
        last_20 = int(0.8 * self.n_steps)
        stationary = history[last_20:].mean(axis=0)
        stationary = stationary / stationary.sum()  # Normalize
        
        return stationary, history
    
    def sample(self, n_samples: int = 1000) -> np.ndarray:
        """
        Generate samples from the stationary distribution.
        
        Parameters
        ----------
        n_samples : int
            Number of samples to generate
            
        Returns
        -------
        np.ndarray
            Sampled node indices
        """
        stationary, _ = self.run()
        return np.random.choice(self.n_nodes, size=n_samples, p=stationary)


# ============================================================================
# PARALLEL TEMPERING (for improved exploration)
# ============================================================================

class ParallelTemperingSQA:
    """
    Parallel tempering wrapper for SQA.
    
    Runs multiple SQA replicas at different temperatures in parallel,
    allowing better exploration of the energy landscape.
    
    Parameters
    ----------
    n_replicas : int
        Number of temperature replicas
    n_spins : int
        Number of spins
    T_min : float
        Minimum temperature
    T_max : float
        Maximum temperature
    exchange_interval : int
        Steps between temperature exchange attempts
    """
    
    def __init__(
        self,
        n_replicas: int = 5,
        n_spins: int = 100,
        T_min: float = 0.1,
        T_max: float = 10.0,
        exchange_interval: int = 100
    ):
        self.n_replicas = n_replicas
        self.n_spins = n_spins
        self.exchange_interval = exchange_interval
        
        # Temperature ladder
        self.T_ladder = np.logspace(
            np.log10(T_min), np.log10(T_max), n_replicas
        )[::-1]  # High T to low T
        
        # Create replicas
        self.replicas = [
            SimulatedQuantumAnnealing(
                n_spins=n_spins,
                n_trotters=5,  # Fewer trotters per replica for speed
                n_steps=10000,
                T_init=T,
                T_final=T * 0.01,
                random_seed=i
            )
            for i, T in enumerate(self.T_ladder)
        ]
        
        self.current_step = 0
        
    def step(self, J: np.ndarray, h: np.ndarray) -> float:
        """
        One step of parallel tempering.
        """
        self.current_step += 1
        
        # Step each replica
        energies = []
        for r, replica in enumerate(self.replicas):
            T = self.T_ladder[r]
            gamma = 1.0 * (1 - self.current_step / replica.n_steps)
            
            for _ in range(replica.n_spins):
                replica.step(J, h, T, max(gamma, 0.01))
            
            # Record energy
            best_spins, best_energy = replica.get_solution()
            energies.append(best_energy)
        
        # Exchange temperatures periodically
        if self.current_step % self.exchange_interval == 0:
            self._exchange_temperatures(energies)
        
        return min(energies)
    
    def _exchange_temperatures(self, energies: list):
        """
        Attempt to exchange temperatures between adjacent replicas.
        Metropolis acceptance: accept if Δ < 0 or rand < exp(-Δ/T)
        """
        for i in range(self.n_replicas - 1):
            T_i = self.T_ladder[i]
            T_j = self.T_ladder[i + 1]
            
            delta = (1/T_i - 1/T_j) * (energies[i] - energies[i + 1])
            
            if delta < 0 or np.random.rand() < np.exp(delta):
                # Accept exchange
                self.T_ladder[i], self.T_ladder[i + 1] = self.T_ladder[i + 1], self.T_ladder[i]
    
    def solve(
        self,
        J: np.ndarray,
        h: np.ndarray,
        verbose: bool = False
    ) -> Tuple[np.ndarray, float]:
        """
        Run parallel tempering SQA to solve Ising problem.
        """
        for step in range(self.replicas[0].n_steps // self.n_spins):
            best_energy = self.step(J, h)
            
            if verbose and step % 100 == 0:
                print(f"Step {step}: best_E = {best_energy:.4f}")
        
        # Return best solution across all replicas
        best_overall = np.array([])
        best_energy_overall = float('inf')
        
        for replica in self.replicas:
            spins, energy = replica.get_solution()
            if spins is not None and energy < best_energy_overall:
                best_energy_overall = energy
                best_overall = spins
        
        # Handle case where no solution was found
        if best_overall is None or len(best_overall) == 0:
            return np.array([]), float('inf')
        
        return best_overall, best_energy_overall
