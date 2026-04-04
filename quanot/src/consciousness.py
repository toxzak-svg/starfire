"""
Consciousness Module
==================
Consciousness proxy metrics and architectures.

This module implements computational consciousness measures based on:
- Integrated Information Theory (IIT) proxies
- Global Workspace Theory (GWT) architecture
- Recurrence Quantification Analysis (RQA)
- Active Information Storage (AIS)

IMPORTANT: These are PROXY METRICS, not direct measures of consciousness.
The exact Φ (Integrated Information) is computationally intractable for N > ~15.
These metrics provide operationalized proxies for consciousness-like properties.

References:
- Tononi (2004) - Integrated Information Theory
- Baars (1997) - Global Workspace Theory
- Webber & Zbilut (1994) - RQA
- Lizier et al. - Active Information Storage
"""

import numpy as np
from scipy.spatial.distance import cdist
from typing import Optional, Tuple, List


# ============================================================================
# RECURRENCE QUANTIFICATION ANALYSIS (RQA)
# Primary consciousness proxy metric - computationally tractable
# ============================================================================

def recurrence_matrix(
    trajectory: np.ndarray,
    threshold: float = 0.1,
    method: str = 'euclidean',
    auto_threshold: bool = True
) -> np.ndarray:
    """
    Compute recurrence matrix from a state trajectory.
    
    A recurrence matrix R[i,j] = 1 if states i and j are "close" in state space.
    
    Parameters
    ----------
    trajectory : np.ndarray
        State trajectory of shape (n_steps, state_dim) or (n_steps,)
    threshold : float
        Distance threshold for considering states as "recurrent"
    method : str
        'euclidean' or 'cosine' distance
        
    Returns
    -------
    np.ndarray
        Binary recurrence matrix of shape (n_steps, n_steps)
    """
    if trajectory.ndim == 1:
        trajectory = trajectory.reshape(-1, 1)
    
    n_steps = len(trajectory)
    
    # Normalize trajectory
    trajectory = trajectory - trajectory.mean(axis=0)
    std_vals = trajectory.std(axis=0)
    std_vals[std_vals == 0] = 1.0  # Avoid division by zero
    trajectory = trajectory / (std_vals + 1e-10)
    
    # Compute distance matrix
    if method == 'euclidean':
        dists = cdist(trajectory, trajectory, 'euclidean')
        np.fill_diagonal(dists, np.inf)
        
        # Auto-compute threshold if needed
        if auto_threshold:
            all_dists = dists.flatten()
            all_dists = all_dists[np.isfinite(all_dists)]
            if len(all_dists) > 0:
                threshold = float(np.percentile(all_dists, 15))
    if method == 'cosine':
        # Cosine distance = 1 - cosine similarity
        norms = np.linalg.norm(trajectory, axis=1, keepdims=True)
        dists = 1 - trajectory @ trajectory.T / (norms @ norms.T + 1e-10)
    else:
        dists = cdist(trajectory, trajectory, 'euclidean')
    
    return (dists < threshold).astype(np.float32)


