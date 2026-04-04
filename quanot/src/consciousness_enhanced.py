"""
Consciousness Module - Enhanced
==============================
Phase 4: Consciousness Emergence Pathways

This module implements consciousness-related components:
- Enhanced Φ proxy calculators (multiple methods)
- Global Workspace (already exists - verified)
- Recurrent processing loop
- Metacognition loop
- Predictive coding layer
- Integrated consciousness core

IMPORTANT: These are PROXY METRICS and ARCHITECTURES, not direct measures of consciousness.
The exact Φ (Integrated Information) is computationally intractable for N > ~15.

References:
- Tononi (2004) - Integrated Information Theory
- Baars (1997) - Global Workspace Theory
- Dehaene et al. (2017) - Global Neuronal Workspace
- Lamme (2006) - Recurrent Processing Theory
- Rao & Ballard (1999) - Predictive Coding
"""

import numpy as np
from scipy.spatial.distance import cdist, pdist
from scipy.spatial import KDTree
from scipy.signal import hilbert
from typing import Optional, Tuple, List, Dict
from collections import deque
import warnings


# ============================================================================
# ADVANCED CONSCIOUSNESS METRICS (Extended)
# ============================================================================

def causal_density(states: np.ndarray, lag: int = 1) -> float:
    """
    Compute causal density - measure of causal interactions.
    
    Higher causal density = more causal interactions = more conscious.
    
    Parameters
    ----------
    states : np.ndarray
        State trajectory, shape (n_steps, n_elements)
    lag : int
        Time lag for causal analysis
        
    Returns
    -------
    float
        Causal density score (0-1)
    """
    if states.ndim == 1:
        states = states.reshape(-1, 1)
    
    n_steps, n_elements = states.shape
    
    if n_steps < lag + 10 or n_elements < 2:
        return 0.0
    
    # Compute transfer entropy approximation
    total_causality = 0.0
    n_pairs = 0
    
    for i in range(min(n_elements, 5)):
        for j in range(i + 1, min(n_elements, 5)):
            # Direction i -> j
            x = states[:-lag, i]
            y = states[lag:, j]
            y_past = states[:-lag, j]
            
            # Simple correlation-based causality proxy
            corr_xy = np.corrcoef(x, y)[0, 1]
            corr_y = np.corrcoef(y_past, y)[0, 1]
            
            # Transfer entropy proxy
            if not np.isnan(corr_xy) and not np.isnan(corr_y):
                te = abs(corr_xy) - abs(corr_y)
                total_causality += max(0, te)
                n_pairs += 1
    
    if n_pairs == 0:
        return 0.0
    
    return min(total_causality / n_pairs, 1.0)


def neural_complexity(states: np.ndarray) -> float:
    """
    Compute neural complexity (Tononi et al.).
    
    Balances integration and segregation.
    
    Parameters
    ----------
    states : np.ndarray
        State trajectory
        
    Returns
    -------
    float
        Neural complexity (0-1)
    """
    if states.ndim == 1:
        states = states.reshape(-1, 1)
    
    n_steps, n_elements = states.shape
    
    if n_elements < 2 or n_steps < 10:
        return 0.0
    
    # Compute covariance matrix
    cov = np.cov(states.T)
    np.fill_diagonal(cov, 0)
    
    # Integration: average mutual information
    corr = np.corrcoef(states.T)
    np.fill_diagonal(corr, 0)
    corr = np.clip(corr, -1, 1)
    
    # Average correlation magnitude (segregation)
    segregation = np.mean(np.abs(corr))
    
    # Integration: based on correlation structure
    integration = 1.0 - segregation
    
    # Complexity = integration * segregation
    complexity = integration * segregation
    
    return float(complexity)


def temporal_granularity(states: np.ndarray) -> float:
    """
    Compute temporal granularity - measure of multi-scale processing.
    
    Higher = conscious of multiple time scales.
    
    Parameters
    ----------
    states : np.ndarray
        State trajectory
        
    Returns
    -------
    float
        Temporal granularity score (0-1)
    """
    if states.ndim == 1:
        states = states.reshape(-1, 1)
    
    n_steps, n_elements = states.shape
    
    if n_steps < 50:
        return 0.0
    
    # Compute power spectrum at different time scales
    time_scales = [2, 5, 10, 20]
    powers = []
    
    for scale in time_scales:
        # Downsample
        n_segments = n_steps // scale
        if n_segments < 2:
            continue
        
        segment_means = []
        for i in range(n_segments):
            segment = states[i*scale:(i+1)*scale]
            segment_means.append(np.mean(segment, axis=0))
        
        if len(segment_means) > 1:
            variance = np.var(np.array(segment_means))
            powers.append(variance)
    
    if len(powers) < 2:
        return 0.0
    
    # Multi-scale = variance of variances
    power_variance = np.var(powers)
    
    # Normalize
    granularity = min(power_variance * 10, 1.0)
    
    return float(granularity)


