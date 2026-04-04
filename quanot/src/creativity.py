"""
Creativity Module
=================
Phase 3: Creative Synthesis Framework

This module implements computational creativity components:
- Novelty detection (k-NN surprise metric)
- Conceptual blending (Fauconnier & Turner framework)
- Creative evaluation (surprise, usefulness, coherence)
- Metaphor generation via attractor mapping

References:
- Boden (1990, 2004) - Computational creativity framework
- Fauconnier & Turner (2002) - Conceptual blending theory
- Thaler (2013) - The Creativity Machine
"""

import numpy as np
from scipy.spatial.distance import cdist, pdist
from typing import Optional, Tuple, List, Dict
from collections import deque


# ============================================================================
# NOVELTY DETECTION (k-NN Surprise Metric)
# ============================================================================

class NoveltyDetector:
    """
    Novelty detection using k-nearest neighbor distance.
    
    Computes statistical surprise based on distance to k nearest
    historical states. Higher distance = higher novelty.
    
    Based on the "statistical surprise" metric in computational creativity.
    """
    
    def __init__(
        self,
        state_dim: int,
        history_size: int = 1000,
        k: int = 5,
        novelty_threshold: float = 0.5,
        learning_rate: float = 0.01
    ):
        """
        Initialize novelty detector.
        
        Parameters
        ----------
        state_dim : int
            Dimension of state vectors
        history_size : int
            Maximum number of historical states to store
        k : int
            Number of nearest neighbors to consider
        novelty_threshold : float
            Threshold above which a state is considered "novel"
        learning_rate : float
            Rate at which to adapt baseline novelty
        """
        self.state_dim = state_dim
        self.history_size = history_size
        self.k = min(k, history_size)
        self.novelty_threshold = novelty_threshold
        self.learning_rate = learning_rate
        
        # History buffer
        self.history: deque = deque(maxlen=history_size)
        
        # Statistics for adaptive threshold
        self.novelty_baseline = 0.0
        self.novelty_std = 1.0
        self.novelty_history: deque = deque(maxlen=100)
        
        # Cumulative novelty score
        self.cumulative_novelty = 0.0
        self.n_evaluations = 0
    
    def add_state(self, state: np.ndarray):
        """Add a state to history."""
        if len(state) != self.state_dim:
            raise ValueError(f"Expected state_dim={self.state_dim}, got {len(state)}")
        
        # Normalize and store
        norm_state = state / (np.linalg.norm(state) + 1e-10)
        self.history.append(norm_state)
    
    def compute_novelty(self, state: np.ndarray) -> float:
        """
        Compute novelty score for a state.
        
        Novelty = average distance to k nearest neighbors in history.
        Higher values = more novel.
        
        Parameters
        ----------
        state : np.ndarray
            State vector of shape (state_dim,)
            
        Returns
        -------
        float
            Novelty score (0 = not novel, 1 = highly novel)
        """
        if len(self.history) < self.k:
            # Not enough history - consider maximally novel
            return 1.0
        
        # Normalize input state
        norm_state = state / (np.linalg.norm(state) + 1e-10)
        
        # Compute distances to all history
        history_array = np.array(self.history)
        dists = np.linalg.norm(history_array - norm_state, axis=1)
        
        # Get k nearest distances
        k_nearest_dists = np.partition(dists, self.k - 1)[:self.k]
        
        # Average distance is the novelty score
        avg_distance = np.mean(k_nearest_dists)
        
        # Normalize by typical state space extent
        if len(self.history) > 100:
            # Use running statistics for normalization
            typical_extent = max(self.novelty_std, 0.1)
            normalized_novelty = avg_distance / (self.novelty_baseline + 3 * typical_extent)
        else:
            # Fallback: use raw distance scaled
            normalized_novelty = avg_distance / (np.sqrt(self.state_dim) + 1e-10)
        
        # Clip to [0, 1]
        normalized_novelty = np.clip(normalized_novelty, 0.0, 1.0)
        
        # Update baseline statistics
        self._update_statistics(normalized_novelty)
        
        # Add to history
        self.add_state(state)
        
        return normalized_novelty
    
    def _update_statistics(self, novelty: float):
        """Update running statistics for adaptive threshold."""
        self.novelty_history.append(novelty)
        
        if len(self.novelty_history) >= 20:
            recent = np.array(self.novelty_history)
            self.novelty_baseline = np.mean(recent)
            self.novelty_std = np.std(recent) + 1e-10
    
    def is_novel(self, state: np.ndarray, threshold: Optional[float] = None) -> bool:
        """Check if state is novel relative to history."""
        threshold = threshold or self.novelty_threshold
        return self.compute_novelty(state) > threshold
    
    def get_novelty_trend(self, window: int = 50) -> float:
        """Get trend of novelty over recent evaluations."""
        if len(self.novelty_history) < window:
            return 0.0
        
        recent = list(self.novelty_history)[-window:]
        if len(recent) < 2:
            return 0.0
        
        # Simple linear trend
        times = np.arange(len(recent))
        coeffs = np.polyfit(times, recent, 1)
        return coeffs[0]  # Positive = increasing novelty (exploring)
    
    def reset(self):
        """Reset novelty detector."""
        self.history.clear()
        self.novelty_history.clear()
        self.cumulative_novelty = 0.0
        self.n_evaluations = 0