def rqa_metrics(recurrence_matrix: np.ndarray) -> dict:
    """
    Compute Recurrence Quantification Analysis metrics.
    
    These metrics quantify the structure of recurring patterns in a dynamical system.
    Based on Webber & Zbilut (1994).
    
    Parameters
    ----------
    recurrence_matrix : np.ndarray
        Binary recurrence matrix from recurrence_matrix()
        
    Returns
    -------
    dict
        RQA metrics:
        - REC: Recurrence - fraction of recurrent points
        - DET: Determinism - fraction of recurrent points forming diagonal lines
        - LAM: Laminarity - fraction forming vertical/horizontal lines
        - TT: Trapping Time - average length of vertical lines
        - L_max: Maximum diagonal line length
        - ENTR: Entropy of diagonal line lengths
        - FD: Fractal dimension estimate from line statistics
    """
    R = recurrence_matrix
    n = len(R)
    
    # Exclude main diagonal (self-recurrence)
    R_no_diag = R.copy()
    np.fill_diagonal(R_no_diag, 0)
    
    # REC: Recurrence rate
    total_possible = n * (n - 1)
    n_recurrent = np.sum(R_no_diag)
    REC = n_recurrent / total_possible if total_possible > 0 else 0.0
    
    # Find diagonal line structures
    diag_lengths = []
    for d in range(1, n):  # Diagonal offset
        diag = np.diag(R_no_diag, k=d)
        current_length = 0
        for val in diag:
            if val > 0.5:
                current_length += 1
            else:
                if current_length > 0:
                    diag_lengths.append(current_length)
                current_length = 0
        if current_length > 0:
            diag_lengths.append(current_length)
    
    # DET: Determinism (fraction of recurrence in diagonal lines)
    if n_recurrent > 0 and len(diag_lengths) > 0:
        DET = sum(diag_lengths) / n_recurrent
        L_max = max(diag_lengths) if diag_lengths else 0
        mean_diag_length = np.mean(diag_lengths) if diag_lengths else 0
        
        # Entropy of diagonal line lengths
        diag_counts = np.bincount(diag_lengths, minlength=L_max+1)[1:]
        diag_probs = diag_counts[diag_counts > 0] / sum(diag_counts)
        ENTR = -np.sum(diag_probs * np.log(diag_probs + 1e-16))
    else:
        DET = 0.0
        L_max = 0
        mean_diag_length = 0
        ENTR = 0.0
    
    # Find vertical/horizontal line structures (same as diagonal for square matrix)
    vert_lengths = []
    for col in range(n):
        current_length = 0
        for row in range(n):
            if R_no_diag[row, col] > 0.5:
                current_length += 1
            else:
                if current_length > 0:
                    vert_lengths.append(current_length)
                current_length = 0
        if current_length > 0:
            vert_lengths.append(current_length)
    
    # LAM: Laminarity (fraction of recurrence in vertical lines)
    if n_recurrent > 0 and len(vert_lengths) > 0:
        LAM = sum(vert_lengths) / n_recurrent
        TT = np.mean(vert_lengths) if vert_lengths else 0  # Trapping time
    else:
        LAM = 0.0
        TT = 0.0
    
    # FD: Fractal dimension estimate from RQA
    # Capacity dimension approximation: D_0 ≈ log(N) / log(1/r)
    # Using line statistics instead
    if L_max > 0 and REC > 0:
        FD = np.log(sum(diag_lengths)) / np.log(L_max * n * REC)
    else:
        FD = 0.0
    
    return {
        'REC': REC,
        'DET': DET,
        'LAM': LAM,
        'TT': TT,
        'L_max': L_max,
        'mean_diag_length': mean_diag_length,
        'ENTR': ENTR,
        'FD': FD
    }