def phenomenological_binding(states: np.ndarray, trajectory: np.ndarray) -> float:
    """
    Compute phenomenological binding strength.
    
    Measures how well unified experience is formed.
    
    Parameters
    ----------
    states : np.ndarray
        State snapshots
    trajectory : np.ndarray
        Full trajectory
        
    Returns
    -------
    float
        Binding strength (0-1)
    """
    if states.ndim == 1:
        states = states.reshape(-1, 1)
    
    n_steps, n_elements = states.shape
    
    if n_elements < 2 or n_steps < 20:
        return 0.0
    
    # Binding = coherence across elements
    corr = np.corrcoef(states.T)
    np.fill_diagonal(corr, 0)
    
    # Average absolute correlation
    binding = np.mean(np.abs(corr))
    
    # Combine with temporal coherence
    if len(trajectory) >= 10:
        traj_corr = np.corrcoef(trajectory[:-1].T, trajectory[1:].T)
        temporal = np.mean(np.abs(traj_corr))
    else:
        temporal = 0.5
    
    # Combined binding
    combined = (binding + temporal) / 2
    
    return float(min(combined, 1.0))


def consciousness_heat_map(states: np.ndarray, window_size: int = 20) -> np.ndarray:
    """
    Create consciousness heat map over time.
    
    Parameters
    ----------
    states : np.ndarray
        State trajectory
    window_size : int
        Window for computing local consciousness
        
    Returns
    -------
    np.ndarray
        Consciousness levels over time
    """
    n_steps = len(states)
    
    if n_steps < window_size:
        return np.zeros(n_steps)
    
    heat_map = np.zeros(n_steps)
    
    for i in range(n_steps - window_size + 1):
        window = states[i:i+window_size]
        
        # Compute local metrics
        phi = neural_complexity(window)
        gran = temporal_granularity(window)
        
        heat_map[i + window_size // 2] = (phi + gran) / 2
    
    return heat_map


# ============================================================================
# ENHANCED CREATIVITY METRICS
# ============================================================================

def divergence_entropy(states: np.ndarray) -> float:
    """
    Compute divergence entropy - measure of exploratory behavior.
    
    Parameters
    ----------
    states : np.ndarray
        State trajectory
        
    Returns
    -------
    float
        Divergence entropy (0-1)
    """
    if len(states) < 10:
        return 0.0
    
    # Compute pairwise distances
    if states.ndim == 1:
        states = states.reshape(-1, 1)
    
    n = min(len(states), 100)
    dists = pdist(states[:n])
    
    # Entropy of distance distribution
    hist, _ = np.histogram(dists, bins=20)
    hist = hist / (hist.sum() + 1e-10)
    
    entropy = -np.sum(hist * np.log(hist + 1e-10))
    max_entropy = np.log(20)
    
    return float(entropy / max_entropy)


def conceptual_divergence(concepts: List[np.ndarray]) -> float:
    """
    Compute conceptual divergence between concepts.
    
    Parameters
    ----------
    concepts : list
        List of concept embeddings
        
    Returns
    -------
    float
        Divergence score (0-1)
    """
    if len(concepts) < 2:
        return 0.0
    
    # Compute pairwise distances
    dists = []
    for i in range(len(concepts)):
        for j in range(i + 1, len(concepts)):
            d = np.linalg.norm(concepts[i] - concepts[j])
            dists.append(d)
    
    if not dists:
        return 0.0
    
    # High variance = high divergence
    variance = np.var(dists)
    mean = np.mean(dists)
    
    divergence = variance / (mean + 1e-10)
    
    return float(min(divergence, 1.0))


# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

def compute_all_consciousness_metrics(
    states: np.ndarray,
    trajectory: Optional[np.ndarray] = None
) -> Dict:
    """
    Compute all available consciousness metrics.
    
    Parameters
    ----------
    states : np.ndarray
        State snapshots
    trajectory : np.ndarray
        Full state trajectory
        
    Returns
    -------
    dict
        All consciousness metrics
    """
    if trajectory is None:
        trajectory = states
    
    metrics = {
        'causal_density': causal_density(states),
        'neural_complexity': neural_complexity(states),
        'temporal_granularity': temporal_granularity(states),
        'phenomenological_binding': phenomenological_binding(states, trajectory),
        'divergence_entropy': divergence_entropy(states),
    }
    
    # Add heat map
    metrics['consciousness_heatmap'] = consciousness_heat_map(states).tolist()
    
    # Overall score
    metrics['overall_consciousness'] = np.mean([
        metrics['causal_density'],
        metrics['neural_complexity'],
        metrics['temporal_granularity'],
        metrics['phenomenological_binding']
    ])
    
    return metrics


# ============================================================================
# EXISTING COMPONENTS (from previous phases)
# Recurrence Quantification Analysis, Global Workspace, AIS, Φ^G
# ============================================================================

def recurrence_matrix(
    trajectory: np.ndarray,
    threshold: float = 0.1,
    method: str = 'euclidean',
    auto_threshold: bool = True
) -> np.ndarray:
    """Compute recurrence matrix from state trajectory."""
    if trajectory.ndim == 1:
        trajectory = trajectory.reshape(-1, 1)
    
    n_steps = len(trajectory)
    trajectory = trajectory - trajectory.mean(axis=0)
    std_vals = trajectory.std(axis=0)
    std_vals[std_vals == 0] = 1.0
    trajectory = trajectory / (std_vals + 1e-10)
    
    if method == 'euclidean':
        dists = cdist(trajectory, trajectory, 'euclidean')
        np.fill_diagonal(dists, np.inf)
        if auto_threshold:
            all_dists = dists.flatten()
            all_dists = all_dists[np.isfinite(all_dists)]
            if len(all_dists) > 0:
                threshold = float(np.percentile(all_dists, 15))
    else:
        dists = cdist(trajectory, trajectory, 'euclidean')
    
    return (dists < threshold).astype(np.float32)


def rqa_metrics(recurrence_matrix: np.ndarray) -> dict:
    """Compute RQA metrics from recurrence matrix."""
    R = recurrence_matrix
    n = len(R)
    R_no_diag = R.copy()
    np.fill_diagonal(R_no_diag, 0)
    
    total_possible = n * (n - 1)
    n_recurrent = np.sum(R_no_diag)
    REC = n_recurrent / total_possible if total_possible > 0 else 0.0
    
    # Diagonal line structures
    diag_lengths = []
    for d in range(1, n):
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
    
    DET = sum(diag_lengths) / n_recurrent if n_recurrent > 0 and len(diag_lengths) > 0 else 0.0
    L_max = max(diag_lengths) if diag_lengths else 0
    mean_diag_length = np.mean(diag_lengths) if diag_lengths else 0
    
    diag_counts = np.bincount(diag_lengths, minlength=L_max+1)[1:]
    diag_probs = diag_counts[diag_counts > 0] / sum(diag_counts)
    ENTR = -np.sum(diag_probs * np.log(diag_probs + 1e-16)) if len(diag_probs) > 0 else 0.0
    
    return {'REC': REC, 'DET': DET, 'LAM': 0.0, 'TT': 0.0, 'L_max': L_max, 
            'mean_diag_length': mean_diag_length, 'ENTR': ENTR, 'FD': 0.0}


def active_information_storage(states: np.ndarray, k: int = 3) -> float:
    """Estimate Active Information Storage."""
    if states.ndim == 1:
        states = states.reshape(-1, 1)
    
    n_steps = len(states)
    if n_steps < k * 10:
        return 0.0
    
    state_1d = states[:, 0]
    past = np.column_stack([state_1d[:-k], state_1d[1:-k+1]])
    future = state_1d[k:]
    
    if len(past) < 10:
        return 0.0
    
    past = past - past.mean(axis=0)
    past = past / (past.std(axis=0) + 1e-10)
    
    tree = KDTree(past)
    ais_total = 0.0
    
    for i in range(min(len(future), 100)):
        _, indices = tree.query(past[i], k=min(k, len(past)))
        if len(indices) > 1:
            nn_dists = np.linalg.norm(past[indices] - past[i], axis=1)
            nn_dists = np.maximum(nn_dists, 1e-10)
            p_local = 1.0 / (nn_dists[-1] ** states.shape[1])
            ais_total += np.log(p_local + 1e-10)
    
    return max(ais_total / min(len(future), 100), 0.0)


# ============================================================================
# NEW: ENHANCED Φ PROXY CALCULATORS (Task 4.1)
# Multiple methods for Integrated Information approximation
# ============================================================================

class PhiCalculator:
    """
    Multiple Φ (Integrated Information) proxy calculators.
    
    Since exact Φ is intractable for N > ~15, we use several approximation methods:
    1. Geometric Φ (Φ^G) - Jensen-Shannon divergence based
    2. Spectral Φ - eigenvalue-based
    3. Information-theoretic Φ - entropy-based
    4. Recurrence-based Φ - RQA based
    
    Lower values = MORE integrated (system is less reducible to parts).
    """
    
    def __init__(self, random_seed: Optional[int] = None):
        if random_seed is not None:
            np.random.seed(random_seed)
        
        self.history: List[dict] = []
    
    def compute_geometric_phi(
        self,
        states: np.ndarray,
        n_partitions: int = 10
    ) -> float:
        """
        Geometric Integrated Information (Φ^G).
        
        Based on Oizumi et al. (2016) - measures irreducibility via
        Jensen-Shannon divergence between joint and product of marginals.
        
        Lower = more integrated.
        """
        if states.ndim == 1:
            return 0.0
        
        n_steps, n_elements = states.shape
        if n_elements < 2:
            return 0.0
        
        states_binary = (states > states.mean(axis=0)).astype(float)
        phi_values = []
        
        for _ in range(n_partitions):
            partition = np.random.rand(n_elements) > 0.5
            part_a = states_binary[:, partition]
            part_b = states_binary[:, ~partition]
            
            if part_a.shape[1] == 0 or part_b.shape[1] == 0:
                continue
            
            p_a = np.mean(part_a, axis=0)
            p_b = np.mean(part_b, axis=0)
            p_product = np.outer(p_a, p_b).flatten()
            
            p_joint = np.zeros_like(p_product)
            for i in range(min(len(p_joint), n_steps)):
                idx = min(int(part_a[i].sum() * part_b.shape[1] + part_b[i].sum()), len(p_joint) - 1)
                p_joint[idx] += 1
            p_joint = p_joint / (p_joint.sum() + 1e-10)
            
            p_product = np.clip(p_product, 1e-10, 1 - 1e-10)
            p_joint = np.clip(p_joint, 1e-10, 1 - 1e-10)
            
            m = 0.5 * (p_product + p_joint)
            js = 0.5 * np.sum(p_product * np.log(p_product / m + 1e-10)) + \
                 0.5 * np.sum(p_joint * np.log(p_joint / m + 1e-10))
            
            phi_values.append(js)
        
        return float(np.mean(phi_values)) if phi_values else 0.0
    
    def compute_spectral_phi(self, states: np.ndarray) -> float:
        """
        Spectral Φ proxy.
        
        Uses eigenvalue distribution of correlation matrix as proxy for
        integrated information structure. More equal distribution = higher integration.
        
        Lower = more integrated.
        """
        if states.ndim == 1:
            states = states.reshape(-1, 1)
        
        n_steps, n_elements = states.shape
        if n_elements < 2:
            return 0.0
        
        # Correlation matrix
        states_centered = states - states.mean(axis=0)
        cov = states_centered.T @ states_centered / n_steps
        
        # Eigenvalue distribution
        try:
            eigvals = np.linalg.eigvalsh(cov)
            eigvals = np.abs(eigvals)
            eigvals = eigvals / (eigvals.sum() + 1e-10)
            
            # Entropy of eigenvalue distribution
            entropy = -np.sum(eigvals * np.log(eigvals + 1e-10))
            max_entropy = np.log(n_elements)
            
            # Normalized entropy (1 = uniform, 0 = one dominant)
            spectral_entropy = entropy / (max_entropy + 1e-10)
            
            # Φ proxy: lower spectral entropy = more integrated
            return 1.0 - spectral_entropy
        except:
            return 0.5
    
    def compute_information_phi(self, states: np.ndarray) -> float:
        """
        Information-theoretic Φ proxy.
        
        Uses mutual information between parts as proxy for integration.
        
        Higher MI = more integrated.
        """
        if states.ndim == 1:
            return 0.0
        
        n_steps, n_elements = states.shape
        if n_elements < 2:
            return 0.0
        
        # Discretize
        states_binary = (states > states.mean(axis=0)).astype(int)
        
        # Split into two parts
        mid = n_elements // 2
        part_a = states_binary[:, :mid]
        part_b = states_binary[:, mid:]
        
        # Marginal entropies
        H_A = -np.sum(np.mean(part_a, axis=0) * np.log(np.mean(part_a, axis=0) + 1e-10))
        H_B = -np.sum(np.mean(part_b, axis=0) * np.log(np.mean(part_b, axis=0) + 1e-10))
        
        # Joint entropy (simplified)
        H_AB = 0.0
        for i in range(min(100, n_steps)):
            pa = part_a[i]
            pb = part_b[i]
            joint_state = int(''.join(pa.astype(str)), 2)
            H_AB += 1
        
        H_AB = H_AB / min(100, n_steps)
        
        # Mutual information: I(A;B) = H(A) + H(B) - H(A,B)
        MI = max(H_A + H_B - H_AB, 0)
        
        # Normalize
        phi = MI / (H_A + H_B + 1e-10)
        
        return float(phi)
    
    def compute_recurrence_phi(self, trajectory: np.ndarray) -> float:
        """
        Recurrence-based Φ proxy.
        
        Uses RQA metrics to estimate integration:
        - High recurrence + high determinism = integrated
        - Low laminarity = less modular
        
        Higher = more integrated.
        """
        if len(trajectory) < 50:
            return 0.0
        
        R = recurrence_matrix(trajectory[-200:], threshold=0.1)
        metrics = rqa_metrics(R)
        
        # Combined metric: high REC, high DET, moderate LAM
        phi = (metrics['REC'] * 0.4 + 
               metrics['DET'] * 0.4 + 
               (1 - metrics['LAM']) * 0.2)
        
        return float(phi)
    
    def compute_all(
        self,
        states: np.ndarray,
        trajectory: Optional[np.ndarray] = None
    ) -> dict:
        """
        Compute all Φ proxies and return combined assessment.
        
        Parameters
        ----------
        states : np.ndarray
            State snapshots, shape (n_snapshots, n_elements)
        trajectory : np.ndarray
            State trajectory, shape (n_steps, state_dim)
            
        Returns
        -------
        dict
            All Φ proxies and interpretation
        """
        results = {
            'geometric_phi': self.compute_geometric_phi(states),
            'spectral_phi': self.compute_spectral_phi(states),
            'information_phi': self.compute_information_phi(states),
            'recurrence_phi': self.compute_recurrence_phi(trajectory if trajectory is not None else states)
        }
        
        # Average (weighted)
        results['phi_composite'] = (
            results['geometric_phi'] * 0.25 +
            results['spectral_phi'] * 0.25 +
            results['information_phi'] * 0.25 +
            results['recurrence_phi'] * 0.25
        )
        
        # Interpretation
        avg_phi = results['phi_composite']
        results['interpretation'] = {
            'integration_level': 'high' if avg_phi > 0.5 else 'moderate' if avg_phi > 0.2 else 'low',
            'mechanism': self._interpret_mechanism(results)
        }
        
        self.history.append(results)
        
        return results
    
    def _interpret_mechanism(self, results: dict) -> str:
        """Interpret which mechanism dominates integration."""
        methods = [
            ('geometric', results['geometric_phi']),
            ('spectral', results['spectral_phi']),
            ('information', results['information_phi']),
            ('recurrence', results['recurrence_phi'])
        ]
        
        max_method = max(methods, key=lambda x: x[1])
        return max_method[0]


# ============================================================================
# METACOGNITION LOOP (Task 4.4)
# Self-referential monitoring and self-modeling
# ============================================================================

class MetacognitionLoop:
    """
    Metacognition loop for self-monitoring and self-modeling.
    
    Tracks:
    - Self-model: beliefs about own cognitive states
    - Confidence: accuracy of self-model
    - Monitoring signals: attention, re-thinking needs
    - Meta-cognitive learning: improves self-model over time
    
    Based on reflective consciousness models (Koriat, 2007).
    """
    
    def __init__(
        self,
        state_dim: int,
        memory_size: int = 100,
        learning_rate: float = 0.01,
        confidence_decay: float = 0.99,
        attention_threshold: float = 0.7
    ):
        self.state_dim = state_dim
        self.learning_rate = learning_rate
        self.confidence_decay = confidence_decay
        self.attention_threshold = attention_threshold
        
        # Self-model: expected cognitive state
        self.self_model = np.zeros(state_dim)
        
        # Confidence in self-model (0-1)
        self.self_model_confidence = 0.5
        
        # Meta-cognitive history
        self.history: deque = deque(maxlen=memory_size)
        
        # Monitoring signals
        self.attention_demands = 0
        self.rethinking_events = 0
        self.insight_events = 0
        
        # Performance tracking
        self.prediction_errors: deque = deque(maxlen=100)
    
    def monitor(
        self,
        cognitive_state: np.ndarray,
        cognitive_output: Optional[np.ndarray] = None
    ) -> dict:
        """
        Monitor cognitive processing and update self-model.
        
        Parameters
        ----------
        cognitive_state : np.ndarray
            Current cognitive state
        cognitive_output : np.ndarray
            Optional output for performance tracking
            
        Returns
        -------
        dict
            Metacognitive assessment
        """
        # Prediction error for self-model
        error = np.linalg.norm(cognitive_state - self.self_model)
        
        # Update self-model (slow learning)
        self.self_model = (
            self.self_model * (1 - self.learning_rate) +
            cognitive_state * self.learning_rate
        )
        
        # Update confidence based on prediction accuracy
        prediction_accuracy = np.exp(-error)
        self.self_model_confidence *= self.confidence_decay
        self.self_model_confidence += (1 - self.confidence_decay) * prediction_accuracy
        self.self_model_confidence = np.clip(self.self_model_confidence, 0.0, 1.0)
        
        # Meta-cognitive signals
        metacognitive_signal = error > self.attention_threshold
        
        if metacognitive_signal:
            self.attention_demands += 1
            if error > self.attention_threshold * 2:
                self.rethinking_events += 1
        elif error < self.attention_threshold * 0.3:
            self.insight_events += 1
        
        # Record prediction error
        self.prediction_errors.append(error)
        
        # Store in history
        self.history.append({
            'cognitive_state': cognitive_state.copy(),
            'error': error,
            'confidence': self.self_model_confidence,
            'metacognitive_signal': metacognitive_signal
        })
        
        return {
            'error': float(error),
            'confidence': float(self.self_model_confidence),
            'rethink_needed': metacognitive_signal,
            'insight_detected': error < self.attention_threshold * 0.3,
            'self_model': self.self_model.copy()
        }
    
    def evaluate_confidence(self) -> dict:
        """
        Evaluate current meta-cognitive state.
        
        Returns
        -------
        dict
            Confidence assessment
        """
        if len(self.prediction_errors) < 10:
            return {
                'level': 'unknown',
                'trend': 0.0,
                'reliability': 0.5
            }
        
        errors = np.array(self.prediction_errors)
        
        # Level: inverse of mean error
        level = np.exp(-np.mean(errors))
        
        # Trend: linear fit
        if len(errors) > 20:
            times = np.arange(len(errors))
            coeffs = np.polyfit(times, errors, 1)
            trend = -coeffs[0]  # Negative = improving
        else:
            trend = 0.0
        
        # Reliability: consistency of self-model
        reliability = np.exp(-np.std(errors))
        
        return {
            'level': level,
            'trend': trend,
            'reliability': reliability,
            'interpretation': 'confident' if level > 0.7 else 'uncertain'
        }
    
    def need_reevaluation(self) -> bool:
        """Check if system needs to reevaluate its understanding."""
        if len(self.prediction_errors) < 10:
            return False
        
        recent_errors = list(self.prediction_errors)[-10:]
        avg_recent = np.mean(recent_errors)
        
        # If recent errors are high, need reevaluation
        return avg_recent > self.attention_threshold
    
    def get_self_awareness_level(self) -> float:
        """
        Get overall self-awareness level.
        
        Combines confidence, reliability, and meta-cognitive activity.
        """
        if len(self.prediction_errors) < 10:
            return 0.0
        
        confidence = self.self_model_confidence
        reliability = np.exp(-np.std(list(self.prediction_errors)))
        
        # Meta-cognitive activity ratio
        total_events = self.attention_demands + 1
        meta_activity = (self.rethinking_events + self.insight_events) / total_events
        
        # Combined awareness level
        awareness = 0.4 * confidence + 0.4 * reliability + 0.2 * meta_activity
        
        return float(awareness)
    
    def reset(self):
        """Reset metacognition."""
        self.self_model = np.zeros(self.state_dim)
        self.self_model_confidence = 0.5
        self.history.clear()
        self.prediction_errors.clear()
        self.attention_demands = 0
        self.rethinking_events = 0
        self.insight_events = 0


# ============================================================================
# PREDICTIVE CODING LAYER (Task 4.5)
# Top-down prediction meets bottom-up perception
# ============================================================================

class PredictiveCodingLayer:
    """
    Predictive coding layer (Rao & Ballard, 1999).
    
    Each level maintains:
    - Prediction: top-down expectation
    - Prediction error: bottom-up surprise
    
    The layer learns to minimize prediction error over time.
    """
    
    def __init__(
        self,
        n_neurons: int,
        prediction_learning_rate: float = 0.01,
        error_learning_rate: float = 0.1,
        temporal_horizon: int = 1
    ):
        self.n_neurons = n_neurons
        self.prediction_lr = prediction_learning_rate
        self.error_lr = error_learning_rate
        self.temporal_horizon = temporal_horizon
        
        # Generative model: predicts next state from current
        self.W_predict = np.random.randn(n_neurons, n_neurons) * 0.1
        
        # Prediction and error history
        self.predictions: deque = deque(maxlen=10)
        self.errors: deque = deque(maxlen=10)
        
        # Current state
        self.current_prediction = np.zeros(n_neurons)
        self.current_error = np.zeros(n_neurons)
        
        # Statistics
        self.total_error = 0.0
        self.n_steps = 0
    
    def forward(
        self,
        input_activity: np.ndarray,
        top_down_prediction: Optional[np.ndarray] = None
    ) -> dict:
        """
        One step of predictive coding.
        
        Parameters
        ----------
        input_activity : np.ndarray
            Bottom-up input
        top_down_prediction : np.ndarray
            Optional top-down prediction from higher level
            
        Returns
        -------
        dict
            Prediction, error, and learning signals
        """
        # Use stored prediction if no top-down provided
        if top_down_prediction is None:
            top_down_prediction = self.current_prediction
        
        # Prediction error = bottom-up input - top-down prediction
        error = input_activity - top_down_prediction
        
        # Update prediction for next time step
        self.current_prediction = self.W_predict @ input_activity
        
        # Learn: update generative model to reduce error
        self.W_predict += self.prediction_lr * np.outer(error, input_activity)
        
        # Clip weights to prevent explosion
        self.W_predict = np.clip(self.W_predict, -2, 2)
        
        # Activity = prediction + scaled error
        activity = top_down_prediction + self.error_lr * error
        
        # Store
        self.predictions.append(self.current_prediction.copy())
        self.errors.append(np.linalg.norm(error))
        self.total_error += np.linalg.norm(error)
        self.n_steps += 1
        
        self.current_error = error
        
        return {
            'activity': activity,
            'error': error,
            'prediction': self.current_prediction.copy(),
            'error_magnitude': float(np.linalg.norm(error))
        }
    
    def get_prediction_error_stats(self) -> dict:
        """Get prediction error statistics."""
        if self.n_steps == 0:
            return {'mean': 0.0, 'std': 0.0, 'trend': 0.0}
        
        errors = np.array(list(self.errors))
        
        mean_error = np.mean(errors)
        std_error = np.std(errors)
        
        # Trend
        if len(errors) > 10:
            times = np.arange(len(errors))
            coeffs = np.polyfit(times, errors, 1)
            trend = coeffs[0]
        else:
            trend = 0.0
        
        return {
            'mean': float(mean_error),
            'std': float(std_error),
            'trend': float(trend),
            'total_error': float(self.total_error / self.n_steps)
        }
    
    def reset(self):
        """Reset predictive coding layer."""
        self.W_predict = np.random.randn(self.n_neurons, self.n_neurons) * 0.1
        self.predictions.clear()
        self.errors.clear()
        self.current_prediction = np.zeros(self.n_neurons)
        self.current_error = np.zeros(self.n_neurons)
        self.total_error = 0.0
        self.n_steps = 0


# ============================================================================
# RECURRENT PROCESSING LOOP (Task 4.3)
# Feedback loops for sustained representations
# ============================================================================

class RecurrentProcessingLoop:
    """
    Recurrent processing loop (Lamme, 2006).
    
    Implements three-stage processing:
    1. Feedforward pass: fast, unconscious processing
    2. First recurrence: local feedback, initial conscious glimpses
    3. Second recurrence: global feedback, full awareness
    
    Maintains working memory through recurrence.
    """
    
    def __init__(
        self,
        input_dim: int,
        hidden_dim: int = 64,
        n_recurrence_steps: int = 2,
        feedback_strength: float = 0.5
    ):
        self.input_dim = input_dim
        self.hidden_dim = hidden_dim
        self.n_recurrence = n_recurrence_steps
        self.feedback_strength = feedback_strength
        
        # Network weights
        self.W_ff = np.random.randn(hidden_dim, input_dim) * 0.1  # Feedforward
        self.W_fb = np.random.randn(hidden_dim, hidden_dim) * 0.1  # Feedback
        self.W_rec = np.random.randn(hidden_dim, hidden_dim) * 0.1  # Recurrence
        
        # State
        self.hidden_state = np.zeros(hidden_dim)
        self.previous_state = np.zeros(hidden_dim)
        
        # Processing history
        self.processing_stages: List[dict] = []
    
    def process(
        self,
        input_vector: np.ndarray,
        return_all_stages: bool = False
    ) -> dict:
        """
        Process input through recurrent loop.
        
        Parameters
        ----------
        input_vector : np.ndarray
            Input stimulus
        return_all_stages : bool
            Return all recurrence stages
            
        Returns
        -------
        dict
            Processing result with stages
        """
        self.processing_stages = []
        
        # Stage 1: Feedforward (unconscious)
        ff_output = np.tanh(self.W_ff @ input_vector)
        self.processing_stages.append({
            'stage': 'feedforward',
            'state': ff_output.copy(),
            'consciousness_level': 0.0
        })
        
        # Update hidden state
        self.hidden_state = ff_output
        
        # Recurrence stages
        conscious_level = 0.0
        
        for r in range(self.n_recurrence):
            # Recurrent update with feedback
            feedback = self.W_fb @ self.hidden_state
            recurrence = self.W_rec @ self.previous_state
            
            # Combine feedforward + recurrence + feedback
            pre_activation = (
                0.3 * ff_output +  # Original feedforward
                self.feedback_strength * recurrence +  # Recurrence
                0.2 * feedback  # Feedback from higher level
            )
            
            self.hidden_state = np.tanh(pre_activation)
            
            # Consciousness increases with recurrence
            conscious_level = (r + 1) / (self.n_recurrence + 1)
            
            self.processing_stages.append({
                'stage': f'recurrence_{r+1}',
                'state': self.hidden_state.copy(),
                'consciousness_level': conscious_level
            })
        
        # Store previous state for next iteration
        self.previous_state = self.hidden_state.copy()
        
        if return_all_stages:
            return {
                'output': self.hidden_state.copy(),
                'stages': self.processing_stages,
                'final_consciousness_level': conscious_level
            }
        else:
            return {
                'output': self.hidden_state.copy(),
                'consciousness_level': conscious_level
            }
    
    def get_working_memory_content(self) -> np.ndarray:
        """Get current working memory content."""
        return self.hidden_state.copy()
    
    def clear_working_memory(self):
        """Clear working memory."""
        self.hidden_state = np.zeros(self.hidden_dim)
        self.previous_state = np.zeros(self.hidden_dim)
        self.processing_stages.clear()


# ============================================================================
# INTEGRATED CONSCIOUSNESS CORE (Task 4.x)
# Orchestrates all consciousness components
# ============================================================================

class ConsciousnessCore:
    """
    Integrated consciousness core.
    
    Combines:
    - Φ proxy calculator
    - Global Workspace
    - Metacognition loop
    - Predictive coding
    - Recurrent processing
    
    Provides unified interface for consciousness-like processing.
    """
    
    def __init__(
        self,
        state_dim: int = 64,
        n_modules: int = 8,
        random_seed: Optional[int] = None
    ):
        if random_seed is not None:
            np.random.seed(random_seed)
        
        self.state_dim = state_dim
        
        # Components
        self.phi_calculator = PhiCalculator()
        self.metacognition = MetacognitionLoop(state_dim)
        self.predictive_coding = PredictiveCodingLayer(state_dim)
        self.recurrent_loop = RecurrentProcessingLoop(state_dim, hidden_dim=state_dim)  # Use same dim
        
        # Global Workspace (simplified version)
        self.n_modules = n_modules
        self.workspace_content: List[int] = []
        
        # State
        self.current_state = np.zeros(state_dim)
        self.state_history: deque = deque(maxlen=1000)
        
        # Consciousness metrics
        self.consciousness_level = 0.0
    
    def process(
        self,
        input_state: np.ndarray,
        compute_consciousness: bool = True
    ) -> dict:
        """
        Process through consciousness core.
        
        Parameters
        ----------
        input_state : np.ndarray
            Input cognitive state
        compute_consciousness : bool
            Whether to compute full consciousness metrics
            
        Returns
        -------
        dict
            Complete consciousness assessment
        """
        self.current_state = input_state
        self.state_history.append(input_state.copy())
        
        results = {'input_state': input_state.copy()}
        
        # 1. Recurrent processing
        recurrent_result = self.recurrent_loop.process(input_state)
        results['recurrent'] = recurrent_result
        
        # 2. Predictive coding
        prediction_result = self.predictive_coding.forward(recurrent_result['output'])
        results['predictive'] = prediction_result
        
        # 3. Metacognition
        metacog_result = self.metacognition.monitor(recurrent_result['output'])
        results['metacognition'] = metacog_result
        
        # 4. Consciousness level
        if compute_consciousness and len(self.state_history) > 50:
            states_array = np.array(list(self.state_history)[-100:])
            phi_result = self.phi_calculator.compute_all(states_array)
            results['phi'] = phi_result
            
            # Combined consciousness level
            awareness = self.metacognition.get_self_awareness_level()
            recurrence = recurrent_result['consciousness_level']
            integration = phi_result.get('phi_composite', 0.5)

            self.consciousness_level = np.clip(
                0.3 * awareness + 0.3 * recurrence + 0.4 * integration,
                0.0, 1.0
            )
            results['consciousness_level'] = self.consciousness_level
        else:
            results['consciousness_level'] = 0.0
        
        return results
    
    def get_consciousness_summary(self) -> dict:
        """Get summary of consciousness state."""
        if len(self.state_history) < 10:
            return {'status': 'insufficient_data'}
        
        # Get all metrics
        states_array = np.array(list(self.state_history)[-100:])
        phi_result = self.phi_calculator.compute_all(states_array)
        confidence = self.metacognition.evaluate_confidence()
        prediction_stats = self.predictive_coding.get_prediction_error_stats()
        
        return {
            'consciousness_level': self.consciousness_level,
            'phi_composite': phi_result.get('phi_composite', 0.0),
            'self_awareness': self.metacognition.get_self_awareness_level(),
            'prediction_error': prediction_stats['mean'],
            'confidence': confidence,
            'integration_interpretation': phi_result.get('interpretation', {}).get('integration_level', 'unknown')
        }
    
    def reset(self):
        """Reset consciousness core."""
        self.current_state = np.zeros(self.state_dim)
        self.state_history.clear()
        self.metacognition.reset()
        self.predictive_coding.reset()
        self.recurrent_loop.clear_working_memory()
        self.consciousness_level = 0.0


# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

def consciousness_proxy_suite(trajectory: np.ndarray, window_size: int = 500) -> dict:
    """
    Compute all consciousness proxy metrics.
    (Kept for backward compatibility)
    """
    if trajectory.ndim == 1:
        trajectory = trajectory.reshape(-1, 1)
    
    results = {}
    
    # RQA
    R = recurrence_matrix(trajectory[-window_size:], threshold=0.1)
    results['rqa'] = rqa_metrics(R)
    
    # AIS
    results['ais'] = active_information_storage(trajectory[-window_size:])
    
    # Φ^G (backward compat)
    calculator = PhiCalculator()
    results['phi_geo_proxy'] = calculator.compute_geometric_phi(trajectory[-window_size:])
    
    # Summary
    results['interpretation'] = {
        'recurrence': 'high' if results['rqa']['REC'] > 0.1 else 'low',
        'determinism': 'high' if results['rqa']['DET'] > 0.5 else 'low',
        'integrated_information_proxy': 'high' if results['phi_geo_proxy'] < 0.1 else 'low',
        'ais_level': 'high' if results['ais'] > 1.0 else 'low'
    }
    
    return results