# ============================================================================
# CONCEPTUAL BLENDING
# Based on Fauconnier & Turner (2002) Conceptual Integration Theory
# ============================================================================

class ConceptualBlender:
    """
    Conceptual blending for creative idea generation.
    
    Blends two input concepts by:
    1. Mapping common structure (analogy)
    2. Creating blend space with emergent structure
    3. Generating novel outputs
    
    Parameters
    ----------
    embedding_dim : int
        Dimension of concept embeddings
    blend_temperature : float
        Controls randomness in blending (higher = more creative)
    """
    
    def __init__(
        self,
        embedding_dim: int = 64,
        blend_temperature: float = 0.5,
        random_seed: Optional[int] = None
    ):
        self.embedding_dim = embedding_dim
        self.blend_temperature = blend_temperature
        
        if random_seed is not None:
            np.random.seed(random_seed)
        
        # Blend history for tracking
        self.blend_history: List[dict] = []
    
    def create_concept_embedding(
        self,
        concept_name: str,
        properties: Optional[Dict[str, float]] = None
    ) -> np.ndarray:
        """
        Create a concept embedding vector.
        
        For demonstration, creates a synthetic embedding.
        In production, would use learned embeddings from a language model.
        
        Parameters
        ----------
        concept_name : str
            Name of the concept
        properties : dict
            Optional properties to encode
            
        Returns
        -------
        np.ndarray
            Concept embedding of shape (embedding_dim,)
        """
        # Create deterministic embedding based on concept name
        np.random.seed(hash(concept_name) % (2**32))
        embedding = np.random.randn(self.embedding_dim)
        
        # Normalize
        embedding = embedding / (np.linalg.norm(embedding) + 1e-10)
        
        # If properties provided, modulate embedding
        if properties:
            for key, value in properties.items():
                # Add property-based modulation
                prop_hash = hash(key) % (2**32)
                np.random.seed(prop_hash)
                prop_vec = np.random.randn(self.embedding_dim)
                embedding += value * prop_vec * 0.3
        
        return embedding
    
    def blend(
        self,
        concept_a: np.ndarray,
        concept_b: np.ndarray,
        blend_ratio: float = 0.5,
        emergent_weight: float = 0.3
    ) -> dict:
        """
        Blend two concepts.
        
        Creates a blend space with:
        - Shared structure (averaged)
        - Unique structure from each
        - Emergent structure (nonlinear combination)
        
        Parameters
        ----------
        concept_a : np.ndarray
            First concept embedding
        concept_b : np.ndarray
            Second concept embedding
        blend_ratio : float
            Ratio for linear blend (0 = all A, 1 = all B)
        emergent_weight : float
            Weight for emergent component
            
        Returns
        -------
        dict
            Blend result with components and novelty estimate
        """
        if len(concept_a) != self.embedding_dim or len(concept_b) != self.embedding_dim:
            raise ValueError(f"Expected embedding_dim={self.embedding_dim}")
        
        # Linear blend
        linear_blend = blend_ratio * concept_a + (1 - blend_ratio) * concept_b
        
        # Emergent structure (tensor product + nonlinearity)
        tensor_product = concept_a[:, np.newaxis] * concept_b[np.newaxis, :]
        
        # Project down to embedding dimension
        np.random.seed(42)
        projection = np.random.randn(tensor_product.shape[0], self.embedding_dim)
        emergent = np.tanh(projection @ tensor_product.diagonal() * 0.1)
        
        # Add noise for creativity (temperature)
        noise = self.blend_temperature * np.random.randn(self.embedding_dim)
        
        # Final blend
        final_blend = (
            (1 - emergent_weight) * linear_blend + 
            emergent_weight * emergent + 
            noise
        )
        final_blend = final_blend / (np.linalg.norm(final_blend) + 1e-10)
        
        # Estimate novelty (distance from original concepts)
        dist_a = np.linalg.norm(final_blend - concept_a)
        dist_b = np.linalg.norm(final_blend - concept_b)
        novelty_estimate = (dist_a + dist_b) / 2
        
        result = {
            'blended': final_blend,
            'linear': linear_blend,
            'emergent': emergent,
            'novelty': novelty_estimate,
            'blend_ratio': blend_ratio
        }
        
        self.blend_history.append(result)
        
        return result
    
    def run_blending_chain(
        self,
        concepts: List[np.ndarray],
        n_blends: int = 3
    ) -> np.ndarray:
        """
        Run a chain of conceptual blends.
        
        Starts with initial concepts and iteratively blends them.
        
        Parameters
        ----------
        concepts : list
            Initial concept embeddings
        n_blends : int
            Number of blending iterations
            
        Returns
        -------
        np.ndarray
            Final blended concept
        """
        if len(concepts) < 2:
            raise ValueError("Need at least 2 concepts")
        
        current = concepts[0]
        
        for i in range(min(n_blends, len(concepts) - 1)):
            next_concept = concepts[i + 1]
            
            # Blend current with next
            result = self.blend(current, next_concept)
            current = result['blended']
        
        return current


