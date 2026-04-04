import numpy as np
from scipy.sparse import random as sparse_random
from scipy.sparse import csr_matrix
from joblib import Parallel, delayed
from typing import Optional, Tuple

class ChaoticReservoir:
    """
    Echo State Network with chaotic modulation.
    
    The reservoir maintains a nonlinear temporal memory through recurrent
    connections. Training is performed only on output weights (linear readout).
    
    Parameters
    ----------
    input_dim : int
        Dimension of input vectors
    reservoir_size : int
        Number of neurons in the reservoir (default 500)
    spectral_radius : float
        Largest eigenvalue of reservoir weight matrix (0.0-1.0+)
        Values near 1.0 = edge of chaos, maximal computational power
        Values > 1.0 = chaotic regime with long-term memory
    input_scaling : float
        Scale factor for input weights (default 0.1)
    connectivity : float
        Fraction of possible reservoir connections (default 0.01 = 1%)
    noise_level : float
        Scale of chaotic modulation perturbation (default 0.001)
    """

    def __init__(self,
                 input_dim: int,
                 reservoir_size: int = 500,
                 spectral_radius: float = 0.95,
                 input_scaling: float = 0.1,
                 connectivity: float = 0.01,
                 noise_level: float = 0.001,
                 random_seed: Optional[int] = None):
        if random_seed is not None:
            np.random.seed(random_seed)
        
        self.input_dim = input_dim
        self.reservoir_size = reservoir_size
        self.spectral_radius = spectral_radius
        self.noise_level = noise_level
        
        # Input weights: random, sparse
        self.W_in = np.random.randn(reservoir_size, input_dim) * input_scaling
        
        # Reservoir weights: random, sparse
        W_full = csr_matrix(sparse_random(reservoir_size, reservoir_size, density=connectivity))
        
        # Scale to desired spectral radius
        eigvals = np.linalg.eigvals(W_full.toarray())
        max_eig = np.max(np.abs(eigvals))
        if max_eig > 0:
            self.W = W_full * (spectral_radius / max_eig)
        else:
            self.W = W_full
        
        # Output weights (trained via ridge regression)
        self.W_out: Optional[np.ndarray] = None
        
        # State
        self.state = np.zeros(reservoir_size)
        
    def reset(self):
        """Reset reservoir state to zero."""
        self.state = np.zeros(self.reservoir_size)
    
    def forward(self, input_sequence: np.ndarray) -> np.ndarray:
        """
        Process an input sequence through the reservoir.
        
        Parameters
        ----------
        input_sequence : np.ndarray
            Shape (timesteps, input_dim)
        
        Returns
        -------
        np.ndarray
            Reservoir states, shape (timesteps, reservoir_size)
        """
        timesteps = input_sequence.shape[0]
        states = np.zeros((timesteps, self.reservoir_size))
        
        for t in range(timesteps):
            input_vec = input_sequence[t]
            chaos_term = self.noise_level * np.tanh(self.state)
            self.state = np.tanh(
                self.W_in @ input_vec + 
                self.W @ self.state + 
                chaos_term
            )
            states[t] = self.state.copy()
        
        return states
    
    def forward_step(self, input_vec: np.ndarray) -> np.ndarray:
        """
        Single step forward pass (for real-time processing).
        
        Parameters
        ----------
        input_vec : np.ndarray
            Shape (input_dim,)
        
        Returns
        -------
        np.ndarray
            New reservoir state, shape (reservoir_size,)
        """
        chaos_term = self.noise_level * np.tanh(self.state)
        self.state = np.tanh(self.W_in @ input_vec + self.W @ self.state + chaos_term)
        return self.state.copy()
    
    def train_readout(self,
                     states: np.ndarray,
                     targets: np.ndarray,
                     regularization: float = 1e-6
                     ) -> float:
        """
        Train linear readout weights using ridge regression.
        """
        n_samples = states.shape[0]
        
        # Add bias term (column of ones)
        X = np.hstack([states, np.ones((n_samples, 1))])
        Y = targets
        
        # W_out = Y^T @ X @ (XX^T + λI)^{-1}
        # Using solve instead of inverse for numerical stability
        A = X.T @ X + regularization * np.eye(X.shape[1])
        B = Y.T @ X
        self.W_out = np.linalg.solve(A, B.T).T
        
        # Compute training error
        predictions = X @ self.W_out.T
        rmse = np.sqrt(np.mean((predictions - Y) ** 2))
        
        return rmse
    
    def predict(self, states: np.ndarray) -> np.ndarray:
        """
        Generate predictions from reservoir states.
        
        Parameters
        ----------
        states : np.ndarray
            Reservoir states, shape (timesteps, reservoir_size)
        
        Returns
        -------
        np.ndarray
            Predictions, shape (timesteps, output_dim)
        """
        if self.W_out is None:
            raise ValueError("Readout weights not trained. Call train_readout first.")
        
        n_samples = states.shape[0]
        X = np.hstack([states, np.ones((n_samples, 1))])  # Add bias
        return X @ self.W_out.T
    
    def get_state(self) -> np.ndarray:
        """Get current reservoir state."""
        return self.state.copy()
    
    def estimate_lyapunov_online(
        self,
        n_timesteps: int = 100,
        separation: float = 1e-6
    ) -> float:
        """
        Estimate maximal Lyapunov exponent via direct divergence method.
        
        This is a lightweight online approximation - simpler than Benettin
        but sufficient for reservoir regime monitoring.
        
        Parameters
        ----------
        n_timesteps : int
            Number of divergence measurements to average
        separation : float
            Initial separation between test trajectories
            
        Returns
        -------
        float
            Estimated maximal Lyapunov exponent
        """
        # Generate small perturbation
        perturbation = np.random.randn(self.reservoir_size)
        perturbation = perturbation / np.linalg.norm(perturbation) * separation
        
        state_perturbed = self.state + perturbation
        divergences = []
        
        for _ in range(n_timesteps):
            # Evolve both states
            chaos_term = self.noise_level * np.tanh(self.state)
            self.state = np.tanh(
                self.W_in @ np.zeros(self.input_dim) +  # No input during lyapunov test
                self.W @ self.state + 
                chaos_term
            )
            
            chaos_term_p = self.noise_level * np.tanh(state_perturbed)
            state_perturbed = np.tanh(
                self.W_in @ np.zeros(self.input_dim) + 
                self.W @ state_perturbed + 
                chaos_term_p
            )
            
            # Track separation
            sep = np.linalg.norm(state_perturbed - self.state)
            divergences.append(max(sep, separation))
            
            # Renormalize if too far apart
            if sep > 0.1:
                direction = (state_perturbed - self.state) / sep
                state_perturbed = self.state + direction * separation
        
        # Compute MLE: slope of log(divergence) vs time
        divergences = np.array(divergences)
        valid = divergences > separation * 1.1
        
        if np.sum(valid) < 10:
            return 0.0
        
        times = np.arange(len(divergences))
        log_div = np.log(divergences[valid])
        times_valid = times[valid]
        
        coeffs = np.polyfit(times_valid, log_div, 1)
        return coeffs[0]
    
    def get_regime(self, mle_threshold: float = 0.1) -> str:
        """
        Get current dynamical regime based on estimated behavior.
        
        Parameters
        ----------
        mle_threshold : float
            MLE threshold for chaotic boundary
            
        Returns
        -------
        str
            'stable', 'edge_of_chaos', or 'chaotic'
        """
        # Simple heuristic based on spectral radius
        if self.spectral_radius < 0.8:
            return 'stable'
        elif self.spectral_radius < 1.0:
            return 'edge_of_chaos'
        else:
            return 'chaotic'


