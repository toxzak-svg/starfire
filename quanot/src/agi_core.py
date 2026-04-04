"""
AGI Core Module - Phase 5
=========================
Phase 5: AGI Architecture Integration

This module integrates all previous phases into a unified AGI architecture:
- Quantum-inspired encoding (superposition for multi-hypothesis)
- Chaotic reservoir (dynamic adaptation and pattern completion)
- Creative oscillator (novelty generation and divergence)
- Consciousness core (self-modeling and meta-cognition)
- World model (predictive understanding of environment)
- Multi-modal integration
- Continuous learning pipeline

References:
- Plan: plans/unconventional_agi_plan.md
"""

import numpy as np
from typing import Optional, List, Dict, Tuple
from collections import deque
import warnings

# Import all previous phase components
from quantum_inspired import (
    SimulatedQuantumAnnealing,
    solve_ising,
    solve_qubo,
    cognitive_state_compress,
    QuantumWalkSampler
)
from reservoir import ChaoticReservoir, CreativeOscillator
from chaos import lorenz_attractor, rossler_attractor, henon_map, ChaoticReservoir as ChaosChaoticReservoir
from creativity import (
    NoveltyDetector,
    ConceptualBlender,
    CreativeEvaluator,
    MetaphorGenerator,
    CreativeSynthesizer,
    divergence_metric
)
from consciousness import GlobalWorkspace
from consciousness_enhanced import (
    PhiCalculator,
    MetacognitionLoop,
    PredictiveCodingLayer,
    RecurrentProcessingLoop,
    ConsciousnessCore
)


# ============================================================================
# UNIFIED AGI ARCHITECTURE
# ============================================================================