# ============================================================================
# CREATIVE EVALUATION FRAMEWORK
# ============================================================================

class CreativeEvaluator:
    """
    Evaluates creative outputs on three dimensions:
    - Surprise: how unexpected is the output?
    - Usefulness: how valuable/applicable is it?
    - Coherence: how internally consistent is it?
    
    Based on Boden (2004) and Newell, Shaw & Simon's creativity criteria.
    """
    
    def __init__(
        self,
        surprise_weight: float = 0.4,
        usefulness_weight: float = 0.3,
        coherence_weight: float = 0.3,
        random_seed: Optional[int] = None
    ):
        self.surprise_weight = surprise_weight
        self.usefulness_weight = usefulness_weight
        self.coherence_weight = coherence_weight
        
        if random_seed is not None:
            np.random.seed(random_seed)
        
        # Evaluation history
        self.evaluation_history: List[dict] = []
    
    def evaluate(
        self,
        output: np.ndarray,
        context: Optional[dict] = None,
        reference_outputs: Optional[List[np.ndarray]] = None
    ) -> dict:
        """
        Evaluate a creative output.
        
        Parameters
        ----------
        output : np.ndarray
            The creative output to evaluate
        context : dict
            Additional context (task goals, constraints, etc.)
        reference_outputs : list
            Previous outputs for comparison
            
        Returns
        -------
        dict
            Evaluation scores for all dimensions and overall creativity
        """
        # 1. Surprise score
        surprise = self._compute_surprise(output, reference_outputs)
        
        # 2. Usefulness score
        usefulness = self._compute_usefulness(output, context)
        
        # 3. Coherence score
        coherence = self._compute_coherence(output)
        
        # Weighted overall score
        overall_creativity = (
            self.surprise_weight * surprise +
            self.usefulness_weight * usefulness +
            self.coherence_weight * coherence
        )
        
        result = {
            'surprise': surprise,
            'usefulness': usefulness,
            'coherence': coherence,
            'overall_creativity': overall_creativity,
            'evaluation': self._interpret_score(overall_creativity)
        }
        
        self.evaluation_history.append(result)
        
        return result
    
    def _compute_surprise(
        self,
        output: np.ndarray,
        reference_outputs: Optional[List[np.ndarray]] = None
    ) -> float:
        """Compute surprise score (unexpectedness)."""
        if reference_outputs is None or len(reference_outputs) < 3:
            # Without references, use entropy-based surprise
            output_normalized = output / (np.linalg.norm(output) + 1e-10)
            entropy = -np.sum(output_normalized ** 2 * np.log(output_normalized ** 2 + 1e-10))
            return min(entropy / np.log(len(output)), 1.0)
        
        # Distance from nearest references
        ref_array = np.array(reference_outputs)
        dists = np.linalg.norm(ref_array - output, axis=1)
        min_dist = np.min(dists)
        
        # Normalize: larger distance = more surprise
        surprise = min_dist / (np.sqrt(output.shape[0]) + 1e-10)
        
        return min(surprise, 1.0)
    
    def _compute_usefulness(
        self,
        output: np.ndarray,
        context: Optional[dict] = None
    ) -> float:
        """Compute usefulness score (value/applicability)."""
        if context is None:
            # Without context, estimate from output properties
            # Higher variance = potentially more useful (more expressiveness)
            variance = np.var(output)
            usefulness = min(variance * 10, 1.0)
        else:
            # Check against task goals
            goals = context.get('goals', [])
            if not goals:
                usefulness = 0.5
            else:
                # Simple matching: check if output has relevant features
                usefulness = min(len(goals) / 10, 1.0)
        
        return usefulness
    
    def _compute_coherence(self, output: np.ndarray) -> float:
        """Compute coherence score (internal consistency)."""
        # Use local variance as proxy for coherence
        # Low variance in different regions = coherent structure
        
        n = len(output)
        
        # Split into segments
        n_segments = min(5, n)
        segment_size = n // n_segments
        
        segment_means = []
        for i in range(n_segments):
            start = i * segment_size
            end = start + segment_size if i < n_segments - 1 else n
            segment_means.append(np.mean(output[start:end]))
        
        # Coherence = low variance between segment means
        if len(segment_means) > 1:
            between_var = np.var(segment_means)
            # Normalize: lower between-segment variance = higher coherence
            coherence = 1.0 / (1.0 + between_var)
        else:
            coherence = 0.5
        
        return min(coherence, 1.0)
    
    def _interpret_score(self, score: float) -> str:
        """Interpret creativity score."""
        if score < 0.2:
            return "low_creativity"
        elif score < 0.4:
            return "moderate_creativity"
        elif score < 0.6:
            return "good_creativity"
        elif score < 0.8:
            return "high_creativity"
        else:
            return "exceptional_creativity"
    
    def get_average_scores(self, window: int = 50) -> dict:
        """Get average scores over recent evaluations."""
        if not self.evaluation_history:
            return {'surprise': 0, 'usefulness': 0, 'coherence': 0, 'overall': 0}
        
        recent = self.evaluation_history[-window:]
        
        return {
            'surprise': np.mean([e['surprise'] for e in recent]),
            'usefulness': np.mean([e['usefulness'] for e in recent]),
            'coherence': np.mean([e['coherence'] for e in recent]),
            'overall': np.mean([e['overall_creativity'] for e in recent])
        }


