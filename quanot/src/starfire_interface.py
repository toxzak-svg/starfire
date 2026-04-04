"""
Starfire Bridge Interface
======================
Bridge module for connecting QuaNot to Starfire AGI.

Encodes reservoir states for Starfire input,
decodes Starfire responses for QuaNot feedback.

References:
- INTEGRATION_PLAN.md
- starfire/SPEC.md
"""

import uuid
import time
import json
import subprocess
import os
from typing import Optional, Dict, Any, Tuple
from dataclasses import dataclass, asdict, field
import numpy as np

# Import QuaNot components
from src.reservoir import ChaoticReservoir, CreativeOscillator


# ============================================================================
# MESSAGE PROTOCOL
# ============================================================================

@dataclass
class BridgeMessage:
    """Message passed between QuaNot and Starfire."""
    
    # Header
    message_id: str
    timestamp: float
    
    # QuaNot → Starfire
    input_text: Optional[str] = None
    reservoir_state: Optional[list] = None  # 1000-dim vector
    consciousness_proxy: Optional[float] = None  # ψ
    novelty_score: Optional[float] = None
    creativity_scores: Optional[dict] = None
    divergence_metric: Optional[float] = None
    
    # Starfire → QuaNot
    reasoning_result: Optional[str] = None
    reasoning_chain: Optional[list] = field(default_factory=list)
    confidence: Optional[float] = None
    curiosity: Optional[float] = None
    surprise: Optional[float] = None
    identity_consistent: Optional[bool] = None
    
    # Control
    request_mode: str = "query"  # query | reason | learn | introspect
    
    def to_json(self) -> str:
        """Serialize for IPC."""
        data = {}
        for key, value in asdict(self).items():
            if value is not None:
                data[key] = value
        return json.dumps(data)
    
    @classmethod
    def from_json(cls, data: str) -> 'BridgeMessage':
        """Deserialize from IPC."""
        obj = json.loads(data)
        return cls(**obj)


# ============================================================================
# CONSCIOUSNESS METRICS (simplified)
# ============================================================================

def compute_consciousness_proxy(states_history: list) -> float:
    """
    Compute simple consciousness proxy from state history.
    
    Uses correlation-based integration as a proxy for Φ.
    """
    if len(states_history) < 10:
        return 0.5
    
    states = np.array(states_history[-50:])  # Last 50 states
    
    # Compute average pairwise correlation
    # Higher correlation = more integrated = higher consciousness
    corr_matrix = np.corrcoef(states)
    
    # Mask self-correlations
    np.fill_diagonal(corr_matrix, 0)
    
    # Average absolute correlation
    avg_corr = np.mean(np.abs(corr_matrix))
    
    # Scale to 0-1 (arbitrary but useful)
    return float(np.clip(avg_corr * 2, 0.0, 1.0))


def compute_novelty_score(state: np.ndarray, history: list) -> float:
    """
    Compute novelty score from state history.
    
    Uses k-NN distance to measure how novel this state is.
    """
    if len(history) < 5:
        return 1.0
    
    # Compute distances to recent history
    recent = np.array(history[-100:])
    
    dists = np.linalg.norm(recent - state, axis=1)
    
    # Average distance to k nearest
    k = min(5, len(recent))
    k_nearest = np.partition(dists, k-1)[:k]
    avg_dist = np.mean(k_nearest)
    
    # Normalize (rough heuristic)
    return float(np.clip(avg_dist / 10, 0.0, 1.0))


def compute_creativity_scores(state: np.ndarray, history: list) -> dict:
    """Compute creativity-related metrics."""
    if len(history) < 10:
        return {'surprise': 0.5, 'usefulness': 0.5, 'coherence': 0.5}
    
    recent = np.array(history[-20:])
    
    # Surprise: distance from recent mean
    recent_mean = np.mean(recent, axis=0)
    surprise = float(np.linalg.norm(state - recent_mean) / 10)
    
    # Coherence: variance in recent states (lower = more coherent)
    variance = np.var(recent, axis=0).mean()
    coherence = float(1.0 / (1.0 + variance))
    
    # Usefulness: arbitrary baseline
    usefulness = 0.5
    
    return {
        'surprise': np.clip(surprise, 0, 1),
        'usefulness': usefulness,
        'coherence': np.clip(coherence, 0, 1)
    }


# ============================================================================
# STARFIRE BRIDGE
# ============================================================================