class AGICore:
    """
    Unified AGI Core integrating all phases.
    
    Combines:
    - Quantum-inspired processing for exploration
    - Chaotic reservoir for dynamic memory
    - Creative system for novelty generation
    - Consciousness for self-awareness
    - World model for predictive understanding
    """
    
    def __init__(
        self,
        state_dim: int = 128,
        reservoir_size: int = 256,
        n_modules: int = 8,
        random_seed: Optional[int] = None
    ):
        """
        Initialize AGI Core.
        
        Parameters
        ----------
        state_dim : int
            Dimension of internal state vectors
        reservoir_size : int
            Size of chaotic reservoir
        n_modules : int
            Number of modules in global workspace
        random_seed : int
            Random seed for reproducibility
        """
        if random_seed is not None:
            np.random.seed(random_seed)
        
        self.state_dim = state_dim
        self.reservoir_size = reservoir_size
        self.n_modules = n_modules
        self.random_seed = random_seed
        
        # ========================================
        # Phase 1: Quantum-Inspired Components
        # ========================================
        self.sqa = SimulatedQuantumAnnealing(
            n_spins=min(state_dim, 20),
            n_trotters=8,
            random_seed=random_seed
        )
        
        # ========================================
        # Phase 2: Chaotic Reservoir
        # ========================================
        self.reservoir = ChaoticReservoir(
            input_dim=state_dim,
            reservoir_size=reservoir_size,
            spectral_radius=0.95,
            input_scaling=0.1,
            noise_level=0.001,
            connectivity=0.1,
            random_seed=random_seed
        )
        
        # ========================================
        # Phase 3: Creative System
        # ========================================
        self.creative_synthesizer = CreativeSynthesizer(
            state_dim=state_dim,
            creative_temperature=0.5,
            random_seed=random_seed
        )
        self.novelty_detector = NoveltyDetector(
            state_dim=state_dim,
            history_size=1000,
            k=5,
            novelty_threshold=0.3
        )
        
        # ========================================
        # Phase 4: Consciousness Core
        # ========================================
        self.consciousness_core = ConsciousnessCore(
            state_dim=state_dim,
            n_modules=n_modules,
            random_seed=random_seed
        )
        self.metacognition = MetacognitionLoop(
            state_dim=state_dim,
            memory_size=100,
            learning_rate=0.01
        )
        
        # ========================================
        # Phase 5: World Model & Goal Reasoning
        # ========================================
        self.world_model = WorldModel(state_dim=state_dim)
        self.goal_manager = GoalManager(state_dim=state_dim)
        
        # ========================================
        # Multi-modal Integration
        # ========================================
        self.modalities: Dict[str, np.ndarray] = {}
        
        # Internal state
        self.current_state = np.zeros(state_dim)
        self.state_history: deque = deque(maxlen=1000)
        self.cycle_count = 0
        
        # Performance metrics
        self.metrics = {
            'total_cycles': 0,
            'avg_creativity': 0.0,
            'avg_consciousness': 0.0,
            'avg_novelty': 0.0,
            'goals_completed': 0
        }
    
    def process(
        self,
        input_data: np.ndarray,
        modality: str = 'default',
        goal: Optional[Dict] = None
    ) -> Dict:
        """
        Process input through the full AGI pipeline.
        
        Parameters
        ----------
        input_data : np.ndarray
            Input data vector
        modality : str
            Input modality (e.g., 'vision', 'language', 'sensor')
        goal : dict
            Optional goal specification
            
        Returns
        -------
        dict
            Complete processing results
        """
        # Store modality input
        self.modalities[modality] = input_data
        
        # Normalize input to state_dim
        if len(input_data) != self.state_dim:
            input_state = self._resize_input(input_data)
        else:
            input_state = input_data.copy()
        
        results = {'input': input_state.copy(), 'modality': modality}
        
        # ========================================
        # Step 1: Quantum-inspired encoding
        # ========================================
        quantum_result = self._quantum_encode(input_state)
        results['quantum'] = quantum_result
        
        # ========================================
        # Step 2: Chaotic reservoir processing
        # ========================================
        reservoir_result = self._reservoir_process(quantum_result['encoded'])
        results['reservoir'] = reservoir_result
        
        # ========================================
        # Step 3: Creative synthesis
        # ========================================
        creative_result = self._creative_process(reservoir_result['state'])
        results['creative'] = creative_result
        
        # ========================================
        # Step 4: Consciousness processing
        # ========================================
        consciousness_result = self._consciousness_process(creative_result['output'])
        results['consciousness'] = consciousness_result
        
        # ========================================
        # Step 5: World model prediction
        # ========================================
        if goal is not None:
            self.goal_manager.set_goal(goal)
        
        world_result = self._world_model_update(consciousness_result['state'], goal)
        results['world'] = world_result
        
        # ========================================
        # Update internal state
        # ========================================
        self.current_state = consciousness_result['state']
        self.state_history.append(self.current_state.copy())
        self.cycle_count += 1
        
        # Update novelty detector
        novelty = self.novelty_detector.compute_novelty(self.current_state)
        results['novelty'] = novelty
        
        # Update metrics
        self._update_metrics(results)
        
        return results
    
    def _resize_input(self, input_data: np.ndarray) -> np.ndarray:
        """Resize input to match state_dim."""
        if len(input_data) < self.state_dim:
            # Pad with zeros
            resized = np.zeros(self.state_dim)
            resized[:len(input_data)] = input_data
            return resized
        else:
            # Truncate or compress
            return input_data[:self.state_dim].copy()
    
    def _quantum_encode(self, input_state: np.ndarray) -> Dict:
        """Quantum-inspired state encoding."""
        # Use quantum walk for exploration
        # Create simple graph based on state dimensions
        n_nodes = min(len(input_state), 10)
        adj = np.random.rand(n_nodes, n_nodes)
        adj = (adj + adj.T) / 2
        np.fill_diagonal(adj, 0)
        
        # Simple sampling via random walk
        sampler = QuantumWalkSampler(adj, n_steps=20, n_walkers=5)
        stationary = sampler.run()[0]
        
        # Combine with original state
        encoded = 0.7 * input_state + 0.3 * np.tile(stationary, len(input_state) // n_nodes + 1)[:len(input_state)]
        
        return {
            'encoded': encoded / (np.linalg.norm(encoded) + 1e-10),
            'quantum_features': stationary
        }
    
    def _reservoir_process(self, input_state: np.ndarray) -> Dict:
        """Chaotic reservoir processing."""
        # Reshape for reservoir
        input_2d = input_state.reshape(1, -1)
        states = self.reservoir.forward(input_2d)
        
        # Extract the last state and resize to state_dim if needed
        reservoir_state = states[-1]
        if len(reservoir_state) != self.state_dim:
            # Project down to state_dim
            if len(reservoir_state) > self.state_dim:
                reservoir_state = reservoir_state[:self.state_dim]
            else:
                # Pad with zeros
                resized = np.zeros(self.state_dim)
                resized[:len(reservoir_state)] = reservoir_state
                reservoir_state = resized
        
        return {
            'state': reservoir_state,
            'regime': self.reservoir.get_regime(),
            'lyapunov': self.reservoir.estimate_lyapunov_online(20) if len(states) > 20 else 0.0
        }
    
    def _creative_process(self, state: np.ndarray) -> Dict:
        """Creative synthesis processing."""
        # Alternate between exploration and exploitation
        mode = 'explore' if self.cycle_count % 2 == 0 else 'exploit'
        
        result = self.creative_synthesizer.creative_step(state, mode=mode)
        
        return {
            'output': result['output'],
            'novelty': result['novelty'],
            'creativity': result['evaluation']['overall_creativity'],
            'mode': mode
        }
    
    def _consciousness_process(self, state: np.ndarray) -> Dict:
        """Consciousness core processing."""
        result = self.consciousness_core.process(state)
        
        return {
            'state': result['recurrent']['output'],
            'consciousness_level': result.get('consciousness_level', 0.0),
            'metacognition': result['metacognition']
        }
    
    def _world_model_update(self, state: np.ndarray, goal: Optional[Dict]) -> Dict:
        """World model and goal processing."""
        # Update world model
        self.world_model.update(state, goal)
        
        # Get prediction
        prediction = self.world_model.predict()
        
        return {
            'prediction': prediction,
            'goal_progress': self.goal_manager.get_progress(),
            'world_state': self.world_model.get_state()
        }
    
    def _update_metrics(self, results: Dict):
        """Update performance metrics."""
        self.metrics['total_cycles'] += 1
        
        # Running average for creativity and consciousness
        alpha = 0.05
        self.metrics['avg_creativity'] = (
            (1 - alpha) * self.metrics['avg_creativity'] + 
            alpha * results['creative']['creativity']
        )
        
        if 'consciousness_level' in results['consciousness']:
            self.metrics['avg_consciousness'] = (
                (1 - alpha) * self.metrics['avg_consciousness'] +
                alpha * results['consciousness']['consciousness_level']
            )
        
        self.metrics['avg_novelty'] = (
            (1 - alpha) * self.metrics['avg_novelty'] +
            alpha * results['novelty']
        )
    
    def get_status(self) -> Dict:
        """Get current AGI status."""
        return {
            'cycle_count': self.cycle_count,
            'metrics': self.metrics.copy(),
            'reservoir_regime': self.reservoir.get_regime(),
            'consciousness_level': self.consciousness_core.consciousness_level,
            'active_goal': self.goal_manager.get_current_goal(),
            'modalities': list(self.modalities.keys())
        }
    
    def reset(self):
        """Reset AGI core."""
        self.current_state = np.zeros(self.state_dim)
        self.state_history.clear()
        self.cycle_count = 0
        self.metrics = {
            'total_cycles': 0,
            'avg_creativity': 0.0,
            'avg_consciousness': 0.0,
            'avg_novelty': 0.0,
            'goals_completed': 0
        }
        self.reservoir.reset()
        self.consciousness_core.reset()
        self.world_model.reset()
        self.goal_manager.reset()
        self.novelty_detector.reset()


# ============================================================================
# WORLD MODEL (Phase 5)
# Predictive understanding of environment
# ============================================================================

class WorldModel:
    """
    World model for predictive understanding.
    
    Maintains:
    - Current state representation
    - Transition dynamics
    - Predictive model
    """
    
    def __init__(
        self,
        state_dim: int = 128,
        history_size: int = 100,
        prediction_horizon: int = 5
    ):
        self.state_dim = state_dim
        self.history_size = history_size
        self.prediction_horizon = prediction_horizon
        
        # State history
        self.state_history: deque = deque(maxlen=history_size)
        
        # Transition model (simple linear)
        self.transition_matrix = np.random.randn(state_dim, state_dim) * 0.01
        
        # Prediction buffer
        self.predictions: deque = deque(maxlen=prediction_horizon)
        
        # Learning rate
        self.learning_rate = 0.01
    
    def update(self, state: np.ndarray, goal: Optional[Dict] = None):
        """Update world model with new state."""
        self.state_history.append(state.copy())
        
        # Learn transition if we have enough history
        if len(self.state_history) >= 2:
            self._learn_transition()
    
    def _learn_transition(self):
        """Learn transition dynamics from history."""
        states = np.array(self.state_history)
        
        if len(states) < 3:
            return
        
        # Simple linear regression for transition
        X = states[:-1]  # Previous states
        Y = states[1:]   # Next states
        
        # Solve for transition matrix using least squares
        # Y = X @ W^T + noise
        try:
            self.transition_matrix = np.linalg.lstsq(X, Y, rcond=None)[0].T
        except:
            pass  # Keep previous matrix if computation fails
    
    def predict(self, steps: int = 1) -> np.ndarray:
        """Predict future state."""
        if len(self.state_history) == 0:
            return np.zeros(self.state_dim)
        
        current = self.state_history[-1].copy()
        
        predictions = []
        for _ in range(steps):
            next_state = self.transition_matrix @ current
            predictions.append(next_state.copy())
            current = next_state
        
        if steps == 1:
            return np.asarray(predictions[-1])
        return np.array(predictions)
    
    def get_state(self) -> Dict:
        """Get world model state."""
        return {
            'history_size': len(self.state_history),
            'transition_norm': float(np.linalg.norm(self.transition_matrix)),
            'last_prediction': self.predict().tolist() if len(self.state_history) > 0 else []
        }
    
    def reset(self):
        """Reset world model."""
        self.state_history.clear()
        self.predictions.clear()
        self.transition_matrix = np.random.randn(self.state_dim, self.state_dim) * 0.01


# ============================================================================
# GOAL MANAGER (Phase 5)
# Goal-oriented reasoning and planning
# ============================================================================

class GoalManager:
    """
    Goal manager for goal-oriented reasoning.
    
    Handles:
    - Goal specification and tracking
    - Progress monitoring
    - Goal selection and switching
    """
    
    def __init__(
        self,
        state_dim: int = 128,
        max_goals: int = 5
    ):
        self.state_dim = state_dim
        self.max_goals = max_goals
        
        # Active goals
        self.goals: deque = deque(maxlen=max_goals)
        
        # Goal history
        self.goal_history: List[Dict] = []
        
        # Current goal progress
        self.current_progress = 0.0
    
    def set_goal(self, goal: Dict):
        """Set a new goal."""
        # Create goal object
        goal_obj = {
            'description': goal.get('description', 'unnamed'),
            'target': goal.get('target', np.zeros(self.state_dim)),
            'priority': goal.get('priority', 0.5),
            'deadline': goal.get('deadline', None),
            'created_at': self.goal_history[-1]['completed_at'] if self.goal_history else 0,
            'progress': 0.0
        }
        
        self.goals.append(goal_obj)
    
    def update_progress(self, current_state: np.ndarray):
        """Update progress toward current goal."""
        if len(self.goals) == 0:
            return
        
        goal = self.goals[-1]
        target = goal['target']
        
        # Compute distance to target
        dist = np.linalg.norm(current_state - target)
        
        # Progress = 1 / (1 + distance)
        progress = 1.0 / (1.0 + dist)
        
        goal['progress'] = progress
        self.current_progress = progress
        
        # Check if goal is complete
        if progress > 0.95:
            self._complete_goal()
    
    def _complete_goal(self):
        """Mark current goal as completed."""
        if len(self.goals) > 0:
            completed = self.goals.pop()
            completed['completed_at'] = len(self.goal_history) + 1
            self.goal_history.append(completed)
    
    def get_progress(self) -> Dict:
        """Get current goal progress."""
        if len(self.goals) == 0:
            return {'status': 'no_goal', 'progress': 0.0}
        
        return {
            'status': 'active',
            'description': self.goals[-1]['description'],
            'progress': self.goals[-1]['progress'],
            'n_goals': len(self.goals)
        }
    
    def get_current_goal(self) -> Optional[Dict]:
        """Get current goal."""
        if len(self.goals) == 0:
            return None
        return self.goals[-1].copy()
    
    def reset(self):
        """Reset goal manager."""
        self.goals.clear()
        self.goal_history.clear()
        self.current_progress = 0.0


# ============================================================================
# MULTI-MODAL INTEGRATOR (Phase 5)
# Integration layer for different input modalities
# ============================================================================

class MultiModalIntegrator:
    """
    Multi-modal integration layer.
    
    Handles integration of:
    - Vision
    - Language/text
    - Sensor data
    - Audio
    """
    
    def __init__(
        self,
        state_dim: int = 128,
        modality_dims: Optional[Dict[str, int]] = None
    ):
        self.state_dim = state_dim
        
        # Default modality dimensions
        if modality_dims is None:
            modality_dims = {
                'vision': 784,    # 28x28 image
                'language': 128,  # Embedded text
                'sensor': 64,     # Sensor readings
                'audio': 128      # Audio features
            }
        
        self.modality_dims = modality_dims
        
        # Modality encoders (simple projection layers)
        self.encoders: Dict[str, np.ndarray] = {}
        for modality, dim in modality_dims.items():
            if dim != state_dim:
                # Random projection matrix
                self.encoders[modality] = np.random.randn(state_dim, dim) * 0.01
            else:
                self.encoders[modality] = np.eye(state_dim)
        
        # Modality weights (learned attention)
        self.modality_weights: Dict[str, float] = {m: 1.0/len(modality_dims) for m in modality_dims}
        
        # Last inputs per modality
        self.last_inputs: Dict[str, np.ndarray] = {}
    
    def encode(self, modality: str, input_data: np.ndarray) -> np.ndarray:
        """Encode input from a specific modality."""
        self.last_inputs[modality] = input_data.copy()
        
        if modality not in self.encoders:
            # Create encoder for unknown modality
            dim = len(input_data)
            self.encoders[modality] = np.random.randn(self.state_dim, dim) * 0.01
            self.modality_dims[modality] = dim
        
        # Project to state dimension
        if len(input_data) == self.state_dim:
            return input_data
        
        encoder = self.encoders[modality]
        encoded = encoder @ input_data
        
        return encoded / (np.linalg.norm(encoded) + 1e-10)
    
    def integrate(self, inputs: Dict[str, np.ndarray]) -> np.ndarray:
        """
        Integrate multiple modality inputs.
        
        Parameters
        ----------
        inputs : dict
            Dictionary of {modality: input_array}
            
        Returns
        -------
        np.ndarray
            Integrated state vector
        """
        encoded_inputs = {}
        
        for modality, input_data in inputs.items():
            encoded_inputs[modality] = self.encode(modality, input_data)
        
        # Weighted integration
        total_weight = sum(self.modality_weights.values())
        integrated = np.zeros(self.state_dim)
        
        for modality, encoded in encoded_inputs.items():
            weight = self.modality_weights.get(modality, 1.0 / len(inputs))
            integrated += (weight / total_weight) * encoded
        
        return integrated / (np.linalg.norm(integrated) + 1e-10)
    
    def update_weights(self, performance: Dict[str, float]):
        """Update modality weights based on performance."""
        # Simple weight update
        total = sum(performance.values())
        for modality, perf in performance.items():
            self.modality_weights[modality] = perf / (total + 1e-10)


# ============================================================================
# CONTINUOUS LEARNING PIPELINE (Phase 5)
# Online adaptation and learning
# ============================================================================

class ContinuousLearning:
    """
    Continuous learning pipeline.
    
    Implements:
    - Online adaptation
    - Experience replay
    - Knowledge consolidation
    """
    
    def __init__(
        self,
        state_dim: int = 128,
        buffer_size: int = 1000,
        batch_size: int = 32
    ):
        self.state_dim = state_dim
        self.buffer_size = buffer_size
        self.batch_size = batch_size
        
        # Experience buffer
        self.experience_buffer: deque = deque(maxlen=buffer_size)
        
        # Learning parameters
        self.learning_rate = 0.01
        self.replay_alpha = 0.5  # Importance of replay
        
        # Consolidation threshold
        self.consolidation_threshold = 100
    
    def add_experience(
        self,
        state: np.ndarray,
        action: Optional[np.ndarray],
        reward: float,
        next_state: np.ndarray
    ):
        """Add experience to buffer."""
        experience = {
            'state': state.copy(),
            'action': action.copy() if action is not None else None,
            'reward': reward,
            'next_state': next_state.copy()
        }
        self.experience_buffer.append(experience)
    
    def sample_batch(self) -> List[Dict]:
        """Sample a batch for learning."""
        if len(self.experience_buffer) < self.batch_size:
            return list(self.experience_buffer)
        
        indices = np.random.choice(
            len(self.experience_buffer),
            self.batch_size,
            replace=False
        )
        
        return [self.experience_buffer[i] for i in indices]
    
    def learn(self, model_weights: Dict) -> Dict:
        """
        Perform one learning step.
        
        Parameters
        ----------
        model_weights : dict
            Current model weights to update
            
        Returns
        -------
        dict
            Learning results
        """
        if len(self.experience_buffer) < 10:
            return {'status': 'insufficient_data'}
        
        batch = self.sample_batch()
        
        # Simple weight update based on batch
        # (In practice, would use proper gradient updates)
        avg_reward = np.mean([exp['reward'] for exp in batch])
        
        return {
            'status': 'learned',
            'batch_size': len(batch),
            'avg_reward': float(avg_reward)
        }
    
    def consolidate(self) -> bool:
        """Consolidate knowledge when threshold reached."""
        if len(self.experience_buffer) >= self.consolidation_threshold:
            # In practice, would perform knowledge consolidation
            return True
        return False


# ============================================================================
# MAIN AGI SYSTEM (Complete Integration)
# ============================================================================

class AGISystem:
    """
    Complete AGI System integrating all components.
    
    This is the main interface for the complete AGI architecture.
    
    Parameters
    ----------
    state_dim : int
        Dimension of internal state vectors
    reservoir_size : int
        Size of chaotic reservoir
    n_modules : int
        Number of modules in global workspace
    random_seed : int
        Random seed for reproducibility
    starfire_path : Optional[str]
        Path to Starfire executable (enables Starfire integration)
    """
    
    def __init__(
        self,
        state_dim: int = 128,
        reservoir_size: int = 256,
        n_modules: int = 8,
        random_seed: Optional[int] = None,
        starfire_path: Optional[str] = None
    ):
        if random_seed is not None:
            np.random.seed(random_seed)
        
        # Core AGI
        self.core = AGICore(
            state_dim=state_dim,
            reservoir_size=reservoir_size,
            n_modules=n_modules,
            random_seed=random_seed
        )
        
        # Multi-modal integrator
        self.multimodal = MultiModalIntegrator(state_dim=state_dim)
        
        # Continuous learning
        self.learning = ContinuousLearning(state_dim=state_dim)
        
        # Starfire integration (optional)
        self.starfire_bridge = None
        self.starfire_available = False
        
        if starfire_path is not None:
            self._init_starfire(starfire_path)
        
        # State
        self.is_running = False
    
    def _init_starfire(self, starfire_path: str) -> None:
        """Initialize Starfire bridge if available."""
        try:
            from starfire_interface import StarfireBridge
            self.starfire_bridge = StarfireBridge(
                reservoir=self.core.reservoir,
                starfire_path=starfire_path,
                mode='stdio'
            )
            self.starfire_available = self.starfire_bridge.available
            if self.starfire_available:
                print(f"[Starfire] Connected to {starfire_path}")
            else:
                print(f"[Starfire] Not available at {starfire_path}")
        except ImportError as e:
            print(f"[Starfire] Bridge not available: {e}")
            self.starfire_available = False
    
    def starfire_process(
        self,
        text_input: str,
        include_state: bool = True
    ) -> Tuple[str, Dict]:
        """
        Process text input through QuaNot → Starfire pipeline.
        
        Requires Starfire to be available and initialized.
        
        Parameters
        ----------
        text_input : str
            Text input from user
        include_state : bool
            Include reservoir state in message
            
        Returns
        -------
        Tuple[str, Dict]
            (output_text, feedback_dict)
        """
        if not self.starfire_available or self.starfire_bridge is None:
            raise RuntimeError("Starfire not available. Initialize with starfire_path.")
        
        return self.starfire_bridge.process(text_input, include_state)
    
    def run(
        self,
        inputs: Dict[str, np.ndarray],
        goal: Optional[Dict] = None
    ) -> Dict:
        """
        Run AGI system on multi-modal inputs.
        
        Parameters
        ----------
        inputs : dict
            Dictionary of {modality: input_array}
        goal : dict
            Optional goal specification
            
        Returns
        -------
        dict
            Complete system output
        """
        # Integrate inputs
        integrated = self.multimodal.integrate(inputs)
        
        # Process through core
        result = self.core.process(integrated, goal=goal)
        
        # Add learning
        if 'reward' in result:
            self.learning.add_experience(
                result['input'],
                None,
                result['reward'],
                result['consciousness']['state']
            )
        
        # Perform learning step
        learning_result = self.learning.learn({})
        result['learning'] = learning_result
        
        # Get system status
        result['status'] = self.core.get_status()
        
        return result
    
    def get_system_info(self) -> Dict:
        """Get system information."""
        return {
            'state_dim': self.core.state_dim,
            'reservoir_size': self.core.reservoir_size,
            'n_modules': self.core.n_modules,
            'is_running': self.is_running,
            'status': self.core.get_status()
        }
    
    def reset(self):
        """Reset entire system."""
        self.core.reset()
        self.learning = ContinuousLearning(state_dim=self.core.state_dim)
        self.is_running = False


# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

def create_agi_system(
    state_dim: int = 128,
    reservoir_size: int = 256,
    random_seed: Optional[int] = 42
) -> AGISystem:
    """
    Create a configured AGI system.
    
    Parameters
    ----------
    state_dim : int
        State dimension
    reservoir_size : int
        Reservoir size
    random_seed : int
        Random seed
        
    Returns
    -------
    AGISystem
        Configured AGI system
    """
    return AGISystem(
        state_dim=state_dim,
        reservoir_size=reservoir_size,
        random_seed=random_seed
    )


# ============================================================================
# DEMO / TEST
# ============================================================================

if __name__ == "__main__":
    print("=" * 60)
    print("QuaNot Phase 5: AGI Architecture Integration")
    print("=" * 60)
    
    # Create system
    np.random.seed(42)
    agi = create_agi_system(state_dim=64, reservoir_size=128, random_seed=42)
    
    print("\n[1] Testing basic processing...")
    
    # Test with random input
    input_data = np.random.randn(64)
    
    result = agi.run({'default': input_data})
    
    print(f"  Cycles: {result['status']['cycle_count']}")
    print(f"  Creativity: {result['creative']['creativity']:.3f}")
    print(f"  Novelty: {result['novelty']:.3f}")
    print(f"  Consciousness: {result['consciousness']['consciousness_level']:.3f}")
    
    print("\n[2] Testing multi-modal integration...")
    
    # Test multi-modal
    vision_input = np.random.randn(64)
    language_input = np.random.randn(64)
    
    result = agi.run({
        'vision': vision_input,
        'language': language_input
    })
    
    print(f"  Modalities: {result['status']['modalities']}")
    print(f"  Cycles: {result['status']['cycle_count']}")
    
    print("\n[3] Testing with goal...")
    
    # Test with goal
    goal = {
        'description': 'explore',
        'target': np.random.randn(64),
        'priority': 0.8
    }
    
    result = agi.run({'default': np.random.randn(64)}, goal=goal)
    
    print(f"  Goal: {result['world']['goal_progress']['description']}")
    print(f"  Progress: {result['world']['goal_progress']['progress']:.3f}")
    
    print("\n[4] Running multiple cycles...")
    
    for i in range(10):
        result = agi.run({'default': np.random.randn(64)})
    
    status = agi.get_system_info()
    print(f"  Total cycles: {status['status']['metrics']['total_cycles']}")
    print(f"  Avg creativity: {status['status']['metrics']['avg_creativity']:.3f}")
    print(f"  Avg consciousness: {status['status']['metrics']['avg_consciousness']:.3f}")
    
    print("\n" + "=" * 60)
    print("Phase 5 AGI Core - Ready!")
    print("=" * 60)