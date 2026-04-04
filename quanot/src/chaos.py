"""
Chaos Theory Module
=================
Lyapunov exponent estimation, fractal dimension calculation,
and strange attractor generation.

CPU-optimized NumPy implementations.

References:
- Benettin et al. (1980) - Lyapunov exponent computation
- Grassberger & Procaccia (1983) - Correlation dimension
- Packard et al. (1980) - Phase space reconstruction
"""

import numpy as np
from scipy.spatial.distance import pdist, cdist
from typing import Optional, Tuple


def lyapunov_exponent_benettin(
    system_func,
    x0: np.ndarray,
    t_max: float = 100.0,
    dt: float = 0.1,
    n_vectors: int = 4,
    orthogonalize_every: int = 1
) -> np.ndarray:
    """
    Estimate Lyapunov exponent spectrum using the Benettin algorithm.
    
    The Benettin algorithm tracks the evolution of tangent vectors
    (infinitesimal perturbations) and measures their exponential growth rate.
    
    A positive maximal Lyapunov exponent (MLE) indicates chaos.
    - MLE < 0: stable, convergent
    - MLE ≈ 0: edge of chaos
    - MLE > 0: chaotic
    
    Parameters
    ----------
    system_func : callable
        Function that computes dx/dt for the dynamical system.
        Takes a state vector x (1D array) and returns derivative (1D array).
    x0 : np.ndarray
        Initial state vector
    t_max : float
        Maximum integration time
    dt : float
        Time step for integration
    n_vectors : int
        Number of tangent vectors to track (should be <= state_dim)
    orthogonalize_every : int
        How often to orthonormalize tangent vectors (1 = every step)
        
    Returns
    -------
    np.ndarray
        Lyapunov exponent spectrum, shape (n_vectors,)
        Largest exponent is in index 0.
    """
    n_steps = int(t_max / dt)
    n_dims = len(x0)
    n_vectors = min(n_vectors, n_dims)
    
    # Initialize state and tangent vectors
    x = x0.copy()
    Q = np.eye(n_dims)[:, :n_vectors]  # Tangent vectors as columns
    
    # Accumulate Lyapunov exponents
    lyapunov_sum = np.zeros(n_vectors)
    n_orthonormalizations = 0
    
    for step in range(n_steps):
        # RK4 integration of the system
        k1 = system_func(x)
        k2 = system_func(x + 0.5 * dt * k1)
        k3 = system_func(x + 0.5 * dt * k2)
        k4 = system_func(x + dt * k3)
        x = x + (dt / 6.0) * (k1 + 2*k2 + 2*k3 + k4)
        
        # Linearized evolution of tangent vectors
        # Compute Jacobian at current state numerically
        eps = 1e-8
        J = np.zeros((n_dims, n_dims))
        f_x = system_func(x)
        for i in range(n_dims):
            x_plus = x.copy()
            x_plus[i] += eps
            f_plus = system_func(x_plus)
            J[:, i] = (f_plus - f_x) / eps
        
        # Evolve tangent vectors
        Q = Q + dt * (J @ Q)
        
        # Gram-Schmidt orthonormalization every N steps
        if (step + 1) % orthogonalize_every == 0:
            Q, R = np.linalg.qr(Q)
            # Accumulate log of diagonal elements (eigenvalues of R)
            # Handle negative diagonal elements (flip sign)
            diag_signs = np.sign(np.diag(R))
            lyapunov_sum += np.log(np.abs(np.diag(R)) + 1e-16) * diag_signs
            n_orthonormalizations += 1
    
    if n_orthonormalizations > 0:
        lyapunov_exponents = lyapunov_sum / (n_orthonormalizations * dt * orthogonalize_every)
    else:
        lyapunov_exponents = np.zeros(n_vectors)
    
    return lyapunov_exponents