# ============================================================================
# METAPHOR GENERATION VIA ATTRACTOR MAPPING
# ============================================================================

class MetaphorGenerator:
    """
    Generate metaphors by mapping between attractor domains.
    
    Uses strange attractor geometry as a framework for 
    understanding and expressing relationships between concepts.
    
    Different attractors have different "personalities":
    - Lorenz: complex, butterfly-shaped, sensitive
    - Rössler: spirograph-like, smoother
    - Hénon: discrete, map-based
    """
    
    def __init__(
        self,
        attractor_type: str = "lorenz",
        trajectory_length: int = 100,
        random_seed: Optional[int] = None
    ):
        self.attractor_type = attractor_type
        self.trajectory_length = trajectory_length
        
        if random_seed is not None:
            np.random.seed(random_seed)
        
        # Generate base attractor trajectory
        self.base_trajectory = self._generate_attractor()
        
        # Metaphor mappings
        self.mappings: List[dict] = []
    
    def _generate_attractor(self) -> np.ndarray:
        """Generate attractor trajectory based on type."""
        if self.attractor_type == "lorenz":
            return self._lorenz()
        elif self.attractor_type == "rossler":
            return self._rossler()
        elif self.attractor_type == "henon":
            return self._henon()
        else:
            raise ValueError(f"Unknown attractor type: {self.attractor_type}")
    
    def _lorenz(self) -> np.ndarray:
        """Generate Lorenz attractor."""
        sigma, rho, beta = 10.0, 28.0, 8.0 / 3.0
        dt = 0.01
        
        trajectory = np.zeros((self.trajectory_length, 3))
        x, y, z = 0.1, 0.0, 0.0
        
        for i in range(self.trajectory_length):
            dx = sigma * (y - x) * dt
            dy = (x * (rho - z) - y) * dt
            dz = (x * y - beta * z) * dt
            x, y, z = x + dx, y + dy, z + dz
            trajectory[i] = [x, y, z]
        
        return trajectory
    
    def _rossler(self) -> np.ndarray:
        """Generate Rössler attractor."""
        a, b, c = 0.2, 0.2, 5.7
        dt = 0.01
        
        trajectory = np.zeros((self.trajectory_length, 3))
        x, y, z = 0.1, 0.1, 0.1
        
        for i in range(self.trajectory_length):
            dx = (-y - z) * dt
            dy = (x + a * y) * dt
            dz = (b + z * (x - c)) * dt
            x, y, z = x + dx, y + dy, z + dz
            trajectory[i] = [x, y, z]
        
        return trajectory
    
    def _henon(self) -> np.ndarray:
        """Generate Hénon map."""
        a, b = 1.4, 0.3
        
        trajectory = np.zeros((self.trajectory_length, 2))
        x, y = 0.1, 0.3
        
        for i in range(self.trajectory_length):
            x_new = 1 - a * x**2 + y
            y_new = b * x
            x, y = x_new, y_new
            trajectory[i] = [x, y]
        
        # Pad to 3D
        trajectory = np.hstack([trajectory, np.zeros((self.trajectory_length, 1))])
        
        return trajectory
    
    def generate_metaphor(
        self,
        source_concept: str,
        target_domain: str,
        mapping_strength: float = 0.5
    ) -> dict:
        """
        Generate a metaphor by mapping concept to attractor geometry.
        
        Parameters
        ----------
        source_concept : str
            The concept to metaphorize
        target_domain : str
            The target domain (e.g., "emotion", "process", "structure")
        mapping_strength : float
            Strength of the mapping (0-1)
            
        Returns
        -------
        dict
            Metaphor with mapping details
        """
        # Normalize trajectory
        traj = self.base_trajectory - self.base_trajectory.mean(axis=0)
        traj = traj / (np.std(traj, axis=0) + 1e-10)
        
        # Create mapping based on attractor characteristics
        if self.attractor_type == "lorenz":
            metaphor_base = "complex, intertwined, sensitive to initial conditions"
            structure = "butterfly-shaped attractor with two wings"
            dynamics = "sensitive, unpredictable, beautiful chaos"
        elif self.attractor_type == "rossler":
            metaphor_base = "spiraling, oscillating, evolving"
            structure = "spirograph-like folds"
            dynamics = "smoother, more predictable chaos"
        else:
            metaphor_base = "discrete, map-based, stepwise"
            structure = "discrete points forming pattern"
            dynamics = "iterative transformation"
        
        # Create output metaphor
        metaphor = f"{source_concept} is like {metaphor_base} - {structure} representing {target_domain}"
        
        mapping = {
            'source': source_concept,
            'target': target_domain,
            'attractor_type': self.attractor_type,
            'metaphor': metaphor,
            'structure': structure,
            'dynamics': dynamics,
            'mapping_strength': mapping_strength,
            'trajectory_sample': traj[:10].tolist()  # First 10 points as sample
        }
        
        self.mappings.append(mapping)
        
        return mapping
    
    def get_attractor_properties(self) -> dict:
        """Get properties of the current attractor."""
        traj = self.base_trajectory
        
        # Compute basic properties
        return {
            'type': self.attractor_type,
            'length': len(traj),
            'dimensionality': traj.shape[1],
            'range_x': [float(traj[:, 0].min()), float(traj[:, 0].max())],
            'range_y': [float(traj[:, 1].min()), float(traj[:, 1].max())],
            'std_x': float(traj[:, 0].std()),
            'std_y': float(traj[:, 1].std())
        }