class StarfireBridge:
    """
    Bridge to Starfire for symbolic reasoning.
    
    This class:
    1. Encodes QuaNot reservoir states for Starfire
    2. Sends messages to Starfire via IPC
    3. Decodes Starfire responses for QuaNot feedback
    
    Parameters
    ----------
    reservoir : ChaoticReservoir
        The QuaNot reservoir
    starfire_path : str
        Path to Starfire executable or directory
    mode : str
        IPC mechanism: 'stdio' | 'pipe' | 'socket'
    """
    
    def __init__(
        self,
        reservoir: ChaoticReservoir,
        starfire_path: str,
        mode: str = 'stdio',
        port: int = 7890
    ):
        self.reservoir = reservoir
        self.oscillator = CreativeOscillator()
        self.starfire_path = starfire_path
        self.mode = mode
        self.port = port
        
        # State history for metrics
        self.state_history = []
        self.max_history = 1000
        
        # Consciousness tracking
        self.psi_history = []
        
        # Output cache
        self.last_response = None
        
        # Check Starfire availability
        self.available = self._check_starfire()
    
    def _check_starfire(self) -> bool:
        """Check if Starfire is available."""
        if not os.path.exists(self.starfire_path):
            return False
        
        # Try to find starfire binary
        starfire_bin = os.path.join(self.starfire_path, 'target', 'release', 'starfire')
        if os.path.exists(starfire_bin):
            self.starfire_binary = starfire_bin
            return True
        
        # Check for cargo.toml (development)
        cargo_toml = os.path.join(self.starfire_path, 'Cargo.toml')
        if os.path.exists(cargo_toml):
            # Need to build first
            return False
        
        return False
    
    def encode_for_starfire(
        self,
        input_text: str,
        include_state: bool = True
    ) -> BridgeMessage:
        """
        Encode input and reservoir state for Starfire.
        
        Parameters
        ----------
        input_text : str
            Text input from user
        include_state : bool
            Include reservoir state in message
            
        Returns
        -------
        BridgeMessage
            Message ready for Starfire
        """
        # Get current reservoir state
        state = self.reservoir.get_state()
        
        # Store in history
        self.state_history.append(state.copy())
        if len(self.state_history) > self.max_history:
            self.state_history.pop(0)
        
        # Compute metrics
        psi = compute_consciousness_proxy(self.state_history)
        self.psi_history.append(psi)
        
        novelty = compute_novelty_score(state, self.state_history)
        
        creativity = compute_creativity_scores(state, self.state_history)
        
        # Compute divergence metric for oscillator
        divergence = self._compute_divergence()
        
        return BridgeMessage(
            message_id=str(uuid.uuid4()),
            timestamp=time.time(),
            input_text=input_text,
            reservoir_state=state.tolist() if include_state else None,
            consciousness_proxy=psi,
            novelty_score=novelty,
            creativity_scores=creativity,
            divergence_metric=divergence,
            request_mode='query'
        )
    
    def _compute_divergence(self) -> float:
        """Compute divergence metric for creative oscillation."""
        if len(self.state_history) < 10:
            return 0.0
        
        # Compare recent state variance to baseline
        recent = self.state_history[-10:]
        baseline = self.state_history[:-10] if len(self.state_history) > 20 else self.state_history[:-1]
        
        if len(baseline) < 2:
            return 0.0
        
        # Average distance from baseline centroid
        centroid = np.mean(baseline, axis=0)
        recent_mean = np.mean(recent, axis=0)
        
        divergence = float(np.linalg.norm(recent_mean - centroid))
        
        # Normalize to 0-1
        return float(np.clip(divergence / 10.0, 0.0, 1.0))
    
    def decode_from_starfire(
        self,
        message: BridgeMessage
    ) -> Dict[str, Any]:
        """
        Interpret Starfire response for QuaNot feedback.
        
        Returns modulation signals for reservoir.
        """
        if not message.reasoning_result:
            return {
                'reasoning': '',
                'confidence': 0.5,
                'curiosity': 0.0,
                'surprise': 0.0,
                'oscillation_modulation': 0.0,
                'psi_feedback': 0.0,
            }
        
        # Extract reasoning result
        reasoning = message.reasoning_result
        
        # Extract confidence to modulate oscillation
        confidence = message.confidence or 0.5
        
        # Curiosity drives exploration
        curiosity = message.curiosity or 0.0
        
        # Surprise indicates novel input
        surprise = message.surprise or 0.0
        
        # Compute modulation
        modulation = self._compute_modulation(confidence, curiosity, surprise)
        
        # Update consciousness based on reasoning
        psi_feedback = self._compute_psi_feedback(
            message.consciousness_proxy or 0.5,
            confidence,
            curiosity
        )
        
        return {
            'reasoning': reasoning,
            'confidence': confidence,
            'curiosity': curiosity,
            'surprise': surprise,
            'oscillation_modulation': modulation,
            'psi_feedback': psi_feedback,
            'reasoning_chain': message.reasoning_chain or [],
        }
    
    def _compute_modulation(
        self,
        confidence: float,
        curiosity: float,
        surprise: float
    ) -> float:
        """
        Compute creative oscillation modulation from Starfire.
        
        High curiosity → increase exploration (chaos)
        High confidence → increase exploitation (order)
        High surprise → trigger re-evaluation
        """
        modulation = curiosity - confidence
        modulation += surprise * 0.5
        
        return float(np.clip(modulation, -1.0, 1.0))
    
    def _compute_psi_feedback(
        self,
        current_psi: float,
        confidence: float,
        curiosity: float
    ) -> float:
        """
        Compute consciousness feedback from reasoning quality.
        """
        psi = current_psi
        psi += confidence * 0.1
        psi += curiosity * 0.1
        
        return float(np.clip(psi, 0.0, 1.0))
    
    def apply_feedback(
        self,
        feedback: Dict[str, Any]
    ) -> None:
        """Apply Starfire feedback to QuaNot components."""
        modulation = feedback['oscillation_modulation']
        
        self.oscillator.step(
            current_value=feedback['confidence'],
            divergence_metric=abs(modulation)
        )
        
        self.last_response = feedback['reasoning']
    
    def process(
        self,
        input_text: str,
        include_state: bool = True
    ) -> Tuple[str, Dict[str, Any]]:
        """
        Process input through QuaNot → Starfire pipeline.
        """
        message = self.encode_for_starfire(input_text, include_state)
        
        if self.available:
            starfire_response = self._send_to_starfire(message)
            feedback = self.decode_from_starfire(starfire_response)
        else:
            feedback = self._local_processing(message)
        
        self.apply_feedback(feedback)
        
        return feedback['reasoning'], feedback
    
    def _send_to_starfire(self, message: BridgeMessage) -> BridgeMessage:
        """Send message to Starfire via IPC."""
        if self.mode == 'stdio':
            return self._stdio_ipc(message)
        elif self.mode == 'socket':
            return self._socket_ipc(message)
        else:
            raise ValueError(f"Unknown IPC mode: {self.mode}")
    
    def _stdio_ipc(self, message: BridgeMessage) -> BridgeMessage:
        """STDIO-based IPC to Starfire."""
        try:
            proc = subprocess.Popen(
                [self.starfire_binary, '--interactive'],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            
            stdout, stderr = proc.communicate(
                input=message.to_json() + '\n',
                timeout=30
            )
            
            if stdout:
                return BridgeMessage.from_json(stdout)
            else:
                raise RuntimeError(f"Starfire returned empty: {stderr}")
                
        except FileNotFoundError:
            raise RuntimeError("Starfire binary not found")
    
    def _socket_ipc(self, message: BridgeMessage) -> BridgeMessage:
        """TCP socket IPC to Starfire."""
        import socket
        
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.connect(('localhost', self.port))
            
            sock.sendall((message.to_json() + '\n').encode())
            
            response = b''
            while b'\n' not in response:
                chunk = sock.recv(4096)
                if not chunk:
                    break
                response += chunk
            
            sock.close()
            
            if response:
                return BridgeMessage.from_json(response.decode())
            else:
                raise RuntimeError("Starfire returned empty")
                
        except ConnectionRefusedError:
            raise RuntimeError("Starfire not running on port")
    
    def _local_processing(self, message: BridgeMessage) -> Dict[str, Any]:
        """Fallback local processing when Starfire unavailable."""
        psi = message.consciousness_proxy or 0.5
        novelty = message.novelty_score or 0.5
        
        reasoning = f"[QuaNot local] Processed with ψ={psi:.2f}, novelty={novelty:.2f}"
        
        return {
            'reasoning': reasoning,
            'confidence': psi,
            'curiosity': novelty,
            'surprise': 0.0,
            'oscillation_modulation': novelty - psi,
            'psi_feedback': psi,
            'reasoning_chain': [],
        }
    
    def get_status(self) -> Dict[str, Any]:
        """Get bridge status."""
        return {
            'starfire_available': self.available,
            'mode': self.mode,
            'last_psi': self.psi_history[-1] if self.psi_history else 0.0,
            'oscillator_state': self.oscillator.get_status(),
            'last_response': self.last_response,
        }


# ============================================================================
# CONVENIENCE FUNCTIONS
# ============================================================================

def create_bridge(
    reservoir: ChaoticReservoir,
    starfire_path: str = '../starfire',
    mode: str = 'stdio'
) -> StarfireBridge:
    """Create a Starfire bridge."""
    return StarfireBridge(reservoir, starfire_path, mode)