# ============================================================================
# CREATIVE OSCILLATION CONTROLLER
# ============================================================================

class CreativeOscillator:
    """
    Creative oscillation between order (exploitation) and chaos (exploration).
    
    Based on the "divergence-exploration cycles" model in computational creativity:
    - Order phase: refine and converge toward known good solutions
    - Chaos phase: inject randomness to escape local minima and explore
    
    Parameters
    ----------
    order_threshold : float
        Divergence metric threshold for triggering chaos injection (0-1)
    chaos_threshold : float
        Divergence threshold for returning to order (0-1)
    max_exploration_steps : int
        Maximum steps in exploratory mode before forcing return
    convergence_rate : float
        Rate of convergence during order phase (0-1)
    """
    
    def __init__(
        self,
        order_threshold: float = 0.7,
        chaos_threshold: float = 0.3,
        max_exploration_steps: int = 50,
        convergence_rate: float = 0.1,
        random_seed: Optional[int] = None
    ):
        if random_seed is not None:
            np.random.seed(random_seed)
        
        self.state = 'ordered'  # 'ordered' or 'exploratory'
        self.order_threshold = order_threshold
        self.chaos_threshold = chaos_threshold
        self.max_exploration = max_exploration_steps
        self.convergence_rate = convergence_rate
        
        # State tracking
        self.exploration_count = 0
        self.best_exploration_value = 0.0
        self.history = []
        
        # Current value (utility of current state)
        self.current_value = 0.0
    
    def step(
        self,
        current_value: float,
        divergence_metric: float
    ) -> dict:
        """
        One step of creative oscillation.
        
        Parameters
        ----------
        current_value : float
            Value/utility of current cognitive state (0-1)
        divergence_metric : float
            Measure of state divergence from baseline (0 = very ordered, 1 = very chaotic)
            
        Returns
        -------
        dict
            {'action': str, 'new_state': str, 'perturbation_scale': float}
        """
        self.current_value = current_value
        
        action = 'converge'
        perturbation_scale = 0.0
        
        if self.state == 'ordered':
            if divergence_metric < self.chaos_threshold:
                # Too ordered - inject chaos!
                self.state = 'exploratory'
                self.exploration_count = 0
                action = 'chaos_injection'
                perturbation_scale = 0.1
            else:
                # Keep refining
                action = 'converge'
                perturbation_scale = self.convergence_rate * (1 - divergence_metric)
        
        else:  # 'exploratory'
            self.exploration_count += 1
            
            if current_value > self.best_exploration_value:
                self.best_exploration_value = current_value
            
            if self.exploration_count >= self.max_exploration:
                # Max exploration reached - converge to best found
                self.state = 'ordered'
                action = 'stabilize'
                perturbation_scale = 0.0
            elif current_value > self.order_threshold:
                # Found good enough - stabilize
                self.state = 'ordered'
                action = 'stabilize'
                perturbation_scale = 0.0
            else:
                action = 'continue_exploring'
                perturbation_scale = -0.1  # Increase chaos
        
        # Record history
        self.history.append({
            'state': self.state,
            'value': current_value,
            'divergence': divergence_metric,
            'action': action
        })
        
        return {
            'action': action,
            'new_state': self.state,
            'perturbation_scale': perturbation_scale,
            'exploration_count': self.exploration_count
        }
    
    def get_attractor_strength(self, value: float) -> float:
        """
        How strongly is this value attracting the system?
        
        Returns
        -------
        float
            Attractor strength (0-1)
        """
        if self.state == 'ordered':
            return np.exp(-((value - 1.0) ** 2) / 0.1)
        else:
            return 0.1  # weak attraction during exploration
    
    def get_status(self) -> dict:
        """Get current oscillator status."""
        return {
            'state': self.state,
            'exploration_count': self.exploration_count,
            'best_value': self.best_exploration_value,
            'current_value': self.current_value
        }