def maximal_lyapunov_exponent(
    system_func,
    x0: np.ndarray,
    t_max: float = 100.0,
    dt: float = 0.1,
    separation_init: float = 1e-6,
    n_separations: int = 4
) -> float:
    """
    Estimate maximal Lyapunov exponent (MLE) via direct divergence method.
    
    Simpler than Benettin - tracks divergence of nearby trajectories.
    
    Parameters
    ----------
    system_func : callable
        System dynamics function dx/dt
    x0 : np.ndarray
        Initial state
    t_max : float
        Maximum time
    dt : float
        Time step
    separation_init : float
        Initial separation between test trajectories
    n_separations : int
        Number of divergence measurements to average
        
    Returns
    -------
    float
        Estimated maximal Lyapunov exponent (nats per second)
    """
    n_steps = int(t_max / dt)
    divergences = []
    
    for _ in range(n_separations):
        # Initialize nearby state
        x = x0.copy()
        x_prime = x0 + np.random.randn(len(x0)) * separation_init
        x_prime = x0 + (x_prime - x0) / np.linalg.norm(x_prime - x0) * separation_init
        
        divergence_curve = []
        
        for step in range(n_steps):
            # RK4 integration for both trajectories
            def f(state):
                return system_func(state)
            
            for state in [x, x_prime]:
                k1 = f(state)
                k2 = f(state + 0.5 * dt * k1)
                k3 = f(state + 0.5 * dt * k2)
                k4 = f(state + dt * k3)
                state[:] = state + (dt / 6.0) * (k1 + 2*k2 + 2*k3 + k4)
            
            # Compute separation
            sep = np.linalg.norm(x_prime - x)
            divergence_curve.append(max(sep, separation_init))
            
            # Renormalize if separation gets too large
            if sep > 0.1:
                direction = (x_prime - x) / sep
                x_prime = x + direction * separation_init
        
        divergences.append(divergence_curve)
    
    # Average divergence curves
    avg_divergence = np.mean(divergences, axis=0)
    times = np.arange(len(avg_divergence)) * dt
    
    # Linear fit in log-log space (divergence ~ exp(mle * t))
    valid = avg_divergence > separation_init * 1.1
    if np.sum(valid) < 10:
        return 0.0
    
    log_times = np.log(times[valid])
    log_div = np.log(avg_divergence[valid])
    
    # MLE is the slope of log(divergence) vs time
    coeffs = np.polyfit(times[valid], log_div, 1)
    mle = coeffs[0]
    
    return mle