def moving_rqa(
    trajectory: np.ndarray,
    window_size: int = 200,
    hop_size: int = 50,
    threshold: float = 0.1
) -> Tuple[dict, np.ndarray]:
    """
    Compute RQA metrics over a sliding window.
    
    Parameters
    ----------
    trajectory : np.ndarray
        Full state trajectory
    window_size : int
        Size of each window
    hop_size : int
        Step size between windows
    threshold : float
        RQA threshold
        
    Returns
    -------
    Tuple[dict, np.ndarray]
        (time_series of metrics, window_centers)
    """
    n = len(trajectory)
    centers = np.arange(window_size//2, n - window_size//2, hop_size)
    
    rec_series = []
    det_series = []
    lam_series = []
    
    for center in centers:
        start = max(0, center - window_size//2)
        end = min(n, center + window_size//2)
        window = trajectory[start:end]
        
        R = recurrence_matrix(window, threshold)
        metrics = rqa_metrics(R)
        
        rec_series.append(metrics['REC'])
        det_series.append(metrics['DET'])
        lam_series.append(metrics['LAM'])
    
    time_series = {
        'REC': np.array(rec_series),
        'DET': np.array(det_series),
        'LAM': np.array(lam_series)
    }
    
    return time_series, centers


# ============================================================================
# ACTIVE INFORMATION STORAGE (AIS)
# Measures how much past states inform future states
# ============================================================================

def active_information_storage(
    states: np.ndarray,
    k: int = 3
) -> float:
    """
    Estimate Active Information Storage (AIS).
    
    AIS measures the amount of information that past states provide
    about future states in a process.
    
    AIS = average mutual information between past and future states.
    
    Parameters
    ----------
    states : np.ndarray
        State trajectory of shape (n_steps, state_dim) or (n_steps,)
    k : int
        Embedding dimension for state space reconstruction
        
    Returns
    -------
    float
        AIS in bits
    """
    if states.ndim == 1:
        states = states.reshape(-1, 1)
    
    n_steps = len(states)
    
    if n_steps < k * 10:
        return 0.0
    
    # Build embedding vectors (Takens-like)
    def embed(state_seq, k):
        """Create k-dimensional embedding."""
        n = len(state_seq) - k + 1
        embedded = np.zeros((n, k))
        for i in range(k):
            embedded[:, i] = state_seq[i:i+n]
        return embedded
    
    # Use first dimension for simplicity
    state_1d = states[:, 0]
    
    # Past states: x_0^{n-k}
    past = embed(state_1d[:-1], k)
    # Future states: x_{k}^{n}
    future = state_1d[k:]
    
    # Normalize
    past = past - past.mean(axis=0)
    past = past / (past.std(axis=0) + 1e-10)
    
    # Compute mutual information approximation using k-NN
    # Find k-nearest neighbors in past space
    from scipy.spatial import KDTree
    
    tree = KDTree(past)
    
    ais_total = 0.0
    for i in range(len(future)):
        # Distance to k nearest past states
        _, indices = tree.query(past[i], k=min(k, len(past)))
        
        # Estimate local density
        if len(indices) > 1:
            nn_dists = np.linalg.norm(past[indices] - past[i], axis=1)
            nn_dists = np.maximum(nn_dists, 1e-10)
            
            # Local probability estimate
            p_local = 1.0 / (nn_dists[-1] ** (states.shape[1] if states.ndim > 1 else 1))
            
            # Simple AIS estimate
            ais_total += np.log(p_local + 1e-10)
    
    ais = ais_total / len(future)
    
    return max(ais, 0.0)


# ============================================================================
# GLOBAL WORKSPACE SIMULATION (GWT)
# Based on Baars (1988) and Franklin & 'IDA' agent
# ============================================================================

class GlobalWorkspace:
    """
    Simplified Global Workspace simulation.
    
    The Global Workspace is a bottleneck for conscious attention.
    Information that enters the workspace is "conscious" and broadcast
    to all other modules.
    
    Based on Bernard Baars' Global Workspace Theory (1997).
    """
    
    def __init__(
        self,
        n_modules: int = 8,
        workspace_capacity: int = 3,
        broadcast_decay: float = 0.9,
        attention_threshold: float = 0.6
    ):
        self.n_modules = n_modules
        self.workspace_capacity = workspace_capacity
        self.broadcast_decay = broadcast_decay
        self.attention_threshold = attention_threshold
        
        # Module states
        self.module_activities = np.zeros(n_modules)
        self.module_importance = np.zeros(n_modules)
        
        # Workspace content
        self.workspace_content: List[int] = []
        self.conscious_history: List[List[int]] = []
        
        # Broadcast history
        self.broadcast_log: List[dict] = []
        
    def step(
        self,
        inputs: np.ndarray,
        module_activations: Optional[np.ndarray] = None
    ) -> dict:
        """
        One step of global workspace processing.
        
        Parameters
        ----------
        inputs : np.ndarray
            External inputs to each module, shape (n_modules,)
        module_activations : np.ndarray
            Internal activation levels for each module, shape (n_modules,)
            
        Returns
        -------
        dict
            Step results including workspace content and consciousness metrics
        """
        if len(inputs) != self.n_modules:
            raise ValueError(f"Expected {self.n_modules} inputs, got {len(inputs)}")
        
        # Update module activities
        if module_activations is not None:
            self.module_importance = module_activations
        else:
            self.module_importance = np.abs(inputs)
        
        # Competition for workspace access
        competition_scores = self.module_activities + self.module_importance * 0.5
        
        # Select top-k for workspace entry
        sorted_indices = np.argsort(competition_scores)[::-1]
        winners = sorted_indices[:self.workspace_capacity].tolist()
        
        # Broadcast winners to all modules
        for w in winners:
            # Modules receive broadcast information
            self.module_activities *= self.broadcast_decay
            self.module_activities += 0.1 * (w == np.arange(self.n_modules))
        
        # Update workspace
        self.workspace_content = winners
        
        # Record history
        self.conscious_history.append(winners.copy())
        
        # Compute consciousness metrics for this step
        consciousness_level = len(winners) / self.n_modules
        broadcast_diversity = len(set(winners)) / self.n_modules
        
        result = {
            'workspace_content': winners,
            'consciousness_level': consciousness_level,
            'broadcast_diversity': broadcast_diversity,
            'module_activities': self.module_activities.copy(),
            'competition_scores': competition_scores.copy()
        }
        
        self.broadcast_log.append(result)
        
        return result
    
    def get_conscious_attention(self) -> List[int]:
        """Get currently conscious content (workspace snapshot)."""
        return self.workspace_content.copy()
    
    def consciousness_summary(self) -> dict:
        """
        Summarize overall consciousness metrics across history.
        
        Returns
        -------
        dict
            Summary statistics
        """
        if not self.conscious_history:
            return {'avg_consciousness': 0.0, 'n_broadcasts': 0}
        
        consciousness_levels = [
            len(content) / self.n_modules 
            for content in self.conscious_history
        ]
        
        # Compute recurrence of workspace content
        all_content = []
        for content in self.conscious_history[-100:]:  # Last 100 broadcasts
            all_content.extend(content)
        
        content_counts = np.bincount(all_content, minlength=self.n_modules)
        
        return {
            'avg_consciousness': np.mean(consciousness_levels),
            'std_consciousness': np.std(consciousness_levels),
            'n_broadcasts': len(self.conscious_history),
            'most_common_content': np.argsort(content_counts)[::-1][:3].tolist(),
            'content_diversity': np.sum(content_counts > 0) / self.n_modules
        }


# ============================================================================
# INTEGRATED INFORMATION PROXY (Φ^G - Geometric approximation)
# Based on Oizumi et al. (2016)
# ============================================================================

def geometric_integrated_information(
    states: np.ndarray,
    n_partitions: int = 10
) -> float:
    """
    Approximate integrated information using geometric method.
    
    Φ^G = 1 - D(p || p_i ⊗ p_j) where D is Jensen-Shannon divergence
    
    This is a proxy for the true Φ (which is intractable).
    Lower values = more integrated.
    
    Parameters
    ----------
    states : np.ndarray
        State trajectory, shape (n_steps, n_elements)
    n_partitions : int
        Number of random partitions to average over
        
    Returns
    -------
    float
        Geometric integrated information proxy (lower = more integrated)
    """
    if states.ndim == 1:
        return 0.0
    
    n_steps, n_elements = states.shape
    
    if n_elements < 2:
        return 0.0
    
    # Discretize states (binary thresholding)
    states_binary = (states > states.mean(axis=0)).astype(float)
    
    # Compute marginal distributions
    p_i = np.mean(states_binary, axis=0)  # Shape (n_elements,)
    p_i = np.clip(p_i, 1e-10, 1 - 1e-10)
    
    # Joint distribution (approximate)
    p_joint_approx = np.mean(states_binary, axis=1)  # Very rough approximation
    
    # Partition the system and compare
    phi_values = []
    
    for _ in range(n_partitions):
        # Random partition
        partition = np.random.rand(n_elements) > 0.5
        part_a = states_binary[:, partition]
        part_b = states_binary[:, ~partition]
        
        if part_a.shape[1] == 0 or part_b.shape[1] == 0:
            continue
        
        # Marginal distributions
        p_a = np.mean(part_a, axis=0)
        p_b = np.mean(part_b, axis=0)
        
        # Simple approximation of integrated information
        # Compare product of marginals to actual joint
        p_product = np.outer(p_a, p_b).flatten()
        p_joint_flat = np.zeros_like(p_product)
        
        # Actual joint (simplified)
        for i in range(min(len(p_joint_flat), n_steps)):
            idx = int(part_a[i].sum() * part_b.shape[1] + part_b[i].sum())
            if idx < len(p_joint_flat):
                p_joint_flat[idx] += 1
        
        p_joint_flat = p_joint_flat / (p_joint_flat.sum() + 1e-10)
        p_product = np.clip(p_product, 1e-10, 1 - 1e-10)
        p_joint_flat = np.clip(p_joint_flat, 1e-10, 1 - 1e-10)
        
        # Jensen-Shannon divergence
        m = 0.5 * (p_product + p_joint_flat)
        js = 0.5 * np.sum(p_product * np.log(p_product / m)) + \
             0.5 * np.sum(p_joint_flat * np.log(p_joint_flat / m))
        
        phi_values.append(js)
    
    return float(np.mean(phi_values)) if phi_values else 0.0


# ============================================================================
# CONSCIOUSNESS PROXY SUITE
# ============================================================================

def consciousness_proxy_suite(
    trajectory: np.ndarray,
    window_size: int = 500
) -> dict:
    """
    Compute all consciousness proxy metrics for a trajectory.
    
    This is the main entry point for consciousness assessment.
    
    Parameters
    ----------
    trajectory : np.ndarray
        State trajectory of shape (n_steps, state_dim)
    window_size : int
        Window size for RQA computation
        
    Returns
    -------
    dict
        All consciousness proxy metrics
    """
    if trajectory.ndim == 1:
        trajectory = trajectory.reshape(-1, 1)
    
    results = {}
    
    # 1. RQA metrics
    R = recurrence_matrix(trajectory[-window_size:], threshold=0.1)
    rqa = rqa_metrics(R)
    results['rqa'] = rqa
    
    # 2. Active Information Storage
    ais = active_information_storage(trajectory[-window_size:])
    results['ais'] = ais
    
    # 3. Geometric Integrated Information
    phi_geo = geometric_integrated_information(trajectory[-window_size:])
    results['phi_geo_proxy'] = phi_geo
    
    # 4. Summary interpretation
    results['interpretation'] = {
        'recurrence': 'high' if rqa['REC'] > 0.1 else 'low',
        'determinism': 'high' if rqa['DET'] > 0.5 else 'low',
        'integrated_information_proxy': 'high' if phi_geo < 0.1 else 'low',
        'ais_level': 'high' if ais > 1.0 else 'low'
    }
    
    return results