# ============================================================================
# CREATIVE SYNTHESIS ORCHESTRATOR
# Combines all creativity components
# ============================================================================

class CreativeSynthesizer:
    """
    Orchestrates creative synthesis from all components.
    
    Coordinates:
    - Novelty detection
    - Conceptual blending
    - Creative evaluation
    - Metaphor generation
    
    Maintains creative state and controls oscillation between
    exploration (chaos) and exploitation (order).
    """
    
    def __init__(
        self,
        state_dim: int = 64,
        creative_temperature: float = 0.5,
        random_seed: Optional[int] = None
    ):
        self.state_dim = state_dim
        self.creative_temperature = creative_temperature
        
        if random_seed is not None:
            np.random.seed(random_seed)
        
        # Initialize components
        self.novelty_detector = NoveltyDetector(state_dim, k=5)
        self.conceptual_blender = ConceptualBlender(embedding_dim=state_dim)
        self.creative_evaluator = CreativeEvaluator()
        self.metaphor_generator = MetaphorGenerator()
        
        # Creative state
        self.current_state = np.zeros(state_dim)
        self.state_history: List[np.ndarray] = []
        
        # Creative cycle tracking
        self.exploration_count = 0
        self.exploitation_count = 0
        self.cycle_count = 0
    
    def creative_step(
        self,
        input_state: np.ndarray,
        mode: str = 'explore',
        context: Optional[dict] = None
    ) -> dict:
        """
        One creative processing step.
        
        Parameters
        ----------
        input_state : np.ndarray
            Input state vector
        mode : str
            'explore' (chaos/divergence) or 'exploit' (order/convergence)
        context : dict
            Optional context for evaluation
            
        Returns
        -------
        dict
            Creative output with all metrics
        """
        # Process through creative pipeline
        if mode == 'explore':
            # Generate novel output
            output = self._explore(input_state)
            self.exploration_count += 1
        else:
            # Refine existing ideas
            output = self._exploit(input_state)
            self.exploitation_count += 1
        
        # Evaluate creativity
        reference = self.state_history[-10:] if len(self.state_history) > 10 else None
        evaluation = self.creative_evaluator.evaluate(
            output, 
            context=context,
            reference_outputs=reference
        )
        
        # Compute novelty
        novelty = self.novelty_detector.compute_novelty(output)
        
        # Update state
        self.current_state = output
        self.state_history.append(output.copy())
        
        # Increment cycle count
        self.cycle_count += 1
        
        return {
            'output': output,
            'novelty': novelty,
            'evaluation': evaluation,
            'mode': mode,
            'cycle_count': self.cycle_count
        }
    
    def _explore(self, input_state: np.ndarray) -> np.ndarray:
        """Exploration: generate novel state."""
        # Add chaotic perturbation
        noise_scale = 0.3 + 0.2 * np.random.rand()
        noise = noise_scale * np.random.randn(self.state_dim)
        
        output = input_state + noise
        
        # Blend with random concept if available
        if len(self.conceptual_blender.blend_history) > 0:
            last_blend = self.conceptual_blender.blend_history[-1]['blended']
            output = 0.5 * output + 0.5 * last_blend
        
        return output / (np.linalg.norm(output) + 1e-10)
    
    def _exploit(self, input_state: np.ndarray) -> np.ndarray:
        """Exploitation: refine toward better states."""
        # Convergence toward higher-value regions
        if len(self.state_history) > 0:
            # Average of recent states
            recent_avg = np.mean(self.state_history[-5:], axis=0)
            
            # Blend toward this average
            output = 0.3 * input_state + 0.7 * recent_avg
        else:
            output = input_state
        
        return output / (np.linalg.norm(output) + 1e-10)
    
    def generate_concept_blend(
        self,
        concept_a: str,
        concept_b: str,
        properties_a: Optional[dict] = None,
        properties_b: Optional[dict] = None
    ) -> dict:
        """Generate a conceptual blend."""
        # Create embeddings
        emb_a = self.conceptual_blender.create_concept_embedding(
            concept_a, properties_a
        )
        emb_b = self.conceptual_blender.create_concept_embedding(
            concept_b, properties_b
        )
        
        # Blend
        result = self.conceptual_blender.blend(emb_a, emb_b)
        
        return {
            'concept_a': concept_a,
            'concept_b': concept_b,
            'blended_embedding': result['blended'],
            'novelty': result['novelty']
        }
    
    def get_creative_summary(self) -> dict:
        """Get summary of creative performance."""
        eval_summary = self.creative_evaluator.get_average_scores()
        
        return {
            'total_cycles': self.cycle_count,
            'exploration_count': self.exploration_count,
            'exploitation_count': self.exploitation_count,
            'avg_surprise': eval_summary['surprise'],
            'avg_usefulness': eval_summary['usefulness'],
            'avg_coherence': eval_summary['coherence'],
            'avg_creativity': eval_summary['overall']
        }
    
    def reset(self):
        """Reset creative synthesizer."""
        self.current_state = np.zeros(self.state_dim)
        self.state_history.clear()
        self.exploration_count = 0
        self.exploitation_count = 0
        self.cycle_count = 0
        self.novelty_detector.reset()


# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

def divergence_metric(state: np.ndarray, baseline: np.ndarray, window: int = 50) -> float:
    """
    Compute divergence metric for creative oscillation.
    
    Measures how far the current state is from a baseline,
    normalized by typical state space extent.
    
    Parameters
    ----------
    state : np.ndarray
        Current state
    baseline : np.ndarray
        Baseline/reference state
    window : int
        Window for computing typical extent
        
    Returns
    -------
    float
        Divergence metric (0 = close to baseline, 1 = far)
    """
    # Compute distance from baseline
    dist = np.linalg.norm(state - baseline)
    
    # Normalize by typical state space extent (sqrt of dimension)
    typical_extent = np.sqrt(len(state))
    
    divergence = dist / (typical_extent + 1e-10)
    
    return min(divergence, 1.0)


def attractor_strength(state: np.ndarray, attractor_center: np.ndarray) -> float:
    """
    Compute strength of attractor basin.
    
    Higher strength = system is strongly attracted to this state.
    
    Parameters
    ----------
    state : np.ndarray
        Current state
    attractor_center : np.ndarray
        Center of attractor basin
        
    Returns
    -------
    float
        Attractor strength (0-1)
    """
    dist = np.linalg.norm(state - attractor_center)
    
    # Inverse distance (closer = stronger attraction)
    strength = 1.0 / (1.0 + dist)
    
    return strength