def correlation_dimension(
    points: np.ndarray,
    n_scales: int = 20,
    r_min: Optional[float] = None,
    r_max: Optional[float] = None
) -> Tuple[float, np.ndarray, np.ndarray]:
    """
    Estimate correlation dimension D_2 using Grassberger-Procaccia algorithm.
    
    The correlation dimension measures the fractal dimension of an attractor
    based on the scaling of correlation integrals.
    
    C(r) ~ r^D_2 for small r
    
    Parameters
    ----------
    points : np.ndarray
        State trajectory, shape (n_samples, n_dims)
    n_scales : int
        Number of radius scales to test
    r_min, r_max : float
        Min/max radii (auto-computed if None)
        
    Returns
    -------
    Tuple[float, np.ndarray, np.ndarray]
        (correlation_dimension, radii, correlation_integral)
    """
    n_samples = len(points)
    
    if n_samples < 20:
        raise ValueError(f"Need at least 20 points, got {n_samples}")
    
    # Compute pairwise distances (memory-intensive for large N)
    # Use scipy for efficiency
    dists = cdist(points, points)
    np.fill_diagonal(dists, np.inf)  # Exclude self-distances
    
    # Set radius range
    all_dists = dists.flatten()
    if r_max is None:
        r_max = float(np.percentile(all_dists, 90))
    if r_min is None:
        r_min = float(np.percentile(all_dists, 1))
    
    radii = np.logspace(np.log10(r_min), np.log10(r_max), n_scales)
    
    # Correlation integral C(r) = (2/N^2) * sum_{i<j} I(||x_i - x_j|| < r)
    C = np.zeros(n_scales)
    total_pairs = n_samples * (n_samples - 1) / 2
    
    for i, r in enumerate(radii):
        C[i] = np.sum(dists < r) / total_pairs
        C[i] = max(C[i], 1e-16)  # Avoid log(0)
    
    # Linear fit in log-log space: log(C) ~ D_2 * log(r)
    # Only use linear regime (middle portion)
    n_fit = max(n_scales // 3, 3)
    fit_start = n_scales // 3
    log_radii = np.log(radii[fit_start:fit_start + n_fit])
    log_C = np.log(C[fit_start:fit_start + n_fit])
    
    coeffs = np.polyfit(log_radii, log_C, 1)
    D_2 = coeffs[0]
    
    return D_2, radii, C


def box_counting_dimension(
    points: np.ndarray,
    n_scales: int = 20
) -> float:
    """
    Estimate box-counting dimension using the box-counting algorithm.
    
    D_0 = -lim_{eps->0} log(N(eps)) / log(eps)
    
    where N(eps) is the number of boxes needed to cover the set.
    
    Parameters
    ----------
    points : np.ndarray
        Shape (n_samples, n_dims)
    n_scales : int
        Number of box sizes to test
        
    Returns
    -------
    float
        Estimated box-counting dimension
    """
    # Normalize points to [0, 1] range
    points = points - points.min(axis=0)
    points = points / (points.max(axis=0) + 1e-16)
    
    n_samples, n_dims = points.shape
    
    # Box sizes
    epsilons = np.logspace(-3, 0, n_scales)
    counts = []
    
    for eps in epsilons:
        # Count boxes using grid approach
        # For high dimensions, this is approximate
        grid_size = (1.0 / eps).astype(int) + 1
        boxes = set()
        
        for point in points:
            # Compute grid indices
            indices = tuple((point * (grid_size - 1)).astype(int))
            boxes.add(indices)
        
        counts.append(len(boxes))
    
    counts = np.array(counts, dtype=float)
    counts = np.maximum(counts, 1)  # Avoid log(0)
    
    # Linear fit: log(N) ~ -D * log(eps)
    log_eps = np.log(epsilons)
    log_counts = np.log(counts)
    
    # Use middle portion (avoid saturation at both ends)
    n_fit = max(n_scales // 3, 5)
    fit_start = n_scales // 3
    
    coeffs = np.polyfit(log_eps[fit_start:fit_start + n_fit], 
                         log_counts[fit_start:fit_start + n_fit], 1)
    
    D_0 = -coeffs[0]  # Negative because eps decreases as N increases
    
    return D_0


def lorenz_attractor(
    x: float = 0.1,
    y: float = 0.0,
    z: float = 0.0,
    sigma: float = 10.0,
    rho: float = 28.0,
    beta: float = 8.0 / 3.0,
    dt: float = 0.01,
    steps: int = 10000
) -> np.ndarray:
    """
    Generate Lorenz attractor trajectory.
    
    Classic strange attractor exhibiting chaotic behavior.
    Parameters: sigma=10, rho=28, beta=8/3 produce the famous butterfly shape.
    
    Parameters
    ----------
    x, y, z : float
        Initial conditions
    sigma, rho, beta : float
        Lorenz system parameters
    dt : float
        Time step
    steps : int
        Number of steps
        
    Returns
    -------
    np.ndarray
        Trajectory of shape (steps, 3)
    """
    trajectory = np.zeros((steps, 3))
    trajectory[0] = [x, y, z]
    
    for i in range(1, steps):
        dx = sigma * (y - x) * dt
        dy = (x * (rho - z) - y) * dt
        dz = (x * y - beta * z) * dt
        x, y, z = x + dx, y + dy, z + dz
        trajectory[i] = [x, y, z]
    
    return trajectory


def rossler_attractor(
    x: float = 0.1,
    y: float = 0.1,
    z: float = 0.1,
    a: float = 0.2,
    b: float = 0.2,
    c: float = 5.7,
    dt: float = 0.01,
    steps: int = 10000
) -> np.ndarray:
    """
    Generate Rössler attractor trajectory.
    
    Simpler than Lorenz but with chaotic behavior.
    Best for cognitive modulation (smoother, less violent chaos).
    
    Parameters
    ----------
    x, y, z : float
        Initial conditions
    a, b, c : float
        Rössler parameters (chaotic for a=0.2, b=0.2, c=5.7)
    dt : float
        Time step
    steps : int
        Number of steps
        
    Returns
    -------
    np.ndarray
        Trajectory of shape (steps, 3)
    """
    trajectory = np.zeros((steps, 3))
    trajectory[0] = [x, y, z]
    
    for i in range(1, steps):
        dx = (-y - z) * dt
        dy = (x + a * y) * dt
        dz = (b + z * (x - c)) * dt
        x, y, z = x + dx, y + dy, z + dz
        trajectory[i] = [x, y, z]
    
    return trajectory


def henon_map(
    x: float = 0.1,
    y: float = 0.3,
    a: float = 1.4,
    b: float = 0.3,
    steps: int = 10000
) -> np.ndarray:
    """
    Generate Hénon map trajectory (discrete-time dynamical system).
    
    Simple 2D map with chaotic behavior:
    x_{n+1} = 1 - a*x_n^2 + y_n
    y_{n+1} = b*x_n
    
    Parameters
    ----------
    x, y : float
        Initial conditions
    a, b : float
        Hénon parameters (classic: a=1.4, b=0.3)
    steps : int
        Number of iterations
        
    Returns
    -------
    np.ndarray
        Trajectory of shape (steps, 2)
    """
    trajectory = np.zeros((steps, 2))
    trajectory[0] = [x, y]
    
    for i in range(1, steps):
        x_new = 1 - a * trajectory[i-1, 0]**2 + trajectory[i-1, 1]
        y_new = b * trajectory[i-1, 0]
        trajectory[i] = [x_new, y_new]
    
    return trajectory


def clifford_attractor(
    x: float = 0.1,
    y: float = 0.1,
    a: float = -1.4,
    b: float = 1.6,
    c: float = 1.0,
    d: float = 0.7,
    steps: int = 10000
) -> np.ndarray:
    """
    Generate Clifford attractor trajectory.
    
    2D map with fractal geometry:
    x_{n+1} = sin(a*y) + c*cos(a*x)
    y_{n+1} = sin(b*x) + d*cos(b*y)
    
    Parameters
    ----------
    x, y : float
        Initial conditions
    a, b, c, d : float
        Clifford attractor parameters
    steps : int
        Number of iterations
        
    Returns
    -------
    np.ndarray
        Trajectory of shape (steps, 2)
    """
    trajectory = np.zeros((steps, 2))
    trajectory[0] = [x, y]
    
    for i in range(1, steps):
        x_new = np.sin(a * trajectory[i-1, 1]) + c * np.cos(a * trajectory[i-1, 0])
        y_new = np.sin(b * trajectory[i-1, 0]) + d * np.cos(b * trajectory[i-1, 1])
        trajectory[i] = [x_new, y_new]
    
    return trajectory


def lyapunov_spectrum_rosenstein(
    trajectory: np.ndarray,
    delay: int = 1,
    max_separation: float = 0.1
) -> Tuple[float, np.ndarray]:
    """
    Estimate maximal Lyapunov exponent using Rosenstein algorithm.
    
    Phase space reconstruction using time-delay embedding.
    
    Parameters
    ----------
    trajectory : np.ndarray
        1D time series of shape (n_steps,)
    delay : int
        Time delay for embedding
    max_separation : float
        Maximum initial separation for neighbor search
        
    Returns
    -------
    Tuple[float, np.ndarray]
        (MLE, divergence_curve)
    """
    n = len(trajectory)
    
    # Time-delay embedding (Takens' theorem)
    # Create delayed coordinates
    emb_dim = 10  # Embedding dimension
    n_embedded = n - (emb_dim - 1) * delay
    
    if n_embedded < 100:
        raise ValueError(f"Trajectory too short: need at least 100 embedded points, got {n_embedded}")
    
    # Build embedding matrix
    embedded = np.zeros((n_embedded, emb_dim))
    for i in range(emb_dim):
        embedded[:, i] = trajectory[i * delay:i * delay + n_embedded]
    
    # Find nearest neighbors
    nn_indices = np.zeros(n_embedded, dtype=int)
    min_dist = np.full(n_embedded, np.inf)
    
    for i in range(n_embedded):
        for j in range(max(0, i - 50), min(n_embedded, i + 50)):
            if abs(i - j) < 2:  # Exclude temporal neighbors
                continue
            d = np.linalg.norm(embedded[i] - embedded[j])
            if d < min_dist[i]:
                min_dist[i] = d
                nn_indices[i] = j
    
    # Compute divergence curve
    n_divergence = min(1000, n_embedded // 2)
    divergence_curve = np.zeros(n_divergence)
    
    for t in range(n_divergence):
        separations = []
        for i in range(n_embedded - t - 1):
            j = nn_indices[i]
            if i + t < n_embedded and j + t < n_embedded:
                sep = np.linalg.norm(embedded[i + t] - embedded[j + t])
                separations.append(sep)
        
        if separations:
            divergence_curve[t] = np.mean(separations)
        else:
            divergence_curve[t] = 0
    
    # Linear fit for MLE
    divergence_curve = np.maximum(divergence_curve, 1e-10)
    times = np.arange(n_divergence) * delay
    
    # Use linear portion (typically middle 20-80%)
    fit_start = len(times) // 5
    fit_end = 4 * len(times) // 5
    
    if fit_end - fit_start < 10:
        return 0.0, divergence_curve
    
    coeffs = np.polyfit(times[fit_start:fit_end], 
                         np.log(divergence_curve[fit_start:fit_end]), 1)
    mle = coeffs[0]
    
    return mle, divergence_curve


class ChaoticReservoir:
    """
    Chaotic Reservoir (Echo State Network)
    ========================================
    A reservoir computer with chaotic dynamics for cognitive modeling.
    
    The reservoir maintains nonlinear temporal memory through recurrent
    connections. The key parameter is spectral radius (ρ):
    - ρ < 1: stable, fading memory
    - ρ ≈ 1: edge of chaos, maximal computational power
    - ρ > 1: chaotic, long-term memory but unstable
    
    Features:
    - Strange attractor context modulation for creative exploration
    - Real-time Lyapunov exponent monitoring
    - Adaptive spectral radius control to maintain edge-of-chaos
    - Cognitive state encoding
    
    References:
    - Jaeger & Haas (2004) - Harnessing Nonlinearity
    - Verstraeten et al. (2007) - Reservoir computing
    """
    
    def __init__(
        self,
        input_dim: int,
        reservoir_size: int = 1000,
        spectral_radius: float = 0.95,
        input_scaling: float = 0.1,
        noise_level: float = 0.001,
        connectivity: float = 0.01,
        leak_rate: float = 0.3,
        activation: str = "tanh"
    ):
        """
        Initialize the chaotic reservoir.
        
        Parameters
        ----------
        input_dim : int
            Dimension of input signals
        reservoir_size : int
            Number of reservoir neurons (N)
        spectral_radius : float
            Desired spectral radius (ρ). Controls chaos level.
        input_scaling : float
            Scaling of input weights
        noise_level : float
            Level of chaotic noise injection
        connectivity : float
            Connection probability (sparsity)
        leak_rate : float
            Leak rate for state update (α)
        activation : str
            Activation function: "tanh", "relu", "sigmoid"
        """
        self.input_dim = input_dim
        self.reservoir_size = reservoir_size
        self.spectral_radius = spectral_radius
        self.input_scaling = input_scaling
        self.noise_level = noise_level
        self.connectivity = connectivity
        self.leak_rate = leak_rate
        
        # Activation function
        if activation == "tanh":
            self.activation = np.tanh
            self.activation_deriv = lambda x: 1 - np.tanh(x)**2
        elif activation == "relu":
            self.activation = lambda x: np.maximum(0, x)
            self.activation_deriv = lambda x: (x > 0).astype(float)
        elif activation == "sigmoid":
            self.activation = lambda x: 1 / (1 + np.exp(-np.clip(x, -500, 500)))
            self.activation_deriv = lambda x: self.activation(x) * (1 - self.activation(x))
        else:
            raise ValueError(f"Unknown activation: {activation}")
        
        # Initialize weights
        self._initialize_weights()
        
        # State
        self.state = np.zeros(reservoir_size)
        self.state_history = []
        
        # Lyapunov tracking
        self._lyapunov_history = []
        self._target_lyapunov = 0.0  # Edge of chaos
        
        # Attractor context
        self.attractor_state = np.zeros(3)  # Lorenz-style
        self._attractor_type = "lorenz"
        
        # Training
        self.W_out = None
        self.is_trained = False
    
    def _initialize_weights(self):
        """Initialize input and reservoir weights."""
        # Input weights: random, dense
        self.W_in = np.random.randn(self.reservoir_size, self.input_dim) * self.input_scaling
        
        # Reservoir weights: random, sparse
        W = np.random.randn(self.reservoir_size, self.reservoir_size)
        W[np.random.rand(*W.shape) > self.connectivity] = 0
        
        # Scale to desired spectral radius
        eigvals = np.linalg.eigvals(W)
        max_eig = np.max(np.abs(eigvals))
        if max_eig > 0:
            self.W = W * (self.spectral_radius / max_eig)
        else:
            self.W = W
        
        # Store original spectral radius for adaptive control
        self._target_spectral_radius = self.spectral_radius
    
    def set_attractor_type(self, attractor_type: str):
        """
        Set the strange attractor type for context modulation.
        
        Parameters
        ----------
        attractor_type : str
            "lorenz", "rossler", or "none"
        """
        valid_types = ["lorenz", "rossler", "none"]
        if attractor_type not in valid_types:
            raise ValueError(f"Unknown attractor type: {attractor_type}")
        self._attractor_type = attractor_type
        
        # Initialize attractor state
        if attractor_type == "lorenz":
            self.attractor_state = np.array([0.1, 0.0, 0.0])
        elif attractor_type == "rossler":
            self.attractor_state = np.array([0.1, 0.1, 0.1])
    
    def _update_attractor(self, dt: float = 0.01):
        """Update the attractor context state."""
        if self._attractor_type == "none":
            return
        
        s = self.attractor_state
        ds = np.zeros(3)
        
        if self._attractor_type == "lorenz":
            sigma, rho, beta = 10.0, 28.0, 8.0 / 3.0
            ds = np.array([
                sigma * (s[1] - s[0]),
                s[0] * (rho - s[2]) - s[1],
                s[0] * s[1] - beta * s[2]
            ])
        elif self._attractor_type == "rossler":
            a, b, c = 0.2, 0.2, 5.7
            ds = np.array([
                -s[1] - s[2],
                s[0] + a * s[1],
                b + s[2] * (s[0] - c)
            ])
        
        self.attractor_state = s + dt * ds
    
    def _get_chaotic_modulation(self) -> np.ndarray:
        """Get chaotic modulation from attractor state."""
        if self._attractor_type == "none":
            # Pure noise modulation
            return self.noise_level * np.random.randn(self.reservoir_size)
        
        # Use attractor state to modulate chaos
        # The attractor provides a smooth, structured perturbation
        modulation = np.zeros(self.reservoir_size)
        
        # Project attractor 3D state to reservoir space
        # Use different components for different neuron groups
        n_groups = min(3, self.reservoir_size)
        group_size = self.reservoir_size // n_groups
        
        for i in range(n_groups):
            start = i * group_size
            end = start + group_size if i < n_groups - 1 else self.reservoir_size
            modulation[start:end] = self.attractor_state[i % 3] * 0.1
        
        # Add noise
        modulation += self.noise_level * np.random.randn(self.reservoir_size)
        
        return modulation
    
    def forward(self, input_sequence: np.ndarray) -> np.ndarray:
        """
        Process input sequence through the reservoir.
        
        Parameters
        ----------
        input_sequence : np.ndarray
            Input sequence of shape (timesteps, input_dim)
            
        Returns
        -------
        np.ndarray
            Reservoir states of shape (timesteps, reservoir_size)
        """
        n_steps = len(input_sequence)
        states = np.zeros((n_steps, self.reservoir_size))
        
        for t in range(n_steps):
            # Get input
            u = input_sequence[t]
            
            # Compute pre-activation
            pre_activation = (
                self.W_in @ u +
                self.W @ self.state
            )
            
            # Add chaotic modulation from attractor
            chaos = self._get_chaotic_modulation()
            pre_activation += chaos
            
            # Apply activation with leak
            new_state = self.activation(pre_activation)
            self.state = (1 - self.leak_rate) * self.state + self.leak_rate * new_state
            
            states[t] = self.state.copy()
            
            # Update attractor context
            self._update_attractor()
            
            # Store history (limit length for memory)
            if len(self.state_history) > 10000:
                self.state_history.pop(0)
            self.state_history.append(self.state.copy())
        
        return states
    
    def step(self, input_vector: np.ndarray) -> np.ndarray:
        """
        Process single input vector.
        
        Parameters
        ----------
        input_vector : np.ndarray
            Input of shape (input_dim,)
            
        Returns
        -------
        np.ndarray
            Reservoir state of shape (reservoir_size,)
        """
        return self.forward(input_vector.reshape(1, -1))[0]
    
    def train_readout(
        self,
        states: np.ndarray,
        targets: np.ndarray,
        regularization: float = 1e-6
    ) -> float:
        """
        Train linear readout using ridge regression.
        
        Parameters
        ----------
        states : np.ndarray
            Reservoir states (n_samples, reservoir_size)
        targets : np.ndarray
            Target outputs (n_samples, output_dim)
        regularization : float
            L2 regularization strength
            
        Returns
        -------
        float
            Training MSE
        """
        n_samples = len(states)
        
        # Ridge regression: W_out = (X^T@X + λI)^{-1} @ X^T @ Y
        XtX = states.T @ states + regularization * np.eye(self.reservoir_size)
        XtY = states.T @ targets
        
        try:
            self.W_out = np.linalg.solve(XtX, XtY)
        except np.linalg.LinAlgError:
            # Fallback to pseudoinverse
            self.W_out = np.linalg.pinv(states + 1e-6) @ targets
        
        # Compute training error
        predictions = states @ self.W_out
        mse = float(np.mean((predictions - targets) ** 2))
        
        self.is_trained = True
        return mse
    
    def predict(self, state: np.ndarray) -> np.ndarray:
        """
        Predict output from reservoir state.
        
        Parameters
        ----------
        state : np.ndarray
            Reservoir state (reservoir_size,)
            
        Returns
        -------
        np.ndarray
            Predicted output
        """
        if not self.is_trained or self.W_out is None:
            raise RuntimeError("Reservoir not trained. Call train_readout first.")
        return self.W_out.T @ state
    
    def compute_lyapunov_exponent(
        self,
        window_size: int = 500,
        dt: float = 0.01
    ) -> float:
        """
        Estimate local Lyapunov exponent from recent state history.
        
        Uses Rosenstein algorithm on state trajectory.
        
        Parameters
        ----------
        window_size : int
            Number of recent states to analyze
        dt : float
            Time step (for scaling)
            
        Returns
        -------
        float
            Estimated maximal Lyapunov exponent
        """
        if len(self.state_history) < window_size + 100:
            return 0.0
        
        # Use 1D projection (first dimension)
        trajectory = np.array(self.state_history[-window_size - 100:])
        trajectory = trajectory[:, 0]  # Project to 1D
        
        try:
            mle, _ = lyapunov_spectrum_rosenstein(trajectory, delay=1)
            return mle
        except Exception:
            return 0.0
    
    def adapt_spectral_radius(
        self,
        target_lyapunov: float = 0.0,
        adaptation_rate: float = 0.01
    ):
        """
        Adapt spectral radius to maintain target Lyapunov exponent.
        
        This keeps the reservoir at the edge of chaos.
        
        Parameters
        ----------
        target_lyapunov : float
            Target MLE (0 = edge of chaos)
        adaptation_rate : float
            Rate of adaptation
        """
        # Compute current Lyapunov exponent
        current_lyapunov = self.compute_lyapunov_exponent()
        
        # Compute adjustment
        delta = target_lyapunov - current_lyapunov
        
        # Adjust spectral radius
        self.spectral_radius *= (1 + adaptation_rate * delta)
        self.spectral_radius = np.clip(self.spectral_radius, 0.5, 1.5)
        
        # Rescale reservoir weights
        self._initialize_weights()
        
        # Store history
        self._lyapunov_history.append(current_lyapunov)
        if len(self._lyapunov_history) > 1000:
            self._lyapunov_history.pop(0)
    
    def get_chaos_level(self) -> str:
        """
        Get the current chaos level description.
        
        Returns
        -------
        str
            "stable", "edge_of_chaos", or "chaotic"
        """
        if len(self._lyapunov_history) < 10:
            return "unknown"
        
        recent_mle = np.mean(self._lyapunov_history[-10:])
        
        if recent_mle < -0.1:
            return "stable"
        elif recent_mle > 0.1:
            return "chaotic"
        else:
            return "edge_of_chaos"
    
    def get_context_vector(self) -> np.ndarray:
        """
        Get current cognitive context vector.
        
        Combines reservoir state with attractor state.
        
        Returns
        -------
        np.ndarray
            Context vector
        """
        # Normalize reservoir state
        norm_state = self.state / (np.linalg.norm(self.state) + 1e-10)
        
        # Combine with attractor state
        context = np.concatenate([
            norm_state,
            self.attractor_state
        ])
        
        return context
    
    def reset(self):
        """Reset reservoir state."""
        self.state = np.zeros(self.reservoir_size)
        self.state_history = []
        self._lyapunov_history = []
        
        # Reset attractor
        if self._attractor_type == "lorenz":
            self.attractor_state = np.array([0.1, 0.0, 0.0])
        elif self._attractor_type == "rossler":
            self.attractor_state = np.array([0.1, 0.1, 0.1])