# ============================================================================
# ATTRACTOR BASIN NAVIGATION
# ============================================================================

def attractor_escape(
    current_state: np.ndarray,
    basin_strength: float,
    escape_threshold: float = 0.5,
    perturbation_scale: float = 0.1
) -> np.ndarray:
    """
    Escape weak attractor basin via chaotic perturbation.
    
    When the system is in a weak attractor basin (local optimum),
    inject chaotic perturbation to escape and explore new basins.
    
    Parameters
    ----------
    current_state : np.ndarray
        Current cognitive state
    basin_strength : float
        Strength of attractor basin (0 = very weak, 1 = very strong)
    escape_threshold : float
        Basin strength below which to trigger escape
    perturbation_scale : float
        Scale of perturbation to apply
        
    Returns
    -------
    np.ndarray
        New state (possibly perturbed)
    """
    if basin_strength < escape_threshold:
        # Escape the basin via chaotic modulation
        perturbation = perturbation_scale * np.random.randn(*current_state.shape)
        return current_state + perturbation
    return current_state


def compute_basin_strength(
    states: np.ndarray,
    current_state: np.ndarray,
    window: int = 50
) -> float:
    """
    Estimate attractor basin strength.
    
    Uses local variance in state space to estimate how strongly
    the current state is trapped in a basin.
    
    Parameters
    ----------
    states : np.ndarray
        Historical states, shape (n_steps, state_dim)
    current_state : np.ndarray
        Current state
    window : int
        Window for local variance calculation
        
    Returns
    -------
    float
        Basin strength estimate (0 = weak, 1 = strong)
    """
    if len(states) < window:
        return 0.5  # No history - assume moderate
    
    recent = states[-window:]
    
    # Distance from current to recent states
    dists = np.linalg.norm(recent - current_state, axis=1)
    
    # Low variance = strong basin, high variance = weak basin
    variance = np.var(dists)
    
    # Convert to strength (0-1)
    strength = 1.0 / (1.0 + variance)
    return np.clip(strength, 0.0, 1.0)
    

def narma_task(
    reservoir: ChaoticReservoir,
    n_timesteps: int = 2000,
    n_skip: int = 100,
    seed: int = 42
) -> Tuple[float, np.ndarray, np.ndarray]:
    """
    NARMA-10 (Nonlinear Autoregressive Moving Average) task benchmark.
    
    The standard benchmark for testing reservoir computing:
    y(n+1) = 0.3 * y(n) + 0.05 * y(n) * sum(y(n-i)) + 1.5 * u(n-9) * u(n) + 0.1
    
    where u(n) is the input and y(n) is the output.
    
    Parameters
    ----------
    reservoir : ChaoticReservoir
        The reservoir to train
    n_timesteps : int
        Total timesteps (default 2000)
    n_skip : int
        Initial timesteps to skip (default 100 for washout)
    seed : int
        Random seed
        
    Returns
    -------
    Tuple[float, np.ndarray, np.ndarray]
        (RMSE, predictions, targets)
    """
    np.random.seed(seed)
    
    # Generate input: uniform random in [0, 0.5]
    inputs = np.random.rand(n_timesteps, reservoir.input_dim) * 0.5
    
    # Compute NARMA-10 targets
    targets = np.zeros(n_timesteps)
    delay = 10
    
    # Initialize with small values
    for n in range(delay):
        targets[n] = 0.1 * np.random.rand()
    
    # Compute NARMA targets
    for n in range(delay, n_timesteps):
        y = 0.3 * targets[n - 1]
        y += 0.05 * targets[n - 1] * sum(targets[n-delay:n])
        y += 1.5 * inputs[n - delay, 0] * inputs[n, 0]
        y += 0.1
        targets[n] = float(y)
    
    # Run reservoir on inputs
    states = reservoir.forward(inputs)
    
    # Skip initial transient
    states = states[n_skip:]
    targets_out = targets[n_skip:]
    
    # Train readout
    rmse = reservoir.train_readout(states, targets_out, regularization=1e-6)
    
    # Get predictions
    predictions = reservoir.predict(states)
    
    return rmse, predictions, targets